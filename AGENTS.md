# Scribe – Agent & Developer Guide

This file is the single source of truth for AI coding agents (Claude Code,
Codex, etc.) and human contributors working on this repository.
`CLAUDE.md` is a symlink to this file.

---

## 1. Project intent

**Scribe** is a Tauri 2 desktop app (macOS + Linux) that automatically records
meetings in the background, transcribes them locally with **NVIDIA Parakeet v3**,
summarises them with a local LLM, and stores only the transcript + summary in a
local SQLite database. **Audio is never written to disk.**

Key design principles:
- Privacy-first: all processing is local; no cloud APIs are called.
- Single process: one long-running Tauri process handles the tray, window, and
  recording engine. No separate daemon.
- Graceful degradation: if ASR or summarization is unavailable the app still
  runs; the UI explains what is missing.
- Audio never persists: WAV bytes live in memory only for the duration of one
  ASR segment call.

---

## 2. Repository layout

```
scribe-tauri/
├── src/                          # React/TypeScript frontend
│   ├── components/
│   │   ├── RecordingDetail.tsx   # Editable title / transcript / summary
│   │   ├── RecordingIndicator.tsx# Live recording status bar + Start/Stop
│   │   ├── RecordingsList.tsx    # Date-sorted list of past recordings
│   │   └── SettingsPanel.tsx     # All user-configurable options
│   ├── hooks/
│   │   └── useRecorder.ts        # Tauri event → React state bridge
│   ├── types/index.ts            # TypeScript mirrors of Rust structs
│   ├── test/                     # Vitest test suite
│   │   ├── setup.ts              # Global mocks (@tauri-apps/api/*)
│   │   ├── mocks.ts              # Shared test fixtures
│   │   ├── RecordingIndicator.test.tsx
│   │   ├── RecordingsList.test.tsx
│   │   ├── RecordingDetail.test.tsx
│   │   ├── SettingsPanel.test.tsx
│   │   └── useRecorder.test.tsx
│   └── App.tsx                   # Root component + sidebar navigation
│
├── src-tauri/
│   └── src/
│       ├── audio/
│       │   ├── activity.rs       # RMS-based activity detection state machine
│       │   ├── capture.rs        # CPAL mic + system-audio capture; rms/to_dbfs helpers
│       │   ├── mod.rs            # AudioEngine glue; encode_wav helper
│       │   └── segmentation.rs   # Timer + dominant-input-switch segmenter
│       ├── asr.rs                # ASR backends: Parakeet (primary), WhisperCli, HttpServer
│       ├── commands.rs           # All Tauri IPC commands
│       ├── db.rs                 # SQLite CRUD (rusqlite, bundled)
│       ├── lib.rs                # App setup, AppState, system tray
│       ├── recorder.rs           # Background Tokio task: full pipeline
│       ├── settings.rs           # JSON-persisted Settings struct
│       └── summarization.rs      # Summarization backends: Ollama, OpenAI-compat
│
├── scripts/
│   └── parakeet_server.py        # FastAPI HTTP wrapper for NVIDIA Parakeet v3
│
├── vitest.config.ts              # Frontend test config (jsdom, coverage)
├── AGENTS.md                     # This file (also CLAUDE.md via symlink)
└── tauri.conf.json               # App metadata, window, tray, bundle config
```

---

## 3. Architecture

### 3.1 Audio pipeline

```
Mic (CPAL)  ──┐
               ├─► ActivityDetector ─► RecorderCommand channel
Sys (CPAL)  ──┘        │
                        │  (when Active)
                        ▼
                    Segmenter ──► encode_wav ──► ASR (Parakeet HTTP)
                                                      │
                                              running transcript
                                                      │
                                  (on stop)   summarize (Ollama)
                                                      │
                                                   SQLite DB
```

- **ActivityDetector**: two-stream RMS threshold state machine.
  `Idle → Starting{count} → Active → Stopping{count} → Idle`.
  Both mic AND system audio must be above threshold for `min_active_windows`
  consecutive 1-second windows to trigger a recording.
- **Segmenter**: flushes audio to ASR on a timer *or* when the dominant input
  source switches (mic ↔ system). Each segment is a fresh WAV byte slice; no
  audio is retained between segments.
- **ASR**: `transcribe_http` POSTs WAV bytes to a local HTTP server and reads
  `{"text": "…"}`. `Parakeet` is the recommended backend; `WhisperCli` is the
  CPU fallback.
- **Summarization**: Ollama `/api/generate` (primary) or any OpenAI-compatible
  `/chat/completions` endpoint.

### 3.2 State shared between threads

`AppState` is managed by Tauri and passed to every command handler:

```rust
pub struct AppState {
    pub db:            Arc<Mutex<Connection>>,   // SQLite (parking_lot)
    pub cmd_tx:        Arc<Mutex<mpsc::Sender<RecorderCommand>>>,
    pub settings:      Arc<Mutex<Settings>>,
    pub settings_path: Arc<Mutex<Option<PathBuf>>>,
}
```

The background recorder task owns the `Receiver` and processes one `RecorderCommand` at a time.

### 3.3 Tauri events (backend → frontend)

| Event              | Payload                | When emitted                         |
|--------------------|------------------------|--------------------------------------|
| `recorder-status`  | `RecorderStatus`       | Every analysis tick (1 s)            |
| `recording-started`| `String` (id)          | New recording row inserted           |
| `recording-stopped`| `String` (id)          | Recording finalised in DB            |
| `settings-changed` | `Settings`             | Auto-record toggled from tray        |

### 3.4 Platform differences

| Concern            | macOS 13+               | macOS 10.15–12         | Linux                        |
|--------------------|-------------------------|------------------------|------------------------------|
| System audio       | ScreenCaptureKit        | BlackHole + aggregate  | PulseAudio/PipeWire monitor  |
| System audio impl  | `open_system_audio()`   | `open_system_audio()`  | `open_system_audio()`        |
| Detected by        | Device name heuristic   | Device name heuristic  | `*.monitor` suffix           |
| Entitlements       | `audio-input` + screen  | `audio-input`          | N/A                          |

---

## 4. ASR: NVIDIA Parakeet v3

Parakeet is the **primary** ASR backend. It delivers better accuracy and
lower latency than Whisper on NVIDIA GPUs and Apple Silicon.

### 4.1 Starting the server

```bash
pip install "nemo_toolkit[asr]" fastapi uvicorn
python scripts/parakeet_server.py --model parakeet-tdt-0.6b-v2 --port 9000
```

The server exposes:
- `POST /transcribe` – WAV body → `{"text": "…"}`
- `GET  /health`     – `{"status": "ok", "model": "…"}`

### 4.2 Supported models

| ID                        | Size  | Notes                            |
|---------------------------|-------|----------------------------------|
| `parakeet-tdt-0.6b-v2`    | 0.6 B | **Default.** Fast, accurate.     |
| `parakeet-tdt-1.1b`       | 1.1 B | Higher accuracy, more VRAM.      |
| `parakeet-ctc-0.6b`       | 0.6 B | CTC variant (no timestamp info). |

### 4.3 Fallback to Whisper

If Parakeet is not available, switch the ASR backend in Settings to
**Whisper CLI** and set the binary path. Install with:

```bash
pip install openai-whisper   # CPU
# or
brew install whisper          # macOS
```

---

## 5. Data model

### 5.1 SQLite schema

```sql
CREATE TABLE recordings (
    id          TEXT PRIMARY KEY NOT NULL,   -- UUID v4
    title       TEXT NOT NULL,               -- Editable; auto-set to "Meeting YYYY-MM-DD HH:MM"
    started_at  TEXT NOT NULL,               -- ISO-8601
    ended_at    TEXT,                        -- NULL while recording; set on stop
    transcript  TEXT,                        -- Full running transcript (flat text)
    summary     TEXT,                        -- LLM summary; NULL if unavailable
    created_at  TEXT NOT NULL,
    updated_at  TEXT NOT NULL
);
```

- One row per recording session. No segments table.
- Audio is never stored. Only transcript + summary are persisted.
- All timestamps are RFC-3339 UTC strings.

### 5.2 Settings (JSON)

Stored in `<app-data-dir>/settings.json`. Falls back to compiled defaults if
missing or unparseable.

```jsonc
{
  "auto_record": true,
  "idle_timeout_secs": 180,
  "segment_interval_secs": 60,
  "activity_threshold_dbfs": -40.0,
  "min_active_windows": 3,
  "asr_backend": {
    "Parakeet": {
      "url": "http://127.0.0.1:9000",
      "model": "parakeet-tdt-0.6b-v2"
    }
  },
  "summarization_backend": {
    "Ollama": {
      "base_url": "http://localhost:11434",
      "model": "llama3.2:3b"
    }
  }
}
```

---

## 6. Running tests

### 6.1 Frontend (Vitest)

```bash
npm test              # run all tests once
npm run test:coverage # run with coverage report
```

Coverage targets: `src/**/*.{ts,tsx}`, excluding `test/`, `main.tsx`,
`vite-env.d.ts`. Target ≥ 90 % line coverage.

**Mocking strategy**: `@tauri-apps/api/core` (invoke) and
`@tauri-apps/api/event` (listen/emit) are auto-mocked in
`src/test/setup.ts`. Every test that needs specific `invoke` responses
calls `(invoke as Mock).mockResolvedValue(...)` in `beforeEach`.

### 6.2 Rust unit tests

```bash
cd src-tauri
cargo test          # runs all #[cfg(test)] modules
```

The Rust tests target only **pure-logic modules** that have no system-library
dependencies (GTK / webkit2gtk / ALSA). Modules with tests:

| Module                    | Coverage focus                                  |
|---------------------------|-------------------------------------------------|
| `audio/capture.rs`        | `rms()`, `to_dbfs()`, `drain()`                 |
| `audio/activity.rs`       | Full state machine: Idle/Starting/Active/Stopping|
| `audio/segmentation.rs`   | Timer flush, source-switch flush, `reset()`     |
| `db.rs`                   | All CRUD with in-memory SQLite                  |
| `settings.rs`             | `load`/`save` round-trips, corrupt-file fallback|
| `asr.rs`                  | Serialisation, None backend, server-down paths  |
| `summarization.rs`        | Serialisation, None/empty, server-down paths    |

**Important**: `cargo test` in the full Tauri project requires system packages
(`libwebkit2gtk-4.1-dev`, `libasound2-dev`, etc.). In CI environments without
these, run tests against a stripped workspace that excludes the Tauri/CPAL
build targets (see `.github/workflows/` if added).

---

## 7. Development setup

### 7.1 Prerequisites

**macOS**
```bash
xcode-select --install
rustup update stable
brew install node
# For Tauri:
# No extra packages needed on macOS 13+
```

**Linux (Ubuntu / Debian)**
```bash
sudo apt install \
  libwebkit2gtk-4.1-dev \
  libgtk-3-dev \
  libayatana-appindicator3-dev \
  librsvg2-dev \
  libasound2-dev \
  pkg-config
rustup update stable
```

### 7.2 Running in dev mode

```bash
npm install
# In a separate terminal: start the Parakeet server (optional)
python scripts/parakeet_server.py
# Then:
npm run tauri dev
```

### 7.3 Building for production

```bash
npm run tauri build
```

---

## 8. Coding best practices

### 8.1 Rust backend

- **Avoid `unwrap()` in production paths.** Use `?` with `anyhow::Result` or
  return `Option<T>`. Log errors with `log::warn!` before returning `None`.
- **Never write audio to disk** outside of a temporary file needed for a single
  ASR call. Delete the temp file immediately after the subprocess reads it.
- **Keep `AppState` fields behind `Arc<Mutex<T>>`** (using `parking_lot::Mutex`
  for deadlock-free performance). Acquire locks for the shortest possible scope.
- **All DB operations are synchronous** (rusqlite is sync). Run them on the
  Tokio blocking pool if needed to avoid blocking the async runtime:
  `tokio::task::spawn_blocking(|| ...)`.
- **Tauri commands must be `async`** to allow `await` within handlers.
- **Emit events after state changes** so the frontend stays in sync:
  `app.emit("recorder-status", status)`.
- **Add `#[cfg(test)]` modules** directly in the source file (not in
  a separate file). Use in-memory SQLite (`Connection::open_in_memory()`)
  for all DB tests.

### 8.2 React / TypeScript frontend

- **Mock Tauri APIs centrally** in `src/test/setup.ts`. Do not duplicate mocks
  in individual test files.
- **Use `waitFor` or `findBy*` queries** for async state changes after `invoke`
  calls – never query synchronously after a mock that returns a promise.
- **Keep components controlled.** State lives in `useState`; parent receives
  updates via `onUpdated` / `onDeleted` callbacks. No global state store needed
  at this scale.
- **Type `AsrBackend` and `SummarizationBackend`** as tagged-union types
  mirroring the Rust enums. Add new variants to `src/types/index.ts` whenever
  a Rust enum variant is added.
- **CSS class naming** follows BEM (`block__element--modifier`). Add new styles
  to `App.css`; do not use inline styles.
- **Do not use `any` casts** for backend data that has a corresponding
  TypeScript type. Only use `(x as any).Field` for enum-variant field access
  where TypeScript's narrowing is insufficient, and keep those in the component
  files that already use the pattern.

### 8.3 Testing guidelines

- Target **≥ 90 % line coverage** across both Rust logic modules and the
  React frontend.
- **Every new Tauri command** should have at least one corresponding frontend
  test that verifies `invoke` is called with the right arguments.
- **Every new state-machine transition** in `activity.rs` or `segmentation.rs`
  should have a dedicated unit test.
- **Async Rust tests** use `#[tokio::test]`. Ensure the tokio runtime features
  (`features = ["full"]`) are enabled in `Cargo.toml`.
- **Do not test UI styling** (exact pixel positions, colours). Test behaviour:
  visibility, click handlers, text content.

### 8.4 Adding a new ASR backend

1. Add a new variant to `AsrBackend` in `src-tauri/src/asr.rs`.
2. Add a `pub(crate)` transcription function in the same file.
3. Add the variant to the `match` in `transcribe()`.
4. Add serialisation round-trip + error-path tests in `asr.rs`.
5. Add the variant to `AsrBackend` in `src/types/index.ts`.
6. Update `asrBackendType()` and the JSX in `SettingsPanel.tsx`.
7. Update `src/test/mocks.ts` if the default backend changes.

### 8.5 Git workflow

- Branch naming: `claude/<short-description>-<session-id>` for AI-generated
  branches; `feat/<description>` / `fix/<description>` for human branches.
- Commit messages: imperative mood, ≤ 72 chars subject, blank line before body.
- **Never force-push to `main` or `master`.**
- Run `npm test` and verify `cargo check` before opening a PR (full
  `cargo test` requires system libs).

---

## 9. Known limitations and future work

- **macOS 10.15–12**: system audio requires BlackHole. The app warns if no
  loopback device is found but still starts. Document the setup in the UI.
- **Multiple output devices on Linux**: the user may need to manually select the
  correct monitor sink in Settings if the heuristic picks the wrong device.
- **Parakeet GPU detection**: the server script loads the model on CPU if no
  CUDA/ROCm GPU is available. Performance may be slow; fall back to WhisperCli
  in that case.
- **Large transcripts**: very long meetings produce large SQLite TEXT blobs.
  Consider chunking if transcripts exceed ~100 KB.
- **No export**: transcripts and summaries can only be viewed in the GUI. A
  future "Export as Markdown/PDF" feature is tracked but not yet implemented.

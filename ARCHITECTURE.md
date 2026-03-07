# Architecture

> Keep this file up to date as the system evolves. Update it in the same PR
> as any module addition, removal, or significant responsibility change.
> Also keep `README.md` and `AGENTS.md` in sync with user-facing and
> agent-facing changes respectively.

---

## System overview

```
┌─────────────────────────────────────────────────────────────┐
│                        Tauri Process                         │
│                                                             │
│  ┌──────────────┐          ┌──────────────────────────────┐ │
│  │  WebView UI  │◄─events──│      AppState                │ │
│  │  (React/TS)  │──invoke──►  db: Arc<Mutex<Connection>>  │ │
│  └──────────────┘          │  cmd_tx: mpsc::Sender        │ │
│                            │  settings: Arc<Mutex<...>>   │ │
│                            └──────────────┬───────────────┘ │
│                                           │ RecorderCommand  │
│                                           ▼                  │
│                            ┌──────────────────────────────┐ │
│                            │   Background Recorder Task   │ │
│                            │   (Tokio async task)         │ │
│                            │                              │ │
│  CPAL Mic  ──────────────► │  ActivityDetector            │ │
│  CPAL SysAudio ──────────► │       │                      │ │
│                            │  Segmenter ──► encode_wav    │ │
│                            └──────────────────┬───────────┘ │
└──────────────────────────────────────────────┼─────────────┘
                                               │ WAV bytes
                      ┌────────────────────────┼──────────────┐
                      │  Local services        │              │
                      │                        ▼              │
                      │  Parakeet server ◄─ POST /transcribe  │
                      │  (scripts/parakeet_server.py)         │
                      │                        │              │
                      │                    transcript          │
                      │                        │              │
                      │  Ollama ◄──────── /api/generate       │
                      │                        │              │
                      │                    summary             │
                      └────────────────────────┼──────────────┘
                                               │
                                               ▼
                                         SQLite DB
                                    (recordings table)
```

---

## Module map

### Rust backend (`src-tauri/src/`)

| Module | Responsibility |
|---|---|
| `lib.rs` | App bootstrap, `AppState`, system tray setup, plugin registration |
| `commands.rs` | All Tauri IPC command handlers (async fn with `#[tauri::command]`) |
| `recorder.rs` | Background Tokio task: orchestrates audio → ASR → DB pipeline |
| `audio/mod.rs` | `AudioEngine` glue, `encode_wav` helper (hound, cursor-backed) |
| `audio/capture.rs` | CPAL device open, `drain()`, `rms()`, `to_dbfs()` |
| `audio/activity.rs` | RMS state machine: `Idle→Starting→Active→Stopping→Idle` |
| `audio/segmentation.rs` | Flush on timer or dominant-source switch |
| `asr.rs` | `AsrBackend` enum, `transcribe()`, HTTP + Whisper CLI impls |
| `db.rs` | SQLite CRUD: insert, finalize, list, get, update, delete |
| `settings.rs` | `Settings` struct, JSON load/save with fallback to defaults |
| `summarization.rs` | `SummarizationBackend` enum, Ollama + OpenAI-compat impls |

### React frontend (`src/`)

| Path | Responsibility |
|---|---|
| `App.tsx` | Root layout, sidebar nav, recorder state wiring |
| `hooks/useRecorder.ts` | Subscribes to `recorder-status` event; exposes `startRecording`/`stopRecording` |
| `components/RecordingIndicator.tsx` | Status bar (Idle / Recording… + preview) + Start/Stop buttons |
| `components/RecordingsList.tsx` | Sidebar: date-sorted list, re-fetches on `recording-stopped` event |
| `components/RecordingDetail.tsx` | Editable title, transcript, summary; Save, Delete, Regenerate actions |
| `components/SettingsPanel.tsx` | All user-configurable settings; ASR and summarization backend pickers |
| `types/index.ts` | TypeScript mirrors of all Rust public structs and enums |

### Scripts

| Path | Responsibility |
|---|---|
| `scripts/parakeet_server.py` | FastAPI + NeMo server exposing `POST /transcribe` and `GET /health` |

---

## Data flow: recording lifecycle

```
1. ActivityDetector fires Some(true)
      └─► recorder.rs inserts DB row, emits "recording-started"

2. Segmenter fires flush
      └─► encode_wav(mic_buf + sys_buf)
      └─► asr::transcribe(wav_bytes) → Option<String>
      └─► DB: append to transcript

3. ActivityDetector fires Some(false)   OR   stop_recording() command
      └─► Final ASR segment
      └─► summarization::summarize(transcript) → Option<String>
      └─► DB: finalize_recording (ended_at, transcript, summary)
      └─► emits "recording-stopped"
```

---

## Shared state

```rust
pub struct AppState {
    pub db:            Arc<Mutex<Connection>>,       // parking_lot
    pub cmd_tx:        Arc<Mutex<mpsc::Sender<RecorderCommand>>>,
    pub settings:      Arc<Mutex<Settings>>,
    pub settings_path: Arc<Mutex<Option<PathBuf>>>,
}
```

Lock discipline: acquire for the shortest possible scope. Never hold `db`
lock while awaiting an async call.

---

## Platform notes

| Concern | macOS 13+ | macOS 10.15–12 | Linux |
|---|---|---|---|
| System audio capture | ScreenCaptureKit | BlackHole + aggregate device | PulseAudio/PipeWire `*.monitor` |
| Device heuristic | name contains "monitor"/"loopback"/"blackhole"/"soundflower" | same | same + `.monitor` suffix |
| Build deps | Xcode CLI tools only | same | `libwebkit2gtk-4.1-dev`, `libasound2-dev`, etc. |

---

## Dependency choices

| Crate / Package | Why |
|---|---|
| `tauri 2` | Cross-platform desktop shell; single process |
| `cpal` | Cross-platform audio capture; WASAPI/CoreAudio/ALSA |
| `rusqlite` (bundled) | Zero-install SQLite; sync API; WAL mode |
| `hound` | Pure-Rust WAV encode/decode; cursor-backed in-memory encode |
| `reqwest` | Async HTTP client for ASR + LLM calls |
| `parking_lot` | Faster, deadlock-friendly `Mutex` vs `std::sync::Mutex` |
| `tokio` (full) | Async runtime; `spawn_blocking` for sync DB calls |
| `uuid` v4 | Random IDs for recordings |
| `chrono` | UTC timestamps in RFC-3339 format |
| `vitest` | Fast Vite-native test runner; jsdom env for React |
| `@testing-library/react` | Behaviour-focused component testing |

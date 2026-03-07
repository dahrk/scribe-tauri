# Scribe

**Scribe** is a privacy-first meeting recorder for macOS and Linux. It runs in
the background, detects when you're in a call, transcribes everything locally
with NVIDIA Parakeet v3, and summarises the meeting with a local LLM. Your
audio never leaves your machine.

---

## Features

- **Automatic detection** – starts recording when both your mic and system
  audio are active for several consecutive seconds; stops after a configurable
  period of silence.
- **Local transcription** – NVIDIA Parakeet v3 (TDT) via a local Python
  server. Whisper CLI is available as a CPU fallback.
- **Local summarization** – Ollama (default: `llama3.2:3b`) or any
  OpenAI-compatible endpoint running on your machine.
- **Privacy by design** – audio is never written to disk and never sent to
  any remote server.
- **Editable records** – browse, search, and edit past meeting titles,
  transcripts, and summaries.
- **System tray** – lives quietly in your tray. Toggle auto-record or open
  the window from the tray icon.

---

## Screenshots

> _Add screenshots here once the UI is finalised._

---

## Getting Started

### Prerequisites

**macOS**
```bash
xcode-select --install
rustup update stable
brew install node
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
# Install Node.js via nvm or your package manager
```

### System audio (Linux)

Scribe captures system audio via the PulseAudio/PipeWire monitor of your
default output sink. No extra setup is required on most Linux distributions –
if the device isn't detected automatically, select it manually in Settings.

### System audio (macOS)

- **macOS 13+**: ScreenCaptureKit is used automatically. Grant screen-capture
  permission when prompted.
- **macOS 10.15–12**: Install [BlackHole](https://existential.audio/blackhole/)
  and create an aggregate device that combines your normal output + BlackHole.
  Select the aggregate device as your system output.

---

## Installation

### From source (development)

```bash
git clone <repo-url>
cd scribe-tauri
bun install
bun run tauri dev
```

### Production build

```bash
bun run tauri build
# Output: src-tauri/target/release/bundle/
```

---

## Setting up Parakeet v3 (recommended ASR)

Parakeet is the primary transcription backend. It provides significantly
better accuracy than Whisper at low latency on NVIDIA and Apple Silicon.

**1. Install dependencies**
```bash
pip install "nemo_toolkit[asr]" fastapi uvicorn
```

**2. Start the server**
```bash
python scripts/parakeet_server.py --model parakeet-tdt-0.6b-v2 --port 9000
```

The server runs at `http://127.0.0.1:9000` by default. Keep it running
in the background while using Scribe.

**Supported models**

| Model | Size | Notes |
|---|---|---|
| `parakeet-tdt-0.6b-v2` | 0.6 B | Default. Fast and accurate. |
| `parakeet-tdt-1.1b` | 1.1 B | Higher accuracy, needs more VRAM. |
| `parakeet-ctc-0.6b` | 0.6 B | CTC variant; no timestamps. |

> **No GPU?** Parakeet will run on CPU but may be slow. Switch to
> **Whisper CLI** in Settings for a lighter CPU option.

---

## Setting up Whisper CLI (fallback ASR)

```bash
pip install openai-whisper   # CPU
# or:
brew install whisper          # macOS
```

In Settings → Transcription, select **Whisper CLI**, enter the binary path
(e.g. `/usr/local/bin/whisper`), and choose a model size.

---

## Setting up Ollama (summarization)

```bash
# Install Ollama: https://ollama.com
ollama pull llama3.2:3b
```

Scribe points at `http://localhost:11434` by default. Change the model or
URL in Settings → Summarization.

---

## Configuration

All settings are stored in `<app-data-dir>/settings.json` and exposed in
the Settings panel.

| Setting | Default | Description |
|---|---|---|
| Auto-record | On | Start/stop recording automatically based on audio activity |
| Idle timeout | 180 s | Silence duration before recording stops |
| Transcription interval | 60 s | How often audio is sent to the ASR server |
| Activity threshold | −40 dBFS | Minimum audio level to be considered "active" |
| Min active windows | 3 | Consecutive active windows required to start a recording |
| ASR backend | Parakeet | See ASR setup sections above |
| Summarization backend | Ollama | See Ollama setup above |

---

## Usage

1. Start the Parakeet server (or configure Whisper in Settings).
2. Launch Scribe – it appears in your system tray.
3. Join a call. Scribe detects activity and starts recording automatically.
4. The status bar shows **Recording…** with a live transcript preview.
5. After your call ends, Scribe finalises the transcript and generates a summary.
6. Open the Scribe window to browse recordings, read summaries, and edit titles.

### Manual recording

Click **Start** in the status bar to force-start a recording without audio
activity detection. Click **Stop** to end it.

---

## Privacy

- **Audio never persists.** Raw audio samples live in RAM only for the
  duration of one transcription segment (~60 s). No WAV files are saved.
- **No network calls.** All ASR and LLM requests go to `127.0.0.1` by default.
  If you change these URLs to remote endpoints, your transcripts will be sent
  to those servers.
- **Local database.** Recordings are stored in a SQLite file in your
  app data directory. You own and control the file.

---

## Development

### Run tests

```bash
bun run test              # frontend (Vitest, 48 tests)
bun run test:coverage # with coverage report

cd src-tauri
cargo test            # Rust unit tests (72 tests, needs system libs)
```

### Repository structure

```
scribe-tauri/
├── src/                  # React/TypeScript frontend
├── src-tauri/            # Rust backend (Tauri)
├── scripts/              # Python helper scripts
├── docs/                 # Deep-dive documentation
├── AGENTS.md             # Agent/developer map (table of contents)
├── ARCHITECTURE.md       # System architecture
└── README.md             # This file
```

For contributors and AI coding agents, start with [AGENTS.md](AGENTS.md).

---

## Known limitations

- **English only** – Parakeet TDT is an English-only model. Multilingual
  support would require a different model.
- **No audio export** – transcripts and summaries are text only; audio is
  never stored.
- **macOS 10.15–12** – requires manual BlackHole setup for system audio.
- **Multiple audio devices** – manual device selection is not yet available
  in the UI (see [docs/product-specs/multi-device.md](docs/product-specs/multi-device.md)).

---

## Roadmap

See [docs/PLANS.md](docs/PLANS.md) for the full roadmap.

Upcoming: export as Markdown/PDF, manual device selection, pagination for
large recording libraries.

---

## Contributing

1. Read [AGENTS.md](AGENTS.md) – it links to design docs, coding conventions,
   and the ASR backend checklist.
2. Check [docs/exec-plans/tech-debt-tracker.md](docs/exec-plans/tech-debt-tracker.md)
   for good first issues.
3. Run `npm test` and `cargo check` before opening a PR.
4. Follow the branch naming convention: `feat/<description>` or `fix/<description>`.

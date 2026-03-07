# Design Principles

These principles govern every implementation decision. When in doubt, refer back here.

## 1. Privacy above all

Audio is never written to disk and never leaves the machine. Only text
(transcript + summary) is stored. No telemetry, no cloud APIs.

**In practice:** Every code path that handles raw audio bytes must be reviewed
to confirm they are not written to a persistent location. The only exception is
`WhisperCli` which requires a temp file – deleted synchronously after the
subprocess exits.

## 2. Single long-running process

One Tauri process owns the tray, window, recorder engine, and DB connection.
No separate daemons, no background services. Simplifies installation,
uninstallation, and debugging.

## 3. Graceful degradation

If any optional component is absent (ASR server down, Ollama not installed,
no system audio device), the app continues operating with a reduced feature
set and explains clearly in the UI what is missing.

**Never** `unwrap()` in production paths. Log warnings and return `None` / an
empty state.

## 4. Minimal blast radius

- No cloud state to corrupt.
- SQLite DB is a single file. Worst case: delete it and start fresh.
- Settings are a JSON file. Corrupt JSON falls back to defaults.
- Audio buffers are in-memory only. Process crash loses at most one in-flight
  segment.

## 5. Simplicity over cleverness

Three similar lines of code are better than a premature abstraction. Only add
complexity when the current design visibly breaks under actual use.

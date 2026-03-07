# Reliability

## Failure modes and mitigations

| Failure | Mitigation |
|---|---|
| ASR server unreachable | `transcribe()` returns `None`; recording continues; transcript stored as empty string; UI shows "Transcription unavailable" |
| Summarization server unreachable | `summarize()` returns `None`; recording saved without summary; user can regenerate later |
| No microphone device | `open_mic()` returns `Err`; recorder logs warning and returns; tray status shows "No microphone" |
| No system audio device | `open_system_audio()` returns `Err`; same graceful path |
| SQLite write failure | `anyhow::Result` propagates to Tauri command; frontend receives error string and displays it |
| Settings JSON corrupt | `serde_json::from_str` returns `Err`; `Settings::load` falls back to `Default::default()`; no crash |
| Process crash mid-recording | In-flight audio segment is lost; DB row has `ended_at = NULL`; displayed as "in progress" until user deletes it |

## Crash recovery

On next launch Scribe scans for recordings with `ended_at IS NULL`. These are
displayed with a "⚠ Recording was interrupted" badge. The user can review the
partial transcript and delete the row if desired.

*(Not yet implemented – tracked as a future improvement.)*

## Long-running stability

The recorder runs on a Tokio background task. Audio capture callbacks write
to `Arc<Mutex<Vec<f32>>>` buffers; the main task drains them on a 1-second
analysis tick. This design avoids buffer overflow as long as the analysis
tick keeps up with sample rate.

If the Tokio runtime is starved (e.g. blocking DB call on the async executor),
the analysis tick will lag and audio will accumulate. All DB calls should
be run via `spawn_blocking` to prevent this.

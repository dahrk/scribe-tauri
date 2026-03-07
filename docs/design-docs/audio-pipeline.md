# Audio Pipeline Design

## Why two-stream RMS gating?

Meeting-recorder apps that start on any voice activity produce excessive
false-positives (typing noise, fans, ambient speech). Requiring **both** mic
and system audio to exceed threshold means the app only records when a local
participant is actively speaking *and* the remote call audio is flowing – the
exact condition that defines "a meeting is in progress".

## State machine rationale

```
Idle ──(both_active × min_active_windows)──► Active
                                                │
                                         (both_idle)
                                                ▼
                                           Stopping {count}
                                                │
                         (both_idle × min_idle_windows) ──► Idle
                         (either active)        ──► Active
```

`min_active_windows` (default 3 × 1 s windows) debounces brief spikes.
`min_idle_windows` (default 180 × 1 s = 3 min) prevents premature
stop on a few seconds of silence mid-meeting.

## Memory-only audio

Audio bytes are accumulated in a `Arc<Mutex<Vec<f32>>>` (per stream). At each
segment boundary `drain()` swaps the Vec out in one lock acquisition, encodes
to WAV in memory, and ships the byte slice to the ASR call. No file is
ever opened for audio data. The WAV encoding uses the `hound` crate entirely
in memory (cursor-backed writer).

The only exception is `WhisperCli` which requires a file path for the subprocess.
In that case a temp file is written immediately before the CLI call and deleted
immediately after `output.await`.

## Segmenter flush triggers

Two conditions flush the current audio buffer to ASR:

1. **Timer**: `segment_interval_secs` (default 60 s) – keeps transcription
   near-real-time for long uninterrupted speech.
2. **Dominant source switch**: when the RMS-dominant source flips between mic
   and system audio. This produces better per-speaker segment quality, since
   Parakeet transcribes one audio type at a time more accurately than a mix.

Equal RMS is tie-broken to mic-dominant to prefer the local speaker.

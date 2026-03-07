# ASR Backend Design

## Primary: NVIDIA Parakeet

Parakeet TDT (Token-and-Duration Transducer) from NVIDIA NeMo delivers
state-of-the-art accuracy at low latency on NVIDIA and Apple Silicon GPUs.
It is the recommended backend for production use.

### HTTP interface contract

Both `Parakeet` and `HttpServer` variants use the same HTTP contract:

```
POST <base_url>/transcribe
Content-Type: audio/wav
Body: raw WAV bytes (16-bit PCM, any sample rate – server resamples)

200 OK
{"text": "…transcribed text…"}
```

The Rust `transcribe_http()` function appends `/transcribe` if the URL doesn't
already end with it, normalising both `http://host:port` and
`http://host:port/transcribe` inputs.

### Why keep WhisperCli?

Parakeet requires NeMo + a compatible GPU. `WhisperCli` provides a
zero-dependency CPU fallback:

```bash
pip install openai-whisper
```

It is intentionally kept in the enum so existing settings.json files
with `WhisperCli` continue to work without migration.

### Adding a future backend

See `AGENTS.md` § Adding a new ASR backend for the full checklist.

## Graceful degradation

If the ASR server is unreachable, `transcribe()` returns `None`. The recorder
continues accumulating audio and the transcript is stored as an empty string.
The UI shows "Transcription unavailable" rather than crashing.

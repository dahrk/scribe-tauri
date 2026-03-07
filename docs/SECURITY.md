# Security Model

## Threat model

Scribe is a local-only desktop app. There is no network-facing surface
beyond loopback HTTP calls to local ASR/LLM servers. The primary concerns are:

1. **Inadvertent data exfiltration**: audio or transcript sent to a remote server.
2. **Local privilege escalation**: malicious IPC commands from a compromised webview.
3. **Settings tampering**: a malicious process modifying `settings.json` to
   point ASR/summarization at a remote endpoint.

## Mitigations

### 1. No audio ever leaves the machine

- Audio bytes are kept in `Arc<Mutex<Vec<f32>>>` heap memory.
- The only file write for audio is the `WhisperCli` temp file, deleted
  synchronously after the subprocess exits.
- ASR and summarization endpoints are configured to `127.0.0.1` by default.
- There is no code path that connects to a remote host; all `reqwest` calls
  use user-configured URLs stored in `settings.json`.

**Recommendation**: review `settings.json` if you suspect tampering.
The ASR `url` and summarization `base_url` fields must point to localhost.

### 2. Tauri IPC allowlist

All frontend→backend calls go through named Tauri commands defined in
`commands.rs`. The webview cannot execute arbitrary shell commands.
Shell plugin (`tauri-plugin-shell`) is included but scoped to the commands
listed in `tauri.conf.json`.

### 3. Input validation

All user-supplied strings (recording title, transcript, settings fields)
are passed as parameterised SQLite bindings (`params![...]`) – no string
interpolation into SQL. No SQL injection surface.

## Known risks

| Risk | Severity | Notes |
|---|---|---|
| User configures ASR URL to remote host | Medium | By design (power user). Document the privacy implication clearly in Settings UI. |
| `WhisperCli` temp file visible to other local processes | Low | Short-lived; deleted after subprocess exit. Use OS temp dir permissions. |
| LLM prompt contains sensitive transcript content | Low | Sent to local LLM only; no remote call by default. |

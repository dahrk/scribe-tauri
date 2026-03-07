# Tech Debt Tracker

| ID | Area | Description | Priority | Effort |
|---|---|---|---|---|
| TD-001 | db.rs | Large transcripts (>100 KB) stored as single TEXT blob; consider chunking | Low | Medium |
| TD-002 | recorder.rs | No reconnect logic if ASR server goes down mid-recording | Medium | Small |
| TD-003 | capture.rs | System audio device selection is heuristic-only; no manual override UI | Medium | Medium |
| TD-004 | asr.rs | WhisperCli writes a temp file per segment; no connection pooling | Low | Small |
| TD-005 | commands.rs | No pagination for `list_recordings`; will slow down with large databases | Medium | Small |
| TD-006 | lib.rs | Tray icon menu items are hardcoded strings, not localised | Low | Small |

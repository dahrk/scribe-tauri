# Plans

## Roadmap

### Now (v0.1 – shipped)
- [x] Auto-detect meetings via dual-stream RMS gating
- [x] Parakeet v3 HTTP ASR (primary) + Whisper CLI (fallback)
- [x] Ollama + OpenAI-compatible summarization
- [x] SQLite recording storage (text only, no audio)
- [x] Tray icon, system menu, auto-record toggle
- [x] ≥90% unit test coverage (Rust + React)

### Next (v0.2)
- [ ] Export recording as Markdown / PDF
- [ ] Manual device selection in Settings (TD-003)
- [ ] Pagination for large recording lists (TD-005)
- [ ] Reconnect/retry when ASR server goes down mid-recording (TD-002)

### Later
- [ ] Speaker diarisation (requires diarisation model)
- [ ] Multi-language support (requires multilingual Parakeet variant)
- [ ] Keyboard shortcuts for start/stop

## Active exec plans

See [exec-plans/active/](exec-plans/active/) for in-progress work.

## Tech debt

See [exec-plans/tech-debt-tracker.md](exec-plans/tech-debt-tracker.md).

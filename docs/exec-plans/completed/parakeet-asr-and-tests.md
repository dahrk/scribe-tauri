# Completed: Parakeet v3 ASR + Comprehensive Tests

**Completed:** 2026-03-07

## Goal

1. Switch the primary ASR backend from Whisper CLI to NVIDIA Parakeet v3.
2. Add ≥90% unit test coverage across Rust logic modules and React frontend.
3. Add AGENTS.md and structured docs/.

## Outcome

- `AsrBackend::Parakeet` variant added as the new default.
- `scripts/parakeet_server.py` – FastAPI NeMo server serving `/transcribe`.
- 72 Rust unit tests across 7 modules (all passing).
- 48 Vitest frontend tests across 5 test files (all passing).
- `docs/` knowledge base created; `AGENTS.md` refactored to TOC.
- `ARCHITECTURE.md` added.
- `README.md` rewritten with full user-facing documentation.

## Key decisions made

- Parakeet HTTP and generic HttpServer share the same `transcribe_http()` code
  path, differing only in how the URL is configured.
- WhisperCli retained as a supported fallback (no migration needed for
  existing settings.json files).
- Rust tests run against a stripped workspace (no GTK/ALSA deps) to support
  CI environments without full Tauri system libraries.

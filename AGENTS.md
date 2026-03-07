# Scribe – Agent Map

> This file is the **table of contents** for the repository's knowledge base.
> Read it first; follow links for depth.
> `CLAUDE.md` is a symlink to this file.

---

## What is Scribe?

A Tauri 2 desktop app (macOS + Linux) that **auto-records meetings, transcribes
locally with NVIDIA Parakeet v3, summarises with a local LLM, and stores only
text in SQLite**. Audio never leaves the machine.

→ Full user docs: [README.md](README.md)
→ Product vision: [docs/PRODUCT_SENSE.md](docs/PRODUCT_SENSE.md)
→ Feature specs: [docs/product-specs/index.md](docs/product-specs/index.md)

---

## Architecture overview

→ [ARCHITECTURE.md](ARCHITECTURE.md) – system diagram, data flow, module map

Key modules:

| Path | Role |
|---|---|
| `src-tauri/src/audio/` | Capture, RMS gating, segmentation |
| `src-tauri/src/asr.rs` | Parakeet HTTP / Whisper CLI / HttpServer |
| `src-tauri/src/recorder.rs` | Background pipeline orchestrator |
| `src-tauri/src/db.rs` | SQLite CRUD |
| `src-tauri/src/settings.rs` | JSON-persisted settings |
| `src-tauri/src/summarization.rs` | Ollama / OpenAI-compat |
| `src/components/` | React UI components |
| `src/hooks/useRecorder.ts` | Tauri event → React state |

---

## Design and decisions

→ [docs/DESIGN.md](docs/DESIGN.md) – core principles (privacy, graceful degradation)
→ [docs/design-docs/audio-pipeline.md](docs/design-docs/audio-pipeline.md) – why two-stream RMS gating
→ [docs/design-docs/asr-backends.md](docs/design-docs/asr-backends.md) – Parakeet vs Whisper, HTTP contract
→ [docs/design-docs/database.md](docs/design-docs/database.md) – SQLite decisions

---

## Running the app

```bash
# 1. Start the Parakeet ASR server (optional but recommended)
python scripts/parakeet_server.py --model parakeet-tdt-0.6b-v2 --port 9000

# 2. Start the Tauri dev server
bun install && bun run tauri dev
```

→ Full setup: [README.md](README.md)
→ Parakeet reference: [docs/references/parakeet-quickref.txt](docs/references/parakeet-quickref.txt)

---

## Running tests

```bash
bun run test                         # 48 frontend tests
cd src-tauri && cargo test       # 72 Rust unit tests (needs system libs)
```

→ Quality targets and coverage: [docs/QUALITY_SCORE.md](docs/QUALITY_SCORE.md)
→ Frontend conventions: [docs/FRONTEND.md](docs/FRONTEND.md)

---

## Coding conventions

→ [docs/DESIGN.md](docs/DESIGN.md) – first-principles
→ [docs/FRONTEND.md](docs/FRONTEND.md) – React/TypeScript patterns
→ [docs/RELIABILITY.md](docs/RELIABILITY.md) – failure modes and Option/Result discipline
→ [docs/SECURITY.md](docs/SECURITY.md) – local-only threat model

**Quick rules:**
- No `unwrap()` in production Rust paths. Use `?` or log + return `None`.
- All DB calls via `spawn_blocking`. All Tauri commands are `async`.
- Mirror every Rust enum variant in `src/types/index.ts`.
- Add `#[cfg(test)]` modules inline; use `Connection::open_in_memory()` for DB tests.
- Mock `@tauri-apps/api/*` globally in `src/test/setup.ts` only.

---

## Adding a new ASR backend (checklist)

1. Add variant to `AsrBackend` in `src-tauri/src/asr.rs`
2. Implement transcription function; add to `transcribe()` match
3. Add serialisation + error-path tests in `asr.rs`
4. Add variant to `AsrBackend` in `src/types/index.ts`
5. Update `asrBackendType()` and JSX in `SettingsPanel.tsx`
6. Update `src/test/mocks.ts` if default backend changes

---

## Plans and roadmap

→ [docs/PLANS.md](docs/PLANS.md) – what's shipped, what's next
→ [docs/exec-plans/tech-debt-tracker.md](docs/exec-plans/tech-debt-tracker.md)
→ [docs/exec-plans/active/](docs/exec-plans/active/) – in-progress work

---

## Tauri IPC command reference

→ [docs/references/tauri-commands-quickref.txt](docs/references/tauri-commands-quickref.txt)

---

## Keeping this map current

> **Rule**: whenever a module is added, renamed, or its purpose changes,
> update `AGENTS.md` and `ARCHITECTURE.md` in the same PR.
> When a feature ships, move its exec-plan to `docs/exec-plans/completed/`
> and update `docs/PLANS.md`. Keep `README.md` in sync with user-facing
> changes (new setup steps, new settings, new features).

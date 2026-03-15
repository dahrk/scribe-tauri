# Scribe – Agent Map

> This file is the **table of contents** for the repository's knowledge base.
> Read it first; follow links for depth.
> `CLAUDE.md` is a symlink to this file.

---

## What is Scribe?

A Tauri 2 desktop app (macOS + Linux) that **auto-records meetings, transcribes
locally with NVIDIA Parakeet, summarises with a local LLM, and stores only
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
bun run test                         # 65 frontend tests (unit + integration)
bun run test:integration             # integration tests only (App, recorder-flow)
cd src-tauri && cargo test       # Rust unit tests + 13 integration tests (needs system libs)
```

Test helpers and integration suites:

| Path | Role |
|---|---|
| `src/test/helpers/ipc.ts` | Typed `setupInvoke()` / `setupListen()` IPC helpers |
| `src/test/helpers/render.tsx` | `renderWithProviders()` wrapper for React tests |
| `src/test/integration/App.test.tsx` | 12 full-`<App />` integration tests |
| `src/test/integration/recorder-flow.test.ts` | 5 Idle→Recording→Idle state-machine tests |
| `src-tauri/tests/integration.rs` | 13 wiremock HTTP integration tests (ASR + summarization) |

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

### Mandatory doc checklist – include in every commit

After every commit, verify each item that applies:

- [ ] **New module or file** → add a row to the Key modules table above and update `ARCHITECTURE.md`
- [ ] **New or renamed test file** → update the test counts and file table in the "Running tests" section
- [ ] **New Tauri command** → add to `docs/references/tauri-commands-quickref.txt`
- [ ] **New ASR backend** → follow the "Adding a new ASR backend" checklist above
- [ ] **Feature shipped** → move exec-plan from `docs/exec-plans/active/` to `docs/exec-plans/completed/` and update `docs/PLANS.md`
- [ ] **User-facing change** → update `README.md` (setup steps, settings, features)
- [ ] **Coding convention added or changed** → update the relevant doc under `docs/` and the Quick rules section above

> Commits that add code without updating affected docs are considered incomplete.
> Reviewers should treat a missing doc update as a blocking issue.

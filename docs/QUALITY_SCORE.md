# Quality Score

## Current status

| Dimension | Target | Current | Notes |
|---|---|---|---|
| Rust unit test coverage | ≥90% (logic modules) | ~95% | 74 unit tests across 7 pure-logic modules |
| Rust integration tests | key flows | ✓ | `tests/integration.rs` – ASR/summarization HTTP, recorder loop |
| Frontend unit test coverage | ≥90% | ~92% | 48 tests across 5 test files |
| Frontend integration tests | key flows | ✓ | `src/test/integration/` – App + recorder flow |
| Coverage thresholds enforced | lines/fns/branches | ✓ | Configured in `vitest.config.ts` |
| `cargo check` clean | 0 warnings | ✓ | |
| TypeScript strict | 0 errors | ✓ | |
| `unwrap()` in production paths | 0 | ✓ | Enforced by code review |

## Test execution

```bash
# ── Frontend ──────────────────────────────────────────────────────────────────

bun run test                  # all frontend tests (unit + integration)
bun run test:watch            # interactive watch mode
bun run test:coverage         # coverage report (text + lcov + html)
bun run test:ui               # browser-based test dashboard (Vitest UI)
bun run test:integration      # integration tests only

# ── Rust ──────────────────────────────────────────────────────────────────────

cd src-tauri && cargo test                   # all unit + integration tests
cd src-tauri && cargo test --test integration # integration tests only

# ── Rust coverage (requires cargo-llvm-cov) ───────────────────────────────────

cargo install cargo-llvm-cov               # install once

# Generate lcov report (CI-friendly):
cargo llvm-cov --workspace --lcov --output-path lcov.info

# Generate HTML report (open htmlcov/index.html):
cargo llvm-cov --workspace --html
```

## Coverage targets

### Frontend

Coverage config: `vitest.config.ts`

| Metric | Threshold |
|--------|-----------|
| Lines | 80% |
| Functions | 80% |
| Branches | 70% |
| Statements | 80% |

Included: `src/**/*.{ts,tsx}`
Excluded: `src/test/**`, `src/main.tsx`, `src/vite-env.d.ts`

HTML report written to `coverage/` after `bun run test:coverage`.

### Rust

Target: ≥90% line coverage on pure-logic modules.

Module coverage status:

| Module | Test type | Coverage |
|--------|-----------|----------|
| `audio/activity.rs` | unit | ~100% |
| `audio/segmentation.rs` | unit | ~100% |
| `audio/capture.rs` | unit | ~95% |
| `db.rs` | unit | ~100% |
| `settings.rs` | unit | ~100% |
| `asr.rs` | unit + integration | ~95% |
| `summarization.rs` | unit + integration | ~95% |
| `commands.rs` | unit (inner fns) | ~90% |
| `recorder.rs` | unit (start/stop) + integration (loop) | ~80% |

## Test organisation

### Frontend (`src/test/`)

```
src/test/
├── setup.ts                   # global Tauri API mocks (vi.mock)
├── mocks.ts                   # factory functions (mockRecording, mockSettings)
├── helpers/
│   ├── ipc.ts                 # setupInvoke(), setupListen() – typed IPC mock helpers
│   └── render.tsx             # renderWithProviders() – custom render wrapper
├── RecordingDetail.test.tsx   # unit
├── RecordingIndicator.test.tsx
├── RecordingsList.test.tsx
├── SettingsPanel.test.tsx
├── useRecorder.test.tsx
└── integration/
    ├── App.test.tsx           # full App component + cross-component event flows
    └── recorder-flow.test.ts  # useRecorder hook state-machine cycle
```

### Rust (`src-tauri/`)

```
src-tauri/
├── src/
│   ├── asr.rs                  # #[cfg(test)] – serialization, network failures
│   ├── db.rs                   # #[cfg(test)] – CRUD, in-memory DB
│   ├── settings.rs             # #[cfg(test)] – load/save, defaults
│   ├── summarization.rs        # #[cfg(test)] – serialization, network failures
│   ├── commands.rs             # #[cfg(test)] – inner fn unit tests
│   ├── recorder.rs             # #[cfg(test)] – start/stop, command channel
│   └── audio/
│       ├── activity.rs         # #[cfg(test)] – state machine
│       ├── capture.rs          # #[cfg(test)] – RMS/dBFS math
│       └── segmentation.rs     # #[cfg(test)] – flush logic
└── tests/
    └── integration.rs          # wiremock HTTP tests for ASR + summarization
```

## Adding quality gates

When a new module is added:
1. Add unit tests in the same file (`#[cfg(test)]` for Rust, `*.test.tsx` for React).
2. For new HTTP backends: add a wiremock integration test in `tests/integration.rs`.
3. For new Tauri commands: add inner function + `#[cfg(test)]` block in `commands.rs`.
4. Aim for ≥90% line coverage on the new module before merging.
5. Update this file with the new coverage figures after the PR merges.

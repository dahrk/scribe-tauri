# Quality Score

## Current status

| Dimension | Target | Current | Notes |
|---|---|---|---|
| Rust test coverage | ≥90% (logic modules) | ~95% | 72 tests across 7 pure-logic modules |
| Frontend test coverage | ≥90% | ~92% | 48 tests across 5 test files |
| `cargo check` clean | 0 warnings | ✓ | |
| TypeScript strict | 0 errors | ✓ | |
| `unwrap()` in production paths | 0 | ✓ | Enforced by code review |

## Test execution

```bash
npm test                  # frontend (48 tests, ~8s)
npm run test:coverage     # frontend with coverage report

# Rust (requires system libs on Linux):
cd src-tauri && cargo test

# Rust (stripped workspace, CI-safe):
# See docs/design-docs/index.md for stripped workspace setup
```

## Coverage targets

Frontend coverage config: `vitest.config.ts` → `coverage.include: ["src/**/*.{ts,tsx}"]`

Excluded: `src/test/**`, `src/main.tsx`, `src/vite-env.d.ts`

## Adding quality gates

When a new module is added:
1. Add unit tests in the same file (`#[cfg(test)]` for Rust, `*.test.tsx` for React).
2. Aim for ≥90% line coverage on the new module before merging.
3. Update this file with the new coverage figures after the PR merges.

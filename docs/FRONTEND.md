# Frontend Guide

## Stack

| Layer | Technology |
|---|---|
| Framework | React 19 + TypeScript |
| Package manager | Bun 1 |
| Build | Vite 7 (via `bun run build`) |
| IPC | `@tauri-apps/api/core` (`invoke`) |
| Events | `@tauri-apps/api/event` (`listen`) |
| Styling | Plain CSS (BEM naming) in `App.css` |
| Tests | Vitest 4 + Testing Library + jsdom |

## Component tree

```
App.tsx
├── RecordingIndicator   # Status bar + Start/Stop
├── RecordingsList       # Sidebar: date-sorted list
└── RecordingDetail      # Main: editable title/transcript/summary
    └── SettingsPanel    # Shown when "Settings" nav item selected
```

## State flow

- `useRecorder` hook: subscribes to `recorder-status` Tauri events →
  feeds `RecorderStatus` up to `App.tsx`.
- `RecordingsList` fetches on mount and re-fetches on `recording-stopped`
  event.
- `RecordingDetail` is fully controlled: all edits live in local
  `useState`; saved explicitly via the Save button.
- No global state store. Props + callbacks are sufficient at this scale.

## TypeScript types

All Rust structs mirrored in `src/types/index.ts`. Rust enums are
TypeScript tagged unions:

```typescript
type AsrBackend =
  | { Parakeet: { url: string; model: string } }
  | { WhisperCli: { bin: string; model: string } }
  | { HttpServer: { url: string } }
  | "None";
```

## Testing conventions

| Rule | Rationale |
|---|---|
| Mock `@tauri-apps/api/*` globally in `setup.ts` | Single source of truth for mocks |
| `(invoke as Mock).mockResolvedValue(...)` per test | Explicit, readable per-test responses |
| `findBy*` / `waitFor` for async updates | Avoids flaky synchronous queries after promise resolution |
| Never assert on CSS colours or pixel positions | Test behaviour, not style |

## Adding a component

1. Create `src/components/MyComponent.tsx`.
2. Export a named (not default) function component.
3. Add corresponding `src/test/MyComponent.test.tsx`.
4. Import and place in `App.tsx`.
5. Add CSS to `App.css` using BEM: `.my-component__element--modifier`.

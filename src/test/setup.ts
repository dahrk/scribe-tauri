import "@testing-library/jest-dom";

// ── Tauri API mocks ───────────────────────────────────────────────────────────
// Tauri's IPC bridge doesn't exist in jsdom; mock the modules globally so every
// test file gets clean, controllable fakes without additional boilerplate.

vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn(),
}));

vi.mock("@tauri-apps/api/event", () => ({
  listen: vi.fn(() => Promise.resolve(() => {})),
  emit: vi.fn(),
}));

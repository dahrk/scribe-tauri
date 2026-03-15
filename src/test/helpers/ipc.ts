// ── IPC test helpers ──────────────────────────────────────────────────────────
// Typed utilities for setting up invoke/listen mocks in integration tests.
// Each test should call setupInvoke() and setupListen() in beforeEach so
// the mocks are reset and reconfigured for each test.

import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import type { Recording, RecorderStatus, Settings } from "../../types";

const invokeM = invoke as ReturnType<typeof vi.fn>;
const listenM = listen as ReturnType<typeof vi.fn>;

// ── Invoke ────────────────────────────────────────────────────────────────────

type InvokeHandlers = {
  list_recordings?: () => Recording[];
  get_recording?: (args: { id: string }) => Recording | null;
  update_recording?: (args: {
    id: string;
    title?: string;
    transcript?: string;
    summary?: string;
  }) => void;
  delete_recording?: (args: { id: string }) => void;
  start_recording?: () => void;
  stop_recording?: () => void;
  get_settings?: () => Settings;
  save_settings?: (args: { newSettings: Settings }) => void;
  retry_summary?: (args: { id: string }) => void;
};

/**
 * Resets and re-configures the invoke mock to dispatch to per-command handlers.
 * Any command not listed in `handlers` resolves with `undefined`.
 */
export function setupInvoke(handlers: InvokeHandlers = {}) {
  invokeM.mockReset();
  invokeM.mockImplementation((cmd: string, args?: unknown) => {
    const handler = handlers[cmd as keyof InvokeHandlers];
    if (handler) {
      return Promise.resolve((handler as (a: unknown) => unknown)(args));
    }
    return Promise.resolve(undefined);
  });
  return invokeM;
}

// ── Listen ────────────────────────────────────────────────────────────────────

type AnyPayload = RecorderStatus | string | unknown;
type EventCallback = (event: { payload: AnyPayload }) => void;

/**
 * Resets the listen mock and captures callbacks per event name.
 * Returns `emit(name, payload)` to fire events programmatically in tests.
 */
export function setupListen() {
  const eventMap = new Map<string, EventCallback[]>();

  listenM.mockReset();
  listenM.mockImplementation((name: string, cb: EventCallback) => {
    const cbs = eventMap.get(name) ?? [];
    cbs.push(cb);
    eventMap.set(name, cbs);
    return Promise.resolve(() => {
      const current = eventMap.get(name) ?? [];
      eventMap.set(name, current.filter((c) => c !== cb));
    });
  });

  return {
    /** Simulate a backend event being emitted to the frontend. */
    emit: (name: string, payload: AnyPayload) => {
      const cbs = eventMap.get(name) ?? [];
      cbs.forEach((cb) => cb({ payload }));
    },
  };
}

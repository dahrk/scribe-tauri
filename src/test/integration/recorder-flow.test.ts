// ── Recorder flow integration tests ──────────────────────────────────────────
// Tests the IPC event contract between the Rust recorder and the React useRecorder hook.
// Unlike the hook's unit tests (which test one behaviour in isolation),
// these tests verify the full state-machine flow across multiple events.

import { renderHook, act } from "@testing-library/react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { useRecorder } from "../../hooks/useRecorder";
import { setupInvoke, setupListen } from "../helpers/ipc";
import type { RecorderStatus } from "../../types";

let emitEvent: ReturnType<typeof setupListen>["emit"];

beforeEach(() => {
  const listeners = setupListen();
  emitEvent = listeners.emit;
  setupInvoke({
    start_recording: () => undefined,
    stop_recording: () => undefined,
  });
});

describe("Recorder flow – full cycle", () => {
  it("transitions Idle → Recording → Idle through user-initiated start/stop", async () => {
    const { result } = renderHook(() => useRecorder());

    // 1. Initial state: idle
    expect(result.current.status.is_recording).toBe(false);
    expect(result.current.status.recording_id).toBeNull();

    // 2. User clicks Start
    await act(async () => {
      result.current.startRecording();
    });
    expect(vi.mocked(invoke)).toHaveBeenCalledWith("start_recording");

    // 3. Backend emits recording-started status
    act(() => {
      emitEvent("recorder-status", {
        is_recording: true,
        recording_id: "flow-001",
        transcript_preview: "",
      } satisfies RecorderStatus);
    });

    expect(result.current.status.is_recording).toBe(true);
    expect(result.current.status.recording_id).toBe("flow-001");
    expect(result.current.status.transcript_preview).toBe("");

    // 4. Live transcription arrives mid-recording
    act(() => {
      emitEvent("recorder-status", {
        is_recording: true,
        recording_id: "flow-001",
        transcript_preview: "Hello everyone. Let's begin.",
      } satisfies RecorderStatus);
    });

    expect(result.current.status.transcript_preview).toBe(
      "Hello everyone. Let's begin."
    );

    // 5. User clicks Stop
    await act(async () => {
      result.current.stopRecording();
    });
    expect(vi.mocked(invoke)).toHaveBeenCalledWith("stop_recording");

    // 6. Backend emits idle status after finalising the recording
    act(() => {
      emitEvent("recorder-status", {
        is_recording: false,
        recording_id: null,
        transcript_preview: "",
      } satisfies RecorderStatus);
    });

    expect(result.current.status.is_recording).toBe(false);
    expect(result.current.status.recording_id).toBeNull();
    expect(result.current.status.transcript_preview).toBe("");
  });

  it("handles rapid back-to-back start/stop cycles without interleaving state", async () => {
    const { result } = renderHook(() => useRecorder());

    // First session
    act(() => {
      emitEvent("recorder-status", {
        is_recording: true,
        recording_id: "session-a",
        transcript_preview: "Session A",
      } satisfies RecorderStatus);
    });
    expect(result.current.status.recording_id).toBe("session-a");

    act(() => {
      emitEvent("recorder-status", {
        is_recording: false,
        recording_id: null,
        transcript_preview: "",
      } satisfies RecorderStatus);
    });
    expect(result.current.status.is_recording).toBe(false);

    // Second session immediately after
    act(() => {
      emitEvent("recorder-status", {
        is_recording: true,
        recording_id: "session-b",
        transcript_preview: "",
      } satisfies RecorderStatus);
    });
    expect(result.current.status.recording_id).toBe("session-b");
    expect(result.current.status.is_recording).toBe(true);
  });

  it("transcript preview grows as segments arrive", async () => {
    const { result } = renderHook(() => useRecorder());

    act(() => {
      emitEvent("recorder-status", {
        is_recording: true,
        recording_id: "seg-001",
        transcript_preview: "First segment.",
      } satisfies RecorderStatus);
    });
    expect(result.current.status.transcript_preview).toBe("First segment.");

    act(() => {
      emitEvent("recorder-status", {
        is_recording: true,
        recording_id: "seg-001",
        transcript_preview: "First segment. Second segment.",
      } satisfies RecorderStatus);
    });
    expect(result.current.status.transcript_preview).toBe(
      "First segment. Second segment."
    );
  });
});

describe("Recorder flow – cleanup", () => {
  it("unsubscribes from recorder-status on unmount", async () => {
    const unsubscribe = vi.fn();
    vi.mocked(listen).mockResolvedValue(unsubscribe);

    const { unmount } = renderHook(() => useRecorder());
    await act(async () => {});
    unmount();
    await act(async () => {});

    expect(unsubscribe).toHaveBeenCalled();
  });

  it("does not update state after unmount when events arrive", async () => {
    const consoleError = vi
      .spyOn(console, "error")
      .mockImplementation(() => {});

    const { result, unmount } = renderHook(() => useRecorder());
    unmount();

    // Firing event after unmount should not cause React state-update warnings.
    act(() => {
      emitEvent("recorder-status", {
        is_recording: true,
        recording_id: "ghost",
        transcript_preview: "",
      } satisfies RecorderStatus);
    });

    // No state update errors emitted
    expect(consoleError).not.toHaveBeenCalledWith(
      expect.stringContaining("Can't perform a React state update")
    );
    consoleError.mockRestore();
  });
});

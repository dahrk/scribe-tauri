import { renderHook, act } from "@testing-library/react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { useRecorder } from "../hooks/useRecorder";
import type { RecorderStatus } from "../types";

const invokeM = invoke as ReturnType<typeof vi.fn>;
const listenM = listen as ReturnType<typeof vi.fn>;

beforeEach(() => {
  invokeM.mockReset();
  listenM.mockReset();
  listenM.mockResolvedValue(() => {});
});

describe("useRecorder", () => {
  it("initialises with idle status", () => {
    const { result } = renderHook(() => useRecorder());
    expect(result.current.status.is_recording).toBe(false);
    expect(result.current.status.recording_id).toBeNull();
    expect(result.current.status.transcript_preview).toBe("");
  });

  it("subscribes to recorder-status event on mount", () => {
    renderHook(() => useRecorder());
    expect(listenM).toHaveBeenCalledWith("recorder-status", expect.any(Function));
  });

  it("unsubscribes on unmount", async () => {
    const unsubscribe = vi.fn();
    listenM.mockResolvedValue(unsubscribe);
    const { unmount } = renderHook(() => useRecorder());
    // Allow the promise to resolve
    await act(async () => {});
    unmount();
    await act(async () => {});
    expect(unsubscribe).toHaveBeenCalled();
  });

  it("updates status when recorder-status event fires", async () => {
    let capturedCallback: ((e: { payload: RecorderStatus }) => void) | null = null;
    listenM.mockImplementation(
      (_name: string, cb: (e: { payload: RecorderStatus }) => void) => {
        capturedCallback = cb;
        return Promise.resolve(() => {});
      }
    );

    const { result } = renderHook(() => useRecorder());
    // Simulate incoming event
    act(() => {
      capturedCallback?.({
        payload: {
          is_recording: true,
          recording_id: "xyz",
          transcript_preview: "Live text",
        },
      });
    });

    expect(result.current.status.is_recording).toBe(true);
    expect(result.current.status.recording_id).toBe("xyz");
    expect(result.current.status.transcript_preview).toBe("Live text");
  });

  it("startRecording calls invoke with start_recording", async () => {
    invokeM.mockResolvedValue(undefined);
    const { result } = renderHook(() => useRecorder());
    await act(async () => {
      result.current.startRecording();
    });
    expect(invokeM).toHaveBeenCalledWith("start_recording");
  });

  it("stopRecording calls invoke with stop_recording", async () => {
    invokeM.mockResolvedValue(undefined);
    const { result } = renderHook(() => useRecorder());
    await act(async () => {
      result.current.stopRecording();
    });
    expect(invokeM).toHaveBeenCalledWith("stop_recording");
  });
});

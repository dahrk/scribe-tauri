// ── App integration tests ────────────────────────────────────────────────────
// Tests the full <App /> component end-to-end with all Tauri IPC mocked.
// These tests verify cross-component behaviour that unit tests cannot cover:
//   - Events from the backend ripple through multiple components
//   - Navigation changes which components mount/unmount
//   - User flows that span RecordingIndicator, RecordingsList, RecordingDetail

import { render, screen, fireEvent, waitFor, act } from "@testing-library/react";
import App from "../../App";
import { setupInvoke, setupListen } from "../helpers/ipc";
import { mockRecording, mockSettings } from "../mocks";
import type { RecorderStatus } from "../../types";

// Each test gets a fresh mock state.
let emitEvent: ReturnType<typeof setupListen>["emit"];

beforeEach(() => {
  const listeners = setupListen();
  emitEvent = listeners.emit;
  setupInvoke({
    list_recordings: () => [],
    get_settings: () => mockSettings(),
    start_recording: () => undefined,
    stop_recording: () => undefined,
  });
});

describe("App – initial render", () => {
  it("shows Idle indicator and Start button by default", async () => {
    render(<App />);
    await waitFor(() => expect(screen.getByText("Idle")).toBeInTheDocument());
    expect(screen.getByRole("button", { name: /start/i })).toBeInTheDocument();
  });

  it("shows the empty recordings state on first load", async () => {
    render(<App />);
    await waitFor(() =>
      expect(screen.getByText(/No recordings yet/i)).toBeInTheDocument()
    );
  });

  it("shows the Scribe logo in the sidebar", async () => {
    render(<App />);
    // Wait for async component mounts to settle before asserting.
    await waitFor(() => expect(screen.getByText("Scribe")).toBeInTheDocument());
  });
});

describe("App – recorder status events", () => {
  it("switches to 'Recording…' when recorder-status fires with is_recording=true", async () => {
    render(<App />);
    await waitFor(() => screen.getByText("Idle"));

    act(() => {
      emitEvent("recorder-status", {
        is_recording: true,
        recording_id: "rec-001",
        transcript_preview: "",
      } satisfies RecorderStatus);
    });

    await waitFor(() =>
      expect(screen.getByText("Recording…")).toBeInTheDocument()
    );
    expect(screen.getByRole("button", { name: /stop/i })).toBeInTheDocument();
  });

  it("shows live transcript preview while recording", async () => {
    render(<App />);
    await waitFor(() => screen.getByText("Idle"));

    act(() => {
      emitEvent("recorder-status", {
        is_recording: true,
        recording_id: "rec-002",
        transcript_preview: "Hello, this is the meeting transcript.",
      } satisfies RecorderStatus);
    });

    await waitFor(() =>
      expect(
        screen.getByText(/Hello, this is the meeting transcript/)
      ).toBeInTheDocument()
    );
  });

  it("returns to Idle when recorder-status fires with is_recording=false", async () => {
    render(<App />);
    await waitFor(() => screen.getByText("Idle"));

    act(() => {
      emitEvent("recorder-status", {
        is_recording: true,
        recording_id: "rec-003",
        transcript_preview: "",
      } satisfies RecorderStatus);
    });
    await waitFor(() => screen.getByText("Recording…"));

    act(() => {
      emitEvent("recorder-status", {
        is_recording: false,
        recording_id: null,
        transcript_preview: "",
      } satisfies RecorderStatus);
    });

    await waitFor(() =>
      expect(screen.getByText("Idle")).toBeInTheDocument()
    );
  });
});

describe("App – recording list refresh", () => {
  it("adds a new recording to the list when recording-stopped fires", async () => {
    const newRec = mockRecording({ id: "fresh-1", title: "Auto Recording 1" });
    let callCount = 0;
    setupInvoke({
      list_recordings: () => {
        callCount += 1;
        return callCount === 1 ? [] : [newRec];
      },
      get_settings: () => mockSettings(),
    });

    render(<App />);
    await waitFor(() => screen.getByText(/No recordings yet/i));

    act(() => {
      emitEvent("recording-stopped", "fresh-1");
    });

    await waitFor(() =>
      expect(screen.getByText("Auto Recording 1")).toBeInTheDocument()
    );
  });

  it("shows multiple recordings after multiple stops", async () => {
    const recs = [
      mockRecording({ id: "m1", title: "Meeting Alpha" }),
      mockRecording({ id: "m2", title: "Meeting Beta" }),
    ];
    let callCount = 0;
    setupInvoke({
      list_recordings: () => {
        callCount += 1;
        return callCount <= 1 ? [] : recs;
      },
      get_settings: () => mockSettings(),
    });

    render(<App />);
    await waitFor(() => screen.getByText(/No recordings yet/i));

    act(() => {
      emitEvent("recording-stopped", "m1");
    });

    await waitFor(() => {
      expect(screen.getByText("Meeting Alpha")).toBeInTheDocument();
      expect(screen.getByText("Meeting Beta")).toBeInTheDocument();
    });
  });
});

describe("App – navigation", () => {
  it("switches to Settings view on sidebar click", async () => {
    render(<App />);
    fireEvent.click(screen.getByRole("button", { name: "Settings" }));
    await waitFor(() =>
      expect(screen.getByText("Transcription (ASR)")).toBeInTheDocument()
    );
  });

  it("switches back to Recordings view from Settings", async () => {
    render(<App />);
    fireEvent.click(screen.getByRole("button", { name: "Settings" }));
    await waitFor(() => screen.getByText("Transcription (ASR)"));

    fireEvent.click(screen.getByRole("button", { name: "Recordings" }));
    await waitFor(() =>
      expect(screen.getByText(/No recordings yet/i)).toBeInTheDocument()
    );
  });
});

describe("App – recording selection", () => {
  it("shows recording detail when a recording is clicked", async () => {
    const rec = mockRecording({ id: "sel-1", title: "Selected Recording" });
    setupInvoke({
      list_recordings: () => [rec],
      get_settings: () => mockSettings(),
    });

    render(<App />);
    await waitFor(() => screen.getByText("Selected Recording"));
    fireEvent.click(screen.getByText("Selected Recording"));

    await waitFor(() =>
      expect(screen.getByDisplayValue("Selected Recording")).toBeInTheDocument()
    );
  });

  it("clears detail pane when recording is deleted", async () => {
    const rec = mockRecording({ id: "del-1", title: "To Be Deleted" });
    setupInvoke({
      list_recordings: () => [rec],
      delete_recording: () => undefined,
      get_settings: () => mockSettings(),
    });
    // RecordingDetail uses window.confirm() for delete confirmation.
    vi.spyOn(window, "confirm").mockReturnValue(true);

    render(<App />);
    await waitFor(() => screen.getByText("To Be Deleted"));
    fireEvent.click(screen.getByText("To Be Deleted"));
    await waitFor(() => screen.getByDisplayValue("To Be Deleted"));

    fireEvent.click(screen.getByRole("button", { name: /delete/i }));

    await waitFor(() =>
      expect(screen.getByText(/Select a recording/i)).toBeInTheDocument()
    );
    vi.restoreAllMocks();
  });
});

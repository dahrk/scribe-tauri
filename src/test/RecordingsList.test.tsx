import { render, screen, fireEvent, waitFor } from "@testing-library/react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { RecordingsList } from "../components/RecordingsList";
import { mockRecording } from "./mocks";

const invokeM = invoke as ReturnType<typeof vi.fn>;
const listenM = listen as ReturnType<typeof vi.fn>;

beforeEach(() => {
  invokeM.mockReset();
  listenM.mockReset();
  listenM.mockResolvedValue(() => {});
});

describe("RecordingsList", () => {
  it("shows empty state when no recordings", async () => {
    invokeM.mockResolvedValue([]);
    render(<RecordingsList onSelect={vi.fn()} selectedId={null} />);
    await waitFor(() =>
      expect(screen.getByText(/No recordings yet/i)).toBeInTheDocument()
    );
  });

  it("renders a list of recordings", async () => {
    invokeM.mockResolvedValue([
      mockRecording({ id: "a", title: "Meeting A" }),
      mockRecording({ id: "b", title: "Meeting B" }),
    ]);
    render(<RecordingsList onSelect={vi.fn()} selectedId={null} />);
    await waitFor(() => {
      expect(screen.getByText("Meeting A")).toBeInTheDocument();
      expect(screen.getByText("Meeting B")).toBeInTheDocument();
    });
  });

  it("calls onSelect when a recording is clicked", async () => {
    const rec = mockRecording({ id: "x", title: "Clickable Meeting" });
    invokeM.mockResolvedValue([rec]);
    const onSelect = vi.fn();
    render(<RecordingsList onSelect={onSelect} selectedId={null} />);
    await waitFor(() => screen.getByText("Clickable Meeting"));
    fireEvent.click(screen.getByText("Clickable Meeting"));
    expect(onSelect).toHaveBeenCalledWith(rec);
  });

  it("marks selected recording with active class", async () => {
    const rec = mockRecording({ id: "sel", title: "Selected" });
    invokeM.mockResolvedValue([rec]);
    const { container } = render(
      <RecordingsList onSelect={vi.fn()} selectedId="sel" />
    );
    await waitFor(() => screen.getByText("Selected"));
    expect(
      container.querySelector(".recordings-list__item--selected")
    ).toBeInTheDocument();
  });

  it("shows summary hint when summary exists", async () => {
    invokeM.mockResolvedValue([
      mockRecording({ id: "s", summary: "• Key action item discussed" }),
    ]);
    render(<RecordingsList onSelect={vi.fn()} selectedId={null} />);
    await waitFor(() =>
      expect(screen.getByText(/Key action item/)).toBeInTheDocument()
    );
  });

  it("shows duration for completed recordings", async () => {
    invokeM.mockResolvedValue([
      mockRecording({
        started_at: "2026-03-07T14:00:00Z",
        ended_at: "2026-03-07T14:30:00Z",
      }),
    ]);
    render(<RecordingsList onSelect={vi.fn()} selectedId={null} />);
    await waitFor(() =>
      expect(screen.getByText("30m 0s")).toBeInTheDocument()
    );
  });

  it("shows dash for recordings with no end time", async () => {
    invokeM.mockResolvedValue([
      mockRecording({ ended_at: null }),
    ]);
    render(<RecordingsList onSelect={vi.fn()} selectedId={null} />);
    await waitFor(() =>
      expect(screen.getByText("—")).toBeInTheDocument()
    );
  });

  it("re-fetches when recording-stopped event fires", async () => {
    let eventCallback: ((e: any) => void) | null = null;
    listenM.mockImplementation((_name: string, cb: (e: any) => void) => {
      eventCallback = cb;
      return Promise.resolve(() => {});
    });
    invokeM
      .mockResolvedValueOnce([])
      .mockResolvedValueOnce([mockRecording({ id: "new" })]);

    render(<RecordingsList onSelect={vi.fn()} selectedId={null} />);
    await waitFor(() => screen.getByText(/No recordings/));

    // Simulate the event
    eventCallback?.({ event: "recording-stopped", payload: "new" });
    await waitFor(() =>
      expect(screen.queryByText(/No recordings/)).not.toBeInTheDocument()
    );
  });
});

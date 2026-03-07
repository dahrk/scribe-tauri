import { render, screen, fireEvent, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { invoke } from "@tauri-apps/api/core";
import { RecordingDetail } from "../components/RecordingDetail";
import { mockRecording } from "./mocks";

const invokeM = invoke as ReturnType<typeof vi.fn>;

beforeEach(() => {
  invokeM.mockReset();
  // Default: invoke resolves successfully
  invokeM.mockResolvedValue(undefined);
});

describe("RecordingDetail", () => {
  it("renders the title", () => {
    render(
      <RecordingDetail
        recording={mockRecording()}
        onDeleted={vi.fn()}
        onUpdated={vi.fn()}
      />
    );
    expect(screen.getByDisplayValue("Meeting 2026-03-07 14:30"));
  });

  it("renders the transcript", () => {
    render(
      <RecordingDetail
        recording={mockRecording()}
        onDeleted={vi.fn()}
        onUpdated={vi.fn()}
      />
    );
    expect(
      screen.getByDisplayValue("Hello everyone. Let's get started.")
    );
  });

  it("renders the summary", () => {
    render(
      <RecordingDetail
        recording={mockRecording()}
        onDeleted={vi.fn()}
        onUpdated={vi.fn()}
      />
    );
    const summaryArea = screen
      .getAllByRole("textbox")
      .find((el) => (el as HTMLTextAreaElement).value.includes("Meeting started"));
    expect(summaryArea).toBeDefined();
  });

  it("shows 'No summary available' when summary is null", () => {
    render(
      <RecordingDetail
        recording={mockRecording({ summary: null })}
        onDeleted={vi.fn()}
        onUpdated={vi.fn()}
      />
    );
    expect(screen.getByText(/No summary available/));
  });

  it("shows 'No transcript yet' when transcript is null", () => {
    render(
      <RecordingDetail
        recording={mockRecording({ transcript: null })}
        onDeleted={vi.fn()}
        onUpdated={vi.fn()}
      />
    );
    expect(screen.getByText("No transcript yet."));
  });

  it("calls update_recording on Save", async () => {
    const onUpdated = vi.fn();
    render(
      <RecordingDetail
        recording={mockRecording()}
        onDeleted={vi.fn()}
        onUpdated={onUpdated}
      />
    );
    fireEvent.click(screen.getByRole("button", { name: "Save" }));
    await waitFor(() => {
      expect(invokeM).toHaveBeenCalledWith("update_recording", expect.any(Object));
      expect(onUpdated).toHaveBeenCalled();
    });
  });

  it("shows error when save fails", async () => {
    invokeM.mockRejectedValue(new Error("DB error"));
    render(
      <RecordingDetail
        recording={mockRecording()}
        onDeleted={vi.fn()}
        onUpdated={vi.fn()}
      />
    );
    fireEvent.click(screen.getByRole("button", { name: "Save" }));
    await waitFor(() =>
      expect(screen.getByText(/DB error/)).toBeInTheDocument()
    );
  });

  it("calls delete_recording and onDeleted on Delete confirm", async () => {
    const onDeleted = vi.fn();
    window.confirm = vi.fn().mockReturnValue(true);
    render(
      <RecordingDetail
        recording={mockRecording()}
        onDeleted={onDeleted}
        onUpdated={vi.fn()}
      />
    );
    fireEvent.click(screen.getByRole("button", { name: "Delete" }));
    await waitFor(() => {
      expect(invokeM).toHaveBeenCalledWith("delete_recording", { id: "rec-1" });
      expect(onDeleted).toHaveBeenCalled();
    });
  });

  it("does not delete when confirm is cancelled", async () => {
    window.confirm = vi.fn().mockReturnValue(false);
    const onDeleted = vi.fn();
    render(
      <RecordingDetail
        recording={mockRecording()}
        onDeleted={onDeleted}
        onUpdated={vi.fn()}
      />
    );
    fireEvent.click(screen.getByRole("button", { name: "Delete" }));
    expect(onDeleted).not.toHaveBeenCalled();
    expect(invokeM).not.toHaveBeenCalledWith("delete_recording", expect.any(Object));
  });

  it("calls retry_summary and refreshes on Regenerate", async () => {
    const updated = mockRecording({ summary: "New summary" });
    invokeM
      .mockResolvedValueOnce(undefined) // retry_summary
      .mockResolvedValueOnce(updated);  // get_recording

    const onUpdated = vi.fn();
    render(
      <RecordingDetail
        recording={mockRecording()}
        onDeleted={vi.fn()}
        onUpdated={onUpdated}
      />
    );
    fireEvent.click(screen.getByRole("button", { name: "Regenerate summary" }));
    await waitFor(() => {
      expect(invokeM).toHaveBeenCalledWith("retry_summary", { id: "rec-1" });
      expect(onUpdated).toHaveBeenCalledWith(updated);
    });
  });

  it("updates local title state on input change", async () => {
    const user = userEvent.setup();
    render(
      <RecordingDetail
        recording={mockRecording()}
        onDeleted={vi.fn()}
        onUpdated={vi.fn()}
      />
    );
    const input = screen.getByDisplayValue("Meeting 2026-03-07 14:30");
    await user.clear(input);
    await user.type(input, "Stand-up");
    expect(screen.getByDisplayValue("Stand-up"));
  });

  it("syncs fields when a different recording is selected", () => {
    const { rerender } = render(
      <RecordingDetail
        recording={mockRecording({ id: "r1", title: "Meeting A" })}
        onDeleted={vi.fn()}
        onUpdated={vi.fn()}
      />
    );
    expect(screen.getByDisplayValue("Meeting A"));
    rerender(
      <RecordingDetail
        recording={mockRecording({ id: "r2", title: "Meeting B" })}
        onDeleted={vi.fn()}
        onUpdated={vi.fn()}
      />
    );
    expect(screen.getByDisplayValue("Meeting B"));
  });
});

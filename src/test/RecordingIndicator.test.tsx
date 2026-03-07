import { render, screen, fireEvent } from "@testing-library/react";
import { RecordingIndicator } from "../components/RecordingIndicator";
import type { RecorderStatus } from "../types";

const idle: RecorderStatus = {
  is_recording: false,
  recording_id: null,
  transcript_preview: "",
};

const recording: RecorderStatus = {
  is_recording: true,
  recording_id: "rec-1",
  transcript_preview: "Hello world",
};

describe("RecordingIndicator", () => {
  it("shows Idle label when not recording", () => {
    render(
      <RecordingIndicator status={idle} onStart={vi.fn()} onStop={vi.fn()} />
    );
    expect(screen.getByText("Idle")).toBeInTheDocument();
  });

  it("shows Recording label when recording", () => {
    render(
      <RecordingIndicator status={recording} onStart={vi.fn()} onStop={vi.fn()} />
    );
    expect(screen.getByText("Recording…")).toBeInTheDocument();
  });

  it("shows Start button when idle", () => {
    render(
      <RecordingIndicator status={idle} onStart={vi.fn()} onStop={vi.fn()} />
    );
    expect(screen.getByRole("button", { name: "Start" })).toBeInTheDocument();
  });

  it("shows Stop button when recording", () => {
    render(
      <RecordingIndicator status={recording} onStart={vi.fn()} onStop={vi.fn()} />
    );
    expect(screen.getByRole("button", { name: "Stop" })).toBeInTheDocument();
  });

  it("calls onStart when Start is clicked", () => {
    const onStart = vi.fn();
    render(
      <RecordingIndicator status={idle} onStart={onStart} onStop={vi.fn()} />
    );
    fireEvent.click(screen.getByRole("button", { name: "Start" }));
    expect(onStart).toHaveBeenCalledTimes(1);
  });

  it("calls onStop when Stop is clicked", () => {
    const onStop = vi.fn();
    render(
      <RecordingIndicator status={recording} onStart={vi.fn()} onStop={onStop} />
    );
    fireEvent.click(screen.getByRole("button", { name: "Stop" }));
    expect(onStop).toHaveBeenCalledTimes(1);
  });

  it("shows transcript preview when recording", () => {
    render(
      <RecordingIndicator status={recording} onStart={vi.fn()} onStop={vi.fn()} />
    );
    expect(screen.getByText("Hello world")).toBeInTheDocument();
  });

  it("does not show transcript preview when idle", () => {
    render(
      <RecordingIndicator
        status={{ ...idle, transcript_preview: "stale" }}
        onStart={vi.fn()}
        onStop={vi.fn()}
      />
    );
    expect(screen.queryByText("stale")).not.toBeInTheDocument();
  });

  it("applies active CSS class when recording", () => {
    const { container } = render(
      <RecordingIndicator status={recording} onStart={vi.fn()} onStop={vi.fn()} />
    );
    expect(container.querySelector(".rec-bar--active")).toBeInTheDocument();
  });

  it("does not apply active CSS class when idle", () => {
    const { container } = render(
      <RecordingIndicator status={idle} onStart={vi.fn()} onStop={vi.fn()} />
    );
    expect(container.querySelector(".rec-bar--active")).not.toBeInTheDocument();
  });
});

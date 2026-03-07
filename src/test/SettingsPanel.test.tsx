import { render, screen, fireEvent, waitFor } from "@testing-library/react";
import { invoke } from "@tauri-apps/api/core";
import { SettingsPanel } from "../components/SettingsPanel";
import { mockSettings } from "./mocks";

const invokeM = invoke as ReturnType<typeof vi.fn>;

beforeEach(() => {
  invokeM.mockReset();
  invokeM.mockResolvedValue(mockSettings());
});

describe("SettingsPanel", () => {
  it("loads and displays settings on mount", async () => {
    render(<SettingsPanel />);
    await waitFor(() =>
      expect(invokeM).toHaveBeenCalledWith("get_settings")
    );
    expect(screen.getByText("Settings")).toBeInTheDocument();
  });

  it("shows auto-record checkbox as checked by default", async () => {
    render(<SettingsPanel />);
    const checkbox = await screen.findByRole("checkbox");
    expect(checkbox).toBeChecked();
  });

  it("toggles auto-record checkbox", async () => {
    render(<SettingsPanel />);
    const checkbox = await screen.findByRole("checkbox");
    fireEvent.click(checkbox);
    expect(checkbox).not.toBeChecked();
  });

  it("shows all section headings", async () => {
    render(<SettingsPanel />);
    await waitFor(() => screen.getByText("Recording"));
    expect(screen.getByText("Transcription (ASR)")).toBeInTheDocument();
    expect(screen.getByText("Summarization")).toBeInTheDocument();
  });

  it("calls save_settings on Save button click", async () => {
    invokeM.mockResolvedValueOnce(mockSettings()).mockResolvedValueOnce(undefined);
    render(<SettingsPanel />);
    const btn = await screen.findByRole("button", { name: "Save settings" });
    fireEvent.click(btn);
    await waitFor(() =>
      expect(invokeM).toHaveBeenCalledWith("save_settings", { newSettings: expect.any(Object) })
    );
  });

  it("shows success message after save", async () => {
    invokeM.mockResolvedValueOnce(mockSettings()).mockResolvedValueOnce(undefined);
    render(<SettingsPanel />);
    const btn = await screen.findByRole("button", { name: "Save settings" });
    fireEvent.click(btn);
    await waitFor(() =>
      expect(screen.getByText("Settings saved.")).toBeInTheDocument()
    );
  });

  it("shows error message when save fails", async () => {
    invokeM
      .mockResolvedValueOnce(mockSettings())
      .mockRejectedValueOnce(new Error("write error"));
    render(<SettingsPanel />);
    const btn = await screen.findByRole("button", { name: "Save settings" });
    fireEvent.click(btn);
    await waitFor(() =>
      expect(screen.getByText(/write error/)).toBeInTheDocument()
    );
  });

  it("shows Parakeet URL input when Parakeet backend is selected", async () => {
    render(<SettingsPanel />);
    await waitFor(() => screen.getByText("Transcription (ASR)"));
    // Default is Parakeet
    const selects = screen.getAllByRole("combobox");
    // ASR select should show parakeet
    expect(selects[0]).toHaveValue("parakeet");
  });

  it("shows Whisper options when Whisper CLI is selected", async () => {
    render(<SettingsPanel />);
    const selects = await screen.findAllByRole("combobox");
    fireEvent.change(selects[0], { target: { value: "whisper" } });
    await waitFor(() =>
      expect(screen.getByText("Whisper binary path")).toBeInTheDocument()
    );
  });

  it("shows HTTP URL input when HTTP server is selected", async () => {
    render(<SettingsPanel />);
    const selects = await screen.findAllByRole("combobox");
    fireEvent.change(selects[0], { target: { value: "http" } });
    await waitFor(() =>
      expect(screen.getByText("Server URL")).toBeInTheDocument()
    );
  });

  it("shows hint when ASR is disabled", async () => {
    render(<SettingsPanel />);
    const selects = await screen.findAllByRole("combobox");
    fireEvent.change(selects[0], { target: { value: "none" } });
    await waitFor(() =>
      expect(screen.getByText(/Transcription is disabled/)).toBeInTheDocument()
    );
  });

  it("shows Ollama URL when Ollama summarization is selected", async () => {
    render(<SettingsPanel />);
    await waitFor(() => screen.getByText("Summarization"));
    expect(screen.getByDisplayValue("http://localhost:11434")).toBeInTheDocument();
  });
});

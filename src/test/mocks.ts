import type { Recording, Settings } from "../types";

export const mockRecording = (overrides: Partial<Recording> = {}): Recording => ({
  id: "rec-1",
  title: "Meeting 2026-03-07 14:30",
  started_at: "2026-03-07T14:30:00Z",
  ended_at: "2026-03-07T15:00:00Z",
  transcript: "Hello everyone. Let's get started.",
  summary: "• Meeting started\n• Action item: follow up",
  created_at: "2026-03-07T14:30:00Z",
  updated_at: "2026-03-07T15:00:00Z",
  ...overrides,
});

export const mockSettings = (): Settings => ({
  auto_record: true,
  idle_timeout_secs: 180,
  segment_interval_secs: 60,
  activity_threshold_dbfs: -40,
  min_active_windows: 3,
  asr_backend: {
    Parakeet: { url: "http://127.0.0.1:9000", model: "parakeet-tdt-0.6b-v2" },
  },
  summarization_backend: {
    Ollama: { base_url: "http://localhost:11434", model: "llama3.2:3b" },
  },
});

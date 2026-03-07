export interface Recording {
  id: string;
  title: string;
  started_at: string;
  ended_at: string | null;
  transcript: string | null;
  summary: string | null;
  created_at: string;
  updated_at: string;
}

export type AsrBackend =
  | { WhisperCli: { bin: string; model: string } }
  | { HttpServer: { url: string } }
  | "None";

export type SummarizationBackend =
  | { Ollama: { base_url: string; model: string } }
  | { OpenAiCompatible: { base_url: string; model: string; api_key: string | null } }
  | "None";

export interface Settings {
  auto_record: boolean;
  idle_timeout_secs: number;
  segment_interval_secs: number;
  activity_threshold_dbfs: number;
  min_active_windows: number;
  asr_backend: AsrBackend;
  summarization_backend: SummarizationBackend;
}

export interface RecorderStatus {
  is_recording: boolean;
  recording_id: string | null;
  transcript_preview: string;
}

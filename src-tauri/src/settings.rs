use crate::asr::AsrBackend;
use crate::summarization::SummarizationBackend;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Settings {
    /// Automatically start recording when both mic and system audio are active.
    pub auto_record: bool,
    /// Idle timeout in seconds before a recording is finalized.
    pub idle_timeout_secs: f32,
    /// Transcription segment interval in seconds.
    pub segment_interval_secs: f32,
    /// Activity detection threshold in dBFS.
    pub activity_threshold_dbfs: f32,
    /// Minimum consecutive active windows before recording starts.
    pub min_active_windows: u32,
    /// ASR backend configuration.
    pub asr_backend: AsrBackend,
    /// Summarization backend configuration.
    pub summarization_backend: SummarizationBackend,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            auto_record: true,
            idle_timeout_secs: 180.0,
            segment_interval_secs: 60.0,
            activity_threshold_dbfs: -40.0,
            min_active_windows: 3,
            asr_backend: AsrBackend::default(),
            summarization_backend: SummarizationBackend::default(),
        }
    }
}

impl Settings {
    pub fn load(path: &Path) -> Result<Self> {
        if path.exists() {
            let s = std::fs::read_to_string(path)?;
            Ok(serde_json::from_str(&s).unwrap_or_default())
        } else {
            Ok(Self::default())
        }
    }

    pub fn save(&self, path: &Path) -> Result<()> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let s = serde_json::to_string_pretty(self)?;
        std::fs::write(path, s)?;
        Ok(())
    }
}

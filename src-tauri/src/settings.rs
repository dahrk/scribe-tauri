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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::asr::AsrBackend;
    use crate::summarization::SummarizationBackend;
    use std::fs;
    use tempfile::TempDir;

    fn tmp_settings_path() -> (TempDir, std::path::PathBuf) {
        let dir = TempDir::new().expect("tempdir");
        let path = dir.path().join("settings.json");
        (dir, path)
    }

    // ── Default ───────────────────────────────────────────────────────────────

    #[test]
    fn default_auto_record_is_true() {
        assert!(Settings::default().auto_record);
    }

    #[test]
    fn default_segment_interval_is_60s() {
        let s = Settings::default();
        assert!((s.segment_interval_secs - 60.0).abs() < f32::EPSILON);
    }

    #[test]
    fn default_idle_timeout_is_180s() {
        let s = Settings::default();
        assert!((s.idle_timeout_secs - 180.0).abs() < f32::EPSILON);
    }

    #[test]
    fn default_asr_backend_is_parakeet() {
        assert!(matches!(Settings::default().asr_backend, AsrBackend::Parakeet { .. }));
    }

    // ── Save / Load round-trip ────────────────────────────────────────────────

    #[test]
    fn save_then_load_roundtrip() {
        let (_dir, path) = tmp_settings_path();
        let original = Settings {
            auto_record: false,
            idle_timeout_secs: 300.0,
            segment_interval_secs: 30.0,
            activity_threshold_dbfs: -35.0,
            min_active_windows: 5,
            asr_backend: AsrBackend::WhisperCli {
                bin: "/usr/local/bin/whisper".into(),
                model: "small".into(),
            },
            summarization_backend: SummarizationBackend::None,
        };
        original.save(&path).expect("save");
        let loaded = Settings::load(&path).expect("load");
        assert_eq!(loaded.auto_record, false);
        assert!((loaded.idle_timeout_secs - 300.0).abs() < f32::EPSILON);
        assert!((loaded.segment_interval_secs - 30.0).abs() < f32::EPSILON);
        assert_eq!(loaded.min_active_windows, 5);
    }

    #[test]
    fn load_nonexistent_returns_default() {
        let dir = TempDir::new().expect("tempdir");
        let path = dir.path().join("does_not_exist.json");
        let s = Settings::load(&path).expect("load");
        assert_eq!(s.auto_record, Settings::default().auto_record);
    }

    #[test]
    fn load_corrupt_json_returns_default() {
        let (_dir, path) = tmp_settings_path();
        fs::write(&path, b"this is not json").expect("write");
        let s = Settings::load(&path).expect("load");
        // Falls back to default
        assert_eq!(s.auto_record, Settings::default().auto_record);
    }

    #[test]
    fn save_creates_parent_directories() {
        let dir = TempDir::new().expect("tempdir");
        let path = dir.path().join("nested/sub/settings.json");
        Settings::default().save(&path).expect("save");
        assert!(path.exists());
    }

    #[test]
    fn save_produces_valid_json() {
        let (_dir, path) = tmp_settings_path();
        Settings::default().save(&path).expect("save");
        let content = fs::read_to_string(&path).expect("read");
        serde_json::from_str::<Settings>(&content).expect("valid JSON");
    }
}

/// Recording state machine and engine.
/// Runs in a Tokio background task; communicates with Tauri via app.emit().

use crate::audio::{encode_wav, AudioEngine};
use crate::audio::activity::ActivityConfig;
use crate::asr::{self, AsrBackend};
use crate::db;
use crate::settings::Settings;
use crate::summarization::{self, SummarizationBackend};
use anyhow::Result;
use chrono::Utc;
use parking_lot::Mutex;
use rusqlite::Connection;
use std::sync::Arc;
use tauri::{AppHandle, Emitter};
use tokio::sync::mpsc;
use tokio::time::{interval, Duration};

#[derive(Debug, Clone, serde::Serialize)]
pub struct RecordingStatus {
    pub is_recording: bool,
    pub recording_id: Option<String>,
    pub transcript_preview: String,
}

pub enum RecorderCommand {
    StartManual,
    StopManual,
    UpdateSettings(Settings),
}

// ── Event emission abstraction ────────────────────────────────────────────────
// Decouples the recorder loop from `tauri::AppHandle` so it can be driven by
// a mock emitter in tests without requiring the full Tauri runtime.

pub trait EventEmitter: Send + Sync + 'static {
    fn emit_status(&self, status: RecordingStatus);
    fn emit_started(&self, id: &str);
    fn emit_stopped(&self, id: &str);
}

impl EventEmitter for AppHandle {
    fn emit_status(&self, status: RecordingStatus) {
        let _ = self.emit("recorder-status", status);
    }

    fn emit_started(&self, id: &str) {
        let _ = self.emit("recording-started", id);
    }

    fn emit_stopped(&self, id: &str) {
        let _ = self.emit("recording-stopped", id);
    }
}

// ── Recorder state ────────────────────────────────────────────────────────────

struct InProgress {
    id: String,
    started_at: chrono::DateTime<Utc>,
    transcript: String,
    /// Accumulated audio since the last segment flush (mic + sys mixed).
    audio_buf: Vec<f32>,
}

pub async fn run_recorder<E: EventEmitter>(
    emitter: Arc<E>,
    db_conn: Arc<Mutex<Connection>>,
    initial_settings: Settings,
    mut cmd_rx: mpsc::Receiver<RecorderCommand>,
) {
    let settings = Arc::new(Mutex::new(initial_settings));

    // Try to open audio engine; if it fails we can still handle manual commands.
    let engine = match {
        let s = settings.lock();
        AudioEngine::new(
            ActivityConfig {
                threshold_dbfs: s.activity_threshold_dbfs,
                min_active_windows: s.min_active_windows,
                min_idle_windows: idle_windows(&s),
            },
            s.segment_interval_secs,
        )
    } {
        Ok(e) => Some(Arc::new(e)),
        Err(err) => {
            log::warn!("Audio engine init failed: {err}. Activity detection disabled.");
            emitter.emit_status(RecordingStatus {
                is_recording: false,
                recording_id: None,
                transcript_preview: String::new(),
            });
            None
        }
    };

    let mut in_progress: Option<InProgress> = None;
    let sample_rate = engine.as_ref().map(|e| e.sample_rate).unwrap_or(16_000);

    // Analysis tick: 1 s
    let mut tick = interval(Duration::from_secs(1));

    loop {
        tokio::select! {
            _ = tick.tick() => {
                let (mic, sys) = match &engine {
                    Some(e) => e.drain_buffers(),
                    None => continue,
                };

                let s = settings.lock().clone();
                let auto = s.auto_record;

                // Activity detection (only triggers auto-recording)
                if auto {
                    let transition = engine.as_ref().unwrap().activity.lock().feed(&mic, &sys);
                    match transition {
                        Some(true) if in_progress.is_none() => {
                            in_progress = start_recording(&db_conn, &*emitter);
                        }
                        Some(false) => {
                            if let Some(ip) = in_progress.take() {
                                stop_recording(ip, &db_conn, &s.asr_backend, &s.summarization_backend, &*emitter, sample_rate).await;
                            }
                        }
                        _ => {}
                    }
                }

                // Segmentation
                if let Some(ip) = in_progress.as_mut() {
                    let combined: Vec<f32> = mic.iter().zip(sys.iter()).map(|(m, s)| (m + s) * 0.5).collect();
                    ip.audio_buf.extend_from_slice(&combined);

                    let flush = engine.as_ref().unwrap().segmenter.lock().advance(combined.len(), &mic, &sys);
                    if flush {
                        let segment_audio = std::mem::take(&mut ip.audio_buf);
                        if let Ok(wav) = encode_wav(&segment_audio, sample_rate) {
                            let asr = s.asr_backend.clone();
                            if let Some(text) = asr::transcribe(&wav, &asr).await {
                                if !text.is_empty() {
                                    if !ip.transcript.is_empty() {
                                        ip.transcript.push(' ');
                                    }
                                    ip.transcript.push_str(&text);
                                    emitter.emit_status(RecordingStatus {
                                        is_recording: true,
                                        recording_id: Some(ip.id.clone()),
                                        transcript_preview: ip.transcript.chars().take(200).collect(),
                                    });
                                }
                            }
                        }
                    }
                }
            }

            cmd = cmd_rx.recv() => {
                match cmd {
                    None => break,
                    Some(RecorderCommand::StartManual) => {
                        if in_progress.is_none() {
                            in_progress = start_recording(&db_conn, &*emitter);
                            // Reset segmenter
                            if let Some(e) = &engine {
                                e.segmenter.lock().reset();
                            }
                        }
                    }
                    Some(RecorderCommand::StopManual) => {
                        if let Some(ip) = in_progress.take() {
                            let s = settings.lock().clone();
                            stop_recording(ip, &db_conn, &s.asr_backend, &s.summarization_backend, &*emitter, sample_rate).await;
                        }
                    }
                    Some(RecorderCommand::UpdateSettings(new_settings)) => {
                        if let Some(e) = &engine {
                            e.activity.lock().update_config(ActivityConfig {
                                threshold_dbfs: new_settings.activity_threshold_dbfs,
                                min_active_windows: new_settings.min_active_windows,
                                min_idle_windows: idle_windows(&new_settings),
                            });
                            e.segmenter.lock().update_interval(new_settings.segment_interval_secs, sample_rate);
                        }
                        *settings.lock() = new_settings;
                    }
                }
            }
        }
    }
}

fn idle_windows(s: &Settings) -> u32 {
    (s.idle_timeout_secs).max(1.0) as u32
}

pub(crate) fn start_recording(
    db_conn: &Arc<Mutex<Connection>>,
    emitter: &dyn EventEmitter,
) -> Option<InProgress> {
    let started_at = Utc::now();
    let id = {
        let conn = db_conn.lock();
        db::insert_recording(&conn, &started_at).ok()?
    };
    log::info!("Recording started: {id}");
    emitter.emit_status(RecordingStatus {
        is_recording: true,
        recording_id: Some(id.clone()),
        transcript_preview: String::new(),
    });
    emitter.emit_started(&id);
    Some(InProgress {
        id,
        started_at,
        transcript: String::new(),
        audio_buf: Vec::new(),
    })
}

pub(crate) async fn stop_recording(
    mut ip: InProgress,
    db_conn: &Arc<Mutex<Connection>>,
    asr_backend: &AsrBackend,
    sum_backend: &SummarizationBackend,
    emitter: &dyn EventEmitter,
    sample_rate: u32,
) {
    log::info!("Recording stopped: {}", ip.id);

    // Flush remaining audio
    if !ip.audio_buf.is_empty() {
        if let Ok(wav) = encode_wav(&ip.audio_buf, sample_rate) {
            if let Some(text) = asr::transcribe(&wav, asr_backend).await {
                if !text.is_empty() {
                    if !ip.transcript.is_empty() {
                        ip.transcript.push(' ');
                    }
                    ip.transcript.push_str(&text);
                }
            }
        }
    }

    let ended_at = Utc::now();

    // Summarize
    let summary = summarization::summarize(&ip.transcript, sum_backend).await;

    {
        let conn = db_conn.lock();
        let _ = db::finalize_recording(
            &conn,
            &ip.id,
            &ended_at,
            &ip.transcript,
            summary.as_deref(),
        );
    }

    emitter.emit_status(RecordingStatus {
        is_recording: false,
        recording_id: None,
        transcript_preview: String::new(),
    });
    emitter.emit_stopped(&ip.id);
}

// ── Convenience: construct recorder for the real app ─────────────────────────

/// Wraps `run_recorder` to accept a `tauri::AppHandle` directly.
/// Called from `lib.rs`.
pub async fn run_recorder_with_app(
    app: AppHandle,
    db_conn: Arc<Mutex<Connection>>,
    initial_settings: Settings,
    cmd_rx: mpsc::Receiver<RecorderCommand>,
) {
    run_recorder(Arc::new(app), db_conn, initial_settings, cmd_rx).await;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db;
    use rusqlite::Connection;
    use std::sync::Mutex as StdMutex;
    use tokio::sync::mpsc;

    // ── Mock emitter ──────────────────────────────────────────────────────────

    #[derive(Debug)]
    enum EmittedEvent {
        Status(RecordingStatus),
        Started(String),
        Stopped(String),
    }

    struct MockEmitter {
        events: StdMutex<Vec<EmittedEvent>>,
    }

    impl MockEmitter {
        fn new() -> Self {
            Self { events: StdMutex::new(Vec::new()) }
        }

        fn events(&self) -> Vec<String> {
            self.events
                .lock()
                .unwrap()
                .iter()
                .map(|e| format!("{e:?}"))
                .collect()
        }

        fn has_started(&self) -> bool {
            self.events
                .lock()
                .unwrap()
                .iter()
                .any(|e| matches!(e, EmittedEvent::Started(_)))
        }

        fn has_stopped(&self) -> bool {
            self.events
                .lock()
                .unwrap()
                .iter()
                .any(|e| matches!(e, EmittedEvent::Stopped(_)))
        }

        fn recording_ids_started(&self) -> Vec<String> {
            self.events
                .lock()
                .unwrap()
                .iter()
                .filter_map(|e| {
                    if let EmittedEvent::Started(id) = e { Some(id.clone()) } else { None }
                })
                .collect()
        }
    }

    impl EventEmitter for MockEmitter {
        fn emit_status(&self, status: RecordingStatus) {
            self.events.lock().unwrap().push(EmittedEvent::Status(status));
        }

        fn emit_started(&self, id: &str) {
            self.events.lock().unwrap().push(EmittedEvent::Started(id.to_string()));
        }

        fn emit_stopped(&self, id: &str) {
            self.events.lock().unwrap().push(EmittedEvent::Stopped(id.to_string()));
        }
    }

    // ── Helpers ───────────────────────────────────────────────────────────────

    fn in_memory_db() -> Arc<Mutex<Connection>> {
        let conn = Connection::open_in_memory().expect("in-memory DB");
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS recordings (
                id TEXT PRIMARY KEY NOT NULL,
                title TEXT NOT NULL,
                started_at TEXT NOT NULL,
                ended_at TEXT,
                transcript TEXT,
                summary TEXT,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL
            );",
        )
        .expect("create table");
        Arc::new(Mutex::new(conn))
    }

    // ── start_recording ───────────────────────────────────────────────────────

    #[test]
    fn start_recording_inserts_db_row() {
        let db = in_memory_db();
        let emitter = MockEmitter::new();

        let ip = start_recording(&db, &emitter);
        assert!(ip.is_some(), "Should return InProgress");

        let conn = db.lock();
        let recordings = db::list_recordings(&conn).expect("list");
        assert_eq!(recordings.len(), 1);
    }

    #[test]
    fn start_recording_emits_status_and_started_events() {
        let db = in_memory_db();
        let emitter = MockEmitter::new();

        let ip = start_recording(&db, &emitter).expect("InProgress");
        assert!(emitter.has_started(), "Should emit Started event");

        let ids = emitter.recording_ids_started();
        assert_eq!(ids, vec![ip.id]);
    }

    #[test]
    fn start_recording_returns_empty_transcript() {
        let db = in_memory_db();
        let emitter = MockEmitter::new();
        let ip = start_recording(&db, &emitter).expect("InProgress");
        assert!(ip.transcript.is_empty());
        assert!(ip.audio_buf.is_empty());
    }

    // ── stop_recording ────────────────────────────────────────────────────────

    #[tokio::test]
    async fn stop_recording_finalizes_db_row() {
        let db = in_memory_db();
        let emitter = MockEmitter::new();

        let ip = start_recording(&db, &emitter).expect("InProgress");
        let id = ip.id.clone();

        stop_recording(
            ip,
            &db,
            &AsrBackend::None,
            &crate::summarization::SummarizationBackend::None,
            &emitter,
            16_000,
        )
        .await;

        let conn = db.lock();
        let rec = db::get_recording(&conn, &id)
            .expect("query ok")
            .expect("record exists");
        assert!(rec.ended_at.is_some(), "ended_at should be set");
    }

    #[tokio::test]
    async fn stop_recording_emits_stopped_event() {
        let db = in_memory_db();
        let emitter = MockEmitter::new();

        let ip = start_recording(&db, &emitter).expect("InProgress");
        stop_recording(
            ip,
            &db,
            &AsrBackend::None,
            &crate::summarization::SummarizationBackend::None,
            &emitter,
            16_000,
        )
        .await;

        assert!(emitter.has_stopped(), "Should emit Stopped event");
    }

    // ── run_recorder command channel ──────────────────────────────────────────

    #[tokio::test]
    async fn start_manual_command_creates_recording() {
        let db = in_memory_db();
        let emitter = Arc::new(MockEmitter::new());
        let settings = Settings::default();
        let (tx, rx) = mpsc::channel(8);

        let db_clone = db.clone();
        let em_clone = emitter.clone();
        tokio::spawn(async move {
            run_recorder(em_clone, db_clone, settings, rx).await;
        });

        tx.send(RecorderCommand::StartManual).await.expect("send");
        // Give the loop time to process the command.
        tokio::time::sleep(Duration::from_millis(50)).await;

        let conn = db.lock();
        assert_eq!(
            db::list_recordings(&conn).expect("list").len(),
            1,
            "One recording should exist after StartManual"
        );
    }

    #[tokio::test]
    async fn stop_manual_command_finalizes_recording() {
        let db = in_memory_db();
        let emitter = Arc::new(MockEmitter::new());
        let settings = Settings::default();
        let (tx, rx) = mpsc::channel(8);

        let db_clone = db.clone();
        let em_clone = emitter.clone();
        tokio::spawn(async move {
            run_recorder(em_clone, db_clone, settings, rx).await;
        });

        tx.send(RecorderCommand::StartManual).await.expect("send start");
        tokio::time::sleep(Duration::from_millis(50)).await;

        tx.send(RecorderCommand::StopManual).await.expect("send stop");
        tokio::time::sleep(Duration::from_millis(50)).await;

        assert!(emitter.has_stopped(), "Should emit Stopped after StopManual");

        let conn = db.lock();
        let recs = db::list_recordings(&conn).expect("list");
        assert_eq!(recs.len(), 1);
        assert!(recs[0].ended_at.is_some(), "ended_at should be finalised");
    }

    #[tokio::test]
    async fn update_settings_command_does_not_crash_without_engine() {
        // When audio engine init fails (no hardware in CI), UpdateSettings
        // should still be processed without panicking.
        let db = in_memory_db();
        let emitter = Arc::new(MockEmitter::new());
        let mut settings = Settings::default();
        settings.auto_record = false; // ensure engine path is skipped cleanly
        let (tx, rx) = mpsc::channel(8);

        tokio::spawn(async move {
            run_recorder(emitter, db, settings, rx).await;
        });

        let mut new_settings = Settings::default();
        new_settings.activity_threshold_dbfs = -50.0;
        tx.send(RecorderCommand::UpdateSettings(new_settings))
            .await
            .expect("send");
        tokio::time::sleep(Duration::from_millis(50)).await;
        // If we reach here without panic, the test passes.
    }
}

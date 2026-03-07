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

struct InProgress {
    id: String,
    started_at: chrono::DateTime<Utc>,
    transcript: String,
    /// Accumulated audio since the last segment flush (mic + sys mixed).
    audio_buf: Vec<f32>,
}

pub async fn run_recorder(
    app: AppHandle,
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
            emit_status(&app, false, None, "");
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
                            in_progress = start_recording(&db_conn, &app);
                        }
                        Some(false) => {
                            if let Some(ip) = in_progress.take() {
                                stop_recording(ip, &db_conn, &s.asr_backend, &s.summarization_backend, &app, sample_rate).await;
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
                                    emit_status(&app, true, Some(ip.id.clone()), &ip.transcript);
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
                            in_progress = start_recording(&db_conn, &app);
                            // Reset segmenter
                            if let Some(e) = &engine {
                                e.segmenter.lock().reset();
                            }
                        }
                    }
                    Some(RecorderCommand::StopManual) => {
                        if let Some(ip) = in_progress.take() {
                            let s = settings.lock().clone();
                            stop_recording(ip, &db_conn, &s.asr_backend, &s.summarization_backend, &app, sample_rate).await;
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

fn start_recording(
    db_conn: &Arc<Mutex<Connection>>,
    app: &AppHandle,
) -> Option<InProgress> {
    let started_at = Utc::now();
    let id = {
        let conn = db_conn.lock();
        db::insert_recording(&conn, &started_at).ok()?
    };
    log::info!("Recording started: {id}");
    emit_status(app, true, Some(id.clone()), "");
    app.emit("recording-started", id.clone()).ok();
    Some(InProgress {
        id,
        started_at,
        transcript: String::new(),
        audio_buf: Vec::new(),
    })
}

async fn stop_recording(
    mut ip: InProgress,
    db_conn: &Arc<Mutex<Connection>>,
    asr_backend: &AsrBackend,
    sum_backend: &SummarizationBackend,
    app: &AppHandle,
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

    emit_status(app, false, None, "");
    app.emit("recording-stopped", ip.id.clone()).ok();
}

fn emit_status(app: &AppHandle, is_recording: bool, id: Option<String>, preview: &str) {
    let _ = app.emit(
        "recorder-status",
        RecordingStatus {
            is_recording,
            recording_id: id,
            transcript_preview: preview.chars().take(200).collect(),
        },
    );
}

use crate::db::{self, Recording};
use crate::recorder::RecorderCommand;
use crate::settings::Settings;
use crate::AppState;
use parking_lot::Mutex;
use rusqlite::Connection;
use std::sync::Arc;
use tauri::State;
use tokio::sync::mpsc;

type DbConn = Arc<Mutex<Connection>>;
type CmdTx = Arc<Mutex<mpsc::Sender<RecorderCommand>>>;

#[tauri::command]
pub async fn list_recordings(state: State<'_, AppState>) -> Result<Vec<Recording>, String> {
    let conn = state.db.lock();
    db::list_recordings(&conn).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_recording(id: String, state: State<'_, AppState>) -> Result<Option<Recording>, String> {
    let conn = state.db.lock();
    db::get_recording(&conn, &id).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn update_recording(
    id: String,
    title: Option<String>,
    transcript: Option<String>,
    summary: Option<String>,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let conn = state.db.lock();
    db::update_recording(
        &conn,
        &id,
        title.as_deref(),
        transcript.as_deref(),
        summary.as_deref(),
    )
    .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn delete_recording(id: String, state: State<'_, AppState>) -> Result<(), String> {
    let conn = state.db.lock();
    db::delete_recording(&conn, &id).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn start_recording(state: State<'_, AppState>) -> Result<(), String> {
    let tx = state.cmd_tx.lock();
    tx.send(RecorderCommand::StartManual)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn stop_recording(state: State<'_, AppState>) -> Result<(), String> {
    let tx = state.cmd_tx.lock();
    tx.send(RecorderCommand::StopManual)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_settings(state: State<'_, AppState>) -> Result<Settings, String> {
    Ok(state.settings.lock().clone())
}

#[tauri::command]
pub async fn save_settings(new_settings: Settings, state: State<'_, AppState>) -> Result<(), String> {
    // Persist to disk
    state
        .settings_path
        .lock()
        .as_ref()
        .map(|p| state.settings.lock().save(p))
        .transpose()
        .map_err(|e| e.to_string())?;

    // Notify recorder
    {
        let tx = state.cmd_tx.lock();
        let _ = tx
            .send(RecorderCommand::UpdateSettings(new_settings.clone()))
            .await;
    }

    *state.settings.lock() = new_settings;
    Ok(())
}

#[tauri::command]
pub async fn retry_summary(id: String, state: State<'_, AppState>) -> Result<(), String> {
    let transcript = {
        let conn = state.db.lock();
        db::get_recording(&conn, &id)
            .map_err(|e| e.to_string())?
            .and_then(|r| r.transcript)
            .ok_or_else(|| "Recording not found or has no transcript".to_string())?
    };

    let backend = state.settings.lock().summarization_backend.clone();
    let summary = crate::summarization::summarize(&transcript, &backend)
        .await
        .ok_or_else(|| "Summarization failed or backend not configured".to_string())?;

    let conn = state.db.lock();
    db::update_recording(&conn, &id, None, None, Some(&summary)).map_err(|e| e.to_string())
}

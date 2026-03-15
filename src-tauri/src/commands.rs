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

// ── Tauri command handlers ────────────────────────────────────────────────────
// Each handler is a thin delegator to the corresponding `*_inner` function so
// that the business logic can be exercised in unit tests without the Tauri
// runtime.

#[tauri::command]
pub async fn list_recordings(state: State<'_, AppState>) -> Result<Vec<Recording>, String> {
    let conn = state.db.lock();
    list_recordings_inner(&conn)
}

#[tauri::command]
pub async fn get_recording(id: String, state: State<'_, AppState>) -> Result<Option<Recording>, String> {
    let conn = state.db.lock();
    get_recording_inner(&conn, &id)
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
    update_recording_inner(&conn, &id, title.as_deref(), transcript.as_deref(), summary.as_deref())
}

#[tauri::command]
pub async fn delete_recording(id: String, state: State<'_, AppState>) -> Result<(), String> {
    let conn = state.db.lock();
    delete_recording_inner(&conn, &id)
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
    let backend = state.settings.lock().summarization_backend.clone();
    let conn = state.db.lock();
    retry_summary_inner(&conn, &backend, &id).await
}

// ── Testable inner functions ──────────────────────────────────────────────────
// These functions take concrete types instead of `tauri::State` so they can be
// called directly in `#[cfg(test)]` blocks.

pub(crate) fn list_recordings_inner(conn: &Connection) -> Result<Vec<Recording>, String> {
    db::list_recordings(conn).map_err(|e| e.to_string())
}

pub(crate) fn get_recording_inner(
    conn: &Connection,
    id: &str,
) -> Result<Option<Recording>, String> {
    db::get_recording(conn, id).map_err(|e| e.to_string())
}

pub(crate) fn update_recording_inner(
    conn: &Connection,
    id: &str,
    title: Option<&str>,
    transcript: Option<&str>,
    summary: Option<&str>,
) -> Result<(), String> {
    db::update_recording(conn, id, title, transcript, summary).map_err(|e| e.to_string())
}

pub(crate) fn delete_recording_inner(conn: &Connection, id: &str) -> Result<(), String> {
    db::delete_recording(conn, id).map_err(|e| e.to_string())
}

pub(crate) async fn retry_summary_inner(
    conn: &Connection,
    backend: &crate::summarization::SummarizationBackend,
    id: &str,
) -> Result<(), String> {
    let transcript = db::get_recording(conn, id)
        .map_err(|e| e.to_string())?
        .and_then(|r| r.transcript)
        .ok_or_else(|| "Recording not found or has no transcript".to_string())?;

    let summary = crate::summarization::summarize(&transcript, backend)
        .await
        .ok_or_else(|| "Summarization failed or backend not configured".to_string())?;

    db::update_recording(conn, id, None, None, Some(&summary)).map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db;
    use chrono::Utc;
    use rusqlite::Connection;

    fn in_memory_db() -> Connection {
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
        conn
    }

    // ── list_recordings_inner ─────────────────────────────────────────────────

    #[test]
    fn list_recordings_returns_empty_on_fresh_db() {
        let conn = in_memory_db();
        let result = list_recordings_inner(&conn).expect("ok");
        assert!(result.is_empty());
    }

    #[test]
    fn list_recordings_returns_inserted_rows() {
        let conn = in_memory_db();
        db::insert_recording(&conn, &Utc::now()).expect("insert");
        db::insert_recording(&conn, &Utc::now()).expect("insert");

        let result = list_recordings_inner(&conn).expect("ok");
        assert_eq!(result.len(), 2);
    }

    // ── get_recording_inner ───────────────────────────────────────────────────

    #[test]
    fn get_recording_returns_none_for_missing_id() {
        let conn = in_memory_db();
        let result = get_recording_inner(&conn, "no-such-id").expect("ok");
        assert!(result.is_none());
    }

    #[test]
    fn get_recording_returns_some_for_existing_id() {
        let conn = in_memory_db();
        let id = db::insert_recording(&conn, &Utc::now()).expect("insert");
        let result = get_recording_inner(&conn, &id).expect("ok");
        assert!(result.is_some());
        assert_eq!(result.unwrap().id, id);
    }

    // ── update_recording_inner ────────────────────────────────────────────────

    #[test]
    fn update_recording_changes_title() {
        let conn = in_memory_db();
        let id = db::insert_recording(&conn, &Utc::now()).expect("insert");
        update_recording_inner(&conn, &id, Some("New Title"), None, None).expect("ok");
        let rec = get_recording_inner(&conn, &id).expect("ok").expect("exists");
        assert_eq!(rec.title, "New Title");
    }

    #[test]
    fn update_recording_returns_err_for_missing_id_gracefully() {
        // rusqlite UPDATE with no matching row succeeds (0 rows affected); this
        // is intentional – callers should not treat a no-op as an error.
        let conn = in_memory_db();
        let result = update_recording_inner(&conn, "ghost", Some("X"), None, None);
        assert!(result.is_ok());
    }

    // ── delete_recording_inner ────────────────────────────────────────────────

    #[test]
    fn delete_recording_removes_the_row() {
        let conn = in_memory_db();
        let id = db::insert_recording(&conn, &Utc::now()).expect("insert");
        delete_recording_inner(&conn, &id).expect("ok");
        let result = get_recording_inner(&conn, &id).expect("ok");
        assert!(result.is_none());
    }

    #[test]
    fn delete_recording_nonexistent_id_is_ok() {
        let conn = in_memory_db();
        // Should not return an error for a missing ID.
        let result = delete_recording_inner(&conn, "no-such-id");
        assert!(result.is_ok());
    }

    // ── retry_summary_inner ───────────────────────────────────────────────────

    #[tokio::test]
    async fn retry_summary_returns_err_when_recording_missing() {
        let conn = in_memory_db();
        let backend = crate::summarization::SummarizationBackend::None;
        let result = retry_summary_inner(&conn, &backend, "missing").await;
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("not found"));
    }

    #[tokio::test]
    async fn retry_summary_returns_err_when_transcript_is_null() {
        let conn = in_memory_db();
        let id = db::insert_recording(&conn, &Utc::now()).expect("insert");
        // transcript is NULL at this point (only inserted, not finalized)
        let backend = crate::summarization::SummarizationBackend::None;
        let result = retry_summary_inner(&conn, &backend, &id).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("no transcript"));
    }

    #[tokio::test]
    async fn retry_summary_returns_err_when_backend_is_none() {
        let conn = in_memory_db();
        let id = db::insert_recording(&conn, &Utc::now()).expect("insert");
        // Finalize with a transcript so the recording is ready for summarization.
        db::finalize_recording(&conn, &id, &Utc::now(), "Hello world", None).expect("finalize");

        let backend = crate::summarization::SummarizationBackend::None;
        let result = retry_summary_inner(&conn, &backend, &id).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Summarization failed"));
    }
}

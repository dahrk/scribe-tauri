use anyhow::Result;
use chrono::{DateTime, Utc};
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use std::path::Path;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Recording {
    pub id: String,
    pub title: String,
    pub started_at: String,
    pub ended_at: Option<String>,
    pub transcript: Option<String>,
    pub summary: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

pub fn open(db_path: &Path) -> Result<Connection> {
    let conn = Connection::open(db_path)?;
    conn.execute_batch("PRAGMA journal_mode=WAL;")?;
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS recordings (
            id          TEXT PRIMARY KEY NOT NULL,
            title       TEXT NOT NULL,
            started_at  TEXT NOT NULL,
            ended_at    TEXT,
            transcript  TEXT,
            summary     TEXT,
            created_at  TEXT NOT NULL,
            updated_at  TEXT NOT NULL
        );",
    )?;
    Ok(conn)
}

pub fn insert_recording(conn: &Connection, started_at: &DateTime<Utc>) -> Result<String> {
    let id = Uuid::new_v4().to_string();
    let now = Utc::now().to_rfc3339();
    let title = format!(
        "Meeting {}",
        started_at.format("%Y-%m-%d %H:%M")
    );
    conn.execute(
        "INSERT INTO recordings (id, title, started_at, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?4)",
        params![id, title, started_at.to_rfc3339(), now],
    )?;
    Ok(id)
}

pub fn finalize_recording(
    conn: &Connection,
    id: &str,
    ended_at: &DateTime<Utc>,
    transcript: &str,
    summary: Option<&str>,
) -> Result<()> {
    let now = Utc::now().to_rfc3339();
    conn.execute(
        "UPDATE recordings SET ended_at=?1, transcript=?2, summary=?3, updated_at=?4 WHERE id=?5",
        params![ended_at.to_rfc3339(), transcript, summary, now, id],
    )?;
    Ok(())
}

pub fn list_recordings(conn: &Connection) -> Result<Vec<Recording>> {
    let mut stmt = conn.prepare(
        "SELECT id, title, started_at, ended_at, transcript, summary, created_at, updated_at
         FROM recordings ORDER BY started_at DESC",
    )?;
    let rows = stmt.query_map([], |row| {
        Ok(Recording {
            id: row.get(0)?,
            title: row.get(1)?,
            started_at: row.get(2)?,
            ended_at: row.get(3)?,
            transcript: row.get(4)?,
            summary: row.get(5)?,
            created_at: row.get(6)?,
            updated_at: row.get(7)?,
        })
    })?;
    let mut out = Vec::new();
    for r in rows {
        out.push(r?);
    }
    Ok(out)
}

pub fn get_recording(conn: &Connection, id: &str) -> Result<Option<Recording>> {
    let mut stmt = conn.prepare(
        "SELECT id, title, started_at, ended_at, transcript, summary, created_at, updated_at
         FROM recordings WHERE id=?1",
    )?;
    let mut rows = stmt.query_map(params![id], |row| {
        Ok(Recording {
            id: row.get(0)?,
            title: row.get(1)?,
            started_at: row.get(2)?,
            ended_at: row.get(3)?,
            transcript: row.get(4)?,
            summary: row.get(5)?,
            created_at: row.get(6)?,
            updated_at: row.get(7)?,
        })
    })?;
    Ok(rows.next().transpose()?)
}

pub fn update_recording(
    conn: &Connection,
    id: &str,
    title: Option<&str>,
    transcript: Option<&str>,
    summary: Option<&str>,
) -> Result<()> {
    let now = Utc::now().to_rfc3339();
    if let Some(t) = title {
        conn.execute(
            "UPDATE recordings SET title=?1, updated_at=?2 WHERE id=?3",
            params![t, now, id],
        )?;
    }
    if let Some(t) = transcript {
        conn.execute(
            "UPDATE recordings SET transcript=?1, updated_at=?2 WHERE id=?3",
            params![t, now, id],
        )?;
    }
    if let Some(s) = summary {
        conn.execute(
            "UPDATE recordings SET summary=?1, updated_at=?2 WHERE id=?3",
            params![s, now, id],
        )?;
    }
    Ok(())
}

pub fn delete_recording(conn: &Connection, id: &str) -> Result<()> {
    conn.execute("DELETE FROM recordings WHERE id=?1", params![id])?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    fn in_memory_db() -> Connection {
        let conn = Connection::open_in_memory().expect("in-memory DB");
        conn.execute_batch("PRAGMA journal_mode=WAL;").ok();
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS recordings (
                id          TEXT PRIMARY KEY NOT NULL,
                title       TEXT NOT NULL,
                started_at  TEXT NOT NULL,
                ended_at    TEXT,
                transcript  TEXT,
                summary     TEXT,
                created_at  TEXT NOT NULL,
                updated_at  TEXT NOT NULL
            );",
        )
        .expect("schema");
        conn
    }

    // ── insert_recording ──────────────────────────────────────────────────────

    #[test]
    fn insert_returns_uuid() {
        let conn = in_memory_db();
        let started = Utc::now();
        let id = insert_recording(&conn, &started).expect("insert");
        assert!(!id.is_empty());
        // Should be a valid UUID
        assert_eq!(id.len(), 36);
    }

    #[test]
    fn insert_sets_autogenerated_title() {
        let conn = in_memory_db();
        let started = Utc::now();
        let id = insert_recording(&conn, &started).expect("insert");
        let rec = get_recording(&conn, &id).expect("get").expect("exists");
        assert!(rec.title.starts_with("Meeting "));
    }

    #[test]
    fn insert_leaves_ended_at_null() {
        let conn = in_memory_db();
        let id = insert_recording(&conn, &Utc::now()).expect("insert");
        let rec = get_recording(&conn, &id).expect("get").expect("exists");
        assert!(rec.ended_at.is_none());
    }

    // ── finalize_recording ────────────────────────────────────────────────────

    #[test]
    fn finalize_sets_transcript_and_summary() {
        let conn = in_memory_db();
        let id = insert_recording(&conn, &Utc::now()).expect("insert");
        finalize_recording(&conn, &id, &Utc::now(), "Hello world", Some("Summary here"))
            .expect("finalize");
        let rec = get_recording(&conn, &id).expect("get").expect("exists");
        assert_eq!(rec.transcript.as_deref(), Some("Hello world"));
        assert_eq!(rec.summary.as_deref(), Some("Summary here"));
        assert!(rec.ended_at.is_some());
    }

    #[test]
    fn finalize_without_summary() {
        let conn = in_memory_db();
        let id = insert_recording(&conn, &Utc::now()).expect("insert");
        finalize_recording(&conn, &id, &Utc::now(), "text", None).expect("finalize");
        let rec = get_recording(&conn, &id).expect("get").expect("exists");
        assert!(rec.summary.is_none());
        assert_eq!(rec.transcript.as_deref(), Some("text"));
    }

    // ── list_recordings ───────────────────────────────────────────────────────

    #[test]
    fn list_empty_returns_empty_vec() {
        let conn = in_memory_db();
        let list = list_recordings(&conn).expect("list");
        assert!(list.is_empty());
    }

    #[test]
    fn list_returns_all_recordings_newest_first() {
        let conn = in_memory_db();
        let id1 = insert_recording(&conn, &Utc::now()).expect("a");
        // tiny sleep to guarantee different started_at
        std::thread::sleep(std::time::Duration::from_millis(5));
        let id2 = insert_recording(&conn, &Utc::now()).expect("b");
        let list = list_recordings(&conn).expect("list");
        assert_eq!(list.len(), 2);
        // Newest first
        assert_eq!(list[0].id, id2);
        assert_eq!(list[1].id, id1);
    }

    // ── get_recording ─────────────────────────────────────────────────────────

    #[test]
    fn get_nonexistent_returns_none() {
        let conn = in_memory_db();
        let r = get_recording(&conn, "no-such-id").expect("get");
        assert!(r.is_none());
    }

    #[test]
    fn get_existing_returns_some() {
        let conn = in_memory_db();
        let id = insert_recording(&conn, &Utc::now()).expect("insert");
        let r = get_recording(&conn, &id).expect("get");
        assert!(r.is_some());
        assert_eq!(r.unwrap().id, id);
    }

    // ── update_recording ──────────────────────────────────────────────────────

    #[test]
    fn update_title() {
        let conn = in_memory_db();
        let id = insert_recording(&conn, &Utc::now()).expect("insert");
        update_recording(&conn, &id, Some("New Title"), None, None).expect("update");
        let rec = get_recording(&conn, &id).expect("get").expect("exists");
        assert_eq!(rec.title, "New Title");
    }

    #[test]
    fn update_transcript_only() {
        let conn = in_memory_db();
        let id = insert_recording(&conn, &Utc::now()).expect("insert");
        update_recording(&conn, &id, None, Some("transcript text"), None).expect("update");
        let rec = get_recording(&conn, &id).expect("get").expect("exists");
        assert_eq!(rec.transcript.as_deref(), Some("transcript text"));
    }

    #[test]
    fn update_summary_only() {
        let conn = in_memory_db();
        let id = insert_recording(&conn, &Utc::now()).expect("insert");
        update_recording(&conn, &id, None, None, Some("summary")).expect("update");
        let rec = get_recording(&conn, &id).expect("get").expect("exists");
        assert_eq!(rec.summary.as_deref(), Some("summary"));
    }

    #[test]
    fn update_none_fields_is_noop() {
        let conn = in_memory_db();
        let id = insert_recording(&conn, &Utc::now()).expect("insert");
        let before = get_recording(&conn, &id).expect("get").expect("exists");
        update_recording(&conn, &id, None, None, None).expect("update");
        let after = get_recording(&conn, &id).expect("get").expect("exists");
        assert_eq!(before.title, after.title);
    }

    // ── delete_recording ──────────────────────────────────────────────────────

    #[test]
    fn delete_removes_record() {
        let conn = in_memory_db();
        let id = insert_recording(&conn, &Utc::now()).expect("insert");
        delete_recording(&conn, &id).expect("delete");
        let r = get_recording(&conn, &id).expect("get");
        assert!(r.is_none());
    }

    #[test]
    fn delete_nonexistent_is_ok() {
        let conn = in_memory_db();
        // Should not error even if the row doesn't exist
        delete_recording(&conn, "phantom-id").expect("delete");
    }

    #[test]
    fn delete_one_of_two_leaves_the_other() {
        let conn = in_memory_db();
        let id1 = insert_recording(&conn, &Utc::now()).expect("a");
        let id2 = insert_recording(&conn, &Utc::now()).expect("b");
        delete_recording(&conn, &id1).expect("delete");
        assert!(get_recording(&conn, &id1).expect("get").is_none());
        assert!(get_recording(&conn, &id2).expect("get").is_some());
    }
}

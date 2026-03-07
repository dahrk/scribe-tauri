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

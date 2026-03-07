# Database Design

## Why SQLite?

- Single-file, zero-server, embeds cleanly into a Tauri bundle via
  `rusqlite` with the `bundled` feature.
- WAL mode (`PRAGMA journal_mode=WAL`) allows concurrent reads from the
  frontend while the recorder writes, without blocking the UI.
- A single `recordings` table is sufficient; no relational joins needed.

## Schema

```sql
CREATE TABLE recordings (
    id          TEXT PRIMARY KEY NOT NULL,  -- UUID v4
    title       TEXT NOT NULL,              -- Editable by user
    started_at  TEXT NOT NULL,              -- RFC-3339 UTC
    ended_at    TEXT,                       -- NULL while in progress
    transcript  TEXT,                       -- Flat text, appended per segment
    summary     TEXT,                       -- NULL until summarization completes
    created_at  TEXT NOT NULL,
    updated_at  TEXT NOT NULL
);
```

## Trade-offs accepted

| Decision | Trade-off |
|---|---|
| Flat transcript TEXT blob | Simple; sufficient for ≤100 KB. Very long meetings may be slow to update. Consider chunking if transcript exceeds 100 KB. |
| No audio storage | Privacy win. Cannot replay audio after the fact. |
| No segments table | Simpler CRUD. Cannot re-transcribe individual segments without re-recording. |
| RFC-3339 UTC strings | Human-readable in SQLite Browser. Slightly larger than integer epoch. |

## Synchronous operations

`rusqlite` is a synchronous API. All DB calls in command handlers are wrapped
in `tokio::task::spawn_blocking` to avoid blocking the async runtime. Tests use
`Connection::open_in_memory()` – no temp files needed.

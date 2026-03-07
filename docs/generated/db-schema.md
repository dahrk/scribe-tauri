# Database Schema (generated reference)

> This file is the canonical human-readable schema. Update whenever
> `db.rs` migration SQL changes.

## Table: `recordings`

| Column | Type | Nullable | Notes |
|---|---|---|---|
| `id` | TEXT | NOT NULL | UUID v4, primary key |
| `title` | TEXT | NOT NULL | Default: `"Meeting YYYY-MM-DD HH:MM"` |
| `started_at` | TEXT | NOT NULL | RFC-3339 UTC |
| `ended_at` | TEXT | YES | NULL while recording is in progress |
| `transcript` | TEXT | YES | Running transcript; appended per segment |
| `summary` | TEXT | YES | LLM summary; NULL if unavailable |
| `created_at` | TEXT | NOT NULL | RFC-3339 UTC, set on insert |
| `updated_at` | TEXT | NOT NULL | RFC-3339 UTC, updated on any change |

## Indexes

None beyond the primary key. Query patterns are all `WHERE id = ?` or
full-table `ORDER BY started_at DESC`; no secondary indexes needed at
current scale.

## Pragmas applied at open

```sql
PRAGMA journal_mode=WAL;
```

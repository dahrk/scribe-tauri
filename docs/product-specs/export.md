# Product Spec: Export (Planned)

## Status: Not yet implemented

## Summary

Allow users to export a recording's transcript and summary to a portable
format (Markdown, PDF).

## Proposed UX

A "Export" button in `RecordingDetail` opens a native save-file dialog.

## Proposed formats

| Format | Notes |
|---|---|
| Markdown (.md) | Title as H1, summary as bullet list, transcript as blockquote. Useful for pasting into Notion / Obsidian. |
| PDF | Rendered via a headless HTML→PDF approach (e.g. `tauri-plugin-shell` + `wkhtmltopdf`) or a pure-Rust PDF crate. |

## Implementation notes

- New Tauri command: `export_recording(id, format)` returning a file path.
- File dialog: `tauri-plugin-dialog`.
- No changes to the DB schema needed.

import React, { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import type { Recording } from "../types";

interface Props {
  recording: Recording;
  onDeleted: () => void;
  onUpdated: (r: Recording) => void;
}

export function RecordingDetail({ recording, onDeleted, onUpdated }: Props) {
  const [title, setTitle] = useState(recording.title);
  const [transcript, setTranscript] = useState(recording.transcript ?? "");
  const [summary, setSummary] = useState(recording.summary ?? "");
  const [saving, setSaving] = useState(false);
  const [retrying, setRetrying] = useState(false);
  const [error, setError] = useState<string | null>(null);

  // Sync when a different recording is selected
  useEffect(() => {
    setTitle(recording.title);
    setTranscript(recording.transcript ?? "");
    setSummary(recording.summary ?? "");
    setError(null);
  }, [recording.id]);

  async function handleSave() {
    setSaving(true);
    setError(null);
    try {
      await invoke("update_recording", {
        id: recording.id,
        title: title !== recording.title ? title : null,
        transcript: transcript !== (recording.transcript ?? "") ? transcript : null,
        summary: summary !== (recording.summary ?? "") ? summary : null,
      });
      onUpdated({ ...recording, title, transcript, summary });
    } catch (e: any) {
      setError(String(e));
    } finally {
      setSaving(false);
    }
  }

  async function handleDelete() {
    if (!confirm(`Delete "${title}"? This cannot be undone.`)) return;
    try {
      await invoke("delete_recording", { id: recording.id });
      onDeleted();
    } catch (e: any) {
      setError(String(e));
    }
  }

  async function handleRetrySummary() {
    setRetrying(true);
    setError(null);
    try {
      await invoke("retry_summary", { id: recording.id });
      const updated = await invoke<Recording>("get_recording", { id: recording.id });
      if (updated) {
        setSummary(updated.summary ?? "");
        onUpdated(updated);
      }
    } catch (e: any) {
      setError(String(e));
    } finally {
      setRetrying(false);
    }
  }

  return (
    <div className="detail">
      <div className="detail__header">
        <input
          className="detail__title-input"
          value={title}
          onChange={(e) => setTitle(e.target.value)}
          placeholder="Recording title"
          aria-label="Recording title"
        />
        <div className="detail__header-actions">
          <button
            className="btn btn--primary"
            onClick={handleSave}
            disabled={saving}
          >
            {saving ? "Saving…" : "Save"}
          </button>
          <button className="btn btn--ghost btn--danger" onClick={handleDelete}>
            Delete
          </button>
        </div>
      </div>

      {error && <div className="alert alert--error">{error}</div>}

      <section className="detail__section">
        <h3 className="detail__section-title">Summary</h3>
        {summary ? (
          <textarea
            className="detail__textarea"
            value={summary}
            onChange={(e) => setSummary(e.target.value)}
            rows={6}
          />
        ) : (
          <p className="text-muted">
            No summary available.{" "}
            {retrying ? (
              <span>Generating…</span>
            ) : (
              <button className="link-btn" onClick={handleRetrySummary}>
                Generate summary
              </button>
            )}
          </p>
        )}
        {summary && (
          <button
            className="btn btn--ghost btn--sm"
            onClick={handleRetrySummary}
            disabled={retrying}
          >
            {retrying ? "Regenerating…" : "Regenerate summary"}
          </button>
        )}
      </section>

      <section className="detail__section">
        <h3 className="detail__section-title">Transcript</h3>
        {transcript ? (
          <textarea
            className="detail__textarea detail__textarea--tall"
            value={transcript}
            onChange={(e) => setTranscript(e.target.value)}
            rows={16}
          />
        ) : (
          <p className="text-muted">No transcript yet.</p>
        )}
      </section>
    </div>
  );
}

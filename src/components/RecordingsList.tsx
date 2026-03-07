import React, { useEffect, useState, useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import type { Recording } from "../types";

interface Props {
  onSelect: (r: Recording) => void;
  selectedId: string | null;
}

function formatDate(iso: string) {
  return new Date(iso).toLocaleString(undefined, {
    year: "numeric",
    month: "short",
    day: "numeric",
    hour: "2-digit",
    minute: "2-digit",
  });
}

function duration(r: Recording) {
  if (!r.ended_at) return "—";
  const ms = new Date(r.ended_at).getTime() - new Date(r.started_at).getTime();
  const mins = Math.floor(ms / 60_000);
  const secs = Math.floor((ms % 60_000) / 1000);
  return `${mins}m ${secs}s`;
}

export function RecordingsList({ onSelect, selectedId }: Props) {
  const [recordings, setRecordings] = useState<Recording[]>([]);

  const refresh = useCallback(() => {
    invoke<Recording[]>("list_recordings")
      .then(setRecordings)
      .catch(console.error);
  }, []);

  useEffect(() => {
    refresh();
    const unsub1 = listen("recording-stopped", () => refresh());
    const unsub2 = listen("recording-started", () => refresh());
    return () => {
      unsub1.then((fn) => fn());
      unsub2.then((fn) => fn());
    };
  }, [refresh]);

  if (recordings.length === 0) {
    return (
      <div className="empty-state">
        <p>No recordings yet.</p>
        <p className="text-muted">Start a meeting or enable auto-record in Settings.</p>
      </div>
    );
  }

  return (
    <ul className="recordings-list">
      {recordings.map((r) => (
        <li
          key={r.id}
          className={`recordings-list__item ${r.id === selectedId ? "recordings-list__item--selected" : ""}`}
          onClick={() => onSelect(r)}
        >
          <div className="recordings-list__title">{r.title}</div>
          <div className="recordings-list__meta">
            <span>{formatDate(r.started_at)}</span>
            <span className="recordings-list__duration">{duration(r)}</span>
          </div>
          {r.summary && (
            <div className="recordings-list__summary-hint">
              {r.summary.slice(0, 80)}…
            </div>
          )}
        </li>
      ))}
    </ul>
  );
}

import React from "react";
import type { RecorderStatus } from "../types";

interface Props {
  status: RecorderStatus;
  onStart: () => void;
  onStop: () => void;
}

export function RecordingIndicator({ status, onStart, onStop }: Props) {
  return (
    <div className={`rec-bar ${status.is_recording ? "rec-bar--active" : ""}`}>
      <span className="rec-bar__dot" aria-hidden="true" />
      <span className="rec-bar__label">
        {status.is_recording ? "Recording…" : "Idle"}
      </span>
      {status.is_recording && status.transcript_preview && (
        <span className="rec-bar__preview" title="Live transcript preview">
          {status.transcript_preview}
        </span>
      )}
      <div className="rec-bar__actions">
        {status.is_recording ? (
          <button className="btn btn--danger" onClick={onStop}>
            Stop
          </button>
        ) : (
          <button className="btn btn--primary" onClick={onStart}>
            Start
          </button>
        )}
      </div>
    </div>
  );
}

import { useState } from "react";
import { RecordingIndicator } from "./components/RecordingIndicator";
import { RecordingsList } from "./components/RecordingsList";
import { RecordingDetail } from "./components/RecordingDetail";
import { SettingsPanel } from "./components/SettingsPanel";
import { useRecorder } from "./hooks/useRecorder";
import type { Recording } from "./types";
import "./App.css";

type View = "recordings" | "settings";

function App() {
  const [view, setView] = useState<View>("recordings");
  const [selectedRecording, setSelectedRecording] = useState<Recording | null>(null);
  const { status, startRecording, stopRecording } = useRecorder();

  function handleSelectRecording(r: Recording) {
    setSelectedRecording(r);
    setView("recordings");
  }

  function handleDeleted() {
    setSelectedRecording(null);
  }

  function handleUpdated(r: Recording) {
    setSelectedRecording(r);
  }

  return (
    <div className="app">
      <RecordingIndicator
        status={status}
        onStart={startRecording}
        onStop={stopRecording}
      />

      <div className="app__layout">
        <nav className="sidebar">
          <div className="sidebar__logo">Scribe</div>
          <ul className="sidebar__nav">
            <li>
              <button
                className={`sidebar__item ${view === "recordings" ? "sidebar__item--active" : ""}`}
                onClick={() => setView("recordings")}
              >
                Recordings
              </button>
            </li>
            <li>
              <button
                className={`sidebar__item ${view === "settings" ? "sidebar__item--active" : ""}`}
                onClick={() => setView("settings")}
              >
                Settings
              </button>
            </li>
          </ul>
        </nav>

        {view === "recordings" ? (
          <div className="recordings-layout">
            <aside className="recordings-layout__list">
              <RecordingsList
                onSelect={handleSelectRecording}
                selectedId={selectedRecording?.id ?? null}
              />
            </aside>
            <main className="recordings-layout__detail">
              {selectedRecording ? (
                <RecordingDetail
                  recording={selectedRecording}
                  onDeleted={handleDeleted}
                  onUpdated={handleUpdated}
                />
              ) : (
                <div className="empty-state">
                  <p>Select a recording to view details.</p>
                </div>
              )}
            </main>
          </div>
        ) : (
          <main className="settings-layout">
            <SettingsPanel />
          </main>
        )}
      </div>
    </div>
  );
}

export default App;

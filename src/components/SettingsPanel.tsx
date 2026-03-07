import React, { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import type { Settings, AsrBackend, SummarizationBackend } from "../types";

const DEFAULT_SETTINGS: Settings = {
  auto_record: true,
  idle_timeout_secs: 180,
  segment_interval_secs: 60,
  activity_threshold_dbfs: -40,
  min_active_windows: 3,
  asr_backend: {
    Parakeet: { url: "http://127.0.0.1:9000", model: "parakeet-tdt-0.6b-v2" },
  },
  summarization_backend: {
    Ollama: { base_url: "http://localhost:11434", model: "llama3.2:3b" },
  },
};

function asrBackendType(b: AsrBackend): string {
  if (b === "None") return "none";
  if ("Parakeet" in b) return "parakeet";
  if ("WhisperCli" in b) return "whisper";
  if ("HttpServer" in b) return "http";
  return "none";
}

function sumBackendType(b: SummarizationBackend): string {
  if (b === "None") return "none";
  if ("Ollama" in b) return "ollama";
  if ("OpenAiCompatible" in b) return "openai";
  return "none";
}

export function SettingsPanel() {
  const [settings, setSettings] = useState<Settings>(DEFAULT_SETTINGS);
  const [saving, setSaving] = useState(false);
  const [saved, setSaved] = useState(false);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    invoke<Settings>("get_settings")
      .then(setSettings)
      .catch(console.error);
  }, []);

  function setField<K extends keyof Settings>(key: K, value: Settings[K]) {
    setSettings((s) => ({ ...s, [key]: value }));
    setSaved(false);
  }

  async function handleSave() {
    setSaving(true);
    setError(null);
    try {
      await invoke("save_settings", { newSettings: settings });
      setSaved(true);
    } catch (e: any) {
      setError(String(e));
    } finally {
      setSaving(false);
    }
  }

  const asrType = asrBackendType(settings.asr_backend);
  const sumType = sumBackendType(settings.summarization_backend);

  return (
    <div className="settings">
      <h2 className="settings__title">Settings</h2>

      {error && <div className="alert alert--error">{error}</div>}
      {saved && <div className="alert alert--success">Settings saved.</div>}

      <section className="settings__section">
        <h3>Recording</h3>

        <label className="settings__row settings__row--toggle">
          <span>Auto-record meetings</span>
          <input
            type="checkbox"
            checked={settings.auto_record}
            onChange={(e) => setField("auto_record", e.target.checked)}
          />
        </label>

        <label className="settings__row">
          <span>Idle timeout (seconds)</span>
          <input
            type="number"
            min={30}
            max={3600}
            value={settings.idle_timeout_secs}
            onChange={(e) => setField("idle_timeout_secs", Number(e.target.value))}
          />
        </label>

        <label className="settings__row">
          <span>Transcription interval (seconds)</span>
          <input
            type="number"
            min={10}
            max={600}
            value={settings.segment_interval_secs}
            onChange={(e) => setField("segment_interval_secs", Number(e.target.value))}
          />
        </label>

        <label className="settings__row">
          <span>Activity threshold (dBFS)</span>
          <input
            type="number"
            min={-80}
            max={0}
            value={settings.activity_threshold_dbfs}
            onChange={(e) =>
              setField("activity_threshold_dbfs", Number(e.target.value))
            }
          />
        </label>

        <label className="settings__row">
          <span>Min active windows before start</span>
          <input
            type="number"
            min={1}
            max={30}
            value={settings.min_active_windows}
            onChange={(e) => setField("min_active_windows", Number(e.target.value))}
          />
        </label>
      </section>

      <section className="settings__section">
        <h3>Transcription (ASR)</h3>
        <label className="settings__row">
          <span>Backend</span>
          <select
            value={asrType}
            onChange={(e) => {
              const v = e.target.value;
              if (v === "none") setField("asr_backend", "None");
              else if (v === "parakeet")
                setField("asr_backend", {
                  Parakeet: { url: "http://127.0.0.1:9000", model: "parakeet-tdt-0.6b-v2" },
                });
              else if (v === "whisper")
                setField("asr_backend", {
                  WhisperCli: { bin: "whisper", model: "base" },
                });
              else if (v === "http")
                setField("asr_backend", { HttpServer: { url: "http://localhost:9000/transcribe" } });
            }}
          >
            <option value="parakeet">Parakeet (recommended)</option>
            <option value="whisper">Whisper CLI (fallback)</option>
            <option value="http">HTTP server</option>
            <option value="none">None (disabled)</option>
          </select>
        </label>

        {asrType === "parakeet" && "Parakeet" in settings.asr_backend && (
          <>
            <label className="settings__row">
              <span>Server URL</span>
              <input
                type="text"
                value={(settings.asr_backend as any).Parakeet.url}
                onChange={(e) =>
                  setField("asr_backend", {
                    Parakeet: {
                      ...(settings.asr_backend as any).Parakeet,
                      url: e.target.value,
                    },
                  })
                }
              />
            </label>
            <label className="settings__row">
              <span>Model</span>
              <select
                value={(settings.asr_backend as any).Parakeet.model}
                onChange={(e) =>
                  setField("asr_backend", {
                    Parakeet: {
                      ...(settings.asr_backend as any).Parakeet,
                      model: e.target.value,
                    },
                  })
                }
              >
                <option value="parakeet-tdt-0.6b-v2">parakeet-tdt-0.6b-v2 (recommended)</option>
                <option value="parakeet-tdt-1.1b">parakeet-tdt-1.1b</option>
                <option value="parakeet-ctc-0.6b">parakeet-ctc-0.6b</option>
              </select>
            </label>
            <p className="text-muted settings__hint">
              Start the server: <code>python scripts/parakeet_server.py</code>
            </p>
          </>
        )}

        {asrType === "whisper" && "WhisperCli" in settings.asr_backend && (
          <>
            <label className="settings__row">
              <span>Whisper binary path</span>
              <input
                type="text"
                value={(settings.asr_backend as any).WhisperCli.bin}
                onChange={(e) =>
                  setField("asr_backend", {
                    WhisperCli: {
                      ...(settings.asr_backend as any).WhisperCli,
                      bin: e.target.value,
                    },
                  })
                }
              />
            </label>
            <label className="settings__row">
              <span>Model</span>
              <select
                value={(settings.asr_backend as any).WhisperCli.model}
                onChange={(e) =>
                  setField("asr_backend", {
                    WhisperCli: {
                      ...(settings.asr_backend as any).WhisperCli,
                      model: e.target.value,
                    },
                  })
                }
              >
                <option value="tiny">tiny</option>
                <option value="base">base</option>
                <option value="small">small</option>
                <option value="medium">medium</option>
                <option value="large">large</option>
              </select>
            </label>
          </>
        )}

        {asrType === "http" && "HttpServer" in settings.asr_backend && (
          <label className="settings__row">
            <span>Server URL</span>
            <input
              type="text"
              value={(settings.asr_backend as any).HttpServer.url}
              onChange={(e) =>
                setField("asr_backend", { HttpServer: { url: e.target.value } })
              }
            />
          </label>
        )}

        {asrType === "none" && (
          <p className="text-muted settings__hint">
            Transcription is disabled. Run{" "}
            <code>python scripts/parakeet_server.py</code> and select Parakeet,
            or install Whisper CLI as a fallback.
          </p>
        )}
      </section>

      <section className="settings__section">
        <h3>Summarization</h3>
        <label className="settings__row">
          <span>Backend</span>
          <select
            value={sumType}
            onChange={(e) => {
              const v = e.target.value;
              if (v === "none") setField("summarization_backend", "None");
              else if (v === "ollama")
                setField("summarization_backend", {
                  Ollama: { base_url: "http://localhost:11434", model: "llama3.2:3b" },
                });
              else if (v === "openai")
                setField("summarization_backend", {
                  OpenAiCompatible: {
                    base_url: "http://localhost:11434/v1",
                    model: "llama3.2:3b",
                    api_key: null,
                  },
                });
            }}
          >
            <option value="none">None (disabled)</option>
            <option value="ollama">Ollama</option>
            <option value="openai">OpenAI-compatible</option>
          </select>
        </label>

        {sumType === "ollama" && "Ollama" in settings.summarization_backend && (
          <>
            <label className="settings__row">
              <span>Ollama base URL</span>
              <input
                type="text"
                value={(settings.summarization_backend as any).Ollama.base_url}
                onChange={(e) =>
                  setField("summarization_backend", {
                    Ollama: {
                      ...(settings.summarization_backend as any).Ollama,
                      base_url: e.target.value,
                    },
                  })
                }
              />
            </label>
            <label className="settings__row">
              <span>Model</span>
              <input
                type="text"
                value={(settings.summarization_backend as any).Ollama.model}
                onChange={(e) =>
                  setField("summarization_backend", {
                    Ollama: {
                      ...(settings.summarization_backend as any).Ollama,
                      model: e.target.value,
                    },
                  })
                }
              />
            </label>
          </>
        )}

        {sumType === "openai" &&
          "OpenAiCompatible" in settings.summarization_backend && (
            <>
              <label className="settings__row">
                <span>Base URL</span>
                <input
                  type="text"
                  value={
                    (settings.summarization_backend as any).OpenAiCompatible.base_url
                  }
                  onChange={(e) =>
                    setField("summarization_backend", {
                      OpenAiCompatible: {
                        ...(settings.summarization_backend as any).OpenAiCompatible,
                        base_url: e.target.value,
                      },
                    })
                  }
                />
              </label>
              <label className="settings__row">
                <span>Model</span>
                <input
                  type="text"
                  value={
                    (settings.summarization_backend as any).OpenAiCompatible.model
                  }
                  onChange={(e) =>
                    setField("summarization_backend", {
                      OpenAiCompatible: {
                        ...(settings.summarization_backend as any).OpenAiCompatible,
                        model: e.target.value,
                      },
                    })
                  }
                />
              </label>
              <label className="settings__row">
                <span>API Key (optional)</span>
                <input
                  type="password"
                  value={
                    (settings.summarization_backend as any).OpenAiCompatible.api_key ?? ""
                  }
                  onChange={(e) =>
                    setField("summarization_backend", {
                      OpenAiCompatible: {
                        ...(settings.summarization_backend as any).OpenAiCompatible,
                        api_key: e.target.value || null,
                      },
                    })
                  }
                />
              </label>
            </>
          )}
      </section>

      <div className="settings__footer">
        <button className="btn btn--primary" onClick={handleSave} disabled={saving}>
          {saving ? "Saving…" : "Save settings"}
        </button>
      </div>
    </div>
  );
}

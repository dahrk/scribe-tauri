use serde::Deserialize;

/// Backend options for ASR.
///
/// The **recommended** backend is `Parakeet` – start the bundled
/// `scripts/parakeet_server.py` helper and point this at it.
/// `WhisperCli` is kept as a fallback for machines without a GPU.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq)]
pub enum AsrBackend {
    /// NVIDIA Parakeet v3 via the bundled `scripts/parakeet_server.py` HTTP server.
    /// Accepts POST /transcribe with raw WAV bytes; returns `{"text":"…"}`.
    /// Default URL: http://127.0.0.1:9000
    Parakeet { url: String, model: String },

    /// Legacy Whisper CLI – slower cold-start, no GPU required.
    /// Uses the `whisper` binary installed on PATH or at `bin`.
    WhisperCli { bin: String, model: String },

    /// Generic HTTP server – any server that accepts POST /transcribe
    /// (WAV body) and returns `{"text":"…"}`.
    HttpServer { url: String },

    /// ASR disabled – recordings are stored without transcription.
    None,
}

impl Default for AsrBackend {
    fn default() -> Self {
        Self::Parakeet {
            url: "http://127.0.0.1:9000".into(),
            model: "parakeet-tdt-0.6b-v2".into(),
        }
    }
}

#[derive(Deserialize)]
struct AsrHttpResponse {
    text: String,
}

/// Transcribe a WAV byte blob using the configured backend.
/// Returns `None` if the backend is `None` or if an error occurs (logged).
pub async fn transcribe(wav_bytes: &[u8], backend: &AsrBackend) -> Option<String> {
    match backend {
        AsrBackend::None => None,
        AsrBackend::Parakeet { url, .. } => transcribe_http(wav_bytes, url).await,
        AsrBackend::WhisperCli { bin, model } => transcribe_whisper_cli(wav_bytes, bin, model).await,
        AsrBackend::HttpServer { url } => transcribe_http(wav_bytes, url).await,
    }
}

/// Call a server that exposes POST /transcribe → `{"text":"…"}`.
pub(crate) async fn transcribe_http(wav_bytes: &[u8], url: &str) -> Option<String> {
    let endpoint = if url.ends_with("/transcribe") {
        url.to_string()
    } else {
        format!("{}/transcribe", url.trim_end_matches('/'))
    };

    let client = reqwest::Client::new();
    let resp = client
        .post(&endpoint)
        .header("Content-Type", "audio/wav")
        .body(wav_bytes.to_vec())
        .send()
        .await
        .map_err(|e| {
            log::warn!("ASR HTTP request failed: {e}");
            e
        })
        .ok()?;

    if !resp.status().is_success() {
        log::warn!("ASR HTTP server returned {}", resp.status());
        return None;
    }

    let json: AsrHttpResponse = resp.json().await.ok()?;
    let text = json.text.trim().to_string();
    if text.is_empty() { None } else { Some(text) }
}

/// Invoke the `whisper` CLI binary, writing audio to a temp file.
pub(crate) async fn transcribe_whisper_cli(wav_bytes: &[u8], bin: &str, model: &str) -> Option<String> {
    use tokio::process::Command;

    let tmp = std::env::temp_dir().join(format!("scribe_{}.wav", uuid::Uuid::new_v4()));
    tokio::fs::write(&tmp, wav_bytes).await.ok()?;

    let output = Command::new(bin)
        .args(["--model", model, "--output-txt", "--no-prints", tmp.to_str().unwrap()])
        .output()
        .await
        .ok()?;

    let _ = tokio::fs::remove_file(&tmp).await;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        log::warn!("whisper CLI error: {stderr}");
        return None;
    }

    let txt_path = tmp.with_extension("txt");
    if let Ok(text) = tokio::fs::read_to_string(&txt_path).await {
        let _ = tokio::fs::remove_file(&txt_path).await;
        let t = text.trim().to_string();
        return if t.is_empty() { None } else { Some(t) };
    }

    let t = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if t.is_empty() { None } else { Some(t) }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_backend_is_parakeet() {
        let b = AsrBackend::default();
        assert!(matches!(b, AsrBackend::Parakeet { .. }));
    }

    #[tokio::test]
    async fn none_backend_returns_none() {
        let result = transcribe(b"fake-wav", &AsrBackend::None).await;
        assert!(result.is_none());
    }

    #[test]
    fn parakeet_roundtrip() {
        let b = AsrBackend::Parakeet {
            url: "http://127.0.0.1:9000".into(),
            model: "parakeet-tdt-0.6b-v2".into(),
        };
        let json = serde_json::to_string(&b).expect("serialize");
        let b2: AsrBackend = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(b, b2);
    }

    #[test]
    fn whisper_cli_roundtrip() {
        let b = AsrBackend::WhisperCli {
            bin: "/usr/bin/whisper".into(),
            model: "base".into(),
        };
        let json = serde_json::to_string(&b).expect("serialize");
        let b2: AsrBackend = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(b, b2);
    }

    #[test]
    fn http_server_roundtrip() {
        let b = AsrBackend::HttpServer { url: "http://localhost:9001".into() };
        let json = serde_json::to_string(&b).expect("serialize");
        let b2: AsrBackend = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(b, b2);
    }

    #[test]
    fn none_roundtrip() {
        let b = AsrBackend::None;
        let json = serde_json::to_string(&b).expect("serialize");
        let b2: AsrBackend = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(b, b2);
    }

    #[tokio::test]
    async fn http_endpoint_appends_transcribe_path() {
        // Nothing listening – just verifying the URL construction doesn't panic.
        let result = transcribe_http(b"", "http://127.0.0.1:19999/").await;
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn http_endpoint_without_suffix() {
        let result = transcribe_http(b"", "http://127.0.0.1:19999").await;
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn parakeet_backend_returns_none_when_server_down() {
        let backend = AsrBackend::Parakeet {
            url: "http://127.0.0.1:19999".into(),
            model: "parakeet-tdt-0.6b-v2".into(),
        };
        let result = transcribe(b"wav-data", &backend).await;
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn whisper_cli_returns_none_for_missing_binary() {
        let result = transcribe_whisper_cli(b"", "/nonexistent/whisper", "base").await;
        assert!(result.is_none());
    }
}

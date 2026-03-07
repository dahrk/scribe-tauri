use anyhow::{Context, Result};
use serde::Deserialize;

/// Backend options for ASR.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq)]
pub enum AsrBackend {
    /// Path to the `whisper` CLI binary and the model name/path.
    WhisperCli { bin: String, model: String },
    /// A local HTTP server that accepts POST /transcribe with WAV body and returns JSON `{"text": "…"}`.
    HttpServer { url: String },
    /// No ASR configured – transcription will be skipped.
    None,
}

impl Default for AsrBackend {
    fn default() -> Self {
        Self::None
    }
}

#[derive(Deserialize)]
struct HttpAsrResponse {
    text: String,
}

/// Transcribe a WAV byte blob using the configured backend.
/// Returns `None` if the backend is `None` or if an error occurs (logged).
pub async fn transcribe(wav_bytes: &[u8], backend: &AsrBackend) -> Option<String> {
    match backend {
        AsrBackend::None => None,
        AsrBackend::WhisperCli { bin, model } => {
            transcribe_whisper_cli(wav_bytes, bin, model).await
        }
        AsrBackend::HttpServer { url } => transcribe_http(wav_bytes, url).await,
    }
}

async fn transcribe_whisper_cli(wav_bytes: &[u8], bin: &str, model: &str) -> Option<String> {
    use tokio::io::AsyncWriteExt;
    use tokio::process::Command;

    // Write WAV to a temp file, invoke whisper CLI, delete file.
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

    // whisper CLI writes `<file>.txt`; fall back to stdout.
    let txt_path = tmp.with_extension("txt");
    if let Ok(text) = tokio::fs::read_to_string(&txt_path).await {
        let _ = tokio::fs::remove_file(&txt_path).await;
        return Some(text.trim().to_string());
    }

    Some(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

async fn transcribe_http(wav_bytes: &[u8], url: &str) -> Option<String> {
    let client = reqwest::Client::new();
    let resp = client
        .post(url)
        .header("Content-Type", "audio/wav")
        .body(wav_bytes.to_vec())
        .send()
        .await
        .ok()?;

    let json: HttpAsrResponse = resp.json().await.ok()?;
    Some(json.text.trim().to_string())
}

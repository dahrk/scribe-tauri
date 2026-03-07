use anyhow::Result;
use serde::{Deserialize, Serialize};

/// Backend options for summarization.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum SummarizationBackend {
    /// Ollama HTTP API (default: http://localhost:11434).
    Ollama { base_url: String, model: String },
    /// Generic OpenAI-compatible chat API.
    OpenAiCompatible {
        base_url: String,
        model: String,
        api_key: Option<String>,
    },
    /// No summarization configured.
    None,
}

impl Default for SummarizationBackend {
    fn default() -> Self {
        Self::Ollama {
            base_url: "http://localhost:11434".into(),
            model: "llama3.2:3b".into(),
        }
    }
}

#[derive(Serialize)]
struct OllamaRequest<'a> {
    model: &'a str,
    prompt: String,
    stream: bool,
}

#[derive(Deserialize)]
struct OllamaResponse {
    response: String,
}

#[derive(Serialize)]
struct OpenAiRequest<'a> {
    model: &'a str,
    messages: Vec<OpenAiMessage<'a>>,
}

#[derive(Serialize)]
struct OpenAiMessage<'a> {
    role: &'a str,
    content: String,
}

#[derive(Deserialize)]
struct OpenAiResponse {
    choices: Vec<OpenAiChoice>,
}

#[derive(Deserialize)]
struct OpenAiChoice {
    message: OpenAiMessageContent,
}

#[derive(Deserialize)]
struct OpenAiMessageContent {
    content: String,
}

const SYSTEM_PROMPT: &str =
    "You are a concise meeting assistant. Summarize the following transcript in a few \
     bullet points. Focus on decisions, action items, and key topics discussed. \
     Do not invent information not present in the transcript.";

/// Summarize a transcript using the configured backend.
/// Returns `None` if unavailable or on error.
pub async fn summarize(transcript: &str, backend: &SummarizationBackend) -> Option<String> {
    if transcript.trim().is_empty() {
        return None;
    }
    match backend {
        SummarizationBackend::None => None,
        SummarizationBackend::Ollama { base_url, model } => {
            summarize_ollama(transcript, base_url, model).await
        }
        SummarizationBackend::OpenAiCompatible {
            base_url,
            model,
            api_key,
        } => summarize_openai_compat(transcript, base_url, model, api_key.as_deref()).await,
    }
}

async fn summarize_ollama(transcript: &str, base_url: &str, model: &str) -> Option<String> {
    let prompt = format!("{SYSTEM_PROMPT}\n\nTranscript:\n{transcript}");
    let url = format!("{base_url}/api/generate");
    let client = reqwest::Client::new();
    let resp = client
        .post(&url)
        .json(&OllamaRequest {
            model,
            prompt,
            stream: false,
        })
        .send()
        .await
        .ok()?;

    let body: OllamaResponse = resp.json().await.ok()?;
    Some(body.response.trim().to_string())
}

async fn summarize_openai_compat(
    transcript: &str,
    base_url: &str,
    model: &str,
    api_key: Option<&str>,
) -> Option<String> {
    let prompt = format!("{SYSTEM_PROMPT}\n\nTranscript:\n{transcript}");
    let url = format!("{base_url}/chat/completions");
    let client = reqwest::Client::new();
    let mut req = client.post(&url).json(&OpenAiRequest {
        model,
        messages: vec![OpenAiMessage {
            role: "user",
            content: prompt,
        }],
    });
    if let Some(key) = api_key {
        req = req.bearer_auth(key);
    }
    let resp = req.send().await.ok()?;
    let body: OpenAiResponse = resp.json().await.ok()?;
    Some(
        body.choices
            .into_iter()
            .next()?
            .message
            .content
            .trim()
            .to_string(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── Default / None ────────────────────────────────────────────────────────

    #[test]
    fn default_backend_is_ollama() {
        assert!(matches!(
            SummarizationBackend::default(),
            SummarizationBackend::Ollama { .. }
        ));
    }

    #[tokio::test]
    async fn none_backend_returns_none() {
        let r = summarize("hello world", &SummarizationBackend::None).await;
        assert!(r.is_none());
    }

    #[tokio::test]
    async fn empty_transcript_returns_none() {
        let r = summarize("   ", &SummarizationBackend::default()).await;
        assert!(r.is_none());
    }

    // ── Serialization ─────────────────────────────────────────────────────────

    #[test]
    fn ollama_roundtrip() {
        let b = SummarizationBackend::Ollama {
            base_url: "http://localhost:11434".into(),
            model: "llama3.2:3b".into(),
        };
        let json = serde_json::to_string(&b).expect("serialize");
        let b2: SummarizationBackend = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(b, b2);
    }

    #[test]
    fn openai_compat_roundtrip_with_key() {
        let b = SummarizationBackend::OpenAiCompatible {
            base_url: "http://localhost:8080/v1".into(),
            model: "phi3:mini".into(),
            api_key: Some("secret".into()),
        };
        let json = serde_json::to_string(&b).expect("serialize");
        let b2: SummarizationBackend = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(b, b2);
    }

    #[test]
    fn openai_compat_roundtrip_without_key() {
        let b = SummarizationBackend::OpenAiCompatible {
            base_url: "http://localhost:8080/v1".into(),
            model: "phi3:mini".into(),
            api_key: None,
        };
        let json = serde_json::to_string(&b).expect("serialize");
        let b2: SummarizationBackend = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(b, b2);
    }

    #[test]
    fn none_roundtrip() {
        let b = SummarizationBackend::None;
        let json = serde_json::to_string(&b).expect("serialize");
        let b2: SummarizationBackend = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(b, b2);
    }

    // ── Network failures return None gracefully ───────────────────────────────

    #[tokio::test]
    async fn ollama_returns_none_when_server_down() {
        let b = SummarizationBackend::Ollama {
            base_url: "http://127.0.0.1:19998".into(),
            model: "llama3.2:3b".into(),
        };
        let r = summarize("Some meeting transcript", &b).await;
        assert!(r.is_none());
    }

    #[tokio::test]
    async fn openai_compat_returns_none_when_server_down() {
        let b = SummarizationBackend::OpenAiCompatible {
            base_url: "http://127.0.0.1:19997/v1".into(),
            model: "phi3".into(),
            api_key: None,
        };
        let r = summarize("Some meeting transcript", &b).await;
        assert!(r.is_none());
    }
}

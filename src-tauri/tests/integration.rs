/// Integration tests for Scribe's HTTP-facing layers.
///
/// These tests spin up real (in-process) mock HTTP servers via `wiremock`
/// so we can verify the full request/response contract – not just that the
/// code gracefully returns `None` when a server is unreachable.
///
/// Modules:
///   - `asr_http`          – ASR backends that talk HTTP (Parakeet, HttpServer)
///   - `summarization_http`– Ollama and OpenAI-compatible summarization backends

use scribe_tauri_lib::asr::AsrBackend;
use scribe_tauri_lib::summarization::SummarizationBackend;
use wiremock::matchers::{header, method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

// ── ASR HTTP integration ──────────────────────────────────────────────────────

mod asr_http {
    use super::*;
    use scribe_tauri_lib::asr;

    #[tokio::test]
    async fn parakeet_returns_transcript_on_200() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/transcribe"))
            .respond_with(
                ResponseTemplate::new(200)
                    .set_body_json(serde_json::json!({"text": "hello world"})),
            )
            .mount(&server)
            .await;

        let backend = AsrBackend::Parakeet {
            url: server.uri(),
            model: "parakeet-tdt-0.6b-v2".into(),
        };
        let result = asr::transcribe(b"fake-wav-bytes", &backend).await;
        assert_eq!(result.as_deref(), Some("hello world"));
    }

    #[tokio::test]
    async fn parakeet_returns_none_on_server_error() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/transcribe"))
            .respond_with(ResponseTemplate::new(500))
            .mount(&server)
            .await;

        let backend = AsrBackend::Parakeet {
            url: server.uri(),
            model: "parakeet-tdt-0.6b-v2".into(),
        };
        let result = asr::transcribe(b"fake-wav-bytes", &backend).await;
        assert!(result.is_none(), "500 response should yield None");
    }

    #[tokio::test]
    async fn parakeet_returns_none_on_empty_text_field() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/transcribe"))
            .respond_with(
                ResponseTemplate::new(200)
                    .set_body_json(serde_json::json!({"text": "   "})),
            )
            .mount(&server)
            .await;

        let backend = AsrBackend::Parakeet {
            url: server.uri(),
            model: "parakeet-tdt-0.6b-v2".into(),
        };
        let result = asr::transcribe(b"fake-wav-bytes", &backend).await;
        assert!(result.is_none(), "Whitespace-only text should yield None");
    }

    #[tokio::test]
    async fn parakeet_returns_none_on_malformed_json() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/transcribe"))
            .respond_with(
                ResponseTemplate::new(200).set_body_string("not json at all"),
            )
            .mount(&server)
            .await;

        let backend = AsrBackend::Parakeet {
            url: server.uri(),
            model: "parakeet-tdt-0.6b-v2".into(),
        };
        let result = asr::transcribe(b"fake-wav-bytes", &backend).await;
        assert!(result.is_none(), "Malformed JSON should yield None");
    }

    #[tokio::test]
    async fn http_server_variant_returns_transcript() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/transcribe"))
            .respond_with(
                ResponseTemplate::new(200)
                    .set_body_json(serde_json::json!({"text": "meeting notes"})),
            )
            .mount(&server)
            .await;

        let backend = AsrBackend::HttpServer { url: server.uri() };
        let result = asr::transcribe(b"fake-wav-bytes", &backend).await;
        assert_eq!(result.as_deref(), Some("meeting notes"));
    }

    #[tokio::test]
    async fn url_with_trailing_slash_still_hits_transcribe_path() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/transcribe"))
            .respond_with(
                ResponseTemplate::new(200)
                    .set_body_json(serde_json::json!({"text": "trimmed slash"})),
            )
            .mount(&server)
            .await;

        // Server URI with trailing slash – the implementation should normalise it.
        let url_with_slash = format!("{}/", server.uri());
        let backend = AsrBackend::HttpServer { url: url_with_slash };
        let result = asr::transcribe(b"fake-wav-bytes", &backend).await;
        assert_eq!(result.as_deref(), Some("trimmed slash"));
    }

    #[tokio::test]
    async fn none_backend_skips_network_entirely() {
        // No MockServer needed – None backend should not make any HTTP calls.
        let result = asr::transcribe(b"fake-wav-bytes", &AsrBackend::None).await;
        assert!(result.is_none());
    }
}

// ── Summarization HTTP integration ────────────────────────────────────────────

mod summarization_http {
    use super::*;
    use scribe_tauri_lib::summarization;

    #[tokio::test]
    async fn ollama_returns_summary_on_200() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/api/generate"))
            .respond_with(
                ResponseTemplate::new(200).set_body_json(
                    serde_json::json!({"response": "• Action item: follow up\n• Decision: proceed"}),
                ),
            )
            .mount(&server)
            .await;

        let backend = SummarizationBackend::Ollama {
            base_url: server.uri(),
            model: "llama3.2:3b".into(),
        };
        let result = summarization::summarize("Meeting transcript here", &backend).await;
        assert!(result.is_some());
        let summary = result.unwrap();
        assert!(summary.contains("Action item"), "summary should include bullet points");
    }

    #[tokio::test]
    async fn ollama_returns_none_on_server_error() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/api/generate"))
            .respond_with(ResponseTemplate::new(503))
            .mount(&server)
            .await;

        let backend = SummarizationBackend::Ollama {
            base_url: server.uri(),
            model: "llama3.2:3b".into(),
        };
        let result = summarization::summarize("Meeting transcript here", &backend).await;
        assert!(result.is_none(), "503 should yield None");
    }

    #[tokio::test]
    async fn openai_compat_returns_summary_on_200() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/chat/completions"))
            .respond_with(
                ResponseTemplate::new(200).set_body_json(serde_json::json!({
                    "choices": [{
                        "message": { "content": "• Key decision made" }
                    }]
                })),
            )
            .mount(&server)
            .await;

        let backend = SummarizationBackend::OpenAiCompatible {
            base_url: server.uri(),
            model: "gpt-4o-mini".into(),
            api_key: None,
        };
        let result = summarization::summarize("Meeting transcript here", &backend).await;
        assert_eq!(result.as_deref(), Some("• Key decision made"));
    }

    #[tokio::test]
    async fn openai_compat_sends_bearer_token_when_api_key_set() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/chat/completions"))
            .and(header("authorization", "Bearer secret-key-123"))
            .respond_with(
                ResponseTemplate::new(200).set_body_json(serde_json::json!({
                    "choices": [{
                        "message": { "content": "• Summary with auth" }
                    }]
                })),
            )
            .expect(1)   // Assert exactly one matching request
            .mount(&server)
            .await;

        let backend = SummarizationBackend::OpenAiCompatible {
            base_url: server.uri(),
            model: "gpt-4o".into(),
            api_key: Some("secret-key-123".into()),
        };
        let result = summarization::summarize("Meeting transcript here", &backend).await;
        assert!(result.is_some(), "Request with correct bearer token should succeed");
        // wiremock verifies the `expect(1)` assertion on Drop.
    }

    #[tokio::test]
    async fn empty_transcript_skips_network_call() {
        // No MockServer – summarize() should bail early for empty transcripts.
        let backend = SummarizationBackend::Ollama {
            base_url: "http://127.0.0.1:19996".into(),
            model: "llama3.2:3b".into(),
        };
        let result = summarization::summarize("   ", &backend).await;
        assert!(result.is_none(), "Empty transcript should yield None without any HTTP call");
    }

    #[tokio::test]
    async fn none_backend_skips_network_entirely() {
        let result = summarization::summarize("Meeting text", &SummarizationBackend::None).await;
        assert!(result.is_none());
    }
}

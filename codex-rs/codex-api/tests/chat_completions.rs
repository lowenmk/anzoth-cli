#![allow(clippy::expect_used)]

use std::sync::Arc;
use std::sync::Mutex;
use std::time::Duration;

use anyhow::Result;
use bytes::Bytes;
use codex_api::ApiError;
use codex_api::AuthProvider;
use codex_api::ChatCompletionsClient;
use codex_api::ChatCompletionsOptions;
use codex_api::Compression;
use codex_api::Provider;
use codex_client::HttpTransport;
use codex_client::Request;
use codex_client::RequestBody;
use codex_client::Response;
use codex_client::StreamResponse;
use codex_client::TransportError;
use codex_protocol::protocol::SessionSource;
use futures::StreamExt;
use http::HeaderMap;
use http::HeaderValue;
use http::StatusCode;
use pretty_assertions::assert_eq;
use serde_json::json;

fn provider() -> Provider {
    Provider {
        name: "anzoth".to_string(),
        base_url: "https://example.com/v1".to_string(),
        query_params: None,
        headers: HeaderMap::new(),
        retry: codex_api::RetryConfig {
            max_attempts: 1,
            base_delay: Duration::from_millis(1),
            retry_429: false,
            retry_5xx: false,
            retry_transport: true,
        },
        stream_idle_timeout: Duration::from_millis(20),
    }
}

#[derive(Clone, Default)]
struct NoAuth;

impl AuthProvider for NoAuth {
    fn add_auth_headers(&self, _headers: &mut HeaderMap) {}
}

#[derive(Clone)]
struct StaticAuth {
    token: String,
}

impl StaticAuth {
    fn new(token: &str) -> Self {
        Self {
            token: token.to_string(),
        }
    }
}

impl AuthProvider for StaticAuth {
    fn add_auth_headers(&self, headers: &mut HeaderMap) {
        if let Ok(header) = HeaderValue::from_str(&format!("Bearer {}", self.token)) {
            headers.insert(http::header::AUTHORIZATION, header);
        }
    }
}

#[derive(Debug, Default, Clone)]
struct RecordingState {
    requests: Arc<Mutex<Vec<Request>>>,
}

#[derive(Clone)]
struct RecordingTransport {
    state: RecordingState,
    stream: Arc<Vec<Bytes>>,
    status: StatusCode,
    headers: HeaderMap,
}

impl RecordingTransport {
    fn new(stream: Vec<Bytes>) -> Self {
        Self {
            state: RecordingState::default(),
            stream: Arc::new(stream),
            status: StatusCode::OK,
            headers: HeaderMap::new(),
        }
    }

    fn with_status(status: StatusCode) -> Self {
        Self {
            state: RecordingState::default(),
            stream: Arc::new(Vec::new()),
            status,
            headers: HeaderMap::new(),
        }
    }

    fn take_requests(&self) -> Vec<Request> {
        std::mem::take(&mut *self.state.requests.lock().expect("mutex"))
    }

    fn push_request(&self, request: Request) {
        self.state
            .requests
            .lock()
            .expect("mutex")
            .push(request);
    }
}

impl HttpTransport for RecordingTransport {
    async fn execute(&self, _req: Request) -> Result<Response, TransportError> {
        Err(TransportError::Build("execute should not run".to_string()))
    }

    async fn stream(&self, req: Request) -> Result<StreamResponse, TransportError> {
        self.push_request(req);
        if self.status != StatusCode::OK {
            return Err(TransportError::Http {
                status: self.status,
                url: None,
                headers: None,
                body: Some(self.status.to_string()),
            });
        }
        Ok(StreamResponse {
            status: self.status,
            headers: self.headers.clone(),
            bytes: Box::pin(futures::stream::iter(
                self.stream.iter().cloned().map(Ok::<Bytes, TransportError>),
            )),
        })
    }
}

fn client_with_transport(transport: RecordingTransport) -> ChatCompletionsClient<RecordingTransport> {
    ChatCompletionsClient::new(transport, provider(), Arc::new(NoAuth))
}

#[tokio::test]
async fn chat_client_uses_chat_completions_path() -> Result<()> {
    let transport = RecordingTransport::new(Vec::new());
    let client = client_with_transport(transport.clone());

    let _stream = client
        .stream(
            json!({"model":"demo","messages":[],"stream":true}),
            HeaderMap::new(),
            Compression::None,
            None,
        )
        .await?;

    let requests = transport.take_requests();
    assert_eq!(requests.len(), 1);
    assert!(requests[0].url.ends_with("/chat/completions"));
    Ok(())
}

#[tokio::test]
async fn chat_client_adds_auth_headers() -> Result<()> {
    let transport = RecordingTransport::new(Vec::new());
    let client = ChatCompletionsClient::new(
        transport.clone(),
        provider(),
        Arc::new(StaticAuth::new("secret-token")),
    );

    let _stream = client
        .stream(
            json!({"model":"demo","messages":[],"stream":true}),
            HeaderMap::new(),
            Compression::None,
            None,
        )
        .await?;

    let requests = transport.take_requests();
    assert_eq!(
        requests[0]
            .headers
            .get(http::header::AUTHORIZATION)
            .and_then(|value| value.to_str().ok()),
        Some("Bearer secret-token")
    );
    Ok(())
}

#[tokio::test]
async fn chat_stream_parses_text_and_model_headers() -> Result<()> {
    let transport = RecordingTransport {
        state: RecordingState::default(),
        stream: Arc::new(vec![Bytes::from(
            r#"data: {"id":"chat-1","choices":[{"index":0,"delta":{"role":"assistant","content":"hello"},"finish_reason":"stop"}],"usage":{"prompt_tokens":1,"completion_tokens":2,"total_tokens":3}}

data: [DONE]

"#,
        )]),
        status: StatusCode::OK,
        headers: {
            let mut headers = HeaderMap::new();
            headers.insert("openai-model", HeaderValue::from_static("Anzoth-Coder"));
            headers
        },
    };
    let client = client_with_transport(transport.clone());

    let mut stream = client
        .stream(
            json!({"model":"demo","messages":[],"stream":true}),
            HeaderMap::new(),
            Compression::None,
            Some(Arc::new(std::sync::OnceLock::new())),
        )
        .await?;

    let mut saw_model = false;
    let mut saw_text = false;
    let mut saw_completion = false;
    while let Some(event) = stream.next().await {
        match event? {
            codex_api::ResponseEvent::ServerModel(model) => {
                assert_eq!(model, "Anzoth-Coder");
                saw_model = true;
            }
            codex_api::ResponseEvent::OutputTextDelta(delta) => {
                assert_eq!(delta, "hello");
                saw_text = true;
            }
            codex_api::ResponseEvent::Completed { response_id, .. } => {
                assert_eq!(response_id, "chat-1");
                saw_completion = true;
            }
            _ => {}
        }
    }

    assert!(saw_model);
    assert!(saw_text);
    assert!(saw_completion);
    Ok(())
}

#[tokio::test]
async fn chat_stream_parses_tool_calls() -> Result<()> {
    let transport = RecordingTransport::new(vec![Bytes::from(
        r#"data: {"id":"chat-2","choices":[{"index":0,"delta":{"tool_calls":[{"index":0,"id":"call-1","function":{"name":"shell_command","arguments":"{\"command\":\"echo hi\"}"}}]},"finish_reason":"tool_calls"}],"usage":{"prompt_tokens":1,"completion_tokens":1,"total_tokens":2}}

data: [DONE]

"#,
    )]);
    let client = client_with_transport(transport);

    let mut stream = client
        .stream(
            json!({"model":"demo","messages":[],"stream":true}),
            HeaderMap::new(),
            Compression::None,
            None,
        )
        .await?;

    let mut saw_tool_call = false;
    let mut saw_completion = false;
    while let Some(event) = stream.next().await {
        match event? {
            codex_api::ResponseEvent::OutputItemDone(codex_protocol::models::ResponseItem::FunctionCall {
                call_id,
                name,
                arguments,
                ..
            }) => {
                assert_eq!(call_id, "call-1");
                assert_eq!(name, "shell_command");
                assert_eq!(arguments, "{\"command\":\"echo hi\"}");
                saw_tool_call = true;
            }
            codex_api::ResponseEvent::Completed { end_turn, .. } => {
                assert_eq!(end_turn, Some(false));
                saw_completion = true;
            }
            _ => {}
        }
    }

    assert!(saw_tool_call);
    assert!(saw_completion);
    Ok(())
}

#[tokio::test]
async fn chat_stream_reports_backend_error() -> Result<()> {
    let transport = RecordingTransport::new(vec![Ok(Bytes::from(
        r#"data: {"error":{"message":"bad backend"}}

"#,
    ))]);
    let client = client_with_transport(transport);

    let mut stream = client
        .stream(
            json!({"model":"demo","messages":[],"stream":true}),
            HeaderMap::new(),
            Compression::None,
            None,
        )
        .await?;

    let err = stream
        .next()
        .await
        .expect("stream should yield an error")
        .expect_err("expected stream error");
    assert!(matches!(err, ApiError::Stream(message) if message == "bad backend"));
    Ok(())
}

#[tokio::test]
async fn chat_client_surfaces_http_status_errors() -> Result<()> {
    for status in [
        StatusCode::UNAUTHORIZED,
        StatusCode::FORBIDDEN,
        StatusCode::NOT_FOUND,
        StatusCode::TOO_MANY_REQUESTS,
        StatusCode::INTERNAL_SERVER_ERROR,
    ] {
        let transport = RecordingTransport::with_status(status);
        let client = client_with_transport(transport);
        let err = client
            .stream(
                json!({"model":"demo","messages":[],"stream":true}),
                HeaderMap::new(),
                Compression::None,
                None,
            )
            .await
            .expect_err("expected transport failure");
        assert!(matches!(
            err,
            ApiError::Transport(TransportError::Http { status: actual, .. }) if actual == status
        ));
    }

    Ok(())
}

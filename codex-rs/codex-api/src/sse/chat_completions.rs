use crate::common::ResponseEvent;
use crate::common::ResponseStream;
use crate::error::ApiError;
use crate::rate_limits::parse_all_rate_limits;
use crate::safety_buffering::treatment_from_headers;
use crate::telemetry::SseTelemetry;
use codex_client::ByteStream;
use codex_client::StreamResponse;
use codex_protocol::ResponseItemId;
use codex_protocol::models::ContentItem;
use codex_protocol::models::ResponseItem;
use codex_protocol::protocol::TokenUsage;
use eventsource_stream::Eventsource;
use futures::StreamExt;
use serde_json::Value;
use std::sync::Arc;
use std::sync::OnceLock;
use std::time::Duration;
use tokio::sync::mpsc;
use tokio::time::Instant;
use tokio::time::timeout;
use tracing::debug;
use tracing::trace;

const OPENAI_MODEL_HEADER: &str = "openai-model";
const REQUEST_ID_HEADER: &str = "x-request-id";

pub fn spawn_chat_completion_stream(
    stream_response: StreamResponse,
    idle_timeout: Duration,
    telemetry: Option<Arc<dyn SseTelemetry>>,
    turn_state: Option<Arc<OnceLock<String>>>,
) -> ResponseStream {
    let rate_limit_snapshots = parse_all_rate_limits(&stream_response.headers);
    let server_model = stream_response
        .headers
        .get(OPENAI_MODEL_HEADER)
        .and_then(|v| v.to_str().ok())
        .map(ToString::to_string);
    let upstream_request_id = stream_response
        .headers
        .get(REQUEST_ID_HEADER)
        .and_then(|value| value.to_str().ok())
        .map(str::to_string);
    let safety_buffering_treatment =
        treatment_from_headers(&stream_response.headers).unwrap_or_default();
    let (tx_event, rx_event) = mpsc::channel::<Result<ResponseEvent, ApiError>>(1600);
    tokio::spawn(async move {
        if let Some(model) = server_model {
            let _ = tx_event.send(Ok(ResponseEvent::ServerModel(model))).await;
        }
        for snapshot in rate_limit_snapshots {
            let _ = tx_event.send(Ok(ResponseEvent::RateLimits(snapshot))).await;
        }
        process_chat_completion_sse(
            stream_response.bytes,
            tx_event,
            idle_timeout,
            telemetry,
            safety_buffering_treatment,
            turn_state,
        )
        .await;
    });

    ResponseStream {
        rx_event,
        upstream_request_id,
    }
}

#[derive(Debug, Default)]
struct ToolCallState {
    call_id: String,
    name: String,
    arguments: String,
    announced: bool,
}

#[derive(Debug, Default)]
struct ChatStreamState {
    response_id: Option<String>,
    assistant_text: String,
    assistant_announced: bool,
    tool_calls: Vec<ToolCallState>,
    output_flushed: bool,
    completed: bool,
    usage: Option<TokenUsage>,
    end_turn: Option<bool>,
}

async fn process_chat_completion_sse(
    stream: ByteStream,
    tx_event: mpsc::Sender<Result<ResponseEvent, ApiError>>,
    idle_timeout: Duration,
    telemetry: Option<Arc<dyn SseTelemetry>>,
    _safety_buffering_treatment: crate::common::SafetyBufferingTreatment,
    _turn_state: Option<Arc<OnceLock<String>>>,
) {
    let mut stream = stream.eventsource();
    let mut state = ChatStreamState::default();

    loop {
        let start = Instant::now();
        let response = timeout(idle_timeout, stream.next()).await;
        if let Some(t) = telemetry.as_ref() {
            t.on_sse_poll(&response, start.elapsed());
        }
        let sse = match response {
            Ok(Some(Ok(sse))) => sse,
            Ok(Some(Err(e))) => {
                debug!("chat SSE error: {e:#}");
                let _ = tx_event.send(Err(ApiError::Stream(e.to_string()))).await;
                return;
            }
            Ok(None) => {
                if state.completed {
                    if tx_event.send(Ok(build_completed(&state))).await.is_err() {
                        return;
                    }
                    return;
                }
                let _ = tx_event
                    .send(Err(ApiError::Stream(
                        "stream closed before chat completion finished".into(),
                    )))
                    .await;
                return;
            }
            Err(_) => {
                let _ = tx_event
                    .send(Err(ApiError::Stream("idle timeout waiting for SSE".into())))
                    .await;
                return;
            }
        };

        if sse.data.trim() == "[DONE]" {
            if !state.output_flushed {
                flush_completed_items(&mut state, &tx_event).await;
            }
            if !state.completed {
                state.completed = true;
                if tx_event.send(Ok(build_completed(&state))).await.is_err() {
                    return;
                }
            }
            return;
        }

        trace!("chat SSE event: {}", &sse.data);

        let event: Value = match serde_json::from_str(&sse.data) {
            Ok(event) => event,
            Err(e) => {
                debug!("failed to parse chat SSE event: {e}, data: {}", &sse.data);
                continue;
            }
        };

        if let Some(error) = event.get("error") {
            let message = error
                .get("message")
                .and_then(Value::as_str)
                .unwrap_or("chat completion error")
                .to_string();
            let _ = tx_event.send(Err(ApiError::Stream(message))).await;
            return;
        }

        if state.response_id.is_none() {
            state.response_id = event
                .get("id")
                .and_then(Value::as_str)
                .map(ToString::to_string);
        }

        if let Some(usage) = event.get("usage").and_then(chat_usage_from_value) {
            state.usage = Some(usage);
        }

        let Some(choices) = event.get("choices").and_then(Value::as_array) else {
            continue;
        };

        for choice in choices {
            if let Some(delta) = choice.get("delta") {
                process_choice_delta(delta, &mut state, &tx_event).await;
            }

            if let Some(finish_reason) = choice.get("finish_reason").and_then(Value::as_str) {
                state.end_turn = Some(finish_reason != "tool_calls");
                if !state.output_flushed {
                    flush_completed_items(&mut state, &tx_event).await;
                }
                state.completed = true;
            }
        }
    }
}

async fn process_choice_delta(
    delta: &Value,
    state: &mut ChatStreamState,
    tx_event: &mpsc::Sender<Result<ResponseEvent, ApiError>>,
) {
    if let Some(text) = chat_delta_text(delta) {
        if !text.is_empty() {
            if !state.assistant_announced {
                state.assistant_announced = true;
                let _ = tx_event
                    .send(Ok(ResponseEvent::OutputItemAdded(ResponseItem::Message {
                        id: Some(ResponseItemId::new("msg")),
                        role: "assistant".to_string(),
                        content: Vec::new(),
                        phase: None,
                        internal_chat_message_metadata_passthrough: None,
                    })))
                    .await;
            }
            state.assistant_text.push_str(&text);
            let _ = tx_event.send(Ok(ResponseEvent::OutputTextDelta(text))).await;
        }
    }

    if let Some(tool_calls) = delta.get("tool_calls").and_then(Value::as_array) {
        for (position, tool_call) in tool_calls.iter().enumerate() {
            let index = tool_call
                .get("index")
                .and_then(Value::as_i64)
                .unwrap_or(position as i64);
            let call_state = ensure_tool_call_state(state, index);

            if let Some(call_id) = tool_call.get("id").and_then(Value::as_str)
                && call_state.call_id.is_empty()
            {
                call_state.call_id = call_id.to_string();
            }

            if let Some(function) = tool_call.get("function").and_then(Value::as_object) {
                if let Some(name) = function.get("name").and_then(Value::as_str)
                    && call_state.name.is_empty()
                {
                    call_state.name = name.to_string();
                }
                if let Some(arguments) = function.get("arguments").and_then(Value::as_str)
                    && !arguments.is_empty()
                {
                    if !call_state.announced {
                        call_state.announced = true;
                        let _ = tx_event
                            .send(Ok(ResponseEvent::OutputItemAdded(
                                build_tool_call_item(call_state),
                            )))
                            .await;
                    }
                    call_state.arguments.push_str(arguments);
                    let call_id = call_state.call_id.clone();
                    let _ = tx_event
                        .send(Ok(ResponseEvent::ToolCallInputDelta {
                            item_id: call_id.clone(),
                            call_id: Some(call_id),
                            delta: arguments.to_string(),
                        }))
                        .await;
                }
            }
        }
    }
}

fn ensure_tool_call_state(state: &mut ChatStreamState, index: i64) -> &mut ToolCallState {
    if index < 0 {
        state.tool_calls.push(ToolCallState::default());
        return state.tool_calls.last_mut().expect("tool call state");
    }

    let index = index as usize;
    if state.tool_calls.len() <= index {
        state.tool_calls.resize_with(index + 1, ToolCallState::default);
    }
    &mut state.tool_calls[index]
}

fn build_tool_call_item(call_state: &ToolCallState) -> ResponseItem {
    let item_id = ResponseItemId::new("fc");
    let (namespace, name) = split_chat_tool_name(&call_state.name);
    ResponseItem::FunctionCall {
        id: Some(item_id.clone()),
        name,
        namespace,
        arguments: call_state.arguments.clone(),
        call_id: if call_state.call_id.is_empty() {
            item_id.to_string()
        } else {
            call_state.call_id.clone()
        },
        internal_chat_message_metadata_passthrough: None,
    }
}

async fn flush_completed_items(
    state: &mut ChatStreamState,
    tx_event: &mpsc::Sender<Result<ResponseEvent, ApiError>>,
) {
    if state.output_flushed {
        return;
    }
    state.output_flushed = true;

    for call_state in &state.tool_calls {
        if call_state.call_id.is_empty() && call_state.arguments.is_empty() {
            continue;
        }
        let item = build_tool_call_item(call_state);
        if tx_event
            .send(Ok(ResponseEvent::OutputItemDone(item)))
            .await
            .is_err()
        {
            return;
        }
    }

    if !state.assistant_text.is_empty() {
        let item = ResponseItem::Message {
            id: Some(ResponseItemId::new("msg")),
            role: "assistant".to_string(),
            content: vec![ContentItem::OutputText {
                text: state.assistant_text.clone(),
            }],
            phase: None,
            internal_chat_message_metadata_passthrough: None,
        };
        let _ = tx_event.send(Ok(ResponseEvent::OutputItemDone(item))).await;
    }
}

fn build_completed(state: &ChatStreamState) -> ResponseEvent {
    ResponseEvent::Completed {
        response_id: state
            .response_id
            .clone()
            .unwrap_or_else(|| format!("chat_{}", uuid::Uuid::now_v7())),
        token_usage: state.usage.clone(),
        end_turn: state.end_turn,
    }
}

fn chat_delta_text(delta: &Value) -> Option<String> {
    match delta.get("content")? {
        Value::String(text) => Some(text.clone()),
        Value::Array(parts) => {
            let mut text = String::new();
            for part in parts {
                if let Some(value) = part.get("text").and_then(Value::as_str) {
                    text.push_str(value);
                }
            }
            (!text.is_empty()).then_some(text)
        }
        _ => None,
    }
}

fn chat_usage_from_value(value: &Value) -> Option<TokenUsage> {
    Some(TokenUsage {
        input_tokens: value.get("prompt_tokens").and_then(Value::as_i64)?,
        cached_input_tokens: value
            .get("prompt_tokens_details")
            .and_then(|details| details.get("cached_tokens"))
            .and_then(Value::as_i64)
            .unwrap_or(0),
        cache_write_input_tokens: 0,
        output_tokens: value.get("completion_tokens").and_then(Value::as_i64)?,
        reasoning_output_tokens: 0,
        total_tokens: value.get("total_tokens").and_then(Value::as_i64)?,
    })
}

fn split_chat_tool_name(name: &str) -> (Option<String>, String) {
    if !name.starts_with("mcp__") {
        return (None, name.to_string());
    }

    if let Some(separator) = name.rfind("__")
        && separator + 2 < name.len()
    {
        return (
            Some(name[..separator + 2].to_string()),
            name[separator + 2..].to_string(),
        );
    }

    (None, name.to_string())
}

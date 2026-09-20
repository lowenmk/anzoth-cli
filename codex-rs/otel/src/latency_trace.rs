use std::collections::HashSet;
use std::sync::Mutex;
use std::sync::OnceLock;
use std::sync::atomic::AtomicU64;
use std::sync::atomic::Ordering;
use std::time::Instant;
use std::time::SystemTime;

struct TraceState {
    id: u64,
    started: Instant,
    stages: HashSet<&'static str>,
    stream_events: HashSet<&'static str>,
    thread_id: Option<String>,
    turn_id: Option<String>,
}

static NEXT_ID: AtomicU64 = AtomicU64::new(1);
static ACTIVE_TRACE: OnceLock<Mutex<Option<TraceState>>> = OnceLock::new();

fn enabled() -> bool {
    trace_enabled(std::env::var("ANZOTH_LATENCY_TRACE").ok().as_deref())
}

fn trace_enabled(value: Option<&str>) -> bool {
    value.is_some_and(|value| value == "1")
}

fn trace_lock() -> &'static Mutex<Option<TraceState>> {
    ACTIVE_TRACE.get_or_init(|| Mutex::new(None))
}

fn emit(trace: &mut TraceState, stage: &'static str) {
    let timestamp_ms = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .map_or(0, |duration| duration.as_millis());
    eprintln!(
        "[ANZOTH-LATENCY ts_ms={timestamp_ms} turn={} elapsed_ms={} thread_id={} turn_id={} stage={stage}]",
        trace.id,
        trace.started.elapsed().as_millis(),
        trace.thread_id.as_deref().unwrap_or("none"),
        trace.turn_id.as_deref().unwrap_or("none")
    );
}

pub fn process_marker(stage: &'static str) {
    if enabled() {
        let timestamp_ms = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .map_or(0, |duration| duration.as_millis());
        eprintln!("[ANZOTH-LATENCY ts_ms={timestamp_ms} process stage={stage}]");
    }
}

/// Starts the process-local trace for the next interactive turn.
pub fn begin() {
    begin_for_thread(None);
}

pub fn begin_for_thread(thread_id: Option<&str>) {
    if !enabled() {
        return;
    }

    let id = NEXT_ID.fetch_add(1, Ordering::Relaxed);
    let started = Instant::now();
    if let Ok(mut active) = trace_lock().lock() {
        let mut trace = TraceState {
            id,
            started,
            thread_id: thread_id.map(str::to_owned),
            turn_id: None,
            stages: HashSet::new(),
            stream_events: HashSet::new(),
        };
        emit(&mut trace, "turn_start_received");
        *active = Some(trace);
    }
}

pub fn set_turn_id(turn_id: impl Into<String>) {
    if let Ok(mut active) = trace_lock().lock()
        && let Some(trace) = active.as_mut()
    {
        trace.turn_id = Some(turn_id.into());
    }
}

/// Records a stage once for the active diagnostic turn.
pub fn mark_once(stage: &'static str) {
    mark_once_after(stage, None);
}

/// Records a stage once after an earlier stage has been observed.
pub fn mark_once_after(stage: &'static str, prerequisite: Option<&'static str>) {
    if !enabled() {
        return;
    }

    if let Ok(mut active) = trace_lock().lock()
        && let Some(trace) = active.as_mut()
    {
        if let Some(prerequisite) = prerequisite
            && !trace.stages.contains(prerequisite)
        {
            return;
        }
        if !trace.stages.insert(stage) {
            return;
        }
        emit(trace, stage);
    }
}

/// Records a distinct response-stream event type for the active diagnostic turn.
pub fn mark_stream_event(event: &'static str) {
    if !enabled() {
        return;
    }

    if let Ok(mut active) = trace_lock().lock()
        && let Some(trace) = active.as_mut()
        && trace.stream_events.insert(event)
    {
        emit(trace, event);
    }
}

pub fn record_http_attempt_start(attempt: u64, url: &str) {
    if !enabled() {
        return;
    }
    if let Ok(mut active) = trace_lock().lock()
        && let Some(trace) = active.as_mut()
    {
        let _ = url;
        emit(trace, "responses_http_dispatch");
        eprintln!(
            "[ANZOTH-LATENCY turn={} attempt={} retry_stage=http_dispatch]",
            trace.id, attempt
        );
    }
}

pub fn record_http_attempt_result(
    attempt: u64,
    status: Option<u16>,
    duration_ms: u128,
    error: Option<&str>,
) {
    if !enabled() {
        return;
    }
    if let Ok(mut active) = trace_lock().lock()
        && let Some(trace) = active.as_mut()
    {
        let status = status.map_or_else(|| "none".to_string(), |status| status.to_string());
        let error_category = error.map_or("none", classify_error);
        eprintln!(
            "[ANZOTH-LATENCY turn={} attempt={} stage=responses_http_result status={} duration_ms={} error_category={}]",
            trace.id, attempt, status, duration_ms, error_category
        );
        if attempt > 1 {
            eprintln!(
                "[ANZOTH-LATENCY turn={} retry_stage=responses_http attempt={} reason_category={}]",
                trace.id, attempt, error_category
            );
        }
    }
}

pub fn record_http_response_request_id(request_id: Option<&str>) {
    if !enabled() {
        return;
    }
    let Some(_request_id) = request_id else {
        return;
    };
    if let Ok(mut active) = trace_lock().lock()
        && let Some(trace) = active.as_mut()
    {
        eprintln!(
            "[ANZOTH-LATENCY turn={} stage=response_request_id present=true]",
            trace.id
        );
    }
}

/// Records request shape without retaining or emitting request content.
#[allow(clippy::too_many_arguments)]
pub fn record_request_shape(
    model: &str,
    request_bytes: usize,
    input_items: usize,
    instruction_bytes: usize,
    tool_count: usize,
    tool_schema_bytes: usize,
    image_count: usize,
    reasoning_effort: Option<&str>,
) {
    if !enabled() {
        return;
    }

    if let Ok(mut active) = trace_lock().lock()
        && let Some(trace) = active.as_mut()
    {
        let reasoning_effort = reasoning_effort.unwrap_or("none");
        eprintln!(
            "[ANZOTH-LATENCY turn={} stage=request_shape model={} request_bytes={} input_items={} instruction_bytes={} tool_count={} tool_schema_bytes={} image_count={} reasoning_effort={}]",
            trace.id,
            model,
            request_bytes,
            input_items,
            instruction_bytes,
            tool_count,
            tool_schema_bytes,
            image_count,
            reasoning_effort
        );
    }
}

/// Emits privacy-preserving request composition details. Values are derived from
/// serialized JSON only; prompt, schema, and metadata contents are never logged.
pub fn record_request_accounting(request: &serde_json::Value) {
    if !enabled() {
        return;
    }
    let Some(object) = request.as_object() else {
        return;
    };
    let input = object.get("input").and_then(serde_json::Value::as_array);
    let tools = object.get("tools").and_then(serde_json::Value::as_array);
    let instruction_bytes = object
        .get("instructions")
        .map(serialized_len)
        .unwrap_or_default();
    let input_bytes: usize = input
        .map(|items| items.iter().map(serialized_len).sum())
        .unwrap_or_default();
    let tool_bytes: usize = tools
        .map(|items| items.iter().map(serialized_len).sum())
        .unwrap_or_default();
    let total_bytes = serde_json::to_vec(request).map_or(0, |bytes| bytes.len());
    let item_lines = input
        .map(|items| {
            items
                .iter()
                .enumerate()
                .map(|(index, item)| {
                    format!(
                        "index={index} category={} bytes={}",
                        input_category(item),
                        serialized_len(item)
                    )
                })
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    let tool_lines = tools
        .map(|items| {
            items
                .iter()
                .map(|tool| {
                    let name = tool
                        .get("name")
                        .and_then(serde_json::Value::as_str)
                        .or_else(|| {
                            tool.pointer("/function/name")
                                .and_then(serde_json::Value::as_str)
                        })
                        .unwrap_or("unknown");
                    format!(
                        "name={} category={} bytes={}",
                        safe_name(name),
                        tool_category(name),
                        serialized_len(tool)
                    )
                })
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    let duplicate_lines = duplicate_blocks(input, tools);
    if let Ok(mut active) = trace_lock().lock()
        && let Some(trace) = active.as_mut()
    {
        eprintln!(
            "[ANZOTH-LATENCY turn={} stage=request_accounting total_bytes={} base_system_bytes={} developer_bytes=0 history_user_bytes={} environment_bytes=0 workspace_bytes=0 repository_project_bytes=0 agents_guidance_bytes=0 git_worktree_bytes=0 skills_bytes=0 plugins_bytes=0 mcp_bytes=0 tool_schema_bytes={} other_bytes={}]",
            trace.id,
            total_bytes,
            instruction_bytes,
            input_bytes,
            tool_bytes,
            total_bytes.saturating_sub(instruction_bytes + input_bytes + tool_bytes)
        );
        for line in tool_lines {
            eprintln!(
                "[ANZOTH-LATENCY turn={} tool_accounting {}]",
                trace.id, line
            );
        }
        for line in item_lines {
            eprintln!(
                "[ANZOTH-LATENCY turn={} input_accounting {}]",
                trace.id, line
            );
        }
        for line in duplicate_lines {
            eprintln!(
                "[ANZOTH-LATENCY turn={} duplicate_block {}]",
                trace.id, line
            );
        }
    }
}

fn serialized_len(value: &serde_json::Value) -> usize {
    serde_json::to_vec(value).map_or(0, |bytes| bytes.len())
}

fn safe_name(name: &str) -> String {
    name.chars()
        .filter(|c| c.is_ascii_alphanumeric() || matches!(*c, '_' | '-' | '.'))
        .take(80)
        .collect()
}

fn input_category(value: &serde_json::Value) -> &'static str {
    match value.get("type").and_then(serde_json::Value::as_str) {
        Some("message") => match value.get("role").and_then(serde_json::Value::as_str) {
            Some("user") => "user",
            Some("developer") => "developer",
            Some("system") => "system",
            _ => "message",
        },
        Some("input_text") => "user_text",
        Some("reasoning") => "reasoning",
        Some(kind) if kind.contains("tool") => "tool",
        _ => "other",
    }
}

fn tool_category(name: &str) -> &'static str {
    if name.starts_with("mcp__") {
        "mcp"
    } else if name.starts_with("plugin__") {
        "plugin"
    } else {
        "core"
    }
}

fn block_hash(value: &serde_json::Value) -> String {
    use std::hash::{Hash, Hasher};
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    serde_json::to_vec(value)
        .unwrap_or_default()
        .hash(&mut hasher);
    format!("{:016x}", hasher.finish())
}

fn duplicate_blocks(
    input: Option<&Vec<serde_json::Value>>,
    tools: Option<&Vec<serde_json::Value>>,
) -> Vec<String> {
    use std::collections::HashMap;
    let mut seen: HashMap<String, (usize, usize, &'static str)> = HashMap::new();
    let mut duplicates = Vec::new();
    for (category, values) in [
        ("input", input.map(Vec::as_slice).unwrap_or(&[])),
        ("tool", tools.map(Vec::as_slice).unwrap_or(&[])),
    ] {
        for value in values {
            let hash = block_hash(value);
            let bytes = serialized_len(value);
            if let Some((count, _, _)) = seen.get_mut(&hash) {
                *count += 1;
                duplicates.push(format!(
                    "hash={} bytes={} category={} count={}",
                    &hash[..8],
                    bytes,
                    category,
                    *count
                ));
            } else {
                seen.insert(hash, (1, bytes, category));
            }
        }
    }
    duplicates
}

fn classify_error(error: &str) -> &'static str {
    let lower = error.to_ascii_lowercase();
    if lower.contains("timeout") || lower.contains("timed out") {
        "timeout"
    } else if lower.contains("401") || lower.contains("auth") {
        "authentication"
    } else if lower.contains("429") || lower.contains("rate") {
        "rate_limit"
    } else if lower.contains("connect") || lower.contains("dns") {
        "connection"
    } else {
        "transport"
    }
}

#[cfg(test)]
mod tests {
    use super::classify_error;
    use super::trace_enabled;

    #[test]
    fn trace_is_disabled_unless_explicitly_enabled() {
        assert!(!trace_enabled(None));
        assert!(!trace_enabled(Some("0")));
        assert!(!trace_enabled(Some("true")));
        assert!(trace_enabled(Some("1")));
    }

    #[test]
    fn error_classification_does_not_return_error_content() {
        assert_eq!(classify_error("401 secret-token-value"), "authentication");
        assert_eq!(classify_error("connection timed out"), "timeout");
        assert_eq!(classify_error("unexpected response"), "transport");
    }

    #[test]
    fn accounting_is_structural_and_detects_duplicate_blocks() {
        let request = serde_json::json!({
            "instructions": "secret instruction",
            "input": [{"type": "message", "role": "user", "content": [{"type": "input_text", "text": "same"}]}, {"type": "message", "role": "user", "content": [{"type": "input_text", "text": "same"}]}],
            "tools": [{"type": "function", "name": "exec_command", "parameters": {"type": "object"}}]
        });
        assert_eq!(input_category(&request["input"][0]), "user");
        assert_eq!(tool_category("exec_command"), "core");
        assert_eq!(
            duplicate_blocks(request["input"].as_array(), request["tools"].as_array()).len(),
            1
        );
        assert_eq!(safe_name("exec_command secret"), "exec_commandsecret");
    }
}

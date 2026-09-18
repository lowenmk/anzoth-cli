use codex_protocol::models::ContentItem;
use codex_protocol::models::ResponseItem;
use codex_protocol::protocol::EventMsg;
use codex_protocol::protocol::RolloutItem;
use serde::Deserialize;
use serde_json::Value;

pub const THREAD_TITLE_MAX_CHARS: usize = 36;
pub const THREAD_TITLE_PROMPT_MAX_BYTES: usize = 960;

#[derive(Debug, Deserialize)]
struct GeneratedThreadTitle {
    title: String,
}

pub fn output_schema() -> Value {
    serde_json::json!({
        "type": "object",
        "properties": { "title": {
            "type": "string", "minLength": 1, "maxLength": THREAD_TITLE_MAX_CHARS
        }},
        "required": ["title"],
        "additionalProperties": false
    })
}

pub fn prompt(user_message: &str) -> String {
    let instructions = format!(
        "Generate a concise, single-line task title of at most {THREAD_TITLE_MAX_CHARS} characters and under five words where possible. Start with an imperative verb. Capitalize only the first word unless the user's language, proper nouns, acronyms, or code terms require otherwise. Preserve ticket references exactly. Write in the user's language. Do not use quotes, markdown, or trailing punctuation. Do not answer the request."
    );
    let prefix = format!("{instructions}\n\nUser prompt:\n");
    let remaining = THREAD_TITLE_PROMPT_MAX_BYTES.saturating_sub(prefix.len());
    let bounded: String = user_message
        .trim()
        .char_indices()
        .take_while(|(index, character)| index + character.len_utf8() <= remaining)
        .map(|(_, character)| character)
        .collect();
    format!("{prefix}{bounded}")
}

pub fn parse(value: &str) -> Option<String> {
    let title = serde_json::from_str::<GeneratedThreadTitle>(value)
        .ok()?
        .title;
    let title = title.trim();
    if title.is_empty() || title.chars().count() > THREAD_TITLE_MAX_CHARS || title.contains('\n') {
        return None;
    }
    Some(title.to_string())
}

pub fn completed_turn_message(items: &[RolloutItem], turn_id: &str) -> Option<Option<String>> {
    items.iter().rev().find_map(|item| match item {
        RolloutItem::EventMsg(EventMsg::TurnComplete(completion))
            if completion.turn_id == turn_id =>
        {
            Some(completion.last_agent_message.clone())
        }
        _ => None,
    })
}

pub fn provisional_title(value: &str) -> Option<String> {
    let value = value.trim();
    (!value.is_empty()).then(|| value.chars().take(THREAD_TITLE_MAX_CHARS).collect())
}

fn visible_user_text(value: &str) -> Option<String> {
    let value = value.trim();
    let value = value
        .split_once("\n## My request:")
        .map(|(_, request)| request)
        .unwrap_or(value)
        .trim();
    (!value.is_empty()).then(|| value.to_string())
}

fn supplied_title_context(value: &str) -> Option<String> {
    let value = value.trim();
    (!value.is_empty()).then(|| value.to_string())
}

fn response_message_text(item: &ResponseItem, expected_role: &str) -> Option<String> {
    let ResponseItem::Message { role, content, .. } = item else {
        return None;
    };
    (role == expected_role)
        .then(|| {
            content
                .iter()
                .filter_map(|item| match item {
                    ContentItem::InputText { text } | ContentItem::OutputText { text } => {
                        Some(text.as_str())
                    }
                    ContentItem::InputImage { .. } => None,
                })
                .collect::<Vec<_>>()
                .join("\n")
        })
        .filter(|text| !text.trim().is_empty())
}

fn event_message_text(item: &RolloutItem, expected_role: &str) -> Option<String> {
    let RolloutItem::EventMsg(event) = item else {
        return None;
    };
    match (expected_role, event) {
        ("user", EventMsg::UserMessage(message)) => visible_user_text(&message.message),
        ("assistant", EventMsg::AgentMessage(message)) => {
            (!message.message.trim().is_empty()).then(|| message.message.clone())
        }
        _ => None,
    }
}

fn message_text_for_role(item: &RolloutItem, expected_role: &str) -> Option<String> {
    match item {
        RolloutItem::ResponseItem(response) => response_message_text(response, expected_role),
        RolloutItem::EventMsg(_) => event_message_text(item, expected_role),
        _ => None,
    }
}

fn role_messages(items: &[RolloutItem], role: &str) -> Vec<String> {
    let response_messages = items
        .iter()
        .filter_map(|item| match item {
            RolloutItem::ResponseItem(response) => response_message_text(response, role),
            _ => None,
        })
        .collect::<Vec<_>>();
    if !response_messages.is_empty() {
        return response_messages;
    }
    items
        .iter()
        .filter_map(|item| message_text_for_role(item, role))
        .collect()
}

pub fn conversation_context(
    items: &[RolloutItem],
    first_user_context: Option<&str>,
) -> Option<String> {
    let user_messages = role_messages(items, "user");
    let assistant_messages = role_messages(items, "assistant");
    let first_user = first_user_context
        .and_then(supplied_title_context)
        .or_else(|| user_messages.first().cloned())?;
    let first_assistant = assistant_messages.first().cloned()?;
    let second_user = user_messages.get(1).cloned()?;
    Some(format!(
        "First user message:\n{first_user}\n\nFirst assistant response:\n{first_assistant}\n\nSecond user message:\n{second_user}"
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn title_prompt_is_bounded_without_splitting_utf8() {
        let prompt = prompt(&"🙂".repeat(2_000));
        assert!(prompt.len() <= THREAD_TITLE_PROMPT_MAX_BYTES);
        assert!(std::str::from_utf8(prompt.as_bytes()).is_ok());
    }

    #[test]
    fn title_parser_rejects_unsafe_or_invalid_output() {
        assert_eq!(
            parse(r#"{"title":"Summarize issue"}"#).as_deref(),
            Some("Summarize issue")
        );
        assert!(parse(r#"{"title":""}"#).is_none());
        assert!(parse(r#"{"title":"line\nbreak"}"#).is_none());
        assert!(parse(&format!(r#"{{"title":"{}"}}"#, "x".repeat(37))).is_none());
    }

    #[test]
    fn completed_turn_message_uses_canonical_turn_complete_result() {
        let items = vec![RolloutItem::EventMsg(EventMsg::TurnComplete(
            codex_protocol::protocol::TurnCompleteEvent {
                turn_id: "title-turn".to_string(),
                last_agent_message: Some(r#"{"title":"Summarize issue"}"#.to_string()),
                error: None,
                started_at: None,
                completed_at: None,
                duration_ms: None,
                time_to_first_token_ms: None,
            },
        ))];
        assert_eq!(
            completed_turn_message(&items, "title-turn").flatten(),
            Some(r#"{"title":"Summarize issue"}"#.to_string())
        );
        assert!(completed_turn_message(&items, "other-turn").is_none());
    }

    #[test]
    fn provisional_title_uses_visible_text_only() {
        assert_eq!(
            provisional_title("  visible request  ").as_deref(),
            Some("visible request")
        );
        assert!(provisional_title(" \n\t ").is_none());
    }

    #[test]
    fn conversation_context_requires_two_users_and_one_assistant() {
        let items = vec![
            RolloutItem::ResponseItem(ResponseItem::Message {
                id: None,
                role: "user".to_string(),
                content: vec![ContentItem::InputText {
                    text: "first".to_string(),
                }],
                phase: None,
                internal_chat_message_metadata_passthrough: None,
            }),
            RolloutItem::ResponseItem(ResponseItem::Message {
                id: None,
                role: "assistant".to_string(),
                content: vec![ContentItem::OutputText {
                    text: "answer".to_string(),
                }],
                phase: None,
                internal_chat_message_metadata_passthrough: None,
            }),
            RolloutItem::ResponseItem(ResponseItem::Message {
                id: None,
                role: "user".to_string(),
                content: vec![ContentItem::InputText {
                    text: "second".to_string(),
                }],
                phase: None,
                internal_chat_message_metadata_passthrough: None,
            }),
        ];
        let context = conversation_context(&items, None).expect("conversation should be ready");
        assert!(context.contains("first"));
        assert!(context.contains("answer"));
        assert!(context.contains("second"));
    }
}

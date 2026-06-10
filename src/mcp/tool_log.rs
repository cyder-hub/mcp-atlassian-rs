use crate::upstream::redaction::{REDACTED, redact_text};
use serde_json::{Map, Value, json};

pub(super) const TOOL_LOG_REDACTED: &str = REDACTED;
pub(super) const TOOL_LOG_TRUNCATED: &str = "[truncated]";
const TOOL_LOG_MAX_DEPTH: usize = 8;
const TOOL_LOG_MAX_ARRAY_ITEMS: usize = 50;
pub(super) const TOOL_LOG_MAX_STRING_CHARS: usize = 1_000;

pub(super) fn sanitize_tool_log_arguments(arguments: Option<&Map<String, Value>>) -> Value {
    arguments.map_or_else(
        || Value::Object(Map::new()),
        |arguments| Value::Object(sanitize_tool_log_object(arguments, 0)),
    )
}

fn sanitize_tool_log_object(arguments: &Map<String, Value>, depth: usize) -> Map<String, Value> {
    arguments
        .iter()
        .map(|(key, value)| {
            let value = if is_sensitive_log_key(key) {
                Value::String(TOOL_LOG_REDACTED.to_string())
            } else {
                sanitize_tool_log_value(value, depth + 1)
            };
            (key.clone(), value)
        })
        .collect()
}

fn sanitize_tool_log_value(value: &Value, depth: usize) -> Value {
    if depth > TOOL_LOG_MAX_DEPTH {
        return Value::String(TOOL_LOG_TRUNCATED.to_string());
    }

    match value {
        Value::Array(values) => {
            let mut sanitized = values
                .iter()
                .take(TOOL_LOG_MAX_ARRAY_ITEMS)
                .map(|value| sanitize_tool_log_value(value, depth + 1))
                .collect::<Vec<_>>();
            if values.len() > TOOL_LOG_MAX_ARRAY_ITEMS {
                sanitized.push(json!({
                    "truncated_items": values.len() - TOOL_LOG_MAX_ARRAY_ITEMS,
                }));
            }
            Value::Array(sanitized)
        }
        Value::Object(object) => Value::Object(sanitize_tool_log_object(object, depth + 1)),
        Value::String(value) => Value::String(truncate_tool_log_string(&redact_text(value))),
        value => value.clone(),
    }
}

fn is_sensitive_log_key(key: &str) -> bool {
    let key = key.to_ascii_lowercase();
    if matches!(
        key.as_str(),
        "page_token" | "next_page_token" | "nextpagetoken"
    ) {
        return false;
    }

    [
        "authorization",
        "cookie",
        "password",
        "secret",
        "token",
        "api_token",
        "personal_token",
        "pat",
    ]
    .iter()
    .any(|sensitive| key.contains(sensitive))
}

fn truncate_tool_log_string(value: &str) -> String {
    if value.chars().count() <= TOOL_LOG_MAX_STRING_CHARS {
        return value.to_string();
    }

    let mut truncated = value
        .chars()
        .take(TOOL_LOG_MAX_STRING_CHARS)
        .collect::<String>();
    truncated.push_str(TOOL_LOG_TRUNCATED);
    truncated
}

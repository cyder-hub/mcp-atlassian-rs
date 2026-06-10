use serde_json::{Value, json};

use crate::{
    jira::config::{JiraConfig, JiraDeployment},
    upstream::{error::UpstreamError, redaction::redact_text},
};

pub fn extract_adf_text(value: &Value) -> String {
    let mut parts = Vec::new();
    collect_adf_text(value, &mut parts);
    parts.join("").trim().to_string()
}

pub fn cloud_adf_body(text: &str) -> Value {
    json!({
        "version": 1,
        "type": "doc",
        "content": [
            {
                "type": "paragraph",
                "content": [
                    {
                        "type": "text",
                        "text": text,
                    }
                ]
            }
        ]
    })
}

pub fn comment_body_for_deployment(deployment: JiraDeployment, text: &str) -> Value {
    match deployment {
        JiraDeployment::Cloud => cloud_adf_body(text),
        JiraDeployment::ServerDataCenter => Value::String(text.to_string()),
    }
}

pub fn parse_optional_object(
    value: Option<Value>,
    field_name: &'static str,
) -> Result<Option<Value>, UpstreamError> {
    let Some(value) = value else {
        return Ok(None);
    };

    match value {
        Value::Object(_) => Ok(Some(value)),
        Value::String(raw) => {
            let parsed: Value = serde_json::from_str(&raw).map_err(|_| {
                UpstreamError::invalid_input(format!("{field_name} must be a JSON object"))
            })?;

            if parsed.is_object() {
                Ok(Some(parsed))
            } else {
                Err(UpstreamError::invalid_input(format!(
                    "{field_name} must be a JSON object"
                )))
            }
        }
        _ => Err(UpstreamError::invalid_input(format!(
            "{field_name} must be a JSON object"
        ))),
    }
}

pub fn parse_required_object(
    value: Value,
    field_name: &'static str,
) -> Result<Value, UpstreamError> {
    parse_optional_object(Some(value), field_name)?
        .ok_or_else(|| UpstreamError::invalid_input(format!("{field_name} must be a JSON object")))
}

pub fn parse_optional_string_list(
    value: Option<Value>,
    field_name: &'static str,
) -> Result<Option<Vec<String>>, UpstreamError> {
    let Some(value) = value else {
        return Ok(None);
    };

    match value {
        Value::Array(values) => values
            .into_iter()
            .map(|value| match value {
                Value::String(value) if !value.trim().is_empty() => Ok(value.trim().to_string()),
                _ => Err(UpstreamError::invalid_input(format!(
                    "{field_name} must be a string or array of strings"
                ))),
            })
            .collect::<Result<Vec<_>, _>>()
            .map(Some),
        Value::String(value) => Ok(Some(
            value
                .split(',')
                .map(str::trim)
                .filter(|token| !token.is_empty())
                .map(ToString::to_string)
                .collect(),
        )),
        _ => Err(UpstreamError::invalid_input(format!(
            "{field_name} must be a string or array of strings"
        ))),
    }
}

pub fn parse_required_string_list(
    value: Value,
    field_name: &'static str,
) -> Result<Vec<String>, UpstreamError> {
    parse_optional_string_list(Some(value), field_name).map(|values| values.unwrap_or_default())
}

pub fn parse_required_object_list(
    value: Value,
    field_name: &'static str,
) -> Result<Vec<Value>, UpstreamError> {
    let parsed = match value {
        Value::Array(values) => values,
        Value::String(raw) => {
            let parsed: Value = serde_json::from_str(&raw).map_err(|_| {
                UpstreamError::invalid_input(format!("{field_name} must be a JSON array"))
            })?;
            parsed.as_array().cloned().ok_or_else(|| {
                UpstreamError::invalid_input(format!("{field_name} must be a JSON array"))
            })?
        }
        _ => {
            return Err(UpstreamError::invalid_input(format!(
                "{field_name} must be a JSON array"
            )));
        }
    };

    if parsed.iter().all(Value::is_object) {
        Ok(parsed)
    } else {
        Err(UpstreamError::invalid_input(format!(
            "{field_name} must contain only JSON objects"
        )))
    }
}

pub fn merge_optional_objects(
    base: Value,
    additional: Option<Value>,
    additional_field_name: &'static str,
) -> Result<Value, UpstreamError> {
    let mut base = parse_required_object(base, "fields")?;
    let Some(additional) = parse_optional_object(additional, additional_field_name)? else {
        return Ok(base);
    };

    let base_object = base
        .as_object_mut()
        .ok_or_else(|| UpstreamError::invalid_input("fields must be a JSON object"))?;
    let additional_object = additional.as_object().ok_or_else(|| {
        UpstreamError::invalid_input(format!("{additional_field_name} must be a JSON object"))
    })?;
    for (key, value) in additional_object {
        base_object.insert(key.clone(), value.clone());
    }

    Ok(base)
}

pub fn issue_project_key(issue_key: &str) -> Option<&str> {
    issue_key.split_once('-').map(|(project, _)| project)
}

pub fn ensure_issue_allowed(issue_key: &str, config: &JiraConfig) -> Result<(), UpstreamError> {
    if config.projects_filter.is_empty() {
        return Ok(());
    }

    let Some(project_key) = issue_project_key(issue_key) else {
        return Err(UpstreamError::invalid_input(
            "issue_key must include a project key prefix",
        ));
    };

    if config.projects_filter.contains(project_key) {
        Ok(())
    } else {
        Err(UpstreamError::invalid_input(format!(
            "issue `{issue_key}` is outside the configured Jira project filter"
        )))
    }
}

pub fn ensure_project_allowed(project_key: &str, config: &JiraConfig) -> Result<(), UpstreamError> {
    if config.projects_filter.is_empty() || config.projects_filter.contains(project_key) {
        Ok(())
    } else {
        Err(UpstreamError::invalid_input(format!(
            "project `{project_key}` is outside the configured Jira project filter"
        )))
    }
}

pub fn inject_project_filter(jql: &str, projects: &[String]) -> String {
    let jql = jql.trim();
    let has_order_by_only = jql.to_ascii_uppercase().starts_with("ORDER BY");

    if projects.is_empty() {
        if jql.is_empty() {
            return "issuekey IS NOT EMPTY".to_string();
        } else if has_order_by_only {
            return format!("issuekey IS NOT EMPTY {jql}");
        }
        return jql.to_string();
    }

    let project_clause = if projects.len() == 1 {
        format!("project = \"{}\"", escape_jql_string(&projects[0]))
    } else {
        let values = projects
            .iter()
            .map(|project| format!("\"{}\"", escape_jql_string(project)))
            .collect::<Vec<_>>()
            .join(", ");
        format!("project in ({values})")
    };

    let upper_jql = jql.to_ascii_uppercase();
    if let Some(idx) = upper_jql.rfind(" ORDER BY ") {
        let filter_part = jql[..idx].trim();
        let order_part = jql[idx..].trim();
        if filter_part.is_empty() {
            format!("{project_clause} {order_part}")
        } else {
            format!("({project_clause}) AND ({filter_part}) {order_part}")
        }
    } else if upper_jql.starts_with("ORDER BY") {
        format!("{project_clause} {jql}")
    } else if jql.is_empty() {
        project_clause
    } else {
        format!("({project_clause}) AND ({jql})")
    }
}

pub fn safe_path_segment(segment: &str, name: &'static str) -> Result<String, UpstreamError> {
    let segment = segment.trim();
    if segment.is_empty() || segment.contains('/') || segment.contains('?') || segment.contains('#')
    {
        Err(UpstreamError::invalid_input(format!(
            "{name} must be a non-empty path segment"
        )))
    } else {
        Ok(segment.to_string())
    }
}

pub fn base64_encode(bytes: &[u8]) -> String {
    const TABLE: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut output = String::with_capacity(bytes.len().div_ceil(3) * 4);

    for chunk in bytes.chunks(3) {
        let first = chunk[0];
        let second = chunk.get(1).copied().unwrap_or(0);
        let third = chunk.get(2).copied().unwrap_or(0);
        let combined = ((first as u32) << 16) | ((second as u32) << 8) | third as u32;

        output.push(TABLE[((combined >> 18) & 0x3f) as usize] as char);
        output.push(TABLE[((combined >> 12) & 0x3f) as usize] as char);
        if chunk.len() > 1 {
            output.push(TABLE[((combined >> 6) & 0x3f) as usize] as char);
        } else {
            output.push('=');
        }
        if chunk.len() > 2 {
            output.push(TABLE[(combined & 0x3f) as usize] as char);
        } else {
            output.push('=');
        }
    }

    output
}

pub fn redact_url_query(value: &str) -> String {
    redact_text(value)
}

fn collect_adf_text(value: &Value, parts: &mut Vec<String>) {
    match value {
        Value::Object(object) => {
            if object.get("type").and_then(Value::as_str) == Some("text")
                && let Some(text) = object.get("text").and_then(Value::as_str)
            {
                parts.push(text.to_string());
            }

            if let Some(content) = object.get("content") {
                collect_adf_text(content, parts);
            }
        }
        Value::Array(values) => {
            for value in values {
                collect_adf_text(value, parts);
            }
        }
        Value::String(value) => parts.push(value.clone()),
        _ => {}
    }
}

fn escape_jql_string(value: &str) -> String {
    value.replace('\\', "\\\\").replace('"', "\\\"")
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    #[test]
    fn extracts_minimal_adf_text() {
        let body = json!({
            "type": "doc",
            "content": [
                {"type": "paragraph", "content": [{"type": "text", "text": "Hello"}]},
                {"type": "paragraph", "content": [{"type": "text", "text": " world"}]}
            ]
        });

        assert_eq!(extract_adf_text(&body), "Hello world");
    }

    #[test]
    fn builds_minimal_cloud_adf_body() {
        let body = cloud_adf_body("Hello");

        assert_eq!(body["version"], 1);
        assert_eq!(body["content"][0]["content"][0]["text"], "Hello");
    }

    #[test]
    fn parses_json_object_from_string() {
        let parsed = parse_optional_object(
            Some(Value::String(r#"{"type":"role"}"#.to_string())),
            "visibility",
        )
        .unwrap()
        .unwrap();

        assert_eq!(parsed["type"], "role");
    }

    #[test]
    fn parses_required_object_list_from_json_string() {
        let parsed = parse_required_object_list(
            Value::String(r#"[{"name":"v1"},{"name":"v2"}]"#.to_string()),
            "versions",
        )
        .unwrap();

        assert_eq!(parsed.len(), 2);
        assert_eq!(parsed[0]["name"], "v1");
    }

    #[test]
    fn rejects_non_object_entries_in_required_object_list() {
        let error = parse_required_object_list(json!(["ABC-1"]), "issues").unwrap_err();

        assert!(
            error
                .to_string()
                .contains("issues must contain only JSON objects")
        );
    }

    #[test]
    fn parses_required_string_list_from_csv_or_array() {
        assert_eq!(
            parse_required_string_list(Value::String("ABC-1, ABC-2".to_string()), "issue_keys")
                .unwrap(),
            vec!["ABC-1".to_string(), "ABC-2".to_string()]
        );
        assert_eq!(
            parse_required_string_list(json!(["status", "assignee"]), "fields").unwrap(),
            vec!["status".to_string(), "assignee".to_string()]
        );
    }

    #[test]
    fn merges_optional_objects_with_later_values_winning() {
        let merged = merge_optional_objects(
            json!({"summary": "Old", "priority": {"name": "Low"}}),
            Some(json!({"priority": {"name": "High"}})),
            "additional_fields",
        )
        .unwrap();

        assert_eq!(merged["summary"], "Old");
        assert_eq!(merged["priority"]["name"], "High");
    }

    #[test]
    fn injects_project_filter_without_mutating_original_jql() {
        let jql = inject_project_filter("status = Done", &["ABC".to_string(), "XYZ".to_string()]);

        assert_eq!(jql, "(project in (\"ABC\", \"XYZ\")) AND (status = Done)");
    }

    #[test]
    fn base64_encodes_binary_content() {
        assert_eq!(base64_encode(b"image-bytes"), "aW1hZ2UtYnl0ZXM=");
        assert_eq!(base64_encode(b"a"), "YQ==");
        assert_eq!(base64_encode(b"ab"), "YWI=");
    }

    #[test]
    fn redacts_url_query_strings() {
        let message = r#"failed /secure/attachment/2/notes.txt?token=secret&client=abc", retry"#;
        let redacted = redact_url_query(message);

        assert!(redacted.contains("/secure/attachment/2/notes.txt?token=<redacted>&client=abc"));
        assert!(!redacted.contains("token=secret"));
        assert!(redacted.contains("client=abc"));
    }
}

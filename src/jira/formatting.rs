use serde_json::{Value, json};

use crate::{
    atlassian::error::AtlassianError,
    jira::config::{JiraConfig, JiraDeployment},
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
) -> Result<Option<Value>, AtlassianError> {
    let Some(value) = value else {
        return Ok(None);
    };

    match value {
        Value::Object(_) => Ok(Some(value)),
        Value::String(raw) => {
            let parsed: Value = serde_json::from_str(&raw).map_err(|_| {
                AtlassianError::invalid_input(format!("{field_name} must be a JSON object"))
            })?;

            if parsed.is_object() {
                Ok(Some(parsed))
            } else {
                Err(AtlassianError::invalid_input(format!(
                    "{field_name} must be a JSON object"
                )))
            }
        }
        _ => Err(AtlassianError::invalid_input(format!(
            "{field_name} must be a JSON object"
        ))),
    }
}

pub fn parse_optional_string_list(
    value: Option<Value>,
    field_name: &'static str,
) -> Result<Option<Vec<String>>, AtlassianError> {
    let Some(value) = value else {
        return Ok(None);
    };

    match value {
        Value::Array(values) => values
            .into_iter()
            .map(|value| match value {
                Value::String(value) if !value.trim().is_empty() => Ok(value.trim().to_string()),
                _ => Err(AtlassianError::invalid_input(format!(
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
        _ => Err(AtlassianError::invalid_input(format!(
            "{field_name} must be a string or array of strings"
        ))),
    }
}

pub fn issue_project_key(issue_key: &str) -> Option<&str> {
    issue_key.split_once('-').map(|(project, _)| project)
}

pub fn ensure_issue_allowed(issue_key: &str, config: &JiraConfig) -> Result<(), AtlassianError> {
    if config.projects_filter.is_empty() {
        return Ok(());
    }

    let Some(project_key) = issue_project_key(issue_key) else {
        return Err(AtlassianError::invalid_input(
            "issue_key must include a project key prefix",
        ));
    };

    if config.projects_filter.contains(project_key) {
        Ok(())
    } else {
        Err(AtlassianError::invalid_input(format!(
            "issue `{issue_key}` is outside the configured Jira project filter"
        )))
    }
}

pub fn inject_project_filter(jql: &str, projects: &[String]) -> String {
    if projects.is_empty() {
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

    if jql.trim().is_empty() {
        project_clause
    } else {
        format!("({project_clause}) AND ({jql})")
    }
}

pub fn safe_path_segment(segment: &str, name: &'static str) -> Result<String, AtlassianError> {
    let segment = segment.trim();
    if segment.is_empty() || segment.contains('/') || segment.contains('?') || segment.contains('#')
    {
        Err(AtlassianError::invalid_input(format!(
            "{name} must be a non-empty path segment"
        )))
    } else {
        Ok(segment.to_string())
    }
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
    fn injects_project_filter_without_mutating_original_jql() {
        let jql = inject_project_filter("status = Done", &["ABC".to_string(), "XYZ".to_string()]);

        assert_eq!(jql, "(project in (\"ABC\", \"XYZ\")) AND (status = Done)");
    }
}

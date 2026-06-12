use serde_json::{Value, json};

use super::{OperationError, OperationResult};

const EMPTY_CELL_PLACEHOLDER: &str = "-";

#[derive(Debug, Clone, PartialEq)]
pub enum OutputPresentation {
    KeyValue,
    Table { columns: Vec<&'static str> },
    MutationSummary { label: &'static str },
    Json,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct CliOutputOptions {
    pub json: bool,
    pub pretty: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RenderedOutput {
    pub stdout: String,
    pub stderr: String,
    pub exit_code: i32,
}

pub fn render_success(
    result: &OperationResult,
    options: CliOutputOptions,
) -> Result<RenderedOutput, serde_json::Error> {
    let stdout = if options.json {
        render_json(&result.value, options.pretty)?
    } else {
        render_text(result)?
    };

    Ok(RenderedOutput {
        stdout,
        stderr: String::new(),
        exit_code: 0,
    })
}

pub fn render_error(
    error: &OperationError,
    options: CliOutputOptions,
) -> Result<RenderedOutput, serde_json::Error> {
    let stderr = if options.json {
        render_json(
            &json!({
                "success": false,
                "error": {
                    "category": error.category.as_str(),
                    "message": error.message,
                }
            }),
            options.pretty,
        )?
    } else {
        error.to_string()
    };

    Ok(RenderedOutput {
        stdout: String::new(),
        stderr,
        exit_code: error.exit_code(),
    })
}

fn render_json(value: &Value, pretty: bool) -> Result<String, serde_json::Error> {
    if pretty {
        serde_json::to_string_pretty(value)
    } else {
        serde_json::to_string(value)
    }
}

fn render_text(result: &OperationResult) -> Result<String, serde_json::Error> {
    match &result.presentation {
        OutputPresentation::KeyValue => render_key_value(&result.value),
        OutputPresentation::Table { columns } => render_table(&result.value, columns),
        OutputPresentation::MutationSummary { label } => {
            render_mutation_summary(label, &result.value)
        }
        OutputPresentation::Json => render_json(&result.value, false),
    }
}

fn render_key_value(value: &Value) -> Result<String, serde_json::Error> {
    let Some(object) = value.as_object() else {
        return render_json(value, false);
    };
    let mut keys = object.keys().collect::<Vec<_>>();
    keys.sort();

    let mut lines = Vec::new();
    let mut table_sections = Vec::new();
    for key in keys {
        let value = &object[key];
        if let Some(rows) = tabular_array_rows(value) {
            table_sections.push((key.as_str(), rows));
        } else {
            lines.push(format!("{}: {}", key, render_cell_for_key(key, value)?));
        }
    }

    for (section, rows) in table_sections {
        if !lines.is_empty() {
            lines.push(String::new());
        }
        lines.push(format!("{section}:"));
        lines.push(render_inferred_table(rows)?);
    }

    Ok(lines.join("\n"))
}

fn render_table(value: &Value, columns: &[&str]) -> Result<String, serde_json::Error> {
    let rows = table_rows(value);
    let mut lines = Vec::with_capacity(rows.len() + 1);
    lines.push(columns.join("\t"));

    for row in rows {
        let object = row.as_object();
        let mut cells = Vec::with_capacity(columns.len());
        for column in columns {
            let value = object
                .and_then(|object| object.get(*column))
                .unwrap_or(&Value::Null);
            cells.push(render_cell_for_key(column, value)?);
        }
        lines.push(cells.join("\t"));
    }

    Ok(lines.join("\n"))
}

fn table_rows(value: &Value) -> Vec<&Value> {
    if let Some(rows) = value.as_array() {
        return rows.iter().collect();
    }

    for key in ["items", "results", "values", "issues"] {
        if let Some(rows) = value.get(key).and_then(Value::as_array) {
            return rows.iter().collect();
        }
    }

    Vec::new()
}

fn tabular_array_rows(value: &Value) -> Option<Vec<&Value>> {
    let rows = value.as_array()?;
    if rows.is_empty() {
        return None;
    }
    if rows.iter().any(Value::is_object) {
        Some(rows.iter().collect())
    } else {
        None
    }
}

fn render_inferred_table(rows: Vec<&Value>) -> Result<String, serde_json::Error> {
    let columns = infer_table_columns(&rows);
    if columns.is_empty() {
        return render_json(&Value::Array(rows.into_iter().cloned().collect()), false);
    }

    let column_refs = columns.iter().map(String::as_str).collect::<Vec<_>>();
    render_table_rows(&rows, &column_refs)
}

fn infer_table_columns(rows: &[&Value]) -> Vec<String> {
    const PREFERRED_COLUMNS: &[&str] = &[
        "key",
        "iid",
        "id",
        "title",
        "name",
        "summary",
        "status",
        "state",
        "type",
        "issue_type",
        "project",
        "space",
        "assignee",
        "reporter",
        "author",
        "display_name",
        "username",
        "email",
        "filename",
        "mime_type",
        "size",
        "source_branch",
        "target_branch",
        "sha",
        "created",
        "created_at",
        "updated",
        "updated_at",
        "url",
        "web_url",
        "success",
        "message",
    ];

    let mut present = std::collections::BTreeSet::new();
    for row in rows {
        let Some(object) = row.as_object() else {
            continue;
        };
        for (key, value) in object {
            if is_table_cell_value(key, value) {
                present.insert(key.as_str());
            }
        }
    }

    let mut columns = Vec::new();
    for preferred in PREFERRED_COLUMNS {
        if present.remove(preferred) {
            columns.push((*preferred).to_string());
        }
    }
    columns.extend(present.into_iter().map(ToString::to_string));
    columns
}

fn is_table_cell_value(key: &str, value: &Value) -> bool {
    match value {
        Value::Null | Value::Bool(_) | Value::Number(_) | Value::String(_) => true,
        Value::Object(_) => summarize_object_cell(key, value).is_some(),
        Value::Array(values) => values.iter().all(|value| {
            matches!(
                value,
                Value::Null | Value::Bool(_) | Value::Number(_) | Value::String(_)
            )
        }),
    }
}

fn render_table_rows(rows: &[&Value], columns: &[&str]) -> Result<String, serde_json::Error> {
    let mut lines = Vec::with_capacity(rows.len() + 1);
    lines.push(columns.join("\t"));

    for row in rows {
        let object = row.as_object();
        let mut cells = Vec::with_capacity(columns.len());
        for column in columns {
            let value = object
                .and_then(|object| object.get(*column))
                .unwrap_or(&Value::Null);
            cells.push(render_cell_for_key(column, value)?);
        }
        lines.push(cells.join("\t"));
    }

    Ok(lines.join("\n"))
}

fn render_mutation_summary(label: &str, value: &Value) -> Result<String, serde_json::Error> {
    let Some(object) = value.as_object() else {
        return Ok(format!("{label}: {}", render_cell(value)?));
    };
    let status = object
        .get("success")
        .map(|value| render_cell_for_key("success", value))
        .transpose()?
        .unwrap_or_else(|| "ok".to_string());
    let mut lines = vec![format!("{label}: {status}")];

    for key in ["id", "key", "message"] {
        if let Some(value) = object.get(key) {
            lines.push(format!("{key}: {}", render_cell_for_key(key, value)?));
        }
    }

    Ok(lines.join("\n"))
}

fn render_cell_for_key(key: &str, value: &Value) -> Result<String, serde_json::Error> {
    Ok(match value {
        Value::Array(values)
            if values.iter().all(|value| {
                matches!(
                    value,
                    Value::Null | Value::Bool(_) | Value::Number(_) | Value::String(_)
                )
            }) =>
        {
            if values.is_empty() {
                EMPTY_CELL_PLACEHOLDER.to_string()
            } else {
                values
                    .iter()
                    .map(|value| render_cell_for_key(key, value))
                    .collect::<Result<Vec<_>, _>>()?
                    .join(", ")
            }
        }
        Value::Object(_) => summarize_object_cell(key, value).unwrap_or_else(|| {
            if is_nullish_object(value) {
                EMPTY_CELL_PLACEHOLDER.to_string()
            } else {
                serde_json::to_string(value).unwrap_or_default()
            }
        }),
        _ => render_cell(value)?,
    })
}

fn render_cell(value: &Value) -> Result<String, serde_json::Error> {
    Ok(match value {
        Value::Null => EMPTY_CELL_PLACEHOLDER.to_string(),
        Value::String(value) if value.is_empty() => EMPTY_CELL_PLACEHOLDER.to_string(),
        Value::String(value) => value.clone(),
        Value::Bool(value) => value.to_string(),
        Value::Number(value) => value.to_string(),
        Value::Array(_) | Value::Object(_) => serde_json::to_string(value)?,
    })
}

fn summarize_object_cell(key: &str, value: &Value) -> Option<String> {
    let object = value.as_object()?;
    let preferred = match key {
        "assignee" | "reporter" | "author" | "user" | "owner" => [
            "display_name",
            "displayName",
            "username",
            "name",
            "email",
            "account_id",
            "id",
        ]
        .as_slice(),
        "project" | "space" => ["key", "name", "title", "id"].as_slice(),
        "status" | "state" | "issue_type" | "type" | "priority" | "category" => {
            ["name", "key", "value", "id"].as_slice()
        }
        _ => [
            "display_name",
            "displayName",
            "name",
            "title",
            "key",
            "value",
            "username",
            "email",
            "id",
        ]
        .as_slice(),
    };

    for field in preferred {
        if let Some(value) = object.get(*field).and_then(scalar_cell_string)
            && !value.trim().is_empty()
        {
            return Some(value);
        }
    }

    None
}

fn scalar_cell_string(value: &Value) -> Option<String> {
    match value {
        Value::Null | Value::Array(_) | Value::Object(_) => None,
        Value::String(value) => Some(value.clone()),
        Value::Bool(value) => Some(value.to_string()),
        Value::Number(value) => Some(value.to_string()),
    }
}

fn is_nullish_object(value: &Value) -> bool {
    let Some(object) = value.as_object() else {
        return false;
    };
    object.values().all(is_nullish_value)
}

fn is_nullish_value(value: &Value) -> bool {
    match value {
        Value::Null => true,
        Value::Array(values) => values.iter().all(is_nullish_value),
        Value::Object(_) => is_nullish_object(value),
        Value::Bool(_) | Value::Number(_) | Value::String(_) => false,
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::super::{OperationErrorCategory, OperationResult};
    use super::*;

    #[test]
    fn operations_output_renders_table_text_to_stdout_only() {
        let result = OperationResult::success(json!([
            {"key": "PROJ-1", "summary": "First"},
            {"key": "PROJ-2", "summary": "Second"}
        ]))
        .with_presentation(OutputPresentation::Table {
            columns: vec!["key", "summary"],
        });
        let output = render_success(&result, CliOutputOptions::default()).unwrap();

        assert_eq!(output.stdout, "key\tsummary\nPROJ-1\tFirst\nPROJ-2\tSecond");
        assert!(output.stderr.is_empty());
        assert_eq!(output.exit_code, 0);
    }

    #[test]
    fn operations_output_renders_key_value_text() {
        let result = OperationResult::success(json!({"key": "PROJ-1", "status": "Done"}));
        let output = render_success(&result, CliOutputOptions::default()).unwrap();

        assert_eq!(output.stdout, "key: PROJ-1\nstatus: Done");
    }

    #[test]
    fn operations_output_renders_nested_object_array_as_table() {
        let result = OperationResult::success(json!({
            "is_last": true,
            "total": 2,
            "issues": [
                {
                    "id": "10001",
                    "key": "ABC-1",
                    "summary": "First",
                    "status": {"name": "Done"},
                    "assignee": {"display_name": "Ada"},
                    "project": {"key": "ABC", "name": "Alpha"},
                    "fields": {"customfield_10000": "hidden from table"}
                },
                {
                    "id": "10002",
                    "key": "ABC-2",
                    "summary": "Second",
                    "status": {"name": "To Do"},
                    "assignee": {
                        "account_id": null,
                        "name": null,
                        "display_name": null,
                        "email_address": null
                    },
                    "project": {"key": "ABC"}
                }
            ]
        }));
        let output = render_success(&result, CliOutputOptions::default()).unwrap();

        assert_eq!(
            output.stdout,
            concat!(
                "is_last: true\n",
                "total: 2\n",
                "\n",
                "issues:\n",
                "key\tid\tsummary\tstatus\tproject\tassignee\n",
                "ABC-1\t10001\tFirst\tDone\tABC\tAda\n",
                "ABC-2\t10002\tSecond\tTo Do\tABC\t-"
            )
        );
    }

    #[test]
    fn operations_output_keeps_primitive_arrays_inline() {
        let result = OperationResult::success(json!({
            "labels": ["bug", "urgent"],
            "key": "ABC-1"
        }));
        let output = render_success(&result, CliOutputOptions::default()).unwrap();

        assert_eq!(output.stdout, "key: ABC-1\nlabels: bug, urgent");
    }

    #[test]
    fn operations_output_renders_nullish_object_cell_as_empty() {
        let result = OperationResult::success(json!({
            "assignee": {
                "account_id": null,
                "name": null,
                "display_name": null,
                "email_address": null
            },
            "key": "ABC-1"
        }));
        let output = render_success(&result, CliOutputOptions::default()).unwrap();

        assert_eq!(output.stdout, "assignee: -\nkey: ABC-1");
    }

    #[test]
    fn operations_output_renders_missing_table_cell_as_placeholder() {
        let result = OperationResult::success(json!({
            "issues": [
                {"key": "ABC-1", "summary": "First", "assignee": "Ada"},
                {"key": "ABC-2", "summary": "", "assignee": null}
            ]
        }));
        let output = render_success(&result, CliOutputOptions::default()).unwrap();

        assert_eq!(
            output.stdout,
            "issues:\nkey\tsummary\tassignee\nABC-1\tFirst\tAda\nABC-2\t-\t-"
        );
    }

    #[test]
    fn operations_output_renders_mutation_summary_text() {
        let result = OperationResult::success(json!({
            "success": true,
            "key": "PROJ-1",
            "message": "updated"
        }))
        .with_presentation(OutputPresentation::MutationSummary { label: "issue" });
        let output = render_success(&result, CliOutputOptions::default()).unwrap();

        assert_eq!(output.stdout, "issue: true\nkey: PROJ-1\nmessage: updated");
    }

    #[test]
    fn operations_output_renders_compact_and_pretty_json_success() {
        let result = OperationResult::success(json!({"success": true}));
        let compact = render_success(
            &result,
            CliOutputOptions {
                json: true,
                pretty: false,
            },
        )
        .unwrap();
        let pretty = render_success(
            &result,
            CliOutputOptions {
                json: true,
                pretty: true,
            },
        )
        .unwrap();

        assert_eq!(compact.stdout, r#"{"success":true}"#);
        assert!(pretty.stdout.contains('\n'));
        assert!(compact.stderr.is_empty());
    }

    #[test]
    fn operations_output_renders_errors_to_stderr_and_empty_stdout() {
        let error = OperationError::new(OperationErrorCategory::Business, "partial failure");
        let output = render_error(
            &error,
            CliOutputOptions {
                json: true,
                pretty: false,
            },
        )
        .unwrap();

        assert!(output.stdout.is_empty());
        assert_eq!(output.exit_code, 5);
        assert_eq!(
            output.stderr,
            r#"{"error":{"category":"business","message":"partial failure"},"success":false}"#
        );
    }
}

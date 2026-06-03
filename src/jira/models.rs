use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

use crate::jira::formatting::extract_adf_text;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct JiraIssue {
    pub id: Option<String>,
    pub key: String,
    #[serde(default)]
    pub fields: Value,
    #[serde(flatten)]
    pub extra: Value,
}

impl JiraIssue {
    pub fn to_simplified_value(&self) -> Value {
        let fields = self.fields.as_object();
        json!({
            "id": self.id,
            "key": self.key,
            "summary": fields.and_then(|fields| fields.get("summary")).and_then(Value::as_str),
            "status": fields.and_then(|fields| fields.get("status")).map(simplify_status),
            "assignee": fields.and_then(|fields| fields.get("assignee")).map(simplify_user),
            "reporter": fields.and_then(|fields| fields.get("reporter")).map(simplify_user),
            "issue_type": fields.and_then(|fields| fields.get("issuetype")).map(simplify_named),
            "priority": fields.and_then(|fields| fields.get("priority")).map(simplify_named),
            "project": fields.and_then(|fields| fields.get("project")).map(simplify_named),
            "fields": self.fields,
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct JiraSearchResult {
    #[serde(default)]
    pub issues: Vec<JiraIssue>,
    pub total: Option<u64>,
    pub start_at: Option<u64>,
    pub max_results: Option<u64>,
    pub next_page_token: Option<String>,
    pub is_last: Option<bool>,
}

impl JiraSearchResult {
    pub fn to_simplified_value(&self) -> Value {
        json!({
            "issues": self.issues.iter().map(JiraIssue::to_simplified_value).collect::<Vec<_>>(),
            "total": self.total,
            "start_at": self.start_at,
            "max_results": self.max_results,
            "next_page_token": self.next_page_token,
            "is_last": self.is_last,
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct JiraField {
    pub id: String,
    pub key: Option<String>,
    pub name: String,
    pub custom: Option<bool>,
    pub orderable: Option<bool>,
    pub navigable: Option<bool>,
    pub searchable: Option<bool>,
    #[serde(default)]
    pub schema: Value,
}

impl JiraField {
    pub fn to_simplified_value(&self) -> Value {
        json!({
            "id": self.id,
            "key": self.key,
            "name": self.name,
            "custom": self.custom,
            "orderable": self.orderable,
            "navigable": self.navigable,
            "searchable": self.searchable,
            "schema": self.schema,
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct JiraFieldOption {
    pub id: Option<String>,
    pub value: Option<String>,
    pub name: Option<String>,
    pub disabled: Option<bool>,
    #[serde(default)]
    pub children: Vec<JiraFieldOption>,
    #[serde(flatten)]
    pub extra: Value,
}

impl JiraFieldOption {
    pub fn label(&self) -> Option<&str> {
        self.value.as_deref().or(self.name.as_deref())
    }

    pub fn to_simplified_value(&self) -> Value {
        json!({
            "id": self.id,
            "value": self.label(),
            "disabled": self.disabled,
            "children": self.children.iter().map(JiraFieldOption::to_simplified_value).collect::<Vec<_>>(),
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct JiraComment {
    pub id: String,
    #[serde(default)]
    pub body: Value,
    #[serde(default)]
    pub author: Value,
    pub created: Option<String>,
    pub updated: Option<String>,
    #[serde(default)]
    pub visibility: Option<Value>,
}

impl JiraComment {
    pub fn to_simplified_value(&self) -> Value {
        json!({
            "id": self.id,
            "body": comment_body_text(&self.body),
            "author": simplify_user(&self.author),
            "created": self.created,
            "updated": self.updated,
            "visibility": self.visibility,
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct JiraTransition {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub to: Value,
    pub has_screen: Option<bool>,
    pub is_global: Option<bool>,
    pub is_initial: Option<bool>,
    pub is_conditional: Option<bool>,
    #[serde(default)]
    pub fields: Value,
}

impl JiraTransition {
    pub fn to_simplified_value(&self) -> Value {
        json!({
            "id": self.id,
            "name": self.name,
            "to": simplify_status(&self.to),
            "has_screen": self.has_screen,
            "is_global": self.is_global,
            "is_initial": self.is_initial,
            "is_conditional": self.is_conditional,
            "fields": self.fields,
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct JiraTransitionsResponse {
    #[serde(default)]
    pub transitions: Vec<JiraTransition>,
}

impl JiraTransitionsResponse {
    pub fn to_simplified_value(&self) -> Value {
        json!({
            "transitions": self.transitions.iter().map(JiraTransition::to_simplified_value).collect::<Vec<_>>(),
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct JiraFieldOptionsResponse {
    #[serde(default)]
    pub values: Vec<JiraFieldOption>,
}

impl JiraFieldOptionsResponse {
    pub fn to_simplified_value(&self, values_only: bool) -> Value {
        if values_only {
            json!(
                self.values
                    .iter()
                    .filter_map(|option| option.label().map(ToString::to_string))
                    .collect::<Vec<_>>()
            )
        } else {
            json!({
                "values": self.values.iter().map(JiraFieldOption::to_simplified_value).collect::<Vec<_>>(),
            })
        }
    }
}

pub fn simplify_fields(fields: &[JiraField]) -> Value {
    json!(
        fields
            .iter()
            .map(JiraField::to_simplified_value)
            .collect::<Vec<_>>()
    )
}

pub fn simplify_options(options: &[JiraFieldOption], values_only: bool) -> Value {
    let response = JiraFieldOptionsResponse {
        values: options.to_vec(),
    };
    response.to_simplified_value(values_only)
}

pub fn simplify_comment(comment: &JiraComment) -> Value {
    comment.to_simplified_value()
}

fn simplify_user(value: &Value) -> Value {
    json!({
        "account_id": value.get("accountId").and_then(Value::as_str),
        "name": value.get("name").and_then(Value::as_str),
        "display_name": value.get("displayName").and_then(Value::as_str),
        "email_address": value.get("emailAddress").and_then(Value::as_str),
    })
}

fn simplify_status(value: &Value) -> Value {
    json!({
        "id": value.get("id").and_then(Value::as_str),
        "name": value.get("name").and_then(Value::as_str),
        "description": value.get("description").and_then(Value::as_str),
        "status_category": value.get("statusCategory").map(simplify_named),
    })
}

fn simplify_named(value: &Value) -> Value {
    json!({
        "id": value.get("id").and_then(Value::as_str),
        "key": value.get("key").and_then(Value::as_str),
        "name": value.get("name").and_then(Value::as_str),
    })
}

fn comment_body_text(value: &Value) -> String {
    match value {
        Value::String(value) => value.clone(),
        _ => extract_adf_text(value),
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    #[test]
    fn issue_simplification_tolerates_missing_fields() {
        let issue: JiraIssue = serde_json::from_value(json!({
            "id": "10001",
            "key": "ABC-1",
            "fields": {
                "summary": "Demo",
                "status": {"id": "1", "name": "Done"}
            }
        }))
        .unwrap();
        let simplified = issue.to_simplified_value();

        assert_eq!(simplified["key"], "ABC-1");
        assert_eq!(simplified["summary"], "Demo");
        assert_eq!(simplified["status"]["name"], "Done");
        assert!(simplified["assignee"]["display_name"].is_null());
    }

    #[test]
    fn search_result_simplifies_issues_and_pagination() {
        let result: JiraSearchResult = serde_json::from_value(json!({
            "issues": [{"key": "ABC-1", "fields": {}}],
            "nextPageToken": "next",
            "isLast": false
        }))
        .unwrap();
        let simplified = result.to_simplified_value();

        assert_eq!(simplified["issues"][0]["key"], "ABC-1");
        assert_eq!(simplified["next_page_token"], "next");
    }

    #[test]
    fn field_options_support_value_and_name_shapes() {
        let options: Vec<JiraFieldOption> = serde_json::from_value(json!([
            {"id": "1", "value": "High"},
            {"id": "2", "name": "Low"}
        ]))
        .unwrap();
        let simplified = simplify_options(&options, true);

        assert_eq!(simplified, json!(["High", "Low"]));
    }

    #[test]
    fn comment_simplification_extracts_adf_body() {
        let comment: JiraComment = serde_json::from_value(json!({
            "id": "10",
            "body": {
                "type": "doc",
                "content": [{"type": "paragraph", "content": [{"type": "text", "text": "Hello"}]}]
            },
            "author": {"displayName": "Ada"}
        }))
        .unwrap();
        let simplified = comment.to_simplified_value();

        assert_eq!(simplified["body"], "Hello");
        assert_eq!(simplified["author"]["display_name"], "Ada");
    }

    #[test]
    fn transitions_simplify_status() {
        let response: JiraTransitionsResponse = serde_json::from_value(json!({
            "transitions": [{"id": "31", "name": "Done", "to": {"id": "3", "name": "Done"}}]
        }))
        .unwrap();
        let simplified = response.to_simplified_value();

        assert_eq!(simplified["transitions"][0]["id"], "31");
        assert_eq!(simplified["transitions"][0]["to"]["name"], "Done");
    }
}

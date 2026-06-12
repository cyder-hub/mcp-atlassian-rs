use std::collections::BTreeSet;

use serde_json::{Map, Value, json};

#[cfg(test)]
use crate::upstream::{custom_headers::CustomHeaders, proxy::ProxyConfig};
use crate::{
    jira::{
        config::{JiraConfig, JiraDeployment},
        formatting::{
            base64_encode, comment_body_for_deployment, ensure_issue_allowed,
            ensure_project_allowed, inject_project_filter, merge_optional_objects,
            parse_optional_object, redact_url_query, safe_path_segment,
        },
        models::{
            JiraAttachment, JiraComment, JiraField, JiraFieldOption, JiraFieldOptionsResponse,
            JiraFieldSearchResponse, JiraIssue, JiraOperationResult, JiraPaginatedValues,
            JiraSearchResult, JiraTransitionsResponse, simplify_comment, simplify_fields,
            simplify_options,
        },
    },
    upstream::{
        error::UpstreamError,
        http::{DownloadedContent, UpstreamHttpClient},
    },
};

pub const DEFAULT_LIMIT: u64 = 50;
pub const DEFAULT_ATTACHMENT_MAX_BYTES: u64 = 1_048_576;

#[derive(Clone, Debug)]
pub struct JiraClient {
    config: JiraConfig,
    http: UpstreamHttpClient,
}

#[derive(Debug, Clone, Default)]
pub struct GetIssueRequest {
    pub issue_key: String,
    pub fields: Option<Vec<String>>,
    pub expand: Option<Vec<String>>,
    pub comment_limit: Option<u64>,
    pub properties: Option<Vec<String>>,
    pub update_history: Option<bool>,
}

#[derive(Debug, Clone)]
pub struct AttachmentFetchOptions {
    pub attachment_ids: Option<Vec<String>>,
    pub filename_contains: Option<String>,
    pub media_type: Option<String>,
    pub include_content: bool,
    pub images_only: bool,
    pub max_bytes: u64,
}

impl Default for AttachmentFetchOptions {
    fn default() -> Self {
        Self {
            attachment_ids: None,
            filename_contains: None,
            media_type: None,
            include_content: false,
            images_only: false,
            max_bytes: DEFAULT_ATTACHMENT_MAX_BYTES,
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct SearchRequest {
    pub jql: String,
    pub fields: Option<Vec<String>>,
    pub limit: Option<u64>,
    pub start_at: Option<u64>,
    pub projects_filter: Option<Vec<String>>,
    pub expand: Option<Vec<String>>,
    pub page_token: Option<String>,
}

#[derive(Debug, Clone, Default)]
pub struct FieldOptionsRequest {
    pub field_id: String,
    pub context_id: Option<String>,
    pub project_key: Option<String>,
    pub issue_type: Option<String>,
    pub contains: Option<String>,
    pub return_limit: Option<u64>,
    pub values_only: bool,
}

mod agile;
mod attachments;
mod comments;
mod development;
mod fields;
mod issues;
mod metrics;
mod projects;
mod search;
mod service_desk;
mod transitions;

impl JiraClient {
    pub fn new(config: JiraConfig) -> Result<Self, UpstreamError> {
        let http = UpstreamHttpClient::new_with_proxy_headers_and_mtls(
            &config.base_url,
            config.auth.clone(),
            config.timeout_seconds,
            config.ssl_verify,
            config.proxy.clone(),
            config.custom_headers.clone(),
            config.mtls.clone(),
        )?;
        Ok(Self { config, http })
    }

    pub async fn get_user_profile(&self, user_identifier: String) -> Result<Value, UpstreamError> {
        let identifier = user_identifier.trim();
        if identifier.eq_ignore_ascii_case("currentuser()") || identifier.eq_ignore_ascii_case("me")
        {
            return self
                .http
                .send_json(self.http.get("/rest/api/2/myself")?)
                .await;
        }

        let query_key = match self.config.deployment {
            JiraDeployment::Cloud => "accountId",
            JiraDeployment::ServerDataCenter => "username",
        };
        self.http
            .send_json(
                self.http
                    .get("/rest/api/2/user")?
                    .query(&[(query_key, identifier)]),
            )
            .await
    }
}

fn is_cloud_unbounded_jql_error(error: &UpstreamError) -> bool {
    matches!(
        error,
        UpstreamError::HttpStatus { status: 400, message }
            if message.contains("Unbounded JQL queries are not allowed here")
    )
}

fn is_removed_cloud_legacy_search_error(error: &UpstreamError) -> bool {
    matches!(
        error,
        UpstreamError::HttpStatus { status: 410, message }
            if message.contains("/rest/api/3/search/jql")
    )
}

fn cloud_unbounded_jql_error(jql: &str) -> UpstreamError {
    UpstreamError::invalid_input(format!(
        "Jira Cloud rejected an unbounded JQL query and the legacy search API is removed. Add a search restriction such as `project = \"KEY\"`, `issuekey in (...)`, or a configured JIRA_PROJECTS_FILTER before the order clause. Rejected JQL: {jql}"
    ))
}

fn cloud_offset_pagination_removed_error() -> UpstreamError {
    UpstreamError::invalid_input(
        "Jira Cloud offset pagination with start_at requires the removed /rest/api/3/search API. Use page_token from a previous jira_search_issues response instead.",
    )
}

fn field_context_mapping_context_id(value: &Value) -> Result<Option<String>, UpstreamError> {
    let Some(context_id) = value.get("contextId") else {
        return Err(UpstreamError::unexpected_shape(
            "field context mapping response did not include contextId",
        ));
    };
    if context_id.is_null() {
        return Ok(None);
    }

    field_value_id(value, "contextId")
        .map(Some)
        .ok_or_else(|| UpstreamError::unexpected_shape("contextId must be a string or integer"))
}

fn cloud_issue_type_matches(value: &Value, requested: &str) -> bool {
    value
        .get("id")
        .and_then(Value::as_str)
        .is_some_and(|id| id == requested)
        || value
            .get("name")
            .and_then(Value::as_str)
            .is_some_and(|name| name.eq_ignore_ascii_case(requested))
}

fn field_value_id(value: &Value, field_name: &str) -> Option<String> {
    value.get(field_name).and_then(|id| {
        id.as_str()
            .map(ToString::to_string)
            .or_else(|| id.as_u64().map(|id| id.to_string()))
    })
}

fn jira_issue_status_summary(issue: &JiraIssue, status_changes: &[Value]) -> Value {
    json!({
        "current_status": issue
            .fields
            .get("status")
            .map(jira_status_identity)
            .unwrap_or(Value::Null),
        "created": issue.fields.get("created").cloned().unwrap_or(Value::Null),
        "updated": issue.fields.get("updated").cloned().unwrap_or(Value::Null),
        "due_date": issue.fields.get("duedate").cloned().unwrap_or(Value::Null),
        "resolution_date": issue.fields.get("resolutiondate").cloned().unwrap_or(Value::Null),
        "has_changelog": issue.extra.get("changelog").is_some(),
        "transition_count": status_changes.len(),
        "first_transition": status_changes.first().cloned().unwrap_or(Value::Null),
        "last_transition": status_changes.last().cloned().unwrap_or(Value::Null),
    })
}

fn jira_issue_status_changes(issue: &JiraIssue) -> Vec<Value> {
    let mut changes = Vec::new();
    let Some(histories) = issue
        .extra
        .get("changelog")
        .and_then(|changelog| changelog.get("histories"))
        .and_then(Value::as_array)
    else {
        return changes;
    };

    for history in histories {
        let changed_at = history.get("created").cloned().unwrap_or(Value::Null);
        let history_id = history.get("id").cloned().unwrap_or(Value::Null);
        let Some(items) = history.get("items").and_then(Value::as_array) else {
            continue;
        };
        for item in items {
            if !jira_changelog_item_is_status(item) {
                continue;
            }
            changes.push(json!({
                "history_id": history_id,
                "changed_at": changed_at,
                "from": jira_changelog_status_side(item, "from", "fromString"),
                "to": jira_changelog_status_side(item, "to", "toString"),
            }));
        }
    }

    changes
}

fn jira_changelog_item_is_status(item: &Value) -> bool {
    ["field", "fieldId"]
        .into_iter()
        .filter_map(|field| item.get(field).and_then(Value::as_str))
        .any(|field| field.eq_ignore_ascii_case("status"))
}

fn jira_changelog_status_side(item: &Value, id_field: &str, name_field: &str) -> Value {
    json!({
        "id": field_value_id(item, id_field),
        "name": item.get(name_field).and_then(Value::as_str),
    })
}

fn jira_status_identity(status: &Value) -> Value {
    json!({
        "id": field_value_id(status, "id"),
        "name": status.get("name").and_then(Value::as_str),
        "status_category": status
            .get("statusCategory")
            .map(jira_status_category_identity)
            .unwrap_or(Value::Null),
    })
}

fn jira_status_category_identity(category: &Value) -> Value {
    json!({
        "id": field_value_id(category, "id"),
        "key": category.get("key").and_then(Value::as_str),
        "name": category.get("name").and_then(Value::as_str),
    })
}

fn optional_query_params<const N: usize>(
    pairs: [(&str, Option<String>); N],
) -> Vec<(String, String)> {
    pairs
        .into_iter()
        .filter_map(|(key, value)| value.map(|value| (key.to_string(), value)))
        .collect()
}

fn jira_software_agile_unavailable(error: UpstreamError) -> Result<Value, UpstreamError> {
    match error {
        UpstreamError::HttpStatus {
            status: 403 | 404,
            message,
        } => Ok(JiraOperationResult::product_unavailable(
            "Jira Software Agile REST",
            format!("Jira Software Agile REST is unavailable: {message}"),
        )
        .to_simplified_value()),
        error => Err(error),
    }
}

fn jira_service_management_unavailable(error: UpstreamError) -> Result<Value, UpstreamError> {
    match error {
        UpstreamError::HttpStatus {
            status: 403 | 404,
            message,
        } => Ok(JiraOperationResult::product_unavailable(
            "Jira Service Management",
            format!("Jira Service Management REST is unavailable: {message}"),
        )
        .to_simplified_value()),
        error => Err(error),
    }
}

fn jira_development_unavailable(error: UpstreamError) -> Result<Value, UpstreamError> {
    match error {
        UpstreamError::HttpStatus {
            status: 403 | 404,
            message,
        } => Ok(JiraOperationResult::product_unavailable(
            "Jira development/dev-status",
            format!("Jira development/dev-status REST is unavailable: {message}"),
        )
        .to_simplified_value()),
        error => Err(error),
    }
}

fn extract_sla_metric_values(
    fields: Option<&Map<String, Value>>,
    requested_metrics: Option<&[String]>,
    include_raw_dates: bool,
) -> Vec<Value> {
    let Some(fields) = fields else {
        return Vec::new();
    };
    let requested = requested_metrics
        .unwrap_or_default()
        .iter()
        .map(|metric| normalize_metric_key(metric))
        .filter(|metric| !metric.is_empty())
        .collect::<Vec<_>>();
    let include_all_sla_like = requested.is_empty();

    fields
        .iter()
        .filter(|(field_id, value)| {
            sla_metric_matches(field_id, value, &requested, include_all_sla_like)
        })
        .map(|(field_id, value)| {
            json!({
                "field_id": field_id,
                "name": value.get("name").and_then(Value::as_str),
                "value": simplify_sla_value(value, include_raw_dates),
            })
        })
        .collect()
}

fn sla_metric_matches(
    field_id: &str,
    value: &Value,
    requested_metrics: &[String],
    include_all_sla_like: bool,
) -> bool {
    let field_id = field_id.to_ascii_lowercase();
    let name = value
        .get("name")
        .and_then(Value::as_str)
        .unwrap_or_default()
        .to_ascii_lowercase();
    let field_id_normalized = normalize_metric_key(&field_id);
    let name_normalized = normalize_metric_key(&name);

    if include_all_sla_like {
        field_id.contains("sla") || name.contains("sla")
    } else {
        requested_metrics.iter().any(|metric| {
            metric == &field_id_normalized
                || metric == &name_normalized
                || (!name_normalized.is_empty() && name_normalized.contains(metric))
                || (!name_normalized.is_empty() && metric.contains(&name_normalized))
        })
    }
}

fn normalize_metric_key(value: &str) -> String {
    value
        .chars()
        .filter(|character| character.is_ascii_alphanumeric())
        .flat_map(char::to_lowercase)
        .collect()
}

fn simplify_sla_value(value: &Value, include_raw_dates: bool) -> Value {
    if include_raw_dates {
        return value.clone();
    }

    let Some(object) = value.as_object() else {
        return value.clone();
    };
    let mut simplified = object.clone();
    for key in [
        "startTime",
        "stopTime",
        "breachTime",
        "pauseTime",
        "rawStartTime",
        "rawStopTime",
        "rawBreachTime",
    ] {
        simplified.remove(key);
    }
    Value::Object(simplified)
}

fn insert_optional(target: &mut Value, key: &'static str, value: Option<Value>) {
    if let Some(value) = value
        && let Some(object) = target.as_object_mut()
    {
        object.insert(key.to_string(), value);
    }
}

fn extract_createmeta_options(
    value: &Value,
    field_id: &str,
) -> Result<Vec<JiraFieldOption>, UpstreamError> {
    let projects = value
        .get("projects")
        .and_then(Value::as_array)
        .ok_or_else(|| UpstreamError::unexpected_shape("createmeta response missing projects"))?;

    for project in projects {
        let Some(issue_types) = project.get("issuetypes").and_then(Value::as_array) else {
            continue;
        };
        for issue_type in issue_types {
            let Some(fields) = issue_type.get("fields").and_then(Value::as_object) else {
                continue;
            };
            let Some(field) = fields.get(field_id) else {
                continue;
            };
            let options = field
                .get("allowedValues")
                .and_then(Value::as_array)
                .ok_or_else(|| {
                    UpstreamError::unexpected_shape("createmeta field missing allowedValues")
                })?;
            return options
                .iter()
                .cloned()
                .map(serde_json::from_value)
                .collect::<Result<Vec<JiraFieldOption>, _>>()
                .map_err(|error| UpstreamError::unexpected_shape(error.to_string()));
        }
    }

    Err(UpstreamError::unexpected_shape(format!(
        "createmeta response missing field `{field_id}`"
    )))
}

#[cfg(test)]
mod tests;

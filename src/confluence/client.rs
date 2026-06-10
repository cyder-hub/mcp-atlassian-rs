use std::{fs, path::Path};

use reqwest::multipart;
use serde::{Serialize, de::DeserializeOwned};
use serde_json::{Value, json};

#[cfg(test)]
use crate::upstream::{custom_headers::CustomHeaders, proxy::ProxyConfig};
use crate::{
    confluence::{
        config::{ConfluenceConfig, ConfluenceDeployment},
        formatting::{body_value_as_storage, safe_path_segment},
        models::{
            ConfluenceAttachment, ConfluenceAttachmentListResponse, ConfluenceComment,
            ConfluenceCommentListResponse, ConfluenceLabelListResponse, ConfluencePage,
            ConfluencePageListResponse, ConfluencePageViews, ConfluenceSearchResponse,
            ConfluenceUserListResponse, ConfluenceUserSearchResponse, ConfluenceUserSearchResult,
        },
    },
    upstream::{
        error::UpstreamError,
        http::{DownloadedContent, UpstreamHttpClient},
        redaction::redact_text,
    },
};

pub const DEFAULT_LIMIT: u64 = 25;
pub const DEFAULT_ATTACHMENT_MAX_BYTES: u64 = 1_048_576;
pub const DEFAULT_UPLOAD_ATTACHMENT_MAX_BYTES: u64 = 10 * 1024 * 1024;
pub const DEFAULT_ATTACHMENT_LIST_LIMIT: u64 = 50;
pub const MAX_ATTACHMENT_LIST_LIMIT: u64 = 100;
pub const DEFAULT_SEARCH_LIMIT: u64 = 10;
pub const MAX_SEARCH_LIMIT: u64 = 50;
pub const DEFAULT_USER_SEARCH_LIMIT: u64 = 10;
pub const MAX_USER_SEARCH_LIMIT: u64 = 50;
const SERVER_USER_SEARCH_PAGE_SIZE: u64 = 200;
const DEFAULT_CONFLUENCE_GROUP_NAME: &str = "confluence-users";

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct ConfluencePageChildrenResponse {
    pub results: Vec<ConfluencePage>,
    pub page_results: usize,
    pub folder_results: usize,
    pub page_query: ConfluenceChildrenQueryStats,
    pub folder_query: Option<ConfluenceChildrenQueryStats>,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct ConfluenceChildrenQueryStats {
    pub child_type: &'static str,
    pub requested_start: u64,
    pub requested_limit: u64,
    pub response_start: Option<u64>,
    pub response_limit: Option<u64>,
    pub response_size: Option<u64>,
    pub result_count: usize,
    pub has_more: bool,
    pub next_start: Option<u64>,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct ConfluenceEmojiStatus {
    pub requested: bool,
    pub applied: bool,
    pub emoji: Option<String>,
    pub error: Option<String>,
}

impl ConfluenceEmojiStatus {
    pub fn not_requested() -> Self {
        Self {
            requested: false,
            applied: false,
            emoji: None,
            error: None,
        }
    }

    pub fn failed(emoji: String, error: impl Into<String>) -> Self {
        Self {
            requested: true,
            applied: false,
            emoji: Some(emoji),
            error: Some(redact_text(&error.into())),
        }
    }

    fn applied(emoji: String) -> Self {
        Self {
            requested: true,
            applied: true,
            emoji: Some(emoji),
            error: None,
        }
    }
}

struct ConfluenceUpdatePageWithSpaceRequest<'a> {
    page_id: &'a str,
    space_key: &'a str,
    title: &'a str,
    storage_body: &'a str,
    parent_id: Option<&'a str>,
    version: u64,
    is_minor_edit: bool,
    version_comment: Option<&'a str>,
}

#[derive(Clone, Debug)]
pub struct ConfluenceClient {
    config: ConfluenceConfig,
    http: UpstreamHttpClient,
}

mod analytics;
mod attachments;
mod comments;
mod labels;
mod pages;
mod users;

impl ConfluenceClient {
    pub fn new(config: ConfluenceConfig) -> Result<Self, UpstreamError> {
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

    pub fn config(&self) -> &ConfluenceConfig {
        &self.config
    }

    pub async fn get_json<T>(
        &self,
        path: &str,
        query: Vec<(String, String)>,
    ) -> Result<T, UpstreamError>
    where
        T: DeserializeOwned,
    {
        let builder = self.http.get(path)?.query(&query);
        self.http.send_json(builder).await
    }

    pub async fn download_relative_or_same_origin(
        &self,
        url: &str,
        max_bytes: u64,
    ) -> Result<DownloadedContent, UpstreamError> {
        let builder = self
            .http
            .get_same_origin_or_relative_url(url, "download_url")?;
        self.http.send_bytes_limited(builder, max_bytes).await
    }
}

fn page_write_payload(
    page_id: Option<&str>,
    space_key: String,
    title: String,
    storage_body: &str,
    parent_id: Option<&str>,
    version: Option<u64>,
    version_options: Option<(bool, Option<&str>)>,
) -> Result<Value, UpstreamError> {
    let mut payload = json!({
        "type": "page",
        "title": title,
        "space": {"key": space_key},
        "body": {
            "storage": {
                "value": storage_body,
                "representation": "storage"
            }
        }
    });

    if let Some(page_id) = page_id {
        payload["id"] = Value::String(page_id.to_string());
    }
    if let Some(parent_id) = optional_non_empty_input(parent_id) {
        payload["ancestors"] = json!([{ "id": parent_id }]);
    }
    if let Some(version) = version {
        let (minor_edit, message) = version_options.unwrap_or((false, None));
        payload["version"] = json!({
            "number": version,
            "minorEdit": minor_edit,
        });
        if let Some(message) = optional_non_empty_input(message) {
            payload["version"]["message"] = Value::String(message);
        }
    }

    Ok(payload)
}

fn comment_payload(
    container_id: &str,
    container_type: &'static str,
    storage_body: &str,
) -> Result<Value, UpstreamError> {
    let container_id = safe_path_segment(container_id, "container_id")?;
    Ok(json!({
        "type": "comment",
        "container": {
            "id": container_id,
            "type": container_type,
        },
        "body": {
            "storage": {
                "value": storage_body,
                "representation": "storage"
            }
        }
    }))
}

fn required_non_empty_input(
    value: &str,
    field_name: &'static str,
) -> Result<String, UpstreamError> {
    let value = value.trim();
    if value.is_empty() {
        Err(UpstreamError::invalid_input(format!(
            "{field_name} must not be empty"
        )))
    } else {
        Ok(value.to_string())
    }
}

fn optional_non_empty_input(value: Option<&str>) -> Option<String> {
    value
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToString::to_string)
}

fn children_query_stats(
    child_type: &'static str,
    requested_start: u64,
    requested_limit: u64,
    response: &ConfluencePageListResponse,
) -> ConfluenceChildrenQueryStats {
    ConfluenceChildrenQueryStats {
        child_type,
        requested_start,
        requested_limit,
        response_start: response.start,
        response_limit: response.limit,
        response_size: response.size,
        result_count: response.results.len(),
        has_more: response.has_next_link(),
        next_start: response.next_start(),
    }
}

fn search_limit(value: Option<u64>) -> Result<u64, UpstreamError> {
    match value.unwrap_or(DEFAULT_SEARCH_LIMIT) {
        0 => Err(UpstreamError::invalid_input("limit must be positive")),
        value if value > MAX_SEARCH_LIMIT => Err(UpstreamError::invalid_input(format!(
            "limit must be less than or equal to {MAX_SEARCH_LIMIT}"
        ))),
        value => Ok(value),
    }
}

fn user_search_limit(value: Option<u64>) -> Result<u64, UpstreamError> {
    match value.unwrap_or(DEFAULT_USER_SEARCH_LIMIT) {
        0 => Err(UpstreamError::invalid_input("limit must be positive")),
        value if value > MAX_USER_SEARCH_LIMIT => Err(UpstreamError::invalid_input(format!(
            "limit must be less than or equal to {MAX_USER_SEARCH_LIMIT}"
        ))),
        value => Ok(value),
    }
}

fn attachment_list_limit(value: Option<u64>) -> Result<u64, UpstreamError> {
    match value.unwrap_or(DEFAULT_ATTACHMENT_LIST_LIMIT) {
        0 => Err(UpstreamError::invalid_input("limit must be positive")),
        value if value > MAX_ATTACHMENT_LIST_LIMIT => Err(UpstreamError::invalid_input(format!(
            "limit must be less than or equal to {MAX_ATTACHMENT_LIST_LIMIT}"
        ))),
        value => Ok(value),
    }
}

fn attachment_multipart_form(
    filename: &str,
    file_bytes: Vec<u8>,
    comment: Option<&str>,
    minor_edit: bool,
) -> Result<multipart::Form, UpstreamError> {
    let file_part = multipart::Part::bytes(file_bytes)
        .file_name(filename.to_string())
        .mime_str("application/octet-stream")
        .map_err(|error| {
            UpstreamError::invalid_input(format!("failed to build multipart file part: {error}"))
        })?;
    let mut form = multipart::Form::new().part("file", file_part);

    if let Some(comment) = comment.map(str::trim).filter(|value| !value.is_empty()) {
        form = form.text("comment", comment.to_string());
    }
    form = form.text("minorEdit", if minor_edit { "true" } else { "false" });

    Ok(form)
}

fn confluence_attachment_from_upload_response(
    value: Value,
) -> Result<ConfluenceAttachment, UpstreamError> {
    let attachment = value
        .get("results")
        .and_then(Value::as_array)
        .and_then(|results| results.first())
        .cloned()
        .unwrap_or(value);
    if attachment.is_null() {
        return Err(UpstreamError::unexpected_shape(
            "attachment upload response is missing an attachment result",
        ));
    }

    serde_json::from_value(attachment).map_err(|error| {
        UpstreamError::json_decode_body(error, Some("Confluence attachment upload response"))
    })
}

fn is_simple_search_query(query: &str) -> bool {
    !["=", "~", ">", "<", " AND ", " OR ", "currentUser()"]
        .iter()
        .any(|token| query.contains(token))
}

fn escape_cql_string(value: &str) -> String {
    value.replace('\\', "\\\\").replace('"', "\\\"")
}

fn extract_user_fullname_search_term(cql: &str) -> Option<&str> {
    let marker = "user.fullname";
    let marker_index = cql.find(marker)?;
    let after_marker = &cql[marker_index + marker.len()..];
    let tilde_index = after_marker.find('~')?;
    let after_tilde = after_marker[tilde_index + 1..].trim_start();
    let quoted = after_tilde.strip_prefix('"')?;
    let end = quoted.find('"')?;
    Some(&quoted[..end])
}

fn percent_encode_path_segment(value: &str) -> String {
    let mut output = String::with_capacity(value.len());
    for byte in value.bytes() {
        match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                output.push(byte as char)
            }
            _ => output.push_str(&format!("%{byte:02X}")),
        }
    }
    output
}

fn spaces_filter_values(spaces_filter: Option<&str>, config: &ConfluenceConfig) -> Vec<String> {
    match spaces_filter {
        Some(value) => value
            .split(',')
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(ToString::to_string)
            .collect(),
        None => config.spaces_filter.iter().cloned().collect(),
    }
}

fn apply_spaces_filter(cql: &str, spaces: Vec<String>) -> String {
    if spaces.is_empty() || cql_contains_space_filter(cql) {
        return cql.to_string();
    }

    let space_query = spaces
        .iter()
        .map(|space| format!("space = {}", quote_cql_identifier_if_needed(space)))
        .collect::<Vec<_>>()
        .join(" OR ");

    if cql.trim().is_empty() {
        space_query
    } else {
        format!("({cql}) AND ({space_query})")
    }
}

fn cql_contains_space_filter(cql: &str) -> bool {
    let normalized = cql.to_ascii_lowercase().replace(char::is_whitespace, "");
    normalized.contains("space=")
}

fn quote_cql_identifier_if_needed(identifier: &str) -> String {
    let starts_with_digit = identifier
        .chars()
        .next()
        .is_some_and(|character| character.is_ascii_digit());
    let has_special_character = !identifier
        .chars()
        .all(|character| character.is_ascii_alphanumeric() || character == '_');
    let is_reserved = matches!(
        identifier.to_ascii_lowercase().as_str(),
        "and" | "or" | "not" | "in" | "order" | "by" | "space"
    );

    if identifier.starts_with('~') || starts_with_digit || has_special_character || is_reserved {
        format!("\"{}\"", escape_cql_string(identifier))
    } else {
        identifier.to_string()
    }
}

#[cfg(test)]
mod tests;

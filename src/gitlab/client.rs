use crate::{
    gitlab::config::GitlabConfig,
    upstream::{error::UpstreamError, http::UpstreamHttpClient},
};
use serde_json::{Map, Value, json};

pub const DEFAULT_GITLAB_PER_PAGE: u64 = 20;
pub const MAX_GITLAB_PER_PAGE: u64 = 100;
pub const DEFAULT_GITLAB_DIFF_MAX_BYTES: u64 = 65_536;

#[derive(Clone, Debug)]
pub struct GitlabClient {
    config: GitlabConfig,
    http: UpstreamHttpClient,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ListMergeRequestsRequest {
    pub project: String,
    pub state: Option<String>,
    pub author_username: Option<String>,
    pub reviewer_username: Option<String>,
    pub source_branch: Option<String>,
    pub target_branch: Option<String>,
    pub labels: Option<Vec<String>>,
    pub page: Option<u64>,
    pub per_page: Option<u64>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct GetMergeRequestRequest {
    pub project: String,
    pub merge_request_iid: u64,
    pub include_diverged_commits_count: Option<bool>,
    pub include_rebase_in_progress: Option<bool>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ListMergeRequestCommitsRequest {
    pub project: String,
    pub merge_request_iid: u64,
    pub page: Option<u64>,
    pub per_page: Option<u64>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ListMergeRequestDiffsRequest {
    pub project: String,
    pub merge_request_iid: u64,
    pub max_diff_bytes: Option<u64>,
    pub page: Option<u64>,
    pub per_page: Option<u64>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ListMergeRequestPipelinesRequest {
    pub project: String,
    pub merge_request_iid: u64,
    pub page: Option<u64>,
    pub per_page: Option<u64>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct CreateMergeRequestRequest {
    pub project: String,
    pub source_branch: String,
    pub target_branch: String,
    pub title: String,
    pub description: Option<String>,
    pub remove_source_branch: Option<bool>,
    pub squash: Option<bool>,
    pub assignee_ids: Option<Vec<u64>>,
    pub reviewer_ids: Option<Vec<u64>>,
    pub labels: Option<Vec<String>>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct UpdateMergeRequestRequest {
    pub project: String,
    pub merge_request_iid: u64,
    pub title: Option<String>,
    pub description: Option<String>,
    pub state_event: Option<String>,
    pub labels: Option<Vec<String>>,
    pub add_labels: Option<Vec<String>>,
    pub remove_labels: Option<Vec<String>>,
    pub reviewer_ids: Option<Vec<u64>>,
    pub assignee_ids: Option<Vec<u64>>,
    pub target_branch: Option<String>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct AcceptMergeRequestRequest {
    pub project: String,
    pub merge_request_iid: u64,
    pub sha: String,
    pub auto_merge: Option<bool>,
    pub squash: Option<bool>,
    pub should_remove_source_branch: Option<bool>,
    pub merge_commit_message: Option<String>,
    pub squash_commit_message: Option<String>,
}

impl GitlabClient {
    pub fn new(config: GitlabConfig) -> Result<Self, UpstreamError> {
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

    pub async fn get_current_user(&self) -> Result<Value, UpstreamError> {
        self.get_json("/user", Vec::new()).await
    }

    pub async fn get_project(&self, project: &str) -> Result<Value, UpstreamError> {
        let project = self.project_api_segment(project)?;
        self.get_json(&format!("/projects/{project}"), Vec::new())
            .await
    }

    pub async fn list_merge_requests(
        &self,
        request: ListMergeRequestsRequest,
    ) -> Result<Value, UpstreamError> {
        let project = self.project_api_segment(&request.project)?;
        let mut query = pagination_query(request.page, request.per_page)?;
        push_optional_query(&mut query, "state", request.state);
        push_optional_query(&mut query, "author_username", request.author_username);
        push_optional_query(&mut query, "reviewer_username", request.reviewer_username);
        push_optional_query(&mut query, "source_branch", request.source_branch);
        push_optional_query(&mut query, "target_branch", request.target_branch);
        if let Some(labels) = request.labels {
            let labels = labels
                .into_iter()
                .map(|label| label.trim().to_string())
                .filter(|label| !label.is_empty())
                .collect::<Vec<_>>();
            if !labels.is_empty() {
                query.push(("labels".to_string(), labels.join(",")));
            }
        }

        self.get_json(&format!("/projects/{project}/merge_requests"), query)
            .await
    }

    pub async fn get_merge_request(
        &self,
        request: GetMergeRequestRequest,
    ) -> Result<Value, UpstreamError> {
        let project = self.project_api_segment(&request.project)?;
        let mut query = Vec::new();
        push_optional_bool_query(
            &mut query,
            "include_diverged_commits_count",
            request.include_diverged_commits_count,
        );
        push_optional_bool_query(
            &mut query,
            "include_rebase_in_progress",
            request.include_rebase_in_progress,
        );

        self.get_json(
            &format!(
                "/projects/{project}/merge_requests/{}",
                request.merge_request_iid
            ),
            query,
        )
        .await
    }

    pub async fn list_merge_request_commits(
        &self,
        request: ListMergeRequestCommitsRequest,
    ) -> Result<Value, UpstreamError> {
        let project = self.project_api_segment(&request.project)?;
        self.get_json(
            &format!(
                "/projects/{project}/merge_requests/{}/commits",
                request.merge_request_iid
            ),
            pagination_query(request.page, request.per_page)?,
        )
        .await
    }

    pub async fn list_merge_request_diffs(
        &self,
        request: ListMergeRequestDiffsRequest,
    ) -> Result<Value, UpstreamError> {
        let project = self.project_api_segment(&request.project)?;
        let max_diff_bytes = request
            .max_diff_bytes
            .unwrap_or(DEFAULT_GITLAB_DIFF_MAX_BYTES);
        if max_diff_bytes == 0 {
            return Err(UpstreamError::invalid_input(
                "max_diff_bytes must be positive",
            ));
        }
        let value = self
            .get_json(
                &format!(
                    "/projects/{project}/merge_requests/{}/diffs",
                    request.merge_request_iid
                ),
                pagination_query(request.page, request.per_page)?,
            )
            .await?;

        Ok(bounded_diff_response(value, max_diff_bytes))
    }

    pub async fn list_merge_request_pipelines(
        &self,
        request: ListMergeRequestPipelinesRequest,
    ) -> Result<Value, UpstreamError> {
        let project = self.project_api_segment(&request.project)?;
        self.get_json(
            &format!(
                "/projects/{project}/merge_requests/{}/pipelines",
                request.merge_request_iid
            ),
            pagination_query(request.page, request.per_page)?,
        )
        .await
    }

    pub async fn create_merge_request(
        &self,
        request: CreateMergeRequestRequest,
    ) -> Result<Value, UpstreamError> {
        let project = self.project_api_segment(&request.project)?;
        let mut body = Map::new();
        insert_required_string(&mut body, "source_branch", request.source_branch)?;
        insert_required_string(&mut body, "target_branch", request.target_branch)?;
        insert_required_string(&mut body, "title", request.title)?;
        insert_optional_string(&mut body, "description", request.description);
        insert_optional_bool(
            &mut body,
            "remove_source_branch",
            request.remove_source_branch,
        );
        insert_optional_bool(&mut body, "squash", request.squash);
        insert_optional_u64_list(&mut body, "assignee_ids", request.assignee_ids);
        insert_optional_u64_list(&mut body, "reviewer_ids", request.reviewer_ids);
        insert_optional_labels(&mut body, "labels", request.labels);

        self.send_json_value_or_null(
            self.http
                .post_json(&format!("/api/v4/projects/{project}/merge_requests"), &body)?,
        )
        .await
    }

    pub async fn update_merge_request(
        &self,
        request: UpdateMergeRequestRequest,
    ) -> Result<Value, UpstreamError> {
        let project = self.project_api_segment(&request.project)?;
        let mut body = Map::new();
        insert_optional_string_preserving_empty(&mut body, "title", request.title);
        insert_optional_string_preserving_empty(&mut body, "description", request.description);
        insert_optional_string_preserving_empty(&mut body, "state_event", request.state_event);
        insert_optional_string_preserving_empty(&mut body, "target_branch", request.target_branch);
        insert_optional_labels_preserving_empty(&mut body, "labels", request.labels);
        insert_optional_labels_preserving_empty(&mut body, "add_labels", request.add_labels);
        insert_optional_labels_preserving_empty(&mut body, "remove_labels", request.remove_labels);
        insert_optional_u64_list_preserving_empty(&mut body, "reviewer_ids", request.reviewer_ids);
        insert_optional_u64_list_preserving_empty(&mut body, "assignee_ids", request.assignee_ids);

        self.send_json_value_or_null(self.http.put_json(
            &format!(
                "/api/v4/projects/{project}/merge_requests/{}",
                request.merge_request_iid
            ),
            &body,
        )?)
        .await
    }

    pub async fn add_merge_request_note(
        &self,
        project: &str,
        merge_request_iid: u64,
        body: String,
    ) -> Result<Value, UpstreamError> {
        let project = self.project_api_segment(project)?;
        let body = required_non_empty(body, "body")?;

        self.send_json_value_or_null(self.http.post_json(
            &format!("/api/v4/projects/{project}/merge_requests/{merge_request_iid}/notes"),
            &json!({ "body": body }),
        )?)
        .await
    }

    pub async fn reply_merge_request_discussion(
        &self,
        project: &str,
        merge_request_iid: u64,
        discussion_id: &str,
        body: String,
    ) -> Result<Value, UpstreamError> {
        let project = self.project_api_segment(project)?;
        let discussion_id = percent_encode_path_segment(&required_non_empty(
            discussion_id.to_string(),
            "discussion_id",
        )?);
        let body = required_non_empty(body, "body")?;

        self.send_json_value_or_null(self.http.post_json(
            &format!(
                "/api/v4/projects/{project}/merge_requests/{merge_request_iid}/discussions/{discussion_id}/notes"
            ),
            &json!({ "body": body }),
        )?)
        .await
    }

    pub async fn resolve_merge_request_discussion(
        &self,
        project: &str,
        merge_request_iid: u64,
        discussion_id: &str,
        resolved: bool,
    ) -> Result<Value, UpstreamError> {
        let project = self.project_api_segment(project)?;
        let discussion_id = percent_encode_path_segment(&required_non_empty(
            discussion_id.to_string(),
            "discussion_id",
        )?);

        self.send_json_value_or_null(self.http.put_json(
            &format!(
                "/api/v4/projects/{project}/merge_requests/{merge_request_iid}/discussions/{discussion_id}"
            ),
            &json!({ "resolved": resolved }),
        )?)
        .await
    }

    pub async fn get_merge_request_approval_state(
        &self,
        project: &str,
        merge_request_iid: u64,
    ) -> Result<Value, UpstreamError> {
        let project = self.project_api_segment(project)?;
        self.get_json(
            &format!("/projects/{project}/merge_requests/{merge_request_iid}/approval_state"),
            Vec::new(),
        )
        .await
    }

    pub async fn approve_merge_request(
        &self,
        project: &str,
        merge_request_iid: u64,
    ) -> Result<Value, UpstreamError> {
        let project = self.project_api_segment(project)?;
        self.send_json_value_or_null(self.http.post_json(
            &format!("/api/v4/projects/{project}/merge_requests/{merge_request_iid}/approve"),
            &json!({}),
        )?)
        .await
    }

    pub async fn unapprove_merge_request(
        &self,
        project: &str,
        merge_request_iid: u64,
    ) -> Result<Value, UpstreamError> {
        let project = self.project_api_segment(project)?;
        self.send_json_value_or_null(self.http.post_json(
            &format!("/api/v4/projects/{project}/merge_requests/{merge_request_iid}/unapprove"),
            &json!({}),
        )?)
        .await
    }

    pub async fn accept_merge_request(
        &self,
        request: AcceptMergeRequestRequest,
    ) -> Result<Value, UpstreamError> {
        let project = self.project_api_segment(&request.project)?;
        let mut body = Map::new();
        insert_required_string(&mut body, "sha", request.sha)?;
        insert_optional_bool(&mut body, "auto_merge", request.auto_merge);
        insert_optional_bool(&mut body, "squash", request.squash);
        insert_optional_bool(
            &mut body,
            "should_remove_source_branch",
            request.should_remove_source_branch,
        );
        insert_optional_string(
            &mut body,
            "merge_commit_message",
            request.merge_commit_message,
        );
        insert_optional_string(
            &mut body,
            "squash_commit_message",
            request.squash_commit_message,
        );

        self.send_json_value_or_null(self.http.put_json(
            &format!(
                "/api/v4/projects/{project}/merge_requests/{}/merge",
                request.merge_request_iid
            ),
            &body,
        )?)
        .await
    }

    async fn get_json(
        &self,
        path: &str,
        query: Vec<(String, String)>,
    ) -> Result<Value, UpstreamError> {
        let builder = self.http.get(&self.api_path(path))?.query(&query);
        self.http.send_json_value_or_null(builder).await
    }

    async fn send_json_value_or_null(
        &self,
        builder: reqwest::RequestBuilder,
    ) -> Result<Value, UpstreamError> {
        self.http.send_json_value_or_null(builder).await
    }

    fn api_path(&self, path: &str) -> String {
        format!("/api/v4/{}", path.trim_start_matches('/'))
    }

    fn project_api_segment(&self, project: &str) -> Result<String, UpstreamError> {
        let project = normalized_project_ref(project)?;
        self.ensure_project_allowed(&project)?;
        if project.bytes().all(|byte| byte.is_ascii_digit()) {
            Ok(project)
        } else {
            Ok(percent_encode_path_segment(&project))
        }
    }

    fn ensure_project_allowed(&self, project: &str) -> Result<(), UpstreamError> {
        if self.config.projects_filter.is_empty() || self.config.projects_filter.contains(project) {
            return Ok(());
        }

        Err(UpstreamError::invalid_input(format!(
            "GitLab project `{project}` is not allowed by GITLAB_PROJECTS_FILTER"
        )))
    }
}

fn pagination_query(
    page: Option<u64>,
    per_page: Option<u64>,
) -> Result<Vec<(String, String)>, UpstreamError> {
    let page = page.unwrap_or(1);
    let per_page = per_page.unwrap_or(DEFAULT_GITLAB_PER_PAGE);
    if page == 0 {
        return Err(UpstreamError::invalid_input("page must be positive"));
    }
    if per_page == 0 || per_page > MAX_GITLAB_PER_PAGE {
        return Err(UpstreamError::invalid_input(format!(
            "per_page must be between 1 and {MAX_GITLAB_PER_PAGE}"
        )));
    }

    Ok(vec![
        ("page".to_string(), page.to_string()),
        ("per_page".to_string(), per_page.to_string()),
    ])
}

fn push_optional_query(
    query: &mut Vec<(String, String)>,
    name: &'static str,
    value: Option<String>,
) {
    if let Some(value) = value.map(|value| value.trim().to_string())
        && !value.is_empty()
    {
        query.push((name.to_string(), value));
    }
}

fn push_optional_bool_query(
    query: &mut Vec<(String, String)>,
    name: &'static str,
    value: Option<bool>,
) {
    if let Some(value) = value {
        query.push((name.to_string(), value.to_string()));
    }
}

fn bounded_diff_response(value: Value, max_diff_bytes: u64) -> Value {
    let Value::Array(items) = value else {
        return serde_json::json!({
            "truncated": false,
            "max_diff_bytes": max_diff_bytes,
            "diff_bytes": 0,
            "diffs": value,
        });
    };

    let mut truncated = false;
    let mut used = 0_u64;
    let mut bounded_items = Vec::with_capacity(items.len());

    for mut item in items {
        if let Some(diff) = item
            .get("diff")
            .and_then(Value::as_str)
            .map(ToString::to_string)
        {
            let diff_bytes = diff.len() as u64;
            if used >= max_diff_bytes {
                *item.get_mut("diff").expect("diff was just present") =
                    Value::String(String::new());
                truncated = true;
            } else if used + diff_bytes > max_diff_bytes {
                let remaining = (max_diff_bytes - used) as usize;
                let prefix = utf8_prefix(&diff, remaining).to_string();
                *item.get_mut("diff").expect("diff was just present") = Value::String(prefix);
                used = max_diff_bytes;
                truncated = true;
            } else {
                used += diff_bytes;
            }
        }
        bounded_items.push(item);
    }

    serde_json::json!({
        "truncated": truncated,
        "max_diff_bytes": max_diff_bytes,
        "diff_bytes": used,
        "diffs": bounded_items,
    })
}

fn utf8_prefix(value: &str, max_bytes: usize) -> &str {
    if value.len() <= max_bytes {
        return value;
    }
    let mut end = max_bytes;
    while !value.is_char_boundary(end) {
        end -= 1;
    }
    &value[..end]
}

fn insert_required_string(
    body: &mut Map<String, Value>,
    name: &'static str,
    value: String,
) -> Result<(), UpstreamError> {
    body.insert(
        name.to_string(),
        Value::String(required_non_empty(value, name)?),
    );
    Ok(())
}

fn insert_optional_string(
    body: &mut Map<String, Value>,
    name: &'static str,
    value: Option<String>,
) {
    if let Some(value) = value.map(|value| value.trim().to_string())
        && !value.is_empty()
    {
        body.insert(name.to_string(), Value::String(value));
    }
}

fn insert_optional_string_preserving_empty(
    body: &mut Map<String, Value>,
    name: &'static str,
    value: Option<String>,
) {
    if let Some(value) = value {
        body.insert(name.to_string(), Value::String(value.trim().to_string()));
    }
}

fn insert_optional_bool(body: &mut Map<String, Value>, name: &'static str, value: Option<bool>) {
    if let Some(value) = value {
        body.insert(name.to_string(), Value::Bool(value));
    }
}

fn insert_optional_u64_list(
    body: &mut Map<String, Value>,
    name: &'static str,
    value: Option<Vec<u64>>,
) {
    if let Some(value) = value
        && !value.is_empty()
    {
        body.insert(name.to_string(), json!(value));
    }
}

fn insert_optional_u64_list_preserving_empty(
    body: &mut Map<String, Value>,
    name: &'static str,
    value: Option<Vec<u64>>,
) {
    if let Some(value) = value {
        body.insert(name.to_string(), json!(value));
    }
}

fn insert_optional_labels(
    body: &mut Map<String, Value>,
    name: &'static str,
    value: Option<Vec<String>>,
) {
    if let Some(labels) = normalized_string_list(value)
        && !labels.is_empty()
    {
        body.insert(name.to_string(), Value::String(labels.join(",")));
    }
}

fn insert_optional_labels_preserving_empty(
    body: &mut Map<String, Value>,
    name: &'static str,
    value: Option<Vec<String>>,
) {
    if let Some(labels) = normalized_string_list(value) {
        body.insert(name.to_string(), Value::String(labels.join(",")));
    }
}

fn normalized_string_list(value: Option<Vec<String>>) -> Option<Vec<String>> {
    value.map(|values| {
        values
            .into_iter()
            .map(|value| value.trim().to_string())
            .filter(|value| !value.is_empty())
            .collect()
    })
}

fn required_non_empty(value: String, name: &'static str) -> Result<String, UpstreamError> {
    let value = value.trim();
    if value.is_empty() {
        Err(UpstreamError::invalid_input(format!(
            "{name} must not be empty"
        )))
    } else {
        Ok(value.to_string())
    }
}

fn normalized_project_ref(project: &str) -> Result<String, UpstreamError> {
    let project = project.trim();
    if project.is_empty() {
        return Err(UpstreamError::invalid_input("project must not be empty"));
    }

    Ok(percent_decode_utf8(project).unwrap_or_else(|| project.to_string()))
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

fn percent_decode_utf8(value: &str) -> Option<String> {
    let mut bytes = Vec::with_capacity(value.len());
    let mut index = 0;
    let input = value.as_bytes();

    while index < input.len() {
        if input[index] == b'%' {
            let high = *input.get(index + 1)?;
            let low = *input.get(index + 2)?;
            let high = hex_value(high)?;
            let low = hex_value(low)?;
            bytes.push((high << 4) | low);
            index += 3;
        } else {
            bytes.push(input[index]);
            index += 1;
        }
    }

    String::from_utf8(bytes).ok()
}

fn hex_value(byte: u8) -> Option<u8> {
    match byte {
        b'0'..=b'9' => Some(byte - b'0'),
        b'a'..=b'f' => Some(byte - b'a' + 10),
        b'A'..=b'F' => Some(byte - b'A' + 10),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use std::{collections::BTreeSet, net::SocketAddr, sync::Arc};

    use axum::{
        Json, Router,
        body::Bytes,
        extract::State,
        http::{HeaderMap, Method, StatusCode},
        response::{IntoResponse, Response},
        routing::any,
    };
    use serde_json::json;
    use tokio::sync::Mutex;

    use crate::upstream::{
        auth::UpstreamAuth, custom_headers::CustomHeaders, mtls::ClientTlsIdentityConfig,
        proxy::ProxyConfig,
    };

    use super::*;

    #[derive(Debug, Clone, PartialEq)]
    struct RecordedRequest {
        method: Method,
        path: String,
        authorization: Option<String>,
        private_token: Option<String>,
        body: Value,
    }

    #[derive(Clone)]
    struct MockState {
        response: Value,
        status: StatusCode,
        requests: Arc<Mutex<Vec<RecordedRequest>>>,
    }

    async fn mock_handler(
        State(state): State<MockState>,
        method: Method,
        headers: HeaderMap,
        uri: axum::http::Uri,
        body: Bytes,
    ) -> Response {
        let parsed_body = if body.is_empty() {
            Value::Null
        } else {
            serde_json::from_slice(&body).unwrap()
        };
        state.requests.lock().await.push(RecordedRequest {
            method,
            path: uri
                .path_and_query()
                .map(ToString::to_string)
                .unwrap_or_else(|| uri.path().to_string()),
            authorization: headers
                .get("authorization")
                .and_then(|value| value.to_str().ok())
                .map(ToString::to_string),
            private_token: headers
                .get("private-token")
                .and_then(|value| value.to_str().ok())
                .map(ToString::to_string),
            body: parsed_body,
        });

        (state.status, Json(state.response)).into_response()
    }

    async fn mock_server(
        response: Value,
        status: StatusCode,
    ) -> (String, Arc<Mutex<Vec<RecordedRequest>>>) {
        let requests = Arc::new(Mutex::new(Vec::new()));
        let state = MockState {
            response,
            status,
            requests: requests.clone(),
        };
        let app = Router::new().fallback(any(mock_handler)).with_state(state);
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let address: SocketAddr = listener.local_addr().unwrap();
        tokio::spawn(async move {
            axum::serve(listener, app).await.unwrap();
        });

        (format!("http://{address}"), requests)
    }

    fn client(base_url: String) -> GitlabClient {
        client_with_filter(base_url, BTreeSet::new())
    }

    fn client_with_filter(base_url: String, projects_filter: BTreeSet<String>) -> GitlabClient {
        GitlabClient::new(GitlabConfig {
            base_url,
            auth: UpstreamAuth::HeaderToken {
                header_name: reqwest::header::HeaderName::from_static("private-token"),
                token: "gitlab-token".to_string(),
            },
            ssl_verify: true,
            proxy: ProxyConfig::default(),
            custom_headers: CustomHeaders::default(),
            mtls: None::<ClientTlsIdentityConfig>,
            projects_filter,
            timeout_seconds: 75,
        })
        .unwrap()
    }

    fn query_value(path: &str, key: &str) -> Option<String> {
        let url = reqwest::Url::parse(&format!("http://example{path}")).unwrap();
        url.query_pairs()
            .find(|(name, _)| name == key)
            .map(|(_, value)| value.into_owned())
    }

    #[tokio::test]
    async fn get_project_uses_api_root_private_token_and_numeric_project_id() {
        let (base_url, requests) = mock_server(json!({"id": 123}), StatusCode::OK).await;
        let value = client(base_url).get_project("123").await.unwrap();

        assert_eq!(value["id"], 123);
        let requests = requests.lock().await;
        assert_eq!(requests[0].method, Method::GET);
        assert_eq!(requests[0].path, "/api/v4/projects/123");
        assert_eq!(requests[0].private_token.as_deref(), Some("gitlab-token"));
        assert_eq!(requests[0].authorization, None);
    }

    #[tokio::test]
    async fn full_path_project_is_encoded_once_after_optional_decode() {
        let (base_url, requests) =
            mock_server(json!({"path": "group/sub project"}), StatusCode::OK).await;
        let client = client(base_url);

        client.get_project("group/sub project").await.unwrap();
        client.get_project("group%2Fsub%20project").await.unwrap();

        let requests = requests.lock().await;
        assert_eq!(requests[0].path, "/api/v4/projects/group%2Fsub%20project");
        assert_eq!(requests[1].path, "/api/v4/projects/group%2Fsub%20project");
    }

    #[tokio::test]
    async fn project_filter_allows_exact_numeric_or_full_path_and_rejects_others() {
        let (base_url, requests) = mock_server(json!({"ok": true}), StatusCode::OK).await;
        let client = client_with_filter(
            base_url,
            BTreeSet::from(["123".to_string(), "group/project".to_string()]),
        );

        client.get_project("123").await.unwrap();
        client.get_project("group%2Fproject").await.unwrap();
        let error = client.get_project("group/other").await.unwrap_err();

        assert!(error.to_string().contains("GITLAB_PROJECTS_FILTER"));
        assert!(!format!("{error:?}").contains("gitlab-token"));
        assert_eq!(requests.lock().await.len(), 2);
    }

    #[tokio::test]
    async fn list_merge_requests_applies_default_and_bounded_pagination_query() {
        let (base_url, requests) = mock_server(json!([]), StatusCode::OK).await;
        let client = client(base_url);

        client
            .list_merge_requests(ListMergeRequestsRequest {
                project: "group/project".to_string(),
                state: Some("opened".to_string()),
                labels: Some(vec!["bug".to_string(), " backend ".to_string()]),
                ..Default::default()
            })
            .await
            .unwrap();
        client
            .list_merge_requests(ListMergeRequestsRequest {
                project: "group/project".to_string(),
                page: Some(2),
                per_page: Some(MAX_GITLAB_PER_PAGE),
                ..Default::default()
            })
            .await
            .unwrap();

        let requests = requests.lock().await;
        assert!(
            requests[0]
                .path
                .starts_with("/api/v4/projects/group%2Fproject/merge_requests?")
        );
        assert_eq!(query_value(&requests[0].path, "page").as_deref(), Some("1"));
        assert_eq!(
            query_value(&requests[0].path, "per_page").as_deref(),
            Some("20")
        );
        assert_eq!(
            query_value(&requests[0].path, "state").as_deref(),
            Some("opened")
        );
        assert_eq!(
            query_value(&requests[0].path, "labels").as_deref(),
            Some("bug,backend")
        );
        assert_eq!(query_value(&requests[1].path, "page").as_deref(), Some("2"));
        assert_eq!(
            query_value(&requests[1].path, "per_page").as_deref(),
            Some("100")
        );
    }

    #[test]
    fn pagination_rejects_zero_or_above_cap() {
        assert!(pagination_query(Some(0), Some(20)).is_err());
        assert!(pagination_query(Some(1), Some(0)).is_err());
        assert!(pagination_query(Some(1), Some(101)).is_err());
    }

    #[tokio::test]
    async fn get_merge_request_passes_optional_flags() {
        let (base_url, requests) = mock_server(json!({"iid": 7}), StatusCode::OK).await;
        let value = client(base_url)
            .get_merge_request(GetMergeRequestRequest {
                project: "group/project".to_string(),
                merge_request_iid: 7,
                include_diverged_commits_count: Some(true),
                include_rebase_in_progress: Some(false),
            })
            .await
            .unwrap();

        assert_eq!(value["iid"], 7);
        let requests = requests.lock().await;
        assert!(
            requests[0]
                .path
                .starts_with("/api/v4/projects/group%2Fproject/merge_requests/7?")
        );
        assert_eq!(
            query_value(&requests[0].path, "include_diverged_commits_count").as_deref(),
            Some("true")
        );
        assert_eq!(
            query_value(&requests[0].path, "include_rebase_in_progress").as_deref(),
            Some("false")
        );
    }

    #[tokio::test]
    async fn merge_request_commits_and_pipelines_use_project_scoped_endpoints_and_pagination() {
        let (base_url, requests) = mock_server(json!([]), StatusCode::OK).await;
        let client = client(base_url);

        client
            .list_merge_request_commits(ListMergeRequestCommitsRequest {
                project: "group/project".to_string(),
                merge_request_iid: 3,
                page: Some(2),
                per_page: Some(50),
            })
            .await
            .unwrap();
        client
            .list_merge_request_pipelines(ListMergeRequestPipelinesRequest {
                project: "group/project".to_string(),
                merge_request_iid: 3,
                page: Some(4),
                per_page: Some(75),
            })
            .await
            .unwrap();

        let requests = requests.lock().await;
        assert!(
            requests[0]
                .path
                .starts_with("/api/v4/projects/group%2Fproject/merge_requests/3/commits?")
        );
        assert_eq!(query_value(&requests[0].path, "page").as_deref(), Some("2"));
        assert_eq!(
            query_value(&requests[0].path, "per_page").as_deref(),
            Some("50")
        );
        assert!(
            requests[1]
                .path
                .starts_with("/api/v4/projects/group%2Fproject/merge_requests/3/pipelines?")
        );
        assert_eq!(query_value(&requests[1].path, "page").as_deref(), Some("4"));
        assert_eq!(
            query_value(&requests[1].path, "per_page").as_deref(),
            Some("75")
        );
    }

    #[tokio::test]
    async fn merge_request_diffs_return_truncation_status() {
        let (base_url, requests) = mock_server(
            json!([
                {"old_path": "a.txt", "new_path": "a.txt", "diff": "abcdef"},
                {"old_path": "b.txt", "new_path": "b.txt", "diff": "ghijkl"},
            ]),
            StatusCode::OK,
        )
        .await;

        let value = client(base_url)
            .list_merge_request_diffs(ListMergeRequestDiffsRequest {
                project: "group/project".to_string(),
                merge_request_iid: 4,
                max_diff_bytes: Some(8),
                page: Some(3),
                per_page: Some(40),
            })
            .await
            .unwrap();

        assert_eq!(value["truncated"], true);
        assert_eq!(value["max_diff_bytes"], 8);
        assert_eq!(value["diffs"][0]["diff"], "abcdef");
        assert_eq!(value["diffs"][1]["diff"], "gh");
        let requests = requests.lock().await;
        assert!(
            requests[0]
                .path
                .starts_with("/api/v4/projects/group%2Fproject/merge_requests/4/diffs?")
        );
        assert_eq!(query_value(&requests[0].path, "page").as_deref(), Some("3"));
        assert_eq!(
            query_value(&requests[0].path, "per_page").as_deref(),
            Some("40")
        );
    }

    #[tokio::test]
    async fn create_and_update_merge_request_send_expected_json_bodies() {
        let (base_url, requests) = mock_server(json!({"iid": 10}), StatusCode::OK).await;
        let client = client(base_url);

        client
            .create_merge_request(CreateMergeRequestRequest {
                project: "group/project".to_string(),
                source_branch: "feature".to_string(),
                target_branch: "main".to_string(),
                title: "Add feature".to_string(),
                description: Some("Details".to_string()),
                remove_source_branch: Some(true),
                squash: Some(false),
                assignee_ids: Some(vec![11, 12]),
                reviewer_ids: Some(vec![21]),
                labels: Some(vec!["backend".to_string(), " ready ".to_string()]),
            })
            .await
            .unwrap();
        client
            .update_merge_request(UpdateMergeRequestRequest {
                project: "group/project".to_string(),
                merge_request_iid: 10,
                title: Some("Updated".to_string()),
                description: Some("".to_string()),
                state_event: Some("close".to_string()),
                labels: Some(vec![]),
                add_labels: Some(vec!["triaged".to_string()]),
                reviewer_ids: Some(vec![]),
                assignee_ids: Some(vec![]),
                target_branch: Some("release".to_string()),
                ..Default::default()
            })
            .await
            .unwrap();

        let requests = requests.lock().await;
        assert_eq!(requests[0].method, Method::POST);
        assert_eq!(
            requests[0].path,
            "/api/v4/projects/group%2Fproject/merge_requests"
        );
        assert_eq!(requests[0].body["source_branch"], "feature");
        assert_eq!(requests[0].body["target_branch"], "main");
        assert_eq!(requests[0].body["title"], "Add feature");
        assert_eq!(requests[0].body["labels"], "backend,ready");
        assert_eq!(requests[0].body["assignee_ids"], json!([11, 12]));
        assert_eq!(requests[0].body["reviewer_ids"], json!([21]));

        assert_eq!(requests[1].method, Method::PUT);
        assert_eq!(
            requests[1].path,
            "/api/v4/projects/group%2Fproject/merge_requests/10"
        );
        assert_eq!(requests[1].body["title"], "Updated");
        assert_eq!(requests[1].body["description"], "");
        assert_eq!(requests[1].body["state_event"], "close");
        assert_eq!(requests[1].body["labels"], "");
        assert_eq!(requests[1].body["add_labels"], "triaged");
        assert_eq!(requests[1].body["reviewer_ids"], json!([]));
        assert_eq!(requests[1].body["assignee_ids"], json!([]));
        assert_eq!(requests[1].body["target_branch"], "release");
    }

    #[tokio::test]
    async fn notes_and_discussions_send_expected_json_bodies() {
        let (base_url, requests) = mock_server(json!({"ok": true}), StatusCode::OK).await;
        let client = client(base_url);

        client
            .add_merge_request_note("group/project", 5, "Looks good".to_string())
            .await
            .unwrap();
        client
            .reply_merge_request_discussion(
                "group/project",
                5,
                "discussion/with space",
                "Reply".to_string(),
            )
            .await
            .unwrap();
        client
            .resolve_merge_request_discussion("group/project", 5, "discussion/with space", true)
            .await
            .unwrap();

        let requests = requests.lock().await;
        assert_eq!(
            requests[0].path,
            "/api/v4/projects/group%2Fproject/merge_requests/5/notes"
        );
        assert_eq!(requests[0].body["body"], "Looks good");
        assert_eq!(
            requests[1].path,
            "/api/v4/projects/group%2Fproject/merge_requests/5/discussions/discussion%2Fwith%20space/notes"
        );
        assert_eq!(requests[1].body["body"], "Reply");
        assert_eq!(
            requests[2].path,
            "/api/v4/projects/group%2Fproject/merge_requests/5/discussions/discussion%2Fwith%20space"
        );
        assert_eq!(requests[2].body["resolved"], true);
    }

    #[tokio::test]
    async fn note_body_must_not_be_empty_before_http_request() {
        let (base_url, requests) = mock_server(json!({"ok": true}), StatusCode::OK).await;
        let error = client(base_url)
            .add_merge_request_note("group/project", 5, "   ".to_string())
            .await
            .unwrap_err();

        assert!(error.to_string().contains("body must not be empty"));
        assert!(requests.lock().await.is_empty());
    }

    #[tokio::test]
    async fn approvals_use_expected_endpoints() {
        let (base_url, requests) = mock_server(json!({"approved": true}), StatusCode::OK).await;
        let client = client(base_url);

        client
            .get_merge_request_approval_state("group/project", 5)
            .await
            .unwrap();
        client
            .approve_merge_request("group/project", 5)
            .await
            .unwrap();
        client
            .unapprove_merge_request("group/project", 5)
            .await
            .unwrap();

        let requests = requests.lock().await;
        assert_eq!(requests[0].method, Method::GET);
        assert_eq!(
            requests[0].path,
            "/api/v4/projects/group%2Fproject/merge_requests/5/approval_state"
        );
        assert_eq!(requests[1].method, Method::POST);
        assert_eq!(
            requests[1].path,
            "/api/v4/projects/group%2Fproject/merge_requests/5/approve"
        );
        assert_eq!(requests[1].body, json!({}));
        assert_eq!(requests[2].method, Method::POST);
        assert_eq!(
            requests[2].path,
            "/api/v4/projects/group%2Fproject/merge_requests/5/unapprove"
        );
        assert_eq!(requests[2].body, json!({}));
    }

    #[tokio::test]
    async fn accept_merge_request_requires_sha_and_sends_optional_body_fields() {
        let (base_url, requests) = mock_server(json!({"merged": true}), StatusCode::OK).await;
        let client = client(base_url);

        client
            .accept_merge_request(AcceptMergeRequestRequest {
                project: "group/project".to_string(),
                merge_request_iid: 5,
                sha: "abc123".to_string(),
                auto_merge: Some(true),
                squash: Some(false),
                should_remove_source_branch: Some(true),
                merge_commit_message: Some("Merge message".to_string()),
                squash_commit_message: Some("Squash message".to_string()),
            })
            .await
            .unwrap();
        let error = client
            .accept_merge_request(AcceptMergeRequestRequest {
                project: "group/project".to_string(),
                merge_request_iid: 5,
                sha: " ".to_string(),
                ..Default::default()
            })
            .await
            .unwrap_err();

        assert!(error.to_string().contains("sha must not be empty"));
        let requests = requests.lock().await;
        assert_eq!(requests.len(), 1);
        assert_eq!(requests[0].method, Method::PUT);
        assert_eq!(
            requests[0].path,
            "/api/v4/projects/group%2Fproject/merge_requests/5/merge"
        );
        assert_eq!(requests[0].body["sha"], "abc123");
        assert_eq!(requests[0].body["auto_merge"], true);
        assert_eq!(requests[0].body["squash"], false);
        assert_eq!(requests[0].body["should_remove_source_branch"], true);
        assert_eq!(requests[0].body["merge_commit_message"], "Merge message");
        assert_eq!(requests[0].body["squash_commit_message"], "Squash message");
    }

    #[tokio::test]
    async fn accept_merge_request_maps_upstream_conflict() {
        let (base_url, _requests) = mock_server(
            json!({"message": "SHA does not match HEAD"}),
            StatusCode::CONFLICT,
        )
        .await;
        let error = client(base_url)
            .accept_merge_request(AcceptMergeRequestRequest {
                project: "group/project".to_string(),
                merge_request_iid: 5,
                sha: "abc123".to_string(),
                ..Default::default()
            })
            .await
            .unwrap_err();

        assert!(matches!(
            error,
            UpstreamError::HttpStatus { status: 409, .. }
        ));
        assert!(!format!("{error:?}").contains("gitlab-token"));
    }
}

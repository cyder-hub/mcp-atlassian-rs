use std::collections::BTreeSet;

use serde_json::{Value, json};

use crate::{
    atlassian::{error::AtlassianError, http::AtlassianHttpClient},
    jira::{
        config::{JiraConfig, JiraDeployment},
        formatting::{
            comment_body_for_deployment, ensure_issue_allowed, inject_project_filter,
            parse_optional_object, safe_path_segment,
        },
        models::{
            JiraComment, JiraField, JiraFieldOption, JiraFieldOptionsResponse, JiraIssue,
            JiraSearchResult, JiraTransitionsResponse, simplify_comment, simplify_fields,
            simplify_options,
        },
    },
};

pub const DEFAULT_LIMIT: u64 = 50;

#[derive(Clone, Debug)]
pub struct JiraClient {
    config: JiraConfig,
    http: AtlassianHttpClient,
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

impl JiraClient {
    pub fn new(config: JiraConfig) -> Result<Self, AtlassianError> {
        let http = AtlassianHttpClient::new(
            &config.base_url,
            config.auth.clone(),
            config.timeout_seconds,
            config.ssl_verify,
        )?;
        Ok(Self { config, http })
    }

    pub async fn get_issue(&self, request: GetIssueRequest) -> Result<Value, AtlassianError> {
        ensure_issue_allowed(&request.issue_key, &self.config)?;
        let issue_key = safe_path_segment(&request.issue_key, "issue_key")?;
        let mut query = optional_query_params([
            ("fields", request.fields.map(|fields| fields.join(","))),
            ("expand", request.expand.map(|expand| expand.join(","))),
            (
                "properties",
                request.properties.map(|properties| properties.join(",")),
            ),
            (
                "updateHistory",
                request.update_history.map(|value| value.to_string()),
            ),
        ]);

        if let Some(comment_limit) = request.comment_limit {
            query.push(("commentLimit".to_string(), comment_limit.to_string()));
        }

        let issue: JiraIssue = self
            .http
            .send_json(
                self.http
                    .get(&format!("/rest/api/2/issue/{issue_key}"))?
                    .query(&query),
            )
            .await?;
        Ok(issue.to_simplified_value())
    }

    pub async fn search(&self, request: SearchRequest) -> Result<Value, AtlassianError> {
        let limit = request.limit.unwrap_or(DEFAULT_LIMIT);
        let projects = self.effective_projects(request.projects_filter.as_deref())?;
        let jql = inject_project_filter(&request.jql, &projects);
        let result: JiraSearchResult = match self.config.deployment {
            JiraDeployment::Cloud => {
                let mut body = json!({
                    "jql": jql,
                    "maxResults": limit,
                });
                insert_optional(&mut body, "fields", request.fields.map(Value::from));
                insert_optional(
                    &mut body,
                    "expand",
                    request.expand.map(|expand| Value::String(expand.join(","))),
                );
                insert_optional(
                    &mut body,
                    "nextPageToken",
                    request.page_token.map(Value::String),
                );
                self.http
                    .send_json(self.http.post_json("/rest/api/3/search/jql", &body)?)
                    .await?
            }
            JiraDeployment::ServerDataCenter => {
                let body = json!({
                    "jql": jql,
                    "startAt": request.start_at.unwrap_or(0),
                    "maxResults": limit,
                    "fields": request.fields.unwrap_or_default(),
                    "expand": request.expand.unwrap_or_default().join(","),
                });
                self.http
                    .send_json(self.http.post_json("/rest/api/2/search", &body)?)
                    .await?
            }
        };

        Ok(result.to_simplified_value())
    }

    pub async fn get_project_issues(
        &self,
        project_key: String,
        limit: Option<u64>,
        start_at: Option<u64>,
    ) -> Result<Value, AtlassianError> {
        let project_key = safe_path_segment(&project_key, "project_key")?;
        self.search(SearchRequest {
            jql: format!("project = \"{}\"", project_key.replace('"', "\\\"")),
            limit,
            start_at,
            ..Default::default()
        })
        .await
    }

    pub async fn search_fields(
        &self,
        keyword: Option<String>,
        limit: Option<u64>,
    ) -> Result<Value, AtlassianError> {
        let fields: Vec<JiraField> = self
            .http
            .send_json(self.http.get("/rest/api/2/field")?)
            .await?;
        let keyword = keyword.map(|keyword| keyword.to_ascii_lowercase());
        let limit = limit.unwrap_or(DEFAULT_LIMIT) as usize;
        let filtered = fields
            .into_iter()
            .filter(|field| {
                keyword.as_ref().is_none_or(|keyword| {
                    field.id.to_ascii_lowercase().contains(keyword)
                        || field.name.to_ascii_lowercase().contains(keyword)
                        || field
                            .key
                            .as_ref()
                            .is_some_and(|key| key.to_ascii_lowercase().contains(keyword))
                })
            })
            .take(limit)
            .collect::<Vec<_>>();

        Ok(simplify_fields(&filtered))
    }

    pub async fn get_field_options(
        &self,
        request: FieldOptionsRequest,
    ) -> Result<Value, AtlassianError> {
        let field_id = safe_path_segment(&request.field_id, "field_id")?;
        let mut options = match self.config.deployment {
            JiraDeployment::Cloud => {
                let context_id = request
                    .context_id
                    .as_deref()
                    .ok_or_else(|| {
                        AtlassianError::invalid_input(
                            "context_id is required for Jira Cloud field options",
                        )
                    })
                    .and_then(|context_id| safe_path_segment(context_id, "context_id"))?;
                let query = vec![(
                    "maxResults".to_string(),
                    request.return_limit.unwrap_or(DEFAULT_LIMIT).to_string(),
                )];
                let response: JiraFieldOptionsResponse = self
                    .http
                    .send_json(
                        self.http
                            .get(&format!(
                                "/rest/api/3/field/{field_id}/context/{context_id}/option"
                            ))?
                            .query(&query),
                    )
                    .await?;
                response.values
            }
            JiraDeployment::ServerDataCenter => {
                let project_key = request
                    .project_key
                    .as_deref()
                    .ok_or_else(|| {
                        AtlassianError::invalid_input(
                            "project_key is required for Jira Server/Data Center field options",
                        )
                    })
                    .and_then(|project_key| safe_path_segment(project_key, "project_key"))?;
                let issue_type = request.issue_type.as_deref().ok_or_else(|| {
                    AtlassianError::invalid_input(
                        "issue_type is required for Jira Server/Data Center field options",
                    )
                })?;
                let query = vec![
                    ("projectKeys".to_string(), project_key),
                    ("issuetypeNames".to_string(), issue_type.to_string()),
                    (
                        "expand".to_string(),
                        "projects.issuetypes.fields".to_string(),
                    ),
                ];
                let value: Value = self
                    .http
                    .send_json(self.http.get("/rest/api/2/issue/createmeta")?.query(&query))
                    .await?;
                extract_createmeta_options(&value, &field_id)?
            }
        };

        if let Some(contains) = request.contains {
            let contains = contains.to_ascii_lowercase();
            options.retain(|option| {
                option
                    .label()
                    .is_some_and(|label| label.to_ascii_lowercase().contains(&contains))
            });
        }
        options.truncate(request.return_limit.unwrap_or(DEFAULT_LIMIT) as usize);

        Ok(simplify_options(&options, request.values_only))
    }

    pub async fn add_comment(
        &self,
        issue_key: String,
        body: String,
        visibility: Option<Value>,
    ) -> Result<Value, AtlassianError> {
        ensure_issue_allowed(&issue_key, &self.config)?;
        let issue_key = safe_path_segment(&issue_key, "issue_key")?;
        let visibility = parse_optional_object(visibility, "visibility")?;
        let mut payload = json!({
            "body": comment_body_for_deployment(self.config.deployment, &body),
        });
        insert_optional(&mut payload, "visibility", visibility);
        let path = self.issue_comment_path(&issue_key);
        let comment: JiraComment = self
            .http
            .send_json(self.http.post_json(&path, &payload)?)
            .await?;

        Ok(simplify_comment(&comment))
    }

    pub async fn edit_comment(
        &self,
        issue_key: String,
        comment_id: String,
        body: String,
        visibility: Option<Value>,
    ) -> Result<Value, AtlassianError> {
        ensure_issue_allowed(&issue_key, &self.config)?;
        let issue_key = safe_path_segment(&issue_key, "issue_key")?;
        let comment_id = safe_path_segment(&comment_id, "comment_id")?;
        let visibility = parse_optional_object(visibility, "visibility")?;
        let mut payload = json!({
            "body": comment_body_for_deployment(self.config.deployment, &body),
        });
        insert_optional(&mut payload, "visibility", visibility);
        let path = format!("{}/{}", self.issue_comment_path(&issue_key), comment_id);
        let comment: JiraComment = self
            .http
            .send_json(self.http.put_json(&path, &payload)?)
            .await?;

        Ok(simplify_comment(&comment))
    }

    pub async fn get_transitions(&self, issue_key: String) -> Result<Value, AtlassianError> {
        ensure_issue_allowed(&issue_key, &self.config)?;
        let issue_key = safe_path_segment(&issue_key, "issue_key")?;
        let response: JiraTransitionsResponse = self
            .http
            .send_json(
                self.http
                    .get(&format!("/rest/api/2/issue/{issue_key}/transitions"))?,
            )
            .await?;

        Ok(response.to_simplified_value())
    }

    pub async fn transition_issue(
        &self,
        issue_key: String,
        transition_id: String,
        fields: Option<Value>,
        comment: Option<String>,
    ) -> Result<Value, AtlassianError> {
        ensure_issue_allowed(&issue_key, &self.config)?;
        let issue_key = safe_path_segment(&issue_key, "issue_key")?;
        let transition_id = safe_path_segment(&transition_id, "transition_id")?;
        let fields = parse_optional_object(fields, "fields")?;
        let mut payload = json!({
            "transition": {
                "id": transition_id,
            }
        });
        insert_optional(&mut payload, "fields", fields);
        if let Some(comment) = comment {
            insert_optional(
                &mut payload,
                "update",
                Some(json!({
                    "comment": [
                        {
                            "add": {
                                "body": comment_body_for_deployment(self.config.deployment, &comment)
                            }
                        }
                    ]
                })),
            );
        }

        let value: Value = self
            .http
            .send_json_value_or_null(self.http.post_json(
                &format!("/rest/api/2/issue/{issue_key}/transitions"),
                &payload,
            )?)
            .await?;
        Ok(json!({
            "issue_key": issue_key,
            "transition_id": transition_id,
            "response": value,
        }))
    }

    fn effective_projects(
        &self,
        request_projects: Option<&[String]>,
    ) -> Result<Vec<String>, AtlassianError> {
        let config_projects = self
            .config
            .projects_filter
            .iter()
            .cloned()
            .collect::<Vec<_>>();
        let request_projects = request_projects.unwrap_or_default();
        if config_projects.is_empty() {
            return Ok(request_projects.to_vec());
        }
        if request_projects.is_empty() {
            return Ok(config_projects);
        }

        let request_set = request_projects.iter().collect::<BTreeSet<_>>();
        let intersection = config_projects
            .into_iter()
            .filter(|project| request_set.contains(project))
            .collect::<Vec<_>>();
        if intersection.is_empty() {
            Err(AtlassianError::invalid_input(
                "projects_filter does not overlap with configured Jira project filter",
            ))
        } else {
            Ok(intersection)
        }
    }

    fn issue_comment_path(&self, issue_key: &str) -> String {
        match self.config.deployment {
            JiraDeployment::Cloud => format!("/rest/api/3/issue/{issue_key}/comment"),
            JiraDeployment::ServerDataCenter => format!("/rest/api/2/issue/{issue_key}/comment"),
        }
    }
}

fn optional_query_params<const N: usize>(
    pairs: [(&str, Option<String>); N],
) -> Vec<(String, String)> {
    pairs
        .into_iter()
        .filter_map(|(key, value)| value.map(|value| (key.to_string(), value)))
        .collect()
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
) -> Result<Vec<JiraFieldOption>, AtlassianError> {
    let projects = value
        .get("projects")
        .and_then(Value::as_array)
        .ok_or_else(|| AtlassianError::unexpected_shape("createmeta response missing projects"))?;

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
                    AtlassianError::unexpected_shape("createmeta field missing allowedValues")
                })?;
            return options
                .iter()
                .cloned()
                .map(serde_json::from_value)
                .collect::<Result<Vec<JiraFieldOption>, _>>()
                .map_err(|error| AtlassianError::unexpected_shape(error.to_string()));
        }
    }

    Err(AtlassianError::unexpected_shape(format!(
        "createmeta response missing field `{field_id}`"
    )))
}

#[cfg(test)]
mod tests {
    use std::{net::SocketAddr, sync::Arc};

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

    use crate::{
        atlassian::auth::AtlassianAuth,
        jira::config::{DEFAULT_JIRA_TIMEOUT_SECONDS, JiraDeployment},
    };

    use super::*;

    #[derive(Clone, Debug)]
    struct RecordedRequest {
        method: Method,
        path: String,
        authorization: Option<String>,
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
            body: parsed_body,
        });

        (state.status, Json(state.response)).into_response()
    }

    async fn invalid_json_handler(
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
            body: parsed_body,
        });

        (StatusCode::OK, "not-json").into_response()
    }

    async fn mock_server(response: Value) -> (String, Arc<Mutex<Vec<RecordedRequest>>>) {
        mock_server_with_status(response, StatusCode::OK).await
    }

    async fn mock_server_with_status(
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

    async fn invalid_json_mock_server() -> (String, Arc<Mutex<Vec<RecordedRequest>>>) {
        let requests = Arc::new(Mutex::new(Vec::new()));
        let state = MockState {
            response: Value::Null,
            status: StatusCode::OK,
            requests: requests.clone(),
        };
        let app = Router::new()
            .fallback(any(invalid_json_handler))
            .with_state(state);
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let address: SocketAddr = listener.local_addr().unwrap();
        tokio::spawn(async move {
            axum::serve(listener, app).await.unwrap();
        });

        (format!("http://{address}"), requests)
    }

    fn config(base_url: String, deployment: JiraDeployment) -> JiraConfig {
        JiraConfig {
            base_url,
            deployment,
            auth: match deployment {
                JiraDeployment::Cloud => AtlassianAuth::Basic {
                    username: "user@example.com".to_string(),
                    api_token: "test-api-token".to_string(),
                },
                JiraDeployment::ServerDataCenter => AtlassianAuth::Pat {
                    personal_token: "test-pat-value".to_string(),
                },
            },
            ssl_verify: true,
            projects_filter: BTreeSet::new(),
            timeout_seconds: DEFAULT_JIRA_TIMEOUT_SECONDS,
        }
    }

    #[tokio::test]
    async fn get_issue_uses_v2_endpoint_and_auth_header() {
        let (base_url, requests) =
            mock_server(json!({"id": "10001", "key": "ABC-1", "fields": {"summary": "Demo"}}))
                .await;
        let client = JiraClient::new(config(base_url, JiraDeployment::ServerDataCenter)).unwrap();
        let value = client
            .get_issue(GetIssueRequest {
                issue_key: "ABC-1".to_string(),
                fields: Some(vec!["summary".to_string()]),
                ..Default::default()
            })
            .await
            .unwrap();
        let requests = requests.lock().await;

        assert_eq!(value["key"], "ABC-1");
        assert_eq!(requests[0].method, Method::GET);
        assert!(requests[0].path.starts_with("/rest/api/2/issue/ABC-1"));
        let expected_header = format!("Bearer {}", "test-pat-value");
        assert_eq!(
            requests[0].authorization.as_deref(),
            Some(expected_header.as_str())
        );
    }

    #[tokio::test]
    async fn cloud_search_uses_v3_search_jql_and_basic_auth() {
        let (base_url, requests) =
            mock_server(json!({"issues": [], "nextPageToken": "next", "isLast": false})).await;
        let client = JiraClient::new(config(base_url, JiraDeployment::Cloud)).unwrap();
        let value = client
            .search(SearchRequest {
                jql: "status = Done".to_string(),
                limit: Some(10),
                projects_filter: Some(vec!["ABC".to_string()]),
                page_token: Some("token".to_string()),
                ..Default::default()
            })
            .await
            .unwrap();
        let requests = requests.lock().await;

        assert_eq!(value["next_page_token"], "next");
        assert_eq!(requests[0].method, Method::POST);
        assert_eq!(requests[0].path, "/rest/api/3/search/jql");
        assert!(requests[0].authorization.as_deref().is_some_and(|header| {
            header.starts_with("Basic ") && !header.contains("test-api-token")
        }));
        assert_eq!(requests[0].body["maxResults"], 10);
        assert_eq!(requests[0].body["nextPageToken"], "token");
        assert!(
            requests[0].body["jql"]
                .as_str()
                .unwrap()
                .contains("project = \"ABC\"")
        );
    }

    #[tokio::test]
    async fn server_search_uses_v2_search_and_start_at() {
        let (base_url, requests) = mock_server(json!({"issues": [], "total": 0})).await;
        let client = JiraClient::new(config(base_url, JiraDeployment::ServerDataCenter)).unwrap();
        client
            .search(SearchRequest {
                jql: "project = ABC".to_string(),
                start_at: Some(20),
                ..Default::default()
            })
            .await
            .unwrap();
        let requests = requests.lock().await;

        assert_eq!(requests[0].path, "/rest/api/2/search");
        assert_eq!(requests[0].body["startAt"], 20);
    }

    #[tokio::test]
    async fn get_project_issues_builds_project_jql() {
        let (base_url, requests) = mock_server(json!({"issues": [], "total": 0})).await;
        let client = JiraClient::new(config(base_url, JiraDeployment::ServerDataCenter)).unwrap();
        client
            .get_project_issues("ABC".to_string(), Some(5), Some(10))
            .await
            .unwrap();
        let requests = requests.lock().await;

        assert_eq!(requests[0].path, "/rest/api/2/search");
        assert_eq!(requests[0].body["jql"], "project = \"ABC\"");
        assert_eq!(requests[0].body["maxResults"], 5);
        assert_eq!(requests[0].body["startAt"], 10);
    }

    #[tokio::test]
    async fn search_fields_filters_case_insensitively_and_handles_missing_schema() {
        let (base_url, requests) = mock_server(json!([
            {"id": "summary", "name": "Summary"},
            {"id": "customfield_10001", "name": "Customer Impact", "schema": {"type": "string"}}
        ]))
        .await;
        let client = JiraClient::new(config(base_url, JiraDeployment::ServerDataCenter)).unwrap();
        let value = client
            .search_fields(Some("CUSTOMER".to_string()), Some(1))
            .await
            .unwrap();
        let requests = requests.lock().await;

        assert_eq!(requests[0].path, "/rest/api/2/field");
        assert_eq!(value.as_array().unwrap().len(), 1);
        assert_eq!(value[0]["id"], "customfield_10001");
    }

    #[tokio::test]
    async fn field_options_support_cloud_context_options() {
        let (base_url, requests) =
            mock_server(json!({"values": [{"id": "1", "value": "High"}]})).await;
        let client = JiraClient::new(config(base_url, JiraDeployment::Cloud)).unwrap();
        let value = client
            .get_field_options(FieldOptionsRequest {
                field_id: "customfield_10001".to_string(),
                context_id: Some("20001".to_string()),
                values_only: true,
                ..Default::default()
            })
            .await
            .unwrap();
        let requests = requests.lock().await;

        assert_eq!(value, json!(["High"]));
        assert!(
            requests[0]
                .path
                .starts_with("/rest/api/3/field/customfield_10001/context/20001/option")
        );
    }

    #[tokio::test]
    async fn field_options_support_server_createmeta_options() {
        let (base_url, requests) = mock_server(json!({
            "projects": [{
                "issuetypes": [{
                    "fields": {
                        "customfield_10001": {
                            "allowedValues": [{"id": "1", "value": "High"}]
                        }
                    }
                }]
            }]
        }))
        .await;
        let client = JiraClient::new(config(base_url, JiraDeployment::ServerDataCenter)).unwrap();
        let value = client
            .get_field_options(FieldOptionsRequest {
                field_id: "customfield_10001".to_string(),
                project_key: Some("ABC".to_string()),
                issue_type: Some("Bug".to_string()),
                values_only: false,
                ..Default::default()
            })
            .await
            .unwrap();
        let requests = requests.lock().await;

        assert!(requests[0].path.starts_with("/rest/api/2/issue/createmeta"));
        assert_eq!(value["values"][0]["value"], "High");
    }

    #[tokio::test]
    async fn add_comment_uses_server_string_body() {
        let (base_url, requests) = mock_server(json!({
            "id": "10",
            "body": "Hello",
            "author": {"displayName": "Ada"}
        }))
        .await;
        let client = JiraClient::new(config(base_url, JiraDeployment::ServerDataCenter)).unwrap();
        let value = client
            .add_comment("ABC-1".to_string(), "Hello".to_string(), None)
            .await
            .unwrap();
        let requests = requests.lock().await;

        assert_eq!(value["body"], "Hello");
        assert_eq!(requests[0].path, "/rest/api/2/issue/ABC-1/comment");
        assert_eq!(requests[0].body["body"], "Hello");
    }

    #[tokio::test]
    async fn edit_comment_uses_put_endpoint_and_visibility() {
        let (base_url, requests) = mock_server(json!({
            "id": "10",
            "body": "Updated",
            "visibility": {"type": "role", "value": "Developers"}
        }))
        .await;
        let client = JiraClient::new(config(base_url, JiraDeployment::ServerDataCenter)).unwrap();
        let value = client
            .edit_comment(
                "ABC-1".to_string(),
                "10".to_string(),
                "Updated".to_string(),
                Some(json!({"type": "role", "value": "Developers"})),
            )
            .await
            .unwrap();
        let requests = requests.lock().await;

        assert_eq!(value["visibility"]["value"], "Developers");
        assert_eq!(requests[0].method, Method::PUT);
        assert_eq!(requests[0].path, "/rest/api/2/issue/ABC-1/comment/10");
        assert_eq!(requests[0].body["visibility"]["value"], "Developers");
    }

    #[tokio::test]
    async fn get_issue_missing_fields_payload_is_simplified() {
        let (base_url, _requests) = mock_server(json!({"key": "ABC-1"})).await;
        let client = JiraClient::new(config(base_url, JiraDeployment::ServerDataCenter)).unwrap();
        let value = client
            .get_issue(GetIssueRequest {
                issue_key: "ABC-1".to_string(),
                ..Default::default()
            })
            .await
            .unwrap();

        assert_eq!(value["key"], "ABC-1");
        assert!(value["fields"].is_null());
        assert!(value["summary"].is_null());
    }

    #[tokio::test]
    async fn comment_missing_optional_payload_fields_is_simplified() {
        let (base_url, _requests) = mock_server(json!({"id": "10"})).await;
        let client = JiraClient::new(config(base_url, JiraDeployment::ServerDataCenter)).unwrap();
        let value = client
            .add_comment("ABC-1".to_string(), "Hello".to_string(), None)
            .await
            .unwrap();

        assert_eq!(value["id"], "10");
        assert_eq!(value["body"], "");
        assert!(value["author"]["display_name"].is_null());
    }

    #[tokio::test]
    async fn transitions_missing_fields_payload_is_simplified() {
        let (base_url, _requests) =
            mock_server(json!({"transitions": [{"id": "31", "name": "Done"}]})).await;
        let client = JiraClient::new(config(base_url, JiraDeployment::ServerDataCenter)).unwrap();
        let value = client.get_transitions("ABC-1".to_string()).await.unwrap();

        assert_eq!(value["transitions"][0]["id"], "31");
        assert!(value["transitions"][0]["fields"].is_null());
        assert!(value["transitions"][0]["to"]["name"].is_null());
    }

    #[tokio::test]
    async fn transition_issue_posts_transition_payload() {
        let (base_url, requests) = mock_server(json!({})).await;
        let client = JiraClient::new(config(base_url, JiraDeployment::ServerDataCenter)).unwrap();
        let value = client
            .transition_issue(
                "ABC-1".to_string(),
                "31".to_string(),
                Some(json!({"resolution": {"name": "Done"}})),
                Some("Resolved".to_string()),
            )
            .await
            .unwrap();
        let requests = requests.lock().await;

        assert_eq!(value["transition_id"], "31");
        assert_eq!(requests[0].path, "/rest/api/2/issue/ABC-1/transitions");
        assert_eq!(requests[0].body["transition"]["id"], "31");
        assert_eq!(requests[0].body["fields"]["resolution"]["name"], "Done");
    }

    #[tokio::test]
    async fn transition_issue_accepts_no_content_response() {
        let (base_url, _requests) =
            mock_server_with_status(json!({}), StatusCode::NO_CONTENT).await;
        let client = JiraClient::new(config(base_url, JiraDeployment::ServerDataCenter)).unwrap();
        let value = client
            .transition_issue("ABC-1".to_string(), "31".to_string(), None, None)
            .await
            .unwrap();

        assert_eq!(value["transition_id"], "31");
        assert!(value["response"].is_null());
    }

    #[tokio::test]
    async fn issue_not_found_error_is_safe() {
        let (base_url, _requests) =
            mock_server_with_status(json!({"errorMessages": ["missing"]}), StatusCode::NOT_FOUND)
                .await;
        let client = JiraClient::new(config(base_url, JiraDeployment::ServerDataCenter)).unwrap();
        let error = client
            .get_issue(GetIssueRequest {
                issue_key: "ABC-1".to_string(),
                ..Default::default()
            })
            .await
            .unwrap_err()
            .to_string();

        assert!(error.contains("HTTP 404"));
        assert!(!error.contains("Bearer"));
        assert!(!error.contains("test-pat-value"));
    }

    #[tokio::test]
    async fn unauthorized_error_is_safe() {
        assert_status_error_is_safe(StatusCode::UNAUTHORIZED, "HTTP 401").await;
    }

    #[tokio::test]
    async fn forbidden_error_is_safe() {
        assert_status_error_is_safe(StatusCode::FORBIDDEN, "HTTP 403").await;
    }

    #[tokio::test]
    async fn rate_limit_error_is_safe() {
        assert_status_error_is_safe(StatusCode::TOO_MANY_REQUESTS, "HTTP 429").await;
    }

    async fn assert_status_error_is_safe(status: StatusCode, expected: &str) {
        let (base_url, _requests) =
            mock_server_with_status(json!({"errorMessages": ["safe failure"]}), status).await;
        let client = JiraClient::new(config(base_url, JiraDeployment::ServerDataCenter)).unwrap();
        let error = client
            .get_transitions("ABC-1".to_string())
            .await
            .unwrap_err()
            .to_string();

        assert!(error.contains(expected));
        assert!(error.contains("safe failure"));
        assert!(!error.contains("Bearer"));
        assert!(!error.contains("test-pat-value"));
    }

    #[tokio::test]
    async fn invalid_json_response_is_mapped_without_request_details() {
        let (base_url, requests) = invalid_json_mock_server().await;
        let client = JiraClient::new(config(base_url, JiraDeployment::ServerDataCenter)).unwrap();
        let error = client
            .get_issue(GetIssueRequest {
                issue_key: "ABC-1".to_string(),
                ..Default::default()
            })
            .await
            .unwrap_err()
            .to_string();
        let requests = requests.lock().await;

        assert!(error.contains("JSON decode error"));
        assert!(!error.contains("Bearer"));
        assert!(!error.contains("test-pat-value"));
        assert_eq!(requests.len(), 1);
    }

    #[tokio::test]
    async fn project_filter_rejects_issue_without_http_request() {
        let (base_url, requests) =
            mock_server(json!({"id": "10001", "key": "XYZ-1", "fields": {}})).await;
        let mut config = config(base_url, JiraDeployment::ServerDataCenter);
        config.projects_filter = BTreeSet::from(["ABC".to_string()]);
        let client = JiraClient::new(config).unwrap();
        let error = client
            .get_issue(GetIssueRequest {
                issue_key: "XYZ-1".to_string(),
                ..Default::default()
            })
            .await
            .unwrap_err()
            .to_string();
        let requests = requests.lock().await;

        assert!(error.contains("outside the configured Jira project filter"));
        assert!(requests.is_empty());
    }
}

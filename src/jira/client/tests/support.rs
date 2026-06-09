use super::*;

#[derive(Clone, Debug)]
pub(super) struct RecordedRequest {
    pub(super) method: Method,
    pub(super) path: String,
    pub(super) authorization: Option<String>,
    pub(super) body: Value,
}

#[derive(Clone)]
pub(super) struct MockState {
    pub(super) response: Value,
    pub(super) status: StatusCode,
    pub(super) requests: Arc<Mutex<Vec<RecordedRequest>>>,
}

pub(super) async fn mock_handler(
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

pub(super) async fn invalid_json_handler(
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

pub(super) async fn cloud_search_fallback_handler(
    State(requests): State<Arc<Mutex<Vec<RecordedRequest>>>>,
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
    requests.lock().await.push(RecordedRequest {
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

    match uri.path() {
            "/rest/api/3/search/jql" => (
                StatusCode::BAD_REQUEST,
                Json(json!({
                    "errorMessages": [
                        "Unbounded JQL queries are not allowed here. Please add a search restriction to your query."
                    ]
                })),
            )
                .into_response(),
            "/rest/api/3/search" => Json(json!({
                "issues": [{
                    "id": "10001",
                    "key": "ABC-1",
                    "fields": {"summary": "Demo"}
                }],
                "total": 1,
                "startAt": 0,
                "maxResults": 50
            }))
            .into_response(),
            _ => StatusCode::NOT_FOUND.into_response(),
        }
}

pub(super) async fn removed_cloud_legacy_search_handler(
    State(requests): State<Arc<Mutex<Vec<RecordedRequest>>>>,
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
    requests.lock().await.push(RecordedRequest {
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

    match uri.path() {
            "/rest/api/3/search/jql" => (
                StatusCode::BAD_REQUEST,
                Json(json!({
                    "errorMessages": [
                        "Unbounded JQL queries are not allowed here. Please add a search restriction to your query."
                    ]
                })),
            )
                .into_response(),
            "/rest/api/3/search" => (
                StatusCode::GONE,
                Json(json!({
                    "errorMessages": [
                        "The requested API has been removed. Please migrate to the /rest/api/3/search/jql API."
                    ]
                })),
            )
                .into_response(),
            _ => StatusCode::NOT_FOUND.into_response(),
        }
}

pub(super) async fn cloud_field_options_context_handler(
    State(requests): State<Arc<Mutex<Vec<RecordedRequest>>>>,
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
    let request_body = parsed_body.clone();
    requests.lock().await.push(RecordedRequest {
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

    match uri.path() {
        "/rest/api/3/project/ABC" => Json(json!({
            "id": "10000",
            "key": "ABC",
            "issueTypes": [
                {"id": "1", "name": "Bug"},
                {"id": "3", "name": "Task"}
            ]
        }))
        .into_response(),
        "/rest/api/3/field/customfield_10001/context/mapping" => {
            let issue_type_id = request_body
                .pointer("/mappings/0/issueTypeId")
                .and_then(Value::as_str);
            let context_id = if issue_type_id == Some("3") {
                Value::Null
            } else {
                json!("20001")
            };

            Json(json!({
                "values": [{
                    "projectId": "10000",
                    "issueTypeId": issue_type_id.unwrap_or(""),
                    "contextId": context_id
                }],
                "isLast": true
            }))
            .into_response()
        }
        "/rest/api/3/field/customfield_10001/context/20001/option" => Json(json!({
            "values": [{"id": "1", "value": "High"}],
            "isLast": true
        }))
        .into_response(),
        _ => StatusCode::NOT_FOUND.into_response(),
    }
}

pub(super) async fn stage_three_handler(
    State(requests): State<Arc<Mutex<Vec<RecordedRequest>>>>,
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
    let path = uri
        .path_and_query()
        .map(ToString::to_string)
        .unwrap_or_else(|| uri.path().to_string());
    requests.lock().await.push(RecordedRequest {
        method: method.clone(),
        path: path.clone(),
        authorization: headers
            .get("authorization")
            .and_then(|value| value.to_str().ok())
            .map(ToString::to_string),
        body: parsed_body,
    });

    if method == Method::DELETE {
        return StatusCode::NO_CONTENT.into_response();
    }

    let path_only = uri.path();
    if path_only == "/secure/attachment/1/file.png" {
        return (
            StatusCode::OK,
            [(axum::http::header::CONTENT_TYPE, "image/png")],
            "image-bytes",
        )
            .into_response();
    }
    if path_only == "/secure/attachment/2/notes.txt" {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({
                "errorMessages": [
                    "failed /secure/attachment/2/notes.txt?token=secret&client=abc"
                ]
            })),
        )
            .into_response();
    }
    if method == Method::GET
        && path.starts_with("/rest/agile/1.0/board?")
        && path.contains("projectKeyOrId=NOAGILE")
    {
        return (
            StatusCode::NOT_FOUND,
            Json(json!({"errorMessages": ["Jira Software is not available"]})),
        )
            .into_response();
    }
    if method == Method::GET && path_only.starts_with("/jsm-down/rest/servicedeskapi") {
        return (
            StatusCode::NOT_FOUND,
            Json(json!({"errorMessages": ["Jira Service Management is not available"]})),
        )
            .into_response();
    }
    if method == Method::GET && path_only.starts_with("/dev-down/rest/dev-status") {
        return (
            StatusCode::NOT_FOUND,
            Json(json!({"errorMessages": ["Jira development status is not available"]})),
        )
            .into_response();
    }

    match path_only {
        "/rest/api/2/project" => Json(json!([
            {"id": "10000", "key": "ABC", "name": "Alpha"},
            {"id": "10001", "key": "XYZ", "name": "Other"}
        ]))
        .into_response(),
        path if path.ends_with("/versions") => {
            Json(json!([{"id": "1", "name": "v1"}])).into_response()
        }
        path if path.ends_with("/components") => {
            Json(json!([{"id": "2", "name": "API"}])).into_response()
        }
        "/rest/api/2/version" => Json(json!({"id": "1", "name": "v1"})).into_response(),
        "/rest/api/2/user" => {
            Json(json!({"accountId": "abc", "displayName": "Ada"})).into_response()
        }
        path if path.ends_with("/watchers") && method == Method::GET => {
            Json(json!({"watcherCount": 1, "watchers": [{"displayName": "Ada"}]})).into_response()
        }
        path if path.ends_with("/worklog") && method == Method::GET => {
            Json(json!({"worklogs": [{"id": "10", "timeSpent": "1h"}]})).into_response()
        }
        path if path.ends_with("/worklog") => {
            Json(json!({"id": "10", "timeSpent": "1h"})).into_response()
        }
        "/rest/api/2/issueLinkType" => {
            Json(json!({"issueLinkTypes": [{"id": "100", "name": "Blocks"}]})).into_response()
        }
        "/rest/api/2/issueLink" => Json(json!({"id": "200"})).into_response(),
        path if path.ends_with("/remotelink") => Json(json!({"id": "300"})).into_response(),
        "/rest/agile/1.0/board" => {
            Json(json!({"values": [{"id": 1, "name": "Board"}]})).into_response()
        }
        path if path.ends_with("/board/1/issue") => {
            Json(json!({"issues": [{"key": "ABC-1", "fields": {}}]})).into_response()
        }
        path if path.ends_with("/board/1/sprint") => {
            Json(json!({"values": [{"id": 2, "name": "Sprint"}]})).into_response()
        }
        path if path.ends_with("/sprint/2/issue") => {
            Json(json!({"issues": [{"key": "ABC-1", "fields": {}}]})).into_response()
        }
        "/rest/agile/1.0/sprint" => Json(json!({"id": 2, "name": "Sprint"})).into_response(),
        path if path.ends_with("/sprint/2") => {
            Json(json!({"id": 2, "name": "Sprint updated"})).into_response()
        }
        path if path.ends_with("/sprint/2/issue") => Json(Value::Null).into_response(),
        "/rest/servicedeskapi/servicedesk" => Json(
            json!({"values": [{"id": "4", "projectKey": "ABC", "serviceDeskName": "Support"}]}),
        )
        .into_response(),
        path if path.ends_with("/servicedesk/4/queue") => {
            Json(json!({"values": [{"id": "47", "name": "Open"}]})).into_response()
        }
        path if path.ends_with("/servicedesk/4/queue/47/issue") => {
            Json(json!({"values": [{"key": "ABC-1"}]})).into_response()
        }
        "/jira/forms/cloud/cloud-123/issue/ABC-1/form" if method == Method::GET => {
            Json(json!({"forms": [{
                "id": "form-1",
                "name": "Request form",
                "state": {"status": "o"},
                "submitted": false
            }]}))
            .into_response()
        }
        "/jira/forms/cloud/cloud-123/issue/ABC-1/form/form-1" if method == Method::GET => {
            Json(json!({
                "id": "form-1",
                "name": "Request form",
                "state": {"status": "o"},
                "design": {"content": []},
                "answers": {"q1": {"text": "Existing"}}
            }))
            .into_response()
        }
        "/jira/forms/cloud/cloud-123/issue/ABC-1/form/form-1" if method == Method::PUT => {
            Json(json!({"id": "form-1", "updated": true})).into_response()
        }
        path if path.starts_with("/jira/forms/cloud/forms-down/") => (
            StatusCode::NOT_FOUND,
            Json(json!({"errorMessages": ["Jira Forms is not available"]})),
        )
            .into_response(),
        "/rest/dev-status/1.0/issue/detail" => {
            Json(json!({"detail": [{"branches": [], "pullRequests": []}]})).into_response()
        }
        path if path.starts_with("/rest/api/2/issue/ABC-1") && method == Method::GET => {
            Json(json!({
                "id": "10001",
                "key": "ABC-1",
                "fields": {
                    "summary": "Demo",
                    "customfield_sla": {
                        "name": "Time to resolution SLA",
                        "ongoingCycle": {
                            "breached": false,
                            "elapsedTime": {"millis": 60000},
                            "remainingTime": {"millis": 120000},
                            "startTime": "2026-01-01T00:00:00.000+0000"
                        }
                    },
                    "attachment": [{
                        "id": "1",
                        "filename": "file.png",
                        "mimeType": "image/png",
                        "size": 11,
                        "content": "/secure/attachment/1/file.png?token=secret"
                    }, {
                        "id": "2",
                        "filename": "notes.txt",
                        "mimeType": "text/plain",
                        "size": 42,
                        "content": "/secure/attachment/2/notes.txt?token=secret&client=abc"
                    }]
                }
            }))
            .into_response()
        }
        path if path.starts_with("/rest/api/2/issue/ABC-1") => Json(json!({
            "id": "10001",
            "key": "ABC-1",
            "fields": {"summary": "Demo"}
        }))
        .into_response(),
        "/rest/api/2/issue" => Json(json!({
            "id": "10001",
            "key": "ABC-1",
            "fields": {"summary": "Demo"}
        }))
        .into_response(),
        "/rest/api/2/issue/bulk" => Json(json!({"issues": [{"key": "ABC-1"}]})).into_response(),
        "/rest/api/3/changelog/bulkfetch" => {
            Json(json!({"issueChangeLogs": [{"issueId": "10001", "changeHistories": []}]}))
                .into_response()
        }
        _ => Json(json!({"ok": true, "path": path})).into_response(),
    }
}

pub(super) async fn mock_server(response: Value) -> (String, Arc<Mutex<Vec<RecordedRequest>>>) {
    mock_server_with_status(response, StatusCode::OK).await
}

pub(super) async fn mock_server_with_status(
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

pub(super) async fn invalid_json_mock_server() -> (String, Arc<Mutex<Vec<RecordedRequest>>>) {
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

pub(super) async fn cloud_search_fallback_mock_server() -> (String, Arc<Mutex<Vec<RecordedRequest>>>)
{
    let requests = Arc::new(Mutex::new(Vec::new()));
    let app = Router::new()
        .fallback(any(cloud_search_fallback_handler))
        .with_state(requests.clone());
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address: SocketAddr = listener.local_addr().unwrap();
    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });

    (format!("http://{address}"), requests)
}

pub(super) async fn removed_cloud_legacy_search_mock_server()
-> (String, Arc<Mutex<Vec<RecordedRequest>>>) {
    let requests = Arc::new(Mutex::new(Vec::new()));
    let app = Router::new()
        .fallback(any(removed_cloud_legacy_search_handler))
        .with_state(requests.clone());
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address: SocketAddr = listener.local_addr().unwrap();
    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });

    (format!("http://{address}"), requests)
}

pub(super) async fn cloud_field_options_context_mock_server()
-> (String, Arc<Mutex<Vec<RecordedRequest>>>) {
    let requests = Arc::new(Mutex::new(Vec::new()));
    let app = Router::new()
        .fallback(any(cloud_field_options_context_handler))
        .with_state(requests.clone());
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address: SocketAddr = listener.local_addr().unwrap();
    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });

    (format!("http://{address}"), requests)
}

pub(super) async fn stage_three_mock_server() -> (String, Arc<Mutex<Vec<RecordedRequest>>>) {
    let requests = Arc::new(Mutex::new(Vec::new()));
    let app = Router::new()
        .fallback(any(stage_three_handler))
        .with_state(requests.clone());
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address: SocketAddr = listener.local_addr().unwrap();
    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });

    (format!("http://{address}"), requests)
}

pub(super) fn config(base_url: String, deployment: JiraDeployment) -> JiraConfig {
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
        oauth_cloud_id: None,
        ssl_verify: true,
        proxy: ProxyConfig::default(),
        custom_headers: CustomHeaders::default(),
        mtls: None,
        projects_filter: BTreeSet::new(),
        timeout_seconds: DEFAULT_JIRA_TIMEOUT_SECONDS,
    }
}

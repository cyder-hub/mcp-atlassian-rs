use super::*;

#[derive(Debug, Clone, PartialEq)]
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

#[derive(Clone)]
pub(super) struct QueuedMockState {
    pub(super) responses: Arc<Mutex<VecDeque<(StatusCode, Value)>>>,
    pub(super) requests: Arc<Mutex<Vec<RecordedRequest>>>,
}

pub(super) async fn mock_handler(
    State(state): State<MockState>,
    method: Method,
    headers: HeaderMap,
    uri: axum::http::Uri,
    body: Bytes,
) -> Response {
    record_request(state.requests.clone(), method, headers, uri, body).await;
    (state.status, Json(state.response)).into_response()
}

pub(super) async fn invalid_json_handler(
    State(requests): State<Arc<Mutex<Vec<RecordedRequest>>>>,
    method: Method,
    headers: HeaderMap,
    uri: axum::http::Uri,
    body: Bytes,
) -> Response {
    record_request(requests, method, headers, uri, body).await;
    (StatusCode::OK, "not-json").into_response()
}

pub(super) async fn queued_mock_handler(
    State(state): State<QueuedMockState>,
    method: Method,
    headers: HeaderMap,
    uri: axum::http::Uri,
    body: Bytes,
) -> Response {
    record_request(state.requests.clone(), method, headers, uri, body).await;
    let (status, response) = {
        let mut responses = state.responses.lock().await;
        if responses.len() > 1 {
            responses.pop_front().unwrap()
        } else {
            responses
                .front()
                .cloned()
                .unwrap_or((StatusCode::OK, json!({})))
        }
    };

    (status, Json(response)).into_response()
}

pub(super) async fn record_request(
    requests: Arc<Mutex<Vec<RecordedRequest>>>,
    method: Method,
    headers: HeaderMap,
    uri: axum::http::Uri,
    body: Bytes,
) {
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
}

pub(super) async fn mock_server(
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
    let app = Router::new()
        .fallback(any(invalid_json_handler))
        .with_state(requests.clone());
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address: SocketAddr = listener.local_addr().unwrap();
    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });

    (format!("http://{address}"), requests)
}

pub(super) async fn queued_mock_server(
    responses: Vec<(StatusCode, Value)>,
) -> (String, Arc<Mutex<Vec<RecordedRequest>>>) {
    let requests = Arc::new(Mutex::new(Vec::new()));
    let app = Router::new()
        .fallback(any(queued_mock_handler))
        .with_state(QueuedMockState {
            responses: Arc::new(Mutex::new(VecDeque::from(responses))),
            requests: requests.clone(),
        });
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address: SocketAddr = listener.local_addr().unwrap();
    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });

    (format!("http://{address}"), requests)
}

pub(super) fn client(base_url: String) -> ConfluenceClient {
    client_with_spaces_filter(base_url, BTreeSet::new())
}

pub(super) fn cloud_client(base_url: String) -> ConfluenceClient {
    ConfluenceClient::new(ConfluenceConfig {
        base_url,
        deployment: ConfluenceDeployment::Cloud,
        auth: UpstreamAuth::Basic {
            username: "test-user".to_string(),
            api_token: "test-api-token".to_string(),
        },
        oauth_cloud_id: None,
        ssl_verify: true,
        proxy: ProxyConfig::default(),
        custom_headers: CustomHeaders::default(),
        mtls: None,
        spaces_filter: BTreeSet::new(),
        timeout_seconds: 75,
    })
    .unwrap()
}

pub(super) fn client_with_spaces_filter(
    base_url: String,
    spaces_filter: BTreeSet<String>,
) -> ConfluenceClient {
    ConfluenceClient::new(ConfluenceConfig {
        base_url,
        deployment: ConfluenceDeployment::ServerDataCenter,
        auth: UpstreamAuth::Pat {
            personal_token: "test-pat-value".to_string(),
        },
        oauth_cloud_id: None,
        ssl_verify: true,
        proxy: ProxyConfig::default(),
        custom_headers: CustomHeaders::default(),
        mtls: None,
        spaces_filter,
        timeout_seconds: 75,
    })
    .unwrap()
}

pub(super) fn query_value(path: &str, key: &str) -> Option<String> {
    let url = reqwest::Url::parse(&format!("http://example{path}")).unwrap();
    url.query_pairs()
        .find(|(name, _)| name == key)
        .map(|(_, value)| value.into_owned())
}

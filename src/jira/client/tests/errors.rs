use super::support::*;
use super::*;

#[tokio::test]
async fn issue_not_found_error_is_safe() {
    let (base_url, _requests) =
        mock_server_with_status(json!({"errorMessages": ["missing"]}), StatusCode::NOT_FOUND).await;
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
async fn invalid_json_response_includes_safe_request_context() {
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
    assert!(error.contains("GET /rest/api/2/issue/ABC-1"));
    assert!(!error.contains("Bearer"));
    assert!(!error.contains("test-pat-value"));
    assert_eq!(requests.len(), 1);
}

#[tokio::test]
async fn project_filter_rejects_project_metadata_without_http_request() {
    let (base_url, requests) = mock_server(json!([])).await;
    let mut config = config(base_url, JiraDeployment::ServerDataCenter);
    config.projects_filter = BTreeSet::from(["ABC".to_string()]);
    let client = JiraClient::new(config).unwrap();
    let versions_error = client
        .get_project_versions("XYZ".to_string())
        .await
        .unwrap_err()
        .to_string();
    let components_error = client
        .get_project_components("XYZ".to_string())
        .await
        .unwrap_err()
        .to_string();
    let requests = requests.lock().await;

    assert!(versions_error.contains("outside the configured Jira project filter"));
    assert!(components_error.contains("outside the configured Jira project filter"));
    assert!(requests.is_empty());
}

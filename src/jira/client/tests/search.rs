use super::support::*;
use super::*;

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
async fn cloud_search_allows_issue_without_key() {
    let (base_url, requests) = mock_server(json!({
        "issues": [{
            "id": "10001",
            "fields": {"summary": "Demo"}
        }],
        "isLast": true
    }))
    .await;
    let client = JiraClient::new(config(base_url, JiraDeployment::Cloud)).unwrap();
    let value = client
        .search(SearchRequest {
            jql: "project = SCRUM".to_string(),
            limit: Some(20),
            ..Default::default()
        })
        .await
        .unwrap();
    let requests = requests.lock().await;

    assert_eq!(requests[0].path, "/rest/api/3/search/jql");
    assert_eq!(value["issues"][0]["id"], "10001");
    assert!(value["issues"][0]["key"].is_null());
    assert_eq!(value["issues"][0]["summary"], "Demo");
}

#[tokio::test]
async fn cloud_search_retries_legacy_search_when_enhanced_rejects_unbounded_jql() {
    let (base_url, requests) = cloud_search_fallback_mock_server().await;
    let client = JiraClient::new(config(base_url, JiraDeployment::Cloud)).unwrap();
    let value = client
        .search(SearchRequest {
            jql: "created >= -30d ORDER BY updated DESC".to_string(),
            limit: Some(50),
            ..Default::default()
        })
        .await
        .unwrap();
    let requests = requests.lock().await;

    assert_eq!(value["total"], 1);
    assert_eq!(value["issues"][0]["key"], "ABC-1");
    assert_eq!(requests.len(), 2);
    assert_eq!(requests[0].method, Method::POST);
    assert_eq!(requests[0].path, "/rest/api/3/search/jql");
    assert_eq!(requests[1].method, Method::POST);
    assert_eq!(requests[1].path, "/rest/api/3/search");
    assert_eq!(
        requests[1].body["jql"],
        "created >= -30d ORDER BY updated DESC"
    );
    assert_eq!(requests[1].body["startAt"], 0);
    assert_eq!(requests[1].body["maxResults"], 50);
}

#[tokio::test]
async fn cloud_search_reports_unbounded_jql_when_legacy_search_is_removed() {
    let (base_url, requests) = removed_cloud_legacy_search_mock_server().await;
    let client = JiraClient::new(config(base_url, JiraDeployment::Cloud)).unwrap();
    let error = client
        .search(SearchRequest {
            jql: "ORDER BY created DESC".to_string(),
            limit: Some(20),
            ..Default::default()
        })
        .await
        .unwrap_err()
        .to_string();
    let requests = requests.lock().await;

    assert_eq!(requests.len(), 2);
    assert_eq!(requests[0].path, "/rest/api/3/search/jql");
    assert_eq!(requests[1].path, "/rest/api/3/search");
    assert!(error.contains("unbounded JQL"));
    assert!(error.contains("project = \"KEY\""));
    assert!(error.contains("ORDER BY created DESC"));
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

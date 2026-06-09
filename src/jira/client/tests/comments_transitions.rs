use super::support::*;
use super::*;

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
    let (base_url, _requests) = mock_server_with_status(json!({}), StatusCode::NO_CONTENT).await;
    let client = JiraClient::new(config(base_url, JiraDeployment::ServerDataCenter)).unwrap();
    let value = client
        .transition_issue("ABC-1".to_string(), "31".to_string(), None, None)
        .await
        .unwrap();

    assert_eq!(value["transition_id"], "31");
    assert!(value["response"].is_null());
}

use super::support::*;
use super::*;

#[tokio::test]
async fn client_searches_cloud_users_with_cql_endpoint() {
    let (base_url, requests) = mock_server(
        json!({
            "results": [{
                "title": "Ada Lovelace",
                "entityType": "user",
                "score": 0.9,
                "user": {
                    "accountId": "abc",
                    "displayName": "Ada Lovelace",
                    "email": "ada@example.com",
                    "accountStatus": "active"
                }
            }],
            "start": 0,
            "limit": 5,
            "totalSize": 1,
            "cqlQuery": "user.fullname ~ \"Ada\""
        }),
        StatusCode::OK,
    )
    .await;

    let response = cloud_client(base_url)
        .search_user("user.fullname ~ \"Ada\"", Some(5), None)
        .await
        .unwrap();

    assert_eq!(response.results.len(), 1);
    assert_eq!(
        response.results[0]
            .user
            .as_ref()
            .and_then(|user| user.display_name.as_deref()),
        Some("Ada Lovelace")
    );
    let requests = requests.lock().await;
    assert_eq!(requests[0].method, Method::GET);
    assert!(requests[0].path.starts_with("/rest/api/search/user?"));
    assert_eq!(
        query_value(&requests[0].path, "cql").as_deref(),
        Some("user.fullname ~ \"Ada\"")
    );
    assert_eq!(
        query_value(&requests[0].path, "limit").as_deref(),
        Some("5")
    );
}

#[tokio::test]
async fn client_searches_server_users_through_group_members() {
    let (base_url, requests) = mock_server(
        json!({
            "results": [
                {"username": "ada", "displayName": "Ada Lovelace", "email": "ada@example.com"},
                {"username": "grace", "displayName": "Grace Hopper", "email": "grace@example.com"}
            ],
            "start": 0,
            "limit": 200,
            "size": 2,
            "_links": {}
        }),
        StatusCode::OK,
    )
    .await;

    let response = client(base_url)
        .search_user(
            "user.fullname ~ \"Ada\"",
            Some(10),
            Some("confluence users"),
        )
        .await
        .unwrap();

    assert_eq!(response.results.len(), 1);
    assert_eq!(response.results[0].title.as_deref(), Some("Ada Lovelace"));
    assert_eq!(
        response.results[0].to_simplified_value()["user"]["active"],
        json!(true)
    );
    let requests = requests.lock().await;
    assert_eq!(requests[0].method, Method::GET);
    assert!(
        requests[0]
            .path
            .starts_with("/rest/api/group/confluence%20users/member?")
    );
    assert_eq!(
        query_value(&requests[0].path, "start").as_deref(),
        Some("0")
    );
    assert_eq!(
        query_value(&requests[0].path, "limit").as_deref(),
        Some("200")
    );
}

#[tokio::test]
async fn client_search_user_rejects_invalid_limit_before_http_request() {
    let (base_url, requests) = mock_server(json!({}), StatusCode::OK).await;

    let error = client(base_url)
        .search_user("Ada", Some(51), None)
        .await
        .unwrap_err();

    assert!(
        error
            .to_string()
            .contains("limit must be less than or equal to 50")
    );
    assert_eq!(requests.lock().await.len(), 0);
}

#[tokio::test]
async fn client_search_user_preserves_auth_errors() {
    let (base_url, _requests) = mock_server(
        json!({"errorMessages": ["auth failed"]}),
        StatusCode::UNAUTHORIZED,
    )
    .await;

    let error = cloud_client(base_url)
        .search_user("user.fullname ~ \"Ada\"", Some(10), None)
        .await
        .unwrap_err();

    assert!(matches!(
        error,
        AtlassianError::HttpStatus { status: 401, .. }
    ));
}

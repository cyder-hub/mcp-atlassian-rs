use super::support::*;
use super::*;

#[tokio::test]
async fn client_gets_cloud_page_views_with_optional_title() {
    let (base_url, requests) = queued_mock_server(vec![
        (
            StatusCode::OK,
            json!({
                "id": "123",
                "title": "Roadmap"
            }),
        ),
        (
            StatusCode::OK,
            json!({
                "count": 42,
                "lastSeen": "2026-06-04T12:00:00Z"
            }),
        ),
    ])
    .await;

    let views = cloud_client(base_url)
        .get_page_views("123", true, Some("2026-01-01"), Some("2026-01-31"))
        .await
        .unwrap();

    assert_eq!(views.page_id.as_deref(), Some("123"));
    assert_eq!(views.title.as_deref(), Some("Roadmap"));
    assert_eq!(views.count, Some(42));
    assert_eq!(views.last_seen.as_deref(), Some("2026-06-04T12:00:00Z"));
    let requests = requests.lock().await;
    assert_eq!(requests.len(), 2);
    assert_eq!(requests[0].method, Method::GET);
    assert!(requests[0].path.starts_with("/rest/api/content/123?"));
    assert_eq!(
        query_value(&requests[0].path, "expand").as_deref(),
        Some("title")
    );
    assert!(
        requests[1]
            .path
            .starts_with("/rest/api/analytics/content/123/views?")
    );
    assert_eq!(
        query_value(&requests[1].path, "from").as_deref(),
        Some("2026-01-01")
    );
    assert_eq!(
        query_value(&requests[1].path, "to").as_deref(),
        Some("2026-01-31")
    );
}

#[tokio::test]
async fn client_gets_cloud_page_views_without_title_lookup() {
    let (base_url, requests) = mock_server(
        json!({
            "count": 7,
            "lastSeen": "2026-06-04T12:00:00Z"
        }),
        StatusCode::OK,
    )
    .await;

    let views = cloud_client(base_url)
        .get_page_views("123", false, None, None)
        .await
        .unwrap();

    assert_eq!(views.page_id.as_deref(), Some("123"));
    assert_eq!(views.title, None);
    assert_eq!(views.count, Some(7));
    let requests = requests.lock().await;
    assert_eq!(requests.len(), 1);
    assert_eq!(requests[0].path, "/rest/api/analytics/content/123/views");
}

#[tokio::test]
async fn client_rejects_page_views_for_server_before_http_request() {
    let (base_url, requests) = mock_server(json!({}), StatusCode::OK).await;
    let error = client(base_url)
        .get_page_views("123", true, None, None)
        .await
        .unwrap_err();

    assert!(
        error
            .to_string()
            .contains("only available for Confluence Cloud")
    );
    assert!(requests.lock().await.is_empty());
}

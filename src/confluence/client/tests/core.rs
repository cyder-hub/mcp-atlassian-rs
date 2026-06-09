use super::support::*;
use super::*;

#[tokio::test]
async fn client_preserves_wiki_base_path_when_building_rest_url() {
    let (base_url, requests) = mock_server(
        json!({
            "id": "123",
            "title": "Roadmap",
            "body": {"storage": {"value": "<p>Hello</p>"}}
        }),
        StatusCode::OK,
    )
    .await;
    let client = client(format!("{base_url}/wiki"));
    let page = client
        .get_page_by_id("123", &["body.storage", "version", "space"])
        .await
        .unwrap();

    assert_eq!(page.id.as_deref(), Some("123"));
    let requests = requests.lock().await;
    assert_eq!(requests[0].method, Method::GET);
    assert_eq!(
        requests[0].path,
        "/wiki/rest/api/content/123?expand=body.storage%2Cversion%2Cspace"
    );
    assert_eq!(
        requests[0].authorization.as_deref(),
        Some("Bearer test-pat-value")
    );
}

#[tokio::test]
async fn client_maps_http_status_errors_without_echoing_body() {
    let (base_url, _requests) = mock_server(
        json!({"errorMessages": ["page not found"]}),
        StatusCode::NOT_FOUND,
    )
    .await;
    let error = client(base_url)
        .get_page_by_id("missing", &[])
        .await
        .unwrap_err();

    assert!(matches!(
        error,
        AtlassianError::HttpStatus { status: 404, .. }
    ));
    assert!(error.to_string().contains("page not found"));
}

#[tokio::test]
async fn client_maps_invalid_json_with_request_context() {
    let (base_url, _requests) = invalid_json_mock_server().await;
    let error = client(base_url)
        .get_page_by_id("123", &[])
        .await
        .unwrap_err();

    assert!(matches!(error, AtlassianError::JsonDecode { .. }));
    assert!(error.to_string().contains("GET /rest/api/content/123"));
}

#[tokio::test]
async fn client_parses_missing_fields_without_failure() {
    let (base_url, _requests) = mock_server(json!({}), StatusCode::OK).await;
    let page = client(base_url).get_page_by_id("123", &[]).await.unwrap();

    assert_eq!(page.id, None);
    assert_eq!(page.title, None);
    assert!(page.body.is_null());
}

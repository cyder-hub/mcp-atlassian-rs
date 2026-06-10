use super::support::*;
use super::*;

#[tokio::test]
async fn client_parses_search_response() {
    let (base_url, requests) = mock_server(
        json!({
            "results": [{
                "id": "123",
                "title": "Roadmap",
                "content": {"id": "123", "title": "Roadmap"}
            }],
            "start": 0,
            "limit": 10,
            "size": 1
        }),
        StatusCode::OK,
    )
    .await;
    let response = client(base_url)
        .search_cql("space = ENG", Some(0), Some(10))
        .await
        .unwrap();

    assert_eq!(response.results.len(), 1);
    assert_eq!(response.results[0].title.as_deref(), Some("Roadmap"));
    assert!(
        requests.lock().await[0]
            .path
            .starts_with("/rest/api/content/search?")
    );
}

#[tokio::test]
async fn search_content_converts_simple_query_to_site_search_and_applies_spaces_filter() {
    let (base_url, requests) = mock_server(
        json!({
            "results": [{
                "id": "123",
                "title": "Roadmap",
                "content": {"id": "123", "title": "Roadmap"}
            }],
            "start": 0,
            "limit": 10,
            "size": 1
        }),
        StatusCode::OK,
    )
    .await;

    let response = client(base_url)
        .search_content("project docs", Some(10), Some("ENG, ~me"))
        .await
        .unwrap();

    assert_eq!(response.results.len(), 1);
    let requests = requests.lock().await;
    assert_eq!(
        query_value(&requests[0].path, "cql").as_deref(),
        Some(r#"(siteSearch ~ "project docs") AND (space = ENG OR space = "~me")"#)
    );
    assert_eq!(
        query_value(&requests[0].path, "limit").as_deref(),
        Some("10")
    );
}

#[tokio::test]
async fn search_content_falls_back_to_text_search_when_site_search_is_rejected() {
    let (base_url, requests) = queued_mock_server(vec![
        (
            StatusCode::BAD_REQUEST,
            json!({"errorMessages": ["siteSearch unsupported"]}),
        ),
        (
            StatusCode::OK,
            json!({
                "results": [{"id": "123", "title": "Roadmap"}],
                "start": 0,
                "limit": 10,
                "size": 1
            }),
        ),
    ])
    .await;

    let response = client(base_url)
        .search_content("project docs", Some(10), None)
        .await
        .unwrap();

    assert_eq!(response.results.len(), 1);
    let requests = requests.lock().await;
    assert_eq!(requests.len(), 2);
    assert_eq!(
        query_value(&requests[0].path, "cql").as_deref(),
        Some(r#"siteSearch ~ "project docs""#)
    );
    assert_eq!(
        query_value(&requests[1].path, "cql").as_deref(),
        Some(r#"text ~ "project docs""#)
    );
}

#[tokio::test]
async fn search_content_does_not_fallback_on_auth_error() {
    let (base_url, requests) = queued_mock_server(vec![
        (
            StatusCode::UNAUTHORIZED,
            json!({"errorMessages": ["auth failed"]}),
        ),
        (
            StatusCode::OK,
            json!({"results": [{"id": "should-not-be-read"}]}),
        ),
    ])
    .await;

    let error = client(base_url)
        .search_content("project docs", Some(10), None)
        .await
        .unwrap_err();

    assert!(matches!(
        error,
        UpstreamError::HttpStatus { status: 401, .. }
    ));
    assert_eq!(requests.lock().await.len(), 1);
}

#[tokio::test]
async fn search_content_uses_config_space_filter_and_explicit_empty_disables_it() {
    let (base_url, requests) = mock_server(json!({"results": []}), StatusCode::OK).await;
    let client = client_with_spaces_filter(
        base_url,
        BTreeSet::from(["ENG".to_string(), "~personal".to_string()]),
    );

    client
        .search_content("type=page", Some(10), None)
        .await
        .unwrap();
    client
        .search_content("type=page", Some(10), Some(""))
        .await
        .unwrap();

    let requests = requests.lock().await;
    assert_eq!(
        query_value(&requests[0].path, "cql").as_deref(),
        Some(r#"(type=page) AND (space = ENG OR space = "~personal")"#)
    );
    assert_eq!(
        query_value(&requests[1].path, "cql").as_deref(),
        Some("type=page")
    );
}

#[tokio::test]
async fn search_content_rejects_invalid_limit_before_request() {
    let (base_url, requests) = mock_server(json!({"results": []}), StatusCode::OK).await;

    let error = client(base_url)
        .search_content("docs", Some(51), None)
        .await
        .unwrap_err();

    assert!(matches!(error, UpstreamError::InvalidInput { .. }));
    assert_eq!(requests.lock().await.len(), 0);
}

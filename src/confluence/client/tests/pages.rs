use super::support::*;
use super::*;

#[tokio::test]
async fn client_gets_page_by_title_and_space_key() {
    let (base_url, requests) = mock_server(
        json!({
            "results": [{
                "id": "123",
                "title": "Roadmap",
                "space": {"key": "ENG"},
                "body": {"storage": {"value": "<p>Hello</p>"}}
            }],
            "start": 0,
            "limit": 1,
            "size": 1
        }),
        StatusCode::OK,
    )
    .await;
    let page = client(base_url)
        .get_page_by_title("ENG", "Roadmap", &["body.storage", "version", "space"])
        .await
        .unwrap()
        .unwrap();

    assert_eq!(page.id.as_deref(), Some("123"));
    assert_eq!(page.title.as_deref(), Some("Roadmap"));
    let requests = requests.lock().await;
    assert_eq!(requests[0].method, Method::GET);
    assert!(requests[0].path.starts_with("/rest/api/content?"));
    assert_eq!(
        query_value(&requests[0].path, "spaceKey").as_deref(),
        Some("ENG")
    );
    assert_eq!(
        query_value(&requests[0].path, "title").as_deref(),
        Some("Roadmap")
    );
    assert_eq!(
        query_value(&requests[0].path, "limit").as_deref(),
        Some("1")
    );
}

#[tokio::test]
async fn client_gets_specific_page_history_version() {
    let (base_url, requests) = mock_server(
        json!({
            "id": "123",
            "title": "Roadmap",
            "status": "historical",
            "space": {"key": "ENG"},
            "version": {"number": 2},
            "body": {"storage": {"value": "<p>Version two</p>"}}
        }),
        StatusCode::OK,
    )
    .await;

    let page = client(base_url)
        .get_page_history("123", 2, &["body.storage", "version", "space"])
        .await
        .unwrap();

    assert_eq!(page.version.and_then(|version| version.number), Some(2));
    let requests = requests.lock().await;
    assert_eq!(requests[0].method, Method::GET);
    assert!(requests[0].path.starts_with("/rest/api/content/123?"));
    assert_eq!(
        query_value(&requests[0].path, "status").as_deref(),
        Some("historical")
    );
    assert_eq!(
        query_value(&requests[0].path, "version").as_deref(),
        Some("2")
    );
    assert_eq!(
        query_value(&requests[0].path, "expand").as_deref(),
        Some("body.storage,version,space")
    );
}

#[tokio::test]
async fn client_rejects_zero_page_history_version_before_http() {
    let (base_url, requests) = mock_server(json!({}), StatusCode::OK).await;
    let error = client(base_url)
        .get_page_history("123", 0, &[])
        .await
        .unwrap_err();

    assert!(error.to_string().contains("version must be positive"));
    assert!(requests.lock().await.is_empty());
}

#[tokio::test]
async fn client_get_page_by_title_returns_none_for_empty_results() {
    let (base_url, _requests) = mock_server(json!({"results": []}), StatusCode::OK).await;

    let page = client(base_url)
        .get_page_by_title("ENG", "Missing", &["body.storage"])
        .await
        .unwrap();

    assert_eq!(page, None);
}

#[tokio::test]
async fn client_gets_page_children_and_optional_folders() {
    let (base_url, requests) = queued_mock_server(vec![
        (
            StatusCode::OK,
            json!({
                "results": [{
                    "id": "201",
                    "title": "Child page",
                    "type": "page",
                    "body": {"storage": {"value": "<p>Child</p>"}}
                }]
            }),
        ),
        (
            StatusCode::OK,
            json!({
                "results": [{
                    "id": "301",
                    "title": "Folder",
                    "type": "folder"
                }]
            }),
        ),
    ])
    .await;

    let response = client(base_url)
        .get_page_children("123", Some(0), Some(2), &["version", "body.storage"], true)
        .await
        .unwrap();

    let children = response.results;
    assert_eq!(children.len(), 2);
    assert_eq!(children[0].content_type.as_deref(), Some("page"));
    assert_eq!(children[1].content_type.as_deref(), Some("folder"));
    assert_eq!(response.page_results, 1);
    assert_eq!(response.folder_results, 1);
    assert_eq!(response.page_query.requested_limit, 2);
    assert_eq!(response.folder_query.unwrap().requested_limit, 1);
    let requests = requests.lock().await;
    assert_eq!(requests.len(), 2);
    assert!(
        requests[0]
            .path
            .starts_with("/rest/api/content/123/child/page?")
    );
    assert!(
        requests[1]
            .path
            .starts_with("/rest/api/content/123/child/folder?")
    );
    assert_eq!(
        query_value(&requests[0].path, "limit").as_deref(),
        Some("2")
    );
    assert_eq!(
        query_value(&requests[1].path, "limit").as_deref(),
        Some("1")
    );
}

#[tokio::test]
async fn client_page_children_with_folders_respects_combined_limit() {
    let (base_url, requests) = queued_mock_server(vec![(
        StatusCode::OK,
        json!({
            "results": [
                {"id": "201", "title": "Child one", "type": "page"},
                {"id": "202", "title": "Child two", "type": "page"}
            ],
            "start": 0,
            "limit": 2,
            "size": 2,
            "_links": {"next": "/rest/api/content/123/child/page?start=2"}
        }),
    )])
    .await;

    let response = client(base_url)
        .get_page_children("123", Some(0), Some(2), &[], true)
        .await
        .unwrap();

    assert_eq!(response.results.len(), 2);
    assert_eq!(response.page_results, 2);
    assert_eq!(response.folder_results, 0);
    assert!(response.page_query.has_more);
    assert_eq!(response.page_query.next_start, Some(2));
    assert!(response.folder_query.is_none());
    let requests = requests.lock().await;
    assert_eq!(requests.len(), 1);
    assert!(
        requests[0]
            .path
            .starts_with("/rest/api/content/123/child/page?")
    );
}

#[tokio::test]
async fn client_page_children_with_folders_starts_folders_after_page_tail() {
    let page_results: Vec<Value> = (0..10)
        .map(|index| {
            json!({
                "id": format!("2{index:02}"),
                "title": format!("Child {index}"),
                "type": "page"
            })
        })
        .collect();
    let folder_results: Vec<Value> = (0..10)
        .map(|index| {
            json!({
                "id": format!("3{index:02}"),
                "title": format!("Folder {index}"),
                "type": "folder"
            })
        })
        .collect();
    let (base_url, requests) = queued_mock_server(vec![
        (
            StatusCode::OK,
            json!({
                "results": page_results,
                "start": 50,
                "limit": 50,
                "size": 10
            }),
        ),
        (
            StatusCode::OK,
            json!({
                "results": folder_results,
                "start": 0,
                "limit": 40,
                "size": 10
            }),
        ),
    ])
    .await;

    let response = client(base_url)
        .get_page_children("123", Some(50), Some(50), &[], true)
        .await
        .unwrap();

    assert_eq!(response.results.len(), 20);
    assert_eq!(response.page_results, 10);
    assert_eq!(response.folder_results, 10);
    let folder_query = response.folder_query.unwrap();
    assert_eq!(folder_query.requested_start, 0);
    assert_eq!(folder_query.requested_limit, 40);
    let requests = requests.lock().await;
    assert_eq!(requests.len(), 2);
    assert!(
        requests[1]
            .path
            .starts_with("/rest/api/content/123/child/folder?")
    );
    assert_eq!(
        query_value(&requests[1].path, "start").as_deref(),
        Some("0")
    );
    assert_eq!(
        query_value(&requests[1].path, "limit").as_deref(),
        Some("40")
    );
}

#[tokio::test]
async fn client_page_children_with_folders_offsets_folders_after_empty_page_tail() {
    let page_count_results: Vec<Value> = (0..10)
        .map(|index| {
            json!({
                "id": format!("2{index:02}"),
                "title": format!("Child {index}"),
                "type": "page"
            })
        })
        .collect();
    let folder_results: Vec<Value> = (15..18)
        .map(|index| {
            json!({
                "id": format!("3{index:02}"),
                "title": format!("Folder {index}"),
                "type": "folder"
            })
        })
        .collect();
    let (base_url, requests) = queued_mock_server(vec![
        (
            StatusCode::OK,
            json!({
                "results": [],
                "start": 25,
                "limit": 25,
                "size": 0
            }),
        ),
        (
            StatusCode::OK,
            json!({
                "results": page_count_results,
                "start": 0,
                "limit": 25,
                "size": 10
            }),
        ),
        (
            StatusCode::OK,
            json!({
                "results": folder_results,
                "start": 15,
                "limit": 25,
                "size": 3
            }),
        ),
    ])
    .await;

    let response = client(base_url)
        .get_page_children("123", Some(25), Some(25), &[], true)
        .await
        .unwrap();

    assert_eq!(response.results.len(), 3);
    assert_eq!(response.page_results, 0);
    assert_eq!(response.folder_results, 3);
    let folder_query = response.folder_query.unwrap();
    assert_eq!(folder_query.requested_start, 15);
    assert_eq!(folder_query.requested_limit, 25);
    let requests = requests.lock().await;
    assert_eq!(requests.len(), 3);
    assert_eq!(
        query_value(&requests[0].path, "start").as_deref(),
        Some("25")
    );
    assert!(
        requests[1]
            .path
            .starts_with("/rest/api/content/123/child/page?")
    );
    assert_eq!(
        query_value(&requests[1].path, "start").as_deref(),
        Some("0")
    );
    assert!(
        requests[2]
            .path
            .starts_with("/rest/api/content/123/child/folder?")
    );
    assert_eq!(
        query_value(&requests[2].path, "start").as_deref(),
        Some("15")
    );
    assert_eq!(
        query_value(&requests[2].path, "limit").as_deref(),
        Some("25")
    );
}

#[tokio::test]
async fn client_page_children_with_folders_caps_page_recount_for_large_offsets() {
    let recount_limit = 100;
    let max_recount_requests = 20;
    let requested_start = 2_500;
    let mut responses = vec![(
        StatusCode::OK,
        json!({
            "results": [],
            "start": requested_start,
            "limit": 25,
            "size": 0
        }),
    )];

    for page_index in 0..max_recount_requests {
        let start = page_index * recount_limit;
        let next_start = start + recount_limit;
        let page_results: Vec<Value> = (0..recount_limit)
            .map(|index| {
                json!({
                    "id": format!("2{page_index:02}{index:03}"),
                    "title": format!("Child {page_index}-{index}"),
                    "type": "page"
                })
            })
            .collect();
        responses.push((
            StatusCode::OK,
            json!({
                "results": page_results,
                "start": start,
                "limit": recount_limit,
                "size": recount_limit,
                "_links": {"next": format!("/rest/api/content/123/child/page?start={next_start}")}
            }),
        ));
    }

    let (base_url, requests) = queued_mock_server(responses).await;

    let error = client(base_url)
        .get_page_children("123", Some(requested_start), Some(25), &[], true)
        .await
        .unwrap_err();

    assert!(
        error
            .to_string()
            .contains("page-child recount is capped at 20 requests")
    );
    let requests = requests.lock().await;
    assert_eq!(requests.len(), 1 + max_recount_requests as usize);
    assert!(
        requests
            .iter()
            .all(|request| request.path.contains("/child/page?"))
    );
    assert_eq!(
        query_value(&requests[1].path, "limit").as_deref(),
        Some("100")
    );
}

#[tokio::test]
async fn client_gets_space_pages_with_ancestors_expand() {
    let (base_url, requests) = mock_server(
        json!({
            "results": [{
                "id": "100",
                "title": "Home",
                "ancestors": [],
                "extensions": {"position": 0}
            }],
            "_links": {}
        }),
        StatusCode::OK,
    )
    .await;

    let response = client(base_url)
        .get_space_pages("ENG", Some(0), Some(1), &["ancestors"])
        .await
        .unwrap();

    assert_eq!(response.results.len(), 1);
    let requests = requests.lock().await;
    assert!(requests[0].path.starts_with("/rest/api/content?"));
    assert_eq!(
        query_value(&requests[0].path, "spaceKey").as_deref(),
        Some("ENG")
    );
    assert_eq!(
        query_value(&requests[0].path, "type").as_deref(),
        Some("page")
    );
    assert_eq!(
        query_value(&requests[0].path, "expand").as_deref(),
        Some("ancestors")
    );
}

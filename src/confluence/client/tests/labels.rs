use super::support::*;
use super::*;

#[tokio::test]
async fn client_gets_labels_for_content_id() {
    let (base_url, requests) = mock_server(
            json!({
                "results": [
                    {"id": "label-1", "name": "draft", "prefix": "global", "label": "draft", "type": "label"},
                    {"id": "label-2", "name": "team", "prefix": "my", "label": "team", "type": "label"}
                ],
                "start": 0,
                "limit": 200,
                "size": 2
            }),
            StatusCode::OK,
        )
        .await;

    let response = client(base_url).get_labels("123").await.unwrap();

    assert_eq!(response.results.len(), 2);
    assert_eq!(response.results[0].name.as_deref(), Some("draft"));
    assert_eq!(response.results[1].prefix.as_deref(), Some("my"));
    let requests = requests.lock().await;
    assert_eq!(requests[0].method, Method::GET);
    assert_eq!(requests[0].path, "/rest/api/content/123/label");
}

#[tokio::test]
async fn client_adds_label_and_refreshes_label_list() {
    let (base_url, requests) = queued_mock_server(vec![
            (StatusCode::OK, json!({})),
            (
                StatusCode::OK,
                json!({
                    "results": [
                        {"id": "label-1", "name": "draft", "prefix": "global", "label": "draft", "type": "label"}
                    ],
                    "start": 0,
                    "limit": 200,
                    "size": 1
                }),
            ),
        ])
        .await;

    let response = client(base_url).add_label("123", "draft").await.unwrap();

    assert_eq!(response.results.len(), 1);
    assert_eq!(response.results[0].name.as_deref(), Some("draft"));
    let requests = requests.lock().await;
    assert_eq!(requests.len(), 2);
    assert_eq!(requests[0].method, Method::POST);
    assert_eq!(requests[0].path, "/rest/api/content/123/label");
    assert_eq!(requests[0].body[0]["prefix"], json!("global"));
    assert_eq!(requests[0].body[0]["name"], json!("draft"));
    assert_eq!(requests[1].method, Method::GET);
    assert_eq!(requests[1].path, "/rest/api/content/123/label");
}

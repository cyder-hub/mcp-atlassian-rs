use super::support::*;
use super::*;

#[tokio::test]
async fn client_gets_page_comments_with_expanded_author_and_body() {
    let (base_url, requests) = mock_server(
        json!({
            "results": [{
                "id": "c-1",
                "type": "comment",
                "body": {"storage": {"value": "<p>First comment</p>"}},
                "version": {"number": 2, "by": {"displayName": "Ada"}},
                "container": {"id": "123", "type": "page", "title": "Roadmap"},
                "extensions": {"location": "footer"}
            }],
            "start": 0,
            "limit": 25,
            "size": 1
        }),
        StatusCode::OK,
    )
    .await;

    let response = client(base_url).get_page_comments("123").await.unwrap();

    assert_eq!(response.results.len(), 1);
    let simplified = response.results[0].to_simplified_value();
    assert_eq!(simplified["body"], json!("First comment"));
    assert_eq!(simplified["author"]["display_name"], json!("Ada"));
    assert_eq!(simplified["location"], json!("footer"));
    let requests = requests.lock().await;
    assert_eq!(requests[0].method, Method::GET);
    assert!(
        requests[0]
            .path
            .starts_with("/rest/api/content/123/child/comment?")
    );
    assert!(
        query_value(&requests[0].path, "expand")
            .unwrap()
            .contains("body.storage")
    );
    assert_eq!(
        query_value(&requests[0].path, "depth").as_deref(),
        Some("all")
    );
}

#[tokio::test]
async fn client_adds_and_replies_to_comments_with_storage_payloads() {
    let (base_url, requests) = queued_mock_server(vec![
        (
            StatusCode::OK,
            json!({
                "id": "c-1",
                "type": "comment",
                "body": {"storage": {"value": "<p>Comment</p>"}},
                "container": {"id": "123", "type": "page"}
            }),
        ),
        (
            StatusCode::OK,
            json!({
                "id": "c-2",
                "type": "comment",
                "body": {"storage": {"value": "<p>Reply</p>"}},
                "container": {"id": "c-1", "type": "comment"}
            }),
        ),
    ])
    .await;

    let first = client(base_url.clone())
        .add_comment("123", "<p>Comment</p>")
        .await
        .unwrap();
    let reply = client(base_url)
        .reply_to_comment("c-1", "<p>Reply</p>")
        .await
        .unwrap();

    assert_eq!(first.id.as_deref(), Some("c-1"));
    assert_eq!(reply.id.as_deref(), Some("c-2"));
    let requests = requests.lock().await;
    assert_eq!(requests.len(), 2);
    assert_eq!(requests[0].method, Method::POST);
    assert_eq!(requests[0].path, "/rest/api/content");
    assert_eq!(requests[0].body["type"], json!("comment"));
    assert_eq!(requests[0].body["container"]["id"], json!("123"));
    assert_eq!(requests[0].body["container"]["type"], json!("page"));
    assert_eq!(
        requests[0].body["body"]["storage"]["value"],
        json!("<p>Comment</p>")
    );
    assert_eq!(requests[1].body["container"]["id"], json!("c-1"));
    assert_eq!(requests[1].body["container"]["type"], json!("comment"));
    assert_eq!(
        requests[1].body["body"]["storage"]["value"],
        json!("<p>Reply</p>")
    );
}

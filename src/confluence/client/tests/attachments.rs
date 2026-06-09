use super::support::*;
use super::*;

#[tokio::test]
async fn client_gets_content_attachments_with_pagination() {
    let (base_url, requests) = mock_server(
        json!({
            "results": [
                {
                    "id": "att-1",
                    "type": "attachment",
                    "title": "file.png",
                    "status": "current",
                    "extensions": {"mediaType": "image/png", "fileSize": 42},
                    "_links": {"download": "/download/attachments/att-1/file.png"}
                },
                {
                    "id": "att-2",
                    "type": "attachment",
                    "title": "notes.txt",
                    "metadata": {"mediaType": "text/plain", "fileSize": 12}
                }
            ],
            "start": 5,
            "limit": 2,
            "size": 2,
            "_links": {"next": "/rest/api/content/123/child/attachment?start=7"}
        }),
        StatusCode::OK,
    )
    .await;

    let response = client(base_url)
        .get_attachments("123", Some(5), Some(2), None, None)
        .await
        .unwrap();

    assert_eq!(response.results.len(), 2);
    assert_eq!(response.results[0].media_type(), Some("image/png"));
    assert_eq!(response.results[1].media_type(), Some("text/plain"));
    let requests = requests.lock().await;
    assert!(
        requests[0]
            .path
            .starts_with("/rest/api/content/123/child/attachment?")
    );
    assert_eq!(
        query_value(&requests[0].path, "start").as_deref(),
        Some("5")
    );
    assert_eq!(
        query_value(&requests[0].path, "limit").as_deref(),
        Some("2")
    );
    assert_eq!(
        query_value(&requests[0].path, "expand").as_deref(),
        Some("metadata,extensions,version")
    );
}

#[tokio::test]
async fn client_filters_content_attachments_by_filename_and_media_type() {
    let (base_url, requests) = mock_server(
        json!({
            "results": [
                {
                    "id": "att-1",
                    "title": "file.png",
                    "extensions": {"mediaType": "image/png"}
                },
                {
                    "id": "att-2",
                    "title": "notes.txt",
                    "metadata": {"mediaType": "text/plain"}
                }
            ],
            "start": 0,
            "limit": 50,
            "size": 2
        }),
        StatusCode::OK,
    )
    .await;

    let response = client(base_url)
        .get_attachments("123", None, None, Some("notes.txt"), Some("text/plain"))
        .await
        .unwrap();

    assert_eq!(response.results.len(), 1);
    assert_eq!(response.results[0].id.as_deref(), Some("att-2"));
    assert_eq!(response.size, Some(1));
    let requests = requests.lock().await;
    assert_eq!(requests.len(), 1);
    assert!(query_value(&requests[0].path, "filename").is_none());
    assert!(query_value(&requests[0].path, "media-type").is_none());
}

#[tokio::test]
async fn client_rejects_invalid_attachment_limit_before_http() {
    let (base_url, requests) = mock_server(json!({}), StatusCode::OK).await;
    let error = client(base_url)
        .get_attachments("123", None, Some(101), None, None)
        .await
        .unwrap_err();

    assert!(
        error
            .to_string()
            .contains("limit must be less than or equal to 100")
    );
    assert!(requests.lock().await.is_empty());
}

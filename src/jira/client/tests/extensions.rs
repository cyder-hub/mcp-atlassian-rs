use super::support::*;
use super::*;

#[tokio::test]
async fn development_info_resolves_issue_key_to_numeric_id() {
    let (base_url, requests) = jira_extension_mock_server().await;
    let client = JiraClient::new(config(base_url, JiraDeployment::ServerDataCenter)).unwrap();

    client
        .get_issue_development_info(
            "ABC-1".to_string(),
            Some("github".to_string()),
            Some("pullrequest".to_string()),
        )
        .await
        .unwrap();
    let requests = requests.lock().await;

    assert_eq!(requests[0].path, "/rest/api/2/issue/ABC-1?fields=id%2Ckey");
    assert_eq!(
        requests[1].path,
        "/rest/dev-status/1.0/issue/detail?issueId=10001&applicationType=github&dataType=pullrequest"
    );
}

#[tokio::test]
async fn jira_product_extension_helpers_use_expected_endpoints() {
    let (base_url, requests) = jira_extension_mock_server().await;
    let client = JiraClient::new(config(base_url, JiraDeployment::ServerDataCenter)).unwrap();

    client
        .get_agile_boards(
            Some("ABC".to_string()),
            Some("scrum".to_string()),
            Some(0),
            Some(10),
        )
        .await
        .unwrap();
    client
        .get_board_issues(
            1,
            Some("project = ABC".to_string()),
            Some(vec!["summary".to_string()]),
            Some(0),
            Some(10),
        )
        .await
        .unwrap();
    client
        .get_sprints_from_board(1, Some(vec!["active".to_string()]), Some(0), Some(10))
        .await
        .unwrap();
    client
        .get_sprint_issues(2, Some(vec!["summary".to_string()]), Some(0), Some(10))
        .await
        .unwrap();
    client
        .create_sprint(json!({"name": "Sprint", "originBoardId": 1}))
        .await
        .unwrap();
    client
        .update_sprint(2, json!({"name": "Sprint updated"}))
        .await
        .unwrap();
    client
        .add_issues_to_sprint(2, vec!["ABC-1".to_string()])
        .await
        .unwrap();
    client
        .get_service_desk_for_project("ABC".to_string())
        .await
        .unwrap();
    client
        .get_service_desk_queues("4".to_string(), Some(0), Some(50))
        .await
        .unwrap();
    client
        .get_queue_issues("4".to_string(), "47".to_string(), Some(0), Some(50))
        .await
        .unwrap();
    client
        .get_issue_development_info(
            "10001".to_string(),
            Some("github".to_string()),
            Some("pullrequest".to_string()),
        )
        .await
        .unwrap();
    client
        .get_issues_development_info(vec!["10001".to_string()], None, None)
        .await
        .unwrap();
    let requests = requests.lock().await;

    assert!(requests[0].path.starts_with("/rest/agile/1.0/board?"));
    assert!(
        requests[1]
            .path
            .starts_with("/rest/agile/1.0/board/1/issue?")
    );
    assert!(
        requests[2]
            .path
            .starts_with("/rest/agile/1.0/board/1/sprint?")
    );
    assert!(
        requests[3]
            .path
            .starts_with("/rest/agile/1.0/sprint/2/issue?")
    );
    assert_eq!(requests[4].path, "/rest/agile/1.0/sprint");
    assert_eq!(requests[5].path, "/rest/agile/1.0/sprint/2");
    assert_eq!(requests[6].path, "/rest/agile/1.0/sprint/2/issue");
    assert_eq!(requests[7].path, "/rest/servicedeskapi/servicedesk");
    assert_eq!(
        requests[8].path,
        "/rest/servicedeskapi/servicedesk/4/queue?start=0&limit=50"
    );
    assert_eq!(
        requests[9].path,
        "/rest/servicedeskapi/servicedesk/4/queue/47/issue?start=0&limit=50"
    );
    assert!(
        requests[10]
            .path
            .starts_with("/rest/dev-status/1.0/issue/detail?")
    );
    assert!(
        requests[11]
            .path
            .starts_with("/rest/dev-status/1.0/issue/detail?")
    );
}

#[tokio::test]
async fn agile_helpers_return_product_unavailable_when_software_rest_is_missing() {
    let (base_url, requests) = jira_extension_mock_server().await;
    let client = JiraClient::new(config(base_url, JiraDeployment::ServerDataCenter)).unwrap();

    let value = client
        .get_agile_boards(Some("NOAGILE".to_string()), None, None, None)
        .await
        .unwrap();
    let requests = requests.lock().await;

    assert_eq!(value["success"], false);
    assert_eq!(value["product_dependency"]["available"], false);
    assert_eq!(
        value["product_dependency"]["product"],
        "Jira Software Agile REST"
    );
    assert!(
        value["message"]
            .as_str()
            .unwrap()
            .contains("Jira Software is not available")
    );
    assert_eq!(
        requests[0].path,
        "/rest/agile/1.0/board?projectKeyOrId=NOAGILE"
    );
}

#[tokio::test]
async fn service_desk_helpers_return_product_unavailable_when_jsm_rest_is_missing() {
    let (base_url, requests) = jira_extension_mock_server().await;
    let client = JiraClient::new(config(
        format!("{base_url}/jsm-down"),
        JiraDeployment::ServerDataCenter,
    ))
    .unwrap();

    let value = client
        .get_service_desk_for_project("ABC".to_string())
        .await
        .unwrap();
    let requests = requests.lock().await;

    assert_eq!(value["success"], false);
    assert_eq!(value["product_dependency"]["available"], false);
    assert_eq!(
        value["product_dependency"]["product"],
        "Jira Service Management"
    );
    assert!(
        value["message"]
            .as_str()
            .unwrap()
            .contains("Jira Service Management is not available")
    );
    assert_eq!(
        requests[0].path,
        "/jsm-down/rest/servicedeskapi/servicedesk"
    );
}

#[tokio::test]
async fn development_helper_returns_product_unavailable_when_dev_status_is_missing() {
    let (base_url, requests) = jira_extension_mock_server().await;
    let client = JiraClient::new(config(
        format!("{base_url}/dev-down"),
        JiraDeployment::ServerDataCenter,
    ))
    .unwrap();

    let value = client
        .get_issue_development_info("10001".to_string(), None, None)
        .await
        .unwrap();
    let requests = requests.lock().await;

    assert_eq!(value["success"], false);
    assert_eq!(value["product_dependency"]["available"], false);
    assert_eq!(
        value["product_dependency"]["product"],
        "Jira development/dev-status"
    );
    assert!(
        value["message"]
            .as_str()
            .unwrap()
            .contains("Jira development status is not available")
    );
    assert_eq!(
        requests[0].path,
        "/dev-down/rest/dev-status/1.0/issue/detail?issueId=10001"
    );
}

#[tokio::test]
async fn forms_helpers_use_cloud_id_paths_and_config_auth_without_override() {
    let (base_url, requests) = jira_extension_mock_server().await;
    let client = JiraClient::new(config(base_url, JiraDeployment::ServerDataCenter)).unwrap();

    let forms = client
        .get_issue_forms("ABC-1".to_string(), Some("cloud-123"))
        .await
        .unwrap();
    let details = client
        .get_issue_form("ABC-1".to_string(), "form-1".to_string(), Some("cloud-123"))
        .await
        .unwrap();
    let updated = client
        .update_issue_form_answers(
            "ABC-1".to_string(),
            "form-1".to_string(),
            vec![
                json!({"questionId": "q1", "type": "TEXT", "value": "Updated"}),
                json!({"questionId": "q2", "type": "SELECT", "value": "Product A"}),
                json!({"question_id": "q3", "type": "MULTI_USER", "value": ["abc"]}),
            ],
            Some("cloud-123"),
        )
        .await
        .unwrap();
    let requests = requests.lock().await;
    let expected_header = format!("Bearer {}", "test-pat-value");

    assert_eq!(forms["forms"][0]["id"], "form-1");
    assert_eq!(details["answers"]["q1"]["text"], "Existing");
    assert_eq!(updated["updated"], true);
    assert_eq!(
        requests[0].path,
        "/jira/forms/cloud/cloud-123/issue/ABC-1/form"
    );
    assert_eq!(
        requests[1].path,
        "/jira/forms/cloud/cloud-123/issue/ABC-1/form/form-1"
    );
    assert_eq!(requests[2].method, Method::PUT);
    assert_eq!(
        requests[2].path,
        "/jira/forms/cloud/cloud-123/issue/ABC-1/form/form-1"
    );
    for request in requests.iter() {
        assert_eq!(
            request.authorization.as_deref(),
            Some(expected_header.as_str())
        );
    }
    assert_eq!(requests[2].body["answers"]["q1"]["text"], "Updated");
    assert_eq!(
        requests[2].body["answers"]["q2"]["choices"],
        json!(["Product A"])
    );
    assert_eq!(requests[2].body["answers"]["q3"]["users"], json!(["abc"]));
}

#[tokio::test]
async fn forms_helpers_return_product_unavailable_when_cloud_id_missing_without_http() {
    let (base_url, requests) = jira_extension_mock_server().await;
    let client = JiraClient::new(config(base_url, JiraDeployment::ServerDataCenter)).unwrap();

    let missing = client
        .get_issue_forms("ABC-1".to_string(), None)
        .await
        .unwrap();
    let blank = client
        .get_issue_form("ABC-1".to_string(), "form-1".to_string(), Some(" "))
        .await
        .unwrap();
    let requests = requests.lock().await;

    assert_eq!(missing["success"], false);
    assert_eq!(missing["product_dependency"]["available"], false);
    assert_eq!(
        missing["product_dependency"]["product"],
        "Jira Forms/ProForma Cloud ID"
    );
    assert_eq!(blank["product_dependency"]["available"], false);
    assert!(requests.is_empty());
}

#[tokio::test]
async fn forms_helpers_return_product_unavailable_when_forms_api_is_missing() {
    let (base_url, requests) = jira_extension_mock_server().await;
    let client = JiraClient::new(config(base_url, JiraDeployment::ServerDataCenter)).unwrap();

    let value = client
        .get_issue_forms("ABC-1".to_string(), Some("forms-down"))
        .await
        .unwrap();
    let requests = requests.lock().await;

    assert_eq!(value["success"], false);
    assert_eq!(value["product_dependency"]["available"], false);
    assert_eq!(
        value["product_dependency"]["product"],
        "Jira Forms/ProForma"
    );
    assert!(
        value["message"]
            .as_str()
            .unwrap()
            .contains("Jira Forms is not available")
    );
    assert_eq!(
        requests[0].path,
        "/jira/forms/cloud/forms-down/issue/ABC-1/form"
    );
}

#[tokio::test]
async fn jira_attachment_helpers_use_bounded_content_fetch() {
    let (base_url, requests) = jira_extension_mock_server().await;
    let client =
        JiraClient::new(config(base_url.clone(), JiraDeployment::ServerDataCenter)).unwrap();

    let attachments = client
        .get_issue_attachments("ABC-1".to_string())
        .await
        .unwrap();
    let content = client
        .fetch_attachment_content("/secure/attachment/1/file.png", 20)
        .await
        .unwrap();
    let oversized = client
        .fetch_attachment_content("/secure/attachment/1/file.png", 2)
        .await
        .unwrap_err()
        .to_string();
    let absolute = client
        .fetch_attachment_content(
            &format!("{base_url}/secure/attachment/1/file.png?token=secret"),
            20,
        )
        .await
        .unwrap();
    let blocked_external = client
        .fetch_attachment_content("https://evil.example/attachment.png?token=secret", 20)
        .await
        .unwrap_err()
        .to_string();
    let requests = requests.lock().await;

    assert_eq!(attachments.len(), 2);
    assert!(attachments[0].is_image());
    assert_eq!(content.content_type.as_deref(), Some("image/png"));
    assert_eq!(content.bytes, b"image-bytes");
    assert_eq!(absolute.bytes, b"image-bytes");
    assert!(oversized.contains("exceeds configured limit"));
    assert!(blocked_external.contains("absolute URL must use the configured upstream base origin"));
    assert!(!blocked_external.contains("token=secret"));
    assert_eq!(
        requests[0].path,
        "/rest/api/2/issue/ABC-1?fields=attachment"
    );
    assert_eq!(requests[1].path, "/secure/attachment/1/file.png");
    assert_eq!(requests[2].path, "/secure/attachment/1/file.png");
    assert_eq!(
        requests[3].path,
        "/secure/attachment/1/file.png?token=secret"
    );
}

#[tokio::test]
async fn safe_attachment_helper_filters_images_and_redacts_content_errors() {
    let (base_url, requests) = jira_extension_mock_server().await;
    let client = JiraClient::new(config(base_url, JiraDeployment::ServerDataCenter)).unwrap();

    let with_content = client
        .get_safe_issue_attachments(
            "ABC-1".to_string(),
            AttachmentFetchOptions {
                attachment_ids: Some(vec!["1".to_string(), "2".to_string()]),
                include_content: true,
                images_only: false,
                max_bytes: 20,
            },
        )
        .await
        .unwrap();
    let images_only = client
        .get_safe_issue_attachments(
            "ABC-1".to_string(),
            AttachmentFetchOptions {
                images_only: true,
                ..Default::default()
            },
        )
        .await
        .unwrap();
    let oversized = client
        .get_safe_issue_attachments(
            "ABC-1".to_string(),
            AttachmentFetchOptions {
                attachment_ids: Some(vec!["1".to_string()]),
                include_content: true,
                max_bytes: 2,
                ..Default::default()
            },
        )
        .await
        .unwrap();
    let requests = requests.lock().await;

    assert_eq!(with_content["count"], 2);
    assert_eq!(with_content["attachments"][0]["filename"], "file.png");
    assert_eq!(with_content["attachments"][0]["has_content_url"], true);
    assert!(with_content["attachments"][0].get("thumbnail").is_none());
    assert_eq!(
        with_content["attachments"][0]["content"],
        json!({
            "encoding": "base64",
            "content_type": "image/png",
            "size": 11,
            "data": "aW1hZ2UtYnl0ZXM="
        })
    );
    let error = with_content["attachments"][1]["content_error"]["message"]
        .as_str()
        .unwrap();
    assert!(error.contains("/secure/attachment/2/notes.txt?"));
    assert!(error.contains("<redacted>"));
    assert!(!error.contains("token=secret"));
    assert!(error.contains("client=abc"));
    assert_eq!(images_only["count"], 1);
    assert_eq!(images_only["attachments"][0]["filename"], "file.png");
    assert_eq!(oversized["count"], 1);
    assert!(
        oversized["attachments"][0]["content_error"]["message"]
            .as_str()
            .unwrap()
            .contains("exceeds configured limit")
    );
    assert_eq!(
        requests[1].path,
        "/secure/attachment/1/file.png?token=secret"
    );
    assert_eq!(
        requests[2].path,
        "/secure/attachment/2/notes.txt?token=secret&client=abc"
    );
}

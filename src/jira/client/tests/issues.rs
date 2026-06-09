use super::support::*;
use super::*;

#[tokio::test]
async fn get_issue_uses_v2_endpoint_and_auth_header() {
    let (base_url, requests) =
        mock_server(json!({"id": "10001", "key": "ABC-1", "fields": {"summary": "Demo"}})).await;
    let client = JiraClient::new(config(base_url, JiraDeployment::ServerDataCenter)).unwrap();
    let value = client
        .get_issue(GetIssueRequest {
            issue_key: "ABC-1".to_string(),
            fields: Some(vec!["summary".to_string()]),
            ..Default::default()
        })
        .await
        .unwrap();
    let requests = requests.lock().await;

    assert_eq!(value["key"], "ABC-1");
    assert_eq!(requests[0].method, Method::GET);
    assert!(requests[0].path.starts_with("/rest/api/2/issue/ABC-1"));
    let expected_header = format!("Bearer {}", "test-pat-value");
    assert_eq!(
        requests[0].authorization.as_deref(),
        Some(expected_header.as_str())
    );
}

#[tokio::test]
async fn get_issue_missing_fields_payload_is_simplified() {
    let (base_url, _requests) = mock_server(json!({"key": "ABC-1"})).await;
    let client = JiraClient::new(config(base_url, JiraDeployment::ServerDataCenter)).unwrap();
    let value = client
        .get_issue(GetIssueRequest {
            issue_key: "ABC-1".to_string(),
            ..Default::default()
        })
        .await
        .unwrap();

    assert_eq!(value["key"], "ABC-1");
    assert!(value["fields"].is_null());
    assert!(value["summary"].is_null());
}

#[tokio::test]
async fn project_filter_rejects_issue_without_http_request() {
    let (base_url, requests) =
        mock_server(json!({"id": "10001", "key": "XYZ-1", "fields": {}})).await;
    let mut config = config(base_url, JiraDeployment::ServerDataCenter);
    config.projects_filter = BTreeSet::from(["ABC".to_string()]);
    let client = JiraClient::new(config).unwrap();
    let error = client
        .get_issue(GetIssueRequest {
            issue_key: "XYZ-1".to_string(),
            ..Default::default()
        })
        .await
        .unwrap_err()
        .to_string();
    let requests = requests.lock().await;

    assert!(error.contains("outside the configured Jira project filter"));
    assert!(requests.is_empty());
}

#[tokio::test]
async fn jira_extension_issue_helpers_use_expected_endpoints() {
    let (base_url, requests) = jira_extension_mock_server().await;
    let client = JiraClient::new(config(base_url, JiraDeployment::ServerDataCenter)).unwrap();

    let created = client
        .create_issue(json!({
            "project": {"key": "ABC"},
            "summary": "Demo",
            "issuetype": {"name": "Task"}
        }))
        .await
        .unwrap();
    client
        .batch_create_issues(
            vec![json!({
                "fields": {
                    "project": {"key": "ABC"},
                    "summary": "Batch",
                    "issuetype": {"name": "Task"}
                }
            })],
            false,
        )
        .await
        .unwrap();
    client
        .update_issue(
            "ABC-1".to_string(),
            json!({"summary": "Updated"}),
            Some(json!({"priority": {"name": "High"}})),
            Some(false),
        )
        .await
        .unwrap();
    client
        .delete_issue("ABC-1".to_string(), true)
        .await
        .unwrap();
    let requests = requests.lock().await;

    assert_eq!(created["data"]["key"], "ABC-1");
    assert_eq!(requests[0].method, Method::POST);
    assert_eq!(requests[0].path, "/rest/api/2/issue");
    assert_eq!(requests[0].body["fields"]["summary"], "Demo");
    assert_eq!(requests[1].path, "/rest/api/2/issue/bulk");
    assert_eq!(
        requests[1].body["issueUpdates"][0]["fields"]["summary"],
        "Batch"
    );
    assert_eq!(requests[2].method, Method::PUT);
    assert_eq!(
        requests[2].path,
        "/rest/api/2/issue/ABC-1?notifyUsers=false"
    );
    assert_eq!(requests[2].body["fields"]["priority"]["name"], "High");
    assert_eq!(requests[3].method, Method::DELETE);
    assert_eq!(
        requests[3].path,
        "/rest/api/2/issue/ABC-1?deleteSubtasks=true"
    );
}

#[tokio::test]
async fn cloud_issue_create_and_update_use_v3_for_adf_fields() {
    let (base_url, requests) =
        mock_server(json!({"id": "10001", "key": "ABC-1", "fields": {}})).await;
    let client = JiraClient::new(config(base_url, JiraDeployment::Cloud)).unwrap();
    let description = json!({
        "type": "doc",
        "version": 1,
        "content": [{
            "type": "paragraph",
            "content": [{"type": "text", "text": "Cloud description"}]
        }]
    });

    client
        .create_issue(json!({
            "project": {"key": "ABC"},
            "summary": "Demo",
            "issuetype": {"name": "Task"},
            "description": description.clone()
        }))
        .await
        .unwrap();
    client
        .update_issue(
            "ABC-1".to_string(),
            json!({"description": description}),
            None,
            None,
        )
        .await
        .unwrap();
    let requests = requests.lock().await;

    assert_eq!(requests[0].method, Method::POST);
    assert_eq!(requests[0].path, "/rest/api/3/issue");
    assert_eq!(requests[0].body["fields"]["description"]["type"], "doc");
    assert_eq!(requests[1].method, Method::PUT);
    assert_eq!(requests[1].path, "/rest/api/3/issue/ABC-1");
    assert_eq!(requests[1].body["fields"]["description"]["type"], "doc");
}

#[tokio::test]
async fn jira_extension_changelog_and_product_dependency_helpers_are_safe() {
    let (cloud_url, cloud_requests) = jira_extension_mock_server().await;
    let cloud = JiraClient::new(config(cloud_url, JiraDeployment::Cloud)).unwrap();
    let changelog = cloud
        .batch_get_changelogs(
            vec!["ABC-1".to_string()],
            Some(vec!["status".to_string()]),
            Some(50),
        )
        .await
        .unwrap();
    let cloud_requests = cloud_requests.lock().await;

    assert_eq!(changelog["issueChangeLogs"][0]["issueId"], "10001");
    assert_eq!(cloud_requests[0].path, "/rest/api/3/changelog/bulkfetch");
    assert_eq!(cloud_requests[0].body["issueIdsOrKeys"][0], "ABC-1");
    assert_eq!(cloud_requests[0].body["fieldIds"][0], "status");
    assert_eq!(cloud_requests[0].body["maxResults"], 50);

    let (server_url, server_requests) = jira_extension_mock_server().await;
    let server = JiraClient::new(config(server_url, JiraDeployment::ServerDataCenter)).unwrap();
    let unsupported = server
        .batch_get_changelogs(vec!["ABC-1".to_string()], None, None)
        .await
        .unwrap();
    let forms = server
        .get_issue_proforma_forms("ABC-1".to_string(), None)
        .await
        .unwrap();
    let sla = server
        .get_issue_sla("ABC-1".to_string(), None, false)
        .await
        .unwrap();
    let server_requests = server_requests.lock().await;

    assert_eq!(unsupported["success"], false);
    assert_eq!(unsupported["product_dependency"]["available"], false);
    assert_eq!(forms["product_dependency"]["available"], false);
    assert_eq!(sla["success"], true);
    assert_eq!(sla["product_dependency"]["available"], true);
    assert_eq!(sla["metrics"][0]["field_id"], "customfield_sla");
    assert_eq!(
        server_requests[0].path,
        "/rest/api/2/issue/ABC-1?fields=*all"
    );
}

#[tokio::test]
async fn jira_common_extension_helpers_use_expected_endpoints() {
    let (base_url, requests) = jira_extension_mock_server().await;
    let client = JiraClient::new(config(base_url, JiraDeployment::ServerDataCenter)).unwrap();

    client.get_all_projects(false).await.unwrap();
    client
        .get_project_versions("ABC".to_string())
        .await
        .unwrap();
    client
        .get_project_components("ABC".to_string())
        .await
        .unwrap();
    client
        .create_version(json!({"project": "ABC", "name": "v1"}))
        .await
        .unwrap();
    client.get_user_profile("ada".to_string()).await.unwrap();
    client
        .get_issue_watchers("ABC-1".to_string())
        .await
        .unwrap();
    client
        .add_watcher("ABC-1".to_string(), "ada".to_string())
        .await
        .unwrap();
    client
        .remove_watcher("ABC-1".to_string(), "ada".to_string())
        .await
        .unwrap();
    client
        .get_worklog("ABC-1".to_string(), Some(0), Some(10))
        .await
        .unwrap();
    client
        .add_worklog(
            "ABC-1".to_string(),
            json!({"timeSpent": "1h"}),
            vec![("adjustEstimate".to_string(), "auto".to_string())],
        )
        .await
        .unwrap();
    client.get_link_types().await.unwrap();
    client
        .link_to_epic("ABC-1".to_string(), "ABC-EPIC".to_string())
        .await
        .unwrap();
    client
        .create_issue_link(json!({
            "type": {"name": "Blocks"},
            "inwardIssue": {"key": "ABC-1"},
            "outwardIssue": {"key": "ABC-2"}
        }))
        .await
        .unwrap();
    client
        .create_remote_issue_link(
            "ABC-1".to_string(),
            json!({"object": {"url": "https://example.invalid", "title": "Example"}}),
        )
        .await
        .unwrap();
    client.remove_issue_link("200".to_string()).await.unwrap();
    let requests = requests.lock().await;

    assert_eq!(
        requests[0].path,
        "/rest/api/2/project?includeArchived=false"
    );
    assert_eq!(requests[1].path, "/rest/api/2/project/ABC/versions");
    assert_eq!(requests[2].path, "/rest/api/2/project/ABC/components");
    assert_eq!(requests[3].path, "/rest/api/2/version");
    assert_eq!(requests[4].path, "/rest/api/2/user?username=ada");
    assert_eq!(requests[5].path, "/rest/api/2/issue/ABC-1/watchers");
    assert_eq!(requests[6].method, Method::POST);
    assert_eq!(requests[7].method, Method::DELETE);
    assert_eq!(
        requests[8].path,
        "/rest/api/2/issue/ABC-1/worklog?startAt=0&maxResults=10"
    );
    assert_eq!(requests[9].body["timeSpent"], "1h");
    assert_eq!(requests[10].path, "/rest/api/2/issueLinkType");
    assert_eq!(requests[11].path, "/rest/api/2/issue/ABC-1");
    assert_eq!(requests[12].path, "/rest/api/2/issueLink");
    assert_eq!(requests[13].path, "/rest/api/2/issue/ABC-1/remotelink");
    assert_eq!(requests[14].path, "/rest/api/2/issueLink/200");
}

#[tokio::test]
async fn cloud_remove_watcher_uses_account_id_query_parameter() {
    let (base_url, requests) = jira_extension_mock_server().await;
    let client = JiraClient::new(config(base_url, JiraDeployment::Cloud)).unwrap();

    client
        .remove_watcher("ABC-1".to_string(), "account-1".to_string())
        .await
        .unwrap();
    let requests = requests.lock().await;

    assert_eq!(requests[0].method, Method::DELETE);
    assert_eq!(
        requests[0].path,
        "/rest/api/2/issue/ABC-1/watchers?accountId=account-1"
    );
}

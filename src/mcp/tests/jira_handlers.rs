use super::support::*;
use super::*;

#[tokio::test]
async fn jira_get_issue_handler_returns_structured_content_from_mock_rest() {
    let (base_url, requests) = mock_jira_server().await;
    let server = server_with_config(RuntimeConfig {
        jira: Some(jira_config_with_base_url(base_url)),
        ..runtime_config()
    });
    let result = server
        .jira_get_issue(Parameters(tools::JiraGetIssueArgs {
            issue_key: "ABC-1".to_string(),
            fields: Some(json!(["summary"])),
            expand: None,
            comment_limit: None,
            properties: None,
            update_history: None,
        }))
        .await
        .unwrap();
    let requests = requests.lock().await;

    assert_eq!(
        result.structured_content.as_ref().unwrap()["key"],
        json!("ABC-1")
    );
    assert_eq!(requests.len(), 1);
    assert_eq!(requests[0].method, Method::GET);
    assert!(requests[0].path.starts_with("/rest/api/2/issue/ABC-1"));
}

#[tokio::test]
async fn jira_create_issue_handler_posts_expected_payload_to_mock_rest() {
    let (base_url, requests) = mock_jira_server().await;
    let server = server_with_config(RuntimeConfig {
        jira: Some(jira_config_with_base_url(base_url)),
        ..runtime_config()
    });
    let result = server
        .jira_create_issue(Parameters(tools::JiraCreateIssueArgs {
            project_key: "ABC".to_string(),
            summary: "Created issue".to_string(),
            issue_type: "Task".to_string(),
            assignee: None,
            description: Some("Plain description".to_string()),
            components: Some(json!("Frontend, API")),
            additional_fields: Some(json!({"priority": {"name": "High"}})),
        }))
        .await
        .unwrap();
    let requests = requests.lock().await;

    assert_eq!(
        result.structured_content.as_ref().unwrap()["success"],
        json!(true)
    );
    assert_eq!(
        result.structured_content.as_ref().unwrap()["data"]["key"],
        json!("ABC-2")
    );
    assert_eq!(requests.len(), 1);
    assert_eq!(requests[0].method, Method::POST);
    assert_eq!(requests[0].path, "/rest/api/2/issue");
    assert_eq!(requests[0].body["fields"]["project"]["key"], json!("ABC"));
    assert_eq!(
        requests[0].body["fields"]["summary"],
        json!("Created issue")
    );
    assert_eq!(
        requests[0].body["fields"]["issuetype"]["name"],
        json!("Task")
    );
    assert_eq!(
        requests[0].body["fields"]["description"],
        json!("Plain description")
    );
    assert_eq!(
        requests[0].body["fields"]["components"],
        json!([{"name": "Frontend"}, {"name": "API"}])
    );
    assert_eq!(
        requests[0].body["fields"]["priority"]["name"],
        json!("High")
    );
}

#[tokio::test]
async fn jira_create_issue_handler_rejects_invalid_additional_fields_before_http() {
    let (base_url, requests) = mock_jira_server().await;
    let server = server_with_config(RuntimeConfig {
        jira: Some(jira_config_with_base_url(base_url)),
        ..runtime_config()
    });
    let error = server
        .jira_create_issue(Parameters(tools::JiraCreateIssueArgs {
            project_key: "ABC".to_string(),
            summary: "Created issue".to_string(),
            issue_type: "Task".to_string(),
            assignee: None,
            description: None,
            components: None,
            additional_fields: Some(json!("[]")),
        }))
        .await
        .unwrap_err();
    let requests = requests.lock().await;

    assert!(
        error
            .message
            .contains("additional_fields must be a JSON object")
    );
    assert!(requests.is_empty());
}

#[tokio::test]
async fn jira_batch_create_issues_handler_posts_bulk_payload_to_mock_rest() {
    let (base_url, requests) = mock_jira_server().await;
    let server = server_with_config(RuntimeConfig {
        jira: Some(jira_config_with_base_url(base_url)),
        ..runtime_config()
    });
    let result = server
        .jira_batch_create_issues(Parameters(tools::JiraBatchCreateIssuesArgs {
            issues: json!([
                {
                    "project_key": "ABC",
                    "summary": "Batch one",
                    "issue_type": "Task",
                    "description": "First description",
                    "components": ["Frontend"]
                },
                {
                    "project_key": "ABC",
                    "summary": "Batch two",
                    "issue_type": "Bug",
                    "priority": {"name": "High"}
                }
            ]),
            validate_only: Some(false),
        }))
        .await
        .unwrap();
    let requests = requests.lock().await;

    assert_eq!(
        result.structured_content.as_ref().unwrap()["success"],
        json!(true)
    );
    assert_eq!(
        result.structured_content.as_ref().unwrap()["data"]["issues"][0]["key"],
        json!("ABC-3")
    );
    assert_eq!(
        result.structured_content.as_ref().unwrap()["data"]["errors"][0]["failedElementNumber"],
        json!(1)
    );
    assert_eq!(requests.len(), 1);
    assert_eq!(requests[0].method, Method::POST);
    assert_eq!(requests[0].path, "/rest/api/2/issue/bulk");
    assert_eq!(requests[0].body["validateOnly"], json!(false));
    assert_eq!(
        requests[0].body["issueUpdates"][0]["fields"]["summary"],
        json!("Batch one")
    );
    assert_eq!(
        requests[0].body["issueUpdates"][0]["fields"]["components"],
        json!([{"name": "Frontend"}])
    );
    assert_eq!(
        requests[0].body["issueUpdates"][1]["fields"]["priority"]["name"],
        json!("High")
    );
}

#[tokio::test]
async fn jira_batch_create_issues_handler_rejects_invalid_issue_before_http() {
    let (base_url, requests) = mock_jira_server().await;
    let server = server_with_config(RuntimeConfig {
        jira: Some(jira_config_with_base_url(base_url)),
        ..runtime_config()
    });
    let error = server
        .jira_batch_create_issues(Parameters(tools::JiraBatchCreateIssuesArgs {
            issues: json!([{
                "project_key": "ABC",
                "issue_type": "Task"
            }]),
            validate_only: Some(false),
        }))
        .await
        .unwrap_err();
    let requests = requests.lock().await;

    assert!(error.message.contains("summary is required"));
    assert!(requests.is_empty());
}

#[tokio::test]
async fn jira_batch_get_changelogs_handler_posts_cloud_payload_to_mock_rest() {
    let (base_url, requests) = mock_jira_server().await;
    let mut jira = jira_config_with_base_url(base_url);
    jira.deployment = JiraDeployment::Cloud;
    let server = server_with_config(RuntimeConfig {
        jira: Some(jira),
        ..runtime_config()
    });
    let result = server
        .jira_batch_get_changelogs(Parameters(tools::JiraBatchGetChangelogsArgs {
            issue_ids_or_keys: json!(["ABC-1", "ABC-2"]),
            fields: Some(json!("status,assignee")),
            limit: Some(25),
        }))
        .await
        .unwrap();
    let requests = requests.lock().await;

    assert_eq!(
        result.structured_content.as_ref().unwrap()["issueChangeLogs"][0]["issueId"],
        json!("10001")
    );
    assert_eq!(
        result.structured_content.as_ref().unwrap()["nextPageToken"],
        json!("next-token")
    );
    assert_eq!(requests.len(), 1);
    assert_eq!(requests[0].method, Method::POST);
    assert_eq!(requests[0].path, "/rest/api/3/changelog/bulkfetch");
    assert_eq!(
        requests[0].body["issueIdsOrKeys"],
        json!(["ABC-1", "ABC-2"])
    );
    assert_eq!(requests[0].body["fieldIds"], json!(["status", "assignee"]));
    assert_eq!(requests[0].body["maxResults"], json!(25));
}

#[tokio::test]
async fn jira_batch_get_changelogs_handler_returns_safe_server_dc_unsupported_result() {
    let (base_url, requests) = mock_jira_server().await;
    let server = server_with_config(RuntimeConfig {
        jira: Some(jira_config_with_base_url(base_url)),
        ..runtime_config()
    });
    let result = server
        .jira_batch_get_changelogs(Parameters(tools::JiraBatchGetChangelogsArgs {
            issue_ids_or_keys: json!("ABC-1"),
            fields: None,
            limit: None,
        }))
        .await
        .unwrap();
    let requests = requests.lock().await;

    assert_eq!(
        result.structured_content.as_ref().unwrap()["success"],
        json!(false)
    );
    assert_eq!(
        result.structured_content.as_ref().unwrap()["product_dependency"]["available"],
        json!(false)
    );
    assert!(requests.is_empty());
}

#[tokio::test]
async fn jira_update_issue_handler_puts_expected_payload_and_handles_no_content() {
    let (base_url, requests) = mock_jira_server().await;
    let server = server_with_config(RuntimeConfig {
        jira: Some(jira_config_with_base_url(base_url)),
        ..runtime_config()
    });
    let result = server
        .jira_update_issue(Parameters(tools::JiraUpdateIssueArgs {
            issue_key: "ABC-1".to_string(),
            fields: json!({
                "summary": "Updated",
                "description": "Updated description"
            }),
            additional_fields: Some(json!({"priority": {"name": "High"}})),
            components: Some(json!("Frontend, API")),
            notify_users: Some(false),
        }))
        .await
        .unwrap();
    let requests = requests.lock().await;

    assert_eq!(
        result.structured_content.as_ref().unwrap()["success"],
        json!(true)
    );
    assert_eq!(
        result.structured_content.as_ref().unwrap()["data"],
        Value::Null
    );
    assert_eq!(requests.len(), 1);
    assert_eq!(requests[0].method, Method::PUT);
    assert_eq!(
        requests[0].path,
        "/rest/api/2/issue/ABC-1?notifyUsers=false"
    );
    assert_eq!(requests[0].body["fields"]["summary"], json!("Updated"));
    assert_eq!(
        requests[0].body["fields"]["description"],
        json!("Updated description")
    );
    assert_eq!(
        requests[0].body["fields"]["priority"]["name"],
        json!("High")
    );
    assert_eq!(
        requests[0].body["fields"]["components"],
        json!([{"name": "Frontend"}, {"name": "API"}])
    );
}

#[tokio::test]
async fn jira_update_issue_handler_rejects_attachments_before_http() {
    let (base_url, requests) = mock_jira_server().await;
    let server = server_with_config(RuntimeConfig {
        jira: Some(jira_config_with_base_url(base_url)),
        ..runtime_config()
    });
    let error = server
        .jira_update_issue(Parameters(tools::JiraUpdateIssueArgs {
            issue_key: "ABC-1".to_string(),
            fields: json!({"attachments": ["/tmp/file.txt"]}),
            additional_fields: None,
            components: None,
            notify_users: None,
        }))
        .await
        .unwrap_err();
    let requests = requests.lock().await;

    assert!(
        error
            .message
            .contains("attachments is not supported by jira_update_issue")
    );
    assert!(requests.is_empty());
}

#[tokio::test]
async fn jira_delete_issue_handler_sends_delete_subtasks_query_and_handles_no_content() {
    let (base_url, requests) = mock_jira_server().await;
    let server = server_with_config(RuntimeConfig {
        jira: Some(jira_config_with_base_url(base_url)),
        ..runtime_config()
    });
    let result = server
        .jira_delete_issue(Parameters(tools::JiraDeleteIssueArgs {
            issue_key: "ABC-1".to_string(),
            delete_subtasks: Some(true),
        }))
        .await
        .unwrap();
    let requests = requests.lock().await;

    assert_eq!(
        result.structured_content.as_ref().unwrap()["success"],
        json!(true)
    );
    assert_eq!(
        result.structured_content.as_ref().unwrap()["data"],
        Value::Null
    );
    assert_eq!(requests.len(), 1);
    assert_eq!(requests[0].method, Method::DELETE);
    assert_eq!(
        requests[0].path,
        "/rest/api/2/issue/ABC-1?deleteSubtasks=true"
    );
}

#[tokio::test]
async fn jira_project_read_handlers_use_project_filter_and_tolerate_sparse_values() {
    let (base_url, requests) = mock_jira_server().await;
    let mut jira = jira_config_with_base_url(base_url);
    jira.projects_filter = BTreeSet::from(["ABC".to_string()]);
    let server = server_with_config(RuntimeConfig {
        jira: Some(jira),
        ..runtime_config()
    });

    let projects = server
        .jira_get_all_projects(Parameters(tools::JiraGetAllProjectsArgs {
            include_archived: Some(false),
        }))
        .await
        .unwrap();
    let versions = server
        .jira_get_project_versions(Parameters(tools::JiraGetProjectVersionsArgs {
            project_key: "ABC".to_string(),
        }))
        .await
        .unwrap();
    let components = server
        .jira_get_project_components(Parameters(tools::JiraGetProjectComponentsArgs {
            project_key: "ABC".to_string(),
        }))
        .await
        .unwrap();
    let forbidden_versions = server
        .jira_get_project_versions(Parameters(tools::JiraGetProjectVersionsArgs {
            project_key: "XYZ".to_string(),
        }))
        .await
        .unwrap_err();
    let forbidden_components = server
        .jira_get_project_components(Parameters(tools::JiraGetProjectComponentsArgs {
            project_key: "XYZ".to_string(),
        }))
        .await
        .unwrap_err();
    let requests = requests.lock().await;

    assert_eq!(
        projects.structured_content.as_ref().unwrap()["items"][0]["key"],
        json!("ABC")
    );
    assert_eq!(
        projects.structured_content.as_ref().unwrap()["items"]
            .as_array()
            .unwrap()
            .len(),
        1
    );
    assert_eq!(
        versions.structured_content.as_ref().unwrap()["items"][0]["name"],
        json!("v1")
    );
    assert_eq!(
        components.structured_content.as_ref().unwrap()["items"][1],
        json!({})
    );
    assert_eq!(
        requests[0].path,
        "/rest/api/2/project?includeArchived=false"
    );
    assert_eq!(requests[1].path, "/rest/api/2/project/ABC/versions");
    assert_eq!(requests[2].path, "/rest/api/2/project/ABC/components");
    assert!(
        forbidden_versions
            .message
            .contains("outside the configured Jira project filter")
    );
    assert!(
        forbidden_components
            .message
            .contains("outside the configured Jira project filter")
    );
    assert_eq!(requests.len(), 3);
}

#[tokio::test]
async fn jira_create_version_handler_posts_expected_payload_to_mock_rest() {
    let (base_url, requests) = mock_jira_server().await;
    let server = server_with_config(RuntimeConfig {
        jira: Some(jira_config_with_base_url(base_url)),
        ..runtime_config()
    });
    let result = server
        .jira_create_version(Parameters(tools::JiraCreateVersionArgs {
            project_key: "ABC".to_string(),
            name: "v1".to_string(),
            start_date: Some("2026-01-01".to_string()),
            release_date: Some("2026-02-01".to_string()),
            description: Some("First release".to_string()),
        }))
        .await
        .unwrap();
    let requests = requests.lock().await;

    assert_eq!(
        result.structured_content.as_ref().unwrap()["name"],
        json!("v1")
    );
    assert_eq!(requests.len(), 1);
    assert_eq!(requests[0].method, Method::POST);
    assert_eq!(requests[0].path, "/rest/api/2/version");
    assert_eq!(requests[0].body["project"], json!("ABC"));
    assert_eq!(requests[0].body["name"], json!("v1"));
    assert_eq!(requests[0].body["startDate"], json!("2026-01-01"));
    assert_eq!(requests[0].body["releaseDate"], json!("2026-02-01"));
    assert_eq!(requests[0].body["description"], json!("First release"));
}

#[tokio::test]
async fn jira_batch_create_versions_handler_returns_success_and_error_partitions() {
    let (base_url, requests) = mock_jira_server().await;
    let server = server_with_config(RuntimeConfig {
        jira: Some(jira_config_with_base_url(base_url)),
        ..runtime_config()
    });
    let result = server
        .jira_batch_create_versions(Parameters(tools::JiraBatchCreateVersionsArgs {
            project_key: "ABC".to_string(),
            versions: json!([
                {"name": "v2", "released": true},
                {"name": "bad"}
            ]),
        }))
        .await
        .unwrap();
    let requests = requests.lock().await;

    assert_eq!(
        result.structured_content.as_ref().unwrap()["versions"][0]["success"],
        json!(true)
    );
    assert_eq!(
        result.structured_content.as_ref().unwrap()["versions"][1]["success"],
        json!(false)
    );
    assert_eq!(requests.len(), 2);
    assert_eq!(requests[0].path, "/rest/api/2/version");
    assert_eq!(requests[0].body["project"], json!("ABC"));
    assert_eq!(requests[0].body["released"], json!(true));
    assert_eq!(requests[1].body["name"], json!("bad"));
}

#[tokio::test]
async fn jira_get_user_profile_handler_allows_absent_email_privacy_field() {
    let (base_url, requests) = mock_jira_server().await;
    let server = server_with_config(RuntimeConfig {
        jira: Some(jira_config_with_base_url(base_url)),
        ..runtime_config()
    });
    let result = server
        .jira_get_user_profile(Parameters(tools::JiraGetUserProfileArgs {
            user_identifier: "ada".to_string(),
        }))
        .await
        .unwrap();
    let requests = requests.lock().await;

    assert_eq!(
        result.structured_content.as_ref().unwrap()["displayName"],
        json!("Ada Lovelace")
    );
    assert!(
        result.structured_content.as_ref().unwrap()["emailAddress"].is_null(),
        "emailAddress should not be required in privacy-filtered responses"
    );
    assert_eq!(requests.len(), 1);
    assert_eq!(requests[0].path, "/rest/api/2/user?username=ada");
}

#[tokio::test]
async fn jira_watcher_handlers_read_add_and_remove_watchers() {
    let (base_url, requests) = mock_jira_server().await;
    let server = server_with_config(RuntimeConfig {
        jira: Some(jira_config_with_base_url(base_url)),
        ..runtime_config()
    });
    let watchers = server
        .jira_get_issue_watchers(Parameters(tools::JiraGetIssueWatchersArgs {
            issue_key: "ABC-1".to_string(),
        }))
        .await
        .unwrap();
    let add = server
        .jira_add_watcher(Parameters(tools::JiraAddWatcherArgs {
            issue_key: "ABC-1".to_string(),
            user_identifier: "ada".to_string(),
        }))
        .await
        .unwrap();
    let remove = server
        .jira_remove_watcher(Parameters(tools::JiraRemoveWatcherArgs {
            issue_key: "ABC-1".to_string(),
            user_identifier: "ada".to_string(),
        }))
        .await
        .unwrap();
    let requests = requests.lock().await;

    assert_eq!(
        watchers.structured_content.as_ref().unwrap()["watchCount"],
        json!(1)
    );
    assert_eq!(
        watchers.structured_content.as_ref().unwrap()["watchers"][0]["displayName"],
        json!("Ada Lovelace")
    );
    assert_eq!(
        add.structured_content.as_ref().unwrap()["success"],
        json!(true)
    );
    assert_eq!(
        remove.structured_content.as_ref().unwrap()["success"],
        json!(true)
    );
    assert_eq!(requests[0].path, "/rest/api/2/issue/ABC-1/watchers");
    assert_eq!(requests[1].method, Method::POST);
    assert_eq!(requests[1].body, json!("ada"));
    assert_eq!(requests[2].method, Method::DELETE);
    assert_eq!(
        requests[2].path,
        "/rest/api/2/issue/ABC-1/watchers?username=ada"
    );
}

#[tokio::test]
async fn jira_get_worklog_handler_sends_pagination_and_tolerates_missing_optional_fields() {
    let (base_url, requests) = mock_jira_server().await;
    let server = server_with_config(RuntimeConfig {
        jira: Some(jira_config_with_base_url(base_url)),
        ..runtime_config()
    });
    let result = server
        .jira_get_worklog(Parameters(tools::JiraGetWorklogArgs {
            issue_key: "ABC-1".to_string(),
            start_at: Some(0),
            limit: Some(10),
        }))
        .await
        .unwrap();
    let requests = requests.lock().await;

    assert_eq!(
        result.structured_content.as_ref().unwrap()["total"],
        json!(2)
    );
    assert_eq!(
        result.structured_content.as_ref().unwrap()["worklogs"][1]["author"],
        Value::Null
    );
    assert_eq!(
        requests[0].path,
        "/rest/api/2/issue/ABC-1/worklog?startAt=0&maxResults=10"
    );
}

#[tokio::test]
async fn jira_add_worklog_handler_posts_body_and_estimate_query() {
    let (base_url, requests) = mock_jira_server().await;
    let server = server_with_config(RuntimeConfig {
        jira: Some(jira_config_with_base_url(base_url)),
        ..runtime_config()
    });
    let result = server
        .jira_add_worklog(Parameters(tools::JiraAddWorklogArgs {
            issue_key: "ABC-1".to_string(),
            time_spent: "1h".to_string(),
            started: Some("2026-01-01T00:00:00.000+0000".to_string()),
            comment: Some("Worklog note".to_string()),
            visibility: Some(json!({"type": "group", "value": "jira-users"})),
            adjust_estimate: Some("new".to_string()),
            new_estimate: Some("2h".to_string()),
            reduce_by: None,
        }))
        .await
        .unwrap();
    let requests = requests.lock().await;

    assert_eq!(
        result.structured_content.as_ref().unwrap()["id"],
        json!("300")
    );
    assert_eq!(requests[0].method, Method::POST);
    assert_eq!(
        requests[0].path,
        "/rest/api/2/issue/ABC-1/worklog?adjustEstimate=new&newEstimate=2h"
    );
    assert_eq!(requests[0].body["timeSpent"], json!("1h"));
    assert_eq!(
        requests[0].body["started"],
        json!("2026-01-01T00:00:00.000+0000")
    );
    assert_eq!(requests[0].body["comment"], json!("Worklog note"));
    assert_eq!(
        requests[0].body["visibility"],
        json!({"type": "group", "value": "jira-users"})
    );
}

#[tokio::test]
async fn jira_add_worklog_handler_rejects_invalid_visibility_before_http() {
    let (base_url, requests) = mock_jira_server().await;
    let server = server_with_config(RuntimeConfig {
        jira: Some(jira_config_with_base_url(base_url)),
        ..runtime_config()
    });
    let error = server
        .jira_add_worklog(Parameters(tools::JiraAddWorklogArgs {
            issue_key: "ABC-1".to_string(),
            time_spent: "1h".to_string(),
            started: None,
            comment: None,
            visibility: Some(json!("public")),
            adjust_estimate: None,
            new_estimate: None,
            reduce_by: None,
        }))
        .await
        .unwrap_err();
    let requests = requests.lock().await;

    assert!(error.message.contains("visibility must be a JSON object"));
    assert!(requests.is_empty());
}

#[tokio::test]
async fn jira_link_type_and_epic_handlers_use_expected_payloads() {
    let (base_url, requests) = mock_jira_server().await;
    let server = server_with_config(RuntimeConfig {
        jira: Some(jira_config_with_base_url(base_url)),
        ..runtime_config()
    });
    let all_link_types = server
        .jira_get_link_types(Parameters(tools::JiraGetLinkTypesArgs {
            name_filter: None,
        }))
        .await
        .unwrap();
    let link_types = server
        .jira_get_link_types(Parameters(tools::JiraGetLinkTypesArgs {
            name_filter: Some("block".to_string()),
        }))
        .await
        .unwrap();
    let epic = server
        .jira_link_to_epic(Parameters(tools::JiraLinkToEpicArgs {
            issue_key: "ABC-1".to_string(),
            epic_key: "ABC-EPIC".to_string(),
        }))
        .await
        .unwrap();
    let requests = requests.lock().await;

    let all_link_types = &all_link_types.structured_content.as_ref().unwrap()["issueLinkTypes"];
    assert_eq!(all_link_types.as_array().unwrap().len(), 2);
    assert_eq!(all_link_types[1]["name"], json!("Relates"));
    assert!(all_link_types[1]["inward"].is_null());
    assert!(all_link_types[1]["outward"].is_null());
    let link_types = &link_types.structured_content.as_ref().unwrap()["issueLinkTypes"];
    assert_eq!(link_types.as_array().unwrap().len(), 1);
    assert_eq!(link_types[0]["name"], json!("Blocks"));
    assert_eq!(link_types[0]["inward"], json!("is blocked by"));
    assert_eq!(requests[0].method, Method::GET);
    assert_eq!(requests[0].path, "/rest/api/2/issueLinkType");
    assert_eq!(requests[1].method, Method::GET);
    assert_eq!(requests[1].path, "/rest/api/2/issueLinkType");
    assert_eq!(
        epic.structured_content.as_ref().unwrap()["success"],
        json!(true)
    );
    assert_eq!(
        epic.structured_content.as_ref().unwrap()["data"],
        Value::Null
    );
    assert_eq!(requests[2].method, Method::PUT);
    assert_eq!(requests[2].path, "/rest/api/2/issue/ABC-1");
    assert_eq!(
        requests[2].body["fields"]["parent"],
        json!({"key": "ABC-EPIC"})
    );
}

#[tokio::test]
async fn jira_issue_link_handlers_post_remote_and_delete_expected_payloads() {
    let (base_url, requests) = mock_jira_server().await;
    let server = server_with_config(RuntimeConfig {
        jira: Some(jira_config_with_base_url(base_url)),
        ..runtime_config()
    });
    let issue_link = server
        .jira_create_issue_link(Parameters(tools::JiraCreateIssueLinkArgs {
            link_type: "Blocks".to_string(),
            inward_issue_key: "ABC-1".to_string(),
            outward_issue_key: "ABC-2".to_string(),
            comment: Some("Linking related work".to_string()),
        }))
        .await
        .unwrap();
    let remote_link = server
        .jira_create_remote_issue_link(Parameters(tools::JiraCreateRemoteIssueLinkArgs {
            issue_key: "ABC-1".to_string(),
            url: "https://example.invalid/doc".to_string(),
            title: "Design doc".to_string(),
            global_id: Some("system=https://example.invalid&id=doc-1".to_string()),
            summary: Some("Architecture notes".to_string()),
            relationship: Some("documents".to_string()),
            icon_url: Some("https://example.invalid/icon.png".to_string()),
            status: Some(json!({"resolved": false})),
        }))
        .await
        .unwrap();
    let remove = server
        .jira_remove_issue_link(Parameters(tools::JiraRemoveIssueLinkArgs {
            link_id: "200".to_string(),
        }))
        .await
        .unwrap();
    let requests = requests.lock().await;

    assert_eq!(
        issue_link.structured_content.as_ref().unwrap()["id"],
        json!("200")
    );
    assert_eq!(requests[0].method, Method::POST);
    assert_eq!(requests[0].path, "/rest/api/2/issueLink");
    assert_eq!(requests[0].body["type"]["name"], json!("Blocks"));
    assert_eq!(requests[0].body["inwardIssue"]["key"], json!("ABC-1"));
    assert_eq!(requests[0].body["outwardIssue"]["key"], json!("ABC-2"));
    assert_eq!(
        requests[0].body["comment"]["body"],
        json!("Linking related work")
    );
    assert_eq!(
        remote_link.structured_content.as_ref().unwrap()["id"],
        json!("300")
    );
    assert_eq!(requests[1].method, Method::POST);
    assert_eq!(requests[1].path, "/rest/api/2/issue/ABC-1/remotelink");
    assert_eq!(
        requests[1].body["globalId"],
        json!("system=https://example.invalid&id=doc-1")
    );
    assert_eq!(requests[1].body["relationship"], json!("documents"));
    assert_eq!(
        requests[1].body["object"]["url"],
        json!("https://example.invalid/doc")
    );
    assert_eq!(requests[1].body["object"]["title"], json!("Design doc"));
    assert_eq!(
        requests[1].body["object"]["summary"],
        json!("Architecture notes")
    );
    assert_eq!(
        requests[1].body["object"]["icon"],
        json!({"url16x16": "https://example.invalid/icon.png", "title": "Design doc"})
    );
    assert_eq!(
        requests[1].body["object"]["status"],
        json!({"resolved": false})
    );
    assert_eq!(
        remove.structured_content.as_ref().unwrap()["success"],
        json!(true)
    );
    assert_eq!(
        remove.structured_content.as_ref().unwrap()["link_id"],
        json!("200")
    );
    assert_eq!(requests[2].method, Method::DELETE);
    assert_eq!(requests[2].path, "/rest/api/2/issueLink/200");
}

#[tokio::test]
async fn jira_create_remote_issue_link_rejects_invalid_status_before_http() {
    let (base_url, requests) = mock_jira_server().await;
    let server = server_with_config(RuntimeConfig {
        jira: Some(jira_config_with_base_url(base_url)),
        ..runtime_config()
    });
    let error = server
        .jira_create_remote_issue_link(Parameters(tools::JiraCreateRemoteIssueLinkArgs {
            issue_key: "ABC-1".to_string(),
            url: "https://example.invalid/doc".to_string(),
            title: "Design doc".to_string(),
            global_id: None,
            summary: None,
            relationship: None,
            icon_url: None,
            status: Some(json!("resolved")),
        }))
        .await
        .unwrap_err();
    let requests = requests.lock().await;

    assert!(error.message.contains("status must be a JSON object"));
    assert!(requests.is_empty());
}

#[tokio::test]
async fn jira_download_attachments_rejects_invalid_max_bytes_before_http() {
    let (base_url, requests) = mock_jira_server().await;
    let server = server_with_config(RuntimeConfig {
        jira: Some(jira_config_with_base_url(base_url)),
        ..runtime_config()
    });
    let error = server
        .jira_download_attachments(Parameters(tools::JiraDownloadAttachmentsArgs {
            issue_key: "ABC-1".to_string(),
            attachment_ids: None,
            include_content: Some(true),
            max_bytes: Some(0),
        }))
        .await
        .unwrap_err();
    let requests = requests.lock().await;

    assert!(error.message.contains("max_bytes must be positive"));
    assert!(requests.is_empty());
}

#[tokio::test]
async fn jira_get_issue_images_handler_filters_non_images_and_returns_safe_content() {
    let (base_url, requests) = mock_jira_server().await;
    let server = server_with_config(RuntimeConfig {
        jira: Some(jira_config_with_base_url(base_url)),
        ..runtime_config()
    });
    let result = server
        .jira_get_issue_images(Parameters(tools::JiraGetIssueImagesArgs {
            issue_key: "ABC-1".to_string(),
            include_content: Some(true),
            max_bytes: Some(20),
        }))
        .await
        .unwrap();
    let requests = requests.lock().await;
    let structured = result.structured_content.as_ref().unwrap();

    assert_eq!(structured["images_only"], json!(true));
    assert_eq!(structured["count"], json!(1));
    assert_eq!(structured["attachments"][0]["filename"], json!("file.png"));
    assert_eq!(structured["attachments"][0]["is_image"], json!(true));
    assert_eq!(
        structured["attachments"][0]["content"]["data"],
        json!("aW1hZ2UtYnl0ZXM=")
    );
    assert_eq!(
        requests[0].path,
        "/rest/api/2/issue/ABC-1?fields=attachment"
    );
    assert_eq!(
        requests[1].path,
        "/secure/attachment/1/file.png?token=secret"
    );
    assert_eq!(
        requests.len(),
        2,
        "non-image attachment content is not fetched"
    );
}

#[tokio::test]
async fn jira_get_issue_images_handler_returns_empty_list_when_issue_has_no_images() {
    let (base_url, requests) = mock_jira_server().await;
    let server = server_with_config(RuntimeConfig {
        jira: Some(jira_config_with_base_url(base_url)),
        ..runtime_config()
    });
    let result = server
        .jira_get_issue_images(Parameters(tools::JiraGetIssueImagesArgs {
            issue_key: "TXT-1".to_string(),
            include_content: Some(true),
            max_bytes: Some(20),
        }))
        .await
        .unwrap();
    let requests = requests.lock().await;
    let structured = result.structured_content.as_ref().unwrap();

    assert_eq!(structured["images_only"], json!(true));
    assert_eq!(structured["count"], json!(0));
    assert_eq!(structured["attachments"], json!([]));
    assert_eq!(
        requests[0].path,
        "/rest/api/2/issue/TXT-1?fields=attachment"
    );
    assert_eq!(requests.len(), 1, "no image content is fetched");
}

#[tokio::test]
async fn jira_agile_read_handlers_send_expected_queries_and_return_pages() {
    let (base_url, requests) = mock_jira_server().await;
    let server = server_with_config(RuntimeConfig {
        jira: Some(jira_config_with_base_url(base_url)),
        ..runtime_config()
    });
    let boards = server
        .jira_get_agile_boards(Parameters(tools::JiraGetAgileBoardsArgs {
            project_key: Some("ABC".to_string()),
            board_type: Some("scrum".to_string()),
            start_at: Some(0),
            limit: Some(2),
        }))
        .await
        .unwrap();
    let board_issues = server
        .jira_get_board_issues(Parameters(tools::JiraGetBoardIssuesArgs {
            board_id: 1,
            jql: Some("status = Done".to_string()),
            fields: Some(json!("summary,status")),
            start_at: Some(0),
            limit: Some(2),
        }))
        .await
        .unwrap();
    let sprints = server
        .jira_get_sprints_from_board(Parameters(tools::JiraGetSprintsFromBoardArgs {
            board_id: 1,
            state: Some(json!(["active", "future"])),
            start_at: Some(0),
            limit: Some(2),
        }))
        .await
        .unwrap();
    let sprint_issues = server
        .jira_get_sprint_issues(Parameters(tools::JiraGetSprintIssuesArgs {
            sprint_id: 2,
            fields: Some(json!(["summary", "status"])),
            start_at: Some(0),
            limit: Some(2),
        }))
        .await
        .unwrap();
    let requests = requests.lock().await;

    assert_eq!(
        boards.structured_content.as_ref().unwrap()["values"][0]["name"],
        json!("Alpha board")
    );
    assert_eq!(
        board_issues.structured_content.as_ref().unwrap()["issues"][0]["key"],
        json!("ABC-1")
    );
    assert_eq!(
        sprints.structured_content.as_ref().unwrap()["values"][0]["state"],
        json!("active")
    );
    assert_eq!(
        sprint_issues.structured_content.as_ref().unwrap()["issues"][0]["fields"]["summary"],
        json!("Sprint issue")
    );
    assert_eq!(
        requests[0].path,
        "/rest/agile/1.0/board?projectKeyOrId=ABC&type=scrum&startAt=0&maxResults=2"
    );
    assert_eq!(
        requests[1].path,
        "/rest/agile/1.0/board/1/issue?jql=status+%3D+Done&fields=summary%2Cstatus&startAt=0&maxResults=2"
    );
    assert_eq!(
        requests[2].path,
        "/rest/agile/1.0/board/1/sprint?state=active%2Cfuture&startAt=0&maxResults=2"
    );
    assert_eq!(
        requests[3].path,
        "/rest/agile/1.0/sprint/2/issue?fields=summary%2Cstatus&startAt=0&maxResults=2"
    );
}

#[tokio::test]
async fn jira_agile_boards_handler_returns_product_unavailable_when_software_missing() {
    let (base_url, requests) = mock_jira_server().await;
    let server = server_with_config(RuntimeConfig {
        jira: Some(jira_config_with_base_url(base_url)),
        ..runtime_config()
    });
    let result = server
        .jira_get_agile_boards(Parameters(tools::JiraGetAgileBoardsArgs {
            project_key: Some("NOAGILE".to_string()),
            board_type: None,
            start_at: None,
            limit: None,
        }))
        .await
        .unwrap();
    let requests = requests.lock().await;
    let structured = result.structured_content.as_ref().unwrap();

    assert_eq!(structured["success"], json!(false));
    assert_eq!(
        structured["product_dependency"]["product"],
        json!("Jira Software Agile REST")
    );
    assert_eq!(structured["product_dependency"]["available"], json!(false));
    assert_eq!(
        requests[0].path,
        "/rest/agile/1.0/board?projectKeyOrId=NOAGILE"
    );
}

#[tokio::test]
async fn jira_agile_write_handlers_send_expected_payloads_and_handle_no_content() {
    let (base_url, requests) = mock_jira_server().await;
    let server = server_with_config(RuntimeConfig {
        jira: Some(jira_config_with_base_url(base_url)),
        ..runtime_config()
    });
    let created = server
        .jira_create_sprint(Parameters(tools::JiraCreateSprintArgs {
            name: "Sprint 2".to_string(),
            origin_board_id: 1,
            start_date: Some("2026-01-01T00:00:00.000Z".to_string()),
            end_date: Some("2026-01-14T00:00:00.000Z".to_string()),
            goal: Some("Ship scope".to_string()),
        }))
        .await
        .unwrap();
    let updated = server
        .jira_update_sprint(Parameters(tools::JiraUpdateSprintArgs {
            sprint_id: 2,
            name: Some("Sprint 2 updated".to_string()),
            state: Some("active".to_string()),
            start_date: None,
            end_date: None,
            goal: Some("Updated goal".to_string()),
        }))
        .await
        .unwrap();
    let added = server
        .jira_add_issues_to_sprint(Parameters(tools::JiraAddIssuesToSprintArgs {
            sprint_id: 2,
            issue_keys: json!("ABC-1, ABC-2"),
        }))
        .await
        .unwrap();
    let requests = requests.lock().await;

    assert_eq!(
        created.structured_content.as_ref().unwrap()["name"],
        json!("Sprint 2")
    );
    assert_eq!(
        updated.structured_content.as_ref().unwrap()["state"],
        json!("active")
    );
    assert_eq!(added.structured_content.as_ref().unwrap(), &Value::Null);
    assert_eq!(requests[0].method, Method::POST);
    assert_eq!(requests[0].path, "/rest/agile/1.0/sprint");
    assert_eq!(requests[0].body["name"], json!("Sprint 2"));
    assert_eq!(requests[0].body["originBoardId"], json!(1));
    assert_eq!(
        requests[0].body["startDate"],
        json!("2026-01-01T00:00:00.000Z")
    );
    assert_eq!(
        requests[0].body["endDate"],
        json!("2026-01-14T00:00:00.000Z")
    );
    assert_eq!(requests[0].body["goal"], json!("Ship scope"));
    assert_eq!(requests[1].method, Method::PUT);
    assert_eq!(requests[1].path, "/rest/agile/1.0/sprint/2");
    assert_eq!(requests[1].body["name"], json!("Sprint 2 updated"));
    assert_eq!(requests[1].body["state"], json!("active"));
    assert_eq!(requests[1].body["goal"], json!("Updated goal"));
    assert!(requests[1].body["startDate"].is_null());
    assert_eq!(requests[2].method, Method::POST);
    assert_eq!(requests[2].path, "/rest/agile/1.0/sprint/2/issue");
    assert_eq!(requests[2].body["issues"], json!(["ABC-1", "ABC-2"]));
}

#[tokio::test]
async fn jira_update_sprint_rejects_empty_payload_before_http() {
    let (base_url, requests) = mock_jira_server().await;
    let server = server_with_config(RuntimeConfig {
        jira: Some(jira_config_with_base_url(base_url)),
        ..runtime_config()
    });
    let error = server
        .jira_update_sprint(Parameters(tools::JiraUpdateSprintArgs {
            sprint_id: 2,
            name: None,
            state: None,
            start_date: None,
            end_date: None,
            goal: None,
        }))
        .await
        .unwrap_err();
    let requests = requests.lock().await;

    assert!(
        error
            .message
            .contains("sprint update must contain at least one field")
    );
    assert!(requests.is_empty());
}

#[tokio::test]
async fn jira_service_desk_handlers_lookup_queues_and_queue_issues() {
    let (base_url, requests) = mock_jira_server().await;
    let server = server_with_config(RuntimeConfig {
        jira: Some(jira_config_with_base_url(base_url)),
        ..runtime_config()
    });
    let desk = server
        .jira_get_service_desk_for_project(Parameters(tools::JiraGetServiceDeskForProjectArgs {
            project_key: "ABC".to_string(),
        }))
        .await
        .unwrap();
    let queues = server
        .jira_get_service_desk_queues(Parameters(tools::JiraGetServiceDeskQueuesArgs {
            service_desk_id: "4".to_string(),
            start_at: Some(0),
            limit: Some(50),
        }))
        .await
        .unwrap();
    let issues = server
        .jira_get_queue_issues(Parameters(tools::JiraGetQueueIssuesArgs {
            service_desk_id: "4".to_string(),
            queue_id: "47".to_string(),
            start_at: Some(0),
            limit: Some(2),
        }))
        .await
        .unwrap();
    let requests = requests.lock().await;

    assert_eq!(
        desk.structured_content.as_ref().unwrap()["service_desk"]["id"],
        json!("4")
    );
    assert_eq!(
        queues.structured_content.as_ref().unwrap()["values"][0]["name"],
        json!("Open requests")
    );
    assert_eq!(
        issues.structured_content.as_ref().unwrap()["values"][0]["key"],
        json!("ABC-1")
    );
    assert_eq!(requests[0].path, "/rest/servicedeskapi/servicedesk");
    assert_eq!(
        requests[1].path,
        "/rest/servicedeskapi/servicedesk/4/queue?start=0&limit=50"
    );
    assert_eq!(
        requests[2].path,
        "/rest/servicedeskapi/servicedesk/4/queue/47/issue?start=0&limit=2"
    );
}

#[tokio::test]
async fn jira_service_desk_handler_returns_product_unavailable_when_jsm_missing() {
    let (base_url, requests) = mock_jira_server().await;
    let server = server_with_config(RuntimeConfig {
        jira: Some(jira_config_with_base_url(format!("{base_url}/jsm-down"))),
        ..runtime_config()
    });
    let result = server
        .jira_get_service_desk_for_project(Parameters(tools::JiraGetServiceDeskForProjectArgs {
            project_key: "ABC".to_string(),
        }))
        .await
        .unwrap();
    let requests = requests.lock().await;
    let structured = result.structured_content.as_ref().unwrap();

    assert_eq!(structured["success"], json!(false));
    assert_eq!(
        structured["product_dependency"]["product"],
        json!("Jira Service Management")
    );
    assert_eq!(structured["product_dependency"]["available"], json!(false));
    assert_eq!(
        requests[0].path,
        "/jsm-down/rest/servicedeskapi/servicedesk"
    );
}

#[tokio::test]
async fn jira_forms_read_handlers_use_cloud_id_config_and_return_forms() {
    let (base_url, requests) = mock_jira_server().await;
    let server = server_with_config(RuntimeConfig {
        jira: Some(jira_config_with_base_url(base_url)),
        atlassian_oauth_cloud_id: Some("cloud-123".to_string()),
        ..runtime_config()
    });

    let forms = server
        .jira_get_issue_proforma_forms(Parameters(tools::JiraGetIssueProformaFormsArgs {
            issue_key: "ABC-1".to_string(),
        }))
        .await
        .unwrap();
    let details = server
        .jira_get_proforma_form_details(Parameters(tools::JiraGetProformaFormDetailsArgs {
            issue_key: "ABC-1".to_string(),
            form_id: "form-1".to_string(),
        }))
        .await
        .unwrap();
    let requests = requests.lock().await;

    assert_eq!(
        forms.structured_content.as_ref().unwrap()["forms"][0]["id"],
        json!("form-1")
    );
    assert_eq!(
        details.structured_content.as_ref().unwrap()["answers"]["q1"]["text"],
        json!("Existing")
    );
    assert_eq!(
        requests[0].path,
        "/jira/forms/cloud/cloud-123/issue/ABC-1/form"
    );
    assert_eq!(
        requests[1].path,
        "/jira/forms/cloud/cloud-123/issue/ABC-1/form/form-1"
    );
}

#[tokio::test]
async fn jira_forms_read_handlers_return_product_unavailable_when_cloud_id_missing() {
    let (base_url, requests) = mock_jira_server().await;
    let server = server_with_config(RuntimeConfig {
        jira: Some(jira_config_with_base_url(base_url)),
        ..runtime_config()
    });

    let result = server
        .jira_get_issue_proforma_forms(Parameters(tools::JiraGetIssueProformaFormsArgs {
            issue_key: "ABC-1".to_string(),
        }))
        .await
        .unwrap();
    let requests = requests.lock().await;
    let structured = result.structured_content.as_ref().unwrap();

    assert_eq!(structured["success"], json!(false));
    assert_eq!(
        structured["product_dependency"]["product"],
        json!("Jira Forms/ProForma Cloud ID")
    );
    assert_eq!(structured["product_dependency"]["available"], json!(false));
    assert!(requests.is_empty());
}

#[tokio::test]
async fn jira_forms_read_handlers_return_product_unavailable_when_forms_api_missing() {
    let (base_url, requests) = mock_jira_server().await;
    let server = server_with_config(RuntimeConfig {
        jira: Some(jira_config_with_base_url(base_url)),
        atlassian_oauth_cloud_id: Some("forms-down".to_string()),
        ..runtime_config()
    });

    let result = server
        .jira_get_issue_proforma_forms(Parameters(tools::JiraGetIssueProformaFormsArgs {
            issue_key: "ABC-1".to_string(),
        }))
        .await
        .unwrap();
    let requests = requests.lock().await;
    let structured = result.structured_content.as_ref().unwrap();

    assert_eq!(structured["success"], json!(false));
    assert_eq!(
        structured["product_dependency"]["product"],
        json!("Jira Forms/ProForma")
    );
    assert_eq!(structured["product_dependency"]["available"], json!(false));
    assert_eq!(
        requests[0].path,
        "/jira/forms/cloud/forms-down/issue/ABC-1/form"
    );
}

#[tokio::test]
async fn jira_forms_write_handler_sends_answer_payload() {
    let (base_url, requests) = mock_jira_server().await;
    let server = server_with_config(RuntimeConfig {
        jira: Some(jira_config_with_base_url(base_url)),
        atlassian_oauth_cloud_id: Some("cloud-123".to_string()),
        ..runtime_config()
    });

    let result = server
        .jira_update_proforma_form_answers(Parameters(tools::JiraUpdateProformaFormAnswersArgs {
            issue_key: "ABC-1".to_string(),
            form_id: "form-1".to_string(),
            answers: json!([
                {"questionId": "q1", "type": "TEXT", "value": "Updated"},
                {"questionId": "q2", "type": "SELECT", "value": "Product A"},
                {"questionId": "q3", "type": "DATE", "value": "2026-06-04"}
            ]),
        }))
        .await
        .unwrap();
    let requests = requests.lock().await;
    let structured = result.structured_content.as_ref().unwrap();

    assert_eq!(structured["updated"], json!(true));
    assert_eq!(structured["answers"]["q2"]["choices"], json!(["Product A"]));
    assert_eq!(requests[0].method, Method::PUT);
    assert_eq!(
        requests[0].path,
        "/jira/forms/cloud/cloud-123/issue/ABC-1/form/form-1"
    );
    assert_eq!(requests[0].body["answers"]["q1"]["text"], json!("Updated"));
    assert_eq!(
        requests[0].body["answers"]["q2"]["choices"],
        json!(["Product A"])
    );
    assert_eq!(
        requests[0].body["answers"]["q3"]["date"],
        json!("2026-06-04")
    );
}

#[tokio::test]
async fn jira_forms_write_handler_returns_product_unavailable_when_cloud_id_missing() {
    let (base_url, requests) = mock_jira_server().await;
    let server = server_with_config(RuntimeConfig {
        jira: Some(jira_config_with_base_url(base_url)),
        ..runtime_config()
    });

    let result = server
        .jira_update_proforma_form_answers(Parameters(tools::JiraUpdateProformaFormAnswersArgs {
            issue_key: "ABC-1".to_string(),
            form_id: "form-1".to_string(),
            answers: json!([{"questionId": "q1", "type": "TEXT", "value": "Updated"}]),
        }))
        .await
        .unwrap();
    let requests = requests.lock().await;
    let structured = result.structured_content.as_ref().unwrap();

    assert_eq!(structured["success"], json!(false));
    assert_eq!(
        structured["product_dependency"]["product"],
        json!("Jira Forms/ProForma Cloud ID")
    );
    assert_eq!(structured["product_dependency"]["available"], json!(false));
    assert!(requests.is_empty());
}

#[tokio::test]
async fn jira_forms_write_handler_rejects_invalid_answers_before_http() {
    let (base_url, requests) = mock_jira_server().await;
    let server = server_with_config(RuntimeConfig {
        jira: Some(jira_config_with_base_url(base_url)),
        atlassian_oauth_cloud_id: Some("cloud-123".to_string()),
        ..runtime_config()
    });

    let error = server
        .jira_update_proforma_form_answers(Parameters(tools::JiraUpdateProformaFormAnswersArgs {
            issue_key: "ABC-1".to_string(),
            form_id: "form-1".to_string(),
            answers: json!("not-answers"),
        }))
        .await
        .unwrap_err();
    let requests = requests.lock().await;

    assert!(error.message.contains("answers must be a JSON array"));
    assert!(requests.is_empty());
}

#[tokio::test]
async fn jira_issue_dates_handler_returns_date_fields_and_flags() {
    let (base_url, requests) = mock_jira_server().await;
    let server = server_with_config(RuntimeConfig {
        jira: Some(jira_config_with_base_url(base_url)),
        ..runtime_config()
    });

    let result = server
        .jira_get_issue_dates(Parameters(tools::JiraGetIssueDatesArgs {
            issue_key: "ABC-1".to_string(),
            include_status_changes: Some(true),
            include_status_summary: Some(true),
        }))
        .await
        .unwrap();
    let requests = requests.lock().await;
    let structured = result.structured_content.as_ref().unwrap();

    assert_eq!(structured["issue_key"], json!("ABC-1"));
    assert_eq!(structured["include_status_changes"], json!(true));
    assert_eq!(structured["include_status_summary"], json!(true));
    assert_eq!(
        structured["issue"]["fields"]["created"],
        json!("2026-01-01T00:00:00.000+0000")
    );
    assert_eq!(
        structured["issue"]["fields"]["duedate"],
        json!("2026-01-10")
    );
    assert_eq!(structured["issue"]["status"]["name"], json!("Done"));
    assert_eq!(structured["status_changes"].as_array().unwrap().len(), 2);
    assert_eq!(
        structured["status_summary"]["current_status"]["name"],
        json!("Done")
    );
    assert_eq!(
        structured["status_summary"]["created"],
        json!("2026-01-01T00:00:00.000+0000")
    );
    assert_eq!(structured["status_summary"]["transition_count"], json!(2));
    assert_eq!(
        structured["status_summary"]["first_transition"]["to"]["name"],
        json!("In Progress")
    );
    assert_eq!(
        structured["status_summary"]["last_transition"]["to"]["name"],
        json!("Done")
    );
    assert_eq!(
        requests[0].path,
        "/rest/api/2/issue/ABC-1?fields=created%2Cupdated%2Cduedate%2Cresolutiondate%2Cstatus&expand=changelog"
    );
}

#[tokio::test]
async fn jira_issue_dates_handler_handles_missing_date_fields() {
    let (base_url, requests) = mock_jira_server().await;
    let server = server_with_config(RuntimeConfig {
        jira: Some(jira_config_with_base_url(base_url)),
        ..runtime_config()
    });

    let result = server
        .jira_get_issue_dates(Parameters(tools::JiraGetIssueDatesArgs {
            issue_key: "TXT-1".to_string(),
            include_status_changes: None,
            include_status_summary: None,
        }))
        .await
        .unwrap();
    let requests = requests.lock().await;
    let structured = result.structured_content.as_ref().unwrap();

    assert_eq!(structured["issue_key"], json!("TXT-1"));
    assert_eq!(structured["include_status_changes"], json!(false));
    assert_eq!(structured["include_status_summary"], json!(false));
    assert!(structured["issue"]["fields"]["created"].is_null());
    assert!(structured.get("status_changes").is_none());
    assert!(structured.get("status_summary").is_none());
    assert_eq!(
        requests[0].path,
        "/rest/api/2/issue/TXT-1?fields=created%2Cupdated%2Cduedate%2Cresolutiondate%2Cstatus"
    );
}

#[tokio::test]
async fn jira_issue_sla_handler_parses_mock_sla_fields_and_args() {
    let (base_url, requests) = mock_jira_server().await;
    let server = server_with_config(RuntimeConfig {
        jira: Some(jira_config_with_base_url(base_url)),
        ..runtime_config()
    });

    let result = server
        .jira_get_issue_sla(Parameters(tools::JiraGetIssueSlaArgs {
            issue_key: "ABC-1".to_string(),
            metrics: Some(json!("time_to_resolution, time_to_first_response")),
            include_raw_dates: Some(true),
        }))
        .await
        .unwrap();
    let requests = requests.lock().await;
    let structured = result.structured_content.as_ref().unwrap();

    assert_eq!(structured["issue_key"], json!("ABC-1"));
    assert_eq!(
        structured["requested_metrics"],
        json!(["time_to_resolution", "time_to_first_response"])
    );
    assert_eq!(structured["include_raw_dates"], json!(true));
    assert_eq!(structured["success"], json!(true));
    assert_eq!(structured["count"], json!(1));
    assert_eq!(
        structured["metrics"][0]["field_id"],
        json!("customfield_sla")
    );
    assert_eq!(
        structured["product_dependency"]["product"],
        json!("Jira Service Management SLA")
    );
    assert_eq!(
        structured["parsing_limitations"]["working_hours_filtering"],
        json!("not_supported")
    );
    assert_eq!(structured["product_dependency"]["available"], json!(true));
    assert_eq!(
        requests[0].path,
        "/rest/api/2/issue/ABC-1?fields=time_to_resolution%2Ctime_to_first_response"
    );
}

#[tokio::test]
async fn jira_development_handlers_return_single_and_batch_info() {
    let (base_url, requests) = mock_jira_server().await;
    let server = server_with_config(RuntimeConfig {
        jira: Some(jira_config_with_base_url(base_url)),
        ..runtime_config()
    });

    let single = server
        .jira_get_issue_development_info(Parameters(tools::JiraGetIssueDevelopmentInfoArgs {
            issue_key: "ABC-1".to_string(),
            application_type: Some("github".to_string()),
            data_type: Some("pullrequest".to_string()),
        }))
        .await
        .unwrap();
    let batch = server
        .jira_get_issues_development_info(Parameters(tools::JiraGetIssuesDevelopmentInfoArgs {
            issue_keys: json!(["10001", "10002"]),
            application_type: Some("github".to_string()),
            data_type: Some("pullrequest".to_string()),
        }))
        .await
        .unwrap();
    let requests = requests.lock().await;

    assert_eq!(
        single.structured_content.as_ref().unwrap()["detail"][0]["dataType"],
        json!("pullrequest")
    );
    assert_eq!(
        batch.structured_content.as_ref().unwrap()["issues"][0]["issue_key"],
        json!("10001")
    );
    assert_eq!(
        batch.structured_content.as_ref().unwrap()["issues"][1]["development"]["detail"][0]["applicationType"],
        json!("github")
    );
    assert_eq!(requests[0].path, "/rest/api/2/issue/ABC-1?fields=id%2Ckey");
    assert_eq!(
        requests[1].path,
        "/rest/dev-status/1.0/issue/detail?issueId=10001&applicationType=github&dataType=pullrequest"
    );
    assert_eq!(
        requests[2].path,
        "/rest/dev-status/1.0/issue/detail?issueId=10001&applicationType=github&dataType=pullrequest"
    );
    assert_eq!(
        requests[3].path,
        "/rest/dev-status/1.0/issue/detail?issueId=10002&applicationType=github&dataType=pullrequest"
    );
}

#[tokio::test]
async fn jira_development_handler_returns_product_unavailable_when_plugin_missing() {
    let (base_url, requests) = mock_jira_server().await;
    let server = server_with_config(RuntimeConfig {
        jira: Some(jira_config_with_base_url(format!("{base_url}/dev-down"))),
        ..runtime_config()
    });

    let result = server
        .jira_get_issue_development_info(Parameters(tools::JiraGetIssueDevelopmentInfoArgs {
            issue_key: "10001".to_string(),
            application_type: None,
            data_type: None,
        }))
        .await
        .unwrap();
    let requests = requests.lock().await;
    let structured = result.structured_content.as_ref().unwrap();

    assert_eq!(structured["success"], json!(false));
    assert_eq!(
        structured["product_dependency"]["product"],
        json!("Jira development/dev-status")
    );
    assert_eq!(structured["product_dependency"]["available"], json!(false));
    assert_eq!(
        requests[0].path,
        "/dev-down/rest/dev-status/1.0/issue/detail?issueId=10001"
    );
}

#[tokio::test]
async fn jira_tool_handler_rejects_invalid_json_object_input_before_http() {
    let (base_url, requests) = mock_jira_server().await;
    let server = server_with_config(RuntimeConfig {
        jira: Some(jira_config_with_base_url(base_url)),
        ..runtime_config()
    });
    let error = server
        .jira_transition_issue(Parameters(tools::JiraTransitionIssueArgs {
            issue_key: "ABC-1".to_string(),
            transition_id: "31".to_string(),
            fields: Some(json!("[]")),
            comment: None,
        }))
        .await
        .unwrap_err();
    let requests = requests.lock().await;

    assert!(error.message.contains("fields must be a JSON object"));
    assert!(requests.is_empty());
}

#[test]
fn stage_three_handler_arg_helpers_validate_json_shapes() {
    assert!(
        parse_required_object_arg(json!("[]"), "fields")
            .unwrap_err()
            .message
            .contains("fields must be a JSON object")
    );
    assert!(
        parse_required_object_list_arg(json!([{"fields": {"summary": "ok"}}, "bad"]), "issues")
            .unwrap_err()
            .message
            .contains("issues must contain only JSON objects")
    );
    assert!(
        parse_required_string_list_arg(json!({"bad": "shape"}), "issue_keys")
            .unwrap_err()
            .message
            .contains("issue_keys must be a string or array of strings")
    );
}

#[tokio::test]
async fn c4_product_dependency_responses_are_structured() {
    let (base_url, _requests) = mock_jira_server().await;
    let server = server_with_config(RuntimeConfig {
        jira: Some(jira_config_with_base_url(base_url)),
        ..runtime_config()
    });
    let agile = server
        .jira_get_agile_boards(Parameters(tools::JiraGetAgileBoardsArgs {
            project_key: Some("NOAGILE".to_string()),
            board_type: None,
            start_at: None,
            limit: None,
        }))
        .await
        .unwrap();
    let forms = server
        .jira_get_issue_proforma_forms(Parameters(tools::JiraGetIssueProformaFormsArgs {
            issue_key: "ABC-1".to_string(),
        }))
        .await
        .unwrap();
    let sla = server
        .jira_get_issue_sla(Parameters(tools::JiraGetIssueSlaArgs {
            issue_key: "ABC-1".to_string(),
            metrics: None,
            include_raw_dates: None,
        }))
        .await
        .unwrap();

    let (jsm_url, _requests) = mock_jira_server().await;
    let jsm_down = server_with_config(RuntimeConfig {
        jira: Some(jira_config_with_base_url(format!("{jsm_url}/jsm-down"))),
        ..runtime_config()
    })
    .jira_get_service_desk_for_project(Parameters(tools::JiraGetServiceDeskForProjectArgs {
        project_key: "ABC".to_string(),
    }))
    .await
    .unwrap();

    let (dev_url, _requests) = mock_jira_server().await;
    let dev_down = server_with_config(RuntimeConfig {
        jira: Some(jira_config_with_base_url(format!("{dev_url}/dev-down"))),
        ..runtime_config()
    })
    .jira_get_issue_development_info(Parameters(tools::JiraGetIssueDevelopmentInfoArgs {
        issue_key: "10001".to_string(),
        application_type: None,
        data_type: None,
    }))
    .await
    .unwrap();

    let sla_structured = sla.structured_content.as_ref().unwrap();
    assert_eq!(
        sla_structured["product_dependency"]["available"],
        json!(true),
        "sla"
    );
    assert_eq!(sla_structured["success"], json!(true), "sla");

    for (name, result) in [
        ("agile", agile),
        ("forms", forms),
        ("service_desk", jsm_down),
        ("development", dev_down),
    ] {
        let structured = result.structured_content.as_ref().unwrap();
        if structured.get("success").is_some() {
            assert_eq!(structured["success"], json!(false), "{name}");
        }
        assert_eq!(
            structured["product_dependency"]["available"],
            json!(false),
            "{name}"
        );
    }
}

#[tokio::test]
async fn jira_download_attachments_handler_returns_safe_metadata_and_content_results() {
    let (base_url, requests) = mock_jira_server().await;
    let server = server_with_config(RuntimeConfig {
        jira: Some(jira_config_with_base_url(base_url)),
        ..runtime_config()
    });
    let result = server
        .jira_download_attachments(Parameters(tools::JiraDownloadAttachmentsArgs {
            issue_key: "ABC-1".to_string(),
            attachment_ids: Some(json!(["1", "2"])),
            include_content: Some(true),
            max_bytes: Some(20),
        }))
        .await
        .unwrap();
    let requests = requests.lock().await;
    let structured = result.structured_content.as_ref().unwrap();

    assert_eq!(structured["issue_key"], json!("ABC-1"));
    assert_eq!(structured["count"], json!(2));
    assert_eq!(structured["attachments"][0]["filename"], json!("file.png"));
    assert_eq!(structured["attachments"][0]["has_content_url"], json!(true));
    assert!(structured["attachments"][0].get("thumbnail").is_none());
    assert_eq!(
        structured["attachments"][0]["content"],
        json!({
            "encoding": "base64",
            "content_type": "image/png",
            "size": 11,
            "data": "aW1hZ2UtYnl0ZXM="
        })
    );
    assert_eq!(structured["attachments"][1]["filename"], json!("notes.txt"));
    let error = structured["attachments"][1]["content_error"]["message"]
        .as_str()
        .unwrap();
    assert!(error.contains("/secure/attachment/2/notes.txt?"));
    assert!(error.contains("<redacted>"));
    assert!(!error.contains("token=secret"));
    assert!(error.contains("client=abc"));
    assert_eq!(requests[0].method, Method::GET);
    assert_eq!(
        requests[0].path,
        "/rest/api/2/issue/ABC-1?fields=attachment"
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

use super::support::*;
use super::*;

#[tokio::test]
async fn gitlab_read_handlers_return_structured_content_from_mock_rest() {
    let (base_url, requests) = mock_gitlab_server().await;
    let server = server_with_config(RuntimeConfig {
        gitlab: Some(gitlab_config_with_base_url(base_url)),
        ..runtime_config()
    });

    let user = server
        .gitlab_get_current_user(Parameters(gitlab_tools::GitlabGetCurrentUserArgs {}))
        .await
        .unwrap();
    let project = server
        .gitlab_get_project(Parameters(gitlab_tools::GitlabGetProjectArgs {
            project: "group/project".to_string(),
        }))
        .await
        .unwrap();
    let merge_requests = server
        .gitlab_list_merge_requests(Parameters(gitlab_tools::GitlabListMergeRequestsArgs {
            project: "group/project".to_string(),
            state: Some("opened".to_string()),
            author_username: Some("ada".to_string()),
            reviewer_username: None,
            source_branch: None,
            target_branch: None,
            labels: Some(vec!["bug".to_string(), "api".to_string()]),
            page: Some(2),
            per_page: Some(50),
        }))
        .await
        .unwrap();
    let merge_request = server
        .gitlab_get_merge_request(Parameters(gitlab_tools::GitlabGetMergeRequestArgs {
            project: "group/project".to_string(),
            merge_request_iid: 7,
            include_diverged_commits_count: Some(true),
            include_rebase_in_progress: Some(false),
        }))
        .await
        .unwrap();
    let commits = server
        .gitlab_list_merge_request_commits(Parameters(
            gitlab_tools::GitlabListMergeRequestCommitsArgs {
                project: "group/project".to_string(),
                merge_request_iid: 7,
                page: Some(3),
                per_page: Some(40),
            },
        ))
        .await
        .unwrap();
    let diffs = server
        .gitlab_list_merge_request_diffs(Parameters(
            gitlab_tools::GitlabListMergeRequestDiffsArgs {
                project: "group/project".to_string(),
                merge_request_iid: 7,
                max_diff_bytes: Some(5),
                page: Some(4),
                per_page: Some(50),
            },
        ))
        .await
        .unwrap();
    let pipelines = server
        .gitlab_list_merge_request_pipelines(Parameters(
            gitlab_tools::GitlabListMergeRequestPipelinesArgs {
                project: "group/project".to_string(),
                merge_request_iid: 7,
                page: Some(5),
                per_page: Some(60),
            },
        ))
        .await
        .unwrap();

    assert_eq!(
        user.structured_content.as_ref().unwrap()["username"],
        json!("gitlab-bot")
    );
    assert_eq!(
        project.structured_content.as_ref().unwrap()["path_with_namespace"],
        json!("group/project")
    );
    assert_eq!(
        merge_requests.structured_content.as_ref().unwrap()["items"][0]["iid"],
        json!(7)
    );
    assert_eq!(
        merge_request.structured_content.as_ref().unwrap()["title"],
        json!("Mock MR")
    );
    assert_eq!(
        commits.structured_content.as_ref().unwrap()["items"][0]["id"],
        json!("abc123")
    );
    let diffs = diffs.structured_content.as_ref().unwrap();
    assert_eq!(diffs["truncated"], json!(true));
    assert_eq!(diffs["diffs"][0]["diff"], json!("abcde"));
    assert_eq!(
        pipelines.structured_content.as_ref().unwrap()["items"][0]["status"],
        json!("success")
    );

    let requests = requests.lock().await;
    assert_eq!(requests.len(), 7);
    for request in requests.iter() {
        assert_eq!(request.private_token.as_deref(), Some("gitlab-token"));
        assert!(request.authorization.is_none());
    }
    assert_eq!(requests[0].method, Method::GET);
    assert_eq!(requests[0].path, "/api/v4/user");
    assert_eq!(requests[1].path, "/api/v4/projects/group%2Fproject");
    assert!(
        requests[2]
            .path
            .starts_with("/api/v4/projects/group%2Fproject/merge_requests?")
    );
    assert_eq!(
        query_value(&requests[2].path, "state").as_deref(),
        Some("opened")
    );
    assert_eq!(
        query_value(&requests[2].path, "author_username").as_deref(),
        Some("ada")
    );
    assert_eq!(
        query_value(&requests[2].path, "labels").as_deref(),
        Some("bug,api")
    );
    assert_eq!(query_value(&requests[2].path, "page").as_deref(), Some("2"));
    assert_eq!(
        query_value(&requests[2].path, "per_page").as_deref(),
        Some("50")
    );
    assert_eq!(
        requests[3].path,
        "/api/v4/projects/group%2Fproject/merge_requests/7?include_diverged_commits_count=true&include_rebase_in_progress=false"
    );
    assert_eq!(
        requests[4].path,
        "/api/v4/projects/group%2Fproject/merge_requests/7/commits?page=3&per_page=40"
    );
    assert_eq!(
        requests[5].path,
        "/api/v4/projects/group%2Fproject/merge_requests/7/diffs?page=4&per_page=50"
    );
    assert_eq!(
        requests[6].path,
        "/api/v4/projects/group%2Fproject/merge_requests/7/pipelines?page=5&per_page=60"
    );
}

#[tokio::test]
async fn gitlab_write_handlers_send_expected_payloads_to_mock_rest() {
    let (base_url, requests) = mock_gitlab_server().await;
    let server = server_with_config(RuntimeConfig {
        gitlab: Some(gitlab_config_with_base_url(base_url)),
        ..runtime_config()
    });

    let created = server
        .gitlab_create_merge_request(Parameters(gitlab_tools::GitlabCreateMergeRequestArgs {
            project: "group/project".to_string(),
            source_branch: "feature/api".to_string(),
            target_branch: "main".to_string(),
            title: "Add API".to_string(),
            description: Some("Implements the endpoint".to_string()),
            remove_source_branch: Some(true),
            squash: Some(false),
            assignee_ids: Some(vec![101]),
            reviewer_ids: Some(vec![202, 303]),
            labels: Some(vec!["api".to_string(), "backend".to_string()]),
        }))
        .await
        .unwrap();
    let updated = server
        .gitlab_update_merge_request(Parameters(gitlab_tools::GitlabUpdateMergeRequestArgs {
            project: "group/project".to_string(),
            merge_request_iid: 7,
            title: Some("Update API".to_string()),
            description: Some("".to_string()),
            state_event: Some("close".to_string()),
            labels: Some(vec![]),
            add_labels: Some(vec!["reviewed".to_string()]),
            remove_labels: Some(vec!["draft".to_string()]),
            reviewer_ids: Some(vec![]),
            assignee_ids: Some(vec![]),
            target_branch: Some("release".to_string()),
        }))
        .await
        .unwrap();
    let note = server
        .gitlab_add_merge_request_note(Parameters(gitlab_tools::GitlabAddMergeRequestNoteArgs {
            project: "group/project".to_string(),
            merge_request_iid: 7,
            body: "Looks good".to_string(),
        }))
        .await
        .unwrap();
    let reply = server
        .gitlab_reply_merge_request_discussion(Parameters(
            gitlab_tools::GitlabReplyMergeRequestDiscussionArgs {
                project: "group/project".to_string(),
                merge_request_iid: 7,
                discussion_id: "discussion 1".to_string(),
                body: "Reply body".to_string(),
            },
        ))
        .await
        .unwrap();
    let resolved = server
        .gitlab_resolve_merge_request_discussion(Parameters(
            gitlab_tools::GitlabResolveMergeRequestDiscussionArgs {
                project: "group/project".to_string(),
                merge_request_iid: 7,
                discussion_id: "discussion 1".to_string(),
                resolved: true,
            },
        ))
        .await
        .unwrap();

    assert_eq!(
        created.structured_content.as_ref().unwrap()["title"],
        json!("Add API")
    );
    assert_eq!(
        updated.structured_content.as_ref().unwrap()["state_event"],
        json!("close")
    );
    assert_eq!(
        note.structured_content.as_ref().unwrap()["body"],
        json!("Looks good")
    );
    assert_eq!(
        reply.structured_content.as_ref().unwrap()["body"],
        json!("Reply body")
    );
    assert_eq!(
        resolved.structured_content.as_ref().unwrap()["resolved"],
        json!(true)
    );

    let requests = requests.lock().await;
    assert_eq!(requests.len(), 5);
    assert_eq!(requests[0].method, Method::POST);
    assert_eq!(
        requests[0].path,
        "/api/v4/projects/group%2Fproject/merge_requests"
    );
    assert_eq!(requests[0].body["source_branch"], json!("feature/api"));
    assert_eq!(requests[0].body["target_branch"], json!("main"));
    assert_eq!(requests[0].body["title"], json!("Add API"));
    assert_eq!(requests[0].body["remove_source_branch"], json!(true));
    assert_eq!(requests[0].body["squash"], json!(false));
    assert_eq!(requests[0].body["assignee_ids"], json!([101]));
    assert_eq!(requests[0].body["reviewer_ids"], json!([202, 303]));
    assert_eq!(requests[0].body["labels"], json!("api,backend"));

    assert_eq!(requests[1].method, Method::PUT);
    assert_eq!(
        requests[1].path,
        "/api/v4/projects/group%2Fproject/merge_requests/7"
    );
    assert_eq!(requests[1].body["title"], json!("Update API"));
    assert_eq!(requests[1].body["description"], json!(""));
    assert_eq!(requests[1].body["state_event"], json!("close"));
    assert_eq!(requests[1].body["labels"], json!(""));
    assert_eq!(requests[1].body["add_labels"], json!("reviewed"));
    assert_eq!(requests[1].body["remove_labels"], json!("draft"));
    assert_eq!(requests[1].body["reviewer_ids"], json!([]));
    assert_eq!(requests[1].body["assignee_ids"], json!([]));
    assert_eq!(requests[1].body["target_branch"], json!("release"));

    assert_eq!(requests[2].method, Method::POST);
    assert_eq!(
        requests[2].path,
        "/api/v4/projects/group%2Fproject/merge_requests/7/notes"
    );
    assert_eq!(requests[2].body["body"], json!("Looks good"));
    assert_eq!(requests[3].method, Method::POST);
    assert_eq!(
        requests[3].path,
        "/api/v4/projects/group%2Fproject/merge_requests/7/discussions/discussion%201/notes"
    );
    assert_eq!(requests[3].body["body"], json!("Reply body"));
    assert_eq!(requests[4].method, Method::PUT);
    assert_eq!(
        requests[4].path,
        "/api/v4/projects/group%2Fproject/merge_requests/7/discussions/discussion%201"
    );
    assert_eq!(requests[4].body["resolved"], json!(true));
}

#[tokio::test]
async fn gitlab_approval_and_merge_handlers_use_expected_endpoints() {
    let (base_url, requests) = mock_gitlab_server().await;
    let server = server_with_config(RuntimeConfig {
        gitlab: Some(gitlab_config_with_base_url(base_url)),
        ..runtime_config()
    });

    let approval_state = server
        .gitlab_get_merge_request_approval_state(Parameters(
            gitlab_tools::GitlabMergeRequestRefArgs {
                project: "group/project".to_string(),
                merge_request_iid: 7,
            },
        ))
        .await
        .unwrap();
    let approved = server
        .gitlab_set_merge_request_approval(Parameters(
            gitlab_tools::GitlabSetMergeRequestApprovalArgs {
                project: "group/project".to_string(),
                merge_request_iid: 7,
                action: gitlab_tools::GitlabMergeRequestApprovalAction::Approve,
            },
        ))
        .await
        .unwrap();
    let unapproved = server
        .gitlab_set_merge_request_approval(Parameters(
            gitlab_tools::GitlabSetMergeRequestApprovalArgs {
                project: "group/project".to_string(),
                merge_request_iid: 7,
                action: gitlab_tools::GitlabMergeRequestApprovalAction::Unapprove,
            },
        ))
        .await
        .unwrap();
    let accepted = server
        .gitlab_accept_merge_request(Parameters(gitlab_tools::GitlabAcceptMergeRequestArgs {
            project: "group/project".to_string(),
            merge_request_iid: 7,
            sha: "abc123".to_string(),
            auto_merge: Some(true),
            squash: Some(true),
            should_remove_source_branch: Some(false),
            merge_commit_message: Some("Merge feature".to_string()),
            squash_commit_message: Some("Squash feature".to_string()),
        }))
        .await
        .unwrap();

    assert_eq!(
        approval_state.structured_content.as_ref().unwrap()["rules"][0]["approved"],
        json!(true)
    );
    assert_eq!(
        approved.structured_content.as_ref().unwrap()["approved"],
        json!(true)
    );
    assert_eq!(
        unapproved.structured_content.as_ref().unwrap()["approved"],
        json!(false)
    );
    assert_eq!(
        accepted.structured_content.as_ref().unwrap()["state"],
        json!("merged")
    );

    let requests = requests.lock().await;
    assert_eq!(requests.len(), 4);
    assert_eq!(requests[0].method, Method::GET);
    assert_eq!(
        requests[0].path,
        "/api/v4/projects/group%2Fproject/merge_requests/7/approval_state"
    );
    assert_eq!(requests[1].method, Method::POST);
    assert_eq!(
        requests[1].path,
        "/api/v4/projects/group%2Fproject/merge_requests/7/approve"
    );
    assert_eq!(requests[2].method, Method::POST);
    assert_eq!(
        requests[2].path,
        "/api/v4/projects/group%2Fproject/merge_requests/7/unapprove"
    );
    assert_eq!(requests[3].method, Method::PUT);
    assert_eq!(
        requests[3].path,
        "/api/v4/projects/group%2Fproject/merge_requests/7/merge"
    );
    assert_eq!(requests[3].body["sha"], json!("abc123"));
    assert_eq!(requests[3].body["auto_merge"], json!(true));
    assert_eq!(requests[3].body["squash"], json!(true));
    assert_eq!(
        requests[3].body["should_remove_source_branch"],
        json!(false)
    );
    assert_eq!(
        requests[3].body["merge_commit_message"],
        json!("Merge feature")
    );
    assert_eq!(
        requests[3].body["squash_commit_message"],
        json!("Squash feature")
    );
}

#[tokio::test]
async fn gitlab_handlers_reject_invalid_input_before_http() {
    let (base_url, requests) = mock_gitlab_server().await;
    let server = server_with_config(RuntimeConfig {
        gitlab: Some(gitlab_config_with_base_url(base_url.clone())),
        ..runtime_config()
    });
    let error = server
        .gitlab_accept_merge_request(Parameters(gitlab_tools::GitlabAcceptMergeRequestArgs {
            project: "group/project".to_string(),
            merge_request_iid: 7,
            sha: "  ".to_string(),
            auto_merge: None,
            squash: None,
            should_remove_source_branch: None,
            merge_commit_message: None,
            squash_commit_message: None,
        }))
        .await
        .unwrap_err();
    assert!(error.message.contains("sha must not be empty"));
    assert!(requests.lock().await.is_empty());

    let mut gitlab = gitlab_config_with_base_url(base_url);
    gitlab.projects_filter.insert("group/project".to_string());
    let server = server_with_config(RuntimeConfig {
        gitlab: Some(gitlab),
        ..runtime_config()
    });
    let error = server
        .gitlab_get_project(Parameters(gitlab_tools::GitlabGetProjectArgs {
            project: "other/project".to_string(),
        }))
        .await
        .unwrap_err();
    assert!(
        error
            .message
            .contains("GitLab project `other/project` is not allowed")
    );
    assert!(requests.lock().await.is_empty());
}

#[derive(Clone, Debug)]
struct RecordedGitlabRequest {
    method: Method,
    path: String,
    authorization: Option<String>,
    private_token: Option<String>,
    body: Value,
}

#[derive(Clone)]
struct MockGitlabState {
    requests: Arc<Mutex<Vec<RecordedGitlabRequest>>>,
}

async fn mock_gitlab_server() -> (String, Arc<Mutex<Vec<RecordedGitlabRequest>>>) {
    let requests = Arc::new(Mutex::new(Vec::new()));
    let app = Router::new()
        .fallback(any(mock_gitlab_handler))
        .with_state(MockGitlabState {
            requests: requests.clone(),
        });
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address: SocketAddr = listener.local_addr().unwrap();
    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });

    (format!("http://{address}"), requests)
}

async fn mock_gitlab_handler(
    State(state): State<MockGitlabState>,
    method: Method,
    headers: HeaderMap,
    uri: axum::http::Uri,
    body: Bytes,
) -> Response {
    let parsed_body = if body.is_empty() {
        Value::Null
    } else {
        serde_json::from_slice(&body).unwrap()
    };
    let path = uri
        .path_and_query()
        .map(ToString::to_string)
        .unwrap_or_else(|| uri.path().to_string());
    state.requests.lock().await.push(RecordedGitlabRequest {
        method: method.clone(),
        path: path.clone(),
        authorization: headers
            .get("authorization")
            .and_then(|value| value.to_str().ok())
            .map(ToString::to_string),
        private_token: headers
            .get("private-token")
            .and_then(|value| value.to_str().ok())
            .map(ToString::to_string),
        body: parsed_body.clone(),
    });

    if headers
        .get("private-token")
        .and_then(|value| value.to_str().ok())
        != Some("gitlab-token")
    {
        return (
            StatusCode::UNAUTHORIZED,
            Json(json!({"message": "missing token"})),
        )
            .into_response();
    }

    let path_only = uri.path();
    match (method, path_only) {
        (Method::GET, "/api/v4/user") => (
            StatusCode::OK,
            Json(json!({"id": 1, "username": "gitlab-bot"})),
        )
            .into_response(),
        (Method::GET, "/api/v4/projects/group%2Fproject") => (
            StatusCode::OK,
            Json(json!({
                "id": 123,
                "path_with_namespace": "group/project"
            })),
        )
            .into_response(),
        (Method::GET, "/api/v4/projects/group%2Fproject/merge_requests") => (
            StatusCode::OK,
            Json(json!([{
                "iid": 7,
                "title": "Mock MR",
                "state": "opened"
            }])),
        )
            .into_response(),
        (Method::GET, "/api/v4/projects/group%2Fproject/merge_requests/7") => (
            StatusCode::OK,
            Json(json!({
                "iid": 7,
                "title": "Mock MR",
                "include_diverged_commits_count": query_value(&path, "include_diverged_commits_count"),
                "include_rebase_in_progress": query_value(&path, "include_rebase_in_progress")
            })),
        )
            .into_response(),
        (Method::GET, "/api/v4/projects/group%2Fproject/merge_requests/7/commits") => (
            StatusCode::OK,
            Json(json!([{"id": "abc123", "short_id": "abc123"}])),
        )
            .into_response(),
        (Method::GET, "/api/v4/projects/group%2Fproject/merge_requests/7/diffs") => (
            StatusCode::OK,
            Json(json!([{
                "old_path": "src/lib.rs",
                "new_path": "src/lib.rs",
                "diff": "abcdefghi"
            }])),
        )
            .into_response(),
        (Method::GET, "/api/v4/projects/group%2Fproject/merge_requests/7/pipelines") => (
            StatusCode::OK,
            Json(json!([{"id": 55, "status": "success"}])),
        )
            .into_response(),
        (Method::POST, "/api/v4/projects/group%2Fproject/merge_requests") => (
            StatusCode::OK,
            Json(json!({
                "iid": 8,
                "title": parsed_body["title"],
                "source_branch": parsed_body["source_branch"],
                "target_branch": parsed_body["target_branch"]
            })),
        )
            .into_response(),
        (Method::PUT, "/api/v4/projects/group%2Fproject/merge_requests/7") => (
            StatusCode::OK,
            Json(json!({
                "iid": 7,
                "title": parsed_body["title"],
                "state_event": parsed_body["state_event"]
            })),
        )
            .into_response(),
        (Method::POST, "/api/v4/projects/group%2Fproject/merge_requests/7/notes") => (
            StatusCode::OK,
            Json(json!({"id": 1, "body": parsed_body["body"]})),
        )
            .into_response(),
        (
            Method::POST,
            "/api/v4/projects/group%2Fproject/merge_requests/7/discussions/discussion%201/notes",
        ) => (
            StatusCode::OK,
            Json(json!({"id": 2, "body": parsed_body["body"]})),
        )
            .into_response(),
        (
            Method::PUT,
            "/api/v4/projects/group%2Fproject/merge_requests/7/discussions/discussion%201",
        ) => (
            StatusCode::OK,
            Json(json!({"id": "discussion 1", "resolved": parsed_body["resolved"]})),
        )
            .into_response(),
        (Method::GET, "/api/v4/projects/group%2Fproject/merge_requests/7/approval_state") => (
            StatusCode::OK,
            Json(json!({"rules": [{"name": "Maintainers", "approved": true}]})),
        )
            .into_response(),
        (Method::POST, "/api/v4/projects/group%2Fproject/merge_requests/7/approve") => (
            StatusCode::OK,
            Json(json!({"iid": 7, "approved": true})),
        )
            .into_response(),
        (Method::POST, "/api/v4/projects/group%2Fproject/merge_requests/7/unapprove") => (
            StatusCode::OK,
            Json(json!({"iid": 7, "approved": false})),
        )
            .into_response(),
        (Method::PUT, "/api/v4/projects/group%2Fproject/merge_requests/7/merge") => (
            StatusCode::OK,
            Json(json!({
                "iid": 7,
                "state": "merged",
                "sha": parsed_body["sha"]
            })),
        )
            .into_response(),
        _ => (StatusCode::NOT_FOUND, Json(json!({"path": path}))).into_response(),
    }
}

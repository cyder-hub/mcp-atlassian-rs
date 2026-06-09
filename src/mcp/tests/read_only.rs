use super::support::*;
use super::*;

#[tokio::test]
async fn read_only_guard_blocks_c4_write_tools_before_http_request() {
    let (base_url, requests) = mock_jira_server().await;
    let read_only_server = server_with_config(RuntimeConfig {
        read_only: true,
        jira: Some(jira_config_with_base_url(base_url)),
        atlassian_oauth_cloud_id: Some("cloud-123".to_string()),
        ..runtime_config()
    });
    let write_tools = stage_three_c4_write_tool_names();

    assert_eq!(
        write_tools,
        vec![
            tools::JIRA_CREATE_SPRINT_TOOL_NAME,
            tools::JIRA_UPDATE_SPRINT_TOOL_NAME,
            tools::JIRA_ADD_ISSUES_TO_SPRINT_TOOL_NAME,
            tools::JIRA_UPDATE_PROFORMA_FORM_ANSWERS_TOOL_NAME,
        ]
    );
    for name in write_tools {
        let error = read_only_server
            .guard_registered_tool_call(name)
            .unwrap_err();
        assert_eq!(error.message, "tool is disabled in read-only mode");
    }
    let requests = requests.lock().await;

    assert!(requests.is_empty());
}

#[test]
fn project_read_tools_remain_visible_in_read_only_mode() {
    let read_only_projects = server_with_config(RuntimeConfig {
        read_only: true,
        jira: Some(jira_config()),
        enabled_toolsets: BTreeSet::from(["jira_projects".to_string()]),
        ..runtime_config()
    });
    let names = current_tool_names(&read_only_projects);

    assert!(names.contains(&tools::JIRA_GET_ALL_PROJECTS_TOOL_NAME.to_string()));
    assert!(names.contains(&tools::JIRA_GET_PROJECT_VERSIONS_TOOL_NAME.to_string()));
    assert!(names.contains(&tools::JIRA_GET_PROJECT_COMPONENTS_TOOL_NAME.to_string()));
    assert!(!names.contains(&tools::JIRA_CREATE_VERSION_TOOL_NAME.to_string()));
}

#[test]
fn user_profile_tool_remains_visible_in_read_only_mode() {
    let read_only_users = server_with_config(RuntimeConfig {
        read_only: true,
        jira: Some(jira_config()),
        enabled_toolsets: BTreeSet::from(["jira_users".to_string()]),
        ..runtime_config()
    });
    let names = current_tool_names(&read_only_users);

    assert!(names.contains(&tools::JIRA_GET_USER_PROFILE_TOOL_NAME.to_string()));
}

#[test]
fn watcher_read_tool_remains_visible_and_writes_hide_in_read_only_mode() {
    let read_only_watchers = server_with_config(RuntimeConfig {
        read_only: true,
        jira: Some(jira_config()),
        enabled_toolsets: BTreeSet::from(["jira_watchers".to_string()]),
        ..runtime_config()
    });
    let names = current_tool_names(&read_only_watchers);

    assert!(names.contains(&tools::JIRA_GET_ISSUE_WATCHERS_TOOL_NAME.to_string()));
    assert!(!names.contains(&tools::JIRA_ADD_WATCHER_TOOL_NAME.to_string()));
    assert!(!names.contains(&tools::JIRA_REMOVE_WATCHER_TOOL_NAME.to_string()));
}

#[test]
fn worklog_read_tool_remains_visible_in_read_only_mode() {
    let read_only_worklog = server_with_config(RuntimeConfig {
        read_only: true,
        jira: Some(jira_config()),
        enabled_toolsets: BTreeSet::from(["jira_worklog".to_string()]),
        ..runtime_config()
    });
    let names = current_tool_names(&read_only_worklog);

    assert!(names.contains(&tools::JIRA_GET_WORKLOG_TOOL_NAME.to_string()));
    assert!(!names.contains(&tools::JIRA_ADD_WORKLOG_TOOL_NAME.to_string()));
}

#[test]
fn link_read_tool_remains_visible_and_epic_write_hides_in_read_only_mode() {
    let read_only_links = server_with_config(RuntimeConfig {
        read_only: true,
        jira: Some(jira_config()),
        enabled_toolsets: BTreeSet::from(["jira_links".to_string()]),
        ..runtime_config()
    });
    let names = current_tool_names(&read_only_links);

    assert!(names.contains(&tools::JIRA_GET_LINK_TYPES_TOOL_NAME.to_string()));
    assert!(!names.contains(&tools::JIRA_LINK_TO_EPIC_TOOL_NAME.to_string()));
    assert!(!names.contains(&tools::JIRA_CREATE_ISSUE_LINK_TOOL_NAME.to_string()));
    assert!(!names.contains(&tools::JIRA_CREATE_REMOTE_ISSUE_LINK_TOOL_NAME.to_string()));
    assert!(!names.contains(&tools::JIRA_REMOVE_ISSUE_LINK_TOOL_NAME.to_string()));
}

#[test]
fn attachment_read_tools_remain_visible_in_read_only_mode() {
    let read_only_attachments = server_with_config(RuntimeConfig {
        read_only: true,
        jira: Some(jira_config()),
        enabled_toolsets: BTreeSet::from(["jira_attachments".to_string()]),
        ..runtime_config()
    });
    let names = current_tool_names(&read_only_attachments);

    assert!(names.contains(&tools::JIRA_DOWNLOAD_ATTACHMENTS_TOOL_NAME.to_string()));
    assert!(names.contains(&tools::JIRA_GET_ISSUE_IMAGES_TOOL_NAME.to_string()));
}

#[test]
fn agile_read_tools_remain_visible_in_read_only_mode() {
    let read_only_agile = server_with_config(RuntimeConfig {
        read_only: true,
        jira: Some(jira_config()),
        enabled_toolsets: BTreeSet::from(["jira_agile".to_string()]),
        ..runtime_config()
    });
    let names = current_tool_names(&read_only_agile);

    assert!(names.contains(&tools::JIRA_GET_AGILE_BOARDS_TOOL_NAME.to_string()));
    assert!(names.contains(&tools::JIRA_GET_BOARD_ISSUES_TOOL_NAME.to_string()));
    assert!(names.contains(&tools::JIRA_GET_SPRINTS_FROM_BOARD_TOOL_NAME.to_string()));
    assert!(names.contains(&tools::JIRA_GET_SPRINT_ISSUES_TOOL_NAME.to_string()));
    assert!(!names.contains(&tools::JIRA_CREATE_SPRINT_TOOL_NAME.to_string()));
    assert!(!names.contains(&tools::JIRA_UPDATE_SPRINT_TOOL_NAME.to_string()));
    assert!(!names.contains(&tools::JIRA_ADD_ISSUES_TO_SPRINT_TOOL_NAME.to_string()));
}

#[test]
fn service_desk_read_tools_remain_visible_in_read_only_mode() {
    let read_only_service_desk = server_with_config(RuntimeConfig {
        read_only: true,
        jira: Some(jira_config()),
        enabled_toolsets: BTreeSet::from(["jira_service_desk".to_string()]),
        ..runtime_config()
    });
    let names = current_tool_names(&read_only_service_desk);

    assert!(names.contains(&tools::JIRA_GET_SERVICE_DESK_FOR_PROJECT_TOOL_NAME.to_string()));
    assert!(names.contains(&tools::JIRA_GET_SERVICE_DESK_QUEUES_TOOL_NAME.to_string()));
    assert!(names.contains(&tools::JIRA_GET_QUEUE_ISSUES_TOOL_NAME.to_string()));
}

#[test]
fn forms_read_tools_remain_visible_in_read_only_mode() {
    let read_only_forms = server_with_config(RuntimeConfig {
        read_only: true,
        jira: Some(jira_config()),
        enabled_toolsets: BTreeSet::from(["jira_forms".to_string()]),
        ..runtime_config()
    });
    let names = current_tool_names(&read_only_forms);

    assert!(names.contains(&tools::JIRA_GET_ISSUE_PROFORMA_FORMS_TOOL_NAME.to_string()));
    assert!(names.contains(&tools::JIRA_GET_PROFORMA_FORM_DETAILS_TOOL_NAME.to_string()));
    assert!(!names.contains(&tools::JIRA_UPDATE_PROFORMA_FORM_ANSWERS_TOOL_NAME.to_string()));
}

#[test]
fn metrics_date_tool_remains_visible_in_read_only_mode() {
    let read_only_metrics = server_with_config(RuntimeConfig {
        read_only: true,
        jira: Some(jira_config()),
        enabled_toolsets: BTreeSet::from(["jira_metrics".to_string()]),
        ..runtime_config()
    });
    let names = current_tool_names(&read_only_metrics);

    assert!(names.contains(&tools::JIRA_GET_ISSUE_DATES_TOOL_NAME.to_string()));
    assert!(names.contains(&tools::JIRA_GET_ISSUE_SLA_TOOL_NAME.to_string()));
}

#[test]
fn development_read_tools_remain_visible_in_read_only_mode() {
    let read_only_development = server_with_config(RuntimeConfig {
        read_only: true,
        jira: Some(jira_config()),
        enabled_toolsets: BTreeSet::from(["jira_development".to_string()]),
        ..runtime_config()
    });
    let names = current_tool_names(&read_only_development);

    assert!(names.contains(&tools::JIRA_GET_ISSUE_DEVELOPMENT_INFO_TOOL_NAME.to_string()));
    assert!(names.contains(&tools::JIRA_GET_ISSUES_DEVELOPMENT_INFO_TOOL_NAME.to_string()));
}

#[test]
fn confluence_write_tools_are_blocked_by_read_only_guard() {
    let read_only = server_with_config(RuntimeConfig {
        read_only: true,
        confluence: Some(confluence_config()),
        ..runtime_config()
    });

    for name in [
        confluence_tools::CONFLUENCE_CREATE_PAGE_TOOL_NAME,
        confluence_tools::CONFLUENCE_UPDATE_PAGE_TOOL_NAME,
        confluence_tools::CONFLUENCE_DELETE_PAGE_TOOL_NAME,
        confluence_tools::CONFLUENCE_MOVE_PAGE_TOOL_NAME,
        confluence_tools::CONFLUENCE_ADD_COMMENT_TOOL_NAME,
        confluence_tools::CONFLUENCE_REPLY_TO_COMMENT_TOOL_NAME,
        confluence_tools::CONFLUENCE_ADD_LABEL_TOOL_NAME,
        confluence_tools::CONFLUENCE_UPLOAD_ATTACHMENT_TOOL_NAME,
        confluence_tools::CONFLUENCE_UPLOAD_ATTACHMENTS_TOOL_NAME,
        confluence_tools::CONFLUENCE_DELETE_ATTACHMENT_TOOL_NAME,
    ] {
        assert_eq!(
            read_only
                .guard_registered_tool_call(name)
                .unwrap_err()
                .message,
            "tool is disabled in read-only mode",
            "{name}"
        );
    }
}

#[tokio::test]
async fn read_only_guard_blocks_c3_write_tools_before_http_request() {
    let (base_url, requests) = mock_jira_server().await;
    let read_only_server = server_with_config(RuntimeConfig {
        read_only: true,
        jira: Some(jira_config_with_base_url(base_url)),
        ..runtime_config()
    });

    for name in stage_three_c3_write_tool_names() {
        let error = read_only_server
            .guard_registered_tool_call(name)
            .unwrap_err();
        assert_eq!(error.message, "tool is disabled in read-only mode");
    }
    let requests = requests.lock().await;

    assert!(requests.is_empty());
}

#[tokio::test]
async fn read_only_guard_blocks_real_jira_write_tool_before_http_request() {
    let (base_url, requests) = mock_jira_server().await;
    let read_only_server = server_with_config(RuntimeConfig {
        read_only: true,
        jira: Some(jira_config_with_base_url(base_url)),
        ..runtime_config()
    });
    let error = read_only_server
        .guard_registered_tool_call(tools::JIRA_ADD_COMMENT_TOOL_NAME)
        .unwrap_err();
    let requests = requests.lock().await;

    assert_eq!(error.message, "tool is disabled in read-only mode");
    assert!(requests.is_empty());
}

#[tokio::test]
async fn read_only_guard_blocks_jira_create_issue_before_http_request() {
    let (base_url, requests) = mock_jira_server().await;
    let read_only_server = server_with_config(RuntimeConfig {
        read_only: true,
        jira: Some(jira_config_with_base_url(base_url)),
        ..runtime_config()
    });
    let error = read_only_server
        .guard_registered_tool_call(tools::JIRA_CREATE_ISSUE_TOOL_NAME)
        .unwrap_err();
    let requests = requests.lock().await;

    assert_eq!(error.message, "tool is disabled in read-only mode");
    assert!(requests.is_empty());
}

#[tokio::test]
async fn read_only_guard_blocks_jira_batch_create_issues_before_http_request() {
    let (base_url, requests) = mock_jira_server().await;
    let read_only_server = server_with_config(RuntimeConfig {
        read_only: true,
        jira: Some(jira_config_with_base_url(base_url)),
        ..runtime_config()
    });
    let error = read_only_server
        .guard_registered_tool_call(tools::JIRA_BATCH_CREATE_ISSUES_TOOL_NAME)
        .unwrap_err();
    let requests = requests.lock().await;

    assert_eq!(error.message, "tool is disabled in read-only mode");
    assert!(requests.is_empty());
}

#[tokio::test]
async fn read_only_guard_blocks_jira_update_issue_before_http_request() {
    let (base_url, requests) = mock_jira_server().await;
    let read_only_server = server_with_config(RuntimeConfig {
        read_only: true,
        jira: Some(jira_config_with_base_url(base_url)),
        ..runtime_config()
    });
    let error = read_only_server
        .guard_registered_tool_call(tools::JIRA_UPDATE_ISSUE_TOOL_NAME)
        .unwrap_err();
    let requests = requests.lock().await;

    assert_eq!(error.message, "tool is disabled in read-only mode");
    assert!(requests.is_empty());
}

#[tokio::test]
async fn read_only_guard_blocks_jira_delete_issue_before_http_request() {
    let (base_url, requests) = mock_jira_server().await;
    let read_only_server = server_with_config(RuntimeConfig {
        read_only: true,
        jira: Some(jira_config_with_base_url(base_url)),
        ..runtime_config()
    });
    let error = read_only_server
        .guard_registered_tool_call(tools::JIRA_DELETE_ISSUE_TOOL_NAME)
        .unwrap_err();
    let requests = requests.lock().await;

    assert_eq!(error.message, "tool is disabled in read-only mode");
    assert!(requests.is_empty());
}

#[tokio::test]
async fn read_only_guard_blocks_version_write_tools_before_http_request() {
    let (base_url, requests) = mock_jira_server().await;
    let read_only_server = server_with_config(RuntimeConfig {
        read_only: true,
        jira: Some(jira_config_with_base_url(base_url)),
        ..runtime_config()
    });

    for name in [
        tools::JIRA_CREATE_VERSION_TOOL_NAME,
        tools::JIRA_BATCH_CREATE_VERSIONS_TOOL_NAME,
    ] {
        let error = read_only_server
            .guard_registered_tool_call(name)
            .unwrap_err();
        assert_eq!(error.message, "tool is disabled in read-only mode");
    }
    let requests = requests.lock().await;

    assert!(requests.is_empty());
}

#[tokio::test]
async fn read_only_guard_blocks_watcher_write_tools_before_http_request() {
    let (base_url, requests) = mock_jira_server().await;
    let read_only_server = server_with_config(RuntimeConfig {
        read_only: true,
        jira: Some(jira_config_with_base_url(base_url)),
        ..runtime_config()
    });

    for name in [
        tools::JIRA_ADD_WATCHER_TOOL_NAME,
        tools::JIRA_REMOVE_WATCHER_TOOL_NAME,
    ] {
        let error = read_only_server
            .guard_registered_tool_call(name)
            .unwrap_err();
        assert_eq!(error.message, "tool is disabled in read-only mode");
    }
    let requests = requests.lock().await;

    assert!(requests.is_empty());
}

#[tokio::test]
async fn read_only_guard_blocks_jira_add_worklog_before_http_request() {
    let (base_url, requests) = mock_jira_server().await;
    let read_only_server = server_with_config(RuntimeConfig {
        read_only: true,
        jira: Some(jira_config_with_base_url(base_url)),
        ..runtime_config()
    });
    let error = read_only_server
        .guard_registered_tool_call(tools::JIRA_ADD_WORKLOG_TOOL_NAME)
        .unwrap_err();
    let requests = requests.lock().await;

    assert_eq!(error.message, "tool is disabled in read-only mode");
    assert!(requests.is_empty());
}

#[tokio::test]
async fn read_only_guard_blocks_jira_link_to_epic_before_http_request() {
    let (base_url, requests) = mock_jira_server().await;
    let read_only_server = server_with_config(RuntimeConfig {
        read_only: true,
        jira: Some(jira_config_with_base_url(base_url)),
        ..runtime_config()
    });
    let error = read_only_server
        .guard_registered_tool_call(tools::JIRA_LINK_TO_EPIC_TOOL_NAME)
        .unwrap_err();
    let requests = requests.lock().await;

    assert_eq!(error.message, "tool is disabled in read-only mode");
    assert!(requests.is_empty());
}

#[tokio::test]
async fn read_only_guard_blocks_issue_link_write_tools_before_http_request() {
    let (base_url, requests) = mock_jira_server().await;
    let read_only_server = server_with_config(RuntimeConfig {
        read_only: true,
        jira: Some(jira_config_with_base_url(base_url)),
        ..runtime_config()
    });

    for name in [
        tools::JIRA_CREATE_ISSUE_LINK_TOOL_NAME,
        tools::JIRA_CREATE_REMOTE_ISSUE_LINK_TOOL_NAME,
        tools::JIRA_REMOVE_ISSUE_LINK_TOOL_NAME,
    ] {
        let error = read_only_server
            .guard_registered_tool_call(name)
            .unwrap_err();
        assert_eq!(error.message, "tool is disabled in read-only mode");
    }
    let requests = requests.lock().await;

    assert!(requests.is_empty());
}

#[tokio::test]
async fn read_only_guard_blocks_agile_write_tools_before_http_request() {
    let (base_url, requests) = mock_jira_server().await;
    let read_only_server = server_with_config(RuntimeConfig {
        read_only: true,
        jira: Some(jira_config_with_base_url(base_url)),
        ..runtime_config()
    });

    for name in [
        tools::JIRA_CREATE_SPRINT_TOOL_NAME,
        tools::JIRA_UPDATE_SPRINT_TOOL_NAME,
        tools::JIRA_ADD_ISSUES_TO_SPRINT_TOOL_NAME,
    ] {
        let error = read_only_server
            .guard_registered_tool_call(name)
            .unwrap_err();
        assert_eq!(error.message, "tool is disabled in read-only mode");
    }
    let requests = requests.lock().await;

    assert!(requests.is_empty());
}

#[tokio::test]
async fn read_only_guard_blocks_forms_write_tool_before_http_request() {
    let (base_url, requests) = mock_jira_server().await;
    let read_only_server = server_with_config(RuntimeConfig {
        read_only: true,
        jira: Some(jira_config_with_base_url(base_url)),
        atlassian_oauth_cloud_id: Some("cloud-123".to_string()),
        ..runtime_config()
    });
    let error = read_only_server
        .guard_registered_tool_call(tools::JIRA_UPDATE_PROFORMA_FORM_ANSWERS_TOOL_NAME)
        .unwrap_err();
    let requests = requests.lock().await;

    assert_eq!(error.message, "tool is disabled in read-only mode");
    assert!(requests.is_empty());
}

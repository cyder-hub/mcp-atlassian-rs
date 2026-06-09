use std::{collections::BTreeSet, sync::Arc};

use rmcp::model::{JsonObject, Tool};

use crate::{
    atlassian::{auth::AtlassianAuth, custom_headers::CustomHeaders, proxy::ProxyConfig},
    config::{HttpConfig, RuntimeConfig},
    confluence::{
        config::{ConfluenceConfig, ConfluenceDeployment},
        tools as confluence_tools,
    },
    context::AppContext,
    jira::config::{JiraConfig, JiraDeployment},
    jira::tools,
};

use super::*;

const SYNTHETIC_JIRA_READ: ToolMetadata = ToolMetadata {
    name: "stage1_synthetic_jira_read",
    service: ToolService::Jira,
    access: ToolAccess::Read,
    toolset: Some("jira_issues"),
    title: "Synthetic Jira read",
    description: "Test-only Jira read metadata.",
};

const SYNTHETIC_JIRA_WRITE: ToolMetadata = ToolMetadata {
    name: "stage1_synthetic_jira_write",
    service: ToolService::Jira,
    access: ToolAccess::Write,
    toolset: Some("jira_issues"),
    title: "Synthetic Jira write",
    description: "Test-only Jira write metadata.",
};

const SYNTHETIC_CONFLUENCE_READ: ToolMetadata = ToolMetadata {
    name: "stage1_synthetic_confluence_read",
    service: ToolService::Confluence,
    access: ToolAccess::Read,
    toolset: Some("confluence_pages"),
    title: "Synthetic Confluence read",
    description: "Test-only Confluence read metadata.",
};

fn metadata_for_test_tool(name: &str) -> Option<ToolMetadata> {
    match name {
        "stage1_synthetic_jira_read" => Some(SYNTHETIC_JIRA_READ),
        "stage1_synthetic_jira_write" => Some(SYNTHETIC_JIRA_WRITE),
        "stage1_synthetic_confluence_read" => Some(SYNTHETIC_CONFLUENCE_READ),
        _ => metadata_for(name),
    }
}

fn tool(name: &'static str) -> Tool {
    Tool::new(name, "", Arc::<JsonObject>::new(Default::default()))
}

fn context(config: RuntimeConfig) -> AppContext {
    AppContext::from_config(&config)
}

fn runtime_config() -> RuntimeConfig {
    RuntimeConfig {
        http: HttpConfig::default(),
        ..RuntimeConfig::default()
    }
}

fn jira_config() -> JiraConfig {
    JiraConfig {
        base_url: "https://jira.example".to_string(),
        deployment: JiraDeployment::ServerDataCenter,
        auth: AtlassianAuth::Pat {
            personal_token: "test-pat-value".to_string(),
        },
        oauth_cloud_id: None,
        ssl_verify: true,
        proxy: ProxyConfig::default(),
        custom_headers: CustomHeaders::default(),
        mtls: None,
        projects_filter: BTreeSet::new(),
        timeout_seconds: 75,
    }
}

fn confluence_config() -> ConfluenceConfig {
    ConfluenceConfig {
        base_url: "https://confluence.example".to_string(),
        deployment: ConfluenceDeployment::ServerDataCenter,
        auth: AtlassianAuth::Pat {
            personal_token: "test-pat-value".to_string(),
        },
        oauth_cloud_id: None,
        ssl_verify: true,
        proxy: ProxyConfig::default(),
        custom_headers: CustomHeaders::default(),
        mtls: None,
        spaces_filter: BTreeSet::new(),
        timeout_seconds: 75,
    }
}

fn names(tools: Vec<Tool>) -> Vec<String> {
    tools
        .into_iter()
        .map(|tool| tool.name.to_string())
        .collect()
}

fn stage_two_jira_tool_names() -> Vec<String> {
    vec![
        tools::JIRA_ADD_COMMENT_TOOL_NAME.to_string(),
        tools::JIRA_EDIT_COMMENT_TOOL_NAME.to_string(),
        tools::JIRA_GET_FIELD_OPTIONS_TOOL_NAME.to_string(),
        tools::JIRA_GET_ISSUE_TOOL_NAME.to_string(),
        tools::JIRA_GET_PROJECT_ISSUES_TOOL_NAME.to_string(),
        tools::JIRA_GET_TRANSITIONS_TOOL_NAME.to_string(),
        tools::JIRA_SEARCH_TOOL_NAME.to_string(),
        tools::JIRA_SEARCH_FIELDS_TOOL_NAME.to_string(),
        tools::JIRA_TRANSITION_ISSUE_TOOL_NAME.to_string(),
    ]
}

fn stage_three_jira_tool_names() -> Vec<String> {
    tools::STAGE3_JIRA_TOOL_NAMES
        .iter()
        .map(|name| (*name).to_string())
        .collect()
}

fn stage4_confluence_tool_names() -> Vec<String> {
    confluence_tools::STAGE4_CONFLUENCE_TOOL_NAMES
        .iter()
        .map(|name| (*name).to_string())
        .collect()
}

#[test]
fn baseline_toolsets_match_stage_one_reference() {
    let all = all_toolsets();
    let defaults = default_toolsets();

    assert_eq!(all.len(), 21);
    assert_eq!(defaults.len(), 6);
    assert!(defaults.is_subset(&all));
    assert!(all.contains("jira_issues"));
    assert!(all.contains("jira_development"));
    assert!(all.contains("confluence_pages"));
    assert!(all.contains("confluence_attachments"));
}

#[test]
fn stage_two_jira_core_metadata_is_registered() {
    let names = stage_two_jira_tool_names();

    for name in &names {
        let metadata = metadata_for(name).unwrap_or_else(|| panic!("{name} missing metadata"));
        assert_eq!(metadata.service, ToolService::Jira);
        assert!(metadata.toolset.is_some());
        assert!(!metadata.title.is_empty());
        assert!(!metadata.description.is_empty());
    }

    assert_eq!(
        metadata_for(tools::JIRA_GET_ISSUE_TOOL_NAME)
            .unwrap()
            .access,
        ToolAccess::Read
    );
    assert_eq!(
        metadata_for(tools::JIRA_ADD_COMMENT_TOOL_NAME)
            .unwrap()
            .access,
        ToolAccess::Write
    );
}

#[test]
fn stage_three_jira_extension_metadata_is_registered() {
    let names = stage_three_jira_tool_names();

    assert_eq!(names.len(), 40);
    for name in &names {
        let metadata = metadata_for(name).unwrap_or_else(|| panic!("{name} missing metadata"));
        assert_eq!(metadata.service, ToolService::Jira);
        assert!(metadata.toolset.is_some());
        assert!(!metadata.title.is_empty());
        assert!(!metadata.description.is_empty());
    }

    for name in [
        tools::JIRA_CREATE_ISSUE_TOOL_NAME,
        tools::JIRA_BATCH_CREATE_ISSUES_TOOL_NAME,
        tools::JIRA_UPDATE_ISSUE_TOOL_NAME,
        tools::JIRA_DELETE_ISSUE_TOOL_NAME,
        tools::JIRA_CREATE_VERSION_TOOL_NAME,
        tools::JIRA_BATCH_CREATE_VERSIONS_TOOL_NAME,
        tools::JIRA_ADD_WATCHER_TOOL_NAME,
        tools::JIRA_REMOVE_WATCHER_TOOL_NAME,
        tools::JIRA_ADD_WORKLOG_TOOL_NAME,
        tools::JIRA_LINK_TO_EPIC_TOOL_NAME,
        tools::JIRA_CREATE_ISSUE_LINK_TOOL_NAME,
        tools::JIRA_CREATE_REMOTE_ISSUE_LINK_TOOL_NAME,
        tools::JIRA_REMOVE_ISSUE_LINK_TOOL_NAME,
        tools::JIRA_CREATE_SPRINT_TOOL_NAME,
        tools::JIRA_UPDATE_SPRINT_TOOL_NAME,
        tools::JIRA_ADD_ISSUES_TO_SPRINT_TOOL_NAME,
        tools::JIRA_UPDATE_PROFORMA_FORM_ANSWERS_TOOL_NAME,
    ] {
        assert_eq!(
            metadata_for(name).unwrap().access,
            ToolAccess::Write,
            "{name} should be registered as write"
        );
    }

    assert_eq!(
        metadata_for(tools::JIRA_GET_ISSUE_SLA_TOOL_NAME)
            .unwrap()
            .toolset,
        Some("jira_metrics")
    );
}

#[test]
fn stage_four_confluence_metadata_is_registered() {
    let names = stage4_confluence_tool_names();

    assert_eq!(names.len(), 24);
    for name in &names {
        let metadata = metadata_for(name).unwrap_or_else(|| panic!("{name} missing metadata"));
        assert_eq!(metadata.service, ToolService::Confluence);
        assert!(metadata.toolset.is_some());
        assert!(!metadata.title.is_empty());
        assert!(!metadata.description.is_empty());
    }

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
            metadata_for(name).unwrap().access,
            ToolAccess::Write,
            "{name} should be registered as write"
        );
    }

    assert_eq!(
        metadata_for(confluence_tools::CONFLUENCE_GET_SPACE_PAGE_TREE_TOOL_NAME)
            .unwrap()
            .toolset,
        Some("confluence_pages")
    );
}

#[test]
fn toolsets_filter_registered_business_tools() {
    let config = RuntimeConfig {
        enabled_toolsets: BTreeSet::from(["jira_issues".to_string()]),
        jira: Some(jira_config()),
        ..runtime_config()
    };
    let context = context(config);

    let visible = visible_tools(
        [
            tool(tools::JIRA_GET_ISSUE_TOOL_NAME),
            tool(tools::JIRA_SEARCH_FIELDS_TOOL_NAME),
        ],
        &context,
    );

    assert_eq!(
        names(visible),
        vec![tools::JIRA_GET_ISSUE_TOOL_NAME.to_string()]
    );
}

#[test]
fn enabled_tools_filter_by_exact_tool_name() {
    let config = RuntimeConfig {
        enabled_tools: Some(BTreeSet::from(["stage1_synthetic_jira_read".to_string()])),
        jira: Some(jira_config()),
        ..runtime_config()
    };
    let context = context(config);

    let tools = visible_tools_with_metadata(
        [
            tool("stage1_synthetic_jira_write"),
            tool("stage1_synthetic_jira_read"),
        ],
        &context,
        metadata_for_test_tool,
    );

    assert_eq!(names(tools), vec!["stage1_synthetic_jira_read"]);
}

#[test]
fn service_availability_filters_jira_and_confluence_tools() {
    let unavailable = AppContext::default();
    let available = context(RuntimeConfig {
        jira: Some(jira_config()),
        confluence: Some(confluence_config()),
        ..runtime_config()
    });

    assert_eq!(
        names(visible_tools_with_metadata(
            [
                tool("stage1_synthetic_jira_read"),
                tool("stage1_synthetic_confluence_read"),
            ],
            &unavailable,
            metadata_for_test_tool,
        )),
        Vec::<String>::new()
    );
    assert_eq!(
        names(visible_tools_with_metadata(
            [
                tool("stage1_synthetic_jira_read"),
                tool("stage1_synthetic_confluence_read"),
            ],
            &available,
            metadata_for_test_tool,
        )),
        vec![
            "stage1_synthetic_confluence_read".to_string(),
            "stage1_synthetic_jira_read".to_string(),
        ]
    );
}

#[test]
fn real_confluence_tools_require_service_availability_and_obey_read_only() {
    let unavailable = AppContext::default();
    let read_write = context(RuntimeConfig {
        confluence: Some(confluence_config()),
        ..runtime_config()
    });
    let read_only = context(RuntimeConfig {
        read_only: true,
        confluence: Some(confluence_config()),
        ..runtime_config()
    });

    assert!(guard_tool_call(confluence_tools::CONFLUENCE_SEARCH_TOOL_NAME, &unavailable).is_err());
    assert!(guard_tool_call(confluence_tools::CONFLUENCE_SEARCH_TOOL_NAME, &read_write).is_ok());
    assert!(
        guard_tool_call(
            confluence_tools::CONFLUENCE_CREATE_PAGE_TOOL_NAME,
            &read_write
        )
        .is_ok()
    );
    assert!(guard_tool_call(confluence_tools::CONFLUENCE_SEARCH_TOOL_NAME, &read_only).is_ok());
    assert!(
        guard_tool_call(
            confluence_tools::CONFLUENCE_CREATE_PAGE_TOOL_NAME,
            &read_only
        )
        .unwrap_err()
        .message
        .contains(READ_ONLY_BLOCK_MESSAGE)
    );
}

#[test]
fn default_confluence_toolsets_show_reads_and_hide_writes_in_read_only() {
    let read_write = context(RuntimeConfig {
        confluence: Some(confluence_config()),
        enabled_toolsets: default_toolsets(),
        ..runtime_config()
    });
    let read_only = context(RuntimeConfig {
        read_only: true,
        confluence: Some(confluence_config()),
        enabled_toolsets: default_toolsets(),
        ..runtime_config()
    });
    let candidate_tools = [
        tool(confluence_tools::CONFLUENCE_SEARCH_TOOL_NAME),
        tool(confluence_tools::CONFLUENCE_GET_COMMENTS_TOOL_NAME),
        tool(confluence_tools::CONFLUENCE_CREATE_PAGE_TOOL_NAME),
        tool(confluence_tools::CONFLUENCE_ADD_COMMENT_TOOL_NAME),
        tool(confluence_tools::CONFLUENCE_GET_LABELS_TOOL_NAME),
    ];

    assert_eq!(
        names(visible_tools(candidate_tools.clone(), &read_write)),
        vec![
            confluence_tools::CONFLUENCE_ADD_COMMENT_TOOL_NAME.to_string(),
            confluence_tools::CONFLUENCE_CREATE_PAGE_TOOL_NAME.to_string(),
            confluence_tools::CONFLUENCE_GET_COMMENTS_TOOL_NAME.to_string(),
            confluence_tools::CONFLUENCE_SEARCH_TOOL_NAME.to_string(),
        ]
    );
    assert_eq!(
        names(visible_tools(candidate_tools, &read_only)),
        vec![
            confluence_tools::CONFLUENCE_GET_COMMENTS_TOOL_NAME.to_string(),
            confluence_tools::CONFLUENCE_SEARCH_TOOL_NAME.to_string(),
        ]
    );
}

#[test]
fn c2_confluence_default_toolset_filter_covers_all_specific_and_unknown_cases() {
    let all_default = context(RuntimeConfig {
        confluence: Some(confluence_config()),
        enabled_toolsets: default_toolsets(),
        ..runtime_config()
    });
    let pages_only = context(RuntimeConfig {
        confluence: Some(confluence_config()),
        enabled_toolsets: BTreeSet::from(["confluence_pages".to_string()]),
        ..runtime_config()
    });
    let comments_only = context(RuntimeConfig {
        confluence: Some(confluence_config()),
        enabled_toolsets: BTreeSet::from(["confluence_comments".to_string()]),
        ..runtime_config()
    });
    let unknown_only = context(RuntimeConfig {
        confluence: Some(confluence_config()),
        enabled_toolsets: BTreeSet::from(["confluence_unknown".to_string()]),
        ..runtime_config()
    });
    let candidate_tools = [
        tool(confluence_tools::CONFLUENCE_SEARCH_TOOL_NAME),
        tool(confluence_tools::CONFLUENCE_GET_PAGE_TOOL_NAME),
        tool(confluence_tools::CONFLUENCE_GET_PAGE_CHILDREN_TOOL_NAME),
        tool(confluence_tools::CONFLUENCE_GET_SPACE_PAGE_TREE_TOOL_NAME),
        tool(confluence_tools::CONFLUENCE_CREATE_PAGE_TOOL_NAME),
        tool(confluence_tools::CONFLUENCE_UPDATE_PAGE_TOOL_NAME),
        tool(confluence_tools::CONFLUENCE_DELETE_PAGE_TOOL_NAME),
        tool(confluence_tools::CONFLUENCE_MOVE_PAGE_TOOL_NAME),
        tool(confluence_tools::CONFLUENCE_GET_COMMENTS_TOOL_NAME),
        tool(confluence_tools::CONFLUENCE_ADD_COMMENT_TOOL_NAME),
        tool(confluence_tools::CONFLUENCE_REPLY_TO_COMMENT_TOOL_NAME),
        tool(confluence_tools::CONFLUENCE_GET_LABELS_TOOL_NAME),
    ];

    assert_eq!(
        names(visible_tools(candidate_tools.clone(), &all_default)),
        vec![
            confluence_tools::CONFLUENCE_ADD_COMMENT_TOOL_NAME.to_string(),
            confluence_tools::CONFLUENCE_CREATE_PAGE_TOOL_NAME.to_string(),
            confluence_tools::CONFLUENCE_DELETE_PAGE_TOOL_NAME.to_string(),
            confluence_tools::CONFLUENCE_GET_COMMENTS_TOOL_NAME.to_string(),
            confluence_tools::CONFLUENCE_GET_PAGE_TOOL_NAME.to_string(),
            confluence_tools::CONFLUENCE_GET_PAGE_CHILDREN_TOOL_NAME.to_string(),
            confluence_tools::CONFLUENCE_GET_SPACE_PAGE_TREE_TOOL_NAME.to_string(),
            confluence_tools::CONFLUENCE_MOVE_PAGE_TOOL_NAME.to_string(),
            confluence_tools::CONFLUENCE_REPLY_TO_COMMENT_TOOL_NAME.to_string(),
            confluence_tools::CONFLUENCE_SEARCH_TOOL_NAME.to_string(),
            confluence_tools::CONFLUENCE_UPDATE_PAGE_TOOL_NAME.to_string(),
        ]
    );
    assert_eq!(
        names(visible_tools(candidate_tools.clone(), &pages_only)),
        vec![
            confluence_tools::CONFLUENCE_CREATE_PAGE_TOOL_NAME.to_string(),
            confluence_tools::CONFLUENCE_DELETE_PAGE_TOOL_NAME.to_string(),
            confluence_tools::CONFLUENCE_GET_PAGE_TOOL_NAME.to_string(),
            confluence_tools::CONFLUENCE_GET_PAGE_CHILDREN_TOOL_NAME.to_string(),
            confluence_tools::CONFLUENCE_GET_SPACE_PAGE_TREE_TOOL_NAME.to_string(),
            confluence_tools::CONFLUENCE_MOVE_PAGE_TOOL_NAME.to_string(),
            confluence_tools::CONFLUENCE_SEARCH_TOOL_NAME.to_string(),
            confluence_tools::CONFLUENCE_UPDATE_PAGE_TOOL_NAME.to_string(),
        ]
    );
    assert_eq!(
        names(visible_tools(candidate_tools.clone(), &comments_only)),
        vec![
            confluence_tools::CONFLUENCE_ADD_COMMENT_TOOL_NAME.to_string(),
            confluence_tools::CONFLUENCE_GET_COMMENTS_TOOL_NAME.to_string(),
            confluence_tools::CONFLUENCE_REPLY_TO_COMMENT_TOOL_NAME.to_string(),
        ]
    );
    assert_eq!(
        names(visible_tools(candidate_tools, &unknown_only)),
        Vec::<String>::new()
    );
}

#[test]
fn c2_confluence_write_tools_are_direct_call_blocked_in_read_only() {
    let read_only = context(RuntimeConfig {
        read_only: true,
        confluence: Some(confluence_config()),
        enabled_toolsets: default_toolsets(),
        ..runtime_config()
    });

    assert!(guard_tool_call(confluence_tools::CONFLUENCE_SEARCH_TOOL_NAME, &read_only).is_ok());
    for name in [
        confluence_tools::CONFLUENCE_CREATE_PAGE_TOOL_NAME,
        confluence_tools::CONFLUENCE_UPDATE_PAGE_TOOL_NAME,
        confluence_tools::CONFLUENCE_DELETE_PAGE_TOOL_NAME,
        confluence_tools::CONFLUENCE_MOVE_PAGE_TOOL_NAME,
        confluence_tools::CONFLUENCE_ADD_COMMENT_TOOL_NAME,
        confluence_tools::CONFLUENCE_REPLY_TO_COMMENT_TOOL_NAME,
    ] {
        let error = guard_tool_call(name, &read_only).unwrap_err();
        assert!(error.message.contains(READ_ONLY_BLOCK_MESSAGE), "{name}");
    }
}

#[test]
fn confluence_labels_toolset_filters_and_blocks_writes_in_read_only() {
    let read_write = context(RuntimeConfig {
        confluence: Some(confluence_config()),
        enabled_toolsets: BTreeSet::from(["confluence_labels".to_string()]),
        ..runtime_config()
    });
    let read_only = context(RuntimeConfig {
        read_only: true,
        confluence: Some(confluence_config()),
        enabled_toolsets: BTreeSet::from(["confluence_labels".to_string()]),
        ..runtime_config()
    });
    let candidate_tools = [
        tool(confluence_tools::CONFLUENCE_GET_LABELS_TOOL_NAME),
        tool(confluence_tools::CONFLUENCE_ADD_LABEL_TOOL_NAME),
        tool(confluence_tools::CONFLUENCE_SEARCH_TOOL_NAME),
    ];

    assert_eq!(
        names(visible_tools(candidate_tools.clone(), &read_write)),
        vec![
            confluence_tools::CONFLUENCE_ADD_LABEL_TOOL_NAME.to_string(),
            confluence_tools::CONFLUENCE_GET_LABELS_TOOL_NAME.to_string(),
        ]
    );
    assert_eq!(
        names(visible_tools(candidate_tools, &read_only)),
        vec![confluence_tools::CONFLUENCE_GET_LABELS_TOOL_NAME.to_string()]
    );
    assert!(
        guard_tool_call(
            confluence_tools::CONFLUENCE_GET_LABELS_TOOL_NAME,
            &read_only
        )
        .is_ok()
    );
    assert!(
        guard_tool_call(
            confluence_tools::CONFLUENCE_ADD_LABEL_TOOL_NAME,
            &read_write
        )
        .is_ok()
    );
    assert_eq!(
        guard_tool_call(confluence_tools::CONFLUENCE_ADD_LABEL_TOOL_NAME, &read_only)
            .unwrap_err()
            .message,
        READ_ONLY_BLOCK_MESSAGE
    );
}

#[test]
fn confluence_attachments_toolset_filters_and_blocks_writes_in_read_only() {
    let read_write = context(RuntimeConfig {
        confluence: Some(confluence_config()),
        enabled_toolsets: BTreeSet::from(["confluence_attachments".to_string()]),
        ..runtime_config()
    });
    let read_only = context(RuntimeConfig {
        read_only: true,
        confluence: Some(confluence_config()),
        enabled_toolsets: BTreeSet::from(["confluence_attachments".to_string()]),
        ..runtime_config()
    });
    let candidate_tools = [
        tool(confluence_tools::CONFLUENCE_UPLOAD_ATTACHMENT_TOOL_NAME),
        tool(confluence_tools::CONFLUENCE_UPLOAD_ATTACHMENTS_TOOL_NAME),
        tool(confluence_tools::CONFLUENCE_GET_ATTACHMENTS_TOOL_NAME),
        tool(confluence_tools::CONFLUENCE_DOWNLOAD_ATTACHMENT_TOOL_NAME),
        tool(confluence_tools::CONFLUENCE_DOWNLOAD_CONTENT_ATTACHMENTS_TOOL_NAME),
        tool(confluence_tools::CONFLUENCE_DELETE_ATTACHMENT_TOOL_NAME),
        tool(confluence_tools::CONFLUENCE_GET_PAGE_IMAGES_TOOL_NAME),
        tool(confluence_tools::CONFLUENCE_SEARCH_TOOL_NAME),
    ];

    assert_eq!(
        names(visible_tools(candidate_tools.clone(), &read_write)),
        vec![
            confluence_tools::CONFLUENCE_DELETE_ATTACHMENT_TOOL_NAME.to_string(),
            confluence_tools::CONFLUENCE_DOWNLOAD_ATTACHMENT_TOOL_NAME.to_string(),
            confluence_tools::CONFLUENCE_DOWNLOAD_CONTENT_ATTACHMENTS_TOOL_NAME.to_string(),
            confluence_tools::CONFLUENCE_GET_ATTACHMENTS_TOOL_NAME.to_string(),
            confluence_tools::CONFLUENCE_GET_PAGE_IMAGES_TOOL_NAME.to_string(),
            confluence_tools::CONFLUENCE_UPLOAD_ATTACHMENT_TOOL_NAME.to_string(),
            confluence_tools::CONFLUENCE_UPLOAD_ATTACHMENTS_TOOL_NAME.to_string(),
        ]
    );
    assert_eq!(
        names(visible_tools(candidate_tools, &read_only)),
        vec![
            confluence_tools::CONFLUENCE_DOWNLOAD_ATTACHMENT_TOOL_NAME.to_string(),
            confluence_tools::CONFLUENCE_DOWNLOAD_CONTENT_ATTACHMENTS_TOOL_NAME.to_string(),
            confluence_tools::CONFLUENCE_GET_ATTACHMENTS_TOOL_NAME.to_string(),
            confluence_tools::CONFLUENCE_GET_PAGE_IMAGES_TOOL_NAME.to_string(),
        ]
    );

    for read_tool in [
        confluence_tools::CONFLUENCE_GET_ATTACHMENTS_TOOL_NAME,
        confluence_tools::CONFLUENCE_DOWNLOAD_ATTACHMENT_TOOL_NAME,
        confluence_tools::CONFLUENCE_DOWNLOAD_CONTENT_ATTACHMENTS_TOOL_NAME,
        confluence_tools::CONFLUENCE_GET_PAGE_IMAGES_TOOL_NAME,
    ] {
        assert!(
            guard_tool_call(read_tool, &read_only).is_ok(),
            "{read_tool}"
        );
    }
    for write_tool in [
        confluence_tools::CONFLUENCE_UPLOAD_ATTACHMENT_TOOL_NAME,
        confluence_tools::CONFLUENCE_UPLOAD_ATTACHMENTS_TOOL_NAME,
        confluence_tools::CONFLUENCE_DELETE_ATTACHMENT_TOOL_NAME,
    ] {
        assert!(
            guard_tool_call(write_tool, &read_write).is_ok(),
            "{write_tool}"
        );
        assert_eq!(
            guard_tool_call(write_tool, &read_only).unwrap_err().message,
            READ_ONLY_BLOCK_MESSAGE,
            "{write_tool}"
        );
    }
}

#[test]
fn enabled_tools_filter_can_select_single_confluence_tool() {
    let context = context(RuntimeConfig {
        confluence: Some(confluence_config()),
        enabled_tools: Some(BTreeSet::from([
            confluence_tools::CONFLUENCE_SEARCH_TOOL_NAME.to_string(),
        ])),
        ..runtime_config()
    });

    assert_eq!(
        names(visible_tools(
            [
                tool(confluence_tools::CONFLUENCE_SEARCH_TOOL_NAME),
                tool(confluence_tools::CONFLUENCE_GET_PAGE_TOOL_NAME),
            ],
            &context,
        )),
        vec![confluence_tools::CONFLUENCE_SEARCH_TOOL_NAME.to_string()]
    );
}

#[test]
fn toolset_filter_hides_synthetic_tools_outside_enabled_toolsets() {
    let context = context(RuntimeConfig {
        jira: Some(jira_config()),
        enabled_toolsets: BTreeSet::from(["jira_fields".to_string()]),
        ..runtime_config()
    });

    let tools = visible_tools_with_metadata(
        [tool("stage1_synthetic_jira_read")],
        &context,
        metadata_for_test_tool,
    );

    assert!(tools.is_empty());
}

#[test]
fn read_only_hides_write_tools_and_direct_call_guard_rejects_them() {
    let context = context(RuntimeConfig {
        read_only: true,
        jira: Some(jira_config()),
        ..runtime_config()
    });

    let tools = visible_tools_with_metadata(
        [
            tool("stage1_synthetic_jira_read"),
            tool("stage1_synthetic_jira_write"),
        ],
        &context,
        metadata_for_test_tool,
    );
    let error = guard_tool_call_with_metadata(
        "stage1_synthetic_jira_write",
        &context,
        metadata_for_test_tool,
    )
    .unwrap_err();

    assert_eq!(names(tools), vec!["stage1_synthetic_jira_read"]);
    assert_eq!(error.message, READ_ONLY_BLOCK_MESSAGE);
    assert_eq!(error.data, None);
}

#[test]
fn direct_call_guard_allows_write_tools_when_read_only_is_disabled() {
    let context = context(RuntimeConfig {
        jira: Some(jira_config()),
        ..runtime_config()
    });

    assert!(
        guard_tool_call_with_metadata(
            "stage1_synthetic_jira_write",
            &context,
            metadata_for_test_tool,
        )
        .is_ok()
    );
}

#[test]
fn real_jira_tools_require_service_availability_and_obey_read_only() {
    let unavailable = AppContext::default();
    let read_write = context(RuntimeConfig {
        jira: Some(jira_config()),
        ..runtime_config()
    });
    let read_only = context(RuntimeConfig {
        read_only: true,
        jira: Some(jira_config()),
        ..runtime_config()
    });

    assert!(
        guard_tool_call(tools::JIRA_GET_ISSUE_TOOL_NAME, &unavailable).is_err(),
        "Jira read tools must not be callable without complete Jira config"
    );
    assert!(guard_tool_call(tools::JIRA_ADD_COMMENT_TOOL_NAME, &read_write).is_ok());
    assert_eq!(
        guard_tool_call(tools::JIRA_ADD_COMMENT_TOOL_NAME, &read_only)
            .unwrap_err()
            .message,
        READ_ONLY_BLOCK_MESSAGE
    );
}

#[test]
fn stage_three_toolset_filter_uses_registered_metadata() {
    let context = context(RuntimeConfig {
        jira: Some(jira_config()),
        enabled_toolsets: BTreeSet::from(["jira_agile".to_string()]),
        ..runtime_config()
    });

    let tools = visible_tools(
        [
            tool(tools::JIRA_GET_AGILE_BOARDS_TOOL_NAME),
            tool(tools::JIRA_GET_ALL_PROJECTS_TOOL_NAME),
            tool(tools::JIRA_CREATE_SPRINT_TOOL_NAME),
        ],
        &context,
    );

    assert_eq!(
        names(tools),
        vec![
            tools::JIRA_CREATE_SPRINT_TOOL_NAME.to_string(),
            tools::JIRA_GET_AGILE_BOARDS_TOOL_NAME.to_string(),
        ]
    );
}

#[test]
fn stage_three_read_only_hides_and_blocks_write_tools() {
    let read_only = context(RuntimeConfig {
        read_only: true,
        jira: Some(jira_config()),
        ..runtime_config()
    });

    let visible = visible_tools(
        [
            tool(tools::JIRA_BATCH_GET_CHANGELOGS_TOOL_NAME),
            tool(tools::JIRA_CREATE_ISSUE_TOOL_NAME),
            tool(tools::JIRA_UPDATE_PROFORMA_FORM_ANSWERS_TOOL_NAME),
        ],
        &read_only,
    );
    let error = guard_tool_call(tools::JIRA_CREATE_ISSUE_TOOL_NAME, &read_only).unwrap_err();

    assert_eq!(
        names(visible),
        vec![tools::JIRA_BATCH_GET_CHANGELOGS_TOOL_NAME.to_string()]
    );
    assert_eq!(error.message, READ_ONLY_BLOCK_MESSAGE);
}

#[test]
fn direct_call_guard_fails_closed_for_unknown_tools() {
    let error = guard_tool_call_with_metadata(
        "unknown_tool",
        &AppContext::default(),
        metadata_for_test_tool,
    )
    .unwrap_err();

    assert_eq!(error.message, TOOL_UNAVAILABLE_MESSAGE);
    assert_eq!(error.data, None);
}

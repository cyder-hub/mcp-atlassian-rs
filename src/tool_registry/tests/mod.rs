use std::{collections::BTreeSet, sync::Arc};

use rmcp::model::{JsonObject, Tool};

use crate::{
    config::{HttpConfig, RuntimeConfig},
    confluence::{
        config::{ConfluenceConfig, ConfluenceDeployment},
        tools as confluence_tools,
    },
    context::AppContext,
    gitlab::{config::GitlabConfig, tools as gitlab_tools},
    jira::config::{JiraConfig, JiraDeployment},
    jira::tools,
    upstream::{auth::UpstreamAuth, custom_headers::CustomHeaders, proxy::ProxyConfig},
};

use super::*;

const SYNTHETIC_JIRA_READ: ToolMetadata = ToolMetadata {
    name: "synthetic_jira_read",
    service: ToolService::Jira,
    access: ToolAccess::Read,
    toolset: Some("jira_issues_read"),
    annotations: ToolAnnotationMetadata::read_only(),
    output_schema: None,
    title: "Synthetic Jira read",
    description: "Test-only Jira read metadata.",
};

const SYNTHETIC_JIRA_WRITE: ToolMetadata = ToolMetadata {
    name: "synthetic_jira_write",
    service: ToolService::Jira,
    access: ToolAccess::Write,
    toolset: Some("jira_issues_write"),
    annotations: ToolAnnotationMetadata::additive_write(),
    output_schema: None,
    title: "Synthetic Jira write",
    description: "Test-only Jira write metadata.",
};

const SYNTHETIC_CONFLUENCE_READ: ToolMetadata = ToolMetadata {
    name: "synthetic_confluence_read",
    service: ToolService::Confluence,
    access: ToolAccess::Read,
    toolset: Some("confluence_content_read"),
    annotations: ToolAnnotationMetadata::read_only(),
    output_schema: None,
    title: "Synthetic Confluence read",
    description: "Test-only Confluence read metadata.",
};

const SYNTHETIC_GITLAB_READ: ToolMetadata = ToolMetadata {
    name: "synthetic_gitlab_read",
    service: ToolService::Gitlab,
    access: ToolAccess::Read,
    toolset: Some("gitlab_projects_read"),
    annotations: ToolAnnotationMetadata::read_only(),
    output_schema: None,
    title: "Synthetic GitLab read",
    description: "Test-only GitLab read metadata.",
};

fn metadata_for_test_tool(name: &str) -> Option<ToolMetadata> {
    match name {
        "synthetic_jira_read" => Some(SYNTHETIC_JIRA_READ),
        "synthetic_jira_write" => Some(SYNTHETIC_JIRA_WRITE),
        "synthetic_confluence_read" => Some(SYNTHETIC_CONFLUENCE_READ),
        "synthetic_gitlab_read" => Some(SYNTHETIC_GITLAB_READ),
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
        auth: UpstreamAuth::Pat {
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
        auth: UpstreamAuth::Pat {
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

fn gitlab_config() -> GitlabConfig {
    GitlabConfig {
        base_url: "https://gitlab.example".to_string(),
        auth: UpstreamAuth::Bearer {
            token: "test-token".to_string(),
        },
        ssl_verify: true,
        proxy: ProxyConfig::default(),
        custom_headers: CustomHeaders::default(),
        mtls: None,
        projects_filter: BTreeSet::new(),
        timeout_seconds: 75,
    }
}

fn names(tools: Vec<Tool>) -> Vec<String> {
    tools
        .into_iter()
        .map(|tool| tool.name.to_string())
        .collect()
}

#[test]
fn toolsets_and_profiles_match_control_plane_contract() {
    let all = all_toolsets();
    let defaults = default_toolsets();

    assert_eq!(all.len(), 52);
    assert_eq!(defaults.len(), 11);
    assert!(defaults.is_subset(&all));
    assert!(all.contains("jira_issues_read"));
    assert!(all.contains("jira_issues_delete"));
    assert!(all.contains("jira_sprints_write"));
    assert!(all.contains("jira_service_desks_read"));
    assert!(all.contains("jira_issue_sla_read"));
    assert!(all.contains("confluence_content_read"));
    assert!(all.contains("confluence_content_update"));
    assert!(all.contains("confluence_content_delete"));
    assert!(all.contains("gitlab_projects_read"));
    assert!(all.contains("gitlab_merge_requests_read"));
    assert!(all.contains("gitlab_merge_requests_write"));
    assert!(all.contains("gitlab_merge_requests_merge"));
    let basic_profile = toolsets_for_profile("basic")
        .unwrap()
        .iter()
        .map(|toolset| (*toolset).to_string())
        .collect::<BTreeSet<_>>();
    assert_eq!(basic_profile, defaults);
    assert!(
        toolsets_for_profile("developer")
            .unwrap()
            .contains(&"jira_sprint_membership_write")
    );
    assert!(
        toolsets_for_profile("developer")
            .unwrap()
            .contains(&"gitlab_merge_requests_merge")
    );
    assert!(
        toolsets_for_profile("developer")
            .unwrap()
            .contains(&"jira_issue_metrics_read")
    );
    assert!(
        !toolsets_for_profile("developer")
            .unwrap()
            .contains(&"jira_issue_worklogs_read")
    );
    assert!(
        !toolsets_for_profile("developer")
            .unwrap()
            .contains(&"jira_issue_worklogs_write")
    );
    assert!(
        !toolsets_for_profile("developer")
            .unwrap()
            .contains(&"jira_issue_sla_read")
    );
    assert!(
        toolsets_for_profile("manager")
            .unwrap()
            .contains(&"jira_issues_delete")
    );
    assert!(
        toolsets_for_profile("manager")
            .unwrap()
            .contains(&"jira_issue_worklogs_read")
    );
    assert!(
        toolsets_for_profile("manager")
            .unwrap()
            .contains(&"jira_issue_worklogs_write")
    );
    assert!(
        toolsets_for_profile("manager")
            .unwrap()
            .contains(&"jira_issue_sla_read")
    );
    assert_eq!(toolsets_for_profile("full").unwrap(), ALL_TOOLSETS);
    assert!(toolsets_for_profile("custom").unwrap().is_empty());
    assert!(toolsets_for_profile("unknown").is_none());
}

#[test]
fn profile_tool_counts_match_registered_taxonomy() {
    let count = |profile: &str| {
        let toolsets = toolsets_for_profile(profile)
            .unwrap()
            .iter()
            .copied()
            .collect::<BTreeSet<_>>();
        registered_tools()
            .filter(|metadata| {
                metadata
                    .toolset
                    .is_some_and(|toolset| toolsets.contains(toolset))
            })
            .count()
    };

    assert_eq!(count("basic"), 23);
    assert_eq!(count("developer"), 47);
    assert_eq!(count("manager"), 85);
    assert_eq!(count("full"), 88);
    assert_eq!(count("custom"), 0);
}

#[test]
fn registered_tool_metadata_is_complete_unique_and_uses_known_toolsets() {
    let all_toolsets = all_toolsets();
    let mut names = BTreeSet::new();
    let mut used_toolsets = BTreeSet::new();
    let tools = registered_tools().collect::<Vec<_>>();

    assert_eq!(tools.len(), 88);
    for metadata in tools {
        assert!(
            names.insert(metadata.name),
            "{} should be registered once",
            metadata.name
        );
        assert!(
            !metadata.name.trim().is_empty(),
            "tool name should not be empty"
        );
        assert!(
            !metadata.title.trim().is_empty(),
            "{} should have a title",
            metadata.name
        );
        assert!(
            !metadata.description.trim().is_empty(),
            "{} should have a description",
            metadata.name
        );

        let toolset = metadata
            .toolset
            .unwrap_or_else(|| panic!("{} should declare a toolset", metadata.name));
        assert!(
            all_toolsets.contains(toolset),
            "{} references unknown toolset {toolset}",
            metadata.name
        );
        used_toolsets.insert(toolset.to_string());

        match metadata.service {
            ToolService::Jira => assert!(metadata.name.starts_with("jira_")),
            ToolService::Confluence => assert!(metadata.name.starts_with("confluence_")),
            ToolService::Gitlab => assert!(metadata.name.starts_with("gitlab_")),
        }
        match metadata.access {
            ToolAccess::Read => assert_eq!(
                metadata.annotations,
                ToolAnnotationMetadata::read_only(),
                "{} read access should use read-only annotations",
                metadata.name
            ),
            ToolAccess::Write => assert!(
                !metadata.annotations.read_only && !metadata.annotations.idempotent,
                "{} write access should not claim read-only or idempotent semantics",
                metadata.name
            ),
        }
    }

    assert_eq!(used_toolsets, all_toolsets);
}

#[test]
fn jira_metadata_uses_capability_toolsets() {
    for name in tools::JIRA_EXTENSION_TOOL_NAMES {
        let metadata = metadata_for(name).unwrap_or_else(|| panic!("{name} missing metadata"));
        assert_eq!(metadata.service, ToolService::Jira);
        assert!(metadata.toolset.is_some());
        assert!(!metadata.title.is_empty());
        assert!(!metadata.description.is_empty());
    }

    assert_eq!(
        metadata_for(tools::JIRA_GET_ISSUE_TOOL_NAME)
            .unwrap()
            .toolset,
        Some("jira_issues_read")
    );
    assert_eq!(
        metadata_for(tools::JIRA_CREATE_ISSUE_TOOL_NAME)
            .unwrap()
            .toolset,
        Some("jira_issues_write")
    );
    assert_eq!(
        metadata_for(tools::JIRA_DELETE_ISSUE_TOOL_NAME)
            .unwrap()
            .toolset,
        Some("jira_issues_delete")
    );
    assert_eq!(
        metadata_for(tools::JIRA_ADD_ISSUES_TO_SPRINT_TOOL_NAME)
            .unwrap()
            .toolset,
        Some("jira_sprint_membership_write")
    );
    assert_eq!(
        metadata_for(tools::JIRA_CREATE_SPRINT_TOOL_NAME)
            .unwrap()
            .toolset,
        Some("jira_sprints_write")
    );
}

#[test]
fn confluence_metadata_uses_risk_split_toolsets() {
    for name in confluence_tools::CONFLUENCE_TOOL_NAMES {
        let metadata = metadata_for(name).unwrap_or_else(|| panic!("{name} missing metadata"));
        assert_eq!(metadata.service, ToolService::Confluence);
        assert!(metadata.toolset.is_some());
        assert!(!metadata.title.is_empty());
        assert!(!metadata.description.is_empty());
    }

    assert_eq!(
        metadata_for(confluence_tools::CONFLUENCE_SEARCH_TOOL_NAME)
            .unwrap()
            .toolset,
        Some("confluence_content_read")
    );
    assert_eq!(
        metadata_for(confluence_tools::CONFLUENCE_CREATE_PAGE_TOOL_NAME)
            .unwrap()
            .toolset,
        Some("confluence_content_write")
    );
    assert_eq!(
        metadata_for(confluence_tools::CONFLUENCE_UPDATE_PAGE_TOOL_NAME)
            .unwrap()
            .toolset,
        Some("confluence_content_update")
    );
    assert_eq!(
        metadata_for(confluence_tools::CONFLUENCE_DELETE_PAGE_TOOL_NAME)
            .unwrap()
            .toolset,
        Some("confluence_content_delete")
    );
}

#[test]
fn gitlab_metadata_uses_coarse_toolsets() {
    for name in gitlab_tools::GITLAB_TOOL_NAMES {
        let metadata = metadata_for(name).unwrap_or_else(|| panic!("{name} missing metadata"));
        assert_eq!(metadata.service, ToolService::Gitlab);
        assert!(metadata.toolset.is_some());
        assert!(!metadata.title.is_empty());
        assert!(!metadata.description.is_empty());
    }

    assert_eq!(
        metadata_for(gitlab_tools::GITLAB_GET_PROJECT_TOOL_NAME)
            .unwrap()
            .toolset,
        Some("gitlab_projects_read")
    );
    assert_eq!(
        metadata_for(gitlab_tools::GITLAB_GET_MERGE_REQUEST_TOOL_NAME)
            .unwrap()
            .toolset,
        Some("gitlab_merge_requests_read")
    );
    assert_eq!(
        metadata_for(gitlab_tools::GITLAB_CREATE_MERGE_REQUEST_TOOL_NAME)
            .unwrap()
            .toolset,
        Some("gitlab_merge_requests_write")
    );
    assert_eq!(
        metadata_for(gitlab_tools::GITLAB_ACCEPT_MERGE_REQUEST_TOOL_NAME)
            .unwrap()
            .toolset,
        Some("gitlab_merge_requests_merge")
    );
}

#[test]
fn metadata_declares_tool_annotations_without_name_heuristics() {
    let read = metadata_for(tools::JIRA_GET_ISSUE_TOOL_NAME).unwrap();
    let additive_write = metadata_for(tools::JIRA_CREATE_ISSUE_TOOL_NAME).unwrap();
    let destructive_write = metadata_for(tools::JIRA_UPDATE_ISSUE_TOOL_NAME).unwrap();
    let product_unavailable_read =
        metadata_for(tools::JIRA_GET_ISSUE_DEVELOPMENT_TOOL_NAME).unwrap();

    assert_eq!(read.access, ToolAccess::Read);
    assert_eq!(read.annotations, ToolAnnotationMetadata::read_only());
    assert_eq!(product_unavailable_read.access, ToolAccess::Read);
    assert_eq!(
        product_unavailable_read.annotations,
        ToolAnnotationMetadata::read_only()
    );

    assert_eq!(additive_write.access, ToolAccess::Write);
    assert_eq!(
        additive_write.annotations,
        ToolAnnotationMetadata::additive_write()
    );
    assert_eq!(destructive_write.access, ToolAccess::Write);
    assert_eq!(
        destructive_write.annotations,
        ToolAnnotationMetadata::destructive_write()
    );
}

#[test]
fn destructive_annotation_set_matches_reviewed_write_tools() {
    let destructive_tools = registered_tools()
        .filter(|metadata| metadata.annotations.destructive)
        .map(|metadata| metadata.name)
        .collect::<BTreeSet<_>>();

    assert_eq!(
        destructive_tools,
        BTreeSet::from([
            tools::JIRA_EDIT_COMMENT_TOOL_NAME,
            tools::JIRA_TRANSITION_ISSUE_TOOL_NAME,
            tools::JIRA_UPDATE_ISSUE_TOOL_NAME,
            tools::JIRA_DELETE_ISSUE_TOOL_NAME,
            tools::JIRA_REMOVE_WATCHER_TOOL_NAME,
            tools::JIRA_SET_ISSUE_PARENT_TOOL_NAME,
            tools::JIRA_DELETE_ISSUE_LINK_TOOL_NAME,
            tools::JIRA_UPDATE_SPRINT_TOOL_NAME,
            tools::JIRA_ADD_ISSUES_TO_SPRINT_TOOL_NAME,
            tools::JIRA_UPDATE_ISSUE_FORM_ANSWERS_TOOL_NAME,
            confluence_tools::CONFLUENCE_UPDATE_PAGE_TOOL_NAME,
            confluence_tools::CONFLUENCE_DELETE_PAGE_TOOL_NAME,
            confluence_tools::CONFLUENCE_MOVE_PAGE_TOOL_NAME,
            confluence_tools::CONFLUENCE_UPLOAD_CONTENT_ATTACHMENT_TOOL_NAME,
            confluence_tools::CONFLUENCE_UPLOAD_CONTENT_ATTACHMENTS_TOOL_NAME,
            confluence_tools::CONFLUENCE_DELETE_ATTACHMENT_TOOL_NAME,
            gitlab_tools::GITLAB_UPDATE_MERGE_REQUEST_TOOL_NAME,
            gitlab_tools::GITLAB_RESOLVE_MERGE_REQUEST_DISCUSSION_TOOL_NAME,
            gitlab_tools::GITLAB_SET_MERGE_REQUEST_APPROVAL_TOOL_NAME,
            gitlab_tools::GITLAB_ACCEPT_MERGE_REQUEST_TOOL_NAME,
        ])
    );
}

#[test]
fn annotations_are_consistent_with_registry_access() {
    for metadata in registered_tools() {
        assert_eq!(
            metadata.annotations.read_only,
            metadata.access == ToolAccess::Read,
            "{} read_only annotation should match registry access",
            metadata.name
        );
        assert!(
            metadata.annotations.open_world,
            "{} should declare that it reads or writes external Atlassian state",
            metadata.name
        );
        if metadata.access == ToolAccess::Read {
            assert!(
                !metadata.annotations.destructive,
                "{} read tool should not be destructive",
                metadata.name
            );
        } else {
            assert!(
                !metadata.annotations.idempotent,
                "{} write tool should not claim idempotence without a stronger API guarantee",
                metadata.name
            );
        }
    }
}

#[test]
fn metadata_declares_output_schemas_for_high_risk_payloads() {
    assert_eq!(
        metadata_for(tools::JIRA_UPDATE_ISSUE_TOOL_NAME)
            .unwrap()
            .output_schema,
        Some(ToolOutputSchema::JiraMutationResult)
    );
    assert_eq!(
        metadata_for(tools::JIRA_CREATE_ISSUES_TOOL_NAME)
            .unwrap()
            .output_schema,
        Some(ToolOutputSchema::JiraCreateIssuesResult)
    );
    assert_eq!(
        metadata_for(tools::JIRA_GET_ISSUE_ATTACHMENTS_TOOL_NAME)
            .unwrap()
            .output_schema,
        Some(ToolOutputSchema::JiraIssueAttachmentsResult)
    );
    assert_eq!(
        metadata_for(tools::JIRA_LIST_AGILE_BOARDS_TOOL_NAME)
            .unwrap()
            .output_schema,
        Some(ToolOutputSchema::JiraProductDependencyResult)
    );
    assert_eq!(
        metadata_for(confluence_tools::CONFLUENCE_UPLOAD_CONTENT_ATTACHMENTS_TOOL_NAME)
            .unwrap()
            .output_schema,
        Some(ToolOutputSchema::ConfluenceBatchAttachmentUploadResult)
    );
}

#[test]
fn default_profile_exposes_basic_tools_only() {
    let context = context(RuntimeConfig {
        jira: Some(jira_config()),
        confluence: Some(confluence_config()),
        ..runtime_config()
    });

    let visible = visible_tools(
        [
            tool(tools::JIRA_GET_ISSUE_TOOL_NAME),
            tool(tools::JIRA_CREATE_ISSUE_TOOL_NAME),
            tool(tools::JIRA_DELETE_ISSUE_TOOL_NAME),
            tool(tools::JIRA_LIST_AGILE_BOARDS_TOOL_NAME),
            tool(confluence_tools::CONFLUENCE_SEARCH_TOOL_NAME),
            tool(confluence_tools::CONFLUENCE_CREATE_PAGE_TOOL_NAME),
        ],
        &context,
    );

    assert_eq!(
        names(visible),
        vec![
            confluence_tools::CONFLUENCE_SEARCH_TOOL_NAME.to_string(),
            tools::JIRA_CREATE_ISSUE_TOOL_NAME.to_string(),
            tools::JIRA_GET_ISSUE_TOOL_NAME.to_string(),
        ]
    );
}

#[test]
fn toolsets_are_additive_and_exact_tools_can_add_or_remove() {
    let context = context(RuntimeConfig {
        jira: Some(jira_config()),
        enabled_tools: Some(BTreeSet::from([
            tools::JIRA_DELETE_ISSUE_TOOL_NAME.to_string()
        ])),
        disabled_tools: BTreeSet::from([tools::JIRA_CREATE_ISSUE_TOOL_NAME.to_string()]),
        enabled_toolsets: BTreeSet::from(["jira_agile_boards_read".to_string()]),
        ..runtime_config()
    });

    let visible = visible_tools(
        [
            tool(tools::JIRA_GET_ISSUE_TOOL_NAME),
            tool(tools::JIRA_CREATE_ISSUE_TOOL_NAME),
            tool(tools::JIRA_DELETE_ISSUE_TOOL_NAME),
            tool(tools::JIRA_LIST_AGILE_BOARDS_TOOL_NAME),
        ],
        &context,
    );

    assert_eq!(
        names(visible),
        vec![
            tools::JIRA_DELETE_ISSUE_TOOL_NAME.to_string(),
            tools::JIRA_LIST_AGILE_BOARDS_TOOL_NAME.to_string(),
        ]
    );
    assert!(guard_tool_call(tools::JIRA_DELETE_ISSUE_TOOL_NAME, &context).is_ok());
    assert!(guard_tool_call(tools::JIRA_CREATE_ISSUE_TOOL_NAME, &context).is_err());
}

#[test]
fn service_availability_filters_jira_confluence_and_gitlab_tools() {
    let unavailable = AppContext::default();
    let available = context(RuntimeConfig {
        jira: Some(jira_config()),
        confluence: Some(confluence_config()),
        gitlab: Some(gitlab_config()),
        ..runtime_config()
    });

    assert_eq!(
        names(visible_tools_with_metadata(
            [
                tool("synthetic_jira_read"),
                tool("synthetic_confluence_read"),
                tool("synthetic_gitlab_read"),
            ],
            &unavailable,
            metadata_for_test_tool,
        )),
        Vec::<String>::new()
    );
    assert_eq!(
        names(visible_tools_with_metadata(
            [
                tool("synthetic_jira_read"),
                tool("synthetic_confluence_read"),
                tool("synthetic_gitlab_read"),
            ],
            &available,
            metadata_for_test_tool,
        )),
        vec![
            "synthetic_confluence_read".to_string(),
            "synthetic_gitlab_read".to_string(),
            "synthetic_jira_read".to_string(),
        ]
    );
}

#[test]
fn gitlab_profile_toolsets_and_disabled_tools_are_enforced() {
    let basic = context(RuntimeConfig {
        gitlab: Some(gitlab_config()),
        disabled_tools: BTreeSet::from([
            gitlab_tools::GITLAB_GET_MERGE_REQUEST_TOOL_NAME.to_string()
        ]),
        ..runtime_config()
    });
    let developer = context(RuntimeConfig {
        gitlab: Some(gitlab_config()),
        enabled_toolsets: toolsets_for_profile("developer")
            .unwrap()
            .iter()
            .map(|toolset| (*toolset).to_string())
            .collect(),
        ..runtime_config()
    });

    let candidates = [
        tool(gitlab_tools::GITLAB_GET_PROJECT_TOOL_NAME),
        tool(gitlab_tools::GITLAB_GET_MERGE_REQUEST_TOOL_NAME),
        tool(gitlab_tools::GITLAB_CREATE_MERGE_REQUEST_TOOL_NAME),
        tool(gitlab_tools::GITLAB_ACCEPT_MERGE_REQUEST_TOOL_NAME),
    ];

    assert_eq!(
        names(visible_tools(candidates.clone(), &basic)),
        vec![gitlab_tools::GITLAB_GET_PROJECT_TOOL_NAME.to_string()]
    );
    assert!(guard_tool_call(gitlab_tools::GITLAB_GET_PROJECT_TOOL_NAME, &basic).is_ok());
    assert!(guard_tool_call(gitlab_tools::GITLAB_GET_MERGE_REQUEST_TOOL_NAME, &basic).is_err());
    assert!(guard_tool_call(gitlab_tools::GITLAB_ACCEPT_MERGE_REQUEST_TOOL_NAME, &basic).is_err());

    assert_eq!(
        names(visible_tools(candidates, &developer)),
        vec![
            gitlab_tools::GITLAB_ACCEPT_MERGE_REQUEST_TOOL_NAME.to_string(),
            gitlab_tools::GITLAB_CREATE_MERGE_REQUEST_TOOL_NAME.to_string(),
            gitlab_tools::GITLAB_GET_MERGE_REQUEST_TOOL_NAME.to_string(),
            gitlab_tools::GITLAB_GET_PROJECT_TOOL_NAME.to_string(),
        ]
    );
}

#[test]
fn guard_fails_closed_for_unknown_or_disabled_tools() {
    let context = context(RuntimeConfig {
        jira: Some(jira_config()),
        disabled_tools: BTreeSet::from(["synthetic_jira_write".to_string()]),
        ..runtime_config()
    });

    let unknown = guard_tool_call_with_metadata("unknown_tool", &context, metadata_for_test_tool)
        .unwrap_err();
    let disabled =
        guard_tool_call_with_metadata("synthetic_jira_write", &context, metadata_for_test_tool)
            .unwrap_err();

    assert_eq!(unknown.message, TOOL_UNAVAILABLE_MESSAGE);
    assert_eq!(disabled.message, TOOL_UNAVAILABLE_MESSAGE);
}

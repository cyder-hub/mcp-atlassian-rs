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
    name: "synthetic_jira_read",
    service: ToolService::Jira,
    access: ToolAccess::Read,
    toolset: Some("jira_issue_read"),
    title: "Synthetic Jira read",
    description: "Test-only Jira read metadata.",
};

const SYNTHETIC_JIRA_WRITE: ToolMetadata = ToolMetadata {
    name: "synthetic_jira_write",
    service: ToolService::Jira,
    access: ToolAccess::Write,
    toolset: Some("jira_issue_write"),
    title: "Synthetic Jira write",
    description: "Test-only Jira write metadata.",
};

const SYNTHETIC_CONFLUENCE_READ: ToolMetadata = ToolMetadata {
    name: "synthetic_confluence_read",
    service: ToolService::Confluence,
    access: ToolAccess::Read,
    toolset: Some("confluence_content_read"),
    title: "Synthetic Confluence read",
    description: "Test-only Confluence read metadata.",
};

fn metadata_for_test_tool(name: &str) -> Option<ToolMetadata> {
    match name {
        "synthetic_jira_read" => Some(SYNTHETIC_JIRA_READ),
        "synthetic_jira_write" => Some(SYNTHETIC_JIRA_WRITE),
        "synthetic_confluence_read" => Some(SYNTHETIC_CONFLUENCE_READ),
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

#[test]
fn toolsets_and_profiles_match_control_plane_contract() {
    let all = all_toolsets();
    let defaults = default_toolsets();

    assert_eq!(all.len(), 37);
    assert_eq!(defaults.len(), 10);
    assert!(defaults.is_subset(&all));
    assert!(all.contains("jira_issue_read"));
    assert!(all.contains("jira_issue_delete"));
    assert!(all.contains("jira_sprint_manage"));
    assert!(all.contains("jira_service_desk"));
    assert!(all.contains("confluence_content_read"));
    assert!(all.contains("confluence_content_delete"));
    let basic_profile = toolsets_for_profile("basic")
        .unwrap()
        .iter()
        .map(|toolset| (*toolset).to_string())
        .collect::<BTreeSet<_>>();
    assert_eq!(basic_profile, defaults);
    assert!(
        toolsets_for_profile("developer")
            .unwrap()
            .contains(&"jira_sprint_planning")
    );
    assert!(
        toolsets_for_profile("manager")
            .unwrap()
            .contains(&"jira_issue_delete")
    );
    assert_eq!(toolsets_for_profile("full").unwrap(), ALL_TOOLSETS);
    assert!(toolsets_for_profile("custom").unwrap().is_empty());
    assert!(toolsets_for_profile("unknown").is_none());
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
        Some("jira_issue_read")
    );
    assert_eq!(
        metadata_for(tools::JIRA_CREATE_ISSUE_TOOL_NAME)
            .unwrap()
            .toolset,
        Some("jira_issue_write")
    );
    assert_eq!(
        metadata_for(tools::JIRA_DELETE_ISSUE_TOOL_NAME)
            .unwrap()
            .toolset,
        Some("jira_issue_delete")
    );
    assert_eq!(
        metadata_for(tools::JIRA_ADD_ISSUES_TO_SPRINT_TOOL_NAME)
            .unwrap()
            .toolset,
        Some("jira_sprint_planning")
    );
    assert_eq!(
        metadata_for(tools::JIRA_CREATE_SPRINT_TOOL_NAME)
            .unwrap()
            .toolset,
        Some("jira_sprint_manage")
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
        metadata_for(confluence_tools::CONFLUENCE_DELETE_PAGE_TOOL_NAME)
            .unwrap()
            .toolset,
        Some("confluence_content_delete")
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
            tool(tools::JIRA_GET_AGILE_BOARDS_TOOL_NAME),
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
        enabled_toolsets: BTreeSet::from(["jira_agile_read".to_string()]),
        ..runtime_config()
    });

    let visible = visible_tools(
        [
            tool(tools::JIRA_GET_ISSUE_TOOL_NAME),
            tool(tools::JIRA_CREATE_ISSUE_TOOL_NAME),
            tool(tools::JIRA_DELETE_ISSUE_TOOL_NAME),
            tool(tools::JIRA_GET_AGILE_BOARDS_TOOL_NAME),
        ],
        &context,
    );

    assert_eq!(
        names(visible),
        vec![
            tools::JIRA_DELETE_ISSUE_TOOL_NAME.to_string(),
            tools::JIRA_GET_AGILE_BOARDS_TOOL_NAME.to_string(),
        ]
    );
    assert!(guard_tool_call(tools::JIRA_DELETE_ISSUE_TOOL_NAME, &context).is_ok());
    assert!(guard_tool_call(tools::JIRA_CREATE_ISSUE_TOOL_NAME, &context).is_err());
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
                tool("synthetic_jira_read"),
                tool("synthetic_confluence_read"),
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
            ],
            &available,
            metadata_for_test_tool,
        )),
        vec![
            "synthetic_confluence_read".to_string(),
            "synthetic_jira_read".to_string(),
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

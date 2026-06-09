use std::collections::BTreeSet;

use rmcp::{ErrorData, model::Tool};

use crate::context::AppContext;

const TOOL_UNAVAILABLE_MESSAGE: &str = "tool not available";
pub const DEFAULT_TOOL_PROFILE: &str = "basic";

const ALL_TOOLSETS: &[&str] = &[
    "jira_issue_read",
    "jira_issue_write",
    "jira_issue_delete",
    "jira_issue_bulk_write",
    "jira_issue_history_read",
    "jira_fields_read",
    "jira_comments_write",
    "jira_workflow_read",
    "jira_workflow_write",
    "jira_project_read",
    "jira_project_metadata_read",
    "jira_project_write",
    "jira_agile_read",
    "jira_sprint_planning",
    "jira_sprint_manage",
    "jira_development_read",
    "jira_attachments_read",
    "jira_worklog",
    "jira_links",
    "jira_users",
    "jira_watchers",
    "jira_service_desk",
    "jira_forms",
    "jira_metrics_read",
    "confluence_content_read",
    "confluence_content_write",
    "confluence_content_delete",
    "confluence_versions_read",
    "confluence_comments_read",
    "confluence_comments_write",
    "confluence_labels_read",
    "confluence_labels_write",
    "confluence_users_read",
    "confluence_analytics_read",
    "confluence_attachments_read",
    "confluence_attachments_write",
    "confluence_attachments_delete",
];

const DEFAULT_TOOLSETS: &[&str] = &[
    "jira_issue_read",
    "jira_issue_write",
    "jira_fields_read",
    "jira_comments_write",
    "jira_workflow_read",
    "jira_workflow_write",
    "jira_project_read",
    "confluence_content_read",
    "confluence_comments_read",
    "confluence_labels_read",
];

const BASIC_PROFILE_TOOLSETS: &[&str] = DEFAULT_TOOLSETS;
const DEVELOPER_PROFILE_TOOLSETS: &[&str] = &[
    "jira_issue_read",
    "jira_issue_write",
    "jira_fields_read",
    "jira_comments_write",
    "jira_workflow_read",
    "jira_workflow_write",
    "jira_project_read",
    "confluence_content_read",
    "confluence_comments_read",
    "confluence_labels_read",
    "jira_agile_read",
    "jira_sprint_planning",
    "jira_development_read",
    "jira_attachments_read",
    "jira_worklog",
    "jira_metrics_read",
    "confluence_versions_read",
    "confluence_attachments_read",
];
const MANAGER_PROFILE_TOOLSETS: &[&str] = &[
    "jira_issue_read",
    "jira_issue_write",
    "jira_fields_read",
    "jira_comments_write",
    "jira_workflow_read",
    "jira_workflow_write",
    "jira_project_read",
    "confluence_content_read",
    "confluence_comments_read",
    "confluence_labels_read",
    "jira_agile_read",
    "jira_sprint_planning",
    "jira_development_read",
    "jira_attachments_read",
    "jira_worklog",
    "jira_metrics_read",
    "confluence_versions_read",
    "confluence_attachments_read",
    "jira_sprint_manage",
    "jira_issue_delete",
    "jira_issue_bulk_write",
    "jira_issue_history_read",
    "jira_project_metadata_read",
    "jira_project_write",
    "jira_links",
    "jira_users",
    "jira_watchers",
    "confluence_content_write",
    "confluence_comments_write",
    "confluence_labels_write",
    "confluence_attachments_write",
];
const FULL_PROFILE_TOOLSETS: &[&str] = ALL_TOOLSETS;
const CUSTOM_PROFILE_TOOLSETS: &[&str] = &[];

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ToolService {
    Jira,
    Confluence,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ToolAccess {
    Read,
    Write,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ToolMetadata {
    pub name: &'static str,
    pub service: ToolService,
    pub access: ToolAccess,
    pub toolset: Option<&'static str>,
    pub title: &'static str,
    pub description: &'static str,
}

macro_rules! jira_metadata {
    ($constant:ident, $name:expr, $access:ident, $toolset:literal, $title:literal, $description:literal) => {
        pub const $constant: ToolMetadata = ToolMetadata {
            name: $name,
            service: ToolService::Jira,
            access: ToolAccess::$access,
            toolset: Some($toolset),
            title: $title,
            description: $description,
        };
    };
}

macro_rules! confluence_metadata {
    ($constant:ident, $name:expr, $access:ident, $toolset:literal, $title:literal, $description:literal) => {
        pub const $constant: ToolMetadata = ToolMetadata {
            name: $name,
            service: ToolService::Confluence,
            access: ToolAccess::$access,
            toolset: Some($toolset),
            title: $title,
            description: $description,
        };
    };
}

mod confluence;
mod jira;

const REGISTERED_TOOLS: &[&[ToolMetadata]] = &[jira::TOOLS, confluence::TOOLS];

fn registered_tools() -> impl Iterator<Item = ToolMetadata> {
    REGISTERED_TOOLS
        .iter()
        .flat_map(|tools| tools.iter())
        .copied()
}

pub fn all_toolsets() -> BTreeSet<String> {
    ALL_TOOLSETS
        .iter()
        .map(|toolset| (*toolset).to_string())
        .collect()
}

pub fn default_toolsets() -> BTreeSet<String> {
    DEFAULT_TOOLSETS
        .iter()
        .map(|toolset| (*toolset).to_string())
        .collect()
}

pub fn toolsets_for_profile(profile: &str) -> Option<&'static [&'static str]> {
    match profile.trim().to_ascii_lowercase().as_str() {
        "basic" => Some(BASIC_PROFILE_TOOLSETS),
        "developer" => Some(DEVELOPER_PROFILE_TOOLSETS),
        "manager" => Some(MANAGER_PROFILE_TOOLSETS),
        "full" => Some(FULL_PROFILE_TOOLSETS),
        "custom" => Some(CUSTOM_PROFILE_TOOLSETS),
        _ => None,
    }
}

pub fn metadata_for(name: &str) -> Option<ToolMetadata> {
    registered_tools().find(|metadata| metadata.name == name)
}

pub fn visible_tools<I>(tools: I, context: &AppContext) -> Vec<Tool>
where
    I: IntoIterator<Item = Tool>,
{
    visible_tools_with_metadata(tools, context, metadata_for)
}

pub fn visible_tools_with_metadata<I, F>(
    tools: I,
    context: &AppContext,
    metadata_for: F,
) -> Vec<Tool>
where
    I: IntoIterator<Item = Tool>,
    F: Fn(&str) -> Option<ToolMetadata>,
{
    let mut tools: Vec<_> = tools
        .into_iter()
        .filter(|tool| {
            metadata_for(tool.name.as_ref())
                .is_some_and(|metadata| is_discoverable(metadata, context))
        })
        .collect();
    tools.sort_by(|left, right| left.name.cmp(&right.name));
    tools
}

pub fn guard_tool_call(name: &str, context: &AppContext) -> Result<(), ErrorData> {
    guard_tool_call_with_metadata(name, context, metadata_for)
}

pub fn guard_tool_call_with_metadata<F>(
    name: &str,
    context: &AppContext,
    metadata_for: F,
) -> Result<(), ErrorData>
where
    F: Fn(&str) -> Option<ToolMetadata>,
{
    let Some(metadata) = metadata_for(name) else {
        return Err(tool_unavailable_error());
    };

    if !is_tool_enabled(metadata, context) || !is_service_available(metadata, context) {
        return Err(tool_unavailable_error());
    }

    Ok(())
}

fn is_discoverable(metadata: ToolMetadata, context: &AppContext) -> bool {
    is_tool_enabled(metadata, context) && is_service_available(metadata, context)
}

fn is_tool_enabled(metadata: ToolMetadata, context: &AppContext) -> bool {
    if context.disabled_tools().contains(metadata.name) {
        return false;
    }

    context
        .enabled_tools()
        .is_some_and(|enabled_tools| enabled_tools.contains(metadata.name))
        || is_toolset_enabled(metadata, context)
}

fn is_service_available(metadata: ToolMetadata, context: &AppContext) -> bool {
    let availability = context.service_availability();

    match metadata.service {
        ToolService::Jira => availability.jira,
        ToolService::Confluence => availability.confluence,
    }
}

fn is_toolset_enabled(metadata: ToolMetadata, context: &AppContext) -> bool {
    match metadata.toolset {
        Some(toolset) => context.enabled_toolsets().contains(toolset),
        None => true,
    }
}

fn tool_unavailable_error() -> ErrorData {
    ErrorData::invalid_params(TOOL_UNAVAILABLE_MESSAGE, None)
}

#[cfg(test)]
mod tests;

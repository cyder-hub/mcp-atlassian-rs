use std::collections::BTreeSet;

use rmcp::{ErrorData, model::Tool};

use crate::context::AppContext;

const TOOL_UNAVAILABLE_MESSAGE: &str = "tool not available";
const READ_ONLY_BLOCK_MESSAGE: &str = "tool is disabled in read-only mode";

const ALL_TOOLSETS: &[&str] = &[
    "jira_issues",
    "jira_fields",
    "jira_comments",
    "jira_transitions",
    "jira_projects",
    "jira_agile",
    "jira_links",
    "jira_worklog",
    "jira_attachments",
    "jira_users",
    "jira_watchers",
    "jira_service_desk",
    "jira_forms",
    "jira_metrics",
    "jira_development",
    "confluence_pages",
    "confluence_comments",
    "confluence_labels",
    "confluence_users",
    "confluence_analytics",
    "confluence_attachments",
];

const DEFAULT_TOOLSETS: &[&str] = &[
    "jira_issues",
    "jira_fields",
    "jira_comments",
    "jira_transitions",
    "confluence_pages",
    "confluence_comments",
];

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

    if !is_name_enabled(metadata, context)
        || !is_service_available(metadata, context)
        || !is_toolset_enabled(metadata, context)
    {
        return Err(tool_unavailable_error());
    }

    if context.read_only() && metadata.access == ToolAccess::Write {
        return Err(ErrorData::invalid_params(READ_ONLY_BLOCK_MESSAGE, None));
    }

    Ok(())
}

fn is_discoverable(metadata: ToolMetadata, context: &AppContext) -> bool {
    is_name_enabled(metadata, context)
        && is_service_available(metadata, context)
        && is_toolset_enabled(metadata, context)
        && !(context.read_only() && metadata.access == ToolAccess::Write)
}

fn is_name_enabled(metadata: ToolMetadata, context: &AppContext) -> bool {
    match context.enabled_tools() {
        Some(enabled_tools) => enabled_tools.contains(metadata.name),
        None => true,
    }
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

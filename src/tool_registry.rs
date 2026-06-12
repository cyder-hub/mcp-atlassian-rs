use std::{borrow::Cow, collections::BTreeSet, sync::Arc};

use rmcp::{
    ErrorData,
    model::{JsonObject, Tool, ToolAnnotations},
};
use serde_json::{Value, json};

use crate::context::AppContext;

const TOOL_UNAVAILABLE_MESSAGE: &str = "tool not available";
pub const DEFAULT_TOOL_PROFILE: &str = "basic";

const ALL_TOOLSETS: &[&str] = &[
    "jira_issues_read",
    "jira_issues_write",
    "jira_issues_update",
    "jira_issues_delete",
    "jira_issues_bulk_write",
    "jira_issues_history_read",
    "jira_fields_read",
    "jira_issue_comments_write",
    "jira_issue_comments_update",
    "jira_issue_workflows_read",
    "jira_issue_workflows_write",
    "jira_projects_read",
    "jira_projects_metadata_read",
    "jira_project_versions_write",
    "jira_agile_boards_read",
    "jira_sprints_read",
    "jira_sprints_write",
    "jira_sprint_membership_write",
    "jira_issue_development_read",
    "jira_issue_attachments_read",
    "jira_issue_worklogs_read",
    "jira_issue_worklogs_write",
    "jira_issue_links_read",
    "jira_issue_links_write",
    "jira_issue_links_delete",
    "jira_users_read",
    "jira_issue_watchers_read",
    "jira_issue_watchers_write",
    "jira_issue_watchers_delete",
    "jira_service_desks_read",
    "jira_issue_metrics_read",
    "jira_issue_sla_read",
    "confluence_content_read",
    "confluence_content_write",
    "confluence_content_update",
    "confluence_content_delete",
    "confluence_page_versions_read",
    "confluence_page_comments_read",
    "confluence_page_comments_write",
    "confluence_content_labels_read",
    "confluence_content_labels_write",
    "confluence_users_read",
    "confluence_page_analytics_read",
    "confluence_attachments_read",
    "confluence_attachments_write",
    "confluence_attachments_delete",
    "gitlab_projects_read",
    "gitlab_merge_requests_read",
    "gitlab_merge_requests_write",
    "gitlab_merge_requests_merge",
];

const DEFAULT_TOOLSETS: &[&str] = &[
    "jira_issues_read",
    "jira_issues_write",
    "jira_fields_read",
    "jira_issue_comments_write",
    "jira_issue_workflows_read",
    "jira_projects_read",
    "confluence_content_read",
    "confluence_page_comments_read",
    "confluence_content_labels_read",
    "gitlab_projects_read",
    "gitlab_merge_requests_read",
];

const BASIC_PROFILE_TOOLSETS: &[&str] = DEFAULT_TOOLSETS;
const DEVELOPER_PROFILE_TOOLSETS: &[&str] = &[
    "jira_issues_read",
    "jira_issues_write",
    "jira_fields_read",
    "jira_issue_comments_write",
    "jira_issue_workflows_read",
    "jira_projects_read",
    "confluence_content_read",
    "confluence_page_comments_read",
    "confluence_content_labels_read",
    "jira_issue_workflows_write",
    "jira_agile_boards_read",
    "jira_sprints_read",
    "jira_sprint_membership_write",
    "jira_issue_development_read",
    "jira_issue_attachments_read",
    "jira_issue_metrics_read",
    "confluence_page_versions_read",
    "confluence_attachments_read",
    "gitlab_projects_read",
    "gitlab_merge_requests_read",
    "gitlab_merge_requests_write",
    "gitlab_merge_requests_merge",
];
const MANAGER_PROFILE_TOOLSETS: &[&str] = &[
    "jira_issues_read",
    "jira_issues_write",
    "jira_fields_read",
    "jira_issue_comments_write",
    "jira_issue_workflows_read",
    "jira_projects_read",
    "confluence_content_read",
    "confluence_page_comments_read",
    "confluence_content_labels_read",
    "jira_issue_workflows_write",
    "jira_agile_boards_read",
    "jira_sprints_read",
    "jira_sprint_membership_write",
    "jira_issue_development_read",
    "jira_issue_attachments_read",
    "jira_issue_worklogs_read",
    "jira_issue_worklogs_write",
    "jira_issue_metrics_read",
    "confluence_page_versions_read",
    "confluence_attachments_read",
    "jira_issues_update",
    "jira_issues_delete",
    "jira_issues_bulk_write",
    "jira_issues_history_read",
    "jira_issue_comments_update",
    "jira_projects_metadata_read",
    "jira_project_versions_write",
    "jira_sprints_write",
    "jira_issue_links_read",
    "jira_issue_links_write",
    "jira_issue_links_delete",
    "jira_users_read",
    "jira_issue_watchers_read",
    "jira_issue_watchers_write",
    "jira_issue_watchers_delete",
    "jira_service_desks_read",
    "jira_issue_sla_read",
    "confluence_content_write",
    "confluence_content_update",
    "confluence_page_comments_write",
    "confluence_content_labels_write",
    "confluence_page_analytics_read",
    "confluence_attachments_write",
    "gitlab_projects_read",
    "gitlab_merge_requests_read",
    "gitlab_merge_requests_write",
    "gitlab_merge_requests_merge",
];
const FULL_PROFILE_TOOLSETS: &[&str] = ALL_TOOLSETS;
const CUSTOM_PROFILE_TOOLSETS: &[&str] = &[];

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ToolService {
    Jira,
    Confluence,
    Gitlab,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ToolAccess {
    Read,
    Write,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ToolGuardError {
    UnknownTool,
    DisabledTool,
    ServiceUnavailable,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ToolMetadata {
    pub name: &'static str,
    pub service: ToolService,
    pub access: ToolAccess,
    pub toolset: Option<&'static str>,
    pub annotations: ToolAnnotationMetadata,
    pub output_schema: Option<ToolOutputSchema>,
    pub title: &'static str,
    pub description: &'static str,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ToolAnnotationMetadata {
    pub read_only: bool,
    pub destructive: bool,
    pub idempotent: bool,
    pub open_world: bool,
}

impl ToolAnnotationMetadata {
    pub const fn read_only() -> Self {
        Self {
            read_only: true,
            destructive: false,
            idempotent: true,
            open_world: true,
        }
    }

    pub const fn additive_write() -> Self {
        Self {
            read_only: false,
            destructive: false,
            idempotent: false,
            open_world: true,
        }
    }

    pub const fn destructive_write() -> Self {
        Self {
            read_only: false,
            destructive: true,
            idempotent: false,
            open_world: true,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ToolOutputSchema {
    JiraMutationResult,
    JiraCreateIssuesResult,
    JiraPartitionedVersionsResult,
    JiraProductDependencyResult,
    JiraIssueAttachmentsResult,
    JiraIssueImageAttachmentsResult,
    ConfluenceBatchAttachmentUploadResult,
    ConfluenceAttachmentDownloadResult,
    ConfluenceBatchAttachmentDownloadResult,
    ConfluenceAnalyticsResult,
}

macro_rules! jira_metadata {
    ($constant:ident, $name:expr, $access:ident, $toolset:literal, $annotations:ident, $title:literal, $description:literal $(, $output_schema:ident)?) => {
        pub const $constant: ToolMetadata = ToolMetadata {
            name: $name,
            service: ToolService::Jira,
            access: ToolAccess::$access,
            toolset: Some($toolset),
            annotations: ToolAnnotationMetadata::$annotations(),
            output_schema: jira_metadata!(@output_schema $($output_schema)?),
            title: $title,
            description: $description,
        };
    };
    (@output_schema) => {
        None
    };
    (@output_schema $output_schema:ident) => {
        Some(ToolOutputSchema::$output_schema)
    };
}

macro_rules! confluence_metadata {
    ($constant:ident, $name:expr, $access:ident, $toolset:literal, $annotations:ident, $title:literal, $description:literal $(, $output_schema:ident)?) => {
        pub const $constant: ToolMetadata = ToolMetadata {
            name: $name,
            service: ToolService::Confluence,
            access: ToolAccess::$access,
            toolset: Some($toolset),
            annotations: ToolAnnotationMetadata::$annotations(),
            output_schema: confluence_metadata!(@output_schema $($output_schema)?),
            title: $title,
            description: $description,
        };
    };
    (@output_schema) => {
        None
    };
    (@output_schema $output_schema:ident) => {
        Some(ToolOutputSchema::$output_schema)
    };
}

macro_rules! gitlab_metadata {
    ($constant:ident, $name:expr, $access:ident, $toolset:literal, $annotations:ident, $title:literal, $description:literal $(, $output_schema:ident)?) => {
        pub const $constant: ToolMetadata = ToolMetadata {
            name: $name,
            service: ToolService::Gitlab,
            access: ToolAccess::$access,
            toolset: Some($toolset),
            annotations: ToolAnnotationMetadata::$annotations(),
            output_schema: gitlab_metadata!(@output_schema $($output_schema)?),
            title: $title,
            description: $description,
        };
    };
    (@output_schema) => {
        None
    };
    (@output_schema $output_schema:ident) => {
        Some(ToolOutputSchema::$output_schema)
    };
}

mod confluence;
mod gitlab;
mod jira;

const REGISTERED_TOOLS: &[&[ToolMetadata]] = &[jira::TOOLS, confluence::TOOLS, gitlab::TOOLS];

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
        .filter_map(|tool| {
            let metadata = metadata_for(tool.name.as_ref())?;
            is_discoverable(metadata, context).then(|| tool_with_metadata(tool, metadata))
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
    guard_tool_access_with_metadata(name, context, metadata_for)
        .map(|_| ())
        .map_err(|_| tool_unavailable_error())
}

pub fn guard_operation_access(
    name: &str,
    context: &AppContext,
) -> Result<ToolMetadata, ToolGuardError> {
    guard_operation_access_with_metadata(name, context, metadata_for)
}

pub fn guard_tool_access_with_metadata<F>(
    name: &str,
    context: &AppContext,
    metadata_for: F,
) -> Result<ToolMetadata, ToolGuardError>
where
    F: Fn(&str) -> Option<ToolMetadata>,
{
    let metadata = metadata_for(name).ok_or(ToolGuardError::UnknownTool)?;
    guard_tool_metadata(metadata, context)?;
    Ok(metadata)
}

pub fn guard_operation_access_with_metadata<F>(
    name: &str,
    context: &AppContext,
    metadata_for: F,
) -> Result<ToolMetadata, ToolGuardError>
where
    F: Fn(&str) -> Option<ToolMetadata>,
{
    let metadata = metadata_for(name).ok_or(ToolGuardError::UnknownTool)?;
    guard_operation_metadata(metadata, context)?;
    Ok(metadata)
}

pub fn guard_tool_metadata(
    metadata: ToolMetadata,
    context: &AppContext,
) -> Result<(), ToolGuardError> {
    if !is_tool_enabled(metadata, context) {
        return Err(ToolGuardError::DisabledTool);
    }

    if !is_service_available(metadata, context) {
        return Err(ToolGuardError::ServiceUnavailable);
    }

    Ok(())
}

pub fn guard_operation_metadata(
    metadata: ToolMetadata,
    context: &AppContext,
) -> Result<(), ToolGuardError> {
    if !is_service_available(metadata, context) {
        return Err(ToolGuardError::ServiceUnavailable);
    }

    Ok(())
}

fn is_discoverable(metadata: ToolMetadata, context: &AppContext) -> bool {
    is_tool_enabled(metadata, context) && is_service_available(metadata, context)
}

fn is_tool_enabled(metadata: ToolMetadata, context: &AppContext) -> bool {
    if context.mcp_disabled_tools().contains(metadata.name) {
        return false;
    }

    context
        .mcp_enabled_tools()
        .is_some_and(|enabled_tools| enabled_tools.contains(metadata.name))
        || is_toolset_enabled(metadata, context)
}

fn is_service_available(metadata: ToolMetadata, context: &AppContext) -> bool {
    let availability = context.service_availability();

    match metadata.service {
        ToolService::Jira => availability.jira,
        ToolService::Confluence => availability.confluence,
        ToolService::Gitlab => availability.gitlab,
    }
}

fn is_toolset_enabled(metadata: ToolMetadata, context: &AppContext) -> bool {
    match metadata.toolset {
        Some(toolset) => context.mcp_enabled_toolsets().contains(toolset),
        None => true,
    }
}

fn tool_with_metadata(mut tool: Tool, metadata: ToolMetadata) -> Tool {
    tool.title = Some(metadata.title.to_string());
    tool.description = Some(Cow::Borrowed(metadata.description));
    tool.annotations = Some(tool_annotations(metadata));
    if let Some(output_schema) = metadata.output_schema {
        tool.output_schema = Some(Arc::new(tool_output_schema(output_schema)));
    }
    tool
}

fn tool_annotations(metadata: ToolMetadata) -> ToolAnnotations {
    let annotations = metadata.annotations;
    ToolAnnotations::with_title(metadata.title)
        .read_only(annotations.read_only)
        .destructive(annotations.destructive)
        .idempotent(annotations.idempotent)
        .open_world(annotations.open_world)
}

fn tool_output_schema(output_schema: ToolOutputSchema) -> JsonObject {
    match output_schema {
        ToolOutputSchema::JiraMutationResult => jira_mutation_result_schema(),
        ToolOutputSchema::JiraCreateIssuesResult => jira_create_issues_schema(),
        ToolOutputSchema::JiraPartitionedVersionsResult => jira_partitioned_versions_schema(),
        ToolOutputSchema::JiraProductDependencyResult => jira_product_dependency_schema(),
        ToolOutputSchema::JiraIssueAttachmentsResult => jira_issue_attachments_schema(false),
        ToolOutputSchema::JiraIssueImageAttachmentsResult => jira_issue_attachments_schema(true),
        ToolOutputSchema::ConfluenceBatchAttachmentUploadResult => {
            confluence_batch_attachment_upload_schema()
        }
        ToolOutputSchema::ConfluenceAttachmentDownloadResult => {
            confluence_attachment_download_schema()
        }
        ToolOutputSchema::ConfluenceBatchAttachmentDownloadResult => {
            confluence_batch_attachment_download_schema()
        }
        ToolOutputSchema::ConfluenceAnalyticsResult => confluence_analytics_schema(),
    }
}

fn schema_object(value: Value) -> JsonObject {
    match value {
        Value::Object(object) => object,
        _ => unreachable!("output schema helpers must return JSON objects"),
    }
}

fn jira_mutation_result_schema() -> JsonObject {
    schema_object(json!({
        "type": "object",
        "description": "structuredContent for Jira mutation tools that return success plus optional Jira REST data.",
        "required": ["success"],
        "properties": {
            "success": { "type": "boolean" },
            "data": {
                "description": "Jira REST response payload; may be null for no-content responses.",
                "anyOf": [
                    { "type": "null" },
                    { "type": "object", "additionalProperties": true },
                    { "type": "array", "items": true },
                    { "type": "string" },
                    { "type": "number" },
                    { "type": "boolean" }
                ]
            },
            "message": { "type": "string" }
        },
        "additionalProperties": true
    }))
}

fn jira_create_issues_schema() -> JsonObject {
    schema_object(json!({
        "type": "object",
        "description": "structuredContent for bulk Jira issue creation.",
        "required": ["success", "data"],
        "properties": {
            "success": { "type": "boolean" },
            "data": {
                "type": "object",
                "properties": {
                    "issues": { "type": "array", "items": { "type": "object", "additionalProperties": true } },
                    "errors": { "type": "array", "items": { "type": "object", "additionalProperties": true } }
                },
                "additionalProperties": true
            }
        },
        "additionalProperties": true
    }))
}

fn jira_partitioned_versions_schema() -> JsonObject {
    schema_object(json!({
        "type": "object",
        "description": "structuredContent for Jira project-version batch creation with per-item success partitions.",
        "required": ["versions"],
        "properties": {
            "versions": {
                "type": "array",
                "items": {
                    "type": "object",
                    "required": ["success"],
                    "properties": {
                        "success": { "type": "boolean" },
                        "version": { "type": "object", "additionalProperties": true },
                        "error": { "type": "string" }
                    },
                    "additionalProperties": true
                }
            }
        },
        "additionalProperties": true
    }))
}

fn jira_product_dependency_schema() -> JsonObject {
    schema_object(json!({
        "type": "object",
        "description": "structuredContent for Jira product-dependent tools when an optional Jira product or API is unavailable.",
        "properties": {
            "success": { "type": "boolean" },
            "product_dependency": {
                "type": "object",
                "properties": {
                    "product": { "type": "string" },
                    "available": { "type": "boolean" },
                    "message": { "type": "string" }
                },
                "additionalProperties": true
            }
        },
        "additionalProperties": true
    }))
}

fn jira_issue_attachments_schema(images_only: bool) -> JsonObject {
    schema_object(json!({
        "type": "object",
        "description": "structuredContent for bounded Jira issue attachment downloads.",
        "required": ["issue_key", "count", "attachments"],
        "properties": {
            "issue_key": { "type": "string" },
            "count": { "type": "integer" },
            "images_only": { "type": "boolean", "const": images_only },
            "attachments": {
                "type": "array",
                "items": attachment_item_schema()
            }
        },
        "additionalProperties": true
    }))
}

fn attachment_item_schema() -> Value {
    json!({
        "type": "object",
        "properties": {
            "id": { "type": "string" },
            "filename": { "type": "string" },
            "media_type": { "type": "string" },
            "file_size": { "type": "integer" },
            "is_image": { "type": "boolean" },
            "content": {
                "type": "object",
                "properties": {
                    "encoding": { "type": "string" },
                    "content_type": { "type": "string" },
                    "size": { "type": "integer" },
                    "data": { "type": "string" }
                },
                "additionalProperties": true
            },
            "content_error": {
                "type": "object",
                "properties": {
                    "message": { "type": "string" }
                },
                "additionalProperties": true
            }
        },
        "additionalProperties": true
    })
}

fn confluence_batch_attachment_upload_schema() -> JsonObject {
    schema_object(json!({
        "type": "object",
        "description": "structuredContent for batch Confluence attachment uploads, including partial success details.",
        "required": ["success", "partial_success", "summary", "attachments", "failed"],
        "properties": {
            "success": { "type": "boolean" },
            "partial_success": { "type": "boolean" },
            "summary": batch_summary_schema(),
            "attachments": { "type": "array", "items": attachment_item_schema() },
            "failed": { "type": "array", "items": failed_attachment_item_schema() }
        },
        "additionalProperties": true
    }))
}

fn confluence_attachment_download_schema() -> JsonObject {
    schema_object(json!({
        "type": "object",
        "description": "structuredContent for a bounded Confluence attachment download.",
        "required": ["success"],
        "properties": {
            "success": { "type": "boolean" },
            "attachment": attachment_item_schema(),
            "error": { "type": "string" }
        },
        "additionalProperties": true
    }))
}

fn confluence_batch_attachment_download_schema() -> JsonObject {
    schema_object(json!({
        "type": "object",
        "description": "structuredContent for bounded Confluence content attachment downloads.",
        "required": ["success", "summary", "attachments", "failed"],
        "properties": {
            "success": { "type": "boolean" },
            "summary": batch_summary_schema(),
            "attachments": { "type": "array", "items": attachment_item_schema() },
            "failed": { "type": "array", "items": failed_attachment_item_schema() }
        },
        "additionalProperties": true
    }))
}

fn confluence_analytics_schema() -> JsonObject {
    schema_object(json!({
        "type": "object",
        "description": "structuredContent for Confluence page-view analytics or Cloud-only availability failures.",
        "properties": {
            "success": { "type": "boolean" },
            "available": { "type": "boolean" },
            "page_id": { "type": "string" },
            "total_views": { "type": "integer" },
            "views": { "type": "array", "items": { "type": "object", "additionalProperties": true } },
            "status": { "type": "integer" },
            "error": { "type": "string" },
            "details": { "type": "string" }
        },
        "additionalProperties": true
    }))
}

fn batch_summary_schema() -> Value {
    json!({
        "type": "object",
        "properties": {
            "total": { "type": "integer" },
            "uploaded": { "type": "integer" },
            "downloaded": { "type": "integer" },
            "failed": { "type": "integer" },
            "pages_fetched": { "type": "integer" },
            "has_more": { "type": "boolean" },
            "limit_applied": { "type": "boolean" }
        },
        "additionalProperties": true
    })
}

fn failed_attachment_item_schema() -> Value {
    json!({
        "type": "object",
        "properties": {
            "filename": { "type": "string" },
            "attachment_id": { "type": "string" },
            "error": { "type": "string" }
        },
        "additionalProperties": true
    })
}

fn tool_unavailable_error() -> ErrorData {
    ErrorData::invalid_params(TOOL_UNAVAILABLE_MESSAGE, None)
}

#[cfg(test)]
mod tests;

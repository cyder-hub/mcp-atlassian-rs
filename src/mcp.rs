use std::sync::Arc;

use crate::{
    atlassian::error::AtlassianError,
    context::AppContext,
    jira::{
        client::{
            AttachmentFetchOptions, DEFAULT_ATTACHMENT_MAX_BYTES, FieldOptionsRequest,
            GetIssueRequest, JiraClient, SearchRequest,
        },
        config::JiraDeployment,
        formatting::{
            comment_body_for_deployment, merge_optional_objects, parse_optional_object,
            parse_optional_string_list, parse_required_object, parse_required_object_list,
            parse_required_string_list,
        },
        tools::{
            JiraAddCommentArgs, JiraAddIssuesToSprintArgs, JiraAddWatcherArgs, JiraAddWorklogArgs,
            JiraBatchCreateIssuesArgs, JiraBatchCreateVersionsArgs, JiraBatchGetChangelogsArgs,
            JiraCreateIssueArgs, JiraCreateIssueLinkArgs, JiraCreateRemoteIssueLinkArgs,
            JiraCreateSprintArgs, JiraCreateVersionArgs, JiraDeleteIssueArgs,
            JiraDownloadAttachmentsArgs, JiraEditCommentArgs, JiraGetAgileBoardsArgs,
            JiraGetAllProjectsArgs, JiraGetBoardIssuesArgs, JiraGetFieldOptionsArgs,
            JiraGetIssueArgs, JiraGetIssueDatesArgs, JiraGetIssueDevelopmentInfoArgs,
            JiraGetIssueImagesArgs, JiraGetIssueProformaFormsArgs, JiraGetIssueSlaArgs,
            JiraGetIssueWatchersArgs, JiraGetIssuesDevelopmentInfoArgs, JiraGetLinkTypesArgs,
            JiraGetProformaFormDetailsArgs, JiraGetProjectComponentsArgs, JiraGetProjectIssuesArgs,
            JiraGetProjectVersionsArgs, JiraGetQueueIssuesArgs, JiraGetServiceDeskForProjectArgs,
            JiraGetServiceDeskQueuesArgs, JiraGetSprintIssuesArgs, JiraGetSprintsFromBoardArgs,
            JiraGetTransitionsArgs, JiraGetUserProfileArgs, JiraGetWorklogArgs, JiraLinkToEpicArgs,
            JiraRemoveIssueLinkArgs, JiraRemoveWatcherArgs, JiraSearchArgs, JiraSearchFieldsArgs,
            JiraTransitionIssueArgs, JiraUpdateIssueArgs, JiraUpdateProformaFormAnswersArgs,
            JiraUpdateSprintArgs,
        },
    },
    tool_registry,
};
use rmcp::{
    ErrorData, RoleServer, ServerHandler,
    handler::server::router::tool::ToolRouter,
    handler::server::tool::ToolCallContext,
    handler::server::wrapper::Parameters,
    model::{
        CallToolRequestParams, CallToolResult, Implementation, ListToolsResult,
        PaginatedRequestParams, ServerCapabilities, ServerInfo, Tool,
    },
    service::RequestContext,
    tool, tool_handler, tool_router,
};
use serde_json::{Map, Value, json};

pub const SERVER_NAME: &str = "mcp-atlassian-rs";

const MIGRATION_STATUS: &str = "mcp-atlassian-rs Stage 2 Jira core migration is complete. \
The Stage 1 MCP runtime/control plane and Stage 2 Jira config/auth/client/models/tool handlers are implemented. \
Jira core tools are available when Jira configuration and authentication are complete. \
Mock REST tests, MCP smoke checks, README, and the migration ledger are up to date.";

#[derive(Clone)]
pub struct AtlassianMcpServer {
    context: Arc<AppContext>,
    tool_router: ToolRouter<Self>,
}

impl AtlassianMcpServer {
    pub fn new(context: Arc<AppContext>) -> Self {
        Self {
            context,
            tool_router: Self::tool_router(),
        }
    }

    fn current_tools_result(&self) -> ListToolsResult {
        ListToolsResult {
            tools: self.filtered_tools_from(self.tool_router.list_all()),
            ..Default::default()
        }
    }

    fn filtered_tools_from<I>(&self, tools: I) -> Vec<Tool>
    where
        I: IntoIterator<Item = Tool>,
    {
        tool_registry::visible_tools(tools, &self.context)
    }

    fn guard_registered_tool_call(&self, name: &str) -> Result<(), ErrorData> {
        if !self.tool_router.has_route(name) {
            return Err(ErrorData::invalid_params("tool not available", None));
        }

        tool_registry::guard_tool_call(name, &self.context)
    }

    fn jira_client(&self) -> Result<JiraClient, ErrorData> {
        let Some(config) = self.context.jira_config() else {
            return Err(ErrorData::invalid_params("Jira is not configured", None));
        };

        JiraClient::new(config.clone()).map_err(jira_error)
    }

    #[cfg(test)]
    fn guard_tool_call_with_metadata<F>(
        &self,
        name: &str,
        route_exists: bool,
        metadata_for: F,
    ) -> Result<(), ErrorData>
    where
        F: Fn(&str) -> Option<tool_registry::ToolMetadata>,
    {
        if !route_exists {
            return Err(ErrorData::invalid_params("tool not available", None));
        }

        tool_registry::guard_tool_call_with_metadata(name, &self.context, metadata_for)
    }

    #[cfg(test)]
    fn filtered_tools_from_with_metadata<I, F>(&self, tools: I, metadata_for: F) -> Vec<Tool>
    where
        I: IntoIterator<Item = Tool>,
        F: Fn(&str) -> Option<tool_registry::ToolMetadata>,
    {
        tool_registry::visible_tools_with_metadata(tools, &self.context, metadata_for)
    }
}

impl Default for AtlassianMcpServer {
    fn default() -> Self {
        Self::new(Arc::new(AppContext::default()))
    }
}

#[tool_router(router = tool_router)]
impl AtlassianMcpServer {
    #[tool(description = "Report the current Rust migration status for MCP Atlassian")]
    fn migration_status(&self) -> String {
        MIGRATION_STATUS.to_string()
    }

    #[tool(description = "Get a Jira issue by key")]
    async fn jira_get_issue(
        &self,
        Parameters(args): Parameters<JiraGetIssueArgs>,
    ) -> Result<CallToolResult, ErrorData> {
        let fields = parse_optional_string_list_arg(args.fields, "fields")?;
        let expand = parse_optional_string_list_arg(args.expand, "expand")?;
        let properties = parse_optional_string_list_arg(args.properties, "properties")?;
        let value = self
            .jira_client()?
            .get_issue(GetIssueRequest {
                issue_key: args.issue_key,
                fields,
                expand,
                comment_limit: args.comment_limit,
                properties,
                update_history: args.update_history,
            })
            .await
            .map_err(jira_error)?;

        Ok(CallToolResult::structured(value))
    }

    #[tool(description = "Search Jira issues with JQL")]
    async fn jira_search(
        &self,
        Parameters(args): Parameters<JiraSearchArgs>,
    ) -> Result<CallToolResult, ErrorData> {
        let fields = parse_optional_string_list_arg(args.fields, "fields")?;
        let expand = parse_optional_string_list_arg(args.expand, "expand")?;
        let projects_filter =
            parse_optional_string_list_arg(args.projects_filter, "projects_filter")?;
        let value = self
            .jira_client()?
            .search(SearchRequest {
                jql: args.jql,
                fields,
                limit: args.limit,
                start_at: args.start_at,
                projects_filter,
                expand,
                page_token: args.page_token,
            })
            .await
            .map_err(jira_error)?;

        Ok(CallToolResult::structured(value))
    }

    #[tool(description = "List Jira issues for a project")]
    async fn jira_get_project_issues(
        &self,
        Parameters(args): Parameters<JiraGetProjectIssuesArgs>,
    ) -> Result<CallToolResult, ErrorData> {
        let value = self
            .jira_client()?
            .get_project_issues(args.project_key, args.limit, args.start_at)
            .await
            .map_err(jira_error)?;

        Ok(CallToolResult::structured(value))
    }

    #[tool(description = "Search Jira fields by keyword")]
    async fn jira_search_fields(
        &self,
        Parameters(args): Parameters<JiraSearchFieldsArgs>,
    ) -> Result<CallToolResult, ErrorData> {
        let value = self
            .jira_client()?
            .search_fields(args.keyword, args.limit)
            .await
            .map_err(jira_error)?;

        Ok(CallToolResult::structured(value))
    }

    #[tool(description = "Get options for a Jira field")]
    async fn jira_get_field_options(
        &self,
        Parameters(args): Parameters<JiraGetFieldOptionsArgs>,
    ) -> Result<CallToolResult, ErrorData> {
        let value = self
            .jira_client()?
            .get_field_options(FieldOptionsRequest {
                field_id: args.field_id,
                context_id: args.context_id,
                project_key: args.project_key,
                issue_type: args.issue_type,
                contains: args.contains,
                return_limit: args.return_limit,
                values_only: args.values_only.unwrap_or(false),
            })
            .await
            .map_err(jira_error)?;

        Ok(CallToolResult::structured(value))
    }

    #[tool(description = "Add a comment to a Jira issue")]
    async fn jira_add_comment(
        &self,
        Parameters(args): Parameters<JiraAddCommentArgs>,
    ) -> Result<CallToolResult, ErrorData> {
        let visibility = parse_optional_object_arg(args.visibility, "visibility")?;
        let value = self
            .jira_client()?
            .add_comment(args.issue_key, args.body, visibility)
            .await
            .map_err(jira_error)?;

        Ok(CallToolResult::structured(value))
    }

    #[tool(description = "Edit a Jira issue comment")]
    async fn jira_edit_comment(
        &self,
        Parameters(args): Parameters<JiraEditCommentArgs>,
    ) -> Result<CallToolResult, ErrorData> {
        let visibility = parse_optional_object_arg(args.visibility, "visibility")?;
        let value = self
            .jira_client()?
            .edit_comment(args.issue_key, args.comment_id, args.body, visibility)
            .await
            .map_err(jira_error)?;

        Ok(CallToolResult::structured(value))
    }

    #[tool(description = "Get available transitions for a Jira issue")]
    async fn jira_get_transitions(
        &self,
        Parameters(args): Parameters<JiraGetTransitionsArgs>,
    ) -> Result<CallToolResult, ErrorData> {
        let value = self
            .jira_client()?
            .get_transitions(args.issue_key)
            .await
            .map_err(jira_error)?;

        Ok(CallToolResult::structured(value))
    }

    #[tool(description = "Transition a Jira issue")]
    async fn jira_transition_issue(
        &self,
        Parameters(args): Parameters<JiraTransitionIssueArgs>,
    ) -> Result<CallToolResult, ErrorData> {
        let fields = parse_optional_object_arg(args.fields, "fields")?;
        let value = self
            .jira_client()?
            .transition_issue(args.issue_key, args.transition_id, fields, args.comment)
            .await
            .map_err(jira_error)?;

        Ok(CallToolResult::structured(value))
    }

    #[tool(description = "Create a Jira issue")]
    async fn jira_create_issue(
        &self,
        Parameters(args): Parameters<JiraCreateIssueArgs>,
    ) -> Result<CallToolResult, ErrorData> {
        let deployment = self
            .context
            .jira_config()
            .ok_or_else(|| ErrorData::invalid_params("Jira is not configured", None))?
            .deployment;
        let fields = create_issue_fields_from_args(args, deployment)?;
        let value = self
            .jira_client()?
            .create_issue(fields)
            .await
            .map_err(jira_error)?;

        Ok(CallToolResult::structured(value))
    }

    #[tool(description = "Create multiple Jira issues in a batch")]
    async fn jira_batch_create_issues(
        &self,
        Parameters(args): Parameters<JiraBatchCreateIssuesArgs>,
    ) -> Result<CallToolResult, ErrorData> {
        let deployment = self
            .context
            .jira_config()
            .ok_or_else(|| ErrorData::invalid_params("Jira is not configured", None))?
            .deployment;
        let issue_updates = batch_create_issue_updates_from_args(args.issues, deployment)?;
        let value = self
            .jira_client()?
            .batch_create_issues(issue_updates, args.validate_only.unwrap_or(false))
            .await
            .map_err(jira_error)?;

        Ok(CallToolResult::structured(value))
    }

    #[tool(description = "Get changelogs for multiple Jira issues")]
    async fn jira_batch_get_changelogs(
        &self,
        Parameters(args): Parameters<JiraBatchGetChangelogsArgs>,
    ) -> Result<CallToolResult, ErrorData> {
        let issue_ids_or_keys =
            parse_required_string_list_arg(args.issue_ids_or_keys, "issue_ids_or_keys")?;
        let fields = parse_optional_string_list_arg(args.fields, "fields")?;
        let limit = optional_positive_i64_arg(args.limit, "limit")?;
        let value = self
            .jira_client()?
            .batch_get_changelogs(issue_ids_or_keys, fields, limit)
            .await
            .map_err(jira_error)?;

        Ok(CallToolResult::structured(value))
    }

    #[tool(description = "Update fields on a Jira issue")]
    async fn jira_update_issue(
        &self,
        Parameters(args): Parameters<JiraUpdateIssueArgs>,
    ) -> Result<CallToolResult, ErrorData> {
        let deployment = self
            .context
            .jira_config()
            .ok_or_else(|| ErrorData::invalid_params("Jira is not configured", None))?
            .deployment;
        let (fields, additional_fields) = update_issue_fields_from_args(args, deployment)?;
        let value = self
            .jira_client()?
            .update_issue(
                fields.issue_key,
                fields.fields,
                additional_fields,
                fields.notify_users,
            )
            .await
            .map_err(jira_error)?;

        Ok(CallToolResult::structured(value))
    }

    #[tool(description = "Delete a Jira issue")]
    async fn jira_delete_issue(
        &self,
        Parameters(args): Parameters<JiraDeleteIssueArgs>,
    ) -> Result<CallToolResult, ErrorData> {
        let value = self
            .jira_client()?
            .delete_issue(args.issue_key, args.delete_subtasks.unwrap_or(false))
            .await
            .map_err(jira_error)?;

        Ok(CallToolResult::structured(value))
    }

    #[tool(description = "List Jira projects visible to the current user")]
    async fn jira_get_all_projects(
        &self,
        Parameters(args): Parameters<JiraGetAllProjectsArgs>,
    ) -> Result<CallToolResult, ErrorData> {
        let value = self
            .jira_client()?
            .get_all_projects(args.include_archived.unwrap_or(false))
            .await
            .map_err(jira_error)?;

        Ok(CallToolResult::structured(value))
    }

    #[tool(description = "List versions for a Jira project")]
    async fn jira_get_project_versions(
        &self,
        Parameters(args): Parameters<JiraGetProjectVersionsArgs>,
    ) -> Result<CallToolResult, ErrorData> {
        let value = self
            .jira_client()?
            .get_project_versions(args.project_key)
            .await
            .map_err(jira_error)?;

        Ok(CallToolResult::structured(value))
    }

    #[tool(description = "List components for a Jira project")]
    async fn jira_get_project_components(
        &self,
        Parameters(args): Parameters<JiraGetProjectComponentsArgs>,
    ) -> Result<CallToolResult, ErrorData> {
        let value = self
            .jira_client()?
            .get_project_components(args.project_key)
            .await
            .map_err(jira_error)?;

        Ok(CallToolResult::structured(value))
    }

    #[tool(description = "Create a Jira project version")]
    async fn jira_create_version(
        &self,
        Parameters(args): Parameters<JiraCreateVersionArgs>,
    ) -> Result<CallToolResult, ErrorData> {
        let value = self
            .jira_client()?
            .create_version(version_payload_from_args(args)?)
            .await
            .map_err(jira_error)?;

        Ok(CallToolResult::structured(value))
    }

    #[tool(description = "Create multiple Jira project versions")]
    async fn jira_batch_create_versions(
        &self,
        Parameters(args): Parameters<JiraBatchCreateVersionsArgs>,
    ) -> Result<CallToolResult, ErrorData> {
        let project_key = required_non_empty_arg(args.project_key, "project_key")?;
        let versions = parse_required_object_list_arg(args.versions, "versions")?
            .into_iter()
            .map(|version| version_payload_from_value(version, &project_key))
            .collect::<Result<Vec<_>, _>>()?;
        let value = self
            .jira_client()?
            .batch_create_versions(versions)
            .await
            .map_err(jira_error)?;

        Ok(CallToolResult::structured(value))
    }

    #[tool(description = "Retrieve a Jira user profile")]
    async fn jira_get_user_profile(
        &self,
        Parameters(args): Parameters<JiraGetUserProfileArgs>,
    ) -> Result<CallToolResult, ErrorData> {
        let value = self
            .jira_client()?
            .get_user_profile(required_non_empty_arg(
                args.user_identifier,
                "user_identifier",
            )?)
            .await
            .map_err(jira_error)?;

        Ok(CallToolResult::structured(value))
    }

    #[tool(description = "Get watchers for a Jira issue")]
    async fn jira_get_issue_watchers(
        &self,
        Parameters(args): Parameters<JiraGetIssueWatchersArgs>,
    ) -> Result<CallToolResult, ErrorData> {
        let value = self
            .jira_client()?
            .get_issue_watchers(args.issue_key)
            .await
            .map_err(jira_error)?;

        Ok(CallToolResult::structured(value))
    }

    #[tool(description = "Add a watcher to a Jira issue")]
    async fn jira_add_watcher(
        &self,
        Parameters(args): Parameters<JiraAddWatcherArgs>,
    ) -> Result<CallToolResult, ErrorData> {
        let value = self
            .jira_client()?
            .add_watcher(
                args.issue_key,
                required_non_empty_arg(args.user_identifier, "user_identifier")?,
            )
            .await
            .map_err(jira_error)?;

        Ok(CallToolResult::structured(value))
    }

    #[tool(description = "Remove a watcher from a Jira issue")]
    async fn jira_remove_watcher(
        &self,
        Parameters(args): Parameters<JiraRemoveWatcherArgs>,
    ) -> Result<CallToolResult, ErrorData> {
        let value = self
            .jira_client()?
            .remove_watcher(
                args.issue_key,
                required_non_empty_arg(args.user_identifier, "user_identifier")?,
            )
            .await
            .map_err(jira_error)?;

        Ok(CallToolResult::structured(value))
    }

    #[tool(description = "Get worklogs for a Jira issue")]
    async fn jira_get_worklog(
        &self,
        Parameters(args): Parameters<JiraGetWorklogArgs>,
    ) -> Result<CallToolResult, ErrorData> {
        let value = self
            .jira_client()?
            .get_worklog(args.issue_key, args.start_at, args.limit)
            .await
            .map_err(jira_error)?;

        Ok(CallToolResult::structured(value))
    }

    #[tool(description = "Add a worklog entry to a Jira issue")]
    async fn jira_add_worklog(
        &self,
        Parameters(args): Parameters<JiraAddWorklogArgs>,
    ) -> Result<CallToolResult, ErrorData> {
        let deployment = self
            .context
            .jira_config()
            .ok_or_else(|| ErrorData::invalid_params("Jira is not configured", None))?
            .deployment;
        let (issue_key, payload, query) = add_worklog_payload_from_args(args, deployment)?;
        let value = self
            .jira_client()?
            .add_worklog(issue_key, payload, query)
            .await
            .map_err(jira_error)?;

        Ok(CallToolResult::structured(value))
    }

    #[tool(description = "Get Jira issue link types")]
    async fn jira_get_link_types(
        &self,
        Parameters(args): Parameters<JiraGetLinkTypesArgs>,
    ) -> Result<CallToolResult, ErrorData> {
        let mut value = self
            .jira_client()?
            .get_link_types()
            .await
            .map_err(jira_error)?;

        if let Some(name_filter) = optional_non_empty_arg(args.name_filter) {
            let name_filter = name_filter.to_lowercase();
            if let Some(link_types) = value
                .get_mut("issueLinkTypes")
                .and_then(Value::as_array_mut)
            {
                link_types.retain(|link_type| {
                    link_type
                        .get("name")
                        .and_then(Value::as_str)
                        .is_some_and(|name| name.to_lowercase().contains(&name_filter))
                });
            }
        }

        Ok(CallToolResult::structured(value))
    }

    #[tool(description = "Link a Jira issue to an epic using parent key")]
    async fn jira_link_to_epic(
        &self,
        Parameters(args): Parameters<JiraLinkToEpicArgs>,
    ) -> Result<CallToolResult, ErrorData> {
        let value = self
            .jira_client()?
            .link_to_epic(
                required_non_empty_arg(args.issue_key, "issue_key")?,
                required_non_empty_arg(args.epic_key, "epic_key")?,
            )
            .await
            .map_err(jira_error)?;

        Ok(CallToolResult::structured(value))
    }

    #[tool(description = "Create a link between two Jira issues")]
    async fn jira_create_issue_link(
        &self,
        Parameters(args): Parameters<JiraCreateIssueLinkArgs>,
    ) -> Result<CallToolResult, ErrorData> {
        let deployment = self
            .context
            .jira_config()
            .ok_or_else(|| ErrorData::invalid_params("Jira is not configured", None))?
            .deployment;
        let value = self
            .jira_client()?
            .create_issue_link(issue_link_payload_from_args(args, deployment)?)
            .await
            .map_err(jira_error)?;

        Ok(CallToolResult::structured(value))
    }

    #[tool(description = "Create a remote link on a Jira issue")]
    async fn jira_create_remote_issue_link(
        &self,
        Parameters(args): Parameters<JiraCreateRemoteIssueLinkArgs>,
    ) -> Result<CallToolResult, ErrorData> {
        let (issue_key, payload) = remote_issue_link_payload_from_args(args)?;
        let value = self
            .jira_client()?
            .create_remote_issue_link(issue_key, payload)
            .await
            .map_err(jira_error)?;

        Ok(CallToolResult::structured(value))
    }

    #[tool(description = "Remove a Jira issue link by id")]
    async fn jira_remove_issue_link(
        &self,
        Parameters(args): Parameters<JiraRemoveIssueLinkArgs>,
    ) -> Result<CallToolResult, ErrorData> {
        let value = self
            .jira_client()?
            .remove_issue_link(required_non_empty_arg(args.link_id, "link_id")?)
            .await
            .map_err(jira_error)?;

        Ok(CallToolResult::structured(value))
    }

    #[tool(description = "Download Jira issue attachments with bounded safe content output")]
    async fn jira_download_attachments(
        &self,
        Parameters(args): Parameters<JiraDownloadAttachmentsArgs>,
    ) -> Result<CallToolResult, ErrorData> {
        let attachment_ids = parse_optional_string_list_arg(args.attachment_ids, "attachment_ids")?;
        let max_bytes = optional_positive_u64_arg(args.max_bytes, "max_bytes")?
            .unwrap_or(DEFAULT_ATTACHMENT_MAX_BYTES);
        let value = self
            .jira_client()?
            .get_safe_issue_attachments(
                required_non_empty_arg(args.issue_key, "issue_key")?,
                AttachmentFetchOptions {
                    attachment_ids,
                    include_content: args.include_content.unwrap_or(false),
                    images_only: false,
                    max_bytes,
                },
            )
            .await
            .map_err(jira_error)?;

        Ok(CallToolResult::structured(value))
    }

    #[tool(description = "Get image attachments for a Jira issue with safe content output")]
    async fn jira_get_issue_images(
        &self,
        Parameters(args): Parameters<JiraGetIssueImagesArgs>,
    ) -> Result<CallToolResult, ErrorData> {
        let max_bytes = optional_positive_u64_arg(args.max_bytes, "max_bytes")?
            .unwrap_or(DEFAULT_ATTACHMENT_MAX_BYTES);
        let value = self
            .jira_client()?
            .get_safe_issue_attachments(
                required_non_empty_arg(args.issue_key, "issue_key")?,
                AttachmentFetchOptions {
                    attachment_ids: None,
                    include_content: args.include_content.unwrap_or(false),
                    images_only: true,
                    max_bytes,
                },
            )
            .await
            .map_err(jira_error)?;

        Ok(CallToolResult::structured(value))
    }

    #[tool(description = "Get Jira Software agile boards")]
    async fn jira_get_agile_boards(
        &self,
        Parameters(args): Parameters<JiraGetAgileBoardsArgs>,
    ) -> Result<CallToolResult, ErrorData> {
        let value = self
            .jira_client()?
            .get_agile_boards(args.project_key, args.board_type, args.start_at, args.limit)
            .await
            .map_err(jira_error)?;

        Ok(CallToolResult::structured(value))
    }

    #[tool(description = "Get issues on a Jira Software agile board")]
    async fn jira_get_board_issues(
        &self,
        Parameters(args): Parameters<JiraGetBoardIssuesArgs>,
    ) -> Result<CallToolResult, ErrorData> {
        let fields = parse_optional_string_list_arg(args.fields, "fields")?;
        let value = self
            .jira_client()?
            .get_board_issues(args.board_id, args.jql, fields, args.start_at, args.limit)
            .await
            .map_err(jira_error)?;

        Ok(CallToolResult::structured(value))
    }

    #[tool(description = "Get sprints for a Jira Software agile board")]
    async fn jira_get_sprints_from_board(
        &self,
        Parameters(args): Parameters<JiraGetSprintsFromBoardArgs>,
    ) -> Result<CallToolResult, ErrorData> {
        let state = parse_optional_string_list_arg(args.state, "state")?;
        let value = self
            .jira_client()?
            .get_sprints_from_board(args.board_id, state, args.start_at, args.limit)
            .await
            .map_err(jira_error)?;

        Ok(CallToolResult::structured(value))
    }

    #[tool(description = "Get issues for a Jira Software sprint")]
    async fn jira_get_sprint_issues(
        &self,
        Parameters(args): Parameters<JiraGetSprintIssuesArgs>,
    ) -> Result<CallToolResult, ErrorData> {
        let fields = parse_optional_string_list_arg(args.fields, "fields")?;
        let value = self
            .jira_client()?
            .get_sprint_issues(args.sprint_id, fields, args.start_at, args.limit)
            .await
            .map_err(jira_error)?;

        Ok(CallToolResult::structured(value))
    }

    #[tool(description = "Create a Jira Software sprint")]
    async fn jira_create_sprint(
        &self,
        Parameters(args): Parameters<JiraCreateSprintArgs>,
    ) -> Result<CallToolResult, ErrorData> {
        let value = self
            .jira_client()?
            .create_sprint(create_sprint_payload_from_args(args)?)
            .await
            .map_err(jira_error)?;

        Ok(CallToolResult::structured(value))
    }

    #[tool(description = "Update a Jira Software sprint")]
    async fn jira_update_sprint(
        &self,
        Parameters(args): Parameters<JiraUpdateSprintArgs>,
    ) -> Result<CallToolResult, ErrorData> {
        let (sprint_id, payload) = update_sprint_payload_from_args(args)?;
        let value = self
            .jira_client()?
            .update_sprint(sprint_id, payload)
            .await
            .map_err(jira_error)?;

        Ok(CallToolResult::structured(value))
    }

    #[tool(description = "Add Jira issues to a sprint")]
    async fn jira_add_issues_to_sprint(
        &self,
        Parameters(args): Parameters<JiraAddIssuesToSprintArgs>,
    ) -> Result<CallToolResult, ErrorData> {
        let issue_keys = parse_required_string_list_arg(args.issue_keys, "issue_keys")?;
        let value = self
            .jira_client()?
            .add_issues_to_sprint(args.sprint_id, issue_keys)
            .await
            .map_err(jira_error)?;

        Ok(CallToolResult::structured(value))
    }

    #[tool(description = "Get the Jira Service Management service desk for a project")]
    async fn jira_get_service_desk_for_project(
        &self,
        Parameters(args): Parameters<JiraGetServiceDeskForProjectArgs>,
    ) -> Result<CallToolResult, ErrorData> {
        let value = self
            .jira_client()?
            .get_service_desk_for_project(required_non_empty_arg(args.project_key, "project_key")?)
            .await
            .map_err(jira_error)?;

        Ok(CallToolResult::structured(value))
    }

    #[tool(description = "Get queues for a Jira Service Management service desk")]
    async fn jira_get_service_desk_queues(
        &self,
        Parameters(args): Parameters<JiraGetServiceDeskQueuesArgs>,
    ) -> Result<CallToolResult, ErrorData> {
        let value = self
            .jira_client()?
            .get_service_desk_queues(args.service_desk_id, args.start_at, args.limit)
            .await
            .map_err(jira_error)?;

        Ok(CallToolResult::structured(value))
    }

    #[tool(description = "Get issues for a Jira Service Management queue")]
    async fn jira_get_queue_issues(
        &self,
        Parameters(args): Parameters<JiraGetQueueIssuesArgs>,
    ) -> Result<CallToolResult, ErrorData> {
        let value = self
            .jira_client()?
            .get_queue_issues(
                args.service_desk_id,
                args.queue_id,
                args.start_at,
                args.limit,
            )
            .await
            .map_err(jira_error)?;

        Ok(CallToolResult::structured(value))
    }

    #[tool(description = "Get Jira Forms or ProForma forms for an issue")]
    async fn jira_get_issue_proforma_forms(
        &self,
        Parameters(args): Parameters<JiraGetIssueProformaFormsArgs>,
    ) -> Result<CallToolResult, ErrorData> {
        let value = self
            .jira_client()?
            .get_issue_proforma_forms(
                required_non_empty_arg(args.issue_key, "issue_key")?,
                self.context.atlassian_oauth_cloud_id(),
            )
            .await
            .map_err(jira_error)?;

        Ok(CallToolResult::structured(value))
    }

    #[tool(description = "Get details for a Jira Form or ProForma form")]
    async fn jira_get_proforma_form_details(
        &self,
        Parameters(args): Parameters<JiraGetProformaFormDetailsArgs>,
    ) -> Result<CallToolResult, ErrorData> {
        let value = self
            .jira_client()?
            .get_proforma_form_details(
                required_non_empty_arg(args.issue_key, "issue_key")?,
                required_non_empty_arg(args.form_id, "form_id")?,
                self.context.atlassian_oauth_cloud_id(),
            )
            .await
            .map_err(jira_error)?;

        Ok(CallToolResult::structured(value))
    }

    #[tool(description = "Update answers on a Jira Form or ProForma form")]
    async fn jira_update_proforma_form_answers(
        &self,
        Parameters(args): Parameters<JiraUpdateProformaFormAnswersArgs>,
    ) -> Result<CallToolResult, ErrorData> {
        let answers = parse_required_object_list_arg(args.answers, "answers")?;
        let value = self
            .jira_client()?
            .update_proforma_form_answers(
                required_non_empty_arg(args.issue_key, "issue_key")?,
                required_non_empty_arg(args.form_id, "form_id")?,
                answers,
                self.context.atlassian_oauth_cloud_id(),
            )
            .await
            .map_err(jira_error)?;

        Ok(CallToolResult::structured(value))
    }

    #[tool(description = "Get Jira issue date and status timing information")]
    async fn jira_get_issue_dates(
        &self,
        Parameters(args): Parameters<JiraGetIssueDatesArgs>,
    ) -> Result<CallToolResult, ErrorData> {
        let value = self
            .jira_client()?
            .get_issue_dates(
                required_non_empty_arg(args.issue_key, "issue_key")?,
                args.include_status_changes.unwrap_or(false),
                args.include_status_summary.unwrap_or(false),
            )
            .await
            .map_err(jira_error)?;

        Ok(CallToolResult::structured(value))
    }

    #[tool(description = "Get Jira Service Management SLA metrics for an issue")]
    async fn jira_get_issue_sla(
        &self,
        Parameters(args): Parameters<JiraGetIssueSlaArgs>,
    ) -> Result<CallToolResult, ErrorData> {
        let metrics = parse_optional_string_list_arg(args.metrics, "metrics")?;
        let value = self
            .jira_client()?
            .get_issue_sla(
                required_non_empty_arg(args.issue_key, "issue_key")?,
                metrics,
                args.working_hours_only,
                args.include_raw_dates.unwrap_or(false),
            )
            .await
            .map_err(jira_error)?;

        Ok(CallToolResult::structured(value))
    }

    #[tool(description = "Get Jira development information for an issue")]
    async fn jira_get_issue_development_info(
        &self,
        Parameters(args): Parameters<JiraGetIssueDevelopmentInfoArgs>,
    ) -> Result<CallToolResult, ErrorData> {
        let value = self
            .jira_client()?
            .get_issue_development_info(
                required_non_empty_arg(args.issue_key, "issue_key")?,
                args.application_type,
                args.data_type,
            )
            .await
            .map_err(jira_error)?;

        Ok(CallToolResult::structured(value))
    }

    #[tool(description = "Get Jira development information for multiple issues")]
    async fn jira_get_issues_development_info(
        &self,
        Parameters(args): Parameters<JiraGetIssuesDevelopmentInfoArgs>,
    ) -> Result<CallToolResult, ErrorData> {
        let issue_keys = parse_required_string_list_arg(args.issue_keys, "issue_keys")?;
        let value = self
            .jira_client()?
            .get_issues_development_info(issue_keys, args.application_type, args.data_type)
            .await
            .map_err(jira_error)?;

        Ok(CallToolResult::structured(value))
    }
}

fn parse_optional_string_list_arg(
    value: Option<Value>,
    field_name: &'static str,
) -> Result<Option<Vec<String>>, ErrorData> {
    parse_optional_string_list(value, field_name).map_err(jira_error)
}

fn parse_required_string_list_arg(
    value: Value,
    field_name: &'static str,
) -> Result<Vec<String>, ErrorData> {
    parse_required_string_list(value, field_name).map_err(jira_error)
}

fn parse_optional_object_arg(
    value: Option<Value>,
    field_name: &'static str,
) -> Result<Option<Value>, ErrorData> {
    parse_optional_object(value, field_name).map_err(jira_error)
}

fn parse_required_object_arg(value: Value, field_name: &'static str) -> Result<Value, ErrorData> {
    parse_required_object(value, field_name).map_err(jira_error)
}

fn parse_required_object_list_arg(
    value: Value,
    field_name: &'static str,
) -> Result<Vec<Value>, ErrorData> {
    parse_required_object_list(value, field_name).map_err(jira_error)
}

fn create_issue_fields_from_args(
    args: JiraCreateIssueArgs,
    deployment: JiraDeployment,
) -> Result<Value, ErrorData> {
    let project_key = required_non_empty_arg(args.project_key, "project_key")?;
    let summary = required_non_empty_arg(args.summary, "summary")?;
    let issue_type = required_non_empty_arg(args.issue_type, "issue_type")?;
    let components = parse_optional_string_list_arg(args.components, "components")?;
    let additional_fields = parse_optional_object_arg(args.additional_fields, "additional_fields")?;
    let mut fields = json!({
        "project": {"key": project_key},
        "summary": summary,
        "issuetype": {"name": issue_type},
    });

    if let Some(description) = optional_non_empty_arg(args.description) {
        fields["description"] = comment_body_for_deployment(deployment, &description);
    }
    if let Some(assignee) = optional_non_empty_arg(args.assignee) {
        let identifier_field = match deployment {
            JiraDeployment::Cloud => "accountId",
            JiraDeployment::ServerDataCenter => "name",
        };
        fields["assignee"] = json!({ identifier_field: assignee });
    }
    if let Some(components) = components {
        let components = components
            .into_iter()
            .map(|name| json!({ "name": name }))
            .collect::<Vec<_>>();
        if !components.is_empty() {
            fields["components"] = Value::Array(components);
        }
    }

    merge_optional_objects(fields, additional_fields, "additional_fields").map_err(jira_error)
}

struct UpdateIssueFields {
    issue_key: String,
    fields: Value,
    notify_users: Option<bool>,
}

fn update_issue_fields_from_args(
    args: JiraUpdateIssueArgs,
    deployment: JiraDeployment,
) -> Result<(UpdateIssueFields, Option<Value>), ErrorData> {
    let issue_key = required_non_empty_arg(args.issue_key, "issue_key")?;
    let fields = normalize_issue_fields(
        parse_required_object_arg(args.fields, "fields")?,
        deployment,
        "fields",
    )?;
    let components = parse_optional_string_list_arg(args.components, "components")?;
    let mut additional_fields =
        parse_optional_object_arg(args.additional_fields, "additional_fields")?
            .map(|value| normalize_issue_fields(value, deployment, "additional_fields"))
            .transpose()?;

    reject_unsupported_attachments(&fields, "fields")?;
    if let Some(additional_fields) = additional_fields.as_ref() {
        reject_unsupported_attachments(additional_fields, "additional_fields")?;
    }

    if let Some(components) = components {
        let components = components
            .into_iter()
            .map(|name| json!({ "name": name }))
            .collect::<Vec<_>>();
        if !components.is_empty() {
            let additional = additional_fields.get_or_insert_with(|| json!({}));
            additional["components"] = Value::Array(components);
        }
    }

    if fields.as_object().is_some_and(Map::is_empty) && additional_fields.is_none() {
        return Err(jira_error(AtlassianError::invalid_input(
            "fields must contain at least one update",
        )));
    }

    Ok((
        UpdateIssueFields {
            issue_key,
            fields,
            notify_users: args.notify_users,
        },
        additional_fields,
    ))
}

fn normalize_issue_fields(
    mut fields: Value,
    deployment: JiraDeployment,
    field_name: &'static str,
) -> Result<Value, ErrorData> {
    reject_unsupported_attachments(&fields, field_name)?;
    let object = fields.as_object_mut().ok_or_else(|| {
        jira_error(AtlassianError::invalid_input(format!(
            "{field_name} must be a JSON object"
        )))
    })?;

    if let Some(Value::String(description)) = object.get("description").cloned() {
        object.insert(
            "description".to_string(),
            comment_body_for_deployment(deployment, &description),
        );
    }
    if let Some(Value::String(assignee)) = object.get("assignee").cloned() {
        let identifier_field = match deployment {
            JiraDeployment::Cloud => "accountId",
            JiraDeployment::ServerDataCenter => "name",
        };
        object.insert(
            "assignee".to_string(),
            json!({ identifier_field: assignee }),
        );
    }

    Ok(fields)
}

fn reject_unsupported_attachments(
    value: &Value,
    field_name: &'static str,
) -> Result<(), ErrorData> {
    if value
        .as_object()
        .is_some_and(|object| object.contains_key("attachments"))
    {
        Err(jira_error(AtlassianError::invalid_input(format!(
            "{field_name}.attachments is not supported by jira_update_issue in Stage 3"
        ))))
    } else {
        Ok(())
    }
}

fn version_payload_from_args(args: JiraCreateVersionArgs) -> Result<Value, ErrorData> {
    let project_key = required_non_empty_arg(args.project_key, "project_key")?;
    let name = required_non_empty_arg(args.name, "name")?;
    let mut payload = json!({
        "project": project_key,
        "name": name,
    });
    insert_optional_value(
        &mut payload,
        "startDate",
        optional_non_empty_arg(args.start_date),
    );
    insert_optional_value(
        &mut payload,
        "releaseDate",
        optional_non_empty_arg(args.release_date),
    );
    insert_optional_value(
        &mut payload,
        "description",
        optional_non_empty_arg(args.description),
    );
    Ok(payload)
}

fn add_worklog_payload_from_args(
    args: JiraAddWorklogArgs,
    deployment: JiraDeployment,
) -> Result<(String, Value, Vec<(String, String)>), ErrorData> {
    let issue_key = required_non_empty_arg(args.issue_key, "issue_key")?;
    let time_spent = required_non_empty_arg(args.time_spent, "time_spent")?;
    let visibility = parse_optional_object_arg(args.visibility, "visibility")?;
    let mut payload = json!({
        "timeSpent": time_spent,
    });
    insert_optional_value(
        &mut payload,
        "started",
        optional_non_empty_arg(args.started),
    );
    if let Some(comment) = optional_non_empty_arg(args.comment) {
        payload["comment"] = comment_body_for_deployment(deployment, &comment);
    }
    if let Some(visibility) = visibility {
        payload["visibility"] = visibility;
    }

    let mut query = Vec::new();
    push_optional_query_value(&mut query, "adjustEstimate", args.adjust_estimate);
    push_optional_query_value(&mut query, "newEstimate", args.new_estimate);
    push_optional_query_value(&mut query, "reduceBy", args.reduce_by);
    Ok((issue_key, payload, query))
}

fn issue_link_payload_from_args(
    args: JiraCreateIssueLinkArgs,
    deployment: JiraDeployment,
) -> Result<Value, ErrorData> {
    let link_type = required_non_empty_arg(args.link_type, "link_type")?;
    let inward_issue_key = required_non_empty_arg(args.inward_issue_key, "inward_issue_key")?;
    let outward_issue_key = required_non_empty_arg(args.outward_issue_key, "outward_issue_key")?;
    let mut payload = json!({
        "type": {"name": link_type},
        "inwardIssue": {"key": inward_issue_key},
        "outwardIssue": {"key": outward_issue_key},
    });

    if let Some(comment) = optional_non_empty_arg(args.comment) {
        payload["comment"] = json!({
            "body": comment_body_for_deployment(deployment, &comment)
        });
    }

    Ok(payload)
}

fn remote_issue_link_payload_from_args(
    args: JiraCreateRemoteIssueLinkArgs,
) -> Result<(String, Value), ErrorData> {
    let issue_key = required_non_empty_arg(args.issue_key, "issue_key")?;
    let url = required_non_empty_arg(args.url, "url")?;
    let title = required_non_empty_arg(args.title, "title")?;
    let status = parse_optional_object_arg(args.status, "status")?;
    let mut object = json!({
        "url": url,
        "title": title,
    });
    insert_optional_value(&mut object, "summary", optional_non_empty_arg(args.summary));
    if let Some(icon_url) = optional_non_empty_arg(args.icon_url) {
        object["icon"] = json!({
            "url16x16": icon_url,
            "title": object["title"].clone(),
        });
    }
    if let Some(status) = status {
        object["status"] = status;
    }

    let mut payload = json!({ "object": object });
    insert_optional_value(
        &mut payload,
        "globalId",
        optional_non_empty_arg(args.global_id),
    );
    insert_optional_value(
        &mut payload,
        "relationship",
        optional_non_empty_arg(args.relationship),
    );
    Ok((issue_key, payload))
}

fn create_sprint_payload_from_args(args: JiraCreateSprintArgs) -> Result<Value, ErrorData> {
    let name = required_non_empty_arg(args.name, "name")?;
    let mut payload = json!({
        "name": name,
        "originBoardId": args.origin_board_id,
    });
    insert_optional_value(
        &mut payload,
        "startDate",
        optional_non_empty_arg(args.start_date),
    );
    insert_optional_value(
        &mut payload,
        "endDate",
        optional_non_empty_arg(args.end_date),
    );
    insert_optional_value(&mut payload, "goal", optional_non_empty_arg(args.goal));
    Ok(payload)
}

fn update_sprint_payload_from_args(args: JiraUpdateSprintArgs) -> Result<(u64, Value), ErrorData> {
    let mut payload = json!({});
    insert_optional_value(&mut payload, "name", optional_non_empty_arg(args.name));
    insert_optional_value(&mut payload, "state", optional_non_empty_arg(args.state));
    insert_optional_value(
        &mut payload,
        "startDate",
        optional_non_empty_arg(args.start_date),
    );
    insert_optional_value(
        &mut payload,
        "endDate",
        optional_non_empty_arg(args.end_date),
    );
    insert_optional_value(&mut payload, "goal", optional_non_empty_arg(args.goal));

    if payload.as_object().is_some_and(Map::is_empty) {
        return Err(jira_error(AtlassianError::invalid_input(
            "sprint update must contain at least one field",
        )));
    }

    Ok((args.sprint_id, payload))
}

fn version_payload_from_value(value: Value, project_key: &str) -> Result<Value, ErrorData> {
    let mut object = value_into_object(value, "version")?;
    let name = take_required_string_field(&mut object, "name")?;
    let start_date = take_optional_string_alias(&mut object, "startDate", "start_date")?;
    let release_date = take_optional_string_alias(&mut object, "releaseDate", "release_date")?;
    let description = take_optional_string_field(&mut object, "description")?;
    let mut payload = Value::Object(object);
    payload["project"] = Value::String(project_key.to_string());
    payload["name"] = Value::String(name);
    insert_optional_value(&mut payload, "startDate", start_date);
    insert_optional_value(&mut payload, "releaseDate", release_date);
    insert_optional_value(&mut payload, "description", description);
    Ok(payload)
}

fn take_optional_string_alias(
    object: &mut Map<String, Value>,
    first: &'static str,
    second: &'static str,
) -> Result<Option<String>, ErrorData> {
    match take_optional_string_field(object, first)? {
        Some(value) => Ok(Some(value)),
        None => take_optional_string_field(object, second),
    }
}

fn insert_optional_value(payload: &mut Value, key: &'static str, value: Option<String>) {
    if let Some(value) = value {
        payload[key] = Value::String(value);
    }
}

fn push_optional_query_value(
    query: &mut Vec<(String, String)>,
    key: &'static str,
    value: Option<String>,
) {
    if let Some(value) = optional_non_empty_arg(value) {
        query.push((key.to_string(), value));
    }
}

fn batch_create_issue_updates_from_args(
    issues: Value,
    deployment: JiraDeployment,
) -> Result<Vec<Value>, ErrorData> {
    parse_required_object_list_arg(issues, "issues")?
        .into_iter()
        .map(|issue| {
            create_issue_fields_from_value(issue, deployment)
                .map(|fields| json!({ "fields": fields }))
        })
        .collect()
}

fn create_issue_fields_from_value(
    issue: Value,
    deployment: JiraDeployment,
) -> Result<Value, ErrorData> {
    let mut fields = value_into_object(issue, "issue")?;
    let project_key = take_required_string_field(&mut fields, "project_key")?;
    let summary = take_required_string_field(&mut fields, "summary")?;
    let issue_type = take_required_string_field(&mut fields, "issue_type")?;
    let assignee = take_optional_string_field(&mut fields, "assignee")?;
    let description = take_optional_string_field(&mut fields, "description")?;
    let components = fields.remove("components");
    let additional_fields = if fields.is_empty() {
        None
    } else {
        Some(Value::Object(fields))
    };

    create_issue_fields_from_args(
        JiraCreateIssueArgs {
            project_key,
            summary,
            issue_type,
            assignee,
            description,
            components,
            additional_fields,
        },
        deployment,
    )
}

fn value_into_object(
    value: Value,
    field_name: &'static str,
) -> Result<Map<String, Value>, ErrorData> {
    match parse_required_object_arg(value, field_name)? {
        Value::Object(object) => Ok(object),
        _ => unreachable!("parse_required_object_arg only returns JSON objects"),
    }
}

fn take_required_string_field(
    object: &mut Map<String, Value>,
    field_name: &'static str,
) -> Result<String, ErrorData> {
    match object.remove(field_name) {
        Some(Value::String(value)) => required_non_empty_arg(value, field_name),
        Some(_) => Err(jira_error(AtlassianError::invalid_input(format!(
            "{field_name} must be a string"
        )))),
        None => Err(jira_error(AtlassianError::invalid_input(format!(
            "{field_name} is required"
        )))),
    }
}

fn take_optional_string_field(
    object: &mut Map<String, Value>,
    field_name: &'static str,
) -> Result<Option<String>, ErrorData> {
    match object.remove(field_name) {
        Some(Value::String(value)) => Ok(optional_non_empty_arg(Some(value))),
        Some(Value::Null) | None => Ok(None),
        Some(_) => Err(jira_error(AtlassianError::invalid_input(format!(
            "{field_name} must be a string"
        )))),
    }
}

fn required_non_empty_arg(value: String, field_name: &'static str) -> Result<String, ErrorData> {
    let value = value.trim();
    if value.is_empty() {
        Err(jira_error(AtlassianError::invalid_input(format!(
            "{field_name} must not be empty"
        ))))
    } else {
        Ok(value.to_string())
    }
}

fn optional_non_empty_arg(value: Option<String>) -> Option<String> {
    value
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
}

fn optional_positive_i64_arg(
    value: Option<i64>,
    field_name: &'static str,
) -> Result<Option<i64>, ErrorData> {
    match value {
        Some(value) if value <= 0 => Err(jira_error(AtlassianError::invalid_input(format!(
            "{field_name} must be positive"
        )))),
        value => Ok(value),
    }
}

fn optional_positive_u64_arg(
    value: Option<u64>,
    field_name: &'static str,
) -> Result<Option<u64>, ErrorData> {
    match value {
        Some(0) => Err(jira_error(AtlassianError::invalid_input(format!(
            "{field_name} must be positive"
        )))),
        value => Ok(value),
    }
}

fn jira_error(error: AtlassianError) -> ErrorData {
    match error {
        AtlassianError::InvalidInput { .. } => ErrorData::invalid_params(error.to_string(), None),
        _ => ErrorData::internal_error(error.to_string(), None),
    }
}

#[tool_handler(router = self.tool_router)]
impl ServerHandler for AtlassianMcpServer {
    async fn call_tool(
        &self,
        request: CallToolRequestParams,
        context: RequestContext<RoleServer>,
    ) -> Result<CallToolResult, ErrorData> {
        self.guard_registered_tool_call(request.name.as_ref())?;

        let tool_call_context = ToolCallContext::new(self, request, context);
        self.tool_router.call(tool_call_context).await
    }

    async fn list_tools(
        &self,
        _request: Option<PaginatedRequestParams>,
        _context: RequestContext<RoleServer>,
    ) -> Result<ListToolsResult, ErrorData> {
        Ok(self.current_tools_result())
    }

    fn get_tool(&self, name: &str) -> Option<Tool> {
        self.tool_router
            .get(name)
            .cloned()
            .filter(|tool| !self.filtered_tools_from([tool.clone()]).is_empty())
    }

    fn get_info(&self) -> ServerInfo {
        let access_mode = if self.context.read_only() {
            "read-only"
        } else {
            "read/write"
        };

        ServerInfo::new(ServerCapabilities::builder().enable_tools().build())
            .with_server_info(Implementation::new(SERVER_NAME, env!("CARGO_PKG_VERSION")))
            .with_instructions(format!(
                "Rust MCP Atlassian Stage 2 migration. The MCP control plane is initialized in {access_mode} mode. Jira core tools are available when Jira configuration and authentication are complete; Confluence tools are not migrated yet."
            ))
    }
}

#[cfg(test)]
mod tests {
    use std::{collections::BTreeSet, net::SocketAddr, sync::Arc};

    use crate::{
        atlassian::auth::AtlassianAuth,
        config::{HttpConfig, RuntimeConfig},
        context::AppContext,
        jira::config::{JiraConfig, JiraDeployment},
        jira::tools,
        tool_registry::{MIGRATION_STATUS_TOOL_NAME, ToolAccess, ToolMetadata, ToolService},
    };
    use axum::{
        Json, Router,
        body::Bytes,
        extract::State,
        http::{HeaderMap, Method, StatusCode},
        response::{IntoResponse, Response},
        routing::any,
    };
    use rmcp::model::{JsonObject, Tool};
    use rmcp::{ServerHandler, handler::server::wrapper::Parameters};
    use serde_json::{Value, json};
    use tokio::sync::Mutex;

    use super::*;

    fn server_with_config(config: RuntimeConfig) -> AtlassianMcpServer {
        AtlassianMcpServer::new(Arc::new(AppContext::from_config(&config)))
    }

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
            _ => tool_registry::metadata_for(name),
        }
    }

    fn runtime_config() -> RuntimeConfig {
        RuntimeConfig {
            http: HttpConfig::default(),
            ..RuntimeConfig::default()
        }
    }

    fn jira_config() -> JiraConfig {
        jira_config_with_base_url("https://jira.example".to_string())
    }

    fn jira_config_with_base_url(base_url: String) -> JiraConfig {
        JiraConfig {
            base_url,
            deployment: JiraDeployment::ServerDataCenter,
            auth: AtlassianAuth::Pat {
                personal_token: "test-pat-value".to_string(),
            },
            ssl_verify: true,
            projects_filter: BTreeSet::new(),
            timeout_seconds: 75,
        }
    }

    fn tool(name: &'static str) -> Tool {
        Tool::new(name, "", Arc::<JsonObject>::new(Default::default()))
    }

    fn current_tool_names(server: &AtlassianMcpServer) -> Vec<String> {
        tool_names(server.current_tools_result().tools)
    }

    fn tool_names(tools: Vec<Tool>) -> Vec<String> {
        tools
            .into_iter()
            .map(|tool| tool.name.to_string())
            .collect()
    }

    fn stage_three_candidate_tools() -> Vec<Tool> {
        tools::STAGE3_JIRA_TOOL_NAMES
            .iter()
            .map(|&name| tool(name))
            .collect()
    }

    fn stage_three_write_tool_names() -> Vec<&'static str> {
        tools::STAGE3_JIRA_TOOL_NAMES
            .iter()
            .copied()
            .filter(|name| {
                tool_registry::metadata_for(name)
                    .is_some_and(|metadata| metadata.access == ToolAccess::Write)
            })
            .collect()
    }

    fn stage_three_c3_tool_names() -> Vec<&'static str> {
        vec![
            tools::JIRA_GET_ALL_PROJECTS_TOOL_NAME,
            tools::JIRA_GET_PROJECT_VERSIONS_TOOL_NAME,
            tools::JIRA_GET_PROJECT_COMPONENTS_TOOL_NAME,
            tools::JIRA_CREATE_VERSION_TOOL_NAME,
            tools::JIRA_BATCH_CREATE_VERSIONS_TOOL_NAME,
            tools::JIRA_GET_USER_PROFILE_TOOL_NAME,
            tools::JIRA_GET_ISSUE_WATCHERS_TOOL_NAME,
            tools::JIRA_ADD_WATCHER_TOOL_NAME,
            tools::JIRA_REMOVE_WATCHER_TOOL_NAME,
            tools::JIRA_GET_WORKLOG_TOOL_NAME,
            tools::JIRA_ADD_WORKLOG_TOOL_NAME,
            tools::JIRA_GET_LINK_TYPES_TOOL_NAME,
            tools::JIRA_LINK_TO_EPIC_TOOL_NAME,
            tools::JIRA_CREATE_ISSUE_LINK_TOOL_NAME,
            tools::JIRA_CREATE_REMOTE_ISSUE_LINK_TOOL_NAME,
            tools::JIRA_REMOVE_ISSUE_LINK_TOOL_NAME,
            tools::JIRA_DOWNLOAD_ATTACHMENTS_TOOL_NAME,
            tools::JIRA_GET_ISSUE_IMAGES_TOOL_NAME,
        ]
    }

    fn stage_three_c3_write_tool_names() -> Vec<&'static str> {
        stage_three_c3_tool_names()
            .into_iter()
            .filter(|name| {
                tool_registry::metadata_for(name)
                    .is_some_and(|metadata| metadata.access == ToolAccess::Write)
            })
            .collect()
    }

    fn stage_three_c4_tool_names() -> Vec<&'static str> {
        vec![
            tools::JIRA_GET_AGILE_BOARDS_TOOL_NAME,
            tools::JIRA_GET_BOARD_ISSUES_TOOL_NAME,
            tools::JIRA_GET_SPRINTS_FROM_BOARD_TOOL_NAME,
            tools::JIRA_GET_SPRINT_ISSUES_TOOL_NAME,
            tools::JIRA_CREATE_SPRINT_TOOL_NAME,
            tools::JIRA_UPDATE_SPRINT_TOOL_NAME,
            tools::JIRA_ADD_ISSUES_TO_SPRINT_TOOL_NAME,
            tools::JIRA_GET_SERVICE_DESK_FOR_PROJECT_TOOL_NAME,
            tools::JIRA_GET_SERVICE_DESK_QUEUES_TOOL_NAME,
            tools::JIRA_GET_QUEUE_ISSUES_TOOL_NAME,
            tools::JIRA_GET_ISSUE_PROFORMA_FORMS_TOOL_NAME,
            tools::JIRA_GET_PROFORMA_FORM_DETAILS_TOOL_NAME,
            tools::JIRA_UPDATE_PROFORMA_FORM_ANSWERS_TOOL_NAME,
            tools::JIRA_GET_ISSUE_DATES_TOOL_NAME,
            tools::JIRA_GET_ISSUE_SLA_TOOL_NAME,
            tools::JIRA_GET_ISSUE_DEVELOPMENT_INFO_TOOL_NAME,
            tools::JIRA_GET_ISSUES_DEVELOPMENT_INFO_TOOL_NAME,
        ]
    }

    fn stage_three_c4_write_tool_names() -> Vec<&'static str> {
        stage_three_c4_tool_names()
            .into_iter()
            .filter(|name| {
                tool_registry::metadata_for(name)
                    .is_some_and(|metadata| metadata.access == ToolAccess::Write)
            })
            .collect()
    }

    fn expected_stage_two_default_tools() -> Vec<String> {
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
            MIGRATION_STATUS_TOOL_NAME.to_string(),
        ]
    }

    #[derive(Clone, Debug)]
    struct RecordedRequest {
        method: Method,
        path: String,
        body: Value,
    }

    #[derive(Clone)]
    struct MockJiraState {
        requests: Arc<Mutex<Vec<RecordedRequest>>>,
    }

    async fn mock_jira_handler(
        State(state): State<MockJiraState>,
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
        state.requests.lock().await.push(RecordedRequest {
            method: method.clone(),
            path: path.clone(),
            body: parsed_body.clone(),
        });

        let expected_header = format!("Bearer {}", "test-pat-value");
        if headers
            .get("authorization")
            .and_then(|value| value.to_str().ok())
            != Some(expected_header.as_str())
        {
            return (
                StatusCode::UNAUTHORIZED,
                Json(json!({"errorMessages": ["auth"]})),
            )
                .into_response();
        }

        let path_only = uri.path();
        if method == Method::GET && path_only == "/secure/attachment/1/file.png" {
            return (
                StatusCode::OK,
                [(axum::http::header::CONTENT_TYPE, "image/png")],
                "image-bytes",
            )
                .into_response();
        }
        if method == Method::GET && path_only == "/secure/attachment/2/notes.txt" {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({
                    "errorMessages": [
                        "failed /secure/attachment/2/notes.txt?token=secret&client=abc"
                    ]
                })),
            )
                .into_response();
        }

        if method == Method::GET
            && (path == "/rest/api/2/issue/ABC-1" || path.starts_with("/rest/api/2/issue/ABC-1?"))
        {
            return (
                StatusCode::OK,
                Json(json!({
                    "id": "10001",
                    "key": "ABC-1",
                    "fields": {
                        "summary": "Mock issue",
                        "created": "2026-01-01T00:00:00.000+0000",
                        "updated": "2026-01-02T00:00:00.000+0000",
                        "duedate": "2026-01-10",
                        "resolutiondate": "2026-01-03T00:00:00.000+0000",
                        "status": {
                            "id": "3",
                            "name": "Done",
                            "statusCategory": {"name": "Done"}
                        },
                        "customfield_sla": {
                            "name": "Time to resolution SLA",
                            "ongoingCycle": {
                                "breached": false,
                                "elapsedTime": {"millis": 60000},
                                "remainingTime": {"millis": 120000},
                                "startTime": "2026-01-01T00:00:00.000+0000"
                            }
                        },
                        "attachment": [
                            {
                                "id": "1",
                                "filename": "file.png",
                                "mimeType": "image/png",
                                "size": 11,
                                "content": "/secure/attachment/1/file.png?token=secret"
                            },
                            {
                                "id": "2",
                                "filename": "notes.txt",
                                "mimeType": "text/plain",
                                "size": 42,
                                "content": "/secure/attachment/2/notes.txt?token=secret&client=abc"
                            }
                        ]
                    }
                })),
            )
                .into_response();
        }
        if method == Method::GET
            && (path == "/rest/api/2/issue/TXT-1" || path.starts_with("/rest/api/2/issue/TXT-1?"))
        {
            return (
                StatusCode::OK,
                Json(json!({
                    "id": "20001",
                    "key": "TXT-1",
                    "fields": {
                        "summary": "Text only",
                        "attachment": [
                            {
                                "id": "2",
                                "filename": "notes.txt",
                                "mimeType": "text/plain",
                                "size": 42,
                                "content": "/secure/attachment/2/notes.txt?token=secret&client=abc"
                            }
                        ]
                    }
                })),
            )
                .into_response();
        }
        if method == Method::GET && path == "/rest/api/2/issue/ABC-1/watchers" {
            return (
                StatusCode::OK,
                Json(json!({
                    "watchCount": 1,
                    "isWatching": false,
                    "watchers": [
                        {"accountId": "account-1", "displayName": "Ada Lovelace", "active": true}
                    ]
                })),
            )
                .into_response();
        }
        if method == Method::POST && path == "/rest/api/2/issue/ABC-1/watchers" {
            return StatusCode::NO_CONTENT.into_response();
        }
        if method == Method::DELETE && path == "/rest/api/2/issue/ABC-1/watchers?username=ada" {
            return StatusCode::NO_CONTENT.into_response();
        }
        if method == Method::GET
            && path == "/rest/api/2/issue/ABC-1/worklog?startAt=0&maxResults=10"
        {
            return (
                StatusCode::OK,
                Json(json!({
                    "startAt": 0,
                    "maxResults": 10,
                    "total": 2,
                    "worklogs": [
                        {
                            "id": "100",
                            "timeSpent": "1h",
                            "started": "2026-01-01T00:00:00.000+0000",
                            "author": {"displayName": "Ada Lovelace"}
                        },
                        {
                            "id": "101",
                            "timeSpent": "30m"
                        }
                    ]
                })),
            )
                .into_response();
        }
        if method == Method::POST
            && path == "/rest/api/2/issue/ABC-1/worklog?adjustEstimate=new&newEstimate=2h"
        {
            return (
                StatusCode::OK,
                Json(json!({
                    "id": "300",
                    "timeSpent": parsed_body["timeSpent"],
                    "started": parsed_body["started"]
                })),
            )
                .into_response();
        }
        if method == Method::PUT && path.starts_with("/rest/api/2/issue/ABC-1") {
            return StatusCode::NO_CONTENT.into_response();
        }
        if method == Method::DELETE && path.starts_with("/rest/api/2/issue/ABC-1") {
            return StatusCode::NO_CONTENT.into_response();
        }
        if method == Method::POST && path == "/rest/api/2/issue" {
            return (
                StatusCode::OK,
                Json(json!({
                    "id": "10002",
                    "key": "ABC-2",
                    "fields": {
                        "summary": "Created issue",
                        "project": {"key": "ABC", "name": "Demo"},
                        "issuetype": {"name": "Task"}
                    }
                })),
            )
                .into_response();
        }
        if method == Method::POST && path == "/rest/api/2/issue/bulk" {
            return (
                StatusCode::OK,
                Json(json!({
                    "issues": [{"id": "10003", "key": "ABC-3", "self": "https://jira.example/rest/api/2/issue/10003"}],
                    "errors": [{"failedElementNumber": 1, "message": "validation failed"}]
                })),
            )
                .into_response();
        }
        if method == Method::POST && path == "/rest/api/3/changelog/bulkfetch" {
            return (
                StatusCode::OK,
                Json(json!({
                    "issueChangeLogs": [
                        {
                            "issueId": "10001",
                            "changeHistories": [
                                {
                                    "id": "20001",
                                    "items": [{"field": "status", "fromString": "Open", "toString": "Done"}]
                                }
                            ]
                        }
                    ],
                    "nextPageToken": "next-token"
                })),
            )
                .into_response();
        }
        if method == Method::GET && path == "/rest/api/2/project?includeArchived=false" {
            return (
                StatusCode::OK,
                Json(json!([
                    {"id": "10000", "key": "ABC", "name": "Allowed"},
                    {"id": "10001", "key": "XYZ", "name": "Filtered"}
                ])),
            )
                .into_response();
        }
        if method == Method::GET && path == "/rest/api/2/project/ABC/versions" {
            return (
                StatusCode::OK,
                Json(json!([
                    {"id": "1", "name": "v1"},
                    {"name": "unnumbered"}
                ])),
            )
                .into_response();
        }
        if method == Method::GET && path == "/rest/api/2/project/ABC/components" {
            return (
                StatusCode::OK,
                Json(json!([
                    {"id": "10", "name": "API"},
                    {}
                ])),
            )
                .into_response();
        }
        if method == Method::POST && path == "/rest/api/2/version" {
            if parsed_body["name"] == json!("bad") {
                return (
                    StatusCode::BAD_REQUEST,
                    Json(json!({"errorMessages": ["bad version"]})),
                )
                    .into_response();
            }
            return (
                StatusCode::OK,
                Json(json!({
                    "id": "20000",
                    "name": parsed_body["name"],
                    "project": parsed_body["project"],
                    "released": parsed_body.get("released").cloned().unwrap_or(Value::Bool(false))
                })),
            )
                .into_response();
        }
        if method == Method::GET && path == "/rest/api/2/user?username=ada" {
            return (
                StatusCode::OK,
                Json(json!({
                    "accountId": "account-1",
                    "name": "ada",
                    "displayName": "Ada Lovelace",
                    "active": true
                })),
            )
                .into_response();
        }
        if method == Method::GET && path == "/rest/api/2/user?accountId=account-1" {
            return (
                StatusCode::OK,
                Json(json!({
                    "accountId": "account-1",
                    "displayName": "Ada Lovelace",
                    "active": true
                })),
            )
                .into_response();
        }
        if method == Method::GET && path == "/rest/api/2/issueLinkType" {
            return (
                StatusCode::OK,
                Json(json!({
                    "issueLinkTypes": [
                        {
                            "id": "10000",
                            "name": "Blocks",
                            "inward": "is blocked by",
                            "outward": "blocks"
                        },
                        {
                            "id": "10001",
                            "name": "Relates"
                        }
                    ]
                })),
            )
                .into_response();
        }
        if method == Method::POST && path == "/rest/api/2/issueLink" {
            return (
                StatusCode::CREATED,
                Json(json!({"id": "200", "type": parsed_body["type"]})),
            )
                .into_response();
        }
        if method == Method::POST && path == "/rest/api/2/issue/ABC-1/remotelink" {
            return (
                StatusCode::CREATED,
                Json(json!({"id": "300", "object": parsed_body["object"]})),
            )
                .into_response();
        }
        if method == Method::DELETE && path == "/rest/api/2/issueLink/200" {
            return StatusCode::NO_CONTENT.into_response();
        }
        if method == Method::GET
            && path.starts_with("/rest/agile/1.0/board?")
            && path.contains("projectKeyOrId=NOAGILE")
        {
            return (
                StatusCode::NOT_FOUND,
                Json(json!({"errorMessages": ["Jira Software is not available"]})),
            )
                .into_response();
        }
        if method == Method::GET && path.starts_with("/rest/agile/1.0/board?") {
            return (
                StatusCode::OK,
                Json(json!({
                    "startAt": 0,
                    "maxResults": 2,
                    "total": 1,
                    "isLast": true,
                    "values": [
                        {"id": 1, "name": "Alpha board", "type": "scrum"}
                    ]
                })),
            )
                .into_response();
        }
        if method == Method::GET && path.starts_with("/rest/agile/1.0/board/1/issue?") {
            return (
                StatusCode::OK,
                Json(json!({
                    "startAt": 0,
                    "maxResults": 2,
                    "total": 1,
                    "issues": [
                        {"id": "10001", "key": "ABC-1", "fields": {"summary": "Sprint issue"}}
                    ]
                })),
            )
                .into_response();
        }
        if method == Method::GET && path.starts_with("/rest/agile/1.0/board/1/sprint?") {
            return (
                StatusCode::OK,
                Json(json!({
                    "startAt": 0,
                    "maxResults": 2,
                    "total": 1,
                    "isLast": true,
                    "values": [
                        {"id": 2, "name": "Sprint 2", "state": "active"}
                    ]
                })),
            )
                .into_response();
        }
        if method == Method::GET && path.starts_with("/rest/agile/1.0/sprint/2/issue?") {
            return (
                StatusCode::OK,
                Json(json!({
                    "startAt": 0,
                    "maxResults": 2,
                    "total": 1,
                    "issues": [
                        {"id": "10001", "key": "ABC-1", "fields": {"summary": "Sprint issue"}}
                    ]
                })),
            )
                .into_response();
        }
        if method == Method::POST && path == "/rest/agile/1.0/sprint" {
            return (
                StatusCode::OK,
                Json(json!({
                    "id": 2,
                    "name": parsed_body["name"],
                    "originBoardId": parsed_body["originBoardId"],
                    "state": "future"
                })),
            )
                .into_response();
        }
        if method == Method::PUT && path == "/rest/agile/1.0/sprint/2" {
            return (
                StatusCode::OK,
                Json(json!({
                    "id": 2,
                    "name": parsed_body["name"],
                    "state": parsed_body["state"],
                    "goal": parsed_body["goal"]
                })),
            )
                .into_response();
        }
        if method == Method::POST && path == "/rest/agile/1.0/sprint/2/issue" {
            return StatusCode::NO_CONTENT.into_response();
        }
        if method == Method::GET && path_only.starts_with("/jsm-down/rest/servicedeskapi") {
            return (
                StatusCode::NOT_FOUND,
                Json(json!({"errorMessages": ["Jira Service Management is not available"]})),
            )
                .into_response();
        }
        if method == Method::GET && path == "/rest/servicedeskapi/servicedesk" {
            return (
                StatusCode::OK,
                Json(json!({
                    "size": 2,
                    "values": [
                        {"id": "4", "projectKey": "ABC", "serviceDeskName": "Support"},
                        {"id": "5", "projectKey": "XYZ", "serviceDeskName": "Other"}
                    ]
                })),
            )
                .into_response();
        }
        if method == Method::GET
            && path == "/rest/servicedeskapi/servicedesk/4/queue?start=0&limit=50"
        {
            return (
                StatusCode::OK,
                Json(json!({
                    "start": 0,
                    "limit": 50,
                    "size": 1,
                    "values": [
                        {"id": "47", "name": "Open requests"}
                    ]
                })),
            )
                .into_response();
        }
        if method == Method::GET
            && path == "/rest/servicedeskapi/servicedesk/4/queue/47/issue?start=0&limit=2"
        {
            return (
                StatusCode::OK,
                Json(json!({
                    "start": 0,
                    "limit": 2,
                    "size": 1,
                    "values": [
                        {"id": "10001", "key": "ABC-1", "fields": {"summary": "Customer request"}}
                    ]
                })),
            )
                .into_response();
        }
        if method == Method::GET && path == "/jira/forms/cloud/cloud-123/issue/ABC-1/form" {
            return (
                StatusCode::OK,
                Json(json!({
                    "forms": [
                        {
                            "id": "form-1",
                            "name": "Request form",
                            "state": {"status": "o"},
                            "submitted": false
                        }
                    ]
                })),
            )
                .into_response();
        }
        if method == Method::GET && path == "/jira/forms/cloud/cloud-123/issue/ABC-1/form/form-1" {
            return (
                StatusCode::OK,
                Json(json!({
                    "id": "form-1",
                    "name": "Request form",
                    "state": {"status": "o"},
                    "design": {"content": []},
                    "answers": {"q1": {"text": "Existing"}}
                })),
            )
                .into_response();
        }
        if method == Method::PUT && path == "/jira/forms/cloud/cloud-123/issue/ABC-1/form/form-1" {
            return (
                StatusCode::OK,
                Json(json!({
                    "id": "form-1",
                    "updated": true,
                    "answers": parsed_body["answers"]
                })),
            )
                .into_response();
        }
        if method == Method::GET && path_only.starts_with("/jira/forms/cloud/forms-down/") {
            return (
                StatusCode::NOT_FOUND,
                Json(json!({"errorMessages": ["Jira Forms is not available"]})),
            )
                .into_response();
        }
        if method == Method::GET && path_only.starts_with("/dev-down/rest/dev-status") {
            return (
                StatusCode::NOT_FOUND,
                Json(json!({"errorMessages": ["Jira development status is not available"]})),
            )
                .into_response();
        }
        if method == Method::GET && path.starts_with("/rest/dev-status/1.0/issue/detail?") {
            return (
                StatusCode::OK,
                Json(json!({
                    "detail": [
                        {
                            "applicationType": "github",
                            "dataType": "pullrequest",
                            "branches": [{"name": "main"}],
                            "pullRequests": [{"id": "pr-1", "name": "Fix bug"}],
                            "commits": [{"id": "commit-1", "displayId": "abc123"}]
                        }
                    ]
                })),
            )
                .into_response();
        }

        (
            StatusCode::NOT_FOUND,
            Json(json!({"errorMessages": ["missing"]})),
        )
            .into_response()
    }

    async fn mock_jira_server() -> (String, Arc<Mutex<Vec<RecordedRequest>>>) {
        let requests = Arc::new(Mutex::new(Vec::new()));
        let app = Router::new()
            .fallback(any(mock_jira_handler))
            .with_state(MockJiraState {
                requests: requests.clone(),
            });
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let address: SocketAddr = listener.local_addr().unwrap();
        tokio::spawn(async move {
            axum::serve(listener, app).await.unwrap();
        });

        (format!("http://{address}"), requests)
    }

    #[test]
    fn server_info_advertises_tools() {
        let info = AtlassianMcpServer::default().get_info();

        assert_eq!(info.server_info.name, SERVER_NAME);
        assert_eq!(info.server_info.version, env!("CARGO_PKG_VERSION"));
        assert!(info.capabilities.tools.is_some());
        assert!(info.capabilities.prompts.is_none());
        assert!(info.capabilities.resources.is_none());
    }

    #[test]
    fn tool_metadata_is_generated() {
        assert_eq!(
            AtlassianMcpServer::migration_status_tool_attr().name,
            MIGRATION_STATUS_TOOL_NAME
        );
    }

    #[test]
    fn migration_status_reports_stage_scope() {
        let server = AtlassianMcpServer::default();
        let status = server.migration_status();

        assert!(status.contains("Stage 2 Jira core migration is complete"));
        assert!(status.contains("Jira config/auth/client/models/tool handlers are implemented"));
    }

    #[test]
    fn server_info_uses_app_context() {
        let config = RuntimeConfig {
            read_only: true,
            ..RuntimeConfig::default()
        };
        let server = AtlassianMcpServer::new(Arc::new(AppContext::from_config(&config)));
        let info = server.get_info();
        let instructions = info.instructions.unwrap_or_default();

        assert!(instructions.contains("read-only mode"));
        assert!(instructions.contains("Jira core tools are available"));
    }

    #[test]
    fn tool_discovery_uses_registry_and_keeps_migration_status_visible_by_default() {
        let server = AtlassianMcpServer::default();

        assert_eq!(
            current_tool_names(&server),
            vec![MIGRATION_STATUS_TOOL_NAME.to_string()]
        );
        assert!(server.get_tool(MIGRATION_STATUS_TOOL_NAME).is_some());
    }

    #[test]
    fn tool_discovery_lists_jira_default_tools_when_configured() {
        let server = server_with_config(RuntimeConfig {
            jira: Some(jira_config()),
            ..runtime_config()
        });
        let names = current_tool_names(&server);

        for name in expected_stage_two_default_tools() {
            assert!(names.contains(&name), "{name} should be visible by default");
        }
        assert!(server.get_tool(tools::JIRA_GET_ISSUE_TOOL_NAME).is_some());
    }

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
            projects.structured_content.as_ref().unwrap()[0]["key"],
            json!("ABC")
        );
        assert_eq!(
            projects
                .structured_content
                .as_ref()
                .unwrap()
                .as_array()
                .unwrap()
                .len(),
            1
        );
        assert_eq!(
            versions.structured_content.as_ref().unwrap()[0]["name"],
            json!("v1")
        );
        assert_eq!(
            components.structured_content.as_ref().unwrap()[1],
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
        assert!(error.contains("/secure/attachment/2/notes.txt?<redacted>"));
        assert!(!error.contains("token=secret"));
        assert!(!error.contains("client=abc"));
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
            .jira_get_service_desk_for_project(Parameters(
                tools::JiraGetServiceDeskForProjectArgs {
                    project_key: "ABC".to_string(),
                },
            ))
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
            .jira_get_service_desk_for_project(Parameters(
                tools::JiraGetServiceDeskForProjectArgs {
                    project_key: "ABC".to_string(),
                },
            ))
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
            .jira_update_proforma_form_answers(Parameters(
                tools::JiraUpdateProformaFormAnswersArgs {
                    issue_key: "ABC-1".to_string(),
                    form_id: "form-1".to_string(),
                    answers: json!([
                        {"questionId": "q1", "type": "TEXT", "value": "Updated"},
                        {"questionId": "q2", "type": "SELECT", "value": "Product A"},
                        {"questionId": "q3", "type": "DATE", "value": "2026-06-04"}
                    ]),
                },
            ))
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
            .jira_update_proforma_form_answers(Parameters(
                tools::JiraUpdateProformaFormAnswersArgs {
                    issue_key: "ABC-1".to_string(),
                    form_id: "form-1".to_string(),
                    answers: json!([{"questionId": "q1", "type": "TEXT", "value": "Updated"}]),
                },
            ))
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
            .jira_update_proforma_form_answers(Parameters(
                tools::JiraUpdateProformaFormAnswersArgs {
                    issue_key: "ABC-1".to_string(),
                    form_id: "form-1".to_string(),
                    answers: json!("not-answers"),
                },
            ))
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
        assert!(structured["issue"]["fields"]["created"].is_null());
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
                working_hours_only: Some(true),
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
        assert_eq!(structured["working_hours_only"], json!(true));
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

    #[test]
    fn tool_discovery_applies_toolsets_and_read_only_to_real_jira_tools() {
        let fields_only = server_with_config(RuntimeConfig {
            jira: Some(jira_config()),
            enabled_toolsets: BTreeSet::from(["jira_fields".to_string()]),
            ..runtime_config()
        });
        let read_only = server_with_config(RuntimeConfig {
            read_only: true,
            jira: Some(jira_config()),
            ..runtime_config()
        });

        assert_eq!(
            current_tool_names(&fields_only),
            vec![
                tools::JIRA_GET_FIELD_OPTIONS_TOOL_NAME.to_string(),
                tools::JIRA_SEARCH_FIELDS_TOOL_NAME.to_string(),
                MIGRATION_STATUS_TOOL_NAME.to_string(),
            ]
        );
        assert!(
            !current_tool_names(&read_only)
                .contains(&tools::JIRA_ADD_COMMENT_TOOL_NAME.to_string())
        );
        assert!(
            read_only
                .guard_registered_tool_call(tools::JIRA_TRANSITION_ISSUE_TOOL_NAME)
                .is_err()
        );
    }

    #[test]
    fn stage_three_candidate_tool_discovery_uses_registered_metadata_at_mcp_boundary() {
        let agile_only = server_with_config(RuntimeConfig {
            jira: Some(jira_config()),
            enabled_toolsets: BTreeSet::from(["jira_agile".to_string()]),
            ..runtime_config()
        });
        let read_only = server_with_config(RuntimeConfig {
            read_only: true,
            jira: Some(jira_config()),
            ..runtime_config()
        });

        assert_eq!(
            tool_names(agile_only.filtered_tools_from(stage_three_candidate_tools())),
            vec![
                tools::JIRA_ADD_ISSUES_TO_SPRINT_TOOL_NAME.to_string(),
                tools::JIRA_CREATE_SPRINT_TOOL_NAME.to_string(),
                tools::JIRA_GET_AGILE_BOARDS_TOOL_NAME.to_string(),
                tools::JIRA_GET_BOARD_ISSUES_TOOL_NAME.to_string(),
                tools::JIRA_GET_SPRINT_ISSUES_TOOL_NAME.to_string(),
                tools::JIRA_GET_SPRINTS_FROM_BOARD_TOOL_NAME.to_string(),
                tools::JIRA_UPDATE_SPRINT_TOOL_NAME.to_string(),
            ]
        );
        assert!(
            !tool_names(read_only.filtered_tools_from(stage_three_candidate_tools()))
                .contains(&tools::JIRA_CREATE_ISSUE_TOOL_NAME.to_string())
        );
        assert!(
            tool_names(read_only.filtered_tools_from(stage_three_candidate_tools()))
                .contains(&tools::JIRA_BATCH_GET_CHANGELOGS_TOOL_NAME.to_string())
        );
    }

    #[test]
    fn c4_product_dependent_tools_have_routes_and_registered_metadata() {
        let server = server_with_config(RuntimeConfig {
            jira: Some(jira_config()),
            atlassian_oauth_cloud_id: Some("cloud-123".to_string()),
            ..runtime_config()
        });
        let names = current_tool_names(&server);
        let c4_tools = stage_three_c4_tool_names();

        assert_eq!(c4_tools.len(), 17);
        for name in c4_tools {
            assert!(
                tool_registry::metadata_for(name).is_some(),
                "{name} should have registered metadata"
            );
            assert!(
                server.get_tool(name).is_some(),
                "{name} should have a route"
            );
            assert!(
                names.contains(&name.to_string()),
                "{name} should be visible"
            );
        }
    }

    #[test]
    fn c4_product_dependent_toolsets_filter_to_expected_tools() {
        let cases = [
            (
                "jira_agile",
                vec![
                    tools::JIRA_GET_AGILE_BOARDS_TOOL_NAME,
                    tools::JIRA_GET_BOARD_ISSUES_TOOL_NAME,
                    tools::JIRA_GET_SPRINTS_FROM_BOARD_TOOL_NAME,
                    tools::JIRA_GET_SPRINT_ISSUES_TOOL_NAME,
                    tools::JIRA_CREATE_SPRINT_TOOL_NAME,
                    tools::JIRA_UPDATE_SPRINT_TOOL_NAME,
                    tools::JIRA_ADD_ISSUES_TO_SPRINT_TOOL_NAME,
                ],
            ),
            (
                "jira_service_desk",
                vec![
                    tools::JIRA_GET_SERVICE_DESK_FOR_PROJECT_TOOL_NAME,
                    tools::JIRA_GET_SERVICE_DESK_QUEUES_TOOL_NAME,
                    tools::JIRA_GET_QUEUE_ISSUES_TOOL_NAME,
                ],
            ),
            (
                "jira_forms",
                vec![
                    tools::JIRA_GET_ISSUE_PROFORMA_FORMS_TOOL_NAME,
                    tools::JIRA_GET_PROFORMA_FORM_DETAILS_TOOL_NAME,
                    tools::JIRA_UPDATE_PROFORMA_FORM_ANSWERS_TOOL_NAME,
                ],
            ),
            (
                "jira_metrics",
                vec![
                    tools::JIRA_GET_ISSUE_DATES_TOOL_NAME,
                    tools::JIRA_GET_ISSUE_SLA_TOOL_NAME,
                ],
            ),
            (
                "jira_development",
                vec![
                    tools::JIRA_GET_ISSUE_DEVELOPMENT_INFO_TOOL_NAME,
                    tools::JIRA_GET_ISSUES_DEVELOPMENT_INFO_TOOL_NAME,
                ],
            ),
        ];
        let c4_tools = stage_three_c4_tool_names();

        for (toolset, expected) in cases {
            let server = server_with_config(RuntimeConfig {
                jira: Some(jira_config()),
                enabled_toolsets: BTreeSet::from([toolset.to_string()]),
                atlassian_oauth_cloud_id: Some("cloud-123".to_string()),
                ..runtime_config()
            });
            let names = current_tool_names(&server);
            for expected_name in expected {
                assert!(
                    names.contains(&expected_name.to_string()),
                    "{toolset} should expose {expected_name}"
                );
            }
            for name in c4_tools.iter().copied() {
                if tool_registry::metadata_for(name)
                    .and_then(|metadata| metadata.toolset)
                    .is_some_and(|metadata_toolset| metadata_toolset != toolset)
                {
                    assert!(
                        !names.contains(&name.to_string()),
                        "{toolset} should not expose {name}"
                    );
                }
            }
        }
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
                working_hours_only: None,
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
    fn tool_discovery_applies_enabled_tools_filter_to_migration_status() {
        let server = server_with_config(RuntimeConfig {
            enabled_tools: Some(BTreeSet::from(["some_other_tool".to_string()])),
            ..runtime_config()
        });

        assert!(current_tool_names(&server).is_empty());
        assert!(server.get_tool(MIGRATION_STATUS_TOOL_NAME).is_none());
        assert!(
            server
                .guard_registered_tool_call(MIGRATION_STATUS_TOOL_NAME)
                .is_err()
        );
    }

    #[test]
    fn tool_discovery_does_not_apply_toolsets_to_migration_status() {
        let server = server_with_config(RuntimeConfig {
            enabled_toolsets: BTreeSet::new(),
            ..runtime_config()
        });

        assert_eq!(
            current_tool_names(&server),
            vec![MIGRATION_STATUS_TOOL_NAME.to_string()]
        );
    }

    #[test]
    fn tool_discovery_fails_closed_for_unmapped_tools() {
        let server = AtlassianMcpServer::default();
        let tools =
            server.filtered_tools_from([tool(MIGRATION_STATUS_TOOL_NAME), tool("unmapped_tool")]);
        let names: Vec<_> = tools
            .into_iter()
            .map(|tool| tool.name.to_string())
            .collect();

        assert_eq!(names, vec![MIGRATION_STATUS_TOOL_NAME.to_string()]);
    }

    #[test]
    fn tool_discovery_applies_future_service_and_toolset_policy_at_server_boundary() {
        let unavailable = AtlassianMcpServer::default();
        let available = server_with_config(RuntimeConfig {
            jira: Some(jira_config()),
            confluence_url: Some("https://confluence.example".to_string()),
            ..runtime_config()
        });
        let jira_fields_only = server_with_config(RuntimeConfig {
            jira: Some(jira_config()),
            enabled_toolsets: BTreeSet::from(["jira_fields".to_string()]),
            ..runtime_config()
        });

        assert_eq!(
            tool_names(unavailable.filtered_tools_from_with_metadata(
                [
                    tool("stage1_synthetic_jira_read"),
                    tool("stage1_synthetic_confluence_read"),
                ],
                metadata_for_test_tool,
            )),
            Vec::<String>::new()
        );
        assert_eq!(
            tool_names(available.filtered_tools_from_with_metadata(
                [
                    tool("stage1_synthetic_jira_read"),
                    tool("stage1_synthetic_confluence_read"),
                ],
                metadata_for_test_tool,
            )),
            vec![
                "stage1_synthetic_confluence_read".to_string(),
                "stage1_synthetic_jira_read".to_string(),
            ]
        );
        assert!(
            jira_fields_only
                .filtered_tools_from_with_metadata(
                    [tool("stage1_synthetic_jira_read")],
                    metadata_for_test_tool,
                )
                .is_empty()
        );
    }

    #[test]
    fn direct_call_guard_applies_future_read_only_policy_at_server_boundary() {
        let read_only_server = server_with_config(RuntimeConfig {
            read_only: true,
            jira: Some(jira_config()),
            ..runtime_config()
        });
        let read_write_server = server_with_config(RuntimeConfig {
            jira: Some(jira_config()),
            ..runtime_config()
        });

        let error = read_only_server
            .guard_tool_call_with_metadata(
                "stage1_synthetic_jira_write",
                true,
                metadata_for_test_tool,
            )
            .unwrap_err();

        assert_eq!(error.message, "tool is disabled in read-only mode");
        assert!(
            read_write_server
                .guard_tool_call_with_metadata(
                    "stage1_synthetic_jira_write",
                    true,
                    metadata_for_test_tool,
                )
                .is_ok()
        );
        assert!(
            read_write_server
                .guard_tool_call_with_metadata(
                    "stage1_synthetic_jira_write",
                    false,
                    metadata_for_test_tool,
                )
                .is_err()
        );
    }

    #[test]
    fn stage_three_direct_call_guard_uses_registered_metadata_at_mcp_boundary() {
        let read_only_server = server_with_config(RuntimeConfig {
            read_only: true,
            jira: Some(jira_config()),
            ..runtime_config()
        });
        let read_write_server = server_with_config(RuntimeConfig {
            jira: Some(jira_config()),
            ..runtime_config()
        });

        for name in stage_three_write_tool_names() {
            let error = read_only_server
                .guard_tool_call_with_metadata(name, true, tool_registry::metadata_for)
                .unwrap_err();
            assert_eq!(error.message, "tool is disabled in read-only mode");
        }
        assert!(
            read_write_server
                .guard_tool_call_with_metadata(
                    tools::JIRA_BATCH_GET_CHANGELOGS_TOOL_NAME,
                    true,
                    tool_registry::metadata_for,
                )
                .is_ok()
        );
        assert!(
            read_write_server
                .guard_tool_call_with_metadata(
                    tools::JIRA_CREATE_ISSUE_TOOL_NAME,
                    false,
                    tool_registry::metadata_for,
                )
                .is_err()
        );
    }

    #[test]
    fn c3_common_tool_cross_check_lists_all_names_and_routes() {
        let server = server_with_config(RuntimeConfig {
            jira: Some(jira_config()),
            ..runtime_config()
        });
        let names = stage_three_c3_tool_names();

        assert_eq!(names.len(), 18);
        for name in names {
            let metadata = tool_registry::metadata_for(name)
                .unwrap_or_else(|| panic!("{name} should have metadata"));
            assert_eq!(metadata.service, ToolService::Jira);
            assert!(
                server.get_tool(name).is_some(),
                "{name} should have a route"
            );
        }
    }

    #[test]
    fn c3_toolset_and_enabled_tools_filters_are_exact_at_mcp_boundary() {
        let projects_only = server_with_config(RuntimeConfig {
            jira: Some(jira_config()),
            enabled_toolsets: BTreeSet::from(["jira_projects".to_string()]),
            ..runtime_config()
        });
        let worklog_only = server_with_config(RuntimeConfig {
            jira: Some(jira_config()),
            enabled_tools: Some(BTreeSet::from([
                tools::JIRA_GET_WORKLOG_TOOL_NAME.to_string()
            ])),
            ..runtime_config()
        });

        assert_eq!(
            current_tool_names(&projects_only),
            vec![
                tools::JIRA_BATCH_CREATE_VERSIONS_TOOL_NAME.to_string(),
                tools::JIRA_CREATE_VERSION_TOOL_NAME.to_string(),
                tools::JIRA_GET_ALL_PROJECTS_TOOL_NAME.to_string(),
                tools::JIRA_GET_PROJECT_COMPONENTS_TOOL_NAME.to_string(),
                tools::JIRA_GET_PROJECT_VERSIONS_TOOL_NAME.to_string(),
                MIGRATION_STATUS_TOOL_NAME.to_string(),
            ]
        );
        assert_eq!(
            current_tool_names(&worklog_only),
            vec![tools::JIRA_GET_WORKLOG_TOOL_NAME.to_string()]
        );
        assert!(
            worklog_only
                .guard_registered_tool_call(tools::JIRA_GET_WORKLOG_TOOL_NAME)
                .is_ok()
        );
        assert!(
            worklog_only
                .guard_registered_tool_call(tools::JIRA_GET_LINK_TYPES_TOOL_NAME)
                .is_err()
        );
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
}

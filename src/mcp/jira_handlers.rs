use crate::{
    jira::{
        client::{
            AttachmentFetchOptions, DEFAULT_ATTACHMENT_MAX_BYTES, FieldOptionsRequest,
            GetIssueRequest, SearchRequest,
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
    mcp_errors::atlassian_error,
};
use rmcp::{
    ErrorData, handler::server::wrapper::Parameters, model::CallToolResult, tool, tool_router,
};
use serde_json::Value;

use super::{
    AtlassianMcpServer,
    jira_payloads::{
        add_worklog_payload_from_args, batch_create_issue_updates_from_args,
        create_issue_fields_from_args, create_sprint_payload_from_args,
        issue_link_payload_from_args, parse_optional_object_arg, parse_optional_string_list_arg,
        parse_required_object_list_arg, parse_required_string_list_arg,
        remote_issue_link_payload_from_args, update_issue_fields_from_args,
        update_sprint_payload_from_args, version_payload_from_args, version_payload_from_value,
    },
    optional_non_empty_arg, optional_positive_i64_arg, optional_positive_u64_arg,
    required_non_empty_arg,
};

#[tool_router(router = jira_tool_router, vis = "pub(super)")]
impl AtlassianMcpServer {
    #[tool(description = "Get a Jira issue by key")]
    pub(super) async fn jira_get_issue(
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
            .map_err(atlassian_error)?;

        Ok(CallToolResult::structured(crate::mcp::wrap_array(value)))
    }

    #[tool(description = "Search Jira issues with JQL")]
    pub(super) async fn jira_search(
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
            .map_err(atlassian_error)?;

        Ok(CallToolResult::structured(crate::mcp::wrap_array(value)))
    }

    #[tool(description = "List Jira issues for a project")]
    pub(super) async fn jira_get_project_issues(
        &self,
        Parameters(args): Parameters<JiraGetProjectIssuesArgs>,
    ) -> Result<CallToolResult, ErrorData> {
        let value = self
            .jira_client()?
            .get_project_issues(args.project_key, args.limit, args.start_at)
            .await
            .map_err(atlassian_error)?;

        Ok(CallToolResult::structured(crate::mcp::wrap_array(value)))
    }

    #[tool(description = "Search Jira fields by keyword")]
    pub(super) async fn jira_search_fields(
        &self,
        Parameters(args): Parameters<JiraSearchFieldsArgs>,
    ) -> Result<CallToolResult, ErrorData> {
        let value = self
            .jira_client()?
            .search_fields(args.keyword, args.limit)
            .await
            .map_err(atlassian_error)?;

        Ok(CallToolResult::structured(crate::mcp::wrap_array(value)))
    }

    #[tool(description = "Get options for a Jira field")]
    pub(super) async fn jira_get_field_options(
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
            .map_err(atlassian_error)?;

        Ok(CallToolResult::structured(crate::mcp::wrap_array(value)))
    }

    #[tool(description = "Add a comment to a Jira issue")]
    pub(super) async fn jira_add_comment(
        &self,
        Parameters(args): Parameters<JiraAddCommentArgs>,
    ) -> Result<CallToolResult, ErrorData> {
        let visibility = parse_optional_object_arg(args.visibility, "visibility")?;
        let value = self
            .jira_client()?
            .add_comment(args.issue_key, args.body, visibility)
            .await
            .map_err(atlassian_error)?;

        Ok(CallToolResult::structured(crate::mcp::wrap_array(value)))
    }

    #[tool(description = "Edit a Jira issue comment")]
    pub(super) async fn jira_edit_comment(
        &self,
        Parameters(args): Parameters<JiraEditCommentArgs>,
    ) -> Result<CallToolResult, ErrorData> {
        let visibility = parse_optional_object_arg(args.visibility, "visibility")?;
        let value = self
            .jira_client()?
            .edit_comment(args.issue_key, args.comment_id, args.body, visibility)
            .await
            .map_err(atlassian_error)?;

        Ok(CallToolResult::structured(crate::mcp::wrap_array(value)))
    }

    #[tool(description = "Get available transitions for a Jira issue")]
    pub(super) async fn jira_get_transitions(
        &self,
        Parameters(args): Parameters<JiraGetTransitionsArgs>,
    ) -> Result<CallToolResult, ErrorData> {
        let value = self
            .jira_client()?
            .get_transitions(args.issue_key)
            .await
            .map_err(atlassian_error)?;

        Ok(CallToolResult::structured(crate::mcp::wrap_array(value)))
    }

    #[tool(description = "Transition a Jira issue")]
    pub(super) async fn jira_transition_issue(
        &self,
        Parameters(args): Parameters<JiraTransitionIssueArgs>,
    ) -> Result<CallToolResult, ErrorData> {
        let fields = parse_optional_object_arg(args.fields, "fields")?;
        let value = self
            .jira_client()?
            .transition_issue(args.issue_key, args.transition_id, fields, args.comment)
            .await
            .map_err(atlassian_error)?;

        Ok(CallToolResult::structured(crate::mcp::wrap_array(value)))
    }

    #[tool(description = "Create a Jira issue")]
    pub(super) async fn jira_create_issue(
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
            .map_err(atlassian_error)?;

        Ok(CallToolResult::structured(crate::mcp::wrap_array(value)))
    }

    #[tool(description = "Create multiple Jira issues in a batch")]
    pub(super) async fn jira_batch_create_issues(
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
            .map_err(atlassian_error)?;

        Ok(CallToolResult::structured(crate::mcp::wrap_array(value)))
    }

    #[tool(description = "Get changelogs for multiple Jira issues")]
    pub(super) async fn jira_batch_get_changelogs(
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
            .map_err(atlassian_error)?;

        Ok(CallToolResult::structured(crate::mcp::wrap_array(value)))
    }

    #[tool(description = "Update fields on a Jira issue")]
    pub(super) async fn jira_update_issue(
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
            .map_err(atlassian_error)?;

        Ok(CallToolResult::structured(crate::mcp::wrap_array(value)))
    }

    #[tool(description = "Delete a Jira issue")]
    pub(super) async fn jira_delete_issue(
        &self,
        Parameters(args): Parameters<JiraDeleteIssueArgs>,
    ) -> Result<CallToolResult, ErrorData> {
        let value = self
            .jira_client()?
            .delete_issue(args.issue_key, args.delete_subtasks.unwrap_or(false))
            .await
            .map_err(atlassian_error)?;

        Ok(CallToolResult::structured(crate::mcp::wrap_array(value)))
    }

    #[tool(description = "List Jira projects visible to the current user")]
    pub(super) async fn jira_get_all_projects(
        &self,
        Parameters(args): Parameters<JiraGetAllProjectsArgs>,
    ) -> Result<CallToolResult, ErrorData> {
        let value = self
            .jira_client()?
            .get_all_projects(args.include_archived.unwrap_or(false))
            .await
            .map_err(atlassian_error)?;

        Ok(CallToolResult::structured(crate::mcp::wrap_array(value)))
    }

    #[tool(description = "List versions for a Jira project")]
    pub(super) async fn jira_get_project_versions(
        &self,
        Parameters(args): Parameters<JiraGetProjectVersionsArgs>,
    ) -> Result<CallToolResult, ErrorData> {
        let value = self
            .jira_client()?
            .get_project_versions(args.project_key)
            .await
            .map_err(atlassian_error)?;

        Ok(CallToolResult::structured(crate::mcp::wrap_array(value)))
    }

    #[tool(description = "List components for a Jira project")]
    pub(super) async fn jira_get_project_components(
        &self,
        Parameters(args): Parameters<JiraGetProjectComponentsArgs>,
    ) -> Result<CallToolResult, ErrorData> {
        let value = self
            .jira_client()?
            .get_project_components(args.project_key)
            .await
            .map_err(atlassian_error)?;

        Ok(CallToolResult::structured(crate::mcp::wrap_array(value)))
    }

    #[tool(description = "Create a Jira project version")]
    pub(super) async fn jira_create_version(
        &self,
        Parameters(args): Parameters<JiraCreateVersionArgs>,
    ) -> Result<CallToolResult, ErrorData> {
        let value = self
            .jira_client()?
            .create_version(version_payload_from_args(args)?)
            .await
            .map_err(atlassian_error)?;

        Ok(CallToolResult::structured(crate::mcp::wrap_array(value)))
    }

    #[tool(description = "Create multiple Jira project versions")]
    pub(super) async fn jira_batch_create_versions(
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
            .map_err(atlassian_error)?;

        Ok(CallToolResult::structured(crate::mcp::wrap_array(value)))
    }

    #[tool(description = "Retrieve a Jira user profile")]
    pub(super) async fn jira_get_user_profile(
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
            .map_err(atlassian_error)?;

        Ok(CallToolResult::structured(crate::mcp::wrap_array(value)))
    }

    #[tool(description = "Get watchers for a Jira issue")]
    pub(super) async fn jira_get_issue_watchers(
        &self,
        Parameters(args): Parameters<JiraGetIssueWatchersArgs>,
    ) -> Result<CallToolResult, ErrorData> {
        let value = self
            .jira_client()?
            .get_issue_watchers(args.issue_key)
            .await
            .map_err(atlassian_error)?;

        Ok(CallToolResult::structured(crate::mcp::wrap_array(value)))
    }

    #[tool(description = "Add a watcher to a Jira issue")]
    pub(super) async fn jira_add_watcher(
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
            .map_err(atlassian_error)?;

        Ok(CallToolResult::structured(crate::mcp::wrap_array(value)))
    }

    #[tool(description = "Remove a watcher from a Jira issue")]
    pub(super) async fn jira_remove_watcher(
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
            .map_err(atlassian_error)?;

        Ok(CallToolResult::structured(crate::mcp::wrap_array(value)))
    }

    #[tool(description = "Get worklogs for a Jira issue")]
    pub(super) async fn jira_get_worklog(
        &self,
        Parameters(args): Parameters<JiraGetWorklogArgs>,
    ) -> Result<CallToolResult, ErrorData> {
        let value = self
            .jira_client()?
            .get_worklog(args.issue_key, args.start_at, args.limit)
            .await
            .map_err(atlassian_error)?;

        Ok(CallToolResult::structured(crate::mcp::wrap_array(value)))
    }

    #[tool(description = "Add a worklog entry to a Jira issue")]
    pub(super) async fn jira_add_worklog(
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
            .map_err(atlassian_error)?;

        Ok(CallToolResult::structured(crate::mcp::wrap_array(value)))
    }

    #[tool(description = "Get Jira issue link types")]
    pub(super) async fn jira_get_link_types(
        &self,
        Parameters(args): Parameters<JiraGetLinkTypesArgs>,
    ) -> Result<CallToolResult, ErrorData> {
        let mut value = self
            .jira_client()?
            .get_link_types()
            .await
            .map_err(atlassian_error)?;

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

        Ok(CallToolResult::structured(crate::mcp::wrap_array(value)))
    }

    #[tool(description = "Link a Jira issue to an epic using parent key")]
    pub(super) async fn jira_link_to_epic(
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
            .map_err(atlassian_error)?;

        Ok(CallToolResult::structured(crate::mcp::wrap_array(value)))
    }

    #[tool(description = "Create a link between two Jira issues")]
    pub(super) async fn jira_create_issue_link(
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
            .map_err(atlassian_error)?;

        Ok(CallToolResult::structured(crate::mcp::wrap_array(value)))
    }

    #[tool(description = "Create a remote link on a Jira issue")]
    pub(super) async fn jira_create_remote_issue_link(
        &self,
        Parameters(args): Parameters<JiraCreateRemoteIssueLinkArgs>,
    ) -> Result<CallToolResult, ErrorData> {
        let (issue_key, payload) = remote_issue_link_payload_from_args(args)?;
        let value = self
            .jira_client()?
            .create_remote_issue_link(issue_key, payload)
            .await
            .map_err(atlassian_error)?;

        Ok(CallToolResult::structured(crate::mcp::wrap_array(value)))
    }

    #[tool(description = "Remove a Jira issue link by id")]
    pub(super) async fn jira_remove_issue_link(
        &self,
        Parameters(args): Parameters<JiraRemoveIssueLinkArgs>,
    ) -> Result<CallToolResult, ErrorData> {
        let value = self
            .jira_client()?
            .remove_issue_link(required_non_empty_arg(args.link_id, "link_id")?)
            .await
            .map_err(atlassian_error)?;

        Ok(CallToolResult::structured(crate::mcp::wrap_array(value)))
    }

    #[tool(description = "Download Jira issue attachments with bounded safe content output")]
    pub(super) async fn jira_download_attachments(
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
            .map_err(atlassian_error)?;

        Ok(CallToolResult::structured(crate::mcp::wrap_array(value)))
    }

    #[tool(description = "Get image attachments for a Jira issue with safe content output")]
    pub(super) async fn jira_get_issue_images(
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
            .map_err(atlassian_error)?;

        Ok(CallToolResult::structured(crate::mcp::wrap_array(value)))
    }

    #[tool(description = "Get Jira Software agile boards")]
    pub(super) async fn jira_get_agile_boards(
        &self,
        Parameters(args): Parameters<JiraGetAgileBoardsArgs>,
    ) -> Result<CallToolResult, ErrorData> {
        let value = self
            .jira_client()?
            .get_agile_boards(args.project_key, args.board_type, args.start_at, args.limit)
            .await
            .map_err(atlassian_error)?;

        Ok(CallToolResult::structured(crate::mcp::wrap_array(value)))
    }

    #[tool(description = "Get issues on a Jira Software agile board")]
    pub(super) async fn jira_get_board_issues(
        &self,
        Parameters(args): Parameters<JiraGetBoardIssuesArgs>,
    ) -> Result<CallToolResult, ErrorData> {
        let fields = parse_optional_string_list_arg(args.fields, "fields")?;
        let value = self
            .jira_client()?
            .get_board_issues(args.board_id, args.jql, fields, args.start_at, args.limit)
            .await
            .map_err(atlassian_error)?;

        Ok(CallToolResult::structured(crate::mcp::wrap_array(value)))
    }

    #[tool(description = "Get sprints for a Jira Software agile board")]
    pub(super) async fn jira_get_sprints_from_board(
        &self,
        Parameters(args): Parameters<JiraGetSprintsFromBoardArgs>,
    ) -> Result<CallToolResult, ErrorData> {
        let state = parse_optional_string_list_arg(args.state, "state")?;
        let value = self
            .jira_client()?
            .get_sprints_from_board(args.board_id, state, args.start_at, args.limit)
            .await
            .map_err(atlassian_error)?;

        Ok(CallToolResult::structured(crate::mcp::wrap_array(value)))
    }

    #[tool(description = "Get issues for a Jira Software sprint")]
    pub(super) async fn jira_get_sprint_issues(
        &self,
        Parameters(args): Parameters<JiraGetSprintIssuesArgs>,
    ) -> Result<CallToolResult, ErrorData> {
        let fields = parse_optional_string_list_arg(args.fields, "fields")?;
        let value = self
            .jira_client()?
            .get_sprint_issues(args.sprint_id, fields, args.start_at, args.limit)
            .await
            .map_err(atlassian_error)?;

        Ok(CallToolResult::structured(crate::mcp::wrap_array(value)))
    }

    #[tool(description = "Create a Jira Software sprint")]
    pub(super) async fn jira_create_sprint(
        &self,
        Parameters(args): Parameters<JiraCreateSprintArgs>,
    ) -> Result<CallToolResult, ErrorData> {
        let value = self
            .jira_client()?
            .create_sprint(create_sprint_payload_from_args(args)?)
            .await
            .map_err(atlassian_error)?;

        Ok(CallToolResult::structured(crate::mcp::wrap_array(value)))
    }

    #[tool(description = "Update a Jira Software sprint")]
    pub(super) async fn jira_update_sprint(
        &self,
        Parameters(args): Parameters<JiraUpdateSprintArgs>,
    ) -> Result<CallToolResult, ErrorData> {
        let (sprint_id, payload) = update_sprint_payload_from_args(args)?;
        let value = self
            .jira_client()?
            .update_sprint(sprint_id, payload)
            .await
            .map_err(atlassian_error)?;

        Ok(CallToolResult::structured(crate::mcp::wrap_array(value)))
    }

    #[tool(description = "Add Jira issues to a sprint")]
    pub(super) async fn jira_add_issues_to_sprint(
        &self,
        Parameters(args): Parameters<JiraAddIssuesToSprintArgs>,
    ) -> Result<CallToolResult, ErrorData> {
        let issue_keys = parse_required_string_list_arg(args.issue_keys, "issue_keys")?;
        let value = self
            .jira_client()?
            .add_issues_to_sprint(args.sprint_id, issue_keys)
            .await
            .map_err(atlassian_error)?;

        Ok(CallToolResult::structured(crate::mcp::wrap_array(value)))
    }

    #[tool(description = "Get the Jira Service Management service desk for a project")]
    pub(super) async fn jira_get_service_desk_for_project(
        &self,
        Parameters(args): Parameters<JiraGetServiceDeskForProjectArgs>,
    ) -> Result<CallToolResult, ErrorData> {
        let value = self
            .jira_client()?
            .get_service_desk_for_project(required_non_empty_arg(args.project_key, "project_key")?)
            .await
            .map_err(atlassian_error)?;

        Ok(CallToolResult::structured(crate::mcp::wrap_array(value)))
    }

    #[tool(description = "Get queues for a Jira Service Management service desk")]
    pub(super) async fn jira_get_service_desk_queues(
        &self,
        Parameters(args): Parameters<JiraGetServiceDeskQueuesArgs>,
    ) -> Result<CallToolResult, ErrorData> {
        let value = self
            .jira_client()?
            .get_service_desk_queues(args.service_desk_id, args.start_at, args.limit)
            .await
            .map_err(atlassian_error)?;

        Ok(CallToolResult::structured(crate::mcp::wrap_array(value)))
    }

    #[tool(description = "Get issues for a Jira Service Management queue")]
    pub(super) async fn jira_get_queue_issues(
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
            .map_err(atlassian_error)?;

        Ok(CallToolResult::structured(crate::mcp::wrap_array(value)))
    }

    #[tool(description = "Get Jira Forms or ProForma forms for an issue")]
    pub(super) async fn jira_get_issue_proforma_forms(
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
            .map_err(atlassian_error)?;

        Ok(CallToolResult::structured(crate::mcp::wrap_array(value)))
    }

    #[tool(description = "Get details for a Jira Form or ProForma form")]
    pub(super) async fn jira_get_proforma_form_details(
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
            .map_err(atlassian_error)?;

        Ok(CallToolResult::structured(crate::mcp::wrap_array(value)))
    }

    #[tool(description = "Update answers on a Jira Form or ProForma form")]
    pub(super) async fn jira_update_proforma_form_answers(
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
            .map_err(atlassian_error)?;

        Ok(CallToolResult::structured(crate::mcp::wrap_array(value)))
    }

    #[tool(description = "Get Jira issue date and status timing information")]
    pub(super) async fn jira_get_issue_dates(
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
            .map_err(atlassian_error)?;

        Ok(CallToolResult::structured(crate::mcp::wrap_array(value)))
    }

    #[tool(description = "Get Jira Service Management SLA metrics for an issue")]
    pub(super) async fn jira_get_issue_sla(
        &self,
        Parameters(args): Parameters<JiraGetIssueSlaArgs>,
    ) -> Result<CallToolResult, ErrorData> {
        let metrics = parse_optional_string_list_arg(args.metrics, "metrics")?;
        let value = self
            .jira_client()?
            .get_issue_sla(
                required_non_empty_arg(args.issue_key, "issue_key")?,
                metrics,
                args.include_raw_dates.unwrap_or(false),
            )
            .await
            .map_err(atlassian_error)?;

        Ok(CallToolResult::structured(crate::mcp::wrap_array(value)))
    }

    #[tool(description = "Get Jira development information for an issue")]
    pub(super) async fn jira_get_issue_development_info(
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
            .map_err(atlassian_error)?;

        Ok(CallToolResult::structured(crate::mcp::wrap_array(value)))
    }

    #[tool(description = "Get Jira development information for multiple issues")]
    pub(super) async fn jira_get_issues_development_info(
        &self,
        Parameters(args): Parameters<JiraGetIssuesDevelopmentInfoArgs>,
    ) -> Result<CallToolResult, ErrorData> {
        let issue_keys = parse_required_string_list_arg(args.issue_keys, "issue_keys")?;
        let value = self
            .jira_client()?
            .get_issues_development_info(issue_keys, args.application_type, args.data_type)
            .await
            .map_err(atlassian_error)?;

        Ok(CallToolResult::structured(crate::mcp::wrap_array(value)))
    }
}

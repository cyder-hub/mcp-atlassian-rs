use crate::{
    jira::{
        client::{
            AttachmentFetchOptions, DEFAULT_ATTACHMENT_MAX_BYTES, FieldOptionsRequest,
            GetIssueRequest, SearchRequest,
        },
        tools::{
            JiraAddCommentArgs, JiraAddIssuesToSprintArgs, JiraAddWatcherArgs, JiraAddWorklogArgs,
            JiraCreateIssueArgs, JiraCreateIssueLinkArgs, JiraCreateIssuesArgs,
            JiraCreateProjectVersionArgs, JiraCreateProjectVersionsArgs,
            JiraCreateRemoteIssueLinkArgs, JiraCreateSprintArgs, JiraDeleteIssueArgs,
            JiraDeleteIssueLinkArgs, JiraEditCommentArgs, JiraGetFieldOptionsArgs,
            JiraGetIssueArgs, JiraGetIssueAttachmentsArgs, JiraGetIssueChangelogsArgs,
            JiraGetIssueDevelopmentArgs, JiraGetIssueFormArgs, JiraGetIssueImagesArgs,
            JiraGetIssueSlaMetricsArgs, JiraGetIssueTimelineArgs, JiraGetIssuesDevelopmentArgs,
            JiraGetProjectIssuesArgs, JiraGetServiceDeskForProjectArgs, JiraGetTransitionsArgs,
            JiraGetUserArgs, JiraListAgileBoardsArgs, JiraListBoardIssuesArgs,
            JiraListBoardSprintsArgs, JiraListIssueFormsArgs, JiraListIssueLinkTypesArgs,
            JiraListIssueWatchersArgs, JiraListIssueWorklogsArgs, JiraListProjectComponentsArgs,
            JiraListProjectVersionsArgs, JiraListProjectsArgs, JiraListServiceDeskQueueIssuesArgs,
            JiraListServiceDeskQueuesArgs, JiraListSprintIssuesArgs, JiraRemoveWatcherArgs,
            JiraSearchArgs, JiraSearchFieldsArgs, JiraSetIssueParentArgs, JiraTransitionIssueArgs,
            JiraUpdateIssueArgs, JiraUpdateIssueFormAnswersArgs, JiraUpdateSprintArgs,
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
    #[tool(name = "jira_get_issue")]
    pub(super) async fn get_issue(
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

    #[tool(name = "jira_search_issues")]
    pub(super) async fn search_issues(
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

    #[tool(name = "jira_list_project_issues")]
    pub(super) async fn list_project_issues(
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

    #[tool(name = "jira_search_fields")]
    pub(super) async fn search_fields(
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

    #[tool(name = "jira_list_field_options")]
    pub(super) async fn list_field_options(
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

    #[tool(name = "jira_add_issue_comment")]
    pub(super) async fn add_issue_comment(
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

    #[tool(name = "jira_update_issue_comment")]
    pub(super) async fn update_issue_comment(
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

    #[tool(name = "jira_list_issue_transitions")]
    pub(super) async fn list_issue_transitions(
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

    #[tool(name = "jira_transition_issue")]
    pub(super) async fn transition_issue(
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

    #[tool(name = "jira_create_issue")]
    pub(super) async fn create_issue(
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

    #[tool(name = "jira_create_issues")]
    pub(super) async fn create_issues(
        &self,
        Parameters(args): Parameters<JiraCreateIssuesArgs>,
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

    #[tool(name = "jira_get_issue_changelogs")]
    pub(super) async fn get_issue_changelogs(
        &self,
        Parameters(args): Parameters<JiraGetIssueChangelogsArgs>,
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

    #[tool(name = "jira_update_issue")]
    pub(super) async fn update_issue(
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

    #[tool(name = "jira_delete_issue")]
    pub(super) async fn delete_issue(
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

    #[tool(name = "jira_list_projects")]
    pub(super) async fn list_projects(
        &self,
        Parameters(args): Parameters<JiraListProjectsArgs>,
    ) -> Result<CallToolResult, ErrorData> {
        let value = self
            .jira_client()?
            .get_all_projects(args.include_archived.unwrap_or(false))
            .await
            .map_err(atlassian_error)?;

        Ok(CallToolResult::structured(crate::mcp::wrap_array(value)))
    }

    #[tool(name = "jira_list_project_versions")]
    pub(super) async fn list_project_versions(
        &self,
        Parameters(args): Parameters<JiraListProjectVersionsArgs>,
    ) -> Result<CallToolResult, ErrorData> {
        let value = self
            .jira_client()?
            .get_project_versions(args.project_key)
            .await
            .map_err(atlassian_error)?;

        Ok(CallToolResult::structured(crate::mcp::wrap_array(value)))
    }

    #[tool(name = "jira_list_project_components")]
    pub(super) async fn list_project_components(
        &self,
        Parameters(args): Parameters<JiraListProjectComponentsArgs>,
    ) -> Result<CallToolResult, ErrorData> {
        let value = self
            .jira_client()?
            .get_project_components(args.project_key)
            .await
            .map_err(atlassian_error)?;

        Ok(CallToolResult::structured(crate::mcp::wrap_array(value)))
    }

    #[tool(name = "jira_create_project_version")]
    pub(super) async fn create_project_version(
        &self,
        Parameters(args): Parameters<JiraCreateProjectVersionArgs>,
    ) -> Result<CallToolResult, ErrorData> {
        let value = self
            .jira_client()?
            .create_version(version_payload_from_args(args)?)
            .await
            .map_err(atlassian_error)?;

        Ok(CallToolResult::structured(crate::mcp::wrap_array(value)))
    }

    #[tool(name = "jira_create_project_versions")]
    pub(super) async fn create_project_versions(
        &self,
        Parameters(args): Parameters<JiraCreateProjectVersionsArgs>,
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

    #[tool(name = "jira_get_user")]
    pub(super) async fn get_user(
        &self,
        Parameters(args): Parameters<JiraGetUserArgs>,
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

    #[tool(name = "jira_list_issue_watchers")]
    pub(super) async fn list_issue_watchers(
        &self,
        Parameters(args): Parameters<JiraListIssueWatchersArgs>,
    ) -> Result<CallToolResult, ErrorData> {
        let value = self
            .jira_client()?
            .get_issue_watchers(args.issue_key)
            .await
            .map_err(atlassian_error)?;

        Ok(CallToolResult::structured(crate::mcp::wrap_array(value)))
    }

    #[tool(name = "jira_add_issue_watcher")]
    pub(super) async fn add_issue_watcher(
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

    #[tool(name = "jira_remove_issue_watcher")]
    pub(super) async fn remove_issue_watcher(
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

    #[tool(name = "jira_list_issue_worklogs")]
    pub(super) async fn list_issue_worklogs(
        &self,
        Parameters(args): Parameters<JiraListIssueWorklogsArgs>,
    ) -> Result<CallToolResult, ErrorData> {
        let value = self
            .jira_client()?
            .get_worklog(args.issue_key, args.start_at, args.limit)
            .await
            .map_err(atlassian_error)?;

        Ok(CallToolResult::structured(crate::mcp::wrap_array(value)))
    }

    #[tool(name = "jira_add_issue_worklog")]
    pub(super) async fn add_issue_worklog(
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

    #[tool(name = "jira_list_issue_link_types")]
    pub(super) async fn list_issue_link_types(
        &self,
        Parameters(args): Parameters<JiraListIssueLinkTypesArgs>,
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

    #[tool(name = "jira_set_issue_parent")]
    pub(super) async fn set_issue_parent(
        &self,
        Parameters(args): Parameters<JiraSetIssueParentArgs>,
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

    #[tool(name = "jira_create_issue_link")]
    pub(super) async fn create_issue_link(
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

    #[tool(name = "jira_create_remote_issue_link")]
    pub(super) async fn create_remote_issue_link(
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

    #[tool(name = "jira_delete_issue_link")]
    pub(super) async fn delete_issue_link(
        &self,
        Parameters(args): Parameters<JiraDeleteIssueLinkArgs>,
    ) -> Result<CallToolResult, ErrorData> {
        let value = self
            .jira_client()?
            .remove_issue_link(required_non_empty_arg(args.link_id, "link_id")?)
            .await
            .map_err(atlassian_error)?;

        Ok(CallToolResult::structured(crate::mcp::wrap_array(value)))
    }

    #[tool(name = "jira_get_issue_attachments")]
    pub(super) async fn get_issue_attachments(
        &self,
        Parameters(args): Parameters<JiraGetIssueAttachmentsArgs>,
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

    #[tool(name = "jira_get_issue_image_attachments")]
    pub(super) async fn get_issue_image_attachments(
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

    #[tool(name = "jira_list_agile_boards")]
    pub(super) async fn list_agile_boards(
        &self,
        Parameters(args): Parameters<JiraListAgileBoardsArgs>,
    ) -> Result<CallToolResult, ErrorData> {
        let value = self
            .jira_client()?
            .get_agile_boards(args.project_key, args.board_type, args.start_at, args.limit)
            .await
            .map_err(atlassian_error)?;

        Ok(CallToolResult::structured(crate::mcp::wrap_array(value)))
    }

    #[tool(name = "jira_list_board_issues")]
    pub(super) async fn list_board_issues(
        &self,
        Parameters(args): Parameters<JiraListBoardIssuesArgs>,
    ) -> Result<CallToolResult, ErrorData> {
        let fields = parse_optional_string_list_arg(args.fields, "fields")?;
        let value = self
            .jira_client()?
            .get_board_issues(args.board_id, args.jql, fields, args.start_at, args.limit)
            .await
            .map_err(atlassian_error)?;

        Ok(CallToolResult::structured(crate::mcp::wrap_array(value)))
    }

    #[tool(name = "jira_list_board_sprints")]
    pub(super) async fn list_board_sprints(
        &self,
        Parameters(args): Parameters<JiraListBoardSprintsArgs>,
    ) -> Result<CallToolResult, ErrorData> {
        let state = parse_optional_string_list_arg(args.state, "state")?;
        let value = self
            .jira_client()?
            .get_sprints_from_board(args.board_id, state, args.start_at, args.limit)
            .await
            .map_err(atlassian_error)?;

        Ok(CallToolResult::structured(crate::mcp::wrap_array(value)))
    }

    #[tool(name = "jira_list_sprint_issues")]
    pub(super) async fn list_sprint_issues(
        &self,
        Parameters(args): Parameters<JiraListSprintIssuesArgs>,
    ) -> Result<CallToolResult, ErrorData> {
        let fields = parse_optional_string_list_arg(args.fields, "fields")?;
        let value = self
            .jira_client()?
            .get_sprint_issues(args.sprint_id, fields, args.start_at, args.limit)
            .await
            .map_err(atlassian_error)?;

        Ok(CallToolResult::structured(crate::mcp::wrap_array(value)))
    }

    #[tool(name = "jira_create_sprint")]
    pub(super) async fn create_sprint(
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

    #[tool(name = "jira_update_sprint")]
    pub(super) async fn update_sprint(
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

    #[tool(name = "jira_add_issues_to_sprint")]
    pub(super) async fn add_issues_to_sprint(
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

    #[tool(name = "jira_get_project_service_desk")]
    pub(super) async fn get_project_service_desk(
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

    #[tool(name = "jira_list_service_desk_queues")]
    pub(super) async fn list_service_desk_queues(
        &self,
        Parameters(args): Parameters<JiraListServiceDeskQueuesArgs>,
    ) -> Result<CallToolResult, ErrorData> {
        let value = self
            .jira_client()?
            .get_service_desk_queues(args.service_desk_id, args.start_at, args.limit)
            .await
            .map_err(atlassian_error)?;

        Ok(CallToolResult::structured(crate::mcp::wrap_array(value)))
    }

    #[tool(name = "jira_list_service_desk_queue_issues")]
    pub(super) async fn list_service_desk_queue_issues(
        &self,
        Parameters(args): Parameters<JiraListServiceDeskQueueIssuesArgs>,
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

    #[tool(name = "jira_list_issue_forms")]
    pub(super) async fn list_issue_forms(
        &self,
        Parameters(args): Parameters<JiraListIssueFormsArgs>,
    ) -> Result<CallToolResult, ErrorData> {
        let value = self
            .jira_client()?
            .get_issue_forms(
                required_non_empty_arg(args.issue_key, "issue_key")?,
                self.context.atlassian_oauth_cloud_id(),
            )
            .await
            .map_err(atlassian_error)?;

        Ok(CallToolResult::structured(crate::mcp::wrap_array(value)))
    }

    #[tool(name = "jira_get_issue_form")]
    pub(super) async fn get_issue_form(
        &self,
        Parameters(args): Parameters<JiraGetIssueFormArgs>,
    ) -> Result<CallToolResult, ErrorData> {
        let value = self
            .jira_client()?
            .get_issue_form(
                required_non_empty_arg(args.issue_key, "issue_key")?,
                required_non_empty_arg(args.form_id, "form_id")?,
                self.context.atlassian_oauth_cloud_id(),
            )
            .await
            .map_err(atlassian_error)?;

        Ok(CallToolResult::structured(crate::mcp::wrap_array(value)))
    }

    #[tool(name = "jira_update_issue_form_answers")]
    pub(super) async fn update_issue_form_answers(
        &self,
        Parameters(args): Parameters<JiraUpdateIssueFormAnswersArgs>,
    ) -> Result<CallToolResult, ErrorData> {
        let answers = parse_required_object_list_arg(args.answers, "answers")?;
        let value = self
            .jira_client()?
            .update_issue_form_answers(
                required_non_empty_arg(args.issue_key, "issue_key")?,
                required_non_empty_arg(args.form_id, "form_id")?,
                answers,
                self.context.atlassian_oauth_cloud_id(),
            )
            .await
            .map_err(atlassian_error)?;

        Ok(CallToolResult::structured(crate::mcp::wrap_array(value)))
    }

    #[tool(name = "jira_get_issue_timeline")]
    pub(super) async fn get_issue_timeline(
        &self,
        Parameters(args): Parameters<JiraGetIssueTimelineArgs>,
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

    #[tool(name = "jira_get_issue_sla_metrics")]
    pub(super) async fn get_issue_sla_metrics(
        &self,
        Parameters(args): Parameters<JiraGetIssueSlaMetricsArgs>,
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

    #[tool(name = "jira_get_issue_development")]
    pub(super) async fn get_issue_development(
        &self,
        Parameters(args): Parameters<JiraGetIssueDevelopmentArgs>,
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

    #[tool(name = "jira_get_issues_development")]
    pub(super) async fn get_issues_development(
        &self,
        Parameters(args): Parameters<JiraGetIssuesDevelopmentArgs>,
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

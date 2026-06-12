use serde_json::{Map, Value, json};

use crate::{
    context::AppContext,
    jira::{
        client::{
            AttachmentFetchOptions, DEFAULT_ATTACHMENT_MAX_BYTES, FieldOptionsRequest,
            GetIssueRequest, SearchRequest,
        },
        config::JiraDeployment,
        formatting::{
            comment_body_for_deployment, merge_optional_objects, parse_optional_object,
            parse_optional_string_list, parse_required_object, parse_required_object_list,
            parse_required_string_list,
        },
        tools::*,
    },
};

use super::{OperationError, OperationResult, guard_operation, jira_client};

pub async fn get_issue(
    context: &AppContext,
    args: JiraGetIssueArgs,
) -> Result<OperationResult, OperationError> {
    guard_operation(JIRA_GET_ISSUE_TOOL_NAME, context)?;
    let fields =
        parse_optional_string_list(args.fields, "fields").map_err(OperationError::from_upstream)?;
    let expand =
        parse_optional_string_list(args.expand, "expand").map_err(OperationError::from_upstream)?;
    let properties = parse_optional_string_list(args.properties, "properties")
        .map_err(OperationError::from_upstream)?;
    let value = jira_client(context)?
        .get_issue(GetIssueRequest {
            issue_key: args.issue_key,
            fields,
            expand,
            comment_limit: args.comment_limit,
            properties,
            update_history: args.update_history,
        })
        .await
        .map_err(OperationError::from_upstream)?;

    Ok(structured(value))
}

pub async fn search_issues(
    context: &AppContext,
    args: JiraSearchArgs,
) -> Result<OperationResult, OperationError> {
    guard_operation(JIRA_SEARCH_TOOL_NAME, context)?;
    let fields =
        parse_optional_string_list(args.fields, "fields").map_err(OperationError::from_upstream)?;
    let expand =
        parse_optional_string_list(args.expand, "expand").map_err(OperationError::from_upstream)?;
    let projects_filter = parse_optional_string_list(args.projects_filter, "projects_filter")
        .map_err(OperationError::from_upstream)?;
    let value = jira_client(context)?
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
        .map_err(OperationError::from_upstream)?;

    Ok(structured(value))
}

pub async fn list_project_issues(
    context: &AppContext,
    args: JiraGetProjectIssuesArgs,
) -> Result<OperationResult, OperationError> {
    guard_operation(JIRA_GET_PROJECT_ISSUES_TOOL_NAME, context)?;
    let value = jira_client(context)?
        .get_project_issues(args.project_key, args.limit, args.start_at)
        .await
        .map_err(OperationError::from_upstream)?;

    Ok(structured(value))
}

pub async fn search_fields(
    context: &AppContext,
    args: JiraSearchFieldsArgs,
) -> Result<OperationResult, OperationError> {
    guard_operation(JIRA_SEARCH_FIELDS_TOOL_NAME, context)?;
    let value = jira_client(context)?
        .search_fields(args.keyword, args.limit)
        .await
        .map_err(OperationError::from_upstream)?;

    Ok(structured(value))
}

pub async fn list_field_options(
    context: &AppContext,
    args: JiraGetFieldOptionsArgs,
) -> Result<OperationResult, OperationError> {
    guard_operation(JIRA_GET_FIELD_OPTIONS_TOOL_NAME, context)?;
    let value = jira_client(context)?
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
        .map_err(OperationError::from_upstream)?;

    Ok(structured(value))
}

pub async fn add_issue_comment(
    context: &AppContext,
    args: JiraAddCommentArgs,
) -> Result<OperationResult, OperationError> {
    guard_operation(JIRA_ADD_COMMENT_TOOL_NAME, context)?;
    let visibility = parse_optional_object(args.visibility, "visibility")
        .map_err(OperationError::from_upstream)?;
    let value = jira_client(context)?
        .add_comment(args.issue_key, args.body, visibility)
        .await
        .map_err(OperationError::from_upstream)?;

    Ok(structured(value))
}

pub async fn update_issue_comment(
    context: &AppContext,
    args: JiraEditCommentArgs,
) -> Result<OperationResult, OperationError> {
    guard_operation(JIRA_EDIT_COMMENT_TOOL_NAME, context)?;
    let visibility = parse_optional_object(args.visibility, "visibility")
        .map_err(OperationError::from_upstream)?;
    let value = jira_client(context)?
        .edit_comment(args.issue_key, args.comment_id, args.body, visibility)
        .await
        .map_err(OperationError::from_upstream)?;

    Ok(structured(value))
}

pub async fn list_issue_transitions(
    context: &AppContext,
    args: JiraGetTransitionsArgs,
) -> Result<OperationResult, OperationError> {
    guard_operation(JIRA_GET_TRANSITIONS_TOOL_NAME, context)?;
    let value = jira_client(context)?
        .get_transitions(args.issue_key)
        .await
        .map_err(OperationError::from_upstream)?;

    Ok(structured(value))
}

pub async fn transition_issue(
    context: &AppContext,
    args: JiraTransitionIssueArgs,
) -> Result<OperationResult, OperationError> {
    guard_operation(JIRA_TRANSITION_ISSUE_TOOL_NAME, context)?;
    let fields =
        parse_optional_object(args.fields, "fields").map_err(OperationError::from_upstream)?;
    let value = jira_client(context)?
        .transition_issue(args.issue_key, args.transition_id, fields, args.comment)
        .await
        .map_err(OperationError::from_upstream)?;

    Ok(structured(value))
}

pub async fn create_issue(
    context: &AppContext,
    args: JiraCreateIssueArgs,
) -> Result<OperationResult, OperationError> {
    guard_operation(JIRA_CREATE_ISSUE_TOOL_NAME, context)?;
    let deployment = jira_deployment(context)?;
    let fields = create_issue_fields_from_args(args, deployment)?;
    let value = jira_client(context)?
        .create_issue(fields)
        .await
        .map_err(OperationError::from_upstream)?;

    Ok(structured(value))
}

pub async fn create_issues(
    context: &AppContext,
    args: JiraCreateIssuesArgs,
) -> Result<OperationResult, OperationError> {
    guard_operation(JIRA_CREATE_ISSUES_TOOL_NAME, context)?;
    let deployment = jira_deployment(context)?;
    let issue_updates = batch_create_issue_updates_from_args(args.issues, deployment)?;
    let value = jira_client(context)?
        .batch_create_issues(issue_updates, args.validate_only.unwrap_or(false))
        .await
        .map_err(OperationError::from_upstream)?;

    Ok(structured(value))
}

pub async fn get_issue_changelogs(
    context: &AppContext,
    args: JiraGetIssueChangelogsArgs,
) -> Result<OperationResult, OperationError> {
    guard_operation(JIRA_GET_ISSUE_CHANGELOGS_TOOL_NAME, context)?;
    let issue_ids_or_keys = parse_required_string_list(args.issue_ids_or_keys, "issue_ids_or_keys")
        .map_err(OperationError::from_upstream)?;
    let fields =
        parse_optional_string_list(args.fields, "fields").map_err(OperationError::from_upstream)?;
    let limit = optional_positive_i64_arg(args.limit, "limit")?;
    let value = jira_client(context)?
        .batch_get_changelogs(issue_ids_or_keys, fields, limit)
        .await
        .map_err(OperationError::from_upstream)?;

    Ok(structured(value))
}

pub async fn update_issue(
    context: &AppContext,
    args: JiraUpdateIssueArgs,
) -> Result<OperationResult, OperationError> {
    guard_operation(JIRA_UPDATE_ISSUE_TOOL_NAME, context)?;
    let deployment = jira_deployment(context)?;
    let (fields, additional_fields) = update_issue_fields_from_args(args, deployment)?;
    let value = jira_client(context)?
        .update_issue(
            fields.issue_key,
            fields.fields,
            additional_fields,
            fields.notify_users,
        )
        .await
        .map_err(OperationError::from_upstream)?;

    Ok(structured(value))
}

pub async fn delete_issue(
    context: &AppContext,
    args: JiraDeleteIssueArgs,
) -> Result<OperationResult, OperationError> {
    guard_operation(JIRA_DELETE_ISSUE_TOOL_NAME, context)?;
    let value = jira_client(context)?
        .delete_issue(args.issue_key, args.delete_subtasks.unwrap_or(false))
        .await
        .map_err(OperationError::from_upstream)?;

    Ok(structured(value))
}

pub async fn list_projects(
    context: &AppContext,
    args: JiraListProjectsArgs,
) -> Result<OperationResult, OperationError> {
    guard_operation(JIRA_LIST_PROJECTS_TOOL_NAME, context)?;
    let value = jira_client(context)?
        .get_all_projects(args.include_archived.unwrap_or(false))
        .await
        .map_err(OperationError::from_upstream)?;

    Ok(structured(value))
}

pub async fn list_project_components(
    context: &AppContext,
    args: JiraListProjectComponentsArgs,
) -> Result<OperationResult, OperationError> {
    guard_operation(JIRA_LIST_PROJECT_COMPONENTS_TOOL_NAME, context)?;
    let value = jira_client(context)?
        .get_project_components(args.project_key)
        .await
        .map_err(OperationError::from_upstream)?;

    Ok(structured(value))
}

pub async fn list_project_versions(
    context: &AppContext,
    args: JiraListProjectVersionsArgs,
) -> Result<OperationResult, OperationError> {
    guard_operation(JIRA_LIST_PROJECT_VERSIONS_TOOL_NAME, context)?;
    let value = jira_client(context)?
        .get_project_versions(args.project_key)
        .await
        .map_err(OperationError::from_upstream)?;

    Ok(structured(value))
}

pub async fn create_project_version(
    context: &AppContext,
    args: JiraCreateProjectVersionArgs,
) -> Result<OperationResult, OperationError> {
    guard_operation(JIRA_CREATE_PROJECT_VERSION_TOOL_NAME, context)?;
    let value = jira_client(context)?
        .create_version(version_payload_from_args(args)?)
        .await
        .map_err(OperationError::from_upstream)?;

    Ok(structured(value))
}

pub async fn create_project_versions(
    context: &AppContext,
    args: JiraCreateProjectVersionsArgs,
) -> Result<OperationResult, OperationError> {
    guard_operation(JIRA_CREATE_PROJECT_VERSIONS_TOOL_NAME, context)?;
    let project_key = required_non_empty_arg(args.project_key, "project_key")?;
    let versions = parse_required_object_list(args.versions, "versions")
        .map_err(OperationError::from_upstream)?
        .into_iter()
        .map(|version| version_payload_from_value(version, &project_key))
        .collect::<Result<Vec<_>, _>>()?;
    let value = jira_client(context)?
        .batch_create_versions(versions)
        .await
        .map_err(OperationError::from_upstream)?;

    Ok(structured(value))
}

pub async fn get_user(
    context: &AppContext,
    args: JiraGetUserArgs,
) -> Result<OperationResult, OperationError> {
    guard_operation(JIRA_GET_USER_TOOL_NAME, context)?;
    let value = jira_client(context)?
        .get_user_profile(required_non_empty_arg(
            args.user_identifier
                .unwrap_or_else(|| "currentuser()".to_string()),
            "user_identifier",
        )?)
        .await
        .map_err(OperationError::from_upstream)?;

    Ok(structured(value))
}

pub async fn list_issue_watchers(
    context: &AppContext,
    args: JiraListIssueWatchersArgs,
) -> Result<OperationResult, OperationError> {
    guard_operation(JIRA_LIST_ISSUE_WATCHERS_TOOL_NAME, context)?;
    let value = jira_client(context)?
        .get_issue_watchers(args.issue_key)
        .await
        .map_err(OperationError::from_upstream)?;

    Ok(structured(value))
}

pub async fn add_issue_watcher(
    context: &AppContext,
    args: JiraAddWatcherArgs,
) -> Result<OperationResult, OperationError> {
    guard_operation(JIRA_ADD_WATCHER_TOOL_NAME, context)?;
    let value = jira_client(context)?
        .add_watcher(
            args.issue_key,
            required_non_empty_arg(args.user_identifier, "user_identifier")?,
        )
        .await
        .map_err(OperationError::from_upstream)?;

    Ok(structured(value))
}

pub async fn remove_issue_watcher(
    context: &AppContext,
    args: JiraRemoveWatcherArgs,
) -> Result<OperationResult, OperationError> {
    guard_operation(JIRA_REMOVE_WATCHER_TOOL_NAME, context)?;
    let value = jira_client(context)?
        .remove_watcher(
            args.issue_key,
            required_non_empty_arg(args.user_identifier, "user_identifier")?,
        )
        .await
        .map_err(OperationError::from_upstream)?;

    Ok(structured(value))
}

pub async fn list_issue_worklogs(
    context: &AppContext,
    args: JiraListIssueWorklogsArgs,
) -> Result<OperationResult, OperationError> {
    guard_operation(JIRA_LIST_ISSUE_WORKLOGS_TOOL_NAME, context)?;
    let value = jira_client(context)?
        .get_worklog(args.issue_key, args.start_at, args.limit)
        .await
        .map_err(OperationError::from_upstream)?;

    Ok(structured(value))
}

pub async fn add_issue_worklog(
    context: &AppContext,
    args: JiraAddWorklogArgs,
) -> Result<OperationResult, OperationError> {
    guard_operation(JIRA_ADD_WORKLOG_TOOL_NAME, context)?;
    let deployment = jira_deployment(context)?;
    let (issue_key, payload, query) = add_worklog_payload_from_args(args, deployment)?;
    let value = jira_client(context)?
        .add_worklog(issue_key, payload, query)
        .await
        .map_err(OperationError::from_upstream)?;

    Ok(structured(value))
}

pub async fn list_issue_link_types(
    context: &AppContext,
    args: JiraListIssueLinkTypesArgs,
) -> Result<OperationResult, OperationError> {
    guard_operation(JIRA_LIST_ISSUE_LINK_TYPES_TOOL_NAME, context)?;
    let mut value = jira_client(context)?
        .get_link_types()
        .await
        .map_err(OperationError::from_upstream)?;

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

    Ok(structured(value))
}

pub async fn set_issue_parent(
    context: &AppContext,
    args: JiraSetIssueParentArgs,
) -> Result<OperationResult, OperationError> {
    guard_operation(JIRA_SET_ISSUE_PARENT_TOOL_NAME, context)?;
    let value = jira_client(context)?
        .link_to_epic(
            required_non_empty_arg(args.issue_key, "issue_key")?,
            required_non_empty_arg(args.epic_key, "epic_key")?,
        )
        .await
        .map_err(OperationError::from_upstream)?;

    Ok(structured(value))
}

pub async fn create_issue_link(
    context: &AppContext,
    args: JiraCreateIssueLinkArgs,
) -> Result<OperationResult, OperationError> {
    guard_operation(JIRA_CREATE_ISSUE_LINK_TOOL_NAME, context)?;
    let deployment = jira_deployment(context)?;
    let value = jira_client(context)?
        .create_issue_link(issue_link_payload_from_args(args, deployment)?)
        .await
        .map_err(OperationError::from_upstream)?;

    Ok(structured(value))
}

pub async fn create_remote_issue_link(
    context: &AppContext,
    args: JiraCreateRemoteIssueLinkArgs,
) -> Result<OperationResult, OperationError> {
    guard_operation(JIRA_CREATE_REMOTE_ISSUE_LINK_TOOL_NAME, context)?;
    let (issue_key, payload) = remote_issue_link_payload_from_args(args)?;
    let value = jira_client(context)?
        .create_remote_issue_link(issue_key, payload)
        .await
        .map_err(OperationError::from_upstream)?;

    Ok(structured(value))
}

pub async fn delete_issue_link(
    context: &AppContext,
    args: JiraDeleteIssueLinkArgs,
) -> Result<OperationResult, OperationError> {
    guard_operation(JIRA_DELETE_ISSUE_LINK_TOOL_NAME, context)?;
    let value = jira_client(context)?
        .remove_issue_link(required_non_empty_arg(args.link_id, "link_id")?)
        .await
        .map_err(OperationError::from_upstream)?;

    Ok(structured(value))
}

pub async fn get_issue_attachments(
    context: &AppContext,
    args: JiraGetIssueAttachmentsArgs,
) -> Result<OperationResult, OperationError> {
    guard_operation(JIRA_GET_ISSUE_ATTACHMENTS_TOOL_NAME, context)?;
    let attachment_ids = parse_optional_string_list(args.attachment_ids, "attachment_ids")
        .map_err(OperationError::from_upstream)?;
    let max_bytes = optional_positive_u64_arg(args.max_bytes, "max_bytes")?
        .unwrap_or(DEFAULT_ATTACHMENT_MAX_BYTES);
    let value = jira_client(context)?
        .get_safe_issue_attachments(
            required_non_empty_arg(args.issue_key, "issue_key")?,
            AttachmentFetchOptions {
                attachment_ids,
                filename_contains: optional_non_empty_arg(args.filename_contains),
                media_type: optional_non_empty_arg(args.media_type),
                include_content: args.include_content.unwrap_or(false),
                images_only: false,
                max_bytes,
            },
        )
        .await
        .map_err(OperationError::from_upstream)?;

    Ok(structured(value))
}

pub async fn get_issue_image_attachments(
    context: &AppContext,
    args: JiraGetIssueImagesArgs,
) -> Result<OperationResult, OperationError> {
    guard_operation(JIRA_GET_ISSUE_IMAGES_TOOL_NAME, context)?;
    let max_bytes = optional_positive_u64_arg(args.max_bytes, "max_bytes")?
        .unwrap_or(DEFAULT_ATTACHMENT_MAX_BYTES);
    let value = jira_client(context)?
        .get_safe_issue_attachments(
            required_non_empty_arg(args.issue_key, "issue_key")?,
            AttachmentFetchOptions {
                attachment_ids: None,
                filename_contains: None,
                media_type: None,
                include_content: args.include_content.unwrap_or(false),
                images_only: true,
                max_bytes,
            },
        )
        .await
        .map_err(OperationError::from_upstream)?;

    Ok(structured(value))
}

pub async fn list_agile_boards(
    context: &AppContext,
    args: JiraListAgileBoardsArgs,
) -> Result<OperationResult, OperationError> {
    guard_operation(JIRA_LIST_AGILE_BOARDS_TOOL_NAME, context)?;
    let mut value = jira_client(context)?
        .get_agile_boards(args.project_key, args.board_type, args.start_at, args.limit)
        .await
        .map_err(OperationError::from_upstream)?;
    filter_named_values(&mut value, args.name);

    Ok(structured(value))
}

pub async fn list_board_issues(
    context: &AppContext,
    args: JiraListBoardIssuesArgs,
) -> Result<OperationResult, OperationError> {
    guard_operation(JIRA_LIST_BOARD_ISSUES_TOOL_NAME, context)?;
    let fields =
        parse_optional_string_list(args.fields, "fields").map_err(OperationError::from_upstream)?;
    let value = jira_client(context)?
        .get_board_issues(args.board_id, args.jql, fields, args.start_at, args.limit)
        .await
        .map_err(OperationError::from_upstream)?;

    Ok(structured(value))
}

pub async fn list_board_sprints(
    context: &AppContext,
    args: JiraListBoardSprintsArgs,
) -> Result<OperationResult, OperationError> {
    guard_operation(JIRA_LIST_BOARD_SPRINTS_TOOL_NAME, context)?;
    let state =
        parse_optional_string_list(args.state, "state").map_err(OperationError::from_upstream)?;
    let value = jira_client(context)?
        .get_sprints_from_board(args.board_id, state, args.start_at, args.limit)
        .await
        .map_err(OperationError::from_upstream)?;

    Ok(structured(value))
}

pub async fn list_sprint_issues(
    context: &AppContext,
    args: JiraListSprintIssuesArgs,
) -> Result<OperationResult, OperationError> {
    guard_operation(JIRA_LIST_SPRINT_ISSUES_TOOL_NAME, context)?;
    let fields =
        parse_optional_string_list(args.fields, "fields").map_err(OperationError::from_upstream)?;
    let value = jira_client(context)?
        .get_sprint_issues(args.sprint_id, fields, args.start_at, args.limit)
        .await
        .map_err(OperationError::from_upstream)?;

    Ok(structured(value))
}

pub async fn create_sprint(
    context: &AppContext,
    args: JiraCreateSprintArgs,
) -> Result<OperationResult, OperationError> {
    guard_operation(JIRA_CREATE_SPRINT_TOOL_NAME, context)?;
    let value = jira_client(context)?
        .create_sprint(create_sprint_payload_from_args(args)?)
        .await
        .map_err(OperationError::from_upstream)?;

    Ok(structured(value))
}

pub async fn update_sprint(
    context: &AppContext,
    args: JiraUpdateSprintArgs,
) -> Result<OperationResult, OperationError> {
    guard_operation(JIRA_UPDATE_SPRINT_TOOL_NAME, context)?;
    let (sprint_id, payload) = update_sprint_payload_from_args(args)?;
    let value = jira_client(context)?
        .update_sprint(sprint_id, payload)
        .await
        .map_err(OperationError::from_upstream)?;

    Ok(structured(value))
}

pub async fn add_issues_to_sprint(
    context: &AppContext,
    args: JiraAddIssuesToSprintArgs,
) -> Result<OperationResult, OperationError> {
    guard_operation(JIRA_ADD_ISSUES_TO_SPRINT_TOOL_NAME, context)?;
    let issue_keys = parse_required_string_list(args.issue_keys, "issue_keys")
        .map_err(OperationError::from_upstream)?;
    let value = jira_client(context)?
        .add_issues_to_sprint(args.sprint_id, issue_keys)
        .await
        .map_err(OperationError::from_upstream)?;

    Ok(structured(value))
}

pub async fn get_project_service_desk(
    context: &AppContext,
    args: JiraGetServiceDeskForProjectArgs,
) -> Result<OperationResult, OperationError> {
    guard_operation(JIRA_GET_SERVICE_DESK_FOR_PROJECT_TOOL_NAME, context)?;
    let value = jira_client(context)?
        .get_service_desk_for_project(required_non_empty_arg(args.project_key, "project_key")?)
        .await
        .map_err(OperationError::from_upstream)?;

    Ok(structured(value))
}

pub async fn list_service_desk_queues(
    context: &AppContext,
    args: JiraListServiceDeskQueuesArgs,
) -> Result<OperationResult, OperationError> {
    guard_operation(JIRA_LIST_SERVICE_DESK_QUEUES_TOOL_NAME, context)?;
    let include_counts = args.include_counts;
    let mut value = jira_client(context)?
        .get_service_desk_queues(args.service_desk_id, args.start_at, args.limit)
        .await
        .map_err(OperationError::from_upstream)?;
    if include_counts == Some(false) {
        remove_count_fields(&mut value);
    }

    Ok(structured(value))
}

pub async fn list_service_desk_queue_issues(
    context: &AppContext,
    args: JiraListServiceDeskQueueIssuesArgs,
) -> Result<OperationResult, OperationError> {
    guard_operation(JIRA_LIST_SERVICE_DESK_QUEUE_ISSUES_TOOL_NAME, context)?;
    let value = jira_client(context)?
        .get_queue_issues(
            args.service_desk_id,
            args.queue_id,
            args.start_at,
            args.limit,
        )
        .await
        .map_err(OperationError::from_upstream)?;

    Ok(structured(value))
}

pub async fn get_issue_timeline(
    context: &AppContext,
    args: JiraGetIssueTimelineArgs,
) -> Result<OperationResult, OperationError> {
    guard_operation(JIRA_GET_ISSUE_TIMELINE_TOOL_NAME, context)?;
    let value = jira_client(context)?
        .get_issue_dates(
            required_non_empty_arg(args.issue_key, "issue_key")?,
            args.include_status_changes.unwrap_or(false),
            args.include_status_summary.unwrap_or(false),
        )
        .await
        .map_err(OperationError::from_upstream)?;

    Ok(structured(value))
}

pub async fn get_issue_sla_metrics(
    context: &AppContext,
    args: JiraGetIssueSlaMetricsArgs,
) -> Result<OperationResult, OperationError> {
    guard_operation(JIRA_GET_ISSUE_SLA_METRICS_TOOL_NAME, context)?;
    let metrics = parse_optional_string_list(args.metrics, "metrics")
        .map_err(OperationError::from_upstream)?;
    let value = jira_client(context)?
        .get_issue_sla(
            required_non_empty_arg(args.issue_key, "issue_key")?,
            metrics,
            args.include_raw_dates.unwrap_or(false),
        )
        .await
        .map_err(OperationError::from_upstream)?;

    Ok(structured(value))
}

pub async fn get_issue_development(
    context: &AppContext,
    args: JiraGetIssueDevelopmentArgs,
) -> Result<OperationResult, OperationError> {
    guard_operation(JIRA_GET_ISSUE_DEVELOPMENT_TOOL_NAME, context)?;
    let value = jira_client(context)?
        .get_issue_development_info(
            required_non_empty_arg(args.issue_key, "issue_key")?,
            args.application_type,
            args.data_type,
        )
        .await
        .map_err(OperationError::from_upstream)?;

    Ok(structured(value))
}

pub async fn get_issues_development(
    context: &AppContext,
    args: JiraGetIssuesDevelopmentArgs,
) -> Result<OperationResult, OperationError> {
    guard_operation(JIRA_GET_ISSUES_DEVELOPMENT_TOOL_NAME, context)?;
    let issue_keys = parse_required_string_list(args.issue_keys, "issue_keys")
        .map_err(OperationError::from_upstream)?;
    let value = jira_client(context)?
        .get_issues_development_info(issue_keys, args.application_type, args.data_type)
        .await
        .map_err(OperationError::from_upstream)?;

    Ok(structured(value))
}

fn structured(value: Value) -> OperationResult {
    OperationResult::success(wrap_array(value))
}

fn wrap_array(value: Value) -> Value {
    match value {
        Value::Array(items) => json!({ "items": items }),
        other => other,
    }
}

fn jira_deployment(context: &AppContext) -> Result<JiraDeployment, OperationError> {
    context
        .jira_config()
        .map(|config| config.deployment)
        .ok_or_else(|| OperationError::config("Jira is not configured"))
}

fn required_non_empty_arg(
    value: String,
    field_name: &'static str,
) -> Result<String, OperationError> {
    let value = value.trim();
    if value.is_empty() {
        Err(OperationError::invalid_input(format!(
            "{field_name} is required"
        )))
    } else {
        Ok(value.to_string())
    }
}

fn optional_non_empty_arg(value: Option<String>) -> Option<String> {
    value.and_then(|value| {
        let value = value.trim();
        if value.is_empty() {
            None
        } else {
            Some(value.to_string())
        }
    })
}

fn optional_positive_u64_arg(
    value: Option<u64>,
    field_name: &'static str,
) -> Result<Option<u64>, OperationError> {
    match value {
        Some(0) => Err(OperationError::invalid_input(format!(
            "{field_name} must be positive"
        ))),
        other => Ok(other),
    }
}

fn optional_positive_i64_arg(
    value: Option<i64>,
    field_name: &'static str,
) -> Result<Option<i64>, OperationError> {
    match value {
        Some(value) if value <= 0 => Err(OperationError::invalid_input(format!(
            "{field_name} must be positive"
        ))),
        other => Ok(other),
    }
}

fn version_payload_from_args(args: JiraCreateProjectVersionArgs) -> Result<Value, OperationError> {
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
    insert_optional_bool(&mut payload, "released", args.released);
    insert_optional_bool(&mut payload, "archived", args.archived);
    Ok(payload)
}

type WorklogPayloadParts = (String, Value, Vec<(String, String)>);

fn add_worklog_payload_from_args(
    args: JiraAddWorklogArgs,
    deployment: JiraDeployment,
) -> Result<WorklogPayloadParts, OperationError> {
    let issue_key = required_non_empty_arg(args.issue_key, "issue_key")?;
    let time_spent = required_non_empty_arg(args.time_spent, "time_spent")?;
    let visibility = parse_optional_object(args.visibility, "visibility")
        .map_err(OperationError::from_upstream)?;
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
) -> Result<Value, OperationError> {
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
) -> Result<(String, Value), OperationError> {
    let issue_key = required_non_empty_arg(args.issue_key, "issue_key")?;
    let url = required_non_empty_arg(args.url, "url")?;
    let title = required_non_empty_arg(args.title, "title")?;
    let status =
        parse_optional_object(args.status, "status").map_err(OperationError::from_upstream)?;
    let mut object = json!({
        "url": url,
        "title": title,
    });
    insert_optional_value(&mut object, "summary", optional_non_empty_arg(args.summary));
    if let Some(icon_url) = optional_non_empty_arg(args.icon_url) {
        let icon_title = optional_non_empty_arg(args.icon_title).unwrap_or_else(|| title.clone());
        object["icon"] = json!({
            "url16x16": icon_url,
            "title": icon_title,
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

fn create_sprint_payload_from_args(args: JiraCreateSprintArgs) -> Result<Value, OperationError> {
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

fn update_sprint_payload_from_args(
    args: JiraUpdateSprintArgs,
) -> Result<(u64, Value), OperationError> {
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
        return Err(OperationError::invalid_input(
            "sprint update must contain at least one field",
        ));
    }

    Ok((args.sprint_id, payload))
}

fn version_payload_from_value(value: Value, project_key: &str) -> Result<Value, OperationError> {
    let mut object = value_into_object(value, "version")?;
    let name = take_required_string_field(&mut object, "name")?;
    let start_date = take_optional_string_alias(&mut object, "startDate", "start_date")?;
    let release_date = take_optional_string_alias(&mut object, "releaseDate", "release_date")?;
    let description = take_optional_string_field(&mut object, "description")?;
    let released = take_optional_bool_field(&mut object, "released")?;
    let archived = take_optional_bool_field(&mut object, "archived")?;
    let mut payload = Value::Object(object);
    payload["project"] = Value::String(project_key.to_string());
    payload["name"] = Value::String(name);
    insert_optional_value(&mut payload, "startDate", start_date);
    insert_optional_value(&mut payload, "releaseDate", release_date);
    insert_optional_value(&mut payload, "description", description);
    insert_optional_bool(&mut payload, "released", released);
    insert_optional_bool(&mut payload, "archived", archived);
    Ok(payload)
}

fn take_optional_string_alias(
    object: &mut Map<String, Value>,
    first: &'static str,
    second: &'static str,
) -> Result<Option<String>, OperationError> {
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

fn insert_optional_bool(payload: &mut Value, key: &'static str, value: Option<bool>) {
    if let Some(value) = value {
        payload[key] = Value::Bool(value);
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

fn filter_named_values(value: &mut Value, name_filter: Option<String>) {
    let Some(name_filter) = optional_non_empty_arg(name_filter).map(|value| value.to_lowercase())
    else {
        return;
    };
    if let Some(values) = value.get_mut("values").and_then(Value::as_array_mut) {
        values.retain(|item| {
            item.get("name")
                .and_then(Value::as_str)
                .is_some_and(|name| name.to_lowercase().contains(&name_filter))
        });
    }
}

fn remove_count_fields(value: &mut Value) {
    if let Some(values) = value.get_mut("values").and_then(Value::as_array_mut) {
        for item in values {
            if let Some(object) = item.as_object_mut() {
                object.remove("issueCount");
                object.remove("count");
            }
        }
    }
}

fn create_issue_fields_from_args(
    args: JiraCreateIssueArgs,
    deployment: JiraDeployment,
) -> Result<Value, OperationError> {
    let project_key = required_non_empty_arg(args.project_key, "project_key")?;
    let summary = required_non_empty_arg(args.summary, "summary")?;
    let issue_type = required_non_empty_arg(args.issue_type, "issue_type")?;
    let components = parse_optional_string_list(args.components, "components")
        .map_err(OperationError::from_upstream)?;
    let labels =
        parse_optional_string_list(args.labels, "labels").map_err(OperationError::from_upstream)?;
    let fix_versions = parse_optional_string_list(args.fix_versions, "fix_versions")
        .map_err(OperationError::from_upstream)?;
    let additional_fields = parse_optional_object(args.additional_fields, "additional_fields")
        .map_err(OperationError::from_upstream)?;
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
    if let Some(priority) = optional_non_empty_arg(args.priority) {
        fields["priority"] = json!({ "name": priority });
    }
    if let Some(labels) = labels
        && !labels.is_empty()
    {
        fields["labels"] = Value::Array(labels.into_iter().map(Value::String).collect());
    }
    if let Some(fix_versions) = fix_versions {
        let fix_versions = fix_versions
            .into_iter()
            .map(|name| json!({ "name": name }))
            .collect::<Vec<_>>();
        if !fix_versions.is_empty() {
            fields["fixVersions"] = Value::Array(fix_versions);
        }
    }

    merge_optional_objects(fields, additional_fields, "additional_fields")
        .map_err(OperationError::from_upstream)
}

#[derive(Debug)]
struct UpdateIssueFields {
    issue_key: String,
    fields: Value,
    notify_users: Option<bool>,
}

fn update_issue_fields_from_args(
    args: JiraUpdateIssueArgs,
    deployment: JiraDeployment,
) -> Result<(UpdateIssueFields, Option<Value>), OperationError> {
    let issue_key = required_non_empty_arg(args.issue_key, "issue_key")?;
    let fields = normalize_issue_fields(
        parse_required_object(args.fields, "fields").map_err(OperationError::from_upstream)?,
        deployment,
        "fields",
    )?;
    let components = parse_optional_string_list(args.components, "components")
        .map_err(OperationError::from_upstream)?;
    let mut additional_fields = parse_optional_object(args.additional_fields, "additional_fields")
        .map_err(OperationError::from_upstream)?
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
        return Err(OperationError::invalid_input(
            "fields must contain at least one update",
        ));
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
) -> Result<Value, OperationError> {
    reject_unsupported_attachments(&fields, field_name)?;
    let object = fields.as_object_mut().ok_or_else(|| {
        OperationError::invalid_input(format!("{field_name} must be a JSON object"))
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
) -> Result<(), OperationError> {
    if value
        .as_object()
        .is_some_and(|object| object.contains_key("attachments"))
    {
        Err(OperationError::invalid_input(format!(
            "{field_name}.attachments is not supported by jira_update_issue"
        )))
    } else {
        Ok(())
    }
}

fn batch_create_issue_updates_from_args(
    issues: Value,
    deployment: JiraDeployment,
) -> Result<Vec<Value>, OperationError> {
    parse_required_object_list(issues, "issues")
        .map_err(OperationError::from_upstream)?
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
) -> Result<Value, OperationError> {
    let mut fields = value_into_object(issue, "issue")?;
    let project_key = take_required_string_field(&mut fields, "project_key")?;
    let summary = take_required_string_field(&mut fields, "summary")?;
    let issue_type = take_required_string_field(&mut fields, "issue_type")?;
    let assignee = take_optional_string_field(&mut fields, "assignee")?;
    let description = take_optional_string_field(&mut fields, "description")?;
    let priority = take_optional_string_field_if_string(&mut fields, "priority");
    let components = take_optional_string_list_field_if_compatible(&mut fields, "components");
    let labels = take_optional_string_list_field_if_compatible(&mut fields, "labels");
    let fix_versions = take_optional_string_list_field_if_compatible(&mut fields, "fixVersions")
        .or_else(|| take_optional_string_list_field_if_compatible(&mut fields, "fix_versions"));
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
            priority,
            labels,
            fix_versions,
            additional_fields,
        },
        deployment,
    )
}

fn value_into_object(
    value: Value,
    field_name: &'static str,
) -> Result<Map<String, Value>, OperationError> {
    match parse_required_object(value, field_name).map_err(OperationError::from_upstream)? {
        Value::Object(object) => Ok(object),
        _ => unreachable!("parse_required_object only returns JSON objects"),
    }
}

fn take_required_string_field(
    object: &mut Map<String, Value>,
    field_name: &'static str,
) -> Result<String, OperationError> {
    match object.remove(field_name) {
        Some(Value::String(value)) => required_non_empty_arg(value, field_name),
        Some(_) => Err(OperationError::invalid_input(format!(
            "{field_name} must be a string"
        ))),
        None => Err(OperationError::invalid_input(format!(
            "{field_name} is required"
        ))),
    }
}

fn take_optional_string_field(
    object: &mut Map<String, Value>,
    field_name: &'static str,
) -> Result<Option<String>, OperationError> {
    match object.remove(field_name) {
        Some(Value::String(value)) => Ok(optional_non_empty_arg(Some(value))),
        Some(Value::Null) | None => Ok(None),
        Some(_) => Err(OperationError::invalid_input(format!(
            "{field_name} must be a string"
        ))),
    }
}

fn take_optional_bool_field(
    object: &mut Map<String, Value>,
    field_name: &'static str,
) -> Result<Option<bool>, OperationError> {
    match object.remove(field_name) {
        Some(Value::Bool(value)) => Ok(Some(value)),
        Some(Value::Null) | None => Ok(None),
        Some(_) => Err(OperationError::invalid_input(format!(
            "{field_name} must be a boolean"
        ))),
    }
}

fn take_optional_string_field_if_string(
    object: &mut Map<String, Value>,
    field_name: &'static str,
) -> Option<String> {
    if object.get(field_name).is_some_and(Value::is_string) {
        match object.remove(field_name) {
            Some(Value::String(value)) => optional_non_empty_arg(Some(value)),
            _ => None,
        }
    } else {
        None
    }
}

fn take_optional_string_list_field_if_compatible(
    object: &mut Map<String, Value>,
    field_name: &'static str,
) -> Option<Value> {
    if object.get(field_name).is_some_and(|value| {
        value.is_string() || value.as_array().is_some_and(|values| string_array(values))
    }) {
        object.remove(field_name)
    } else {
        None
    }
}

fn string_array(values: &[Value]) -> bool {
    values.iter().all(Value::is_string)
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;

    use serde_json::json;

    use crate::{
        config::{HttpConfig, RuntimeConfig},
        context::AppContext,
        jira::{
            config::JiraConfig,
            tools::{
                JIRA_ADD_COMMENT_TOOL_NAME, JIRA_CREATE_ISSUE_TOOL_NAME,
                JIRA_CREATE_ISSUES_TOOL_NAME, JIRA_DELETE_ISSUE_TOOL_NAME,
                JIRA_EDIT_COMMENT_TOOL_NAME, JIRA_EXTENSION_TOOL_NAMES,
                JIRA_GET_FIELD_OPTIONS_TOOL_NAME, JIRA_GET_ISSUE_TOOL_NAME,
                JIRA_GET_PROJECT_ISSUES_TOOL_NAME, JIRA_GET_TRANSITIONS_TOOL_NAME,
                JIRA_LIST_PROJECT_COMPONENTS_TOOL_NAME, JIRA_LIST_PROJECTS_TOOL_NAME,
                JIRA_SEARCH_FIELDS_TOOL_NAME, JIRA_SEARCH_TOOL_NAME,
                JIRA_TRANSITION_ISSUE_TOOL_NAME, JIRA_UPDATE_ISSUE_TOOL_NAME,
            },
        },
        tool_registry,
        upstream::{auth::UpstreamAuth, custom_headers::CustomHeaders, proxy::ProxyConfig},
    };

    use super::*;

    #[test]
    fn operations_jira_create_issue_payload_maps_core_business_fields() {
        let fields = create_issue_fields_from_args(
            JiraCreateIssueArgs {
                project_key: "ABC".to_string(),
                summary: "Add parser".to_string(),
                issue_type: "Task".to_string(),
                assignee: Some("chen".to_string()),
                description: Some("hello".to_string()),
                components: Some(json!("Frontend, API")),
                priority: Some("High".to_string()),
                labels: Some(json!(["cli", "migration"])),
                fix_versions: Some(json!("1.0, 1.1")),
                additional_fields: Some(json!({"customfield_10000": "x"})),
            },
            JiraDeployment::ServerDataCenter,
        )
        .unwrap();

        assert_eq!(fields["project"]["key"], json!("ABC"));
        assert_eq!(fields["priority"]["name"], json!("High"));
        assert_eq!(fields["labels"], json!(["cli", "migration"]));
        assert_eq!(fields["fixVersions"][0]["name"], json!("1.0"));
        assert_eq!(fields["customfield_10000"], json!("x"));
    }

    #[test]
    fn operations_jira_update_issue_rejects_empty_payload() {
        let error = update_issue_fields_from_args(
            JiraUpdateIssueArgs {
                issue_key: "ABC-1".to_string(),
                fields: json!({}),
                additional_fields: None,
                components: None,
                notify_users: None,
            },
            JiraDeployment::ServerDataCenter,
        )
        .unwrap_err();

        assert_eq!(
            error.category,
            super::super::OperationErrorCategory::InvalidInput
        );
    }

    #[test]
    fn operations_jira_batch_create_payload_accepts_fix_versions_alias() {
        let updates = batch_create_issue_updates_from_args(
            json!([{
                "project_key": "ABC",
                "summary": "Batch",
                "issue_type": "Task",
                "fix_versions": ["1.0"]
            }]),
            JiraDeployment::ServerDataCenter,
        )
        .unwrap();

        assert_eq!(updates[0]["fields"]["fixVersions"][0]["name"], json!("1.0"));
    }

    #[test]
    fn operations_jira_extended_payloads_map_shared_business_fields() {
        let version = version_payload_from_args(JiraCreateProjectVersionArgs {
            project_key: "ABC".to_string(),
            name: "v1".to_string(),
            start_date: Some("2026-01-01".to_string()),
            release_date: Some("2026-02-01".to_string()),
            description: Some("First release".to_string()),
            released: Some(true),
            archived: Some(false),
        })
        .unwrap();
        assert_eq!(version["project"], json!("ABC"));
        assert_eq!(version["released"], json!(true));
        assert_eq!(version["archived"], json!(false));

        let version = version_payload_from_value(
            json!({
                "name": "v2",
                "start_date": "2026-03-01",
                "releaseDate": "2026-04-01",
                "released": false,
                "archived": true
            }),
            "ABC",
        )
        .unwrap();
        assert_eq!(version["startDate"], json!("2026-03-01"));
        assert_eq!(version["releaseDate"], json!("2026-04-01"));
        assert_eq!(version["released"], json!(false));
        assert_eq!(version["archived"], json!(true));

        let (issue_key, worklog, query) = add_worklog_payload_from_args(
            JiraAddWorklogArgs {
                issue_key: "ABC-1".to_string(),
                time_spent: "30m".to_string(),
                started: Some("2026-01-01T10:00:00.000+0000".to_string()),
                comment: Some("investigation".to_string()),
                visibility: Some(json!({"type": "group", "value": "jira-users"})),
                adjust_estimate: Some("new".to_string()),
                new_estimate: Some("2h".to_string()),
                reduce_by: None,
            },
            JiraDeployment::ServerDataCenter,
        )
        .unwrap();
        assert_eq!(issue_key, "ABC-1");
        assert_eq!(worklog["timeSpent"], json!("30m"));
        assert_eq!(worklog["comment"], json!("investigation"));
        assert_eq!(worklog["visibility"]["type"], json!("group"));
        assert!(query.contains(&("adjustEstimate".to_string(), "new".to_string())));
        assert!(query.contains(&("newEstimate".to_string(), "2h".to_string())));

        let link = issue_link_payload_from_args(
            JiraCreateIssueLinkArgs {
                link_type: "Blocks".to_string(),
                inward_issue_key: "ABC-1".to_string(),
                outward_issue_key: "ABC-2".to_string(),
                comment: Some("related".to_string()),
            },
            JiraDeployment::ServerDataCenter,
        )
        .unwrap();
        assert_eq!(link["type"]["name"], json!("Blocks"));
        assert_eq!(link["comment"]["body"], json!("related"));

        let (issue_key, remote_link) =
            remote_issue_link_payload_from_args(JiraCreateRemoteIssueLinkArgs {
                issue_key: "ABC-1".to_string(),
                url: "https://example.invalid/design".to_string(),
                title: "Design".to_string(),
                global_id: Some("design-1".to_string()),
                summary: Some("Design doc".to_string()),
                relationship: Some("documents".to_string()),
                icon_url: Some("https://example.invalid/icon.png".to_string()),
                icon_title: Some("Design icon".to_string()),
                status: Some(json!({"resolved": true})),
            })
            .unwrap();
        assert_eq!(issue_key, "ABC-1");
        assert_eq!(remote_link["globalId"], json!("design-1"));
        assert_eq!(remote_link["relationship"], json!("documents"));
        assert_eq!(remote_link["object"]["icon"]["title"], json!("Design icon"));
        assert_eq!(remote_link["object"]["status"]["resolved"], json!(true));
    }

    #[test]
    fn operations_jira_extended_filters_are_operation_owned() {
        let mut boards = json!({
            "values": [
                {"id": 1, "name": "Alpha board"},
                {"id": 2, "name": "Operations"},
                {"id": 3, "name": "alpha kanban"}
            ]
        });
        filter_named_values(&mut boards, Some("ALPHA".to_string()));
        assert_eq!(boards["values"].as_array().unwrap().len(), 2);
        assert_eq!(boards["values"][0]["name"], json!("Alpha board"));
        assert_eq!(boards["values"][1]["name"], json!("alpha kanban"));

        let mut queues = json!({
            "values": [
                {"id": "1", "name": "Open", "issueCount": 4, "count": 9},
                {"id": "2", "name": "Done", "issueCount": 0}
            ]
        });
        remove_count_fields(&mut queues);
        assert!(queues["values"][0].get("issueCount").is_none());
        assert!(queues["values"][0].get("count").is_none());
        assert!(queues["values"][1].get("issueCount").is_none());
    }

    #[tokio::test]
    async fn operations_jira_core_operations_validate_inputs_before_http() {
        let context = jira_context();

        assert!(
            get_issue(
                &context,
                JiraGetIssueArgs {
                    issue_key: "ABC-1".to_string(),
                    fields: Some(json!({})),
                    expand: None,
                    comment_limit: None,
                    properties: None,
                    update_history: None,
                },
            )
            .await
            .unwrap_err()
            .message
            .contains("fields must be a string or array of strings")
        );
        assert!(
            search_issues(
                &context,
                JiraSearchArgs {
                    jql: "project = ABC".to_string(),
                    fields: Some(json!({})),
                    limit: None,
                    start_at: None,
                    projects_filter: None,
                    expand: None,
                    page_token: None,
                },
            )
            .await
            .unwrap_err()
            .message
            .contains("fields must be a string or array of strings")
        );
        assert!(
            add_issue_comment(
                &context,
                JiraAddCommentArgs {
                    issue_key: "ABC-1".to_string(),
                    body: "comment".to_string(),
                    visibility: Some(json!("[]")),
                },
            )
            .await
            .unwrap_err()
            .message
            .contains("visibility must be a JSON object")
        );
        assert!(
            transition_issue(
                &context,
                JiraTransitionIssueArgs {
                    issue_key: "ABC-1".to_string(),
                    transition_id: "31".to_string(),
                    fields: Some(json!("[]")),
                    comment: None,
                },
            )
            .await
            .unwrap_err()
            .message
            .contains("fields must be a JSON object")
        );
        assert!(
            update_issue(
                &context,
                JiraUpdateIssueArgs {
                    issue_key: "ABC-1".to_string(),
                    fields: json!({"attachments": ["/tmp/file.txt"]}),
                    additional_fields: None,
                    components: None,
                    notify_users: None,
                },
            )
            .await
            .unwrap_err()
            .message
            .contains("attachments is not supported")
        );
        assert!(
            create_issues(
                &context,
                JiraCreateIssuesArgs {
                    issues: json!([{"project_key": "ABC", "issue_type": "Task"}]),
                    validate_only: None,
                },
            )
            .await
            .unwrap_err()
            .message
            .contains("summary is required")
        );
    }

    #[tokio::test]
    async fn operations_jira_extended_operations_validate_inputs_before_http() {
        let context = jira_context();

        assert!(
            get_issue_changelogs(
                &context,
                JiraGetIssueChangelogsArgs {
                    issue_ids_or_keys: json!({}),
                    fields: None,
                    limit: None,
                },
            )
            .await
            .unwrap_err()
            .message
            .contains("issue_ids_or_keys must be a string or array of strings")
        );
        assert!(
            create_project_version(
                &context,
                JiraCreateProjectVersionArgs {
                    project_key: "ABC".to_string(),
                    name: " ".to_string(),
                    start_date: None,
                    release_date: None,
                    description: None,
                    released: None,
                    archived: None,
                },
            )
            .await
            .unwrap_err()
            .message
            .contains("name is required")
        );
        assert!(
            add_issue_worklog(
                &context,
                JiraAddWorklogArgs {
                    issue_key: "ABC-1".to_string(),
                    time_spent: "30m".to_string(),
                    started: None,
                    comment: None,
                    visibility: Some(json!("private")),
                    adjust_estimate: None,
                    new_estimate: None,
                    reduce_by: None,
                },
            )
            .await
            .unwrap_err()
            .message
            .contains("visibility must be a JSON object")
        );
        assert!(
            create_remote_issue_link(
                &context,
                JiraCreateRemoteIssueLinkArgs {
                    issue_key: "ABC-1".to_string(),
                    url: "https://example.invalid".to_string(),
                    title: "Design".to_string(),
                    global_id: None,
                    summary: None,
                    relationship: None,
                    icon_url: None,
                    icon_title: None,
                    status: Some(json!("resolved")),
                },
            )
            .await
            .unwrap_err()
            .message
            .contains("status must be a JSON object")
        );
        assert!(
            get_issue_attachments(
                &context,
                JiraGetIssueAttachmentsArgs {
                    issue_key: "ABC-1".to_string(),
                    attachment_ids: None,
                    filename_contains: None,
                    media_type: None,
                    include_content: Some(true),
                    max_bytes: Some(0),
                },
            )
            .await
            .unwrap_err()
            .message
            .contains("max_bytes must be positive")
        );
        assert!(
            add_issues_to_sprint(
                &context,
                JiraAddIssuesToSprintArgs {
                    sprint_id: 7,
                    issue_keys: json!({}),
                },
            )
            .await
            .unwrap_err()
            .message
            .contains("issue_keys must be a string or array of strings")
        );
        assert!(
            get_issue_sla_metrics(
                &context,
                JiraGetIssueSlaMetricsArgs {
                    issue_key: "ABC-1".to_string(),
                    metrics: Some(json!({})),
                    include_raw_dates: None,
                },
            )
            .await
            .unwrap_err()
            .message
            .contains("metrics must be a string or array of strings")
        );
        assert!(
            get_issues_development(
                &context,
                JiraGetIssuesDevelopmentArgs {
                    issue_keys: json!({}),
                    application_type: None,
                    data_type: None,
                },
            )
            .await
            .unwrap_err()
            .message
            .contains("issue_keys must be a string or array of strings")
        );
    }

    #[test]
    fn operations_jira_core_tool_names_have_metadata_guard_mapping() {
        for tool_name in [
            JIRA_GET_ISSUE_TOOL_NAME,
            JIRA_SEARCH_TOOL_NAME,
            JIRA_GET_PROJECT_ISSUES_TOOL_NAME,
            JIRA_SEARCH_FIELDS_TOOL_NAME,
            JIRA_GET_FIELD_OPTIONS_TOOL_NAME,
            JIRA_ADD_COMMENT_TOOL_NAME,
            JIRA_EDIT_COMMENT_TOOL_NAME,
            JIRA_GET_TRANSITIONS_TOOL_NAME,
            JIRA_TRANSITION_ISSUE_TOOL_NAME,
            JIRA_CREATE_ISSUE_TOOL_NAME,
            JIRA_CREATE_ISSUES_TOOL_NAME,
            JIRA_UPDATE_ISSUE_TOOL_NAME,
            JIRA_DELETE_ISSUE_TOOL_NAME,
            JIRA_LIST_PROJECTS_TOOL_NAME,
            JIRA_LIST_PROJECT_COMPONENTS_TOOL_NAME,
        ] {
            assert!(
                tool_registry::metadata_for(tool_name).is_some(),
                "{tool_name} missing metadata"
            );
        }
    }

    #[test]
    fn operations_jira_extended_tool_names_have_metadata_guard_mapping() {
        for tool_name in JIRA_EXTENSION_TOOL_NAMES {
            assert!(
                tool_registry::metadata_for(tool_name).is_some(),
                "{tool_name} missing metadata"
            );
        }
    }

    fn jira_context() -> AppContext {
        AppContext::from_config(&RuntimeConfig {
            jira: Some(JiraConfig {
                base_url: "https://jira.example".to_string(),
                deployment: JiraDeployment::ServerDataCenter,
                auth: UpstreamAuth::Pat {
                    personal_token: "test-pat-value".to_string(),
                },
                ssl_verify: true,
                proxy: ProxyConfig::default(),
                custom_headers: CustomHeaders::default(),
                mtls: None,
                projects_filter: BTreeSet::new(),
                timeout_seconds: 75,
            }),
            mcp_enabled_toolsets: tool_registry::all_toolsets(),
            http: HttpConfig::default(),
            ..RuntimeConfig::default()
        })
    }
}

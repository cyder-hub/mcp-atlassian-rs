use serde_json::{Value, json};

use crate::{
    context::AppContext,
    gitlab::{
        client::{
            AcceptMergeRequestRequest, CreateMergeRequestRequest, GetMergeRequestRequest,
            ListMergeRequestCommitsRequest, ListMergeRequestDiffsRequest,
            ListMergeRequestPipelinesRequest, ListMergeRequestsRequest, UpdateMergeRequestRequest,
        },
        tools::*,
    },
};

use super::{OperationError, OperationResult, gitlab_client, guard_operation};

pub async fn get_current_user(
    context: &AppContext,
    _args: GitlabGetCurrentUserArgs,
) -> Result<OperationResult, OperationError> {
    guard_operation(GITLAB_GET_CURRENT_USER_TOOL_NAME, context)?;
    let value = gitlab_client(context)?
        .get_current_user()
        .await
        .map_err(OperationError::from_upstream)?;

    Ok(structured(wrap_array(value)))
}

pub async fn get_project(
    context: &AppContext,
    args: GitlabGetProjectArgs,
) -> Result<OperationResult, OperationError> {
    guard_operation(GITLAB_GET_PROJECT_TOOL_NAME, context)?;
    let value = gitlab_client(context)?
        .get_project(&args.project)
        .await
        .map_err(OperationError::from_upstream)?;

    Ok(structured(wrap_array(value)))
}

pub async fn list_merge_requests(
    context: &AppContext,
    args: GitlabListMergeRequestsArgs,
) -> Result<OperationResult, OperationError> {
    guard_operation(GITLAB_LIST_MERGE_REQUESTS_TOOL_NAME, context)?;
    let value = gitlab_client(context)?
        .list_merge_requests(ListMergeRequestsRequest {
            project: args.project,
            state: args.state,
            author_username: args.author_username,
            reviewer_username: args.reviewer_username,
            source_branch: args.source_branch,
            target_branch: args.target_branch,
            labels: args.labels,
            page: args.page,
            per_page: args.per_page,
        })
        .await
        .map_err(OperationError::from_upstream)?;

    Ok(structured(wrap_array(value)))
}

pub async fn get_merge_request(
    context: &AppContext,
    args: GitlabGetMergeRequestArgs,
) -> Result<OperationResult, OperationError> {
    guard_operation(GITLAB_GET_MERGE_REQUEST_TOOL_NAME, context)?;
    let value = gitlab_client(context)?
        .get_merge_request(GetMergeRequestRequest {
            project: args.project,
            merge_request_iid: args.merge_request_iid,
            include_diverged_commits_count: args.include_diverged_commits_count,
            include_rebase_in_progress: args.include_rebase_in_progress,
        })
        .await
        .map_err(OperationError::from_upstream)?;

    Ok(structured(wrap_array(value)))
}

pub async fn list_merge_request_commits(
    context: &AppContext,
    args: GitlabListMergeRequestCommitsArgs,
) -> Result<OperationResult, OperationError> {
    guard_operation(GITLAB_LIST_MERGE_REQUEST_COMMITS_TOOL_NAME, context)?;
    let value = gitlab_client(context)?
        .list_merge_request_commits(ListMergeRequestCommitsRequest {
            project: args.project,
            merge_request_iid: args.merge_request_iid,
            page: args.page,
            per_page: args.per_page,
        })
        .await
        .map_err(OperationError::from_upstream)?;

    Ok(structured(wrap_array(value)))
}

pub async fn list_merge_request_diffs(
    context: &AppContext,
    args: GitlabListMergeRequestDiffsArgs,
) -> Result<OperationResult, OperationError> {
    guard_operation(GITLAB_LIST_MERGE_REQUEST_DIFFS_TOOL_NAME, context)?;
    let value = gitlab_client(context)?
        .list_merge_request_diffs(ListMergeRequestDiffsRequest {
            project: args.project,
            merge_request_iid: args.merge_request_iid,
            max_diff_bytes: args.max_diff_bytes,
            page: args.page,
            per_page: args.per_page,
        })
        .await
        .map_err(OperationError::from_upstream)?;

    Ok(structured(wrap_array(value)))
}

pub async fn list_merge_request_pipelines(
    context: &AppContext,
    args: GitlabListMergeRequestPipelinesArgs,
) -> Result<OperationResult, OperationError> {
    guard_operation(GITLAB_LIST_MERGE_REQUEST_PIPELINES_TOOL_NAME, context)?;
    let value = gitlab_client(context)?
        .list_merge_request_pipelines(ListMergeRequestPipelinesRequest {
            project: args.project,
            merge_request_iid: args.merge_request_iid,
            page: args.page,
            per_page: args.per_page,
        })
        .await
        .map_err(OperationError::from_upstream)?;

    Ok(structured(wrap_array(value)))
}

pub async fn create_merge_request(
    context: &AppContext,
    args: GitlabCreateMergeRequestArgs,
) -> Result<OperationResult, OperationError> {
    guard_operation(GITLAB_CREATE_MERGE_REQUEST_TOOL_NAME, context)?;
    let value = gitlab_client(context)?
        .create_merge_request(CreateMergeRequestRequest {
            project: args.project,
            source_branch: args.source_branch,
            target_branch: args.target_branch,
            title: args.title,
            description: args.description,
            remove_source_branch: args.remove_source_branch,
            squash: args.squash,
            assignee_ids: args.assignee_ids,
            reviewer_ids: args.reviewer_ids,
            labels: args.labels,
        })
        .await
        .map_err(OperationError::from_upstream)?;

    Ok(structured(wrap_array(value)))
}

pub async fn update_merge_request(
    context: &AppContext,
    args: GitlabUpdateMergeRequestArgs,
) -> Result<OperationResult, OperationError> {
    guard_operation(GITLAB_UPDATE_MERGE_REQUEST_TOOL_NAME, context)?;
    let value = gitlab_client(context)?
        .update_merge_request(UpdateMergeRequestRequest {
            project: args.project,
            merge_request_iid: args.merge_request_iid,
            title: args.title,
            description: args.description,
            state_event: args.state_event,
            labels: args.labels,
            add_labels: args.add_labels,
            remove_labels: args.remove_labels,
            reviewer_ids: args.reviewer_ids,
            assignee_ids: args.assignee_ids,
            target_branch: args.target_branch,
        })
        .await
        .map_err(OperationError::from_upstream)?;

    Ok(structured(wrap_array(value)))
}

pub async fn add_merge_request_note(
    context: &AppContext,
    args: GitlabAddMergeRequestNoteArgs,
) -> Result<OperationResult, OperationError> {
    guard_operation(GITLAB_ADD_MERGE_REQUEST_NOTE_TOOL_NAME, context)?;
    let value = gitlab_client(context)?
        .add_merge_request_note(&args.project, args.merge_request_iid, args.body)
        .await
        .map_err(OperationError::from_upstream)?;

    Ok(structured(wrap_array(value)))
}

pub async fn reply_merge_request_discussion(
    context: &AppContext,
    args: GitlabReplyMergeRequestDiscussionArgs,
) -> Result<OperationResult, OperationError> {
    guard_operation(GITLAB_REPLY_MERGE_REQUEST_DISCUSSION_TOOL_NAME, context)?;
    let value = gitlab_client(context)?
        .reply_merge_request_discussion(
            &args.project,
            args.merge_request_iid,
            &args.discussion_id,
            args.body,
        )
        .await
        .map_err(OperationError::from_upstream)?;

    Ok(structured(wrap_array(value)))
}

pub async fn resolve_merge_request_discussion(
    context: &AppContext,
    args: GitlabResolveMergeRequestDiscussionArgs,
) -> Result<OperationResult, OperationError> {
    guard_operation(GITLAB_RESOLVE_MERGE_REQUEST_DISCUSSION_TOOL_NAME, context)?;
    let value = gitlab_client(context)?
        .resolve_merge_request_discussion(
            &args.project,
            args.merge_request_iid,
            &args.discussion_id,
            args.resolved,
        )
        .await
        .map_err(OperationError::from_upstream)?;

    Ok(structured(wrap_array(value)))
}

pub async fn get_merge_request_approval_state(
    context: &AppContext,
    args: GitlabMergeRequestRefArgs,
) -> Result<OperationResult, OperationError> {
    guard_operation(GITLAB_GET_MERGE_REQUEST_APPROVAL_STATE_TOOL_NAME, context)?;
    let value = gitlab_client(context)?
        .get_merge_request_approval_state(&args.project, args.merge_request_iid)
        .await
        .map_err(OperationError::from_upstream)?;

    Ok(structured(wrap_array(value)))
}

pub async fn set_merge_request_approval(
    context: &AppContext,
    args: GitlabSetMergeRequestApprovalArgs,
) -> Result<OperationResult, OperationError> {
    guard_operation(GITLAB_SET_MERGE_REQUEST_APPROVAL_TOOL_NAME, context)?;
    let client = gitlab_client(context)?;
    let value = match args.action {
        GitlabMergeRequestApprovalAction::Approve => {
            client
                .approve_merge_request(&args.project, args.merge_request_iid)
                .await
        }
        GitlabMergeRequestApprovalAction::Unapprove => {
            client
                .unapprove_merge_request(&args.project, args.merge_request_iid)
                .await
        }
    }
    .map_err(OperationError::from_upstream)?;

    Ok(structured(wrap_array(value)))
}

pub async fn accept_merge_request(
    context: &AppContext,
    args: GitlabAcceptMergeRequestArgs,
) -> Result<OperationResult, OperationError> {
    guard_operation(GITLAB_ACCEPT_MERGE_REQUEST_TOOL_NAME, context)?;
    let value = gitlab_client(context)?
        .accept_merge_request(AcceptMergeRequestRequest {
            project: args.project,
            merge_request_iid: args.merge_request_iid,
            sha: args.sha,
            auto_merge: args.auto_merge,
            squash: args.squash,
            should_remove_source_branch: args.should_remove_source_branch,
            merge_commit_message: args.merge_commit_message,
            squash_commit_message: args.squash_commit_message,
        })
        .await
        .map_err(OperationError::from_upstream)?;

    Ok(structured(wrap_array(value)))
}

fn structured(value: Value) -> OperationResult {
    OperationResult::success(value)
}

fn wrap_array(value: Value) -> Value {
    match value {
        Value::Array(items) => json!({ "items": items }),
        other => other,
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;

    use crate::{
        config::{HttpConfig, RuntimeConfig},
        gitlab::{config::GitlabConfig, tools::GITLAB_GET_PROJECT_TOOL_NAME},
        operations::OperationErrorCategory,
        tool_registry,
        upstream::{auth::UpstreamAuth, custom_headers::CustomHeaders, proxy::ProxyConfig},
    };

    use super::*;

    #[tokio::test]
    async fn operations_gitlab_guard_ignores_mcp_disabled_tools() {
        let context = gitlab_context(
            BTreeSet::new(),
            BTreeSet::from([GITLAB_GET_PROJECT_TOOL_NAME.to_string()]),
        );

        assert!(guard_operation(GITLAB_GET_PROJECT_TOOL_NAME, &context).is_ok());
    }

    #[tokio::test]
    async fn operations_gitlab_validates_diff_bounds_before_http() {
        let error = list_merge_request_diffs(
            &gitlab_context(BTreeSet::new(), BTreeSet::new()),
            GitlabListMergeRequestDiffsArgs {
                project: "group/project".to_string(),
                merge_request_iid: 7,
                max_diff_bytes: Some(0),
                page: None,
                per_page: None,
            },
        )
        .await
        .unwrap_err();

        assert_eq!(error.category, OperationErrorCategory::InvalidInput);
        assert!(error.message.contains("max_diff_bytes must be positive"));
    }

    #[tokio::test]
    async fn operations_gitlab_preserves_project_filter_validation() {
        let error = get_project(
            &gitlab_context(
                BTreeSet::from(["group/project".to_string()]),
                BTreeSet::new(),
            ),
            GitlabGetProjectArgs {
                project: "other/project".to_string(),
            },
        )
        .await
        .unwrap_err();

        assert_eq!(error.category, OperationErrorCategory::InvalidInput);
        assert!(
            error
                .message
                .contains("GitLab project `other/project` is not allowed")
        );
    }

    #[tokio::test]
    async fn operations_gitlab_accept_validation_ignores_mcp_disabled_tools() {
        let error = accept_merge_request(
            &gitlab_context(
                BTreeSet::new(),
                BTreeSet::from([GITLAB_ACCEPT_MERGE_REQUEST_TOOL_NAME.to_string()]),
            ),
            GitlabAcceptMergeRequestArgs {
                project: "group/project".to_string(),
                merge_request_iid: 7,
                sha: " ".to_string(),
                auto_merge: None,
                squash: None,
                should_remove_source_branch: None,
                merge_commit_message: None,
                squash_commit_message: None,
            },
        )
        .await
        .unwrap_err();

        assert_eq!(error.category, OperationErrorCategory::InvalidInput);
        assert!(error.message.contains("sha must not be empty"));
    }

    #[tokio::test]
    async fn operations_gitlab_accept_validates_required_sha_before_http() {
        let error = accept_merge_request(
            &gitlab_context(BTreeSet::new(), BTreeSet::new()),
            GitlabAcceptMergeRequestArgs {
                project: "group/project".to_string(),
                merge_request_iid: 7,
                sha: "  ".to_string(),
                auto_merge: None,
                squash: None,
                should_remove_source_branch: None,
                merge_commit_message: None,
                squash_commit_message: None,
            },
        )
        .await
        .unwrap_err();

        assert_eq!(error.category, OperationErrorCategory::InvalidInput);
        assert!(error.message.contains("sha must not be empty"));
    }

    fn gitlab_context(
        projects_filter: BTreeSet<String>,
        mcp_disabled_tools: BTreeSet<String>,
    ) -> AppContext {
        AppContext::from_config(&RuntimeConfig {
            gitlab: Some(GitlabConfig {
                base_url: "https://gitlab.example".to_string(),
                auth: UpstreamAuth::HeaderToken {
                    header_name: reqwest::header::HeaderName::from_static("private-token"),
                    token: "gitlab-token".to_string(),
                },
                ssl_verify: true,
                proxy: ProxyConfig::default(),
                custom_headers: CustomHeaders::default(),
                mtls: None,
                projects_filter,
                timeout_seconds: 75,
            }),
            mcp_disabled_tools,
            mcp_enabled_toolsets: tool_registry::all_toolsets(),
            http: HttpConfig::default(),
            ..RuntimeConfig::default()
        })
    }
}

use crate::gitlab::{
    client::{
        AcceptMergeRequestRequest, CreateMergeRequestRequest, GetMergeRequestRequest,
        ListMergeRequestCommitsRequest, ListMergeRequestDiffsRequest,
        ListMergeRequestPipelinesRequest, ListMergeRequestsRequest, UpdateMergeRequestRequest,
    },
    tools::{
        GitlabAcceptMergeRequestArgs, GitlabAddMergeRequestNoteArgs, GitlabCreateMergeRequestArgs,
        GitlabGetCurrentUserArgs, GitlabGetMergeRequestArgs, GitlabGetProjectArgs,
        GitlabListMergeRequestCommitsArgs, GitlabListMergeRequestDiffsArgs,
        GitlabListMergeRequestPipelinesArgs, GitlabListMergeRequestsArgs,
        GitlabMergeRequestApprovalAction, GitlabMergeRequestRefArgs,
        GitlabReplyMergeRequestDiscussionArgs, GitlabResolveMergeRequestDiscussionArgs,
        GitlabSetMergeRequestApprovalArgs, GitlabUpdateMergeRequestArgs,
    },
};
use crate::mcp_errors::upstream_error;
use rmcp::{
    ErrorData, handler::server::wrapper::Parameters, model::CallToolResult, tool, tool_router,
};

use super::{AtlassianMcpServer, wrap_array};

#[tool_router(router = gitlab_tool_router, vis = "pub(super)")]
impl AtlassianMcpServer {
    #[tool(name = "gitlab_get_current_user")]
    pub(super) async fn gitlab_get_current_user(
        &self,
        Parameters(_args): Parameters<GitlabGetCurrentUserArgs>,
    ) -> Result<CallToolResult, ErrorData> {
        let value = self
            .gitlab_client()?
            .get_current_user()
            .await
            .map_err(upstream_error)?;
        Ok(CallToolResult::structured(wrap_array(value)))
    }

    #[tool(name = "gitlab_get_project")]
    pub(super) async fn gitlab_get_project(
        &self,
        Parameters(args): Parameters<GitlabGetProjectArgs>,
    ) -> Result<CallToolResult, ErrorData> {
        let value = self
            .gitlab_client()?
            .get_project(&args.project)
            .await
            .map_err(upstream_error)?;
        Ok(CallToolResult::structured(wrap_array(value)))
    }

    #[tool(name = "gitlab_list_merge_requests")]
    pub(super) async fn gitlab_list_merge_requests(
        &self,
        Parameters(args): Parameters<GitlabListMergeRequestsArgs>,
    ) -> Result<CallToolResult, ErrorData> {
        let value = self
            .gitlab_client()?
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
            .map_err(upstream_error)?;
        Ok(CallToolResult::structured(wrap_array(value)))
    }

    #[tool(name = "gitlab_get_merge_request")]
    pub(super) async fn gitlab_get_merge_request(
        &self,
        Parameters(args): Parameters<GitlabGetMergeRequestArgs>,
    ) -> Result<CallToolResult, ErrorData> {
        let value = self
            .gitlab_client()?
            .get_merge_request(GetMergeRequestRequest {
                project: args.project,
                merge_request_iid: args.merge_request_iid,
                include_diverged_commits_count: args.include_diverged_commits_count,
                include_rebase_in_progress: args.include_rebase_in_progress,
            })
            .await
            .map_err(upstream_error)?;
        Ok(CallToolResult::structured(wrap_array(value)))
    }

    #[tool(name = "gitlab_list_merge_request_commits")]
    pub(super) async fn gitlab_list_merge_request_commits(
        &self,
        Parameters(args): Parameters<GitlabListMergeRequestCommitsArgs>,
    ) -> Result<CallToolResult, ErrorData> {
        let value = self
            .gitlab_client()?
            .list_merge_request_commits(ListMergeRequestCommitsRequest {
                project: args.project,
                merge_request_iid: args.merge_request_iid,
                page: args.page,
                per_page: args.per_page,
            })
            .await
            .map_err(upstream_error)?;
        Ok(CallToolResult::structured(wrap_array(value)))
    }

    #[tool(name = "gitlab_list_merge_request_diffs")]
    pub(super) async fn gitlab_list_merge_request_diffs(
        &self,
        Parameters(args): Parameters<GitlabListMergeRequestDiffsArgs>,
    ) -> Result<CallToolResult, ErrorData> {
        let value = self
            .gitlab_client()?
            .list_merge_request_diffs(ListMergeRequestDiffsRequest {
                project: args.project,
                merge_request_iid: args.merge_request_iid,
                max_diff_bytes: args.max_diff_bytes,
                page: args.page,
                per_page: args.per_page,
            })
            .await
            .map_err(upstream_error)?;
        Ok(CallToolResult::structured(wrap_array(value)))
    }

    #[tool(name = "gitlab_list_merge_request_pipelines")]
    pub(super) async fn gitlab_list_merge_request_pipelines(
        &self,
        Parameters(args): Parameters<GitlabListMergeRequestPipelinesArgs>,
    ) -> Result<CallToolResult, ErrorData> {
        let value = self
            .gitlab_client()?
            .list_merge_request_pipelines(ListMergeRequestPipelinesRequest {
                project: args.project,
                merge_request_iid: args.merge_request_iid,
                page: args.page,
                per_page: args.per_page,
            })
            .await
            .map_err(upstream_error)?;
        Ok(CallToolResult::structured(wrap_array(value)))
    }

    #[tool(name = "gitlab_create_merge_request")]
    pub(super) async fn gitlab_create_merge_request(
        &self,
        Parameters(args): Parameters<GitlabCreateMergeRequestArgs>,
    ) -> Result<CallToolResult, ErrorData> {
        let value = self
            .gitlab_client()?
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
            .map_err(upstream_error)?;
        Ok(CallToolResult::structured(wrap_array(value)))
    }

    #[tool(name = "gitlab_update_merge_request")]
    pub(super) async fn gitlab_update_merge_request(
        &self,
        Parameters(args): Parameters<GitlabUpdateMergeRequestArgs>,
    ) -> Result<CallToolResult, ErrorData> {
        let value = self
            .gitlab_client()?
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
            .map_err(upstream_error)?;
        Ok(CallToolResult::structured(wrap_array(value)))
    }

    #[tool(name = "gitlab_add_merge_request_note")]
    pub(super) async fn gitlab_add_merge_request_note(
        &self,
        Parameters(args): Parameters<GitlabAddMergeRequestNoteArgs>,
    ) -> Result<CallToolResult, ErrorData> {
        let value = self
            .gitlab_client()?
            .add_merge_request_note(&args.project, args.merge_request_iid, args.body)
            .await
            .map_err(upstream_error)?;
        Ok(CallToolResult::structured(wrap_array(value)))
    }

    #[tool(name = "gitlab_reply_merge_request_discussion")]
    pub(super) async fn gitlab_reply_merge_request_discussion(
        &self,
        Parameters(args): Parameters<GitlabReplyMergeRequestDiscussionArgs>,
    ) -> Result<CallToolResult, ErrorData> {
        let value = self
            .gitlab_client()?
            .reply_merge_request_discussion(
                &args.project,
                args.merge_request_iid,
                &args.discussion_id,
                args.body,
            )
            .await
            .map_err(upstream_error)?;
        Ok(CallToolResult::structured(wrap_array(value)))
    }

    #[tool(name = "gitlab_resolve_merge_request_discussion")]
    pub(super) async fn gitlab_resolve_merge_request_discussion(
        &self,
        Parameters(args): Parameters<GitlabResolveMergeRequestDiscussionArgs>,
    ) -> Result<CallToolResult, ErrorData> {
        let value = self
            .gitlab_client()?
            .resolve_merge_request_discussion(
                &args.project,
                args.merge_request_iid,
                &args.discussion_id,
                args.resolved,
            )
            .await
            .map_err(upstream_error)?;
        Ok(CallToolResult::structured(wrap_array(value)))
    }

    #[tool(name = "gitlab_get_merge_request_approval_state")]
    pub(super) async fn gitlab_get_merge_request_approval_state(
        &self,
        Parameters(args): Parameters<GitlabMergeRequestRefArgs>,
    ) -> Result<CallToolResult, ErrorData> {
        let value = self
            .gitlab_client()?
            .get_merge_request_approval_state(&args.project, args.merge_request_iid)
            .await
            .map_err(upstream_error)?;
        Ok(CallToolResult::structured(wrap_array(value)))
    }

    #[tool(name = "gitlab_set_merge_request_approval")]
    pub(super) async fn gitlab_set_merge_request_approval(
        &self,
        Parameters(args): Parameters<GitlabSetMergeRequestApprovalArgs>,
    ) -> Result<CallToolResult, ErrorData> {
        let client = self.gitlab_client()?;
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
        .map_err(upstream_error)?;
        Ok(CallToolResult::structured(wrap_array(value)))
    }

    #[tool(name = "gitlab_accept_merge_request")]
    pub(super) async fn gitlab_accept_merge_request(
        &self,
        Parameters(args): Parameters<GitlabAcceptMergeRequestArgs>,
    ) -> Result<CallToolResult, ErrorData> {
        let value = self
            .gitlab_client()?
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
            .map_err(upstream_error)?;
        Ok(CallToolResult::structured(wrap_array(value)))
    }
}

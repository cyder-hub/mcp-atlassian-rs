use crate::{
    gitlab::tools as gitlab_tools,
    operations::{self, operation_error_to_mcp, operation_result_to_mcp},
};
use rmcp::{
    ErrorData, handler::server::wrapper::Parameters, model::CallToolResult, tool, tool_router,
};

use super::WorkhubMcpServer;

fn gitlab_operation_result(
    result: Result<operations::OperationResult, operations::OperationError>,
) -> Result<CallToolResult, ErrorData> {
    result
        .map(operation_result_to_mcp)
        .map_err(operation_error_to_mcp)
}

#[tool_router(router = gitlab_tool_router, vis = "pub(super)")]
impl WorkhubMcpServer {
    #[tool(name = "gitlab_get_current_user")]
    pub(super) async fn gitlab_get_current_user(
        &self,
        Parameters(args): Parameters<gitlab_tools::GitlabGetCurrentUserArgs>,
    ) -> Result<CallToolResult, ErrorData> {
        gitlab_operation_result(operations::gitlab::get_current_user(&self.context, args).await)
    }

    #[tool(name = "gitlab_get_project")]
    pub(super) async fn gitlab_get_project(
        &self,
        Parameters(args): Parameters<gitlab_tools::GitlabGetProjectArgs>,
    ) -> Result<CallToolResult, ErrorData> {
        gitlab_operation_result(operations::gitlab::get_project(&self.context, args).await)
    }

    #[tool(name = "gitlab_list_merge_requests")]
    pub(super) async fn gitlab_list_merge_requests(
        &self,
        Parameters(args): Parameters<gitlab_tools::GitlabListMergeRequestsArgs>,
    ) -> Result<CallToolResult, ErrorData> {
        gitlab_operation_result(operations::gitlab::list_merge_requests(&self.context, args).await)
    }

    #[tool(name = "gitlab_get_merge_request")]
    pub(super) async fn gitlab_get_merge_request(
        &self,
        Parameters(args): Parameters<gitlab_tools::GitlabGetMergeRequestArgs>,
    ) -> Result<CallToolResult, ErrorData> {
        gitlab_operation_result(operations::gitlab::get_merge_request(&self.context, args).await)
    }

    #[tool(name = "gitlab_list_merge_request_commits")]
    pub(super) async fn gitlab_list_merge_request_commits(
        &self,
        Parameters(args): Parameters<gitlab_tools::GitlabListMergeRequestCommitsArgs>,
    ) -> Result<CallToolResult, ErrorData> {
        gitlab_operation_result(
            operations::gitlab::list_merge_request_commits(&self.context, args).await,
        )
    }

    #[tool(name = "gitlab_list_merge_request_diffs")]
    pub(super) async fn gitlab_list_merge_request_diffs(
        &self,
        Parameters(args): Parameters<gitlab_tools::GitlabListMergeRequestDiffsArgs>,
    ) -> Result<CallToolResult, ErrorData> {
        gitlab_operation_result(
            operations::gitlab::list_merge_request_diffs(&self.context, args).await,
        )
    }

    #[tool(name = "gitlab_list_merge_request_pipelines")]
    pub(super) async fn gitlab_list_merge_request_pipelines(
        &self,
        Parameters(args): Parameters<gitlab_tools::GitlabListMergeRequestPipelinesArgs>,
    ) -> Result<CallToolResult, ErrorData> {
        gitlab_operation_result(
            operations::gitlab::list_merge_request_pipelines(&self.context, args).await,
        )
    }

    #[tool(name = "gitlab_create_merge_request")]
    pub(super) async fn gitlab_create_merge_request(
        &self,
        Parameters(args): Parameters<gitlab_tools::GitlabCreateMergeRequestArgs>,
    ) -> Result<CallToolResult, ErrorData> {
        gitlab_operation_result(operations::gitlab::create_merge_request(&self.context, args).await)
    }

    #[tool(name = "gitlab_update_merge_request")]
    pub(super) async fn gitlab_update_merge_request(
        &self,
        Parameters(args): Parameters<gitlab_tools::GitlabUpdateMergeRequestArgs>,
    ) -> Result<CallToolResult, ErrorData> {
        gitlab_operation_result(operations::gitlab::update_merge_request(&self.context, args).await)
    }

    #[tool(name = "gitlab_add_merge_request_note")]
    pub(super) async fn gitlab_add_merge_request_note(
        &self,
        Parameters(args): Parameters<gitlab_tools::GitlabAddMergeRequestNoteArgs>,
    ) -> Result<CallToolResult, ErrorData> {
        gitlab_operation_result(
            operations::gitlab::add_merge_request_note(&self.context, args).await,
        )
    }

    #[tool(name = "gitlab_reply_merge_request_discussion")]
    pub(super) async fn gitlab_reply_merge_request_discussion(
        &self,
        Parameters(args): Parameters<gitlab_tools::GitlabReplyMergeRequestDiscussionArgs>,
    ) -> Result<CallToolResult, ErrorData> {
        gitlab_operation_result(
            operations::gitlab::reply_merge_request_discussion(&self.context, args).await,
        )
    }

    #[tool(name = "gitlab_resolve_merge_request_discussion")]
    pub(super) async fn gitlab_resolve_merge_request_discussion(
        &self,
        Parameters(args): Parameters<gitlab_tools::GitlabResolveMergeRequestDiscussionArgs>,
    ) -> Result<CallToolResult, ErrorData> {
        gitlab_operation_result(
            operations::gitlab::resolve_merge_request_discussion(&self.context, args).await,
        )
    }

    #[tool(name = "gitlab_get_merge_request_approval_state")]
    pub(super) async fn gitlab_get_merge_request_approval_state(
        &self,
        Parameters(args): Parameters<gitlab_tools::GitlabMergeRequestRefArgs>,
    ) -> Result<CallToolResult, ErrorData> {
        gitlab_operation_result(
            operations::gitlab::get_merge_request_approval_state(&self.context, args).await,
        )
    }

    #[tool(name = "gitlab_set_merge_request_approval")]
    pub(super) async fn gitlab_set_merge_request_approval(
        &self,
        Parameters(args): Parameters<gitlab_tools::GitlabSetMergeRequestApprovalArgs>,
    ) -> Result<CallToolResult, ErrorData> {
        gitlab_operation_result(
            operations::gitlab::set_merge_request_approval(&self.context, args).await,
        )
    }

    #[tool(name = "gitlab_accept_merge_request")]
    pub(super) async fn gitlab_accept_merge_request(
        &self,
        Parameters(args): Parameters<gitlab_tools::GitlabAcceptMergeRequestArgs>,
    ) -> Result<CallToolResult, ErrorData> {
        gitlab_operation_result(operations::gitlab::accept_merge_request(&self.context, args).await)
    }
}

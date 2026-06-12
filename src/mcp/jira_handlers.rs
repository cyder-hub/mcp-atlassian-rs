use crate::{
    jira::tools::{
        JiraAddCommentArgs, JiraAddIssuesToSprintArgs, JiraAddWatcherArgs, JiraAddWorklogArgs,
        JiraCreateIssueArgs, JiraCreateIssueLinkArgs, JiraCreateIssuesArgs,
        JiraCreateProjectVersionArgs, JiraCreateProjectVersionsArgs, JiraCreateRemoteIssueLinkArgs,
        JiraCreateSprintArgs, JiraDeleteIssueArgs, JiraDeleteIssueLinkArgs, JiraEditCommentArgs,
        JiraGetFieldOptionsArgs, JiraGetIssueArgs, JiraGetIssueAttachmentsArgs,
        JiraGetIssueChangelogsArgs, JiraGetIssueDevelopmentArgs, JiraGetIssueImagesArgs,
        JiraGetIssueSlaMetricsArgs, JiraGetIssueTimelineArgs, JiraGetIssuesDevelopmentArgs,
        JiraGetProjectIssuesArgs, JiraGetServiceDeskForProjectArgs, JiraGetTransitionsArgs,
        JiraGetUserArgs, JiraListAgileBoardsArgs, JiraListBoardIssuesArgs,
        JiraListBoardSprintsArgs, JiraListIssueLinkTypesArgs, JiraListIssueWatchersArgs,
        JiraListIssueWorklogsArgs, JiraListProjectComponentsArgs, JiraListProjectVersionsArgs,
        JiraListProjectsArgs, JiraListServiceDeskQueueIssuesArgs, JiraListServiceDeskQueuesArgs,
        JiraListSprintIssuesArgs, JiraRemoveWatcherArgs, JiraSearchArgs, JiraSearchFieldsArgs,
        JiraSetIssueParentArgs, JiraTransitionIssueArgs, JiraUpdateIssueArgs, JiraUpdateSprintArgs,
    },
    operations::{self, operation_error_to_mcp, operation_result_to_mcp},
};
use rmcp::{
    ErrorData, handler::server::wrapper::Parameters, model::CallToolResult, tool, tool_router,
};

use super::WorkhubMcpServer;

fn jira_operation_result(
    result: Result<operations::OperationResult, operations::OperationError>,
) -> Result<CallToolResult, ErrorData> {
    result
        .map(operation_result_to_mcp)
        .map_err(operation_error_to_mcp)
}

#[tool_router(router = jira_tool_router, vis = "pub(super)")]
impl WorkhubMcpServer {
    #[tool(name = "jira_get_issue")]
    pub(super) async fn get_issue(
        &self,
        Parameters(args): Parameters<JiraGetIssueArgs>,
    ) -> Result<CallToolResult, ErrorData> {
        jira_operation_result(operations::jira::get_issue(&self.context, args).await)
    }

    #[tool(name = "jira_search_issues")]
    pub(super) async fn search_issues(
        &self,
        Parameters(args): Parameters<JiraSearchArgs>,
    ) -> Result<CallToolResult, ErrorData> {
        jira_operation_result(operations::jira::search_issues(&self.context, args).await)
    }

    #[tool(name = "jira_list_project_issues")]
    pub(super) async fn list_project_issues(
        &self,
        Parameters(args): Parameters<JiraGetProjectIssuesArgs>,
    ) -> Result<CallToolResult, ErrorData> {
        jira_operation_result(operations::jira::list_project_issues(&self.context, args).await)
    }

    #[tool(name = "jira_search_fields")]
    pub(super) async fn search_fields(
        &self,
        Parameters(args): Parameters<JiraSearchFieldsArgs>,
    ) -> Result<CallToolResult, ErrorData> {
        jira_operation_result(operations::jira::search_fields(&self.context, args).await)
    }

    #[tool(name = "jira_list_field_options")]
    pub(super) async fn list_field_options(
        &self,
        Parameters(args): Parameters<JiraGetFieldOptionsArgs>,
    ) -> Result<CallToolResult, ErrorData> {
        jira_operation_result(operations::jira::list_field_options(&self.context, args).await)
    }

    #[tool(name = "jira_add_issue_comment")]
    pub(super) async fn add_issue_comment(
        &self,
        Parameters(args): Parameters<JiraAddCommentArgs>,
    ) -> Result<CallToolResult, ErrorData> {
        jira_operation_result(operations::jira::add_issue_comment(&self.context, args).await)
    }

    #[tool(name = "jira_update_issue_comment")]
    pub(super) async fn update_issue_comment(
        &self,
        Parameters(args): Parameters<JiraEditCommentArgs>,
    ) -> Result<CallToolResult, ErrorData> {
        jira_operation_result(operations::jira::update_issue_comment(&self.context, args).await)
    }

    #[tool(name = "jira_list_issue_transitions")]
    pub(super) async fn list_issue_transitions(
        &self,
        Parameters(args): Parameters<JiraGetTransitionsArgs>,
    ) -> Result<CallToolResult, ErrorData> {
        jira_operation_result(operations::jira::list_issue_transitions(&self.context, args).await)
    }

    #[tool(name = "jira_transition_issue")]
    pub(super) async fn transition_issue(
        &self,
        Parameters(args): Parameters<JiraTransitionIssueArgs>,
    ) -> Result<CallToolResult, ErrorData> {
        jira_operation_result(operations::jira::transition_issue(&self.context, args).await)
    }

    #[tool(name = "jira_create_issue")]
    pub(super) async fn create_issue(
        &self,
        Parameters(args): Parameters<JiraCreateIssueArgs>,
    ) -> Result<CallToolResult, ErrorData> {
        jira_operation_result(operations::jira::create_issue(&self.context, args).await)
    }

    #[tool(name = "jira_create_issues")]
    pub(super) async fn create_issues(
        &self,
        Parameters(args): Parameters<JiraCreateIssuesArgs>,
    ) -> Result<CallToolResult, ErrorData> {
        jira_operation_result(operations::jira::create_issues(&self.context, args).await)
    }

    #[tool(name = "jira_get_issue_changelogs")]
    pub(super) async fn get_issue_changelogs(
        &self,
        Parameters(args): Parameters<JiraGetIssueChangelogsArgs>,
    ) -> Result<CallToolResult, ErrorData> {
        jira_operation_result(operations::jira::get_issue_changelogs(&self.context, args).await)
    }

    #[tool(name = "jira_update_issue")]
    pub(super) async fn update_issue(
        &self,
        Parameters(args): Parameters<JiraUpdateIssueArgs>,
    ) -> Result<CallToolResult, ErrorData> {
        jira_operation_result(operations::jira::update_issue(&self.context, args).await)
    }

    #[tool(name = "jira_delete_issue")]
    pub(super) async fn delete_issue(
        &self,
        Parameters(args): Parameters<JiraDeleteIssueArgs>,
    ) -> Result<CallToolResult, ErrorData> {
        jira_operation_result(operations::jira::delete_issue(&self.context, args).await)
    }

    #[tool(name = "jira_list_projects")]
    pub(super) async fn list_projects(
        &self,
        Parameters(args): Parameters<JiraListProjectsArgs>,
    ) -> Result<CallToolResult, ErrorData> {
        jira_operation_result(operations::jira::list_projects(&self.context, args).await)
    }

    #[tool(name = "jira_list_project_versions")]
    pub(super) async fn list_project_versions(
        &self,
        Parameters(args): Parameters<JiraListProjectVersionsArgs>,
    ) -> Result<CallToolResult, ErrorData> {
        jira_operation_result(operations::jira::list_project_versions(&self.context, args).await)
    }

    #[tool(name = "jira_list_project_components")]
    pub(super) async fn list_project_components(
        &self,
        Parameters(args): Parameters<JiraListProjectComponentsArgs>,
    ) -> Result<CallToolResult, ErrorData> {
        jira_operation_result(operations::jira::list_project_components(&self.context, args).await)
    }

    #[tool(name = "jira_create_project_version")]
    pub(super) async fn create_project_version(
        &self,
        Parameters(args): Parameters<JiraCreateProjectVersionArgs>,
    ) -> Result<CallToolResult, ErrorData> {
        jira_operation_result(operations::jira::create_project_version(&self.context, args).await)
    }

    #[tool(name = "jira_create_project_versions")]
    pub(super) async fn create_project_versions(
        &self,
        Parameters(args): Parameters<JiraCreateProjectVersionsArgs>,
    ) -> Result<CallToolResult, ErrorData> {
        jira_operation_result(operations::jira::create_project_versions(&self.context, args).await)
    }

    #[tool(name = "jira_get_user")]
    pub(super) async fn get_user(
        &self,
        Parameters(args): Parameters<JiraGetUserArgs>,
    ) -> Result<CallToolResult, ErrorData> {
        jira_operation_result(operations::jira::get_user(&self.context, args).await)
    }

    #[tool(name = "jira_list_issue_watchers")]
    pub(super) async fn list_issue_watchers(
        &self,
        Parameters(args): Parameters<JiraListIssueWatchersArgs>,
    ) -> Result<CallToolResult, ErrorData> {
        jira_operation_result(operations::jira::list_issue_watchers(&self.context, args).await)
    }

    #[tool(name = "jira_add_issue_watcher")]
    pub(super) async fn add_issue_watcher(
        &self,
        Parameters(args): Parameters<JiraAddWatcherArgs>,
    ) -> Result<CallToolResult, ErrorData> {
        jira_operation_result(operations::jira::add_issue_watcher(&self.context, args).await)
    }

    #[tool(name = "jira_remove_issue_watcher")]
    pub(super) async fn remove_issue_watcher(
        &self,
        Parameters(args): Parameters<JiraRemoveWatcherArgs>,
    ) -> Result<CallToolResult, ErrorData> {
        jira_operation_result(operations::jira::remove_issue_watcher(&self.context, args).await)
    }

    #[tool(name = "jira_list_issue_worklogs")]
    pub(super) async fn list_issue_worklogs(
        &self,
        Parameters(args): Parameters<JiraListIssueWorklogsArgs>,
    ) -> Result<CallToolResult, ErrorData> {
        jira_operation_result(operations::jira::list_issue_worklogs(&self.context, args).await)
    }

    #[tool(name = "jira_add_issue_worklog")]
    pub(super) async fn add_issue_worklog(
        &self,
        Parameters(args): Parameters<JiraAddWorklogArgs>,
    ) -> Result<CallToolResult, ErrorData> {
        jira_operation_result(operations::jira::add_issue_worklog(&self.context, args).await)
    }

    #[tool(name = "jira_list_issue_link_types")]
    pub(super) async fn list_issue_link_types(
        &self,
        Parameters(args): Parameters<JiraListIssueLinkTypesArgs>,
    ) -> Result<CallToolResult, ErrorData> {
        jira_operation_result(operations::jira::list_issue_link_types(&self.context, args).await)
    }

    #[tool(name = "jira_set_issue_parent")]
    pub(super) async fn set_issue_parent(
        &self,
        Parameters(args): Parameters<JiraSetIssueParentArgs>,
    ) -> Result<CallToolResult, ErrorData> {
        jira_operation_result(operations::jira::set_issue_parent(&self.context, args).await)
    }

    #[tool(name = "jira_create_issue_link")]
    pub(super) async fn create_issue_link(
        &self,
        Parameters(args): Parameters<JiraCreateIssueLinkArgs>,
    ) -> Result<CallToolResult, ErrorData> {
        jira_operation_result(operations::jira::create_issue_link(&self.context, args).await)
    }

    #[tool(name = "jira_create_remote_issue_link")]
    pub(super) async fn create_remote_issue_link(
        &self,
        Parameters(args): Parameters<JiraCreateRemoteIssueLinkArgs>,
    ) -> Result<CallToolResult, ErrorData> {
        jira_operation_result(operations::jira::create_remote_issue_link(&self.context, args).await)
    }

    #[tool(name = "jira_delete_issue_link")]
    pub(super) async fn delete_issue_link(
        &self,
        Parameters(args): Parameters<JiraDeleteIssueLinkArgs>,
    ) -> Result<CallToolResult, ErrorData> {
        jira_operation_result(operations::jira::delete_issue_link(&self.context, args).await)
    }

    #[tool(name = "jira_get_issue_attachments")]
    pub(super) async fn get_issue_attachments(
        &self,
        Parameters(args): Parameters<JiraGetIssueAttachmentsArgs>,
    ) -> Result<CallToolResult, ErrorData> {
        jira_operation_result(operations::jira::get_issue_attachments(&self.context, args).await)
    }

    #[tool(name = "jira_get_issue_image_attachments")]
    pub(super) async fn get_issue_image_attachments(
        &self,
        Parameters(args): Parameters<JiraGetIssueImagesArgs>,
    ) -> Result<CallToolResult, ErrorData> {
        jira_operation_result(
            operations::jira::get_issue_image_attachments(&self.context, args).await,
        )
    }

    #[tool(name = "jira_list_agile_boards")]
    pub(super) async fn list_agile_boards(
        &self,
        Parameters(args): Parameters<JiraListAgileBoardsArgs>,
    ) -> Result<CallToolResult, ErrorData> {
        jira_operation_result(operations::jira::list_agile_boards(&self.context, args).await)
    }

    #[tool(name = "jira_list_board_issues")]
    pub(super) async fn list_board_issues(
        &self,
        Parameters(args): Parameters<JiraListBoardIssuesArgs>,
    ) -> Result<CallToolResult, ErrorData> {
        jira_operation_result(operations::jira::list_board_issues(&self.context, args).await)
    }

    #[tool(name = "jira_list_board_sprints")]
    pub(super) async fn list_board_sprints(
        &self,
        Parameters(args): Parameters<JiraListBoardSprintsArgs>,
    ) -> Result<CallToolResult, ErrorData> {
        jira_operation_result(operations::jira::list_board_sprints(&self.context, args).await)
    }

    #[tool(name = "jira_list_sprint_issues")]
    pub(super) async fn list_sprint_issues(
        &self,
        Parameters(args): Parameters<JiraListSprintIssuesArgs>,
    ) -> Result<CallToolResult, ErrorData> {
        jira_operation_result(operations::jira::list_sprint_issues(&self.context, args).await)
    }

    #[tool(name = "jira_create_sprint")]
    pub(super) async fn create_sprint(
        &self,
        Parameters(args): Parameters<JiraCreateSprintArgs>,
    ) -> Result<CallToolResult, ErrorData> {
        jira_operation_result(operations::jira::create_sprint(&self.context, args).await)
    }

    #[tool(name = "jira_update_sprint")]
    pub(super) async fn update_sprint(
        &self,
        Parameters(args): Parameters<JiraUpdateSprintArgs>,
    ) -> Result<CallToolResult, ErrorData> {
        jira_operation_result(operations::jira::update_sprint(&self.context, args).await)
    }

    #[tool(name = "jira_add_issues_to_sprint")]
    pub(super) async fn add_issues_to_sprint(
        &self,
        Parameters(args): Parameters<JiraAddIssuesToSprintArgs>,
    ) -> Result<CallToolResult, ErrorData> {
        jira_operation_result(operations::jira::add_issues_to_sprint(&self.context, args).await)
    }

    #[tool(name = "jira_get_project_service_desk")]
    pub(super) async fn get_project_service_desk(
        &self,
        Parameters(args): Parameters<JiraGetServiceDeskForProjectArgs>,
    ) -> Result<CallToolResult, ErrorData> {
        jira_operation_result(operations::jira::get_project_service_desk(&self.context, args).await)
    }

    #[tool(name = "jira_list_service_desk_queues")]
    pub(super) async fn list_service_desk_queues(
        &self,
        Parameters(args): Parameters<JiraListServiceDeskQueuesArgs>,
    ) -> Result<CallToolResult, ErrorData> {
        jira_operation_result(operations::jira::list_service_desk_queues(&self.context, args).await)
    }

    #[tool(name = "jira_list_service_desk_queue_issues")]
    pub(super) async fn list_service_desk_queue_issues(
        &self,
        Parameters(args): Parameters<JiraListServiceDeskQueueIssuesArgs>,
    ) -> Result<CallToolResult, ErrorData> {
        jira_operation_result(
            operations::jira::list_service_desk_queue_issues(&self.context, args).await,
        )
    }

    #[tool(name = "jira_get_issue_timeline")]
    pub(super) async fn get_issue_timeline(
        &self,
        Parameters(args): Parameters<JiraGetIssueTimelineArgs>,
    ) -> Result<CallToolResult, ErrorData> {
        jira_operation_result(operations::jira::get_issue_timeline(&self.context, args).await)
    }

    #[tool(name = "jira_get_issue_sla_metrics")]
    pub(super) async fn get_issue_sla_metrics(
        &self,
        Parameters(args): Parameters<JiraGetIssueSlaMetricsArgs>,
    ) -> Result<CallToolResult, ErrorData> {
        jira_operation_result(operations::jira::get_issue_sla_metrics(&self.context, args).await)
    }

    #[tool(name = "jira_get_issue_development")]
    pub(super) async fn get_issue_development(
        &self,
        Parameters(args): Parameters<JiraGetIssueDevelopmentArgs>,
    ) -> Result<CallToolResult, ErrorData> {
        jira_operation_result(operations::jira::get_issue_development(&self.context, args).await)
    }

    #[tool(name = "jira_get_issues_development")]
    pub(super) async fn get_issues_development(
        &self,
        Parameters(args): Parameters<JiraGetIssuesDevelopmentArgs>,
    ) -> Result<CallToolResult, ErrorData> {
        jira_operation_result(operations::jira::get_issues_development(&self.context, args).await)
    }
}

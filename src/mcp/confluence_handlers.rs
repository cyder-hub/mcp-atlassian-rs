use crate::{
    confluence::tools as confluence_tools,
    operations::{self, operation_error_to_mcp, operation_result_to_mcp},
};
use rmcp::{
    ErrorData, handler::server::wrapper::Parameters, model::CallToolResult, tool, tool_router,
};

use super::WorkhubMcpServer;

#[cfg(test)]
pub(super) const CONFLUENCE_DOWNLOAD_CONTENT_ATTACHMENTS_MAX_PAGES: u64 = 10;

fn confluence_operation_result(
    result: Result<operations::OperationResult, operations::OperationError>,
) -> Result<CallToolResult, ErrorData> {
    result
        .map(operation_result_to_mcp)
        .map_err(operation_error_to_mcp)
}

#[tool_router(router = confluence_tool_router, vis = "pub(super)")]
impl WorkhubMcpServer {
    #[tool(name = "confluence_search_content")]
    pub(super) async fn search_content(
        &self,
        Parameters(args): Parameters<confluence_tools::ConfluenceSearchArgs>,
    ) -> Result<CallToolResult, ErrorData> {
        confluence_operation_result(
            operations::confluence::search_content(&self.context, args).await,
        )
    }

    #[tool(name = "confluence_get_page")]
    pub(super) async fn get_page(
        &self,
        Parameters(args): Parameters<confluence_tools::ConfluenceGetPageArgs>,
    ) -> Result<CallToolResult, ErrorData> {
        confluence_operation_result(operations::confluence::get_page(&self.context, args).await)
    }

    #[tool(name = "confluence_list_page_children")]
    pub(super) async fn list_page_children(
        &self,
        Parameters(args): Parameters<confluence_tools::ConfluenceListPageChildrenArgs>,
    ) -> Result<CallToolResult, ErrorData> {
        confluence_operation_result(
            operations::confluence::list_page_children(&self.context, args).await,
        )
    }

    #[tool(name = "confluence_get_space_page_tree")]
    pub(super) async fn get_space_page_tree(
        &self,
        Parameters(args): Parameters<confluence_tools::ConfluenceGetSpacePageTreeArgs>,
    ) -> Result<CallToolResult, ErrorData> {
        confluence_operation_result(
            operations::confluence::get_space_page_tree(&self.context, args).await,
        )
    }

    #[tool(name = "confluence_create_page")]
    pub(super) async fn create_page(
        &self,
        Parameters(args): Parameters<confluence_tools::ConfluenceCreatePageArgs>,
    ) -> Result<CallToolResult, ErrorData> {
        confluence_operation_result(operations::confluence::create_page(&self.context, args).await)
    }

    #[tool(name = "confluence_update_page")]
    pub(super) async fn update_page(
        &self,
        Parameters(args): Parameters<confluence_tools::ConfluenceUpdatePageArgs>,
    ) -> Result<CallToolResult, ErrorData> {
        confluence_operation_result(operations::confluence::update_page(&self.context, args).await)
    }

    #[tool(name = "confluence_delete_page")]
    pub(super) async fn delete_page(
        &self,
        Parameters(args): Parameters<confluence_tools::ConfluenceDeletePageArgs>,
    ) -> Result<CallToolResult, ErrorData> {
        confluence_operation_result(operations::confluence::delete_page(&self.context, args).await)
    }

    #[tool(name = "confluence_move_page")]
    pub(super) async fn move_page(
        &self,
        Parameters(args): Parameters<confluence_tools::ConfluenceMovePageArgs>,
    ) -> Result<CallToolResult, ErrorData> {
        confluence_operation_result(operations::confluence::move_page(&self.context, args).await)
    }

    #[tool(name = "confluence_list_page_comments")]
    pub(super) async fn list_page_comments(
        &self,
        Parameters(args): Parameters<confluence_tools::ConfluenceListPageCommentsArgs>,
    ) -> Result<CallToolResult, ErrorData> {
        confluence_operation_result(
            operations::confluence::list_page_comments(&self.context, args).await,
        )
    }

    #[tool(name = "confluence_add_page_comment")]
    pub(super) async fn add_page_comment(
        &self,
        Parameters(args): Parameters<confluence_tools::ConfluenceAddCommentArgs>,
    ) -> Result<CallToolResult, ErrorData> {
        confluence_operation_result(
            operations::confluence::add_page_comment(&self.context, args).await,
        )
    }

    #[tool(name = "confluence_reply_to_comment")]
    pub(super) async fn reply_to_comment(
        &self,
        Parameters(args): Parameters<confluence_tools::ConfluenceReplyToCommentArgs>,
    ) -> Result<CallToolResult, ErrorData> {
        confluence_operation_result(
            operations::confluence::reply_to_comment(&self.context, args).await,
        )
    }

    #[tool(name = "confluence_list_content_labels")]
    pub(super) async fn list_content_labels(
        &self,
        Parameters(args): Parameters<confluence_tools::ConfluenceListContentLabelsArgs>,
    ) -> Result<CallToolResult, ErrorData> {
        confluence_operation_result(
            operations::confluence::list_content_labels(&self.context, args).await,
        )
    }

    #[tool(name = "confluence_add_content_label")]
    pub(super) async fn add_content_label(
        &self,
        Parameters(args): Parameters<confluence_tools::ConfluenceAddLabelArgs>,
    ) -> Result<CallToolResult, ErrorData> {
        confluence_operation_result(
            operations::confluence::add_content_label(&self.context, args).await,
        )
    }

    #[tool(name = "confluence_search_users")]
    pub(super) async fn search_users(
        &self,
        Parameters(args): Parameters<confluence_tools::ConfluenceSearchUserArgs>,
    ) -> Result<CallToolResult, ErrorData> {
        confluence_operation_result(operations::confluence::search_users(&self.context, args).await)
    }

    #[tool(name = "confluence_get_page_version")]
    pub(super) async fn get_page_version(
        &self,
        Parameters(args): Parameters<confluence_tools::ConfluenceGetPageVersionArgs>,
    ) -> Result<CallToolResult, ErrorData> {
        confluence_operation_result(
            operations::confluence::get_page_version(&self.context, args).await,
        )
    }

    #[tool(name = "confluence_get_page_diff")]
    pub(super) async fn get_page_diff(
        &self,
        Parameters(args): Parameters<confluence_tools::ConfluenceGetPageDiffArgs>,
    ) -> Result<CallToolResult, ErrorData> {
        confluence_operation_result(
            operations::confluence::get_page_diff(&self.context, args).await,
        )
    }

    #[tool(name = "confluence_get_page_view_analytics")]
    pub(super) async fn get_page_view_analytics(
        &self,
        Parameters(args): Parameters<confluence_tools::ConfluenceGetPageViewAnalyticsArgs>,
    ) -> Result<CallToolResult, ErrorData> {
        confluence_operation_result(
            operations::confluence::get_page_view_analytics(&self.context, args).await,
        )
    }

    #[tool(name = "confluence_upload_content_attachment")]
    pub(super) async fn upload_content_attachment(
        &self,
        Parameters(args): Parameters<confluence_tools::ConfluenceUploadContentAttachmentArgs>,
    ) -> Result<CallToolResult, ErrorData> {
        confluence_operation_result(
            operations::confluence::upload_content_attachment(&self.context, args).await,
        )
    }

    #[tool(name = "confluence_upload_content_attachments")]
    pub(super) async fn upload_content_attachments(
        &self,
        Parameters(args): Parameters<confluence_tools::ConfluenceUploadContentAttachmentsArgs>,
    ) -> Result<CallToolResult, ErrorData> {
        confluence_operation_result(
            operations::confluence::upload_content_attachments(&self.context, args).await,
        )
    }

    #[tool(name = "confluence_list_content_attachments")]
    pub(super) async fn list_content_attachments(
        &self,
        Parameters(args): Parameters<confluence_tools::ConfluenceListContentAttachmentsArgs>,
    ) -> Result<CallToolResult, ErrorData> {
        confluence_operation_result(
            operations::confluence::list_content_attachments(&self.context, args).await,
        )
    }

    #[tool(name = "confluence_download_attachment")]
    pub(super) async fn download_attachment(
        &self,
        Parameters(args): Parameters<confluence_tools::ConfluenceDownloadAttachmentArgs>,
    ) -> Result<CallToolResult, ErrorData> {
        confluence_operation_result(
            operations::confluence::download_attachment(&self.context, args).await,
        )
    }

    #[tool(name = "confluence_download_content_attachments")]
    pub(super) async fn download_content_attachments(
        &self,
        Parameters(args): Parameters<confluence_tools::ConfluenceDownloadContentAttachmentsArgs>,
    ) -> Result<CallToolResult, ErrorData> {
        confluence_operation_result(
            operations::confluence::download_content_attachments(&self.context, args).await,
        )
    }

    #[tool(name = "confluence_delete_attachment")]
    pub(super) async fn delete_attachment(
        &self,
        Parameters(args): Parameters<confluence_tools::ConfluenceDeleteAttachmentArgs>,
    ) -> Result<CallToolResult, ErrorData> {
        confluence_operation_result(
            operations::confluence::delete_attachment(&self.context, args).await,
        )
    }

    #[tool(name = "confluence_get_content_image_attachments")]
    pub(super) async fn get_content_image_attachments(
        &self,
        Parameters(args): Parameters<confluence_tools::ConfluenceGetContentImageAttachmentsArgs>,
    ) -> Result<CallToolResult, ErrorData> {
        confluence_operation_result(
            operations::confluence::get_content_image_attachments(&self.context, args).await,
        )
    }
}

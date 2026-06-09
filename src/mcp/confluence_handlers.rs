use crate::{
    atlassian::error::AtlassianError,
    confluence::{
        config::ConfluenceDeployment,
        formatting::{ConfluenceContentFormat, content_to_storage},
        tools as confluence_tools,
    },
    mcp_confluence_helpers::{
        confluence_attachment_filename, confluence_attachment_id,
        confluence_attachment_with_content_value, confluence_file_path_display,
        confluence_is_image_attachment,
    },
    mcp_errors::atlassian_error,
};
use rmcp::{
    ErrorData, handler::server::wrapper::Parameters, model::CallToolResult, tool, tool_router,
};
use serde_json::{Value, json};

use super::{
    AtlassianMcpServer,
    confluence_values::{
        confluence_child_page_value, confluence_emoji_missing_page_id_status,
        confluence_expand_list, confluence_page_markdown_content, confluence_page_tool_value,
        confluence_positive_version_arg, confluence_split_file_paths,
        confluence_tree_page_sort_value, confluence_unified_diff, confluence_user_search_limit,
        confluence_write_page_value, normalize_confluence_user_search_query,
        optional_confluence_search_limit_arg, optional_u64_range_arg,
        parse_confluence_write_content_format,
    },
    optional_non_empty_arg, required_non_empty_arg,
};

const CONFLUENCE_PAGE_EXPAND: &[&str] = &[
    "body.storage",
    "version",
    "space",
    "ancestors",
    "metadata.labels",
    "history",
    "children.attachment",
];
const CONFLUENCE_CHILDREN_DEFAULT_LIMIT: u64 = 25;
const CONFLUENCE_CHILDREN_MAX_LIMIT: u64 = 50;
const CONFLUENCE_TREE_DEFAULT_LIMIT: u64 = 500;
const CONFLUENCE_TREE_MAX_LIMIT: u64 = 1_000;
const CONFLUENCE_TREE_PAGE_SIZE: u64 = 200;
pub(super) const CONFLUENCE_DOWNLOAD_CONTENT_ATTACHMENTS_MAX_PAGES: u64 = 10;

#[tool_router(router = confluence_tool_router, vis = "pub(super)")]
impl AtlassianMcpServer {
    #[tool(name = "confluence_search_content")]
    pub(super) async fn search_content(
        &self,
        Parameters(args): Parameters<confluence_tools::ConfluenceSearchArgs>,
    ) -> Result<CallToolResult, ErrorData> {
        let query = required_non_empty_arg(args.query, "query")?;
        let limit = optional_confluence_search_limit_arg(args.limit)?;
        let value = self
            .confluence_client()?
            .search_content(&query, limit, args.spaces_filter.as_deref())
            .await
            .map_err(atlassian_error)?
            .to_simplified_value();

        Ok(CallToolResult::structured(crate::mcp::wrap_array(value)))
    }

    #[tool(name = "confluence_get_page")]
    pub(super) async fn get_page(
        &self,
        Parameters(args): Parameters<confluence_tools::ConfluenceGetPageArgs>,
    ) -> Result<CallToolResult, ErrorData> {
        let include_metadata = args.include_metadata.unwrap_or(true);
        let convert_to_markdown = args.convert_to_markdown.unwrap_or(true);
        let client = self.confluence_client()?;

        if let Some(page_id) = optional_non_empty_arg(args.page_id) {
            let page = match client
                .get_page_by_id(&page_id, CONFLUENCE_PAGE_EXPAND)
                .await
            {
                Ok(page) => page,
                Err(AtlassianError::HttpStatus { status: 404, .. }) => {
                    return Ok(CallToolResult::structured_error(json!({
                        "error": format!("Failed to retrieve page by ID '{page_id}': page not found")
                    })));
                }
                Err(error) => return Err(atlassian_error(error)),
            };

            return Ok(CallToolResult::structured(confluence_page_tool_value(
                &page,
                include_metadata,
                convert_to_markdown,
            )));
        }

        let title = optional_non_empty_arg(args.title);
        let space_key = optional_non_empty_arg(args.space_key);
        let (Some(title), Some(space_key)) = (title, space_key) else {
            return Err(atlassian_error(AtlassianError::invalid_input(
                "Either page_id OR both title and space_key must be provided",
            )));
        };

        let Some(page) = client
            .get_page_by_title(&space_key, &title, CONFLUENCE_PAGE_EXPAND)
            .await
            .map_err(atlassian_error)?
        else {
            return Ok(CallToolResult::structured_error(json!({
                "error": format!("Page with title '{title}' not found in space '{space_key}'.")
            })));
        };

        Ok(CallToolResult::structured(confluence_page_tool_value(
            &page,
            include_metadata,
            convert_to_markdown,
        )))
    }

    #[tool(name = "confluence_list_page_children")]
    pub(super) async fn list_page_children(
        &self,
        Parameters(args): Parameters<confluence_tools::ConfluenceListPageChildrenArgs>,
    ) -> Result<CallToolResult, ErrorData> {
        let parent_id = required_non_empty_arg(args.parent_id, "parent_id")?;
        let limit = optional_u64_range_arg(
            args.limit,
            CONFLUENCE_CHILDREN_DEFAULT_LIMIT,
            CONFLUENCE_CHILDREN_MAX_LIMIT,
            "limit",
        )?;
        let start = args.start.unwrap_or(0);
        let include_content = args.include_content.unwrap_or(false);
        let include_folders = args.include_folders.unwrap_or(true);
        let convert_to_markdown = args.convert_to_markdown.unwrap_or(true);
        let expand = confluence_expand_list(args.expand, include_content);
        let expand_refs = expand.iter().map(String::as_str).collect::<Vec<_>>();
        let children = self
            .confluence_client()?
            .get_page_children(
                &parent_id,
                Some(start),
                Some(limit),
                &expand_refs,
                include_folders,
            )
            .await
            .map_err(atlassian_error)?;
        let results = children
            .results
            .iter()
            .map(|page| confluence_child_page_value(page, include_content, convert_to_markdown))
            .collect::<Vec<_>>();

        Ok(CallToolResult::structured(json!({
            "parent_id": parent_id,
            "count": results.len(),
            "limit_requested": limit,
            "start_requested": start,
            "page_results": children.page_results,
            "folder_results": children.folder_results,
            "queries": {
                "page": children.page_query,
                "folder": children.folder_query,
            },
            "results": results,
        })))
    }

    #[tool(name = "confluence_get_space_page_tree")]
    pub(super) async fn get_space_page_tree(
        &self,
        Parameters(args): Parameters<confluence_tools::ConfluenceGetSpacePageTreeArgs>,
    ) -> Result<CallToolResult, ErrorData> {
        let space_key = required_non_empty_arg(args.space_key, "space_key")?;
        let limit = optional_u64_range_arg(
            args.limit,
            CONFLUENCE_TREE_DEFAULT_LIMIT,
            CONFLUENCE_TREE_MAX_LIMIT,
            "limit",
        )?;
        let client = self.confluence_client()?;
        let mut pages = Vec::new();
        let mut start = 0;
        let mut next_link_exists = false;

        while pages.len() < limit as usize {
            let fetch_limit = CONFLUENCE_TREE_PAGE_SIZE.min(limit - pages.len() as u64);
            let response = client
                .get_space_pages(&space_key, Some(start), Some(fetch_limit), &["ancestors"])
                .await
                .map_err(atlassian_error)?;
            let batch_len = response.results.len() as u64;
            next_link_exists = response.links.get("next").and_then(Value::as_str).is_some();
            pages.extend(response.results);

            if batch_len == 0 || !next_link_exists {
                break;
            }
            start += batch_len;
        }

        let has_more = pages.len() >= limit as usize && next_link_exists;
        let mut tree_pages = pages
            .iter()
            .map(confluence_tree_page_sort_value)
            .collect::<Vec<_>>();
        tree_pages.sort_by(|left, right| {
            left.depth
                .cmp(&right.depth)
                .then(left.position_sort.cmp(&right.position_sort))
                .then(left.title.cmp(&right.title))
        });
        let result_pages = tree_pages
            .into_iter()
            .map(|page| page.value)
            .collect::<Vec<_>>();
        let mut result = json!({
            "space_key": space_key,
            "total_pages": result_pages.len(),
            "has_more": has_more,
            "pages": result_pages,
        });
        if has_more {
            result["next_start"] = Value::from(start);
        }

        Ok(CallToolResult::structured(crate::mcp::wrap_array(result)))
    }

    #[tool(name = "confluence_create_page")]
    pub(super) async fn create_page(
        &self,
        Parameters(args): Parameters<confluence_tools::ConfluenceCreatePageArgs>,
    ) -> Result<CallToolResult, ErrorData> {
        let space_key = required_non_empty_arg(args.space_key, "space_key")?;
        let title = required_non_empty_arg(args.title, "title")?;
        let content = required_non_empty_arg(args.content, "content")?;
        let format = parse_confluence_write_content_format(args.content_format.as_deref())?;
        let storage_body = content_to_storage(&content, format);
        let parent_id = optional_non_empty_arg(args.parent_id);
        let client = self.confluence_client()?;
        let page = client
            .create_page(&space_key, &title, &storage_body, parent_id.as_deref())
            .await
            .map_err(atlassian_error)?;
        let emoji_status = match page.id.as_deref() {
            Some(page_id) => {
                client
                    .set_page_emoji_best_effort(page_id, args.emoji.as_deref())
                    .await
            }
            None => confluence_emoji_missing_page_id_status(args.emoji.as_deref()),
        };

        Ok(CallToolResult::structured(json!({
            "message": "Page created successfully",
            "page": confluence_write_page_value(&page, args.include_content.unwrap_or(false)),
            "emoji_status": emoji_status,
        })))
    }

    #[tool(name = "confluence_update_page")]
    pub(super) async fn update_page(
        &self,
        Parameters(args): Parameters<confluence_tools::ConfluenceUpdatePageArgs>,
    ) -> Result<CallToolResult, ErrorData> {
        let page_id = required_non_empty_arg(args.page_id, "page_id")?;
        let title = required_non_empty_arg(args.title, "title")?;
        let content = required_non_empty_arg(args.content, "content")?;
        let format = parse_confluence_write_content_format(args.content_format.as_deref())?;
        let storage_body = content_to_storage(&content, format);
        let parent_id = optional_non_empty_arg(args.parent_id);
        let client = self.confluence_client()?;
        let page = client
            .update_page(
                &page_id,
                &title,
                &storage_body,
                parent_id.as_deref(),
                args.is_minor_edit.unwrap_or(false),
                args.version_comment.as_deref(),
            )
            .await
            .map_err(atlassian_error)?;
        let emoji_status = match page.id.as_deref() {
            Some(page_id) => {
                client
                    .set_page_emoji_best_effort(page_id, args.emoji.as_deref())
                    .await
            }
            None => confluence_emoji_missing_page_id_status(args.emoji.as_deref()),
        };

        Ok(CallToolResult::structured(json!({
            "message": "Page updated successfully",
            "page": confluence_write_page_value(&page, args.include_content.unwrap_or(false)),
            "emoji_status": emoji_status,
        })))
    }

    #[tool(name = "confluence_delete_page")]
    pub(super) async fn delete_page(
        &self,
        Parameters(args): Parameters<confluence_tools::ConfluenceDeletePageArgs>,
    ) -> Result<CallToolResult, ErrorData> {
        let page_id = required_non_empty_arg(args.page_id, "page_id")?;
        match self.confluence_client()?.delete_page(&page_id).await {
            Ok(_) => Ok(CallToolResult::structured(json!({
                "success": true,
                "message": format!("Page {page_id} deleted successfully"),
            }))),
            Err(error) => Ok(CallToolResult::structured_error(json!({
                "success": false,
                "message": format!("Error deleting page {page_id}"),
                "error": error.to_string(),
            }))),
        }
    }

    #[tool(name = "confluence_move_page")]
    pub(super) async fn move_page(
        &self,
        Parameters(args): Parameters<confluence_tools::ConfluenceMovePageArgs>,
    ) -> Result<CallToolResult, ErrorData> {
        let page_id = required_non_empty_arg(args.page_id, "page_id")?;
        let target_parent_id = optional_non_empty_arg(args.target_parent_id);
        let page = self
            .confluence_client()?
            .move_page(
                &page_id,
                target_parent_id.as_deref(),
                args.target_space_key.as_deref(),
                args.position.as_deref(),
            )
            .await
            .map_err(atlassian_error)?;

        Ok(CallToolResult::structured(json!({
            "message": "Page moved successfully",
            "page": confluence_write_page_value(&page, true),
        })))
    }

    #[tool(name = "confluence_list_page_comments")]
    pub(super) async fn list_page_comments(
        &self,
        Parameters(args): Parameters<confluence_tools::ConfluenceListPageCommentsArgs>,
    ) -> Result<CallToolResult, ErrorData> {
        let page_id = required_non_empty_arg(args.page_id, "page_id")?;
        let comments = self
            .confluence_client()?
            .get_page_comments(&page_id)
            .await
            .map_err(atlassian_error)?;
        let values = comments
            .results
            .iter()
            .map(|comment| comment.to_simplified_value())
            .collect::<Vec<_>>();

        Ok(CallToolResult::structured(json!({
            "page_id": page_id,
            "count": values.len(),
            "comments": values,
            "start": comments.start,
            "limit": comments.limit,
            "size": comments.size,
            "links": comments.links,
        })))
    }

    #[tool(name = "confluence_add_page_comment")]
    pub(super) async fn add_page_comment(
        &self,
        Parameters(args): Parameters<confluence_tools::ConfluenceAddCommentArgs>,
    ) -> Result<CallToolResult, ErrorData> {
        let page_id = required_non_empty_arg(args.page_id, "page_id")?;
        let body = required_non_empty_arg(args.body, "body")?;
        let storage_body = content_to_storage(&body, ConfluenceContentFormat::Markdown);

        match self
            .confluence_client()?
            .add_comment(&page_id, &storage_body)
            .await
        {
            Ok(comment) => Ok(CallToolResult::structured(json!({
                "success": true,
                "message": "Comment added successfully",
                "comment": comment.to_simplified_value(),
            }))),
            Err(error) => Ok(CallToolResult::structured_error(json!({
                "success": false,
                "message": format!("Error adding comment to page {page_id}"),
                "error": error.to_string(),
            }))),
        }
    }

    #[tool(name = "confluence_reply_to_comment")]
    pub(super) async fn reply_to_comment(
        &self,
        Parameters(args): Parameters<confluence_tools::ConfluenceReplyToCommentArgs>,
    ) -> Result<CallToolResult, ErrorData> {
        let comment_id = required_non_empty_arg(args.comment_id, "comment_id")?;
        let body = required_non_empty_arg(args.body, "body")?;
        let storage_body = content_to_storage(&body, ConfluenceContentFormat::Markdown);

        match self
            .confluence_client()?
            .reply_to_comment(&comment_id, &storage_body)
            .await
        {
            Ok(comment) => Ok(CallToolResult::structured(json!({
                "success": true,
                "message": "Reply added successfully",
                "comment": comment.to_simplified_value(),
            }))),
            Err(error) => Ok(CallToolResult::structured_error(json!({
                "success": false,
                "message": format!("Error replying to comment {comment_id}"),
                "error": error.to_string(),
            }))),
        }
    }

    #[tool(name = "confluence_list_content_labels")]
    pub(super) async fn list_content_labels(
        &self,
        Parameters(args): Parameters<confluence_tools::ConfluenceListContentLabelsArgs>,
    ) -> Result<CallToolResult, ErrorData> {
        let content_id = required_non_empty_arg(args.page_id, "page_id")?;
        let labels = self
            .confluence_client()?
            .get_labels(&content_id)
            .await
            .map_err(atlassian_error)?;
        let values = labels
            .results
            .iter()
            .map(|label| label.to_simplified_value())
            .collect::<Vec<_>>();

        Ok(CallToolResult::structured(json!({
            "content_id": content_id,
            "count": values.len(),
            "labels": values,
            "start": labels.start,
            "limit": labels.limit,
            "size": labels.size,
            "links": labels.links,
        })))
    }

    #[tool(name = "confluence_add_content_label")]
    pub(super) async fn add_content_label(
        &self,
        Parameters(args): Parameters<confluence_tools::ConfluenceAddLabelArgs>,
    ) -> Result<CallToolResult, ErrorData> {
        let content_id = required_non_empty_arg(args.page_id, "page_id")?;
        let name = required_non_empty_arg(args.name, "name")?;
        let labels = self
            .confluence_client()?
            .add_label(&content_id, &name)
            .await
            .map_err(atlassian_error)?;
        let values = labels
            .results
            .iter()
            .map(|label| label.to_simplified_value())
            .collect::<Vec<_>>();

        Ok(CallToolResult::structured(json!({
            "message": "Label added successfully",
            "content_id": content_id,
            "count": values.len(),
            "labels": values,
            "start": labels.start,
            "limit": labels.limit,
            "size": labels.size,
            "links": labels.links,
        })))
    }

    #[tool(name = "confluence_search_users")]
    pub(super) async fn search_users(
        &self,
        Parameters(args): Parameters<confluence_tools::ConfluenceSearchUserArgs>,
    ) -> Result<CallToolResult, ErrorData> {
        let query = required_non_empty_arg(args.query, "query")?;
        let limit = confluence_user_search_limit(args.limit)?;
        let group_name = optional_non_empty_arg(args.group_name)
            .unwrap_or_else(|| "confluence-users".to_string());
        let cql = normalize_confluence_user_search_query(&query);

        match self
            .confluence_client()?
            .search_user(&cql, Some(limit), Some(&group_name))
            .await
        {
            Ok(response) => {
                let results = response.to_simplified_value()["results"].clone();
                let cql_query = response.cql_query.clone().unwrap_or_else(|| cql.clone());
                Ok(CallToolResult::structured(json!({
                    "group_name": group_name,
                    "count": response.results.len(),
                    "results": results,
                    "start": response.start,
                    "limit": response.limit,
                    "size": response.size,
                    "total_size": response.total_size,
                    "cql_query": cql_query,
                    "search_duration": response.search_duration,
                    "links": response.links,
                })))
            }
            Err(AtlassianError::HttpStatus {
                status, message, ..
            }) if matches!(status, 401 | 403) => Ok(CallToolResult::structured_error(json!({
                "success": false,
                "error": "Authentication failed. Please check your credentials.",
                "status": status,
                "details": message,
            }))),
            Err(error) => Err(atlassian_error(error)),
        }
    }

    #[tool(name = "confluence_get_page_version")]
    pub(super) async fn get_page_version(
        &self,
        Parameters(args): Parameters<confluence_tools::ConfluenceGetPageVersionArgs>,
    ) -> Result<CallToolResult, ErrorData> {
        let page_id = required_non_empty_arg(args.page_id, "page_id")?;
        let version = confluence_positive_version_arg(args.version, "version")?;
        let convert_to_markdown = args.convert_to_markdown.unwrap_or(true);
        let page = self
            .confluence_client()?
            .get_page_history(&page_id, version, CONFLUENCE_PAGE_EXPAND)
            .await
            .map_err(atlassian_error)?;

        Ok(CallToolResult::structured(
            page.to_simplified_value(convert_to_markdown),
        ))
    }

    #[tool(name = "confluence_get_page_diff")]
    pub(super) async fn get_page_diff(
        &self,
        Parameters(args): Parameters<confluence_tools::ConfluenceGetPageDiffArgs>,
    ) -> Result<CallToolResult, ErrorData> {
        let page_id = required_non_empty_arg(args.page_id, "page_id")?;
        let from_version = confluence_positive_version_arg(args.from_version, "from_version")?;
        let to_version = confluence_positive_version_arg(args.to_version, "to_version")?;
        if from_version > to_version {
            return Err(atlassian_error(AtlassianError::invalid_input(
                "from_version must be less than or equal to to_version",
            )));
        }
        let client = self.confluence_client()?;
        let from_page = client
            .get_page_history(&page_id, from_version, CONFLUENCE_PAGE_EXPAND)
            .await
            .map_err(atlassian_error)?;
        let to_page = if from_version == to_version {
            from_page.clone()
        } else {
            client
                .get_page_history(&page_id, to_version, CONFLUENCE_PAGE_EXPAND)
                .await
                .map_err(atlassian_error)?
        };
        let from_content = confluence_page_markdown_content(&from_page);
        let to_content = confluence_page_markdown_content(&to_page);
        let diff = confluence_unified_diff(&from_content, &to_content, from_version, to_version);

        Ok(CallToolResult::structured(json!({
            "page_id": page_id,
            "title": to_page.title,
            "from_version": from_version,
            "to_version": to_version,
            "diff": diff,
            "has_changes": from_content != to_content,
        })))
    }

    #[tool(name = "confluence_get_page_view_analytics")]
    pub(super) async fn get_page_view_analytics(
        &self,
        Parameters(args): Parameters<confluence_tools::ConfluenceGetPageViewAnalyticsArgs>,
    ) -> Result<CallToolResult, ErrorData> {
        let page_id = required_non_empty_arg(args.page_id, "page_id")?;
        let include_title = args.include_title.unwrap_or(true);
        let client = self.confluence_client()?;
        if client.config().deployment != ConfluenceDeployment::Cloud {
            return Ok(CallToolResult::structured(json!({
                "success": false,
                "available": false,
                "page_id": page_id,
                "error": "Page view analytics is only available for Confluence Cloud. Server/Data Center instances do not support the Analytics API.",
            })));
        }

        match client.get_page_views(&page_id, include_title).await {
            Ok(views) => Ok(CallToolResult::structured(crate::mcp::wrap_array(
                views.to_simplified_value(),
            ))),
            Err(AtlassianError::HttpStatus {
                status, message, ..
            }) if matches!(status, 401 | 403) => Ok(CallToolResult::structured_error(json!({
                "success": false,
                "error": "Authentication failed. Please check your credentials.",
                "status": status,
                "details": message,
            }))),
            Err(error) => Err(atlassian_error(error)),
        }
    }

    #[tool(name = "confluence_upload_content_attachment")]
    pub(super) async fn upload_content_attachment(
        &self,
        Parameters(args): Parameters<confluence_tools::ConfluenceUploadContentAttachmentArgs>,
    ) -> Result<CallToolResult, ErrorData> {
        let content_id = required_non_empty_arg(args.content_id, "content_id")?;
        let file_path = required_non_empty_arg(args.file_path, "file_path")?;
        let filename = confluence_file_path_display(&file_path);
        let comment = optional_non_empty_arg(args.comment);
        let minor_edit = args.minor_edit.unwrap_or(false);
        let client = self.confluence_client()?;

        match client
            .upload_attachment(&content_id, &file_path, comment.as_deref(), minor_edit)
            .await
        {
            Ok(attachment) => Ok(CallToolResult::structured(json!({
                "success": true,
                "content_id": content_id,
                "filename": filename,
                "minor_edit": minor_edit,
                "attachment": attachment.to_simplified_value(),
            }))),
            Err(error) => Ok(CallToolResult::structured_error(json!({
                "success": false,
                "content_id": content_id,
                "filename": filename,
                "minor_edit": minor_edit,
                "error": error.to_string(),
            }))),
        }
    }

    #[tool(name = "confluence_upload_content_attachments")]
    pub(super) async fn upload_content_attachments(
        &self,
        Parameters(args): Parameters<confluence_tools::ConfluenceUploadContentAttachmentsArgs>,
    ) -> Result<CallToolResult, ErrorData> {
        let content_id = required_non_empty_arg(args.content_id, "content_id")?;
        let file_paths = confluence_split_file_paths(&args.file_paths)?;
        let comment = optional_non_empty_arg(args.comment);
        let minor_edit = args.minor_edit.unwrap_or(false);
        let client = self.confluence_client()?;
        let mut uploaded = Vec::new();
        let mut failed = Vec::new();

        for (index, file_path) in file_paths.iter().enumerate() {
            let filename = confluence_file_path_display(file_path);
            match client
                .upload_attachment(&content_id, file_path, comment.as_deref(), minor_edit)
                .await
            {
                Ok(attachment) => uploaded.push(json!({
                    "index": index,
                    "filename": filename,
                    "attachment": attachment.to_simplified_value(),
                })),
                Err(error) => failed.push(json!({
                    "index": index,
                    "filename": filename,
                    "error": error.to_string(),
                })),
            }
        }

        Ok(CallToolResult::structured(json!({
            "success": failed.is_empty(),
            "partial_success": !uploaded.is_empty() && !failed.is_empty(),
            "content_id": content_id,
            "minor_edit": minor_edit,
            "summary": {
                "total": uploaded.len() + failed.len(),
                "uploaded": uploaded.len(),
                "failed": failed.len(),
            },
            "attachments": uploaded,
            "failed": failed,
        })))
    }

    #[tool(name = "confluence_list_content_attachments")]
    pub(super) async fn list_content_attachments(
        &self,
        Parameters(args): Parameters<confluence_tools::ConfluenceListContentAttachmentsArgs>,
    ) -> Result<CallToolResult, ErrorData> {
        let content_id = required_non_empty_arg(args.content_id, "content_id")?;
        let start = args.start.unwrap_or(0);
        let limit = optional_u64_range_arg(
            args.limit,
            crate::confluence::client::DEFAULT_ATTACHMENT_LIST_LIMIT,
            crate::confluence::client::MAX_ATTACHMENT_LIST_LIMIT,
            "limit",
        )?;
        let filename = optional_non_empty_arg(args.filename);
        let media_type = optional_non_empty_arg(args.media_type);
        let response = self
            .confluence_client()?
            .get_attachments(
                &content_id,
                Some(start),
                Some(limit),
                filename.as_deref(),
                media_type.as_deref(),
            )
            .await
            .map_err(atlassian_error)?;
        let attachments = response
            .results
            .iter()
            .map(|attachment| attachment.to_simplified_value())
            .collect::<Vec<_>>();

        Ok(CallToolResult::structured(json!({
            "success": true,
            "content_id": content_id,
            "count": attachments.len(),
            "total": response.size.unwrap_or(attachments.len() as u64),
            "start": response.start.unwrap_or(start),
            "limit": response.limit.unwrap_or(limit),
            "filters": {
                "filename": filename,
                "media_type": media_type,
            },
            "attachments": attachments,
            "links": response.links,
        })))
    }

    #[tool(name = "confluence_download_attachment")]
    pub(super) async fn download_attachment(
        &self,
        Parameters(args): Parameters<confluence_tools::ConfluenceDownloadAttachmentArgs>,
    ) -> Result<CallToolResult, ErrorData> {
        let attachment_id = required_non_empty_arg(args.attachment_id, "attachment_id")?;
        let client = self.confluence_client()?;
        let attachment = client
            .get_attachment_by_id(&attachment_id)
            .await
            .map_err(atlassian_error)?;

        match confluence_attachment_with_content_value(
            &client,
            &attachment,
            &attachment_id,
            crate::confluence::client::DEFAULT_ATTACHMENT_MAX_BYTES,
        )
        .await
        {
            Ok(attachment) => Ok(CallToolResult::structured(json!({
                "success": true,
                "attachment": attachment,
            }))),
            Err(error) => Ok(CallToolResult::structured_error(crate::mcp::wrap_array(
                error,
            ))),
        }
    }

    #[tool(name = "confluence_download_content_attachments")]
    pub(super) async fn download_content_attachments(
        &self,
        Parameters(args): Parameters<confluence_tools::ConfluenceDownloadContentAttachmentsArgs>,
    ) -> Result<CallToolResult, ErrorData> {
        let content_id = required_non_empty_arg(args.content_id, "content_id")?;
        let client = self.confluence_client()?;
        let mut attachments = Vec::new();
        let mut failed = Vec::new();
        let mut pages = Vec::new();
        let mut start = 0;
        let mut pages_fetched = 0;
        let mut total = 0;

        let (has_more, next_start, limit_applied) = loop {
            let response = client
                .get_attachments(
                    &content_id,
                    Some(start),
                    Some(crate::confluence::client::MAX_ATTACHMENT_LIST_LIMIT),
                    None,
                    None,
                )
                .await
                .map_err(atlassian_error)?;
            pages_fetched += 1;
            total += response.results.len();
            let response_has_more = response.has_next_link();
            let response_next_start = response.next_start();
            pages.push(json!({
                "start": response.start,
                "limit": response.limit,
                "size": response.size,
                "count": response.results.len(),
                "has_more": response_has_more,
                "next_start": response_next_start,
            }));

            for attachment in &response.results {
                let attachment_id = confluence_attachment_id(attachment);
                match confluence_attachment_with_content_value(
                    &client,
                    attachment,
                    &attachment_id,
                    crate::confluence::client::DEFAULT_ATTACHMENT_MAX_BYTES,
                )
                .await
                {
                    Ok(value) => attachments.push(value),
                    Err(error) => failed.push(error),
                }
            }

            if response.results.is_empty() || !response_has_more {
                break (false, None, false);
            }
            if pages_fetched >= CONFLUENCE_DOWNLOAD_CONTENT_ATTACHMENTS_MAX_PAGES {
                break (response_has_more, response_next_start, response_has_more);
            }
            let Some(next) = response_next_start else {
                break (true, None, false);
            };
            if next == start {
                break (true, Some(next), true);
            }
            start = next;
        };

        Ok(CallToolResult::structured(json!({
            "success": true,
            "summary": {
                "content_id": content_id,
                "total": total,
                "downloaded": attachments.len(),
                "failed": failed.len(),
                "pages_fetched": pages_fetched,
                "page_limit": crate::confluence::client::MAX_ATTACHMENT_LIST_LIMIT,
                "max_pages": CONFLUENCE_DOWNLOAD_CONTENT_ATTACHMENTS_MAX_PAGES,
                "has_more": has_more,
                "next_start": next_start,
                "limit_applied": limit_applied,
                "pages": pages,
            },
            "attachments": attachments,
            "failed": failed,
        })))
    }

    #[tool(name = "confluence_delete_attachment")]
    pub(super) async fn delete_attachment(
        &self,
        Parameters(args): Parameters<confluence_tools::ConfluenceDeleteAttachmentArgs>,
    ) -> Result<CallToolResult, ErrorData> {
        let attachment_id = required_non_empty_arg(args.attachment_id, "attachment_id")?;
        match self
            .confluence_client()?
            .delete_attachment(&attachment_id)
            .await
        {
            Ok(value) => Ok(CallToolResult::structured(json!({
                "success": true,
                "attachment_id": attachment_id,
                "result": value,
            }))),
            Err(error) => Ok(CallToolResult::structured_error(json!({
                "success": false,
                "attachment_id": attachment_id,
                "error": error.to_string(),
            }))),
        }
    }

    #[tool(name = "confluence_get_content_image_attachments")]
    pub(super) async fn get_content_image_attachments(
        &self,
        Parameters(args): Parameters<confluence_tools::ConfluenceGetContentImageAttachmentsArgs>,
    ) -> Result<CallToolResult, ErrorData> {
        let content_id = required_non_empty_arg(args.content_id, "content_id")?;
        let client = self.confluence_client()?;
        let response = client
            .get_attachments(
                &content_id,
                Some(0),
                Some(crate::confluence::client::MAX_ATTACHMENT_LIST_LIMIT),
                None,
                None,
            )
            .await
            .map_err(atlassian_error)?;
        let mut images = Vec::new();
        let mut failed = Vec::new();
        let mut skipped_non_images = 0usize;

        for attachment in &response.results {
            let filename =
                confluence_attachment_filename(attachment, &confluence_attachment_id(attachment));
            let (is_image, resolved_mime_type) =
                confluence_is_image_attachment(attachment.media_type(), &filename);
            if !is_image {
                skipped_non_images += 1;
                continue;
            }

            let attachment_id = confluence_attachment_id(attachment);
            match confluence_attachment_with_content_value(
                &client,
                attachment,
                &attachment_id,
                crate::confluence::client::DEFAULT_ATTACHMENT_MAX_BYTES,
            )
            .await
            {
                Ok(mut value) => {
                    value["is_image"] = Value::Bool(true);
                    value["resolved_mime_type"] = Value::String(resolved_mime_type);
                    images.push(value);
                }
                Err(error) => failed.push(error),
            }
        }

        Ok(CallToolResult::structured(json!({
            "success": true,
            "content_id": content_id,
            "images_only": true,
            "count": images.len(),
            "skipped_non_images": skipped_non_images,
            "images": images,
            "failed": failed,
        })))
    }
}

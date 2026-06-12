use serde_json::{Value, json};

use crate::{
    confluence::{
        client::{
            ConfluenceEmojiStatus, DEFAULT_ATTACHMENT_LIST_LIMIT, DEFAULT_ATTACHMENT_MAX_BYTES,
            MAX_ATTACHMENT_LIST_LIMIT, MAX_SEARCH_LIMIT,
        },
        config::ConfluenceDeployment,
        formatting::{ConfluenceContentFormat, content_to_storage},
        models::ConfluencePage,
        tools::*,
    },
    context::AppContext,
    mcp_confluence_helpers::{
        confluence_attachment_filename, confluence_attachment_id,
        confluence_attachment_with_content_value, confluence_file_path_display,
        confluence_is_image_attachment,
    },
    upstream::error::UpstreamError,
};

use super::{OperationError, OperationResult, confluence_client, guard_operation};

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
const CONFLUENCE_DOWNLOAD_CONTENT_ATTACHMENTS_MAX_PAGES: u64 = 10;

pub async fn search_content(
    context: &AppContext,
    args: ConfluenceSearchArgs,
) -> Result<OperationResult, OperationError> {
    guard_operation(CONFLUENCE_SEARCH_TOOL_NAME, context)?;
    let query = required_non_empty_arg(args.query, "query")?;
    let limit = optional_confluence_search_limit(args.limit)?;
    let value = confluence_client(context)?
        .search_content(&query, limit, args.spaces_filter.as_deref())
        .await
        .map_err(OperationError::from_upstream)?
        .to_simplified_value();

    Ok(structured(wrap_array(value)))
}

pub async fn get_page(
    context: &AppContext,
    args: ConfluenceGetPageArgs,
) -> Result<OperationResult, OperationError> {
    guard_operation(CONFLUENCE_GET_PAGE_TOOL_NAME, context)?;
    let include_metadata = args.include_metadata.unwrap_or(true);
    let convert_to_markdown = args.convert_to_markdown.unwrap_or(true);
    let client = confluence_client(context)?;

    if let Some(page_id) = optional_non_empty_arg(args.page_id) {
        let page = match client
            .get_page_by_id(&page_id, CONFLUENCE_PAGE_EXPAND)
            .await
        {
            Ok(page) => page,
            Err(UpstreamError::HttpStatus { status: 404, .. }) => {
                return Ok(OperationResult::structured_error(json!({
                    "error": format!("Failed to retrieve page by ID '{page_id}': page not found")
                })));
            }
            Err(error) => return Err(OperationError::from_upstream(error)),
        };

        return Ok(structured(confluence_page_tool_value(
            &page,
            include_metadata,
            convert_to_markdown,
        )));
    }

    let title = optional_non_empty_arg(args.title);
    let space_key = optional_non_empty_arg(args.space_key);
    let (Some(title), Some(space_key)) = (title, space_key) else {
        return Err(OperationError::invalid_input(
            "Either page_id OR both title and space_key must be provided",
        ));
    };

    let Some(page) = client
        .get_page_by_title(&space_key, &title, CONFLUENCE_PAGE_EXPAND)
        .await
        .map_err(OperationError::from_upstream)?
    else {
        return Ok(OperationResult::structured_error(json!({
            "error": format!("Page with title '{title}' not found in space '{space_key}'.")
        })));
    };

    Ok(structured(confluence_page_tool_value(
        &page,
        include_metadata,
        convert_to_markdown,
    )))
}

pub async fn list_page_children(
    context: &AppContext,
    args: ConfluenceListPageChildrenArgs,
) -> Result<OperationResult, OperationError> {
    guard_operation(CONFLUENCE_LIST_PAGE_CHILDREN_TOOL_NAME, context)?;
    let parent_id = required_non_empty_arg(args.parent_id, "parent_id")?;
    let limit = optional_u64_range(
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
    let children = confluence_client(context)?
        .get_page_children(
            &parent_id,
            Some(start),
            Some(limit),
            &expand_refs,
            include_folders,
        )
        .await
        .map_err(OperationError::from_upstream)?;
    let results = children
        .results
        .iter()
        .map(|page| confluence_child_page_value(page, include_content, convert_to_markdown))
        .collect::<Vec<_>>();

    Ok(structured(json!({
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

pub async fn get_space_page_tree(
    context: &AppContext,
    args: ConfluenceGetSpacePageTreeArgs,
) -> Result<OperationResult, OperationError> {
    guard_operation(CONFLUENCE_GET_SPACE_PAGE_TREE_TOOL_NAME, context)?;
    let space_key = required_non_empty_arg(args.space_key, "space_key")?;
    let limit = optional_u64_range(
        args.limit,
        CONFLUENCE_TREE_DEFAULT_LIMIT,
        CONFLUENCE_TREE_MAX_LIMIT,
        "limit",
    )?;
    let client = confluence_client(context)?;
    let mut pages = Vec::new();
    let mut start = 0;
    let mut next_link_exists = false;

    while pages.len() < limit as usize {
        let fetch_limit = CONFLUENCE_TREE_PAGE_SIZE.min(limit - pages.len() as u64);
        let response = client
            .get_space_pages(&space_key, Some(start), Some(fetch_limit), &["ancestors"])
            .await
            .map_err(OperationError::from_upstream)?;
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

    Ok(structured(wrap_array(result)))
}

pub async fn create_page(
    context: &AppContext,
    args: ConfluenceCreatePageArgs,
) -> Result<OperationResult, OperationError> {
    guard_operation(CONFLUENCE_CREATE_PAGE_TOOL_NAME, context)?;
    let space_key = required_non_empty_arg(args.space_key, "space_key")?;
    let title = required_non_empty_arg(args.title, "title")?;
    let content = required_non_empty_arg(args.content, "content")?;
    let format = parse_confluence_write_content_format(args.content_format.as_deref())?;
    let storage_body = content_to_storage(&content, format);
    let parent_id = optional_non_empty_arg(args.parent_id);
    let client = confluence_client(context)?;
    let page = client
        .create_page(&space_key, &title, &storage_body, parent_id.as_deref())
        .await
        .map_err(OperationError::from_upstream)?;
    let emoji_status = match page.id.as_deref() {
        Some(page_id) => {
            client
                .set_page_emoji_best_effort(page_id, args.emoji.as_deref())
                .await
        }
        None => confluence_emoji_missing_page_id_status(args.emoji.as_deref()),
    };

    Ok(structured(json!({
        "message": "Page created successfully",
        "page": confluence_write_page_value(&page, args.include_content.unwrap_or(false)),
        "emoji_status": emoji_status,
    })))
}

pub async fn update_page(
    context: &AppContext,
    args: ConfluenceUpdatePageArgs,
) -> Result<OperationResult, OperationError> {
    guard_operation(CONFLUENCE_UPDATE_PAGE_TOOL_NAME, context)?;
    let page_id = required_non_empty_arg(args.page_id, "page_id")?;
    let title = required_non_empty_arg(args.title, "title")?;
    let content = required_non_empty_arg(args.content, "content")?;
    let format = parse_confluence_write_content_format(args.content_format.as_deref())?;
    let storage_body = content_to_storage(&content, format);
    let parent_id = optional_non_empty_arg(args.parent_id);
    let client = confluence_client(context)?;
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
        .map_err(OperationError::from_upstream)?;
    let emoji_status = match page.id.as_deref() {
        Some(page_id) => {
            client
                .set_page_emoji_best_effort(page_id, args.emoji.as_deref())
                .await
        }
        None => confluence_emoji_missing_page_id_status(args.emoji.as_deref()),
    };

    Ok(structured(json!({
        "message": "Page updated successfully",
        "page": confluence_write_page_value(&page, args.include_content.unwrap_or(false)),
        "emoji_status": emoji_status,
    })))
}

pub async fn delete_page(
    context: &AppContext,
    args: ConfluenceDeletePageArgs,
) -> Result<OperationResult, OperationError> {
    guard_operation(CONFLUENCE_DELETE_PAGE_TOOL_NAME, context)?;
    let page_id = required_non_empty_arg(args.page_id, "page_id")?;
    match confluence_client(context)?.delete_page(&page_id).await {
        Ok(_) => Ok(structured(json!({
            "success": true,
            "message": format!("Page {page_id} deleted successfully"),
        }))),
        Err(error) => Ok(OperationResult::structured_error(json!({
            "success": false,
            "message": format!("Error deleting page {page_id}"),
            "error": error.to_string(),
        }))),
    }
}

pub async fn move_page(
    context: &AppContext,
    args: ConfluenceMovePageArgs,
) -> Result<OperationResult, OperationError> {
    guard_operation(CONFLUENCE_MOVE_PAGE_TOOL_NAME, context)?;
    let page_id = required_non_empty_arg(args.page_id, "page_id")?;
    let target_parent_id = optional_non_empty_arg(args.target_parent_id);
    let page = confluence_client(context)?
        .move_page(
            &page_id,
            target_parent_id.as_deref(),
            args.target_space_key.as_deref(),
            args.position.as_deref(),
        )
        .await
        .map_err(OperationError::from_upstream)?;

    Ok(structured(json!({
        "message": "Page moved successfully",
        "page": confluence_write_page_value(&page, true),
    })))
}

pub async fn list_page_comments(
    context: &AppContext,
    args: ConfluenceListPageCommentsArgs,
) -> Result<OperationResult, OperationError> {
    guard_operation(CONFLUENCE_LIST_PAGE_COMMENTS_TOOL_NAME, context)?;
    let page_id = required_non_empty_arg(args.page_id, "page_id")?;
    let comments = confluence_client(context)?
        .get_page_comments(&page_id)
        .await
        .map_err(OperationError::from_upstream)?;
    let values = comments
        .results
        .iter()
        .map(|comment| comment.to_simplified_value())
        .collect::<Vec<_>>();

    Ok(structured(json!({
        "page_id": page_id,
        "count": values.len(),
        "comments": values,
        "start": comments.start,
        "limit": comments.limit,
        "size": comments.size,
        "links": comments.links,
    })))
}

pub async fn add_page_comment(
    context: &AppContext,
    args: ConfluenceAddCommentArgs,
) -> Result<OperationResult, OperationError> {
    guard_operation(CONFLUENCE_ADD_COMMENT_TOOL_NAME, context)?;
    let page_id = required_non_empty_arg(args.page_id, "page_id")?;
    let body = required_non_empty_arg(args.body, "body")?;
    let storage_body = content_to_storage(&body, ConfluenceContentFormat::Markdown);

    match confluence_client(context)?
        .add_comment(&page_id, &storage_body)
        .await
    {
        Ok(comment) => Ok(structured(json!({
            "success": true,
            "message": "Comment added successfully",
            "comment": comment.to_simplified_value(),
        }))),
        Err(error) => Ok(OperationResult::structured_error(json!({
            "success": false,
            "message": format!("Error adding comment to page {page_id}"),
            "error": error.to_string(),
        }))),
    }
}

pub async fn reply_to_comment(
    context: &AppContext,
    args: ConfluenceReplyToCommentArgs,
) -> Result<OperationResult, OperationError> {
    guard_operation(CONFLUENCE_REPLY_TO_COMMENT_TOOL_NAME, context)?;
    let comment_id = required_non_empty_arg(args.comment_id, "comment_id")?;
    let body = required_non_empty_arg(args.body, "body")?;
    let storage_body = content_to_storage(&body, ConfluenceContentFormat::Markdown);

    match confluence_client(context)?
        .reply_to_comment(&comment_id, &storage_body)
        .await
    {
        Ok(comment) => Ok(structured(json!({
            "success": true,
            "message": "Reply added successfully",
            "comment": comment.to_simplified_value(),
        }))),
        Err(error) => Ok(OperationResult::structured_error(json!({
            "success": false,
            "message": format!("Error replying to comment {comment_id}"),
            "error": error.to_string(),
        }))),
    }
}

pub async fn list_content_labels(
    context: &AppContext,
    args: ConfluenceListContentLabelsArgs,
) -> Result<OperationResult, OperationError> {
    guard_operation(CONFLUENCE_LIST_CONTENT_LABELS_TOOL_NAME, context)?;
    let content_id = required_non_empty_arg(args.page_id, "page_id")?;
    let labels = confluence_client(context)?
        .get_labels(&content_id)
        .await
        .map_err(OperationError::from_upstream)?;
    let values = labels
        .results
        .iter()
        .map(|label| label.to_simplified_value())
        .collect::<Vec<_>>();

    Ok(structured(json!({
        "content_id": content_id,
        "count": values.len(),
        "labels": values,
        "start": labels.start,
        "limit": labels.limit,
        "size": labels.size,
        "links": labels.links,
    })))
}

pub async fn add_content_label(
    context: &AppContext,
    args: ConfluenceAddLabelArgs,
) -> Result<OperationResult, OperationError> {
    guard_operation(CONFLUENCE_ADD_LABEL_TOOL_NAME, context)?;
    let content_id = required_non_empty_arg(args.page_id, "page_id")?;
    let name = required_non_empty_arg(args.name, "name")?;
    let labels = confluence_client(context)?
        .add_label(&content_id, &name)
        .await
        .map_err(OperationError::from_upstream)?;
    let values = labels
        .results
        .iter()
        .map(|label| label.to_simplified_value())
        .collect::<Vec<_>>();

    Ok(structured(json!({
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

pub async fn search_users(
    context: &AppContext,
    args: ConfluenceSearchUserArgs,
) -> Result<OperationResult, OperationError> {
    guard_operation(CONFLUENCE_SEARCH_USER_TOOL_NAME, context)?;
    let query = required_non_empty_arg(args.query, "query")?;
    let limit = confluence_user_search_limit(args.limit)?;
    let group_name =
        optional_non_empty_arg(args.group_name).unwrap_or_else(|| "confluence-users".to_string());
    let cql = normalize_confluence_user_search_query(&query);

    match confluence_client(context)?
        .search_user(&cql, Some(limit), Some(&group_name))
        .await
    {
        Ok(response) => {
            let results = response.to_simplified_value()["results"].clone();
            let cql_query = response.cql_query.clone().unwrap_or_else(|| cql.clone());
            Ok(structured(json!({
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
        Err(UpstreamError::HttpStatus {
            status, message, ..
        }) if matches!(status, 401 | 403) => Ok(OperationResult::structured_error(json!({
            "success": false,
            "error": "Authentication failed. Please check your credentials.",
            "status": status,
            "details": message,
        }))),
        Err(error) => Err(OperationError::from_upstream(error)),
    }
}

pub async fn get_page_version(
    context: &AppContext,
    args: ConfluenceGetPageVersionArgs,
) -> Result<OperationResult, OperationError> {
    guard_operation(CONFLUENCE_GET_PAGE_VERSION_TOOL_NAME, context)?;
    let page_id = required_non_empty_arg(args.page_id, "page_id")?;
    let version = confluence_positive_version(args.version, "version")?;
    let convert_to_markdown = args.convert_to_markdown.unwrap_or(true);
    let page = confluence_client(context)?
        .get_page_history(&page_id, version, CONFLUENCE_PAGE_EXPAND)
        .await
        .map_err(OperationError::from_upstream)?;

    Ok(structured(page.to_simplified_value(convert_to_markdown)))
}

pub async fn get_page_diff(
    context: &AppContext,
    args: ConfluenceGetPageDiffArgs,
) -> Result<OperationResult, OperationError> {
    guard_operation(CONFLUENCE_GET_PAGE_DIFF_TOOL_NAME, context)?;
    let page_id = required_non_empty_arg(args.page_id, "page_id")?;
    let from_version = confluence_positive_version(args.from_version, "from_version")?;
    let to_version = confluence_positive_version(args.to_version, "to_version")?;
    if from_version > to_version {
        return Err(OperationError::invalid_input(
            "from_version must be less than or equal to to_version",
        ));
    }
    let client = confluence_client(context)?;
    let from_page = client
        .get_page_history(&page_id, from_version, CONFLUENCE_PAGE_EXPAND)
        .await
        .map_err(OperationError::from_upstream)?;
    let to_page = if from_version == to_version {
        from_page.clone()
    } else {
        client
            .get_page_history(&page_id, to_version, CONFLUENCE_PAGE_EXPAND)
            .await
            .map_err(OperationError::from_upstream)?
    };
    let from_content = confluence_page_markdown_content(&from_page);
    let to_content = confluence_page_markdown_content(&to_page);
    let diff = confluence_unified_diff(
        &from_content,
        &to_content,
        from_version,
        to_version,
        args.context_lines,
    );

    Ok(structured(json!({
        "page_id": page_id,
        "title": to_page.title,
        "from_version": from_version,
        "to_version": to_version,
        "context_lines": args.context_lines,
        "diff": diff,
        "has_changes": from_content != to_content,
    })))
}

pub async fn get_page_view_analytics(
    context: &AppContext,
    args: ConfluenceGetPageViewAnalyticsArgs,
) -> Result<OperationResult, OperationError> {
    guard_operation(CONFLUENCE_GET_PAGE_VIEW_ANALYTICS_TOOL_NAME, context)?;
    let page_id = required_non_empty_arg(args.page_id, "page_id")?;
    let include_title = args.include_title.unwrap_or(true);
    let client = confluence_client(context)?;
    if client.config().deployment != ConfluenceDeployment::Cloud {
        return Ok(structured(json!({
            "success": false,
            "available": false,
            "page_id": page_id,
            "error": "Page view analytics is only available for Confluence Cloud. Server/Data Center instances do not support the Analytics API.",
        })));
    }

    match client
        .get_page_views(
            &page_id,
            include_title,
            args.from_date.as_deref(),
            args.to_date.as_deref(),
        )
        .await
    {
        Ok(views) => Ok(structured(wrap_array(views.to_simplified_value()))),
        Err(UpstreamError::HttpStatus {
            status, message, ..
        }) if matches!(status, 401 | 403) => Ok(OperationResult::structured_error(json!({
            "success": false,
            "error": "Authentication failed. Please check your credentials.",
            "status": status,
            "details": message,
        }))),
        Err(error) => Err(OperationError::from_upstream(error)),
    }
}

pub async fn upload_content_attachment(
    context: &AppContext,
    args: ConfluenceUploadContentAttachmentArgs,
) -> Result<OperationResult, OperationError> {
    guard_operation(CONFLUENCE_UPLOAD_CONTENT_ATTACHMENT_TOOL_NAME, context)?;
    let content_id = required_non_empty_arg(args.content_id, "content_id")?;
    let file_path = required_non_empty_arg(args.file_path, "file_path")?;
    let filename = confluence_file_path_display(&file_path);
    let comment = optional_non_empty_arg(args.comment);
    let minor_edit = args.minor_edit.unwrap_or(false);

    match confluence_client(context)?
        .upload_attachment(&content_id, &file_path, comment.as_deref(), minor_edit)
        .await
    {
        Ok(attachment) => Ok(structured(json!({
            "success": true,
            "content_id": content_id,
            "filename": filename,
            "minor_edit": minor_edit,
            "attachment": attachment.to_simplified_value(),
        }))),
        Err(error) => Ok(OperationResult::structured_error(json!({
            "success": false,
            "content_id": content_id,
            "filename": filename,
            "minor_edit": minor_edit,
            "error": error.to_string(),
        }))),
    }
}

pub async fn upload_content_attachments(
    context: &AppContext,
    args: ConfluenceUploadContentAttachmentsArgs,
) -> Result<OperationResult, OperationError> {
    guard_operation(CONFLUENCE_UPLOAD_CONTENT_ATTACHMENTS_TOOL_NAME, context)?;
    let content_id = required_non_empty_arg(args.content_id, "content_id")?;
    let file_paths = confluence_split_file_paths(&args.file_paths)?;
    let comment = optional_non_empty_arg(args.comment);
    let minor_edit = args.minor_edit.unwrap_or(false);
    let client = confluence_client(context)?;
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

    Ok(structured(json!({
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

pub async fn list_content_attachments(
    context: &AppContext,
    args: ConfluenceListContentAttachmentsArgs,
) -> Result<OperationResult, OperationError> {
    guard_operation(CONFLUENCE_LIST_CONTENT_ATTACHMENTS_TOOL_NAME, context)?;
    let content_id = required_non_empty_arg(args.content_id, "content_id")?;
    let start = args.start.unwrap_or(0);
    let limit = optional_u64_range(
        args.limit,
        DEFAULT_ATTACHMENT_LIST_LIMIT,
        MAX_ATTACHMENT_LIST_LIMIT,
        "limit",
    )?;
    let filename = optional_non_empty_arg(args.filename);
    let media_type = optional_non_empty_arg(args.media_type);
    let response = confluence_client(context)?
        .get_attachments(
            &content_id,
            Some(start),
            Some(limit),
            filename.as_deref(),
            media_type.as_deref(),
        )
        .await
        .map_err(OperationError::from_upstream)?;
    let attachments = response
        .results
        .iter()
        .map(|attachment| attachment.to_simplified_value())
        .collect::<Vec<_>>();

    Ok(structured(json!({
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

pub async fn download_attachment(
    context: &AppContext,
    args: ConfluenceDownloadAttachmentArgs,
) -> Result<OperationResult, OperationError> {
    guard_operation(CONFLUENCE_DOWNLOAD_ATTACHMENT_TOOL_NAME, context)?;
    let attachment_id = required_non_empty_arg(args.attachment_id, "attachment_id")?;
    let max_bytes =
        optional_positive_u64(args.max_bytes, "max_bytes")?.unwrap_or(DEFAULT_ATTACHMENT_MAX_BYTES);
    let client = confluence_client(context)?;
    let attachment = client
        .get_attachment_by_id(&attachment_id)
        .await
        .map_err(OperationError::from_upstream)?;

    match confluence_attachment_with_content_value(&client, &attachment, &attachment_id, max_bytes)
        .await
    {
        Ok(attachment) => Ok(structured(json!({
            "success": true,
            "attachment": attachment,
        }))),
        Err(error) => Ok(OperationResult::structured_error(wrap_array(error))),
    }
}

pub async fn download_content_attachments(
    context: &AppContext,
    args: ConfluenceDownloadContentAttachmentsArgs,
) -> Result<OperationResult, OperationError> {
    guard_operation(CONFLUENCE_DOWNLOAD_CONTENT_ATTACHMENTS_TOOL_NAME, context)?;
    let content_id = required_non_empty_arg(args.content_id, "content_id")?;
    let filename = optional_non_empty_arg(args.filename);
    let media_type = optional_non_empty_arg(args.media_type);
    let max_bytes =
        optional_positive_u64(args.max_bytes, "max_bytes")?.unwrap_or(DEFAULT_ATTACHMENT_MAX_BYTES);
    let requested_limit = optional_positive_u64(args.limit, "limit")?;
    let client = confluence_client(context)?;
    let mut attachments = Vec::new();
    let mut failed = Vec::new();
    let mut pages = Vec::new();
    let mut start = 0;
    let mut pages_fetched = 0;
    let mut total = 0;

    let (has_more, next_start, limit_applied) = loop {
        let remaining = requested_limit
            .map(|limit| limit.saturating_sub((attachments.len() + failed.len()) as u64));
        if remaining == Some(0) {
            break (true, Some(start), true);
        }
        let fetch_limit = remaining
            .map(|remaining| remaining.min(MAX_ATTACHMENT_LIST_LIMIT))
            .unwrap_or(MAX_ATTACHMENT_LIST_LIMIT);
        let response = client
            .get_attachments(
                &content_id,
                Some(start),
                Some(fetch_limit),
                filename.as_deref(),
                media_type.as_deref(),
            )
            .await
            .map_err(OperationError::from_upstream)?;
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
            if requested_limit
                .is_some_and(|limit| attachments.len() + failed.len() >= limit as usize)
            {
                break;
            }
            let attachment_id = confluence_attachment_id(attachment);
            match confluence_attachment_with_content_value(
                &client,
                attachment,
                &attachment_id,
                max_bytes,
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
        if requested_limit.is_some_and(|limit| attachments.len() + failed.len() >= limit as usize) {
            break (response_has_more, response_next_start, true);
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

    Ok(structured(json!({
        "success": true,
        "summary": {
            "content_id": content_id,
            "total": total,
            "downloaded": attachments.len(),
            "failed": failed.len(),
            "pages_fetched": pages_fetched,
            "page_limit": MAX_ATTACHMENT_LIST_LIMIT,
            "max_pages": CONFLUENCE_DOWNLOAD_CONTENT_ATTACHMENTS_MAX_PAGES,
            "has_more": has_more,
            "next_start": next_start,
            "limit_applied": limit_applied,
            "pages": pages,
            "filters": {
                "filename": filename,
                "media_type": media_type,
            },
            "max_bytes": max_bytes,
        },
        "attachments": attachments,
        "failed": failed,
    })))
}

pub async fn delete_attachment(
    context: &AppContext,
    args: ConfluenceDeleteAttachmentArgs,
) -> Result<OperationResult, OperationError> {
    guard_operation(CONFLUENCE_DELETE_ATTACHMENT_TOOL_NAME, context)?;
    let attachment_id = required_non_empty_arg(args.attachment_id, "attachment_id")?;
    match confluence_client(context)?
        .delete_attachment(&attachment_id)
        .await
    {
        Ok(value) => Ok(structured(json!({
            "success": true,
            "attachment_id": attachment_id,
            "result": value,
        }))),
        Err(error) => Ok(OperationResult::structured_error(json!({
            "success": false,
            "attachment_id": attachment_id,
            "error": error.to_string(),
        }))),
    }
}

pub async fn get_content_image_attachments(
    context: &AppContext,
    args: ConfluenceGetContentImageAttachmentsArgs,
) -> Result<OperationResult, OperationError> {
    guard_operation(CONFLUENCE_GET_CONTENT_IMAGE_ATTACHMENTS_TOOL_NAME, context)?;
    let content_id = required_non_empty_arg(args.content_id, "content_id")?;
    let max_bytes =
        optional_positive_u64(args.max_bytes, "max_bytes")?.unwrap_or(DEFAULT_ATTACHMENT_MAX_BYTES);
    let client = confluence_client(context)?;
    let response = client
        .get_attachments(
            &content_id,
            Some(0),
            Some(MAX_ATTACHMENT_LIST_LIMIT),
            None,
            None,
        )
        .await
        .map_err(OperationError::from_upstream)?;
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
            max_bytes,
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

    Ok(structured(json!({
        "success": true,
        "content_id": content_id,
        "images_only": true,
        "count": images.len(),
        "skipped_non_images": skipped_non_images,
        "images": images,
        "failed": failed,
    })))
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

fn required_non_empty_arg(
    value: String,
    field_name: &'static str,
) -> Result<String, OperationError> {
    let value = value.trim();
    if value.is_empty() {
        Err(OperationError::invalid_input(format!(
            "{field_name} must not be empty"
        )))
    } else {
        Ok(value.to_string())
    }
}

fn optional_non_empty_arg(value: Option<String>) -> Option<String> {
    value
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
}

fn optional_confluence_search_limit(value: Option<u64>) -> Result<Option<u64>, OperationError> {
    match value {
        Some(0) => Err(OperationError::invalid_input("limit must be positive")),
        Some(value) if value > MAX_SEARCH_LIMIT => Err(OperationError::invalid_input(format!(
            "limit must be less than or equal to {}",
            MAX_SEARCH_LIMIT
        ))),
        value => Ok(value),
    }
}

fn optional_u64_range(
    value: Option<u64>,
    default: u64,
    max: u64,
    field_name: &'static str,
) -> Result<u64, OperationError> {
    match value.unwrap_or(default) {
        0 => Err(OperationError::invalid_input(format!(
            "{field_name} must be positive"
        ))),
        value if value > max => Err(OperationError::invalid_input(format!(
            "{field_name} must be less than or equal to {max}"
        ))),
        value => Ok(value),
    }
}

fn optional_positive_u64(
    value: Option<u64>,
    field_name: &'static str,
) -> Result<Option<u64>, OperationError> {
    match value {
        Some(0) => Err(OperationError::invalid_input(format!(
            "{field_name} must be positive"
        ))),
        value => Ok(value),
    }
}

fn confluence_split_file_paths(value: &str) -> Result<Vec<String>, OperationError> {
    let file_paths = value
        .split(',')
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToString::to_string)
        .collect::<Vec<_>>();
    if file_paths.is_empty() {
        Err(OperationError::invalid_input(
            "file_paths must contain at least one local file path",
        ))
    } else {
        Ok(file_paths)
    }
}

fn parse_confluence_write_content_format(
    value: Option<&str>,
) -> Result<ConfluenceContentFormat, OperationError> {
    let format = ConfluenceContentFormat::parse(value).map_err(OperationError::from_upstream)?;
    if format == ConfluenceContentFormat::Html {
        return Err(OperationError::invalid_input(
            "content_format must be markdown, wiki, or storage",
        ));
    }
    Ok(format)
}

fn confluence_user_search_limit(value: Option<u64>) -> Result<u64, OperationError> {
    match value.unwrap_or(10) {
        0 => Err(OperationError::invalid_input("limit must be positive")),
        value if value > 50 => Err(OperationError::invalid_input(
            "limit must be less than or equal to 50",
        )),
        value => Ok(value),
    }
}

fn confluence_positive_version(
    value: u64,
    field_name: &'static str,
) -> Result<u64, OperationError> {
    if value == 0 {
        Err(OperationError::invalid_input(format!(
            "{field_name} must be positive"
        )))
    } else {
        Ok(value)
    }
}

fn confluence_page_tool_value(
    page: &ConfluencePage,
    include_metadata: bool,
    convert_to_markdown: bool,
) -> Value {
    let simplified = page.to_simplified_value(convert_to_markdown);
    if include_metadata {
        json!({ "metadata": simplified })
    } else {
        json!({ "content": { "value": simplified.get("content").cloned().unwrap_or(Value::Null) } })
    }
}

fn normalize_confluence_user_search_query(query: &str) -> String {
    if ["=", "~", ">", "<", " AND ", " OR ", "user."]
        .iter()
        .any(|token| query.contains(token))
    {
        query.to_string()
    } else {
        format!(
            "user.fullname ~ \"{}\"",
            query.replace('\\', "\\\\").replace('"', "\\\"")
        )
    }
}

fn confluence_page_markdown_content(page: &ConfluencePage) -> String {
    page.to_simplified_value(true)
        .get("content")
        .and_then(Value::as_str)
        .unwrap_or_default()
        .to_string()
}

fn confluence_unified_diff(
    from_content: &str,
    to_content: &str,
    from_version: u64,
    to_version: u64,
    context_lines: Option<u64>,
) -> String {
    if from_content == to_content {
        return String::new();
    }

    let from_lines = from_content.lines().collect::<Vec<_>>();
    let to_lines = to_content.lines().collect::<Vec<_>>();
    let max_len = from_lines.len().max(to_lines.len());
    let changed_indexes = (0..max_len)
        .filter(|index| from_lines.get(*index) != to_lines.get(*index))
        .collect::<Vec<_>>();
    let context_lines = context_lines.map(|value| value as usize);

    let mut output = vec![
        format!("--- v{from_version}"),
        format!("+++ v{to_version}"),
        format!(
            "@@ -{} +{} @@",
            confluence_diff_range(from_lines.len()),
            confluence_diff_range(to_lines.len())
        ),
    ];

    for index in 0..max_len {
        if let Some(context_lines) = context_lines
            && !changed_indexes
                .iter()
                .any(|changed| index.abs_diff(*changed) <= context_lines)
        {
            continue;
        }
        match (from_lines.get(index), to_lines.get(index)) {
            (Some(left), Some(right)) if left == right => output.push(format!(" {left}")),
            (Some(left), Some(right)) => {
                output.push(format!("-{left}"));
                output.push(format!("+{right}"));
            }
            (Some(left), None) => output.push(format!("-{left}")),
            (None, Some(right)) => output.push(format!("+{right}")),
            (None, None) => {}
        }
    }

    output.join("\n")
}

fn confluence_diff_range(line_count: usize) -> String {
    match line_count {
        0 => "0,0".to_string(),
        1 => "1".to_string(),
        value => format!("1,{value}"),
    }
}

fn confluence_write_page_value(page: &ConfluencePage, include_content: bool) -> Value {
    let mut value = page.to_simplified_value(false);
    if !include_content && let Some(object) = value.as_object_mut() {
        object.remove("content");
    }
    value
}

fn confluence_emoji_missing_page_id_status(emoji: Option<&str>) -> ConfluenceEmojiStatus {
    let Some(emoji) = optional_non_empty_arg(emoji.map(ToString::to_string)) else {
        return ConfluenceEmojiStatus::not_requested();
    };

    ConfluenceEmojiStatus::failed(emoji, "Confluence page response did not include a page id")
}

fn confluence_expand_list(expand: Option<String>, include_content: bool) -> Vec<String> {
    let mut values = expand
        .unwrap_or_else(|| "version".to_string())
        .split(',')
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToString::to_string)
        .collect::<Vec<_>>();

    if values.is_empty() {
        values.push("version".to_string());
    }
    if include_content && !values.iter().any(|value| value.contains("body")) {
        values.push("body.storage".to_string());
    }

    values
}

fn confluence_child_page_value(
    page: &ConfluencePage,
    include_content: bool,
    convert_to_markdown: bool,
) -> Value {
    let mut value = page.to_simplified_value(convert_to_markdown);
    if !include_content && let Some(object) = value.as_object_mut() {
        object.remove("content");
    }
    value
}

#[derive(Debug)]
struct ConfluenceTreePageSortValue {
    depth: usize,
    position_sort: i64,
    title: String,
    value: Value,
}

fn confluence_tree_page_sort_value(page: &ConfluencePage) -> ConfluenceTreePageSortValue {
    let depth = page.ancestors.len();
    let parent_id = page
        .ancestors
        .last()
        .and_then(|ancestor| ancestor.id.clone());
    let position = page.extensions.get("position").and_then(Value::as_i64);
    let title = page
        .title
        .clone()
        .filter(|title| !title.trim().is_empty())
        .unwrap_or_else(|| "Untitled".to_string());
    let value = json!({
        "id": page.id.clone(),
        "title": title,
        "parent_id": parent_id,
        "position": position,
        "depth": depth,
        "space": page.space.as_ref().map(|space| space.to_simplified_value()),
        "version": page.version.as_ref().map(|version| version.to_simplified_value()),
    });

    ConfluenceTreePageSortValue {
        depth,
        position_sort: position.unwrap_or(i64::MAX),
        title,
        value,
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;

    use crate::{
        config::{HttpConfig, RuntimeConfig},
        confluence::config::{ConfluenceConfig, ConfluenceDeployment},
        tool_registry,
        upstream::{auth::UpstreamAuth, custom_headers::CustomHeaders, proxy::ProxyConfig},
    };

    use super::*;

    #[tokio::test]
    async fn operations_confluence_content_validates_inputs_before_http() {
        let context = confluence_context(ConfluenceDeployment::Cloud);

        assert!(
            search_content(
                &context,
                ConfluenceSearchArgs {
                    query: "space = ABC".to_string(),
                    limit: Some(0),
                    spaces_filter: None,
                },
            )
            .await
            .unwrap_err()
            .message
            .contains("limit must be positive")
        );
        assert!(
            get_page(
                &context,
                ConfluenceGetPageArgs {
                    page_id: None,
                    title: None,
                    space_key: None,
                    include_metadata: None,
                    convert_to_markdown: None,
                },
            )
            .await
            .unwrap_err()
            .message
            .contains("Either page_id OR both title and space_key")
        );
        assert!(
            create_page(
                &context,
                ConfluenceCreatePageArgs {
                    space_key: "ABC".to_string(),
                    title: "Title".to_string(),
                    content: "Body".to_string(),
                    parent_id: None,
                    content_format: Some("html".to_string()),
                    include_content: None,
                    emoji: None,
                },
            )
            .await
            .unwrap_err()
            .message
            .contains("content_format must be markdown, wiki, or storage")
        );
        assert!(
            get_page_diff(
                &context,
                ConfluenceGetPageDiffArgs {
                    page_id: "123".to_string(),
                    from_version: 2,
                    to_version: 1,
                    context_lines: Some(1),
                },
            )
            .await
            .unwrap_err()
            .message
            .contains("from_version must be less than or equal to to_version")
        );
    }

    #[tokio::test]
    async fn operations_confluence_analytics_unavailable_is_structured_without_http() {
        let result = get_page_view_analytics(
            &confluence_context(ConfluenceDeployment::ServerDataCenter),
            ConfluenceGetPageViewAnalyticsArgs {
                page_id: "123".to_string(),
                include_title: None,
                from_date: Some("2026-01-01".to_string()),
                to_date: Some("2026-01-31".to_string()),
            },
        )
        .await
        .unwrap();

        assert_eq!(result.is_error, false);
        assert_eq!(result.value["success"], json!(false));
        assert_eq!(result.value["available"], json!(false));
    }

    #[tokio::test]
    async fn operations_confluence_attachment_validates_inputs_before_http() {
        let context = confluence_context(ConfluenceDeployment::Cloud);

        assert!(
            upload_content_attachments(
                &context,
                ConfluenceUploadContentAttachmentsArgs {
                    content_id: "123".to_string(),
                    file_paths: " , ".to_string(),
                    comment: None,
                    minor_edit: None,
                },
            )
            .await
            .unwrap_err()
            .message
            .contains("file_paths must contain at least one local file path")
        );
        assert!(
            list_content_attachments(
                &context,
                ConfluenceListContentAttachmentsArgs {
                    content_id: "123".to_string(),
                    start: None,
                    limit: Some(0),
                    filename: None,
                    media_type: None,
                },
            )
            .await
            .unwrap_err()
            .message
            .contains("limit must be positive")
        );
        assert!(
            download_attachment(
                &context,
                ConfluenceDownloadAttachmentArgs {
                    attachment_id: "456".to_string(),
                    max_bytes: Some(0),
                },
            )
            .await
            .unwrap_err()
            .message
            .contains("max_bytes must be positive")
        );
        assert!(
            get_content_image_attachments(
                &context,
                ConfluenceGetContentImageAttachmentsArgs {
                    content_id: "123".to_string(),
                    max_bytes: Some(0),
                },
            )
            .await
            .unwrap_err()
            .message
            .contains("max_bytes must be positive")
        );
    }

    #[test]
    fn operations_confluence_diff_context_limits_unchanged_lines() {
        let diff = confluence_unified_diff("a\nb\nc\nd", "a\nB\nc\nd", 1, 2, Some(0));

        assert!(diff.contains("-b"));
        assert!(diff.contains("+B"));
        assert!(!diff.contains(" a"));
        assert!(!diff.contains(" c"));
    }

    #[test]
    fn operations_confluence_content_tool_names_have_metadata_guard_mapping() {
        for tool_name in [
            CONFLUENCE_SEARCH_TOOL_NAME,
            CONFLUENCE_GET_PAGE_TOOL_NAME,
            CONFLUENCE_LIST_PAGE_CHILDREN_TOOL_NAME,
            CONFLUENCE_GET_SPACE_PAGE_TREE_TOOL_NAME,
            CONFLUENCE_CREATE_PAGE_TOOL_NAME,
            CONFLUENCE_UPDATE_PAGE_TOOL_NAME,
            CONFLUENCE_DELETE_PAGE_TOOL_NAME,
            CONFLUENCE_MOVE_PAGE_TOOL_NAME,
            CONFLUENCE_LIST_PAGE_COMMENTS_TOOL_NAME,
            CONFLUENCE_ADD_COMMENT_TOOL_NAME,
            CONFLUENCE_REPLY_TO_COMMENT_TOOL_NAME,
            CONFLUENCE_LIST_CONTENT_LABELS_TOOL_NAME,
            CONFLUENCE_ADD_LABEL_TOOL_NAME,
            CONFLUENCE_SEARCH_USER_TOOL_NAME,
            CONFLUENCE_GET_PAGE_VERSION_TOOL_NAME,
            CONFLUENCE_GET_PAGE_DIFF_TOOL_NAME,
            CONFLUENCE_GET_PAGE_VIEW_ANALYTICS_TOOL_NAME,
            CONFLUENCE_UPLOAD_CONTENT_ATTACHMENT_TOOL_NAME,
            CONFLUENCE_UPLOAD_CONTENT_ATTACHMENTS_TOOL_NAME,
            CONFLUENCE_LIST_CONTENT_ATTACHMENTS_TOOL_NAME,
            CONFLUENCE_DOWNLOAD_ATTACHMENT_TOOL_NAME,
            CONFLUENCE_DOWNLOAD_CONTENT_ATTACHMENTS_TOOL_NAME,
            CONFLUENCE_DELETE_ATTACHMENT_TOOL_NAME,
            CONFLUENCE_GET_CONTENT_IMAGE_ATTACHMENTS_TOOL_NAME,
        ] {
            assert!(
                tool_registry::metadata_for(tool_name).is_some(),
                "{tool_name} missing metadata"
            );
        }
    }

    #[test]
    fn operations_confluence_tool_guard_ignores_mcp_disabled_content_tool() {
        let context = AppContext::from_config(&RuntimeConfig {
            confluence: Some(confluence_config(ConfluenceDeployment::Cloud)),
            mcp_disabled_tools: BTreeSet::from([CONFLUENCE_SEARCH_TOOL_NAME.to_string()]),
            mcp_enabled_toolsets: tool_registry::all_toolsets(),
            http: HttpConfig::default(),
            ..RuntimeConfig::default()
        });

        assert!(guard_operation(CONFLUENCE_SEARCH_TOOL_NAME, &context).is_ok());
    }

    fn confluence_context(deployment: ConfluenceDeployment) -> AppContext {
        AppContext::from_config(&RuntimeConfig {
            confluence: Some(confluence_config(deployment)),
            mcp_enabled_toolsets: tool_registry::all_toolsets(),
            http: HttpConfig::default(),
            ..RuntimeConfig::default()
        })
    }

    fn confluence_config(deployment: ConfluenceDeployment) -> ConfluenceConfig {
        ConfluenceConfig {
            base_url: "https://confluence.example".to_string(),
            deployment,
            auth: UpstreamAuth::Pat {
                personal_token: "test-pat-value".to_string(),
            },
            ssl_verify: true,
            proxy: ProxyConfig::default(),
            custom_headers: CustomHeaders::default(),
            mtls: None,
            spaces_filter: BTreeSet::new(),
            timeout_seconds: 75,
        }
    }
}

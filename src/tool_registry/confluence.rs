use crate::confluence::tools as confluence_tools;

use super::{ToolAccess, ToolAnnotationMetadata, ToolMetadata, ToolOutputSchema, ToolService};

confluence_metadata!(
    CONFLUENCE_SEARCH_METADATA,
    confluence_tools::CONFLUENCE_SEARCH_TOOL_NAME,
    Read,
    "confluence_content_read",
    read_only,
    "Search Confluence content",
    "Search Confluence content using plain search terms or CQL, optionally restricted to spaces."
);
confluence_metadata!(
    CONFLUENCE_GET_PAGE_METADATA,
    confluence_tools::CONFLUENCE_GET_PAGE_TOOL_NAME,
    Read,
    "confluence_content_read",
    read_only,
    "Get Confluence page",
    "Get one Confluence page by page_id, or by title plus space_key, with optional metadata and Markdown conversion."
);
confluence_metadata!(
    CONFLUENCE_LIST_PAGE_CHILDREN_METADATA,
    confluence_tools::CONFLUENCE_LIST_PAGE_CHILDREN_TOOL_NAME,
    Read,
    "confluence_content_read",
    read_only,
    "List Confluence page children",
    "List child pages and folders for a Confluence page with bounded pagination and optional content."
);
confluence_metadata!(
    CONFLUENCE_GET_SPACE_PAGE_TREE_METADATA,
    confluence_tools::CONFLUENCE_GET_SPACE_PAGE_TREE_TOOL_NAME,
    Read,
    "confluence_content_read",
    read_only,
    "Get Confluence space page tree",
    "Get a bounded flat page hierarchy for a Confluence space."
);
confluence_metadata!(
    CONFLUENCE_CREATE_PAGE_METADATA,
    confluence_tools::CONFLUENCE_CREATE_PAGE_TOOL_NAME,
    Write,
    "confluence_content_write",
    additive_write,
    "Create Confluence page",
    "Create a Confluence page from Markdown or storage-format content, optionally under a parent page."
);
confluence_metadata!(
    CONFLUENCE_UPDATE_PAGE_METADATA,
    confluence_tools::CONFLUENCE_UPDATE_PAGE_TOOL_NAME,
    Write,
    "confluence_content_update",
    destructive_write,
    "Update Confluence page",
    "Update a Confluence page from Markdown or storage-format content, with minor-edit and version-comment controls."
);
confluence_metadata!(
    CONFLUENCE_DELETE_PAGE_METADATA,
    confluence_tools::CONFLUENCE_DELETE_PAGE_TOOL_NAME,
    Write,
    "confluence_content_delete",
    destructive_write,
    "Delete Confluence page",
    "Delete a Confluence page by page_id."
);
confluence_metadata!(
    CONFLUENCE_MOVE_PAGE_METADATA,
    confluence_tools::CONFLUENCE_MOVE_PAGE_TOOL_NAME,
    Write,
    "confluence_content_update",
    destructive_write,
    "Move Confluence page",
    "Move a Confluence page to a new parent page or target space."
);
confluence_metadata!(
    CONFLUENCE_LIST_PAGE_COMMENTS_METADATA,
    confluence_tools::CONFLUENCE_LIST_PAGE_COMMENTS_TOOL_NAME,
    Read,
    "confluence_page_comments_read",
    read_only,
    "List Confluence page comments",
    "List comments for a Confluence page."
);
confluence_metadata!(
    CONFLUENCE_ADD_COMMENT_METADATA,
    confluence_tools::CONFLUENCE_ADD_COMMENT_TOOL_NAME,
    Write,
    "confluence_page_comments_write",
    additive_write,
    "Add Confluence page comment",
    "Add a Markdown comment to a Confluence page."
);
confluence_metadata!(
    CONFLUENCE_REPLY_TO_COMMENT_METADATA,
    confluence_tools::CONFLUENCE_REPLY_TO_COMMENT_TOOL_NAME,
    Write,
    "confluence_page_comments_write",
    additive_write,
    "Reply to Confluence comment",
    "Reply with Markdown content to a Confluence comment thread."
);
confluence_metadata!(
    CONFLUENCE_LIST_CONTENT_LABELS_METADATA,
    confluence_tools::CONFLUENCE_LIST_CONTENT_LABELS_TOOL_NAME,
    Read,
    "confluence_content_labels_read",
    read_only,
    "List Confluence content labels",
    "List labels for Confluence content by content/page id."
);
confluence_metadata!(
    CONFLUENCE_ADD_LABEL_METADATA,
    confluence_tools::CONFLUENCE_ADD_LABEL_TOOL_NAME,
    Write,
    "confluence_content_labels_write",
    additive_write,
    "Add Confluence content label",
    "Add one label to Confluence content by content/page id."
);
confluence_metadata!(
    CONFLUENCE_SEARCH_USER_METADATA,
    confluence_tools::CONFLUENCE_SEARCH_USER_TOOL_NAME,
    Read,
    "confluence_users_read",
    read_only,
    "Search Confluence users",
    "Search Confluence users, optionally restricted to a group."
);
confluence_metadata!(
    CONFLUENCE_GET_PAGE_VERSION_METADATA,
    confluence_tools::CONFLUENCE_GET_PAGE_VERSION_TOOL_NAME,
    Read,
    "confluence_page_versions_read",
    read_only,
    "Get Confluence page version",
    "Get one historical version of a Confluence page, optionally converted to Markdown."
);
confluence_metadata!(
    CONFLUENCE_GET_PAGE_DIFF_METADATA,
    confluence_tools::CONFLUENCE_GET_PAGE_DIFF_TOOL_NAME,
    Read,
    "confluence_page_versions_read",
    read_only,
    "Get Confluence page diff",
    "Get a unified diff between two Confluence page versions."
);
confluence_metadata!(
    CONFLUENCE_GET_PAGE_VIEW_ANALYTICS_METADATA,
    confluence_tools::CONFLUENCE_GET_PAGE_VIEW_ANALYTICS_TOOL_NAME,
    Read,
    "confluence_page_analytics_read",
    read_only,
    "Get Confluence page view analytics",
    "Get Confluence Cloud page-view analytics; Server/Data Center returns an unavailable response.",
    ConfluenceAnalyticsResult
);
confluence_metadata!(
    CONFLUENCE_UPLOAD_CONTENT_ATTACHMENT_METADATA,
    confluence_tools::CONFLUENCE_UPLOAD_CONTENT_ATTACHMENT_TOOL_NAME,
    Write,
    "confluence_attachments_write",
    destructive_write,
    "Upload Confluence content attachment",
    "Upload one server-local file path as an attachment to Confluence content."
);
confluence_metadata!(
    CONFLUENCE_UPLOAD_CONTENT_ATTACHMENTS_METADATA,
    confluence_tools::CONFLUENCE_UPLOAD_CONTENT_ATTACHMENTS_TOOL_NAME,
    Write,
    "confluence_attachments_write",
    destructive_write,
    "Upload Confluence content attachments",
    "Upload multiple server-local file paths as attachments to Confluence content, returning partial-failure details.",
    ConfluenceBatchAttachmentUploadResult
);
confluence_metadata!(
    CONFLUENCE_LIST_CONTENT_ATTACHMENTS_METADATA,
    confluence_tools::CONFLUENCE_LIST_CONTENT_ATTACHMENTS_TOOL_NAME,
    Read,
    "confluence_attachments_read",
    read_only,
    "List Confluence content attachments",
    "List attachments for Confluence content with pagination and filename/media-type filters."
);
confluence_metadata!(
    CONFLUENCE_DOWNLOAD_ATTACHMENT_METADATA,
    confluence_tools::CONFLUENCE_DOWNLOAD_ATTACHMENT_TOOL_NAME,
    Read,
    "confluence_attachments_read",
    read_only,
    "Download Confluence attachment",
    "Download one Confluence attachment and return bounded inline content.",
    ConfluenceAttachmentDownloadResult
);
confluence_metadata!(
    CONFLUENCE_DOWNLOAD_CONTENT_ATTACHMENTS_METADATA,
    confluence_tools::CONFLUENCE_DOWNLOAD_CONTENT_ATTACHMENTS_TOOL_NAME,
    Read,
    "confluence_attachments_read",
    read_only,
    "Download Confluence content attachments",
    "Download Confluence content attachments with bounded inline content and protected pagination limits.",
    ConfluenceBatchAttachmentDownloadResult
);
confluence_metadata!(
    CONFLUENCE_DELETE_ATTACHMENT_METADATA,
    confluence_tools::CONFLUENCE_DELETE_ATTACHMENT_TOOL_NAME,
    Write,
    "confluence_attachments_delete",
    destructive_write,
    "Delete Confluence attachment",
    "Delete one Confluence attachment by attachment_id."
);
confluence_metadata!(
    CONFLUENCE_GET_CONTENT_IMAGE_ATTACHMENTS_METADATA,
    confluence_tools::CONFLUENCE_GET_CONTENT_IMAGE_ATTACHMENTS_TOOL_NAME,
    Read,
    "confluence_attachments_read",
    read_only,
    "Get Confluence content image attachments",
    "Fetch image attachments for Confluence content from a bounded attachment listing and include bounded inline content."
);

pub(super) const TOOLS: &[ToolMetadata] = &[
    CONFLUENCE_SEARCH_METADATA,
    CONFLUENCE_GET_PAGE_METADATA,
    CONFLUENCE_LIST_PAGE_CHILDREN_METADATA,
    CONFLUENCE_GET_SPACE_PAGE_TREE_METADATA,
    CONFLUENCE_CREATE_PAGE_METADATA,
    CONFLUENCE_UPDATE_PAGE_METADATA,
    CONFLUENCE_DELETE_PAGE_METADATA,
    CONFLUENCE_MOVE_PAGE_METADATA,
    CONFLUENCE_LIST_PAGE_COMMENTS_METADATA,
    CONFLUENCE_ADD_COMMENT_METADATA,
    CONFLUENCE_REPLY_TO_COMMENT_METADATA,
    CONFLUENCE_LIST_CONTENT_LABELS_METADATA,
    CONFLUENCE_ADD_LABEL_METADATA,
    CONFLUENCE_SEARCH_USER_METADATA,
    CONFLUENCE_GET_PAGE_VERSION_METADATA,
    CONFLUENCE_GET_PAGE_DIFF_METADATA,
    CONFLUENCE_GET_PAGE_VIEW_ANALYTICS_METADATA,
    CONFLUENCE_UPLOAD_CONTENT_ATTACHMENT_METADATA,
    CONFLUENCE_UPLOAD_CONTENT_ATTACHMENTS_METADATA,
    CONFLUENCE_LIST_CONTENT_ATTACHMENTS_METADATA,
    CONFLUENCE_DOWNLOAD_ATTACHMENT_METADATA,
    CONFLUENCE_DOWNLOAD_CONTENT_ATTACHMENTS_METADATA,
    CONFLUENCE_DELETE_ATTACHMENT_METADATA,
    CONFLUENCE_GET_CONTENT_IMAGE_ATTACHMENTS_METADATA,
];

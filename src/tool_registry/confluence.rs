use crate::confluence::tools as confluence_tools;

use super::{ToolAccess, ToolMetadata, ToolService};

confluence_metadata!(
    CONFLUENCE_SEARCH_METADATA,
    confluence_tools::CONFLUENCE_SEARCH_TOOL_NAME,
    Read,
    "confluence_pages",
    "Search Confluence content",
    "Search Confluence content using simple terms or CQL."
);
confluence_metadata!(
    CONFLUENCE_GET_PAGE_METADATA,
    confluence_tools::CONFLUENCE_GET_PAGE_TOOL_NAME,
    Read,
    "confluence_pages",
    "Get Confluence page",
    "Get a Confluence page by ID or title and space key."
);
confluence_metadata!(
    CONFLUENCE_GET_PAGE_CHILDREN_METADATA,
    confluence_tools::CONFLUENCE_GET_PAGE_CHILDREN_TOOL_NAME,
    Read,
    "confluence_pages",
    "Get Confluence page children",
    "List child pages and folders for a Confluence page."
);
confluence_metadata!(
    CONFLUENCE_GET_SPACE_PAGE_TREE_METADATA,
    confluence_tools::CONFLUENCE_GET_SPACE_PAGE_TREE_TOOL_NAME,
    Read,
    "confluence_pages",
    "Get Confluence space page tree",
    "Get a flat page hierarchy for a Confluence space."
);
confluence_metadata!(
    CONFLUENCE_CREATE_PAGE_METADATA,
    confluence_tools::CONFLUENCE_CREATE_PAGE_TOOL_NAME,
    Write,
    "confluence_pages",
    "Create Confluence page",
    "Create a Confluence page."
);
confluence_metadata!(
    CONFLUENCE_UPDATE_PAGE_METADATA,
    confluence_tools::CONFLUENCE_UPDATE_PAGE_TOOL_NAME,
    Write,
    "confluence_pages",
    "Update Confluence page",
    "Update a Confluence page."
);
confluence_metadata!(
    CONFLUENCE_DELETE_PAGE_METADATA,
    confluence_tools::CONFLUENCE_DELETE_PAGE_TOOL_NAME,
    Write,
    "confluence_pages",
    "Delete Confluence page",
    "Delete a Confluence page."
);
confluence_metadata!(
    CONFLUENCE_MOVE_PAGE_METADATA,
    confluence_tools::CONFLUENCE_MOVE_PAGE_TOOL_NAME,
    Write,
    "confluence_pages",
    "Move Confluence page",
    "Move a Confluence page to a new parent or space."
);
confluence_metadata!(
    CONFLUENCE_GET_COMMENTS_METADATA,
    confluence_tools::CONFLUENCE_GET_COMMENTS_TOOL_NAME,
    Read,
    "confluence_comments",
    "Get Confluence comments",
    "List comments for a Confluence page."
);
confluence_metadata!(
    CONFLUENCE_ADD_COMMENT_METADATA,
    confluence_tools::CONFLUENCE_ADD_COMMENT_TOOL_NAME,
    Write,
    "confluence_comments",
    "Add Confluence comment",
    "Add a comment to a Confluence page."
);
confluence_metadata!(
    CONFLUENCE_REPLY_TO_COMMENT_METADATA,
    confluence_tools::CONFLUENCE_REPLY_TO_COMMENT_TOOL_NAME,
    Write,
    "confluence_comments",
    "Reply to Confluence comment",
    "Reply to a Confluence comment thread."
);
confluence_metadata!(
    CONFLUENCE_GET_LABELS_METADATA,
    confluence_tools::CONFLUENCE_GET_LABELS_TOOL_NAME,
    Read,
    "confluence_labels",
    "Get Confluence labels",
    "List labels for Confluence content."
);
confluence_metadata!(
    CONFLUENCE_ADD_LABEL_METADATA,
    confluence_tools::CONFLUENCE_ADD_LABEL_TOOL_NAME,
    Write,
    "confluence_labels",
    "Add Confluence label",
    "Add a label to Confluence content."
);
confluence_metadata!(
    CONFLUENCE_SEARCH_USER_METADATA,
    confluence_tools::CONFLUENCE_SEARCH_USER_TOOL_NAME,
    Read,
    "confluence_users",
    "Search Confluence users",
    "Search Confluence users."
);
confluence_metadata!(
    CONFLUENCE_GET_PAGE_HISTORY_METADATA,
    confluence_tools::CONFLUENCE_GET_PAGE_HISTORY_TOOL_NAME,
    Read,
    "confluence_pages",
    "Get Confluence page history",
    "Get a historical version of a Confluence page."
);
confluence_metadata!(
    CONFLUENCE_GET_PAGE_DIFF_METADATA,
    confluence_tools::CONFLUENCE_GET_PAGE_DIFF_TOOL_NAME,
    Read,
    "confluence_pages",
    "Get Confluence page diff",
    "Get a diff between two Confluence page versions."
);
confluence_metadata!(
    CONFLUENCE_GET_PAGE_VIEWS_METADATA,
    confluence_tools::CONFLUENCE_GET_PAGE_VIEWS_TOOL_NAME,
    Read,
    "confluence_analytics",
    "Get Confluence page views",
    "Get Confluence Cloud page view analytics."
);
confluence_metadata!(
    CONFLUENCE_UPLOAD_ATTACHMENT_METADATA,
    confluence_tools::CONFLUENCE_UPLOAD_ATTACHMENT_TOOL_NAME,
    Write,
    "confluence_attachments",
    "Upload Confluence attachment",
    "Upload an attachment to Confluence content."
);
confluence_metadata!(
    CONFLUENCE_UPLOAD_ATTACHMENTS_METADATA,
    confluence_tools::CONFLUENCE_UPLOAD_ATTACHMENTS_TOOL_NAME,
    Write,
    "confluence_attachments",
    "Upload Confluence attachments",
    "Upload multiple attachments to Confluence content."
);
confluence_metadata!(
    CONFLUENCE_GET_ATTACHMENTS_METADATA,
    confluence_tools::CONFLUENCE_GET_ATTACHMENTS_TOOL_NAME,
    Read,
    "confluence_attachments",
    "Get Confluence attachments",
    "List attachments for Confluence content."
);
confluence_metadata!(
    CONFLUENCE_DOWNLOAD_ATTACHMENT_METADATA,
    confluence_tools::CONFLUENCE_DOWNLOAD_ATTACHMENT_TOOL_NAME,
    Read,
    "confluence_attachments",
    "Download Confluence attachment",
    "Download one Confluence attachment with bounded content output."
);
confluence_metadata!(
    CONFLUENCE_DOWNLOAD_CONTENT_ATTACHMENTS_METADATA,
    confluence_tools::CONFLUENCE_DOWNLOAD_CONTENT_ATTACHMENTS_TOOL_NAME,
    Read,
    "confluence_attachments",
    "Download Confluence content attachments",
    "Download all attachments for Confluence content with bounded output."
);
confluence_metadata!(
    CONFLUENCE_DELETE_ATTACHMENT_METADATA,
    confluence_tools::CONFLUENCE_DELETE_ATTACHMENT_TOOL_NAME,
    Write,
    "confluence_attachments",
    "Delete Confluence attachment",
    "Delete a Confluence attachment."
);
confluence_metadata!(
    CONFLUENCE_GET_PAGE_IMAGES_METADATA,
    confluence_tools::CONFLUENCE_GET_PAGE_IMAGES_TOOL_NAME,
    Read,
    "confluence_attachments",
    "Get Confluence page images",
    "Get image attachments for Confluence content."
);

pub(super) const TOOLS: &[ToolMetadata] = &[
    CONFLUENCE_SEARCH_METADATA,
    CONFLUENCE_GET_PAGE_METADATA,
    CONFLUENCE_GET_PAGE_CHILDREN_METADATA,
    CONFLUENCE_GET_SPACE_PAGE_TREE_METADATA,
    CONFLUENCE_CREATE_PAGE_METADATA,
    CONFLUENCE_UPDATE_PAGE_METADATA,
    CONFLUENCE_DELETE_PAGE_METADATA,
    CONFLUENCE_MOVE_PAGE_METADATA,
    CONFLUENCE_GET_COMMENTS_METADATA,
    CONFLUENCE_ADD_COMMENT_METADATA,
    CONFLUENCE_REPLY_TO_COMMENT_METADATA,
    CONFLUENCE_GET_LABELS_METADATA,
    CONFLUENCE_ADD_LABEL_METADATA,
    CONFLUENCE_SEARCH_USER_METADATA,
    CONFLUENCE_GET_PAGE_HISTORY_METADATA,
    CONFLUENCE_GET_PAGE_DIFF_METADATA,
    CONFLUENCE_GET_PAGE_VIEWS_METADATA,
    CONFLUENCE_UPLOAD_ATTACHMENT_METADATA,
    CONFLUENCE_UPLOAD_ATTACHMENTS_METADATA,
    CONFLUENCE_GET_ATTACHMENTS_METADATA,
    CONFLUENCE_DOWNLOAD_ATTACHMENT_METADATA,
    CONFLUENCE_DOWNLOAD_CONTENT_ATTACHMENTS_METADATA,
    CONFLUENCE_DELETE_ATTACHMENT_METADATA,
    CONFLUENCE_GET_PAGE_IMAGES_METADATA,
];

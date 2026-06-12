use rmcp::schemars;
use serde::{Deserialize, Deserializer, Serialize, de::Error as DeError};
use serde_json::Value;

fn string_or_number<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: Deserializer<'de>,
{
    match Value::deserialize(deserializer)? {
        Value::String(value) => Ok(value),
        Value::Number(value) => Ok(value.to_string()),
        _ => Err(D::Error::custom("expected string or number")),
    }
}

fn optional_string_or_number<'de, D>(deserializer: D) -> Result<Option<String>, D::Error>
where
    D: Deserializer<'de>,
{
    match Option::<Value>::deserialize(deserializer)? {
        None | Some(Value::Null) => Ok(None),
        Some(Value::String(value)) => Ok(Some(value)),
        Some(Value::Number(value)) => Ok(Some(value.to_string())),
        _ => Err(D::Error::custom("expected string, number, or null")),
    }
}

fn string_or_number_schema(_: &mut schemars::SchemaGenerator) -> schemars::Schema {
    schemars::json_schema!({
        "description": "Accepts a Confluence numeric id as either a JSON string or number.",
        "oneOf": [
            { "type": "string" },
            { "type": "number" }
        ]
    })
}

fn optional_string_or_number_schema(_: &mut schemars::SchemaGenerator) -> schemars::Schema {
    schemars::json_schema!({
        "description": "Accepts a Confluence numeric id as either a JSON string or number.",
        "oneOf": [
            { "type": "string" },
            { "type": "number" }
        ]
    })
}

pub const CONFLUENCE_SEARCH_TOOL_NAME: &str = "confluence_search_content";
pub const CONFLUENCE_GET_PAGE_TOOL_NAME: &str = "confluence_get_page";
pub const CONFLUENCE_LIST_PAGE_CHILDREN_TOOL_NAME: &str = "confluence_list_page_children";
pub const CONFLUENCE_GET_SPACE_PAGE_TREE_TOOL_NAME: &str = "confluence_get_space_page_tree";
pub const CONFLUENCE_CREATE_PAGE_TOOL_NAME: &str = "confluence_create_page";
pub const CONFLUENCE_UPDATE_PAGE_TOOL_NAME: &str = "confluence_update_page";
pub const CONFLUENCE_DELETE_PAGE_TOOL_NAME: &str = "confluence_delete_page";
pub const CONFLUENCE_MOVE_PAGE_TOOL_NAME: &str = "confluence_move_page";
pub const CONFLUENCE_LIST_PAGE_COMMENTS_TOOL_NAME: &str = "confluence_list_page_comments";
pub const CONFLUENCE_ADD_COMMENT_TOOL_NAME: &str = "confluence_add_page_comment";
pub const CONFLUENCE_REPLY_TO_COMMENT_TOOL_NAME: &str = "confluence_reply_to_comment";
pub const CONFLUENCE_LIST_CONTENT_LABELS_TOOL_NAME: &str = "confluence_list_content_labels";
pub const CONFLUENCE_ADD_LABEL_TOOL_NAME: &str = "confluence_add_content_label";
pub const CONFLUENCE_SEARCH_USER_TOOL_NAME: &str = "confluence_search_users";
pub const CONFLUENCE_GET_PAGE_VERSION_TOOL_NAME: &str = "confluence_get_page_version";
pub const CONFLUENCE_GET_PAGE_DIFF_TOOL_NAME: &str = "confluence_get_page_diff";
pub const CONFLUENCE_GET_PAGE_VIEW_ANALYTICS_TOOL_NAME: &str = "confluence_get_page_view_analytics";
pub const CONFLUENCE_UPLOAD_CONTENT_ATTACHMENT_TOOL_NAME: &str =
    "confluence_upload_content_attachment";
pub const CONFLUENCE_UPLOAD_CONTENT_ATTACHMENTS_TOOL_NAME: &str =
    "confluence_upload_content_attachments";
pub const CONFLUENCE_LIST_CONTENT_ATTACHMENTS_TOOL_NAME: &str =
    "confluence_list_content_attachments";
pub const CONFLUENCE_DOWNLOAD_ATTACHMENT_TOOL_NAME: &str = "confluence_download_attachment";
pub const CONFLUENCE_DOWNLOAD_CONTENT_ATTACHMENTS_TOOL_NAME: &str =
    "confluence_download_content_attachments";
pub const CONFLUENCE_DELETE_ATTACHMENT_TOOL_NAME: &str = "confluence_delete_attachment";
pub const CONFLUENCE_GET_CONTENT_IMAGE_ATTACHMENTS_TOOL_NAME: &str =
    "confluence_get_content_image_attachments";

#[cfg(test)]
pub const CONFLUENCE_TOOL_NAMES: &[&str] = &[
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
];

#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct ConfluenceSearchArgs {
    pub query: String,
    #[serde(default)]
    #[schemars(description = "Maximum number of Confluence search results to return.")]
    pub limit: Option<u64>,
    #[serde(default)]
    #[schemars(
        description = "Comma-separated Confluence space keys used to restrict search results."
    )]
    pub spaces_filter: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct ConfluenceGetPageArgs {
    #[serde(default, deserialize_with = "optional_string_or_number")]
    #[schemars(schema_with = "optional_string_or_number_schema")]
    pub page_id: Option<String>,
    #[serde(default)]
    pub title: Option<String>,
    #[serde(default)]
    pub space_key: Option<String>,
    #[serde(default)]
    pub include_metadata: Option<bool>,
    #[serde(default)]
    pub convert_to_markdown: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct ConfluenceListPageChildrenArgs {
    #[serde(deserialize_with = "string_or_number")]
    #[schemars(schema_with = "string_or_number_schema")]
    pub parent_id: String,
    #[serde(default)]
    pub expand: Option<String>,
    #[serde(default)]
    #[schemars(description = "Maximum number of child pages or folders to return.")]
    pub limit: Option<u64>,
    #[serde(default)]
    #[schemars(description = "When true, include bounded child-page content in the response.")]
    pub include_content: Option<bool>,
    #[serde(default)]
    pub convert_to_markdown: Option<bool>,
    #[serde(default)]
    #[schemars(description = "Offset pagination start index for page children.")]
    pub start: Option<u64>,
    #[serde(default)]
    pub include_folders: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct ConfluenceGetSpacePageTreeArgs {
    pub space_key: String,
    #[serde(default)]
    pub limit: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct ConfluenceCreatePageArgs {
    pub space_key: String,
    pub title: String,
    pub content: String,
    #[serde(default, deserialize_with = "optional_string_or_number")]
    #[schemars(schema_with = "optional_string_or_number_schema")]
    pub parent_id: Option<String>,
    #[serde(default)]
    #[schemars(
        description = "Content input format. Use markdown for normal text, or storage when passing Confluence storage-format XHTML."
    )]
    pub content_format: Option<String>,
    #[serde(default)]
    #[schemars(
        description = "When true, include the created page content in the structured response."
    )]
    pub include_content: Option<bool>,
    #[serde(default)]
    pub emoji: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct ConfluenceUpdatePageArgs {
    #[serde(deserialize_with = "string_or_number")]
    #[schemars(schema_with = "string_or_number_schema")]
    pub page_id: String,
    pub title: String,
    pub content: String,
    #[serde(default)]
    #[schemars(
        description = "When true, mark the update as a minor edit in Confluence version history."
    )]
    pub is_minor_edit: Option<bool>,
    #[serde(default)]
    #[schemars(description = "Optional Confluence version comment to store with the page update.")]
    pub version_comment: Option<String>,
    #[serde(default, deserialize_with = "optional_string_or_number")]
    #[schemars(schema_with = "optional_string_or_number_schema")]
    pub parent_id: Option<String>,
    #[serde(default)]
    #[schemars(
        description = "Content input format. Use markdown for normal text, or storage when passing Confluence storage-format XHTML."
    )]
    pub content_format: Option<String>,
    #[serde(default)]
    #[schemars(
        description = "When true, include the updated page content in the structured response."
    )]
    pub include_content: Option<bool>,
    #[serde(default)]
    pub emoji: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct ConfluenceDeletePageArgs {
    #[serde(deserialize_with = "string_or_number")]
    #[schemars(schema_with = "string_or_number_schema")]
    pub page_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct ConfluenceMovePageArgs {
    #[serde(deserialize_with = "string_or_number")]
    #[schemars(schema_with = "string_or_number_schema")]
    pub page_id: String,
    #[serde(default, deserialize_with = "optional_string_or_number")]
    #[schemars(schema_with = "optional_string_or_number_schema")]
    pub target_parent_id: Option<String>,
    #[serde(default)]
    #[schemars(description = "Target space key when moving the page across spaces.")]
    pub target_space_key: Option<String>,
    #[serde(default)]
    #[schemars(
        description = "Move placement relative to the target, such as append, before, or after depending on Confluence API support."
    )]
    pub position: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct ConfluenceListPageCommentsArgs {
    #[serde(deserialize_with = "string_or_number")]
    #[schemars(schema_with = "string_or_number_schema")]
    pub page_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct ConfluenceAddCommentArgs {
    #[serde(deserialize_with = "string_or_number")]
    #[schemars(schema_with = "string_or_number_schema")]
    pub page_id: String,
    #[schemars(
        description = "Comment body in Markdown; it is converted before calling Confluence."
    )]
    pub body: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct ConfluenceReplyToCommentArgs {
    #[serde(deserialize_with = "string_or_number")]
    #[schemars(schema_with = "string_or_number_schema")]
    pub comment_id: String,
    #[schemars(description = "Reply body in Markdown; it is converted before calling Confluence.")]
    pub body: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct ConfluenceListContentLabelsArgs {
    #[serde(deserialize_with = "string_or_number")]
    #[schemars(schema_with = "string_or_number_schema")]
    pub page_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct ConfluenceAddLabelArgs {
    #[serde(deserialize_with = "string_or_number")]
    #[schemars(schema_with = "string_or_number_schema")]
    pub page_id: String,
    #[schemars(description = "Label name to add to the Confluence content.")]
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct ConfluenceSearchUserArgs {
    pub query: String,
    #[serde(default)]
    pub limit: Option<u64>,
    #[serde(default)]
    pub group_name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct ConfluenceGetPageVersionArgs {
    #[serde(deserialize_with = "string_or_number")]
    #[schemars(schema_with = "string_or_number_schema")]
    pub page_id: String,
    pub version: u64,
    #[serde(default)]
    pub convert_to_markdown: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct ConfluenceGetPageDiffArgs {
    #[serde(deserialize_with = "string_or_number")]
    #[schemars(schema_with = "string_or_number_schema")]
    pub page_id: String,
    pub from_version: u64,
    pub to_version: u64,
    #[serde(default)]
    #[schemars(description = "Optional number of unchanged context lines around diff hunks.")]
    pub context_lines: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct ConfluenceGetPageViewAnalyticsArgs {
    #[serde(deserialize_with = "string_or_number")]
    #[schemars(schema_with = "string_or_number_schema")]
    pub page_id: String,
    #[serde(default)]
    #[schemars(
        description = "When true, include the Confluence page title in the Cloud-only analytics response."
    )]
    pub include_title: Option<bool>,
    #[serde(default, rename = "from")]
    #[schemars(description = "Optional analytics start date accepted by Confluence Cloud.")]
    pub from_date: Option<String>,
    #[serde(default, rename = "to")]
    #[schemars(description = "Optional analytics end date accepted by Confluence Cloud.")]
    pub to_date: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct ConfluenceUploadContentAttachmentArgs {
    #[serde(deserialize_with = "string_or_number")]
    #[schemars(schema_with = "string_or_number_schema")]
    pub content_id: String,
    #[schemars(
        description = "Server-local file path to upload. The path is read on the MCP server host, not the client machine."
    )]
    pub file_path: String,
    #[serde(default)]
    #[schemars(description = "Optional attachment comment stored by Confluence.")]
    pub comment: Option<String>,
    #[serde(default)]
    #[schemars(
        description = "When true, mark the attachment upload as a minor edit when Confluence supports it."
    )]
    pub minor_edit: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct ConfluenceUploadContentAttachmentsArgs {
    #[serde(deserialize_with = "string_or_number")]
    #[schemars(schema_with = "string_or_number_schema")]
    pub content_id: String,
    #[schemars(
        description = "Comma-separated server-local file paths to upload. Paths are read on the MCP server host."
    )]
    pub file_paths: String,
    #[serde(default)]
    #[schemars(description = "Optional attachment comment applied to uploaded files.")]
    pub comment: Option<String>,
    #[serde(default)]
    #[schemars(
        description = "When true, mark uploads as minor edits when Confluence supports it."
    )]
    pub minor_edit: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct ConfluenceListContentAttachmentsArgs {
    #[serde(deserialize_with = "string_or_number")]
    #[schemars(schema_with = "string_or_number_schema")]
    pub content_id: String,
    #[serde(default)]
    #[schemars(description = "Offset pagination start index for attachment listing.")]
    pub start: Option<u64>,
    #[serde(default)]
    #[schemars(description = "Maximum number of attachments to list.")]
    pub limit: Option<u64>,
    #[serde(default)]
    pub filename: Option<String>,
    #[serde(default)]
    pub media_type: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct ConfluenceDownloadAttachmentArgs {
    #[serde(deserialize_with = "string_or_number")]
    #[schemars(schema_with = "string_or_number_schema")]
    pub attachment_id: String,
    #[serde(default)]
    #[schemars(description = "Maximum bytes of inline content to include for this attachment.")]
    pub max_bytes: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct ConfluenceDownloadContentAttachmentsArgs {
    #[serde(deserialize_with = "string_or_number")]
    #[schemars(schema_with = "string_or_number_schema")]
    pub content_id: String,
    #[serde(default)]
    pub filename: Option<String>,
    #[serde(default)]
    pub media_type: Option<String>,
    #[serde(default)]
    #[schemars(description = "Maximum bytes of inline content to include per attachment.")]
    pub max_bytes: Option<u64>,
    #[serde(default)]
    #[schemars(description = "Maximum number of attachments to download for this content.")]
    pub limit: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct ConfluenceDeleteAttachmentArgs {
    #[serde(deserialize_with = "string_or_number")]
    #[schemars(schema_with = "string_or_number_schema")]
    pub attachment_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct ConfluenceGetContentImageAttachmentsArgs {
    #[serde(deserialize_with = "string_or_number")]
    #[schemars(schema_with = "string_or_number_schema")]
    pub content_id: String,
    #[serde(default)]
    #[schemars(description = "Maximum bytes of inline content to include per image attachment.")]
    pub max_bytes: Option<u64>,
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;

    use serde_json::json;

    use super::*;

    #[test]
    fn confluence_tool_names_match_current_surface_count() {
        assert_eq!(CONFLUENCE_TOOL_NAMES.len(), 24);
    }

    #[test]
    fn confluence_tool_names_are_unique() {
        let unique = CONFLUENCE_TOOL_NAMES
            .iter()
            .copied()
            .collect::<BTreeSet<_>>();

        assert_eq!(unique.len(), CONFLUENCE_TOOL_NAMES.len());
    }

    #[test]
    fn confluence_tool_names_use_canonical_action_names() {
        let expected = [
            "confluence_search_content",
            "confluence_get_page",
            "confluence_list_page_children",
            "confluence_get_space_page_tree",
            "confluence_create_page",
            "confluence_update_page",
            "confluence_delete_page",
            "confluence_move_page",
            "confluence_list_page_comments",
            "confluence_add_page_comment",
            "confluence_reply_to_comment",
            "confluence_list_content_labels",
            "confluence_add_content_label",
            "confluence_search_users",
            "confluence_get_page_version",
            "confluence_get_page_diff",
            "confluence_get_page_view_analytics",
            "confluence_upload_content_attachment",
            "confluence_upload_content_attachments",
            "confluence_list_content_attachments",
            "confluence_download_attachment",
            "confluence_download_content_attachments",
            "confluence_delete_attachment",
            "confluence_get_content_image_attachments",
        ];

        assert_eq!(CONFLUENCE_TOOL_NAMES, expected);
    }

    #[test]
    fn page_id_args_accept_numeric_ids_like_python_reference() {
        let args: ConfluenceGetPageArgs = serde_json::from_value(json!({
            "page_id": 123456789,
            "include_metadata": false
        }))
        .unwrap();

        assert_eq!(args.page_id.as_deref(), Some("123456789"));
        assert_eq!(args.include_metadata, Some(false));
    }

    #[test]
    fn create_and_update_args_preserve_write_control_inputs() {
        let create: ConfluenceCreatePageArgs = serde_json::from_value(json!({
            "space_key": "ENG",
            "title": "Roadmap",
            "content": "# Roadmap",
            "parent_id": 123,
            "content_format": "markdown",
            "include_content": false,
            "emoji": "docs"
        }))
        .unwrap();
        let update: ConfluenceUpdatePageArgs = serde_json::from_value(json!({
            "page_id": 456,
            "title": "Roadmap updated",
            "content": "<p>Body</p>",
            "is_minor_edit": true,
            "version_comment": "refresh",
            "parent_id": null,
            "content_format": "storage"
        }))
        .unwrap();

        assert_eq!(create.parent_id.as_deref(), Some("123"));
        assert_eq!(update.page_id, "456");
        assert_eq!(update.is_minor_edit, Some(true));
        assert_eq!(update.parent_id, None);
    }

    #[test]
    fn attachment_args_cover_single_batch_download_and_images() {
        let upload: ConfluenceUploadContentAttachmentArgs = serde_json::from_value(json!({
            "content_id": 123,
            "file_path": "./diagram.png",
            "minor_edit": true
        }))
        .unwrap();
        let batch: ConfluenceUploadContentAttachmentsArgs = serde_json::from_value(json!({
            "content_id": "123",
            "file_paths": "./a.png,./b.png",
            "comment": "assets"
        }))
        .unwrap();
        let download: ConfluenceDownloadAttachmentArgs = serde_json::from_value(json!({
            "attachment_id": "att123"
        }))
        .unwrap();
        let images: ConfluenceGetContentImageAttachmentsArgs = serde_json::from_value(json!({
            "content_id": 123
        }))
        .unwrap();

        assert_eq!(upload.content_id, "123");
        assert_eq!(upload.minor_edit, Some(true));
        assert_eq!(batch.file_paths, "./a.png,./b.png");
        assert_eq!(download.attachment_id, "att123");
        assert_eq!(images.content_id, "123");
    }
}

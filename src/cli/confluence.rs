use std::{
    fs,
    io::{self, Read},
    path::PathBuf,
};

use clap::{Args, Subcommand};

use crate::{
    confluence::tools as tool_args,
    context::AppContext,
    operations::{self, OperationError, OperationResult},
};

#[derive(Debug, Clone, PartialEq, Eq, Args)]
pub struct ConfluenceArgs {
    #[command(subcommand)]
    pub command: ConfluenceCommand,
}

#[derive(Debug, Clone, PartialEq, Eq, Subcommand)]
pub enum ConfluenceCommand {
    Content(ContentArgs),
    Page(PageArgs),
    User(UserArgs),
    Attachment(AttachmentArgs),
}

#[derive(Debug, Clone, PartialEq, Eq, Args)]
pub struct ContentArgs {
    #[command(subcommand)]
    pub command: ContentCommand,
}

#[derive(Debug, Clone, PartialEq, Eq, Subcommand)]
pub enum ContentCommand {
    Search(SearchContentArgs),
    Label(ContentLabelArgs),
}

#[derive(Debug, Clone, PartialEq, Eq, Args)]
pub struct SearchContentArgs {
    #[arg(long)]
    pub query: String,
    #[arg(long)]
    pub limit: Option<u32>,
    #[arg(long)]
    pub spaces: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Args)]
pub struct ContentLabelArgs {
    #[command(subcommand)]
    pub command: ContentLabelCommand,
}

#[derive(Debug, Clone, PartialEq, Eq, Subcommand)]
pub enum ContentLabelCommand {
    List(ListContentLabelsArgs),
    Add(AddContentLabelArgs),
}

#[derive(Debug, Clone, PartialEq, Eq, Args)]
pub struct ListContentLabelsArgs {
    pub content_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Args)]
pub struct AddContentLabelArgs {
    pub content_id: String,
    pub label: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Args)]
pub struct PageArgs {
    #[command(subcommand)]
    pub command: PageCommand,
}

#[derive(Debug, Clone, PartialEq, Eq, Subcommand)]
pub enum PageCommand {
    Get(GetPageArgs),
    Children(ListPageChildrenArgs),
    Tree(GetSpacePageTreeArgs),
    Create(CreatePageArgs),
    Update(UpdatePageArgs),
    Delete(DeletePageArgs),
    Move(MovePageArgs),
    Comment(PageCommentArgs),
    Version(PageVersionArgs),
    Analytics(PageAnalyticsArgs),
}

#[derive(Debug, Clone, PartialEq, Eq, Args)]
#[command(group(clap::ArgGroup::new("page_lookup").args(["id", "space"]).required(true).multiple(false)))]
pub struct GetPageArgs {
    #[arg(long, conflicts_with = "space")]
    pub id: Option<String>,
    #[arg(long, requires = "title")]
    pub space: Option<String>,
    #[arg(long, requires = "space")]
    pub title: Option<String>,
    #[arg(long)]
    pub include_metadata: Option<bool>,
    #[arg(long)]
    pub markdown: Option<bool>,
}

#[derive(Debug, Clone, PartialEq, Eq, Args)]
pub struct ListPageChildrenArgs {
    pub parent_id: String,
    #[arg(long)]
    pub expand: Option<String>,
    #[arg(long)]
    pub limit: Option<u32>,
    #[arg(long)]
    pub include_content: bool,
    #[arg(long)]
    pub markdown: Option<bool>,
    #[arg(long)]
    pub start: Option<u32>,
    #[arg(long)]
    pub include_folders: Option<bool>,
}

#[derive(Debug, Clone, PartialEq, Eq, Args)]
pub struct GetSpacePageTreeArgs {
    pub space_key: String,
    #[arg(long)]
    pub limit: Option<u32>,
}

#[derive(Debug, Clone, PartialEq, Eq, Args)]
#[command(group(clap::ArgGroup::new("content_input").args(["content", "content_file", "content_stdin"]).required(true).multiple(false)))]
pub struct CreatePageArgs {
    #[arg(long)]
    pub space: String,
    #[arg(long)]
    pub title: String,
    #[command(flatten)]
    pub content_input: ContentInput,
    #[arg(long)]
    pub parent_id: Option<String>,
    #[arg(long = "format")]
    pub content_format: Option<String>,
    #[arg(long)]
    pub include_content: bool,
    #[arg(long)]
    pub emoji: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Args)]
#[command(group(clap::ArgGroup::new("content_input").args(["content", "content_file", "content_stdin"]).required(true).multiple(false)))]
pub struct UpdatePageArgs {
    pub page_id: String,
    #[arg(long)]
    pub title: String,
    #[command(flatten)]
    pub content_input: ContentInput,
    #[arg(long)]
    pub minor_edit: Option<bool>,
    #[arg(long)]
    pub version_comment: Option<String>,
    #[arg(long)]
    pub parent_id: Option<String>,
    #[arg(long = "format")]
    pub content_format: Option<String>,
    #[arg(long)]
    pub include_content: bool,
    #[arg(long)]
    pub emoji: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Args)]
pub struct DeletePageArgs {
    pub page_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Args)]
pub struct MovePageArgs {
    pub page_id: String,
    #[arg(long)]
    pub target_parent_id: Option<String>,
    #[arg(long)]
    pub target_space: Option<String>,
    #[arg(long)]
    pub position: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Args)]
pub struct PageCommentArgs {
    #[command(subcommand)]
    pub command: PageCommentCommand,
}

#[derive(Debug, Clone, PartialEq, Eq, Subcommand)]
pub enum PageCommentCommand {
    List(ListPageCommentsArgs),
    Add(AddPageCommentArgs),
    Reply(ReplyPageCommentArgs),
}

#[derive(Debug, Clone, PartialEq, Eq, Args)]
pub struct ListPageCommentsArgs {
    pub page_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Args)]
#[command(group(clap::ArgGroup::new("body_input").args(["body", "body_file", "body_stdin"]).required(true).multiple(false)))]
pub struct AddPageCommentArgs {
    pub page_id: String,
    #[command(flatten)]
    pub body_input: BodyInput,
}

#[derive(Debug, Clone, PartialEq, Eq, Args)]
#[command(group(clap::ArgGroup::new("body_input").args(["body", "body_file", "body_stdin"]).required(true).multiple(false)))]
pub struct ReplyPageCommentArgs {
    pub page_id: String,
    pub comment_id: String,
    #[command(flatten)]
    pub body_input: BodyInput,
}

#[derive(Debug, Clone, PartialEq, Eq, Args)]
pub struct PageVersionArgs {
    #[command(subcommand)]
    pub command: PageVersionCommand,
}

#[derive(Debug, Clone, PartialEq, Eq, Subcommand)]
pub enum PageVersionCommand {
    Get(GetPageVersionArgs),
    Diff(GetPageDiffArgs),
}

#[derive(Debug, Clone, PartialEq, Eq, Args)]
pub struct GetPageVersionArgs {
    pub page_id: String,
    pub version: u32,
    #[arg(long)]
    pub markdown: Option<bool>,
}

#[derive(Debug, Clone, PartialEq, Eq, Args)]
pub struct GetPageDiffArgs {
    pub page_id: String,
    #[arg(long)]
    pub from: u32,
    #[arg(long)]
    pub to: u32,
    #[arg(long)]
    pub context_lines: Option<u32>,
}

#[derive(Debug, Clone, PartialEq, Eq, Args)]
pub struct PageAnalyticsArgs {
    #[command(subcommand)]
    pub command: PageAnalyticsCommand,
}

#[derive(Debug, Clone, PartialEq, Eq, Subcommand)]
pub enum PageAnalyticsCommand {
    Views(GetPageAnalyticsViewsArgs),
}

#[derive(Debug, Clone, PartialEq, Eq, Args)]
pub struct GetPageAnalyticsViewsArgs {
    pub page_id: String,
    #[arg(long)]
    pub from: Option<String>,
    #[arg(long)]
    pub to: Option<String>,
    #[arg(long)]
    pub include_title: Option<bool>,
}

#[derive(Debug, Clone, PartialEq, Eq, Args)]
pub struct UserArgs {
    #[command(subcommand)]
    pub command: UserCommand,
}

#[derive(Debug, Clone, PartialEq, Eq, Subcommand)]
pub enum UserCommand {
    Search(SearchUsersArgs),
}

#[derive(Debug, Clone, PartialEq, Eq, Args)]
pub struct SearchUsersArgs {
    #[arg(long)]
    pub query: String,
    #[arg(long)]
    pub limit: Option<u32>,
    #[arg(long)]
    pub group: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Args)]
pub struct AttachmentArgs {
    #[command(subcommand)]
    pub command: AttachmentCommand,
}

#[derive(Debug, Clone, PartialEq, Eq, Subcommand)]
pub enum AttachmentCommand {
    Upload(UploadAttachmentArgs),
    #[command(name = "upload-batch")]
    UploadBatch(UploadAttachmentsArgs),
    List(ListAttachmentsArgs),
    Download(DownloadAttachmentArgs),
    #[command(name = "download-content")]
    DownloadContent(DownloadContentAttachmentsArgs),
    Delete(DeleteAttachmentArgs),
    Images(GetImageAttachmentsArgs),
}

#[derive(Debug, Clone, PartialEq, Eq, Args)]
pub struct UploadAttachmentArgs {
    pub content_id: String,
    pub file_path: PathBuf,
    #[arg(long)]
    pub comment: Option<String>,
    #[arg(long)]
    pub minor_edit: Option<bool>,
}

#[derive(Debug, Clone, PartialEq, Eq, Args)]
pub struct UploadAttachmentsArgs {
    pub content_id: String,
    #[arg(long)]
    pub files: String,
    #[arg(long)]
    pub comment: Option<String>,
    #[arg(long)]
    pub minor_edit: Option<bool>,
}

#[derive(Debug, Clone, PartialEq, Eq, Args)]
pub struct ListAttachmentsArgs {
    pub content_id: String,
    #[arg(long)]
    pub filename_contains: Option<String>,
    #[arg(long)]
    pub media_type: Option<String>,
    #[arg(long)]
    pub start: Option<u32>,
    #[arg(long)]
    pub limit: Option<u32>,
}

#[derive(Debug, Clone, PartialEq, Eq, Args)]
pub struct DownloadAttachmentArgs {
    pub attachment_id: String,
    #[arg(long)]
    pub max_bytes: Option<usize>,
}

#[derive(Debug, Clone, PartialEq, Eq, Args)]
pub struct DownloadContentAttachmentsArgs {
    pub content_id: String,
    #[arg(long)]
    pub filename_contains: Option<String>,
    #[arg(long)]
    pub media_type: Option<String>,
    #[arg(long)]
    pub max_bytes: Option<usize>,
    #[arg(long)]
    pub limit: Option<u32>,
}

#[derive(Debug, Clone, PartialEq, Eq, Args)]
pub struct DeleteAttachmentArgs {
    pub attachment_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Args)]
pub struct GetImageAttachmentsArgs {
    pub content_id: String,
    #[arg(long)]
    pub max_bytes: Option<usize>,
}

#[derive(Debug, Clone, PartialEq, Eq, Args)]
pub struct ContentInput {
    #[arg(long)]
    pub content: Option<String>,
    #[arg(long)]
    pub content_file: Option<PathBuf>,
    #[arg(long)]
    pub content_stdin: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Args)]
pub struct BodyInput {
    #[arg(long)]
    pub body: Option<String>,
    #[arg(long)]
    pub body_file: Option<PathBuf>,
    #[arg(long)]
    pub body_stdin: bool,
}

pub async fn execute(
    args: ConfluenceArgs,
    context: &AppContext,
) -> Result<OperationResult, OperationError> {
    match args.command {
        ConfluenceCommand::Content(args) => execute_content(args, context).await,
        ConfluenceCommand::Page(args) => execute_page(args, context).await,
        ConfluenceCommand::User(args) => execute_user(args, context).await,
        ConfluenceCommand::Attachment(args) => execute_attachment(args, context).await,
    }
}

async fn execute_content(
    args: ContentArgs,
    context: &AppContext,
) -> Result<OperationResult, OperationError> {
    match args.command {
        ContentCommand::Search(args) => {
            operations::confluence::search_content(
                context,
                tool_args::ConfluenceSearchArgs {
                    query: args.query,
                    limit: args.limit.map(u64::from),
                    spaces_filter: args.spaces,
                },
            )
            .await
        }
        ContentCommand::Label(args) => match args.command {
            ContentLabelCommand::List(args) => {
                operations::confluence::list_content_labels(
                    context,
                    tool_args::ConfluenceListContentLabelsArgs {
                        page_id: args.content_id,
                    },
                )
                .await
            }
            ContentLabelCommand::Add(args) => {
                operations::confluence::add_content_label(
                    context,
                    tool_args::ConfluenceAddLabelArgs {
                        page_id: args.content_id,
                        name: args.label,
                    },
                )
                .await
            }
        },
    }
}

async fn execute_page(
    args: PageArgs,
    context: &AppContext,
) -> Result<OperationResult, OperationError> {
    match args.command {
        PageCommand::Get(args) => {
            operations::confluence::get_page(
                context,
                tool_args::ConfluenceGetPageArgs {
                    page_id: args.id,
                    title: args.title,
                    space_key: args.space,
                    include_metadata: args.include_metadata,
                    convert_to_markdown: args.markdown,
                },
            )
            .await
        }
        PageCommand::Children(args) => {
            operations::confluence::list_page_children(
                context,
                tool_args::ConfluenceListPageChildrenArgs {
                    parent_id: args.parent_id,
                    expand: args.expand,
                    limit: args.limit.map(u64::from),
                    include_content: Some(args.include_content),
                    convert_to_markdown: args.markdown,
                    start: args.start.map(u64::from),
                    include_folders: args.include_folders,
                },
            )
            .await
        }
        PageCommand::Tree(args) => {
            operations::confluence::get_space_page_tree(
                context,
                tool_args::ConfluenceGetSpacePageTreeArgs {
                    space_key: args.space_key,
                    limit: args.limit.map(u64::from),
                },
            )
            .await
        }
        PageCommand::Create(args) => {
            let content = read_content(args.content_input)?;
            operations::confluence::create_page(
                context,
                tool_args::ConfluenceCreatePageArgs {
                    space_key: args.space,
                    title: args.title,
                    content,
                    parent_id: args.parent_id,
                    content_format: args.content_format,
                    include_content: Some(args.include_content),
                    emoji: args.emoji,
                },
            )
            .await
        }
        PageCommand::Update(args) => {
            let content = read_content(args.content_input)?;
            operations::confluence::update_page(
                context,
                tool_args::ConfluenceUpdatePageArgs {
                    page_id: args.page_id,
                    title: args.title,
                    content,
                    is_minor_edit: args.minor_edit,
                    version_comment: args.version_comment,
                    parent_id: args.parent_id,
                    content_format: args.content_format,
                    include_content: Some(args.include_content),
                    emoji: args.emoji,
                },
            )
            .await
        }
        PageCommand::Delete(args) => {
            operations::confluence::delete_page(
                context,
                tool_args::ConfluenceDeletePageArgs {
                    page_id: args.page_id,
                },
            )
            .await
        }
        PageCommand::Move(args) => {
            operations::confluence::move_page(
                context,
                tool_args::ConfluenceMovePageArgs {
                    page_id: args.page_id,
                    target_parent_id: args.target_parent_id,
                    target_space_key: args.target_space,
                    position: args.position,
                },
            )
            .await
        }
        PageCommand::Comment(args) => execute_page_comment(args, context).await,
        PageCommand::Version(args) => execute_page_version(args, context).await,
        PageCommand::Analytics(args) => execute_page_analytics(args, context).await,
    }
}

async fn execute_page_comment(
    args: PageCommentArgs,
    context: &AppContext,
) -> Result<OperationResult, OperationError> {
    match args.command {
        PageCommentCommand::List(args) => {
            operations::confluence::list_page_comments(
                context,
                tool_args::ConfluenceListPageCommentsArgs {
                    page_id: args.page_id,
                },
            )
            .await
        }
        PageCommentCommand::Add(args) => {
            let body = read_body(args.body_input)?;
            operations::confluence::add_page_comment(
                context,
                tool_args::ConfluenceAddCommentArgs {
                    page_id: args.page_id,
                    body,
                },
            )
            .await
        }
        PageCommentCommand::Reply(args) => {
            let body = read_body(args.body_input)?;
            operations::confluence::reply_to_comment(
                context,
                tool_args::ConfluenceReplyToCommentArgs {
                    comment_id: args.comment_id,
                    body,
                },
            )
            .await
        }
    }
}

async fn execute_page_version(
    args: PageVersionArgs,
    context: &AppContext,
) -> Result<OperationResult, OperationError> {
    match args.command {
        PageVersionCommand::Get(args) => {
            operations::confluence::get_page_version(
                context,
                tool_args::ConfluenceGetPageVersionArgs {
                    page_id: args.page_id,
                    version: u64::from(args.version),
                    convert_to_markdown: args.markdown,
                },
            )
            .await
        }
        PageVersionCommand::Diff(args) => {
            operations::confluence::get_page_diff(
                context,
                tool_args::ConfluenceGetPageDiffArgs {
                    page_id: args.page_id,
                    from_version: u64::from(args.from),
                    to_version: u64::from(args.to),
                    context_lines: args.context_lines.map(u64::from),
                },
            )
            .await
        }
    }
}

async fn execute_page_analytics(
    args: PageAnalyticsArgs,
    context: &AppContext,
) -> Result<OperationResult, OperationError> {
    match args.command {
        PageAnalyticsCommand::Views(args) => {
            operations::confluence::get_page_view_analytics(
                context,
                tool_args::ConfluenceGetPageViewAnalyticsArgs {
                    page_id: args.page_id,
                    include_title: args.include_title,
                    from_date: args.from,
                    to_date: args.to,
                },
            )
            .await
        }
    }
}

async fn execute_user(
    args: UserArgs,
    context: &AppContext,
) -> Result<OperationResult, OperationError> {
    match args.command {
        UserCommand::Search(args) => {
            operations::confluence::search_users(
                context,
                tool_args::ConfluenceSearchUserArgs {
                    query: args.query,
                    limit: args.limit.map(u64::from),
                    group_name: args.group,
                },
            )
            .await
        }
    }
}

async fn execute_attachment(
    args: AttachmentArgs,
    context: &AppContext,
) -> Result<OperationResult, OperationError> {
    match args.command {
        AttachmentCommand::Upload(args) => {
            operations::confluence::upload_content_attachment(
                context,
                tool_args::ConfluenceUploadContentAttachmentArgs {
                    content_id: args.content_id,
                    file_path: path_to_string(args.file_path),
                    comment: args.comment,
                    minor_edit: args.minor_edit,
                },
            )
            .await
        }
        AttachmentCommand::UploadBatch(args) => {
            operations::confluence::upload_content_attachments(
                context,
                tool_args::ConfluenceUploadContentAttachmentsArgs {
                    content_id: args.content_id,
                    file_paths: args.files,
                    comment: args.comment,
                    minor_edit: args.minor_edit,
                },
            )
            .await
        }
        AttachmentCommand::List(args) => {
            operations::confluence::list_content_attachments(
                context,
                tool_args::ConfluenceListContentAttachmentsArgs {
                    content_id: args.content_id,
                    start: args.start.map(u64::from),
                    limit: args.limit.map(u64::from),
                    filename: args.filename_contains,
                    media_type: args.media_type,
                },
            )
            .await
        }
        AttachmentCommand::Download(args) => {
            operations::confluence::download_attachment(
                context,
                tool_args::ConfluenceDownloadAttachmentArgs {
                    attachment_id: args.attachment_id,
                    max_bytes: optional_usize_to_u64(args.max_bytes, "max_bytes")?,
                },
            )
            .await
        }
        AttachmentCommand::DownloadContent(args) => {
            operations::confluence::download_content_attachments(
                context,
                tool_args::ConfluenceDownloadContentAttachmentsArgs {
                    content_id: args.content_id,
                    filename: args.filename_contains,
                    media_type: args.media_type,
                    max_bytes: optional_usize_to_u64(args.max_bytes, "max_bytes")?,
                    limit: args.limit.map(u64::from),
                },
            )
            .await
        }
        AttachmentCommand::Delete(args) => {
            operations::confluence::delete_attachment(
                context,
                tool_args::ConfluenceDeleteAttachmentArgs {
                    attachment_id: args.attachment_id,
                },
            )
            .await
        }
        AttachmentCommand::Images(args) => {
            operations::confluence::get_content_image_attachments(
                context,
                tool_args::ConfluenceGetContentImageAttachmentsArgs {
                    content_id: args.content_id,
                    max_bytes: optional_usize_to_u64(args.max_bytes, "max_bytes")?,
                },
            )
            .await
        }
    }
}

fn read_content(input: ContentInput) -> Result<String, OperationError> {
    read_required_text(
        input.content,
        input.content_file,
        input.content_stdin,
        "content",
    )
}

fn read_body(input: BodyInput) -> Result<String, OperationError> {
    read_required_text(input.body, input.body_file, input.body_stdin, "body")
}

fn read_required_text(
    inline: Option<String>,
    file: Option<PathBuf>,
    stdin: bool,
    field_name: &'static str,
) -> Result<String, OperationError> {
    if let Some(value) = inline {
        return Ok(value);
    }
    if let Some(path) = file {
        return read_text_file(&path, field_name);
    }
    if stdin {
        return read_stdin(field_name);
    }
    Err(OperationError::invalid_input(format!(
        "{field_name} is required"
    )))
}

fn read_text_file(path: &PathBuf, field_name: &'static str) -> Result<String, OperationError> {
    fs::read_to_string(path).map_err(|error| {
        OperationError::invalid_input(format!(
            "failed to read {field_name} file {}: {error}",
            path.display()
        ))
    })
}

fn read_stdin(field_name: &'static str) -> Result<String, OperationError> {
    let mut buffer = String::new();
    io::stdin().read_to_string(&mut buffer).map_err(|error| {
        OperationError::invalid_input(format!("failed to read {field_name} from stdin: {error}"))
    })?;
    Ok(buffer)
}

fn optional_usize_to_u64(
    value: Option<usize>,
    field_name: &'static str,
) -> Result<Option<u64>, OperationError> {
    value
        .map(|value| {
            u64::try_from(value)
                .map_err(|_| OperationError::invalid_input(format!("{field_name} is too large")))
        })
        .transpose()
}

fn path_to_string(path: PathBuf) -> String {
    path.display().to_string()
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;

    use super::{ConfluenceArgs, execute};
    use crate::{
        cli::{ProviderCommand, parse_cli_args},
        config::{HttpConfig, RuntimeConfig},
        confluence::{
            config::{ConfluenceConfig, ConfluenceDeployment},
            tools::{CONFLUENCE_DOWNLOAD_ATTACHMENT_TOOL_NAME, CONFLUENCE_SEARCH_TOOL_NAME},
        },
        context::AppContext,
        operations::OperationErrorCategory,
        tool_registry,
        upstream::{auth::UpstreamAuth, custom_headers::CustomHeaders, proxy::ProxyConfig},
    };

    #[test]
    fn cli_confluence_parses_page_lookup_by_space_and_title() {
        parse_cli_args([
            "confluence",
            "page",
            "get",
            "--space",
            "ENG",
            "--title",
            "Roadmap",
            "--markdown",
            "true",
        ])
        .unwrap();
    }

    #[test]
    fn cli_confluence_parses_attachment_download_filters() {
        parse_cli_args([
            "confluence",
            "attachment",
            "download-content",
            "123",
            "--filename-contains",
            "diagram",
            "--max-bytes",
            "4096",
        ])
        .unwrap();
    }

    #[test]
    fn cli_confluence_parses_all_resource_commands() {
        let cases: &[&[&str]] = &[
            &["confluence", "content", "search", "--query", "roadmap"],
            &["confluence", "page", "get", "--id", "123"],
            &["confluence", "page", "children", "123"],
            &["confluence", "page", "tree", "ENG"],
            &[
                "confluence",
                "page",
                "create",
                "--space",
                "ENG",
                "--title",
                "Roadmap",
                "--content",
                "# Roadmap",
            ],
            &[
                "confluence",
                "page",
                "update",
                "123",
                "--title",
                "Roadmap",
                "--content",
                "# Updated",
            ],
            &["confluence", "page", "delete", "123"],
            &[
                "confluence",
                "page",
                "move",
                "123",
                "--target-parent-id",
                "456",
            ],
            &["confluence", "page", "comment", "list", "123"],
            &[
                "confluence",
                "page",
                "comment",
                "add",
                "123",
                "--body",
                "Looks good",
            ],
            &[
                "confluence",
                "page",
                "comment",
                "reply",
                "123",
                "456",
                "--body",
                "Thanks",
            ],
            &["confluence", "content", "label", "list", "123"],
            &["confluence", "content", "label", "add", "123", "decision"],
            &["confluence", "user", "search", "--query", "chen"],
            &["confluence", "page", "version", "get", "123", "2"],
            &[
                "confluence",
                "page",
                "version",
                "diff",
                "123",
                "--from",
                "1",
                "--to",
                "2",
            ],
            &["confluence", "page", "analytics", "views", "123"],
            &["confluence", "attachment", "upload", "123", "./diagram.png"],
            &[
                "confluence",
                "attachment",
                "upload-batch",
                "123",
                "--files",
                "./a.png,./b.png",
            ],
            &["confluence", "attachment", "list", "123"],
            &["confluence", "attachment", "download", "456"],
            &["confluence", "attachment", "download-content", "123"],
            &["confluence", "attachment", "delete", "456"],
            &["confluence", "attachment", "images", "123"],
        ];

        for case in cases {
            parse_cli_args(*case).unwrap_or_else(|error| panic!("{case:?}: {error}"));
        }
    }

    #[tokio::test]
    async fn cli_confluence_ignores_mcp_disabled_tools() {
        let error = execute(
            confluence_args([
                "confluence",
                "content",
                "search",
                "--query",
                "",
                "--limit",
                "1",
            ]),
            &confluence_context(BTreeSet::from([CONFLUENCE_SEARCH_TOOL_NAME.to_string()])),
        )
        .await
        .unwrap_err();

        assert_eq!(error.category, OperationErrorCategory::InvalidInput);
        assert!(error.message.contains("query must not be empty"));
    }

    #[tokio::test]
    async fn cli_confluence_executes_page_write_with_content_file_input() {
        let path = std::env::temp_dir().join(format!(
            "workhub-cli-confluence-content-{}.md",
            std::process::id()
        ));
        std::fs::write(&path, "# Roadmap").unwrap();

        let error = execute(
            confluence_args(vec![
                "confluence".to_string(),
                "page".to_string(),
                "update".to_string(),
                "123".to_string(),
                "--title".to_string(),
                "Roadmap".to_string(),
                "--content-file".to_string(),
                path.display().to_string(),
                "--format".to_string(),
                "html".to_string(),
            ]),
            &confluence_context(BTreeSet::new()),
        )
        .await
        .unwrap_err();
        let _ = std::fs::remove_file(path);

        assert_eq!(error.category, OperationErrorCategory::InvalidInput);
        assert!(
            error
                .message
                .contains("content_format must be markdown, wiki, or storage")
        );
    }

    #[tokio::test]
    async fn cli_confluence_executes_comment_and_label_validation() {
        let comment_error = execute(
            confluence_args(["confluence", "page", "comment", "add", "123", "--body", ""]),
            &confluence_context(BTreeSet::new()),
        )
        .await
        .unwrap_err();
        let label_error = execute(
            confluence_args(["confluence", "content", "label", "add", "123", ""]),
            &confluence_context(BTreeSet::new()),
        )
        .await
        .unwrap_err();

        assert_eq!(comment_error.category, OperationErrorCategory::InvalidInput);
        assert!(comment_error.message.contains("body must not be empty"));
        assert_eq!(label_error.category, OperationErrorCategory::InvalidInput);
        assert!(label_error.message.contains("name must not be empty"));
    }

    #[tokio::test]
    async fn cli_confluence_executes_attachment_preflight_validation() {
        let read_error = execute(
            confluence_args([
                "confluence",
                "attachment",
                "download",
                "456",
                "--max-bytes",
                "0",
            ]),
            &confluence_context(BTreeSet::new()),
        )
        .await
        .unwrap_err();
        let write_error = execute(
            confluence_args([
                "confluence",
                "attachment",
                "upload-batch",
                "123",
                "--files",
                " , ",
            ]),
            &confluence_context(BTreeSet::new()),
        )
        .await
        .unwrap_err();

        assert_eq!(read_error.category, OperationErrorCategory::InvalidInput);
        assert!(read_error.message.contains("max_bytes must be positive"));
        assert_eq!(write_error.category, OperationErrorCategory::InvalidInput);
        assert!(
            write_error
                .message
                .contains("file_paths must contain at least one local file path")
        );
    }

    #[tokio::test]
    async fn cli_confluence_attachment_preflight_ignores_mcp_disabled_tools() {
        let error = execute(
            confluence_args([
                "confluence",
                "attachment",
                "download",
                "456",
                "--max-bytes",
                "0",
            ]),
            &confluence_context(BTreeSet::from([
                CONFLUENCE_DOWNLOAD_ATTACHMENT_TOOL_NAME.to_string()
            ])),
        )
        .await
        .unwrap_err();

        assert_eq!(error.category, OperationErrorCategory::InvalidInput);
        assert!(error.message.contains("max_bytes must be positive"));
    }

    #[tokio::test]
    async fn cli_confluence_search_uses_shared_operation_validation() {
        let error = execute(
            confluence_args([
                "confluence",
                "content",
                "search",
                "--query",
                "",
                "--limit",
                "1",
            ]),
            &confluence_context(BTreeSet::from([CONFLUENCE_SEARCH_TOOL_NAME.to_string()])),
        )
        .await
        .unwrap_err();

        assert_eq!(error.category, OperationErrorCategory::InvalidInput);
        assert!(error.message.contains("query must not be empty"));
    }

    fn confluence_args<I, S>(args: I) -> ConfluenceArgs
    where
        I: IntoIterator<Item = S>,
        S: Into<std::ffi::OsString> + Clone,
    {
        match parse_cli_args(args).unwrap().command {
            ProviderCommand::Confluence(args) => args,
            _ => panic!("expected Confluence args"),
        }
    }

    fn confluence_context(mcp_disabled_tools: BTreeSet<String>) -> AppContext {
        AppContext::from_config(&RuntimeConfig {
            confluence: Some(ConfluenceConfig {
                base_url: "https://confluence.example".to_string(),
                deployment: ConfluenceDeployment::ServerDataCenter,
                auth: UpstreamAuth::Pat {
                    personal_token: "test-pat-value".to_string(),
                },
                ssl_verify: true,
                proxy: ProxyConfig::default(),
                custom_headers: CustomHeaders::default(),
                mtls: None,
                spaces_filter: BTreeSet::new(),
                timeout_seconds: 75,
            }),
            mcp_disabled_tools,
            mcp_enabled_toolsets: tool_registry::all_toolsets(),
            http: HttpConfig::default(),
            ..RuntimeConfig::default()
        })
    }
}

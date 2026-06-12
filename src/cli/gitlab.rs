use std::{
    fs,
    io::{self, Read},
    path::PathBuf,
};

use clap::{Args, Subcommand};

use crate::{
    context::AppContext,
    gitlab::tools as tool_args,
    operations::{self, OperationError, OperationResult},
};

#[derive(Debug, Clone, PartialEq, Eq, Args)]
pub struct GitlabArgs {
    #[command(subcommand)]
    pub command: GitlabCommand,
}

#[derive(Debug, Clone, PartialEq, Eq, Subcommand)]
pub enum GitlabCommand {
    User(UserArgs),
    Project(ProjectArgs),
    #[command(name = "mr")]
    MergeRequest(MergeRequestArgs),
}

#[derive(Debug, Clone, PartialEq, Eq, Args)]
pub struct UserArgs {
    #[command(subcommand)]
    pub command: UserCommand,
}

#[derive(Debug, Clone, PartialEq, Eq, Subcommand)]
pub enum UserCommand {
    Current,
}

#[derive(Debug, Clone, PartialEq, Eq, Args)]
pub struct ProjectArgs {
    #[command(subcommand)]
    pub command: ProjectCommand,
}

#[derive(Debug, Clone, PartialEq, Eq, Subcommand)]
pub enum ProjectCommand {
    Get(GetProjectArgs),
}

#[derive(Debug, Clone, PartialEq, Eq, Args)]
pub struct GetProjectArgs {
    pub project: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Args)]
pub struct MergeRequestArgs {
    #[command(subcommand)]
    pub command: MergeRequestCommand,
}

#[derive(Debug, Clone, PartialEq, Eq, Subcommand)]
pub enum MergeRequestCommand {
    List(ListMergeRequestsArgs),
    Get(GetMergeRequestArgs),
    Commits(ListMergeRequestCommitsArgs),
    Diffs(ListMergeRequestDiffsArgs),
    Pipelines(ListMergeRequestPipelinesArgs),
    Create(CreateMergeRequestArgs),
    Update(UpdateMergeRequestArgs),
    Note(MergeRequestNoteArgs),
    Discussion(MergeRequestDiscussionArgs),
    Approval(MergeRequestApprovalArgs),
    Merge(AcceptMergeRequestArgs),
}

#[derive(Debug, Clone, PartialEq, Eq, Args)]
pub struct ListMergeRequestsArgs {
    pub project: String,
    #[arg(long)]
    pub state: Option<String>,
    #[arg(long)]
    pub author: Option<String>,
    #[arg(long)]
    pub reviewer: Option<String>,
    #[arg(long)]
    pub source_branch: Option<String>,
    #[arg(long)]
    pub target_branch: Option<String>,
    #[arg(long)]
    pub labels: Option<String>,
    #[arg(long)]
    pub page: Option<u32>,
    #[arg(long)]
    pub per_page: Option<u32>,
}

#[derive(Debug, Clone, PartialEq, Eq, Args)]
pub struct GetMergeRequestArgs {
    pub project: String,
    pub iid: String,
    #[arg(long)]
    pub include_diverged_commits_count: Option<bool>,
    #[arg(long)]
    pub include_rebase_in_progress: Option<bool>,
}

#[derive(Debug, Clone, PartialEq, Eq, Args)]
pub struct ListMergeRequestCommitsArgs {
    pub project: String,
    pub iid: String,
    #[arg(long)]
    pub page: Option<u32>,
    #[arg(long)]
    pub per_page: Option<u32>,
}

#[derive(Debug, Clone, PartialEq, Eq, Args)]
pub struct ListMergeRequestDiffsArgs {
    pub project: String,
    pub iid: String,
    #[arg(long)]
    pub max_diff_bytes: Option<usize>,
    #[arg(long)]
    pub page: Option<u32>,
    #[arg(long)]
    pub per_page: Option<u32>,
}

#[derive(Debug, Clone, PartialEq, Eq, Args)]
pub struct ListMergeRequestPipelinesArgs {
    pub project: String,
    pub iid: String,
    #[arg(long)]
    pub page: Option<u32>,
    #[arg(long)]
    pub per_page: Option<u32>,
}

#[derive(Debug, Clone, PartialEq, Eq, Args)]
#[command(group(clap::ArgGroup::new("description_input").args(["description", "description_file", "description_stdin"]).multiple(false)))]
pub struct CreateMergeRequestArgs {
    pub project: String,
    #[arg(long)]
    pub source: String,
    #[arg(long)]
    pub target: String,
    #[arg(long)]
    pub title: String,
    #[command(flatten)]
    pub description_input: DescriptionInput,
    #[arg(long)]
    pub remove_source_branch: Option<bool>,
    #[arg(long)]
    pub squash: Option<bool>,
    #[arg(long)]
    pub assignee_ids: Option<String>,
    #[arg(long)]
    pub reviewer_ids: Option<String>,
    #[arg(long)]
    pub labels: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Args)]
#[command(group(clap::ArgGroup::new("description_input").args(["description", "description_file", "description_stdin"]).multiple(false)))]
pub struct UpdateMergeRequestArgs {
    pub project: String,
    pub iid: String,
    #[arg(long)]
    pub title: Option<String>,
    #[command(flatten)]
    pub description_input: DescriptionInput,
    #[arg(long)]
    pub state_event: Option<String>,
    #[arg(long)]
    pub labels: Option<String>,
    #[arg(long)]
    pub add_labels: Option<String>,
    #[arg(long)]
    pub remove_labels: Option<String>,
    #[arg(long)]
    pub reviewer_ids: Option<String>,
    #[arg(long)]
    pub assignee_ids: Option<String>,
    #[arg(long)]
    pub target_branch: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Args)]
pub struct MergeRequestNoteArgs {
    #[command(subcommand)]
    pub command: MergeRequestNoteCommand,
}

#[derive(Debug, Clone, PartialEq, Eq, Subcommand)]
pub enum MergeRequestNoteCommand {
    Add(AddMergeRequestNoteArgs),
}

#[derive(Debug, Clone, PartialEq, Eq, Args)]
#[command(group(clap::ArgGroup::new("body_input").args(["body", "body_file", "body_stdin"]).required(true).multiple(false)))]
pub struct AddMergeRequestNoteArgs {
    pub project: String,
    pub iid: String,
    #[command(flatten)]
    pub body_input: BodyInput,
}

#[derive(Debug, Clone, PartialEq, Eq, Args)]
pub struct MergeRequestDiscussionArgs {
    #[command(subcommand)]
    pub command: MergeRequestDiscussionCommand,
}

#[derive(Debug, Clone, PartialEq, Eq, Subcommand)]
pub enum MergeRequestDiscussionCommand {
    Reply(ReplyMergeRequestDiscussionArgs),
    Resolve(ResolveMergeRequestDiscussionArgs),
}

#[derive(Debug, Clone, PartialEq, Eq, Args)]
#[command(group(clap::ArgGroup::new("body_input").args(["body", "body_file", "body_stdin"]).required(true).multiple(false)))]
pub struct ReplyMergeRequestDiscussionArgs {
    pub project: String,
    pub iid: String,
    pub discussion_id: String,
    #[command(flatten)]
    pub body_input: BodyInput,
}

#[derive(Debug, Clone, PartialEq, Eq, Args)]
pub struct ResolveMergeRequestDiscussionArgs {
    pub project: String,
    pub iid: String,
    pub discussion_id: String,
    #[arg(long, action = clap::ArgAction::Set)]
    pub resolved: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Args)]
pub struct MergeRequestApprovalArgs {
    #[command(subcommand)]
    pub command: MergeRequestApprovalCommand,
}

#[derive(Debug, Clone, PartialEq, Eq, Subcommand)]
pub enum MergeRequestApprovalCommand {
    Get(GetMergeRequestApprovalArgs),
    Set(SetMergeRequestApprovalArgs),
}

#[derive(Debug, Clone, PartialEq, Eq, Args)]
pub struct GetMergeRequestApprovalArgs {
    pub project: String,
    pub iid: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Args)]
pub struct SetMergeRequestApprovalArgs {
    pub project: String,
    pub iid: String,
    #[arg(long)]
    pub action: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Args)]
pub struct AcceptMergeRequestArgs {
    pub project: String,
    pub iid: String,
    #[arg(long)]
    pub sha: String,
    #[arg(long)]
    pub auto_merge: Option<bool>,
    #[arg(long)]
    pub squash: Option<bool>,
    #[arg(long)]
    pub remove_source_branch: Option<bool>,
    #[arg(long)]
    pub merge_commit_message: Option<String>,
    #[arg(long)]
    pub squash_commit_message: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Args)]
pub struct DescriptionInput {
    #[arg(long)]
    pub description: Option<String>,
    #[arg(long)]
    pub description_file: Option<PathBuf>,
    #[arg(long)]
    pub description_stdin: bool,
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
    args: GitlabArgs,
    context: &AppContext,
) -> Result<OperationResult, OperationError> {
    match args.command {
        GitlabCommand::User(args) => execute_user(args, context).await,
        GitlabCommand::Project(args) => execute_project(args, context).await,
        GitlabCommand::MergeRequest(args) => execute_merge_request(args, context).await,
    }
}

async fn execute_user(
    args: UserArgs,
    context: &AppContext,
) -> Result<OperationResult, OperationError> {
    match args.command {
        UserCommand::Current => {
            operations::gitlab::get_current_user(context, tool_args::GitlabGetCurrentUserArgs {})
                .await
        }
    }
}

async fn execute_project(
    args: ProjectArgs,
    context: &AppContext,
) -> Result<OperationResult, OperationError> {
    match args.command {
        ProjectCommand::Get(args) => {
            operations::gitlab::get_project(
                context,
                tool_args::GitlabGetProjectArgs {
                    project: args.project,
                },
            )
            .await
        }
    }
}

async fn execute_merge_request(
    args: MergeRequestArgs,
    context: &AppContext,
) -> Result<OperationResult, OperationError> {
    match args.command {
        MergeRequestCommand::List(args) => {
            operations::gitlab::list_merge_requests(
                context,
                tool_args::GitlabListMergeRequestsArgs {
                    project: args.project,
                    state: args.state,
                    author_username: args.author,
                    reviewer_username: args.reviewer,
                    source_branch: args.source_branch,
                    target_branch: args.target_branch,
                    labels: optional_string_list(args.labels),
                    page: args.page.map(u64::from),
                    per_page: args.per_page.map(u64::from),
                },
            )
            .await
        }
        MergeRequestCommand::Get(args) => {
            operations::gitlab::get_merge_request(
                context,
                tool_args::GitlabGetMergeRequestArgs {
                    project: args.project,
                    merge_request_iid: parse_u64(&args.iid, "iid")?,
                    include_diverged_commits_count: args.include_diverged_commits_count,
                    include_rebase_in_progress: args.include_rebase_in_progress,
                },
            )
            .await
        }
        MergeRequestCommand::Commits(args) => {
            operations::gitlab::list_merge_request_commits(
                context,
                tool_args::GitlabListMergeRequestCommitsArgs {
                    project: args.project,
                    merge_request_iid: parse_u64(&args.iid, "iid")?,
                    page: args.page.map(u64::from),
                    per_page: args.per_page.map(u64::from),
                },
            )
            .await
        }
        MergeRequestCommand::Diffs(args) => {
            operations::gitlab::list_merge_request_diffs(
                context,
                tool_args::GitlabListMergeRequestDiffsArgs {
                    project: args.project,
                    merge_request_iid: parse_u64(&args.iid, "iid")?,
                    max_diff_bytes: optional_usize_to_u64(args.max_diff_bytes, "max_diff_bytes")?,
                    page: args.page.map(u64::from),
                    per_page: args.per_page.map(u64::from),
                },
            )
            .await
        }
        MergeRequestCommand::Pipelines(args) => {
            operations::gitlab::list_merge_request_pipelines(
                context,
                tool_args::GitlabListMergeRequestPipelinesArgs {
                    project: args.project,
                    merge_request_iid: parse_u64(&args.iid, "iid")?,
                    page: args.page.map(u64::from),
                    per_page: args.per_page.map(u64::from),
                },
            )
            .await
        }
        MergeRequestCommand::Create(args) => {
            let description = read_description(args.description_input)?;
            operations::gitlab::create_merge_request(
                context,
                tool_args::GitlabCreateMergeRequestArgs {
                    project: args.project,
                    source_branch: args.source,
                    target_branch: args.target,
                    title: args.title,
                    description,
                    remove_source_branch: args.remove_source_branch,
                    squash: args.squash,
                    assignee_ids: optional_u64_list(args.assignee_ids, "assignee_ids")?,
                    reviewer_ids: optional_u64_list(args.reviewer_ids, "reviewer_ids")?,
                    labels: optional_string_list(args.labels),
                },
            )
            .await
        }
        MergeRequestCommand::Update(args) => {
            let description = read_description(args.description_input)?;
            operations::gitlab::update_merge_request(
                context,
                tool_args::GitlabUpdateMergeRequestArgs {
                    project: args.project,
                    merge_request_iid: parse_u64(&args.iid, "iid")?,
                    title: args.title,
                    description,
                    state_event: args.state_event,
                    labels: optional_string_list(args.labels),
                    add_labels: optional_string_list(args.add_labels),
                    remove_labels: optional_string_list(args.remove_labels),
                    reviewer_ids: optional_u64_list(args.reviewer_ids, "reviewer_ids")?,
                    assignee_ids: optional_u64_list(args.assignee_ids, "assignee_ids")?,
                    target_branch: args.target_branch,
                },
            )
            .await
        }
        MergeRequestCommand::Note(args) => execute_merge_request_note(args, context).await,
        MergeRequestCommand::Discussion(args) => {
            execute_merge_request_discussion(args, context).await
        }
        MergeRequestCommand::Approval(args) => execute_merge_request_approval(args, context).await,
        MergeRequestCommand::Merge(args) => {
            operations::gitlab::accept_merge_request(
                context,
                tool_args::GitlabAcceptMergeRequestArgs {
                    project: args.project,
                    merge_request_iid: parse_u64(&args.iid, "iid")?,
                    sha: args.sha,
                    auto_merge: args.auto_merge,
                    squash: args.squash,
                    should_remove_source_branch: args.remove_source_branch,
                    merge_commit_message: args.merge_commit_message,
                    squash_commit_message: args.squash_commit_message,
                },
            )
            .await
        }
    }
}

async fn execute_merge_request_note(
    args: MergeRequestNoteArgs,
    context: &AppContext,
) -> Result<OperationResult, OperationError> {
    match args.command {
        MergeRequestNoteCommand::Add(args) => {
            let body = read_body(args.body_input)?;
            operations::gitlab::add_merge_request_note(
                context,
                tool_args::GitlabAddMergeRequestNoteArgs {
                    project: args.project,
                    merge_request_iid: parse_u64(&args.iid, "iid")?,
                    body,
                },
            )
            .await
        }
    }
}

async fn execute_merge_request_discussion(
    args: MergeRequestDiscussionArgs,
    context: &AppContext,
) -> Result<OperationResult, OperationError> {
    match args.command {
        MergeRequestDiscussionCommand::Reply(args) => {
            let body = read_body(args.body_input)?;
            operations::gitlab::reply_merge_request_discussion(
                context,
                tool_args::GitlabReplyMergeRequestDiscussionArgs {
                    project: args.project,
                    merge_request_iid: parse_u64(&args.iid, "iid")?,
                    discussion_id: args.discussion_id,
                    body,
                },
            )
            .await
        }
        MergeRequestDiscussionCommand::Resolve(args) => {
            operations::gitlab::resolve_merge_request_discussion(
                context,
                tool_args::GitlabResolveMergeRequestDiscussionArgs {
                    project: args.project,
                    merge_request_iid: parse_u64(&args.iid, "iid")?,
                    discussion_id: args.discussion_id,
                    resolved: args.resolved,
                },
            )
            .await
        }
    }
}

async fn execute_merge_request_approval(
    args: MergeRequestApprovalArgs,
    context: &AppContext,
) -> Result<OperationResult, OperationError> {
    match args.command {
        MergeRequestApprovalCommand::Get(args) => {
            operations::gitlab::get_merge_request_approval_state(
                context,
                tool_args::GitlabMergeRequestRefArgs {
                    project: args.project,
                    merge_request_iid: parse_u64(&args.iid, "iid")?,
                },
            )
            .await
        }
        MergeRequestApprovalCommand::Set(args) => {
            operations::gitlab::set_merge_request_approval(
                context,
                tool_args::GitlabSetMergeRequestApprovalArgs {
                    project: args.project,
                    merge_request_iid: parse_u64(&args.iid, "iid")?,
                    action: parse_approval_action(&args.action)?,
                },
            )
            .await
        }
    }
}

fn read_description(input: DescriptionInput) -> Result<Option<String>, OperationError> {
    read_optional_text(
        input.description,
        input.description_file,
        input.description_stdin,
        "description",
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
    read_optional_text(inline, file, stdin, field_name)?
        .ok_or_else(|| OperationError::invalid_input(format!("{field_name} is required")))
}

fn read_optional_text(
    inline: Option<String>,
    file: Option<PathBuf>,
    stdin: bool,
    field_name: &'static str,
) -> Result<Option<String>, OperationError> {
    if let Some(value) = inline {
        return Ok(Some(value));
    }
    if let Some(path) = file {
        return read_text_file(&path, field_name).map(Some);
    }
    if stdin {
        return read_stdin(field_name).map(Some);
    }
    Ok(None)
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

fn optional_string_list(value: Option<String>) -> Option<Vec<String>> {
    value.map(|value| {
        value
            .split(',')
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(ToString::to_string)
            .collect()
    })
}

fn optional_u64_list(
    value: Option<String>,
    field_name: &'static str,
) -> Result<Option<Vec<u64>>, OperationError> {
    value
        .map(|value| {
            value
                .split(',')
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .map(|value| parse_u64(value, field_name))
                .collect()
        })
        .transpose()
}

fn parse_approval_action(
    value: &str,
) -> Result<tool_args::GitlabMergeRequestApprovalAction, OperationError> {
    match value {
        "approve" => Ok(tool_args::GitlabMergeRequestApprovalAction::Approve),
        "unapprove" => Ok(tool_args::GitlabMergeRequestApprovalAction::Unapprove),
        _ => Err(OperationError::invalid_input(
            "action must be approve or unapprove",
        )),
    }
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

fn parse_u64(value: &str, field_name: &'static str) -> Result<u64, OperationError> {
    value.parse::<u64>().map_err(|error| {
        OperationError::invalid_input(format!("{field_name} must be an unsigned integer: {error}"))
    })
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;

    use super::{GitlabArgs, execute};
    use crate::{
        cli::{ProviderCommand, parse_cli_args},
        config::{HttpConfig, RuntimeConfig},
        context::AppContext,
        gitlab::{
            config::GitlabConfig,
            tools::{
                GITLAB_ACCEPT_MERGE_REQUEST_TOOL_NAME, GITLAB_LIST_MERGE_REQUEST_DIFFS_TOOL_NAME,
                GITLAB_SET_MERGE_REQUEST_APPROVAL_TOOL_NAME,
            },
        },
        operations::OperationErrorCategory,
        tool_registry,
        upstream::{auth::UpstreamAuth, custom_headers::CustomHeaders, proxy::ProxyConfig},
    };

    #[test]
    fn cli_gitlab_parses_merge_request_list_path() {
        parse_cli_args([
            "gitlab",
            "mr",
            "list",
            "group/project",
            "--state",
            "opened",
            "--labels",
            "backend,urgent",
        ])
        .unwrap();
    }

    #[test]
    fn cli_gitlab_parses_merge_request_merge_path() {
        parse_cli_args([
            "gitlab",
            "mr",
            "merge",
            "group/project",
            "42",
            "--sha",
            "abcdef",
            "--auto-merge",
            "true",
        ])
        .unwrap();
    }

    #[test]
    fn cli_gitlab_parses_all_resource_commands() {
        let cases: &[&[&str]] = &[
            &["gitlab", "user", "current"],
            &["gitlab", "project", "get", "group/project"],
            &["gitlab", "mr", "list", "group/project"],
            &["gitlab", "mr", "get", "group/project", "7"],
            &["gitlab", "mr", "commits", "group/project", "7"],
            &["gitlab", "mr", "diffs", "group/project", "7"],
            &["gitlab", "mr", "pipelines", "group/project", "7"],
            &[
                "gitlab",
                "mr",
                "create",
                "group/project",
                "--source",
                "feature/api",
                "--target",
                "main",
                "--title",
                "Add API",
            ],
            &[
                "gitlab",
                "mr",
                "update",
                "group/project",
                "7",
                "--title",
                "Update API",
            ],
            &[
                "gitlab",
                "mr",
                "note",
                "add",
                "group/project",
                "7",
                "--body",
                "Looks good",
            ],
            &[
                "gitlab",
                "mr",
                "discussion",
                "reply",
                "group/project",
                "7",
                "discussion-1",
                "--body",
                "Reply",
            ],
            &[
                "gitlab",
                "mr",
                "discussion",
                "resolve",
                "group/project",
                "7",
                "discussion-1",
                "--resolved",
                "true",
            ],
            &["gitlab", "mr", "approval", "get", "group/project", "7"],
            &[
                "gitlab",
                "mr",
                "approval",
                "set",
                "group/project",
                "7",
                "--action",
                "approve",
            ],
            &[
                "gitlab",
                "mr",
                "merge",
                "group/project",
                "7",
                "--sha",
                "abc123",
            ],
        ];

        for case in cases {
            parse_cli_args(*case).unwrap_or_else(|error| panic!("{case:?}: {error}"));
        }
    }

    #[tokio::test]
    async fn cli_gitlab_ignores_mcp_disabled_tools() {
        let error = execute(
            gitlab_args([
                "gitlab",
                "mr",
                "diffs",
                "group/project",
                "7",
                "--max-diff-bytes",
                "0",
            ]),
            &gitlab_context(
                BTreeSet::new(),
                BTreeSet::from([GITLAB_LIST_MERGE_REQUEST_DIFFS_TOOL_NAME.to_string()]),
            ),
        )
        .await
        .unwrap_err();

        assert_eq!(error.category, OperationErrorCategory::InvalidInput);
        assert!(error.message.contains("max_diff_bytes must be positive"));
    }

    #[tokio::test]
    async fn cli_gitlab_executes_create_with_description_file_input() {
        let path = std::env::temp_dir().join(format!(
            "workhub-cli-gitlab-description-{}.md",
            std::process::id()
        ));
        std::fs::write(&path, "Implements the endpoint").unwrap();

        let error = execute(
            gitlab_args(vec![
                "gitlab".to_string(),
                "mr".to_string(),
                "create".to_string(),
                "group/project".to_string(),
                "--source".to_string(),
                " ".to_string(),
                "--target".to_string(),
                "main".to_string(),
                "--title".to_string(),
                "Add API".to_string(),
                "--description-file".to_string(),
                path.display().to_string(),
            ]),
            &gitlab_context(BTreeSet::new(), BTreeSet::new()),
        )
        .await
        .unwrap_err();
        let _ = std::fs::remove_file(path);

        assert_eq!(error.category, OperationErrorCategory::InvalidInput);
        assert!(error.message.contains("source_branch must not be empty"));
    }

    #[tokio::test]
    async fn cli_gitlab_executes_note_and_diff_preflight_validation() {
        let note_error = execute(
            gitlab_args([
                "gitlab",
                "mr",
                "note",
                "add",
                "group/project",
                "7",
                "--body",
                "",
            ]),
            &gitlab_context(BTreeSet::new(), BTreeSet::new()),
        )
        .await
        .unwrap_err();
        let diff_error = execute(
            gitlab_args([
                "gitlab",
                "mr",
                "diffs",
                "group/project",
                "7",
                "--max-diff-bytes",
                "0",
            ]),
            &gitlab_context(BTreeSet::new(), BTreeSet::new()),
        )
        .await
        .unwrap_err();

        assert_eq!(note_error.category, OperationErrorCategory::InvalidInput);
        assert!(note_error.message.contains("body must not be empty"));
        assert_eq!(diff_error.category, OperationErrorCategory::InvalidInput);
        assert!(
            diff_error
                .message
                .contains("max_diff_bytes must be positive")
        );
    }

    #[tokio::test]
    async fn cli_gitlab_executes_approval_action_validation() {
        let error = execute(
            gitlab_args([
                "gitlab",
                "mr",
                "approval",
                "set",
                "group/project",
                "7",
                "--action",
                "bad",
            ]),
            &gitlab_context(BTreeSet::new(), BTreeSet::new()),
        )
        .await
        .unwrap_err();

        assert_eq!(error.category, OperationErrorCategory::InvalidInput);
        assert!(
            error
                .message
                .contains("action must be approve or unapprove")
        );
    }

    #[tokio::test]
    async fn cli_gitlab_merge_validation_ignores_mcp_disabled_tools() {
        let error = execute(
            gitlab_args(["gitlab", "mr", "merge", "group/project", "7", "--sha", " "]),
            &gitlab_context(
                BTreeSet::new(),
                BTreeSet::from([GITLAB_ACCEPT_MERGE_REQUEST_TOOL_NAME.to_string()]),
            ),
        )
        .await
        .unwrap_err();

        assert_eq!(error.category, OperationErrorCategory::InvalidInput);
        assert!(error.message.contains("sha must not be empty"));
    }

    #[tokio::test]
    async fn cli_gitlab_approval_validation_ignores_mcp_disabled_tools() {
        let error = execute(
            gitlab_args([
                "gitlab",
                "mr",
                "approval",
                "set",
                "group/project",
                "7",
                "--action",
                "bad",
            ]),
            &gitlab_context(
                BTreeSet::new(),
                BTreeSet::from([GITLAB_SET_MERGE_REQUEST_APPROVAL_TOOL_NAME.to_string()]),
            ),
        )
        .await
        .unwrap_err();

        assert_eq!(error.category, OperationErrorCategory::InvalidInput);
        assert!(
            error
                .message
                .contains("action must be approve or unapprove")
        );
    }

    fn gitlab_args<I, S>(args: I) -> GitlabArgs
    where
        I: IntoIterator<Item = S>,
        S: Into<std::ffi::OsString> + Clone,
    {
        match parse_cli_args(args).unwrap().command {
            ProviderCommand::Gitlab(args) => args,
            _ => panic!("expected GitLab args"),
        }
    }

    fn gitlab_context(
        projects_filter: BTreeSet<String>,
        mcp_disabled_tools: BTreeSet<String>,
    ) -> AppContext {
        AppContext::from_config(&RuntimeConfig {
            gitlab: Some(GitlabConfig {
                base_url: "https://gitlab.example".to_string(),
                auth: UpstreamAuth::HeaderToken {
                    header_name: reqwest::header::HeaderName::from_static("private-token"),
                    token: "gitlab-token".to_string(),
                },
                ssl_verify: true,
                proxy: ProxyConfig::default(),
                custom_headers: CustomHeaders::default(),
                mtls: None,
                projects_filter,
                timeout_seconds: 75,
            }),
            mcp_disabled_tools,
            mcp_enabled_toolsets: tool_registry::all_toolsets(),
            http: HttpConfig::default(),
            ..RuntimeConfig::default()
        })
    }
}

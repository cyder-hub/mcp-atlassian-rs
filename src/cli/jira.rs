use std::{
    fs,
    io::{self, Read},
    path::PathBuf,
};

use clap::{Args, Subcommand};
use serde_json::{Value, json};

use crate::{
    context::AppContext,
    jira::tools as tool_args,
    operations::{self, OperationError, OperationResult},
};

#[derive(Debug, Clone, PartialEq, Eq, Args)]
pub struct JiraArgs {
    #[command(subcommand)]
    pub command: JiraCommand,
}

#[derive(Debug, Clone, PartialEq, Eq, Subcommand)]
pub enum JiraCommand {
    Issue(IssueArgs),
    Project(ProjectArgs),
    Field(FieldArgs),
    Agile(AgileArgs),
    #[command(name = "service-desk")]
    ServiceDesk(ServiceDeskArgs),
    User(UserArgs),
}

#[derive(Debug, Clone, PartialEq, Eq, Args)]
pub struct IssueArgs {
    #[command(subcommand)]
    pub command: IssueCommand,
}

#[derive(Debug, Clone, PartialEq, Eq, Subcommand)]
pub enum IssueCommand {
    Get(GetIssueArgs),
    Search(SearchIssuesArgs),
    Comment(IssueCommentArgs),
    Transition(IssueTransitionArgs),
    Create(CreateIssueArgs),
    #[command(name = "create-batch")]
    CreateBatch(CreateIssuesArgs),
    Changelog(IssueChangelogArgs),
    Update(UpdateIssueArgs),
    Delete(DeleteIssueArgs),
    Watcher(IssueWatcherArgs),
    Worklog(IssueWorklogArgs),
    #[command(name = "link-type")]
    LinkType(IssueLinkTypeArgs),
    Parent(IssueParentArgs),
    Link(IssueLinkArgs),
    #[command(name = "remote-link")]
    RemoteLink(IssueRemoteLinkArgs),
    Attachment(IssueAttachmentArgs),
    Timeline(IssueTimelineArgs),
    Sla(IssueSlaArgs),
    Development(IssueDevelopmentArgs),
    #[command(name = "development-batch")]
    DevelopmentBatch(IssueDevelopmentBatchArgs),
}

#[derive(Debug, Clone, PartialEq, Eq, Args)]
pub struct GetIssueArgs {
    pub issue_key: String,
    #[arg(long)]
    pub fields: Option<String>,
    #[arg(long)]
    pub expand: Option<String>,
    #[arg(long)]
    pub comment_limit: Option<u32>,
    #[arg(long)]
    pub properties: Option<String>,
    #[arg(long)]
    pub update_history: Option<bool>,
}

#[derive(Debug, Clone, PartialEq, Eq, Args)]
pub struct SearchIssuesArgs {
    #[arg(long)]
    pub jql: String,
    #[arg(long)]
    pub fields: Option<String>,
    #[arg(long)]
    pub limit: Option<u32>,
    #[arg(long)]
    pub start_at: Option<u32>,
    #[arg(long)]
    pub projects: Option<String>,
    #[arg(long)]
    pub expand: Option<String>,
    #[arg(long)]
    pub page_token: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Args)]
pub struct IssueCommentArgs {
    #[command(subcommand)]
    pub command: IssueCommentCommand,
}

#[derive(Debug, Clone, PartialEq, Eq, Subcommand)]
pub enum IssueCommentCommand {
    Add(AddIssueCommentArgs),
    Update(UpdateIssueCommentArgs),
}

#[derive(Debug, Clone, PartialEq, Eq, Args)]
#[command(group(clap::ArgGroup::new("body_input").args(["body", "body_file", "body_stdin"]).required(true).multiple(false)))]
pub struct AddIssueCommentArgs {
    pub issue_key: String,
    #[command(flatten)]
    pub body_input: BodyInput,
    #[command(flatten)]
    pub visibility_input: VisibilityInput,
}

#[derive(Debug, Clone, PartialEq, Eq, Args)]
#[command(group(clap::ArgGroup::new("body_input").args(["body", "body_file", "body_stdin"]).required(true).multiple(false)))]
pub struct UpdateIssueCommentArgs {
    pub issue_key: String,
    pub comment_id: String,
    #[command(flatten)]
    pub body_input: BodyInput,
    #[command(flatten)]
    pub visibility_input: VisibilityInput,
}

#[derive(Debug, Clone, PartialEq, Eq, Args)]
pub struct IssueTransitionArgs {
    #[command(subcommand)]
    pub command: IssueTransitionCommand,
}

#[derive(Debug, Clone, PartialEq, Eq, Subcommand)]
pub enum IssueTransitionCommand {
    List(ListIssueTransitionsArgs),
    Apply(ApplyIssueTransitionArgs),
}

#[derive(Debug, Clone, PartialEq, Eq, Args)]
pub struct ListIssueTransitionsArgs {
    pub issue_key: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Args)]
#[command(group(clap::ArgGroup::new("fields_input").args(["fields_json", "fields_file"]).multiple(false)))]
#[command(group(clap::ArgGroup::new("comment_input").args(["comment", "comment_file", "comment_stdin"]).multiple(false)))]
pub struct ApplyIssueTransitionArgs {
    pub issue_key: String,
    pub transition_id: String,
    #[arg(long)]
    pub fields_json: Option<String>,
    #[arg(long)]
    pub fields_file: Option<PathBuf>,
    #[arg(long)]
    pub comment: Option<String>,
    #[arg(long)]
    pub comment_file: Option<PathBuf>,
    #[arg(long)]
    pub comment_stdin: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Args)]
#[command(group(clap::ArgGroup::new("description_input").args(["description", "description_file", "description_stdin"]).multiple(false)))]
#[command(group(clap::ArgGroup::new("additional_fields_input").args(["additional_fields_json", "additional_fields_file"]).multiple(false)))]
pub struct CreateIssueArgs {
    #[arg(long)]
    pub project: String,
    #[arg(long)]
    pub issue_type: String,
    #[arg(long)]
    pub summary: String,
    #[arg(long)]
    pub description: Option<String>,
    #[arg(long)]
    pub description_file: Option<PathBuf>,
    #[arg(long)]
    pub description_stdin: bool,
    #[arg(long)]
    pub assignee: Option<String>,
    #[arg(long)]
    pub priority: Option<String>,
    #[arg(long)]
    pub labels: Option<String>,
    #[arg(long)]
    pub components: Option<String>,
    #[arg(long)]
    pub fix_versions: Option<String>,
    #[arg(long)]
    pub additional_fields_json: Option<String>,
    #[arg(long)]
    pub additional_fields_file: Option<PathBuf>,
}

#[derive(Debug, Clone, PartialEq, Eq, Args)]
pub struct CreateIssuesArgs {
    #[arg(long)]
    pub issues_file: PathBuf,
    #[arg(long)]
    pub validate_only: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Args)]
pub struct IssueChangelogArgs {
    #[command(subcommand)]
    pub command: IssueChangelogCommand,
}

#[derive(Debug, Clone, PartialEq, Eq, Subcommand)]
pub enum IssueChangelogCommand {
    Batch(IssueChangelogBatchArgs),
}

#[derive(Debug, Clone, PartialEq, Eq, Args)]
pub struct IssueChangelogBatchArgs {
    #[arg(long)]
    pub issue_ids: String,
    #[arg(long)]
    pub limit: Option<u32>,
    #[arg(long)]
    pub field_ids: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Args)]
#[command(group(clap::ArgGroup::new("fields_input").args(["fields_json", "fields_file"]).required(true).multiple(false)))]
pub struct UpdateIssueArgs {
    pub issue_key: String,
    #[arg(long)]
    pub fields_json: Option<String>,
    #[arg(long)]
    pub fields_file: Option<PathBuf>,
    #[arg(long)]
    pub notify_users: Option<bool>,
}

#[derive(Debug, Clone, PartialEq, Eq, Args)]
pub struct DeleteIssueArgs {
    pub issue_key: String,
    #[arg(long)]
    pub delete_subtasks: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Args)]
pub struct IssueWatcherArgs {
    #[command(subcommand)]
    pub command: IssueWatcherCommand,
}

#[derive(Debug, Clone, PartialEq, Eq, Subcommand)]
pub enum IssueWatcherCommand {
    List(ListIssueWatchersArgs),
    Add(ChangeIssueWatcherArgs),
    Remove(ChangeIssueWatcherArgs),
}

#[derive(Debug, Clone, PartialEq, Eq, Args)]
pub struct ListIssueWatchersArgs {
    pub issue_key: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Args)]
pub struct ChangeIssueWatcherArgs {
    pub issue_key: String,
    pub user: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Args)]
pub struct IssueWorklogArgs {
    #[command(subcommand)]
    pub command: IssueWorklogCommand,
}

#[derive(Debug, Clone, PartialEq, Eq, Subcommand)]
pub enum IssueWorklogCommand {
    List(ListIssueWorklogsArgs),
    Add(AddIssueWorklogArgs),
}

#[derive(Debug, Clone, PartialEq, Eq, Args)]
pub struct ListIssueWorklogsArgs {
    pub issue_key: String,
    #[arg(long)]
    pub start_at: Option<u32>,
    #[arg(long)]
    pub limit: Option<u32>,
}

#[derive(Debug, Clone, PartialEq, Eq, Args)]
#[command(group(clap::ArgGroup::new("comment_input").args(["comment", "comment_file", "comment_stdin"]).multiple(false)))]
pub struct AddIssueWorklogArgs {
    pub issue_key: String,
    #[arg(long)]
    pub time_spent: String,
    #[arg(long)]
    pub started: Option<String>,
    #[arg(long)]
    pub comment: Option<String>,
    #[arg(long)]
    pub comment_file: Option<PathBuf>,
    #[arg(long)]
    pub comment_stdin: bool,
    #[command(flatten)]
    pub visibility_input: VisibilityInput,
    #[arg(long)]
    pub adjust_estimate: Option<String>,
    #[arg(long)]
    pub new_estimate: Option<String>,
    #[arg(long)]
    pub reduce_by: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Args)]
pub struct IssueLinkTypeArgs {
    #[command(subcommand)]
    pub command: IssueLinkTypeCommand,
}

#[derive(Debug, Clone, PartialEq, Eq, Subcommand)]
pub enum IssueLinkTypeCommand {
    List(ListIssueLinkTypesArgs),
}

#[derive(Debug, Clone, PartialEq, Eq, Args)]
pub struct ListIssueLinkTypesArgs {
    #[arg(long)]
    pub name: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Args)]
pub struct IssueParentArgs {
    #[command(subcommand)]
    pub command: IssueParentCommand,
}

#[derive(Debug, Clone, PartialEq, Eq, Subcommand)]
pub enum IssueParentCommand {
    Set(SetIssueParentArgs),
}

#[derive(Debug, Clone, PartialEq, Eq, Args)]
pub struct SetIssueParentArgs {
    pub issue_key: String,
    pub parent_key: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Args)]
pub struct IssueLinkArgs {
    #[command(subcommand)]
    pub command: IssueLinkCommand,
}

#[derive(Debug, Clone, PartialEq, Eq, Subcommand)]
pub enum IssueLinkCommand {
    Create(CreateIssueLinkArgs),
    Delete(DeleteIssueLinkArgs),
}

#[derive(Debug, Clone, PartialEq, Eq, Args)]
#[command(group(clap::ArgGroup::new("comment_input").args(["comment", "comment_file", "comment_stdin"]).multiple(false)))]
pub struct CreateIssueLinkArgs {
    #[arg(long = "type")]
    pub link_type: String,
    #[arg(long)]
    pub inward: String,
    #[arg(long)]
    pub outward: String,
    #[arg(long)]
    pub comment: Option<String>,
    #[arg(long)]
    pub comment_file: Option<PathBuf>,
    #[arg(long)]
    pub comment_stdin: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Args)]
pub struct DeleteIssueLinkArgs {
    pub link_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Args)]
pub struct IssueRemoteLinkArgs {
    #[command(subcommand)]
    pub command: IssueRemoteLinkCommand,
}

#[derive(Debug, Clone, PartialEq, Eq, Subcommand)]
pub enum IssueRemoteLinkCommand {
    Create(CreateRemoteIssueLinkArgs),
}

#[derive(Debug, Clone, PartialEq, Eq, Args)]
#[command(group(clap::ArgGroup::new("status_input").args(["status_json", "status_file"]).multiple(false)))]
pub struct CreateRemoteIssueLinkArgs {
    pub issue_key: String,
    #[arg(long)]
    pub url: String,
    #[arg(long)]
    pub title: String,
    #[arg(long)]
    pub global_id: Option<String>,
    #[arg(long)]
    pub relationship: Option<String>,
    #[arg(long)]
    pub summary: Option<String>,
    #[arg(long)]
    pub icon_url: Option<String>,
    #[arg(long)]
    pub icon_title: Option<String>,
    #[arg(long)]
    pub status_json: Option<String>,
    #[arg(long)]
    pub status_file: Option<PathBuf>,
}

#[derive(Debug, Clone, PartialEq, Eq, Args)]
pub struct IssueAttachmentArgs {
    #[command(subcommand)]
    pub command: IssueAttachmentCommand,
}

#[derive(Debug, Clone, PartialEq, Eq, Subcommand)]
pub enum IssueAttachmentCommand {
    List(ListIssueAttachmentsArgs),
    Images(GetIssueImageAttachmentsArgs),
}

#[derive(Debug, Clone, PartialEq, Eq, Args)]
pub struct ListIssueAttachmentsArgs {
    pub issue_key: String,
    #[arg(long)]
    pub attachment_ids: Option<String>,
    #[arg(long)]
    pub filename_contains: Option<String>,
    #[arg(long)]
    pub media_type: Option<String>,
    #[arg(long)]
    pub include_content: bool,
    #[arg(long)]
    pub max_bytes: Option<usize>,
}

#[derive(Debug, Clone, PartialEq, Eq, Args)]
pub struct GetIssueImageAttachmentsArgs {
    pub issue_key: String,
    #[arg(long)]
    pub include_content: bool,
    #[arg(long)]
    pub max_bytes: Option<usize>,
}

#[derive(Debug, Clone, PartialEq, Eq, Args)]
pub struct IssueTimelineArgs {
    pub issue_key: String,
    #[arg(long)]
    pub include_status_changes: Option<bool>,
    #[arg(long)]
    pub include_status_summary: Option<bool>,
}

#[derive(Debug, Clone, PartialEq, Eq, Args)]
pub struct IssueSlaArgs {
    pub issue_key: String,
    #[arg(long)]
    pub metrics: Option<String>,
    #[arg(long)]
    pub include_raw_dates: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Args)]
pub struct IssueDevelopmentArgs {
    pub issue_key: String,
    #[arg(long)]
    pub data_type: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Args)]
pub struct IssueDevelopmentBatchArgs {
    #[arg(long)]
    pub issues: String,
    #[arg(long)]
    pub data_type: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Args)]
pub struct ProjectArgs {
    #[command(subcommand)]
    pub command: ProjectCommand,
}

#[derive(Debug, Clone, PartialEq, Eq, Subcommand)]
pub enum ProjectCommand {
    Issues(ProjectIssuesArgs),
    List(ListProjectsArgs),
    Version(ProjectVersionArgs),
    Component(ProjectComponentArgs),
}

#[derive(Debug, Clone, PartialEq, Eq, Args)]
pub struct ProjectIssuesArgs {
    pub project_key: String,
    #[arg(long)]
    pub limit: Option<u32>,
    #[arg(long)]
    pub start_at: Option<u32>,
}

#[derive(Debug, Clone, PartialEq, Eq, Args)]
pub struct ListProjectsArgs {
    #[arg(long)]
    pub include_archived: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Args)]
pub struct ProjectVersionArgs {
    #[command(subcommand)]
    pub command: ProjectVersionCommand,
}

#[derive(Debug, Clone, PartialEq, Eq, Subcommand)]
pub enum ProjectVersionCommand {
    List(ListProjectVersionsArgs),
    Create(CreateProjectVersionArgs),
    #[command(name = "create-batch")]
    CreateBatch(CreateProjectVersionsArgs),
}

#[derive(Debug, Clone, PartialEq, Eq, Args)]
pub struct ListProjectVersionsArgs {
    pub project_key: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Args)]
pub struct CreateProjectVersionArgs {
    pub project_key: String,
    #[arg(long)]
    pub name: String,
    #[arg(long)]
    pub description: Option<String>,
    #[arg(long)]
    pub start_date: Option<String>,
    #[arg(long)]
    pub release_date: Option<String>,
    #[arg(long)]
    pub released: Option<bool>,
    #[arg(long)]
    pub archived: Option<bool>,
}

#[derive(Debug, Clone, PartialEq, Eq, Args)]
pub struct CreateProjectVersionsArgs {
    pub project_key: String,
    #[arg(long)]
    pub versions_file: PathBuf,
}

#[derive(Debug, Clone, PartialEq, Eq, Args)]
pub struct ProjectComponentArgs {
    #[command(subcommand)]
    pub command: ProjectComponentCommand,
}

#[derive(Debug, Clone, PartialEq, Eq, Subcommand)]
pub enum ProjectComponentCommand {
    List(ListProjectComponentsArgs),
}

#[derive(Debug, Clone, PartialEq, Eq, Args)]
pub struct ListProjectComponentsArgs {
    pub project_key: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Args)]
pub struct FieldArgs {
    #[command(subcommand)]
    pub command: FieldCommand,
}

#[derive(Debug, Clone, PartialEq, Eq, Subcommand)]
pub enum FieldCommand {
    Search(SearchFieldsArgs),
    Options(FieldOptionsArgs),
}

#[derive(Debug, Clone, PartialEq, Eq, Args)]
pub struct SearchFieldsArgs {
    #[arg(long)]
    pub keyword: Option<String>,
    #[arg(long)]
    pub limit: Option<u32>,
}

#[derive(Debug, Clone, PartialEq, Eq, Args)]
pub struct FieldOptionsArgs {
    pub field_id: String,
    #[arg(long)]
    pub context_id: Option<String>,
    #[arg(long)]
    pub project: Option<String>,
    #[arg(long)]
    pub issue_type: Option<String>,
    #[arg(long)]
    pub contains: Option<String>,
    #[arg(long)]
    pub return_limit: Option<u32>,
    #[arg(long)]
    pub values_only: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Args)]
pub struct UserArgs {
    #[command(subcommand)]
    pub command: UserCommand,
}

#[derive(Debug, Clone, PartialEq, Eq, Subcommand)]
pub enum UserCommand {
    Get(GetUserArgs),
}

#[derive(Debug, Clone, PartialEq, Eq, Args)]
pub struct GetUserArgs {
    #[arg(long)]
    pub user: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Args)]
pub struct AgileArgs {
    #[command(subcommand)]
    pub command: AgileCommand,
}

#[derive(Debug, Clone, PartialEq, Eq, Subcommand)]
pub enum AgileCommand {
    Board(AgileBoardArgs),
    Sprint(AgileSprintArgs),
}

#[derive(Debug, Clone, PartialEq, Eq, Args)]
pub struct AgileBoardArgs {
    #[command(subcommand)]
    pub command: AgileBoardCommand,
}

#[derive(Debug, Clone, PartialEq, Eq, Subcommand)]
pub enum AgileBoardCommand {
    List(ListAgileBoardsArgs),
    Issues(ListBoardIssuesArgs),
    Sprints(ListBoardSprintsArgs),
}

#[derive(Debug, Clone, PartialEq, Eq, Args)]
pub struct ListAgileBoardsArgs {
    #[arg(long)]
    pub project: Option<String>,
    #[arg(long = "type")]
    pub board_type: Option<String>,
    #[arg(long)]
    pub name: Option<String>,
    #[arg(long)]
    pub start_at: Option<u32>,
    #[arg(long)]
    pub limit: Option<u32>,
}

#[derive(Debug, Clone, PartialEq, Eq, Args)]
pub struct ListBoardIssuesArgs {
    pub board_id: String,
    #[arg(long)]
    pub jql: Option<String>,
    #[arg(long)]
    pub fields: Option<String>,
    #[arg(long)]
    pub start_at: Option<u32>,
    #[arg(long)]
    pub limit: Option<u32>,
}

#[derive(Debug, Clone, PartialEq, Eq, Args)]
pub struct ListBoardSprintsArgs {
    pub board_id: String,
    #[arg(long)]
    pub state: Option<String>,
    #[arg(long)]
    pub start_at: Option<u32>,
    #[arg(long)]
    pub limit: Option<u32>,
}

#[derive(Debug, Clone, PartialEq, Eq, Args)]
pub struct AgileSprintArgs {
    #[command(subcommand)]
    pub command: AgileSprintCommand,
}

#[derive(Debug, Clone, PartialEq, Eq, Subcommand)]
pub enum AgileSprintCommand {
    Issues(ListSprintIssuesArgs),
    Create(CreateSprintArgs),
    Update(UpdateSprintArgs),
    #[command(name = "add-issues")]
    AddIssues(AddIssuesToSprintArgs),
}

#[derive(Debug, Clone, PartialEq, Eq, Args)]
pub struct ListSprintIssuesArgs {
    pub sprint_id: String,
    #[arg(long)]
    pub fields: Option<String>,
    #[arg(long)]
    pub start_at: Option<u32>,
    #[arg(long)]
    pub limit: Option<u32>,
}

#[derive(Debug, Clone, PartialEq, Eq, Args)]
pub struct CreateSprintArgs {
    #[arg(long)]
    pub board_id: String,
    #[arg(long)]
    pub name: String,
    #[arg(long)]
    pub start_date: Option<String>,
    #[arg(long)]
    pub end_date: Option<String>,
    #[arg(long)]
    pub goal: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Args)]
pub struct UpdateSprintArgs {
    pub sprint_id: String,
    #[arg(long)]
    pub name: Option<String>,
    #[arg(long)]
    pub state: Option<String>,
    #[arg(long)]
    pub start_date: Option<String>,
    #[arg(long)]
    pub end_date: Option<String>,
    #[arg(long)]
    pub goal: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Args)]
pub struct AddIssuesToSprintArgs {
    pub sprint_id: String,
    #[arg(long)]
    pub issues: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Args)]
pub struct ServiceDeskArgs {
    #[command(subcommand)]
    pub command: ServiceDeskCommand,
}

#[derive(Debug, Clone, PartialEq, Eq, Subcommand)]
pub enum ServiceDeskCommand {
    Project(GetProjectServiceDeskArgs),
    Queue(ServiceDeskQueueArgs),
}

#[derive(Debug, Clone, PartialEq, Eq, Args)]
pub struct GetProjectServiceDeskArgs {
    pub project_key: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Args)]
pub struct ServiceDeskQueueArgs {
    #[command(subcommand)]
    pub command: ServiceDeskQueueCommand,
}

#[derive(Debug, Clone, PartialEq, Eq, Subcommand)]
pub enum ServiceDeskQueueCommand {
    List(ListServiceDeskQueuesArgs),
    Issues(ListServiceDeskQueueIssuesArgs),
}

#[derive(Debug, Clone, PartialEq, Eq, Args)]
pub struct ListServiceDeskQueuesArgs {
    pub service_desk_id: String,
    #[arg(long)]
    pub include_counts: bool,
    #[arg(long)]
    pub start: Option<u32>,
    #[arg(long)]
    pub limit: Option<u32>,
}

#[derive(Debug, Clone, PartialEq, Eq, Args)]
pub struct ListServiceDeskQueueIssuesArgs {
    pub service_desk_id: String,
    pub queue_id: String,
    #[arg(long)]
    pub start: Option<u32>,
    #[arg(long)]
    pub limit: Option<u32>,
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

#[derive(Debug, Clone, PartialEq, Eq, Args)]
#[command(group(clap::ArgGroup::new("visibility_input").args(["visibility_json", "visibility_file"]).multiple(false)))]
pub struct VisibilityInput {
    #[arg(long)]
    pub visibility_json: Option<String>,
    #[arg(long)]
    pub visibility_file: Option<PathBuf>,
}

pub async fn execute(
    args: JiraArgs,
    context: &AppContext,
) -> Result<OperationResult, OperationError> {
    match args.command {
        JiraCommand::Issue(args) => execute_issue(args, context).await,
        JiraCommand::Project(args) => execute_project(args, context).await,
        JiraCommand::Field(args) => execute_field(args, context).await,
        JiraCommand::Agile(args) => execute_agile(args, context).await,
        JiraCommand::ServiceDesk(args) => execute_service_desk(args, context).await,
        JiraCommand::User(args) => execute_user(args, context).await,
    }
}

async fn execute_issue(
    args: IssueArgs,
    context: &AppContext,
) -> Result<OperationResult, OperationError> {
    match args.command {
        IssueCommand::Get(args) => {
            operations::jira::get_issue(
                context,
                tool_args::JiraGetIssueArgs {
                    issue_key: args.issue_key,
                    fields: optional_string_value(args.fields),
                    expand: optional_string_value(args.expand),
                    comment_limit: args.comment_limit.map(u64::from),
                    properties: optional_string_value(args.properties),
                    update_history: args.update_history,
                },
            )
            .await
        }
        IssueCommand::Search(args) => {
            operations::jira::search_issues(
                context,
                tool_args::JiraSearchArgs {
                    jql: args.jql,
                    fields: optional_string_value(args.fields),
                    limit: args.limit.map(u64::from),
                    start_at: args.start_at.map(u64::from),
                    projects_filter: optional_string_value(args.projects),
                    expand: optional_string_value(args.expand),
                    page_token: args.page_token,
                },
            )
            .await
        }
        IssueCommand::Comment(args) => execute_issue_comment(args, context).await,
        IssueCommand::Transition(args) => execute_issue_transition(args, context).await,
        IssueCommand::Create(args) => {
            let description = read_optional_text(
                args.description,
                args.description_file,
                args.description_stdin,
                "description",
            )?;
            let additional_fields = read_optional_json(
                args.additional_fields_json,
                args.additional_fields_file,
                "additional_fields",
            )?;
            operations::jira::create_issue(
                context,
                tool_args::JiraCreateIssueArgs {
                    project_key: args.project,
                    summary: args.summary,
                    issue_type: args.issue_type,
                    assignee: args.assignee,
                    description,
                    components: optional_string_value(args.components),
                    priority: args.priority,
                    labels: optional_string_value(args.labels),
                    fix_versions: optional_string_value(args.fix_versions),
                    additional_fields,
                },
            )
            .await
        }
        IssueCommand::CreateBatch(args) => {
            operations::jira::create_issues(
                context,
                tool_args::JiraCreateIssuesArgs {
                    issues: read_json_file(&args.issues_file, "issues")?,
                    validate_only: Some(args.validate_only),
                },
            )
            .await
        }
        IssueCommand::Changelog(args) => match args.command {
            IssueChangelogCommand::Batch(args) => {
                operations::jira::get_issue_changelogs(
                    context,
                    tool_args::JiraGetIssueChangelogsArgs {
                        issue_ids_or_keys: string_value(args.issue_ids),
                        fields: optional_string_value(args.field_ids),
                        limit: args.limit.map(i64::from),
                    },
                )
                .await
            }
        },
        IssueCommand::Update(args) => {
            operations::jira::update_issue(
                context,
                tool_args::JiraUpdateIssueArgs {
                    issue_key: args.issue_key,
                    fields: read_required_json(args.fields_json, args.fields_file, "fields")?,
                    additional_fields: None,
                    components: None,
                    notify_users: args.notify_users,
                },
            )
            .await
        }
        IssueCommand::Delete(args) => {
            operations::jira::delete_issue(
                context,
                tool_args::JiraDeleteIssueArgs {
                    issue_key: args.issue_key,
                    delete_subtasks: Some(args.delete_subtasks),
                },
            )
            .await
        }
        IssueCommand::Watcher(args) => execute_issue_watcher(args, context).await,
        IssueCommand::Worklog(args) => execute_issue_worklog(args, context).await,
        IssueCommand::LinkType(args) => match args.command {
            IssueLinkTypeCommand::List(args) => {
                operations::jira::list_issue_link_types(
                    context,
                    tool_args::JiraListIssueLinkTypesArgs {
                        name_filter: args.name,
                    },
                )
                .await
            }
        },
        IssueCommand::Parent(args) => match args.command {
            IssueParentCommand::Set(args) => {
                operations::jira::set_issue_parent(
                    context,
                    tool_args::JiraSetIssueParentArgs {
                        issue_key: args.issue_key,
                        epic_key: args.parent_key,
                    },
                )
                .await
            }
        },
        IssueCommand::Link(args) => execute_issue_link(args, context).await,
        IssueCommand::RemoteLink(args) => execute_issue_remote_link(args, context).await,
        IssueCommand::Attachment(args) => execute_issue_attachment(args, context).await,
        IssueCommand::Timeline(args) => {
            operations::jira::get_issue_timeline(
                context,
                tool_args::JiraGetIssueTimelineArgs {
                    issue_key: args.issue_key,
                    include_status_changes: args.include_status_changes,
                    include_status_summary: args.include_status_summary,
                },
            )
            .await
        }
        IssueCommand::Sla(args) => {
            operations::jira::get_issue_sla_metrics(
                context,
                tool_args::JiraGetIssueSlaMetricsArgs {
                    issue_key: args.issue_key,
                    metrics: optional_string_value(args.metrics),
                    include_raw_dates: Some(args.include_raw_dates),
                },
            )
            .await
        }
        IssueCommand::Development(args) => {
            operations::jira::get_issue_development(
                context,
                tool_args::JiraGetIssueDevelopmentArgs {
                    issue_key: args.issue_key,
                    application_type: None,
                    data_type: args.data_type,
                },
            )
            .await
        }
        IssueCommand::DevelopmentBatch(args) => {
            operations::jira::get_issues_development(
                context,
                tool_args::JiraGetIssuesDevelopmentArgs {
                    issue_keys: string_value(args.issues),
                    application_type: None,
                    data_type: args.data_type,
                },
            )
            .await
        }
    }
}

async fn execute_issue_comment(
    args: IssueCommentArgs,
    context: &AppContext,
) -> Result<OperationResult, OperationError> {
    match args.command {
        IssueCommentCommand::Add(args) => {
            let body = read_body(args.body_input)?;
            let visibility = read_visibility(args.visibility_input)?;
            operations::jira::add_issue_comment(
                context,
                tool_args::JiraAddCommentArgs {
                    issue_key: args.issue_key,
                    body,
                    visibility,
                },
            )
            .await
        }
        IssueCommentCommand::Update(args) => {
            let body = read_body(args.body_input)?;
            let visibility = read_visibility(args.visibility_input)?;
            operations::jira::update_issue_comment(
                context,
                tool_args::JiraEditCommentArgs {
                    issue_key: args.issue_key,
                    comment_id: args.comment_id,
                    body,
                    visibility,
                },
            )
            .await
        }
    }
}

async fn execute_issue_transition(
    args: IssueTransitionArgs,
    context: &AppContext,
) -> Result<OperationResult, OperationError> {
    match args.command {
        IssueTransitionCommand::List(args) => {
            operations::jira::list_issue_transitions(
                context,
                tool_args::JiraGetTransitionsArgs {
                    issue_key: args.issue_key,
                },
            )
            .await
        }
        IssueTransitionCommand::Apply(args) => {
            let fields = read_optional_json(args.fields_json, args.fields_file, "fields")?;
            let comment = read_optional_text(
                args.comment,
                args.comment_file,
                args.comment_stdin,
                "comment",
            )?;
            operations::jira::transition_issue(
                context,
                tool_args::JiraTransitionIssueArgs {
                    issue_key: args.issue_key,
                    transition_id: args.transition_id,
                    fields,
                    comment,
                },
            )
            .await
        }
    }
}

async fn execute_issue_watcher(
    args: IssueWatcherArgs,
    context: &AppContext,
) -> Result<OperationResult, OperationError> {
    match args.command {
        IssueWatcherCommand::List(args) => {
            operations::jira::list_issue_watchers(
                context,
                tool_args::JiraListIssueWatchersArgs {
                    issue_key: args.issue_key,
                },
            )
            .await
        }
        IssueWatcherCommand::Add(args) => {
            operations::jira::add_issue_watcher(
                context,
                tool_args::JiraAddWatcherArgs {
                    issue_key: args.issue_key,
                    user_identifier: args.user,
                },
            )
            .await
        }
        IssueWatcherCommand::Remove(args) => {
            operations::jira::remove_issue_watcher(
                context,
                tool_args::JiraRemoveWatcherArgs {
                    issue_key: args.issue_key,
                    user_identifier: args.user,
                },
            )
            .await
        }
    }
}

async fn execute_issue_worklog(
    args: IssueWorklogArgs,
    context: &AppContext,
) -> Result<OperationResult, OperationError> {
    match args.command {
        IssueWorklogCommand::List(args) => {
            operations::jira::list_issue_worklogs(
                context,
                tool_args::JiraListIssueWorklogsArgs {
                    issue_key: args.issue_key,
                    start_at: args.start_at.map(u64::from),
                    limit: args.limit.map(u64::from),
                },
            )
            .await
        }
        IssueWorklogCommand::Add(args) => {
            let comment = read_optional_text(
                args.comment,
                args.comment_file,
                args.comment_stdin,
                "comment",
            )?;
            let visibility = read_visibility(args.visibility_input)?;
            operations::jira::add_issue_worklog(
                context,
                tool_args::JiraAddWorklogArgs {
                    issue_key: args.issue_key,
                    time_spent: args.time_spent,
                    started: args.started,
                    comment,
                    visibility,
                    adjust_estimate: args.adjust_estimate,
                    new_estimate: args.new_estimate,
                    reduce_by: args.reduce_by,
                },
            )
            .await
        }
    }
}

async fn execute_issue_link(
    args: IssueLinkArgs,
    context: &AppContext,
) -> Result<OperationResult, OperationError> {
    match args.command {
        IssueLinkCommand::Create(args) => {
            let comment = read_optional_text(
                args.comment,
                args.comment_file,
                args.comment_stdin,
                "comment",
            )?;
            operations::jira::create_issue_link(
                context,
                tool_args::JiraCreateIssueLinkArgs {
                    link_type: args.link_type,
                    inward_issue_key: args.inward,
                    outward_issue_key: args.outward,
                    comment,
                },
            )
            .await
        }
        IssueLinkCommand::Delete(args) => {
            operations::jira::delete_issue_link(
                context,
                tool_args::JiraDeleteIssueLinkArgs {
                    link_id: args.link_id,
                },
            )
            .await
        }
    }
}

async fn execute_issue_remote_link(
    args: IssueRemoteLinkArgs,
    context: &AppContext,
) -> Result<OperationResult, OperationError> {
    match args.command {
        IssueRemoteLinkCommand::Create(args) => {
            let status = read_optional_json(args.status_json, args.status_file, "status")?;
            operations::jira::create_remote_issue_link(
                context,
                tool_args::JiraCreateRemoteIssueLinkArgs {
                    issue_key: args.issue_key,
                    url: args.url,
                    title: args.title,
                    global_id: args.global_id,
                    summary: args.summary,
                    relationship: args.relationship,
                    icon_url: args.icon_url,
                    icon_title: args.icon_title,
                    status,
                },
            )
            .await
        }
    }
}

async fn execute_issue_attachment(
    args: IssueAttachmentArgs,
    context: &AppContext,
) -> Result<OperationResult, OperationError> {
    match args.command {
        IssueAttachmentCommand::List(args) => {
            operations::jira::get_issue_attachments(
                context,
                tool_args::JiraGetIssueAttachmentsArgs {
                    issue_key: args.issue_key,
                    attachment_ids: optional_string_value(args.attachment_ids),
                    filename_contains: args.filename_contains,
                    media_type: args.media_type,
                    include_content: Some(args.include_content),
                    max_bytes: optional_usize_to_u64(args.max_bytes, "max_bytes")?,
                },
            )
            .await
        }
        IssueAttachmentCommand::Images(args) => {
            operations::jira::get_issue_image_attachments(
                context,
                tool_args::JiraGetIssueImagesArgs {
                    issue_key: args.issue_key,
                    include_content: Some(args.include_content),
                    max_bytes: optional_usize_to_u64(args.max_bytes, "max_bytes")?,
                },
            )
            .await
        }
    }
}

async fn execute_project(
    args: ProjectArgs,
    context: &AppContext,
) -> Result<OperationResult, OperationError> {
    match args.command {
        ProjectCommand::Issues(args) => {
            operations::jira::list_project_issues(
                context,
                tool_args::JiraGetProjectIssuesArgs {
                    project_key: args.project_key,
                    limit: args.limit.map(u64::from),
                    start_at: args.start_at.map(u64::from),
                },
            )
            .await
        }
        ProjectCommand::List(args) => {
            operations::jira::list_projects(
                context,
                tool_args::JiraListProjectsArgs {
                    include_archived: Some(args.include_archived),
                },
            )
            .await
        }
        ProjectCommand::Version(args) => execute_project_version(args, context).await,
        ProjectCommand::Component(args) => match args.command {
            ProjectComponentCommand::List(args) => {
                operations::jira::list_project_components(
                    context,
                    tool_args::JiraListProjectComponentsArgs {
                        project_key: args.project_key,
                    },
                )
                .await
            }
        },
    }
}

async fn execute_project_version(
    args: ProjectVersionArgs,
    context: &AppContext,
) -> Result<OperationResult, OperationError> {
    match args.command {
        ProjectVersionCommand::List(args) => {
            operations::jira::list_project_versions(
                context,
                tool_args::JiraListProjectVersionsArgs {
                    project_key: args.project_key,
                },
            )
            .await
        }
        ProjectVersionCommand::Create(args) => {
            operations::jira::create_project_version(
                context,
                tool_args::JiraCreateProjectVersionArgs {
                    project_key: args.project_key,
                    name: args.name,
                    start_date: args.start_date,
                    release_date: args.release_date,
                    description: args.description,
                    released: args.released,
                    archived: args.archived,
                },
            )
            .await
        }
        ProjectVersionCommand::CreateBatch(args) => {
            operations::jira::create_project_versions(
                context,
                tool_args::JiraCreateProjectVersionsArgs {
                    project_key: args.project_key,
                    versions: read_json_file(&args.versions_file, "versions")?,
                },
            )
            .await
        }
    }
}

async fn execute_field(
    args: FieldArgs,
    context: &AppContext,
) -> Result<OperationResult, OperationError> {
    match args.command {
        FieldCommand::Search(args) => {
            operations::jira::search_fields(
                context,
                tool_args::JiraSearchFieldsArgs {
                    keyword: args.keyword,
                    limit: args.limit.map(u64::from),
                },
            )
            .await
        }
        FieldCommand::Options(args) => {
            operations::jira::list_field_options(
                context,
                tool_args::JiraGetFieldOptionsArgs {
                    field_id: args.field_id,
                    context_id: args.context_id,
                    project_key: args.project,
                    issue_type: args.issue_type,
                    contains: args.contains,
                    return_limit: args.return_limit.map(u64::from),
                    values_only: Some(args.values_only),
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
        UserCommand::Get(args) => {
            operations::jira::get_user(
                context,
                tool_args::JiraGetUserArgs {
                    user_identifier: args.user,
                },
            )
            .await
        }
    }
}

async fn execute_agile(
    args: AgileArgs,
    context: &AppContext,
) -> Result<OperationResult, OperationError> {
    match args.command {
        AgileCommand::Board(args) => execute_agile_board(args, context).await,
        AgileCommand::Sprint(args) => execute_agile_sprint(args, context).await,
    }
}

async fn execute_agile_board(
    args: AgileBoardArgs,
    context: &AppContext,
) -> Result<OperationResult, OperationError> {
    match args.command {
        AgileBoardCommand::List(args) => {
            operations::jira::list_agile_boards(
                context,
                tool_args::JiraListAgileBoardsArgs {
                    project_key: args.project,
                    board_type: args.board_type,
                    name: args.name,
                    start_at: args.start_at.map(u64::from),
                    limit: args.limit.map(u64::from),
                },
            )
            .await
        }
        AgileBoardCommand::Issues(args) => {
            operations::jira::list_board_issues(
                context,
                tool_args::JiraListBoardIssuesArgs {
                    board_id: parse_u64(&args.board_id, "board_id")?,
                    jql: args.jql,
                    fields: optional_string_value(args.fields),
                    start_at: args.start_at.map(u64::from),
                    limit: args.limit.map(u64::from),
                },
            )
            .await
        }
        AgileBoardCommand::Sprints(args) => {
            operations::jira::list_board_sprints(
                context,
                tool_args::JiraListBoardSprintsArgs {
                    board_id: parse_u64(&args.board_id, "board_id")?,
                    state: optional_string_value(args.state),
                    start_at: args.start_at.map(u64::from),
                    limit: args.limit.map(u64::from),
                },
            )
            .await
        }
    }
}

async fn execute_agile_sprint(
    args: AgileSprintArgs,
    context: &AppContext,
) -> Result<OperationResult, OperationError> {
    match args.command {
        AgileSprintCommand::Issues(args) => {
            operations::jira::list_sprint_issues(
                context,
                tool_args::JiraListSprintIssuesArgs {
                    sprint_id: parse_u64(&args.sprint_id, "sprint_id")?,
                    fields: optional_string_value(args.fields),
                    start_at: args.start_at.map(u64::from),
                    limit: args.limit.map(u64::from),
                },
            )
            .await
        }
        AgileSprintCommand::Create(args) => {
            operations::jira::create_sprint(
                context,
                tool_args::JiraCreateSprintArgs {
                    name: args.name,
                    origin_board_id: parse_u64(&args.board_id, "board_id")?,
                    start_date: args.start_date,
                    end_date: args.end_date,
                    goal: args.goal,
                },
            )
            .await
        }
        AgileSprintCommand::Update(args) => {
            operations::jira::update_sprint(
                context,
                tool_args::JiraUpdateSprintArgs {
                    sprint_id: parse_u64(&args.sprint_id, "sprint_id")?,
                    name: args.name,
                    state: args.state,
                    start_date: args.start_date,
                    end_date: args.end_date,
                    goal: args.goal,
                },
            )
            .await
        }
        AgileSprintCommand::AddIssues(args) => {
            operations::jira::add_issues_to_sprint(
                context,
                tool_args::JiraAddIssuesToSprintArgs {
                    sprint_id: parse_u64(&args.sprint_id, "sprint_id")?,
                    issue_keys: string_value(args.issues),
                },
            )
            .await
        }
    }
}

async fn execute_service_desk(
    args: ServiceDeskArgs,
    context: &AppContext,
) -> Result<OperationResult, OperationError> {
    match args.command {
        ServiceDeskCommand::Project(args) => {
            operations::jira::get_project_service_desk(
                context,
                tool_args::JiraGetServiceDeskForProjectArgs {
                    project_key: args.project_key,
                },
            )
            .await
        }
        ServiceDeskCommand::Queue(args) => match args.command {
            ServiceDeskQueueCommand::List(args) => {
                operations::jira::list_service_desk_queues(
                    context,
                    tool_args::JiraListServiceDeskQueuesArgs {
                        service_desk_id: args.service_desk_id,
                        include_counts: Some(args.include_counts),
                        start_at: args.start.map(u64::from),
                        limit: args.limit.map(u64::from),
                    },
                )
                .await
            }
            ServiceDeskQueueCommand::Issues(args) => {
                operations::jira::list_service_desk_queue_issues(
                    context,
                    tool_args::JiraListServiceDeskQueueIssuesArgs {
                        service_desk_id: args.service_desk_id,
                        queue_id: args.queue_id,
                        start_at: args.start.map(u64::from),
                        limit: args.limit.map(u64::from),
                    },
                )
                .await
            }
        },
    }
}

fn read_body(input: BodyInput) -> Result<String, OperationError> {
    read_required_text(input.body, input.body_file, input.body_stdin, "body")
}

fn read_visibility(input: VisibilityInput) -> Result<Option<Value>, OperationError> {
    read_optional_json(input.visibility_json, input.visibility_file, "visibility")
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

fn read_required_json(
    inline: Option<String>,
    file: Option<PathBuf>,
    field_name: &'static str,
) -> Result<Value, OperationError> {
    match read_optional_json(inline, file, field_name)? {
        Some(value) => Ok(value),
        None => Err(OperationError::invalid_input(format!(
            "{field_name} is required"
        ))),
    }
}

fn read_optional_json(
    inline: Option<String>,
    file: Option<PathBuf>,
    field_name: &'static str,
) -> Result<Option<Value>, OperationError> {
    if let Some(value) = inline {
        return parse_json(&value, field_name).map(Some);
    }
    if let Some(path) = file {
        return read_json_file(&path, field_name).map(Some);
    }
    Ok(None)
}

fn read_json_file(path: &PathBuf, field_name: &'static str) -> Result<Value, OperationError> {
    parse_json(&read_text_file(path, field_name)?, field_name)
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

fn parse_json(value: &str, field_name: &'static str) -> Result<Value, OperationError> {
    serde_json::from_str(value).map_err(|error| {
        OperationError::invalid_input(format!("{field_name} must be valid JSON: {error}"))
    })
}

fn optional_string_value(value: Option<String>) -> Option<Value> {
    value.map(string_value)
}

fn string_value(value: String) -> Value {
    json!(value)
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

    use super::{JiraArgs, execute};
    use crate::{
        cli::{ProviderCommand, parse_cli_args},
        config::{HttpConfig, RuntimeConfig},
        context::AppContext,
        jira::{config::JiraConfig, tools::JIRA_UPDATE_ISSUE_TOOL_NAME},
        operations::OperationErrorCategory,
        tool_registry,
        upstream::{auth::UpstreamAuth, custom_headers::CustomHeaders, proxy::ProxyConfig},
    };

    #[test]
    fn cli_jira_parses_core_issue_get_path() {
        parse_cli_args([
            "jira",
            "issue",
            "get",
            "PROJ-1",
            "--fields",
            "summary,status",
            "--update-history",
            "false",
        ])
        .unwrap();
    }

    #[test]
    fn cli_jira_parses_extended_service_desk_path() {
        parse_cli_args([
            "jira",
            "service-desk",
            "queue",
            "list",
            "12",
            "--include-counts",
            "--limit",
            "25",
        ])
        .unwrap();
    }

    #[tokio::test]
    async fn cli_jira_executes_attachment_preflight_validation() {
        let error = execute(
            jira_args([
                "jira",
                "issue",
                "attachment",
                "list",
                "ABC-1",
                "--filename-contains",
                "file",
                "--media-type",
                "image/png",
                "--include-content",
                "--max-bytes",
                "0",
            ]),
            &jira_context(BTreeSet::new()),
        )
        .await
        .unwrap_err();

        assert_eq!(error.category, OperationErrorCategory::InvalidInput);
        assert!(error.message.contains("max_bytes must be positive"));
    }

    #[tokio::test]
    async fn cli_jira_executes_update_with_json_file_input() {
        let path = std::env::temp_dir().join(format!(
            "workhub-cli-jira-fields-{}.json",
            std::process::id()
        ));
        std::fs::write(&path, "{}").unwrap();

        let error = execute(
            jira_args(vec![
                "jira".to_string(),
                "issue".to_string(),
                "update".to_string(),
                "ABC-1".to_string(),
                "--fields-file".to_string(),
                path.display().to_string(),
            ]),
            &jira_context(BTreeSet::new()),
        )
        .await
        .unwrap_err();
        let _ = std::fs::remove_file(path);

        assert_eq!(error.category, OperationErrorCategory::InvalidInput);
        assert!(
            error
                .message
                .contains("fields must contain at least one update")
        );
    }

    #[tokio::test]
    async fn cli_jira_ignores_mcp_disabled_tools() {
        let error = execute(
            jira_args(["jira", "issue", "update", "ABC-1", "--fields-json", "{}"]),
            &jira_context(BTreeSet::from([JIRA_UPDATE_ISSUE_TOOL_NAME.to_string()])),
        )
        .await
        .unwrap_err();

        assert_eq!(error.category, OperationErrorCategory::InvalidInput);
        assert!(
            error
                .message
                .contains("fields must contain at least one update")
        );
    }

    fn jira_args<I, S>(args: I) -> JiraArgs
    where
        I: IntoIterator<Item = S>,
        S: Into<std::ffi::OsString> + Clone,
    {
        match parse_cli_args(args).unwrap().command {
            ProviderCommand::Jira(args) => args,
            _ => panic!("expected Jira args"),
        }
    }

    fn jira_context(mcp_disabled_tools: BTreeSet<String>) -> AppContext {
        AppContext::from_config(&RuntimeConfig {
            jira: Some(JiraConfig {
                base_url: "https://jira.example".to_string(),
                deployment: crate::jira::config::JiraDeployment::ServerDataCenter,
                auth: UpstreamAuth::Pat {
                    personal_token: "test-pat-value".to_string(),
                },
                ssl_verify: true,
                proxy: ProxyConfig::default(),
                custom_headers: CustomHeaders::default(),
                mtls: None,
                projects_filter: BTreeSet::new(),
                timeout_seconds: 75,
            }),
            mcp_disabled_tools,
            mcp_enabled_toolsets: tool_registry::all_toolsets(),
            http: HttpConfig::default(),
            ..RuntimeConfig::default()
        })
    }
}

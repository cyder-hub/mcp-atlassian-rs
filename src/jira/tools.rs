use rmcp::schemars;
use serde::{Deserialize, Serialize};
use serde_json::Value;

fn string_list_or_string_schema(_: &mut schemars::SchemaGenerator) -> schemars::Schema {
    schemars::json_schema!({
        "description": "Accepts either a comma-separated string or an array of strings.",
        "oneOf": [
            {
                "type": "string",
                "description": "Comma-separated list of strings"
            },
            {
                "type": "array",
                "items": { "type": "string" }
            }
        ]
    })
}

fn object_schema(_: &mut schemars::SchemaGenerator) -> schemars::Schema {
    schemars::json_schema!({
        "type": "object",
        "description": "Free-form JSON object passed through to Jira; allowed keys and value shapes depend on the target Jira API.",
        "additionalProperties": true
    })
}

fn object_list_schema(_: &mut schemars::SchemaGenerator) -> schemars::Schema {
    schemars::json_schema!({
        "type": "array",
        "description": "Array of free-form JSON objects passed through to Jira.",
        "items": {
            "type": "object",
            "additionalProperties": true
        }
    })
}

pub const JIRA_GET_ISSUE_TOOL_NAME: &str = "jira_get_issue";
pub const JIRA_SEARCH_TOOL_NAME: &str = "jira_search_issues";
pub const JIRA_GET_PROJECT_ISSUES_TOOL_NAME: &str = "jira_list_project_issues";
pub const JIRA_SEARCH_FIELDS_TOOL_NAME: &str = "jira_search_fields";
pub const JIRA_GET_FIELD_OPTIONS_TOOL_NAME: &str = "jira_list_field_options";
pub const JIRA_ADD_COMMENT_TOOL_NAME: &str = "jira_add_issue_comment";
pub const JIRA_EDIT_COMMENT_TOOL_NAME: &str = "jira_update_issue_comment";
pub const JIRA_GET_TRANSITIONS_TOOL_NAME: &str = "jira_list_issue_transitions";
pub const JIRA_TRANSITION_ISSUE_TOOL_NAME: &str = "jira_transition_issue";
pub const JIRA_CREATE_ISSUE_TOOL_NAME: &str = "jira_create_issue";
pub const JIRA_CREATE_ISSUES_TOOL_NAME: &str = "jira_create_issues";
pub const JIRA_GET_ISSUE_CHANGELOGS_TOOL_NAME: &str = "jira_get_issue_changelogs";
pub const JIRA_UPDATE_ISSUE_TOOL_NAME: &str = "jira_update_issue";
pub const JIRA_DELETE_ISSUE_TOOL_NAME: &str = "jira_delete_issue";
pub const JIRA_LIST_PROJECTS_TOOL_NAME: &str = "jira_list_projects";
pub const JIRA_LIST_PROJECT_VERSIONS_TOOL_NAME: &str = "jira_list_project_versions";
pub const JIRA_LIST_PROJECT_COMPONENTS_TOOL_NAME: &str = "jira_list_project_components";
pub const JIRA_CREATE_PROJECT_VERSION_TOOL_NAME: &str = "jira_create_project_version";
pub const JIRA_CREATE_PROJECT_VERSIONS_TOOL_NAME: &str = "jira_create_project_versions";
pub const JIRA_GET_USER_TOOL_NAME: &str = "jira_get_user";
pub const JIRA_LIST_ISSUE_WATCHERS_TOOL_NAME: &str = "jira_list_issue_watchers";
pub const JIRA_ADD_WATCHER_TOOL_NAME: &str = "jira_add_issue_watcher";
pub const JIRA_REMOVE_WATCHER_TOOL_NAME: &str = "jira_remove_issue_watcher";
pub const JIRA_LIST_ISSUE_WORKLOGS_TOOL_NAME: &str = "jira_list_issue_worklogs";
pub const JIRA_ADD_WORKLOG_TOOL_NAME: &str = "jira_add_issue_worklog";
pub const JIRA_LIST_ISSUE_LINK_TYPES_TOOL_NAME: &str = "jira_list_issue_link_types";
pub const JIRA_SET_ISSUE_PARENT_TOOL_NAME: &str = "jira_set_issue_parent";
pub const JIRA_CREATE_ISSUE_LINK_TOOL_NAME: &str = "jira_create_issue_link";
pub const JIRA_CREATE_REMOTE_ISSUE_LINK_TOOL_NAME: &str = "jira_create_remote_issue_link";
pub const JIRA_DELETE_ISSUE_LINK_TOOL_NAME: &str = "jira_delete_issue_link";
pub const JIRA_GET_ISSUE_ATTACHMENTS_TOOL_NAME: &str = "jira_get_issue_attachments";
pub const JIRA_GET_ISSUE_IMAGES_TOOL_NAME: &str = "jira_get_issue_image_attachments";
pub const JIRA_LIST_AGILE_BOARDS_TOOL_NAME: &str = "jira_list_agile_boards";
pub const JIRA_LIST_BOARD_ISSUES_TOOL_NAME: &str = "jira_list_board_issues";
pub const JIRA_LIST_BOARD_SPRINTS_TOOL_NAME: &str = "jira_list_board_sprints";
pub const JIRA_LIST_SPRINT_ISSUES_TOOL_NAME: &str = "jira_list_sprint_issues";
pub const JIRA_CREATE_SPRINT_TOOL_NAME: &str = "jira_create_sprint";
pub const JIRA_UPDATE_SPRINT_TOOL_NAME: &str = "jira_update_sprint";
pub const JIRA_ADD_ISSUES_TO_SPRINT_TOOL_NAME: &str = "jira_add_issues_to_sprint";
pub const JIRA_GET_SERVICE_DESK_FOR_PROJECT_TOOL_NAME: &str = "jira_get_project_service_desk";
pub const JIRA_LIST_SERVICE_DESK_QUEUES_TOOL_NAME: &str = "jira_list_service_desk_queues";
pub const JIRA_LIST_SERVICE_DESK_QUEUE_ISSUES_TOOL_NAME: &str =
    "jira_list_service_desk_queue_issues";
pub const JIRA_LIST_ISSUE_FORMS_TOOL_NAME: &str = "jira_list_issue_forms";
pub const JIRA_GET_ISSUE_FORM_TOOL_NAME: &str = "jira_get_issue_form";
pub const JIRA_UPDATE_ISSUE_FORM_ANSWERS_TOOL_NAME: &str = "jira_update_issue_form_answers";
pub const JIRA_GET_ISSUE_TIMELINE_TOOL_NAME: &str = "jira_get_issue_timeline";
pub const JIRA_GET_ISSUE_SLA_METRICS_TOOL_NAME: &str = "jira_get_issue_sla_metrics";
pub const JIRA_GET_ISSUE_DEVELOPMENT_TOOL_NAME: &str = "jira_get_issue_development";
pub const JIRA_GET_ISSUES_DEVELOPMENT_TOOL_NAME: &str = "jira_get_issues_development";

#[cfg(test)]
pub const JIRA_EXTENSION_TOOL_NAMES: &[&str] = &[
    JIRA_CREATE_ISSUE_TOOL_NAME,
    JIRA_CREATE_ISSUES_TOOL_NAME,
    JIRA_GET_ISSUE_CHANGELOGS_TOOL_NAME,
    JIRA_UPDATE_ISSUE_TOOL_NAME,
    JIRA_DELETE_ISSUE_TOOL_NAME,
    JIRA_LIST_PROJECTS_TOOL_NAME,
    JIRA_LIST_PROJECT_VERSIONS_TOOL_NAME,
    JIRA_LIST_PROJECT_COMPONENTS_TOOL_NAME,
    JIRA_CREATE_PROJECT_VERSION_TOOL_NAME,
    JIRA_CREATE_PROJECT_VERSIONS_TOOL_NAME,
    JIRA_GET_USER_TOOL_NAME,
    JIRA_LIST_ISSUE_WATCHERS_TOOL_NAME,
    JIRA_ADD_WATCHER_TOOL_NAME,
    JIRA_REMOVE_WATCHER_TOOL_NAME,
    JIRA_LIST_ISSUE_WORKLOGS_TOOL_NAME,
    JIRA_ADD_WORKLOG_TOOL_NAME,
    JIRA_LIST_ISSUE_LINK_TYPES_TOOL_NAME,
    JIRA_SET_ISSUE_PARENT_TOOL_NAME,
    JIRA_CREATE_ISSUE_LINK_TOOL_NAME,
    JIRA_CREATE_REMOTE_ISSUE_LINK_TOOL_NAME,
    JIRA_DELETE_ISSUE_LINK_TOOL_NAME,
    JIRA_GET_ISSUE_ATTACHMENTS_TOOL_NAME,
    JIRA_GET_ISSUE_IMAGES_TOOL_NAME,
    JIRA_LIST_AGILE_BOARDS_TOOL_NAME,
    JIRA_LIST_BOARD_ISSUES_TOOL_NAME,
    JIRA_LIST_BOARD_SPRINTS_TOOL_NAME,
    JIRA_LIST_SPRINT_ISSUES_TOOL_NAME,
    JIRA_CREATE_SPRINT_TOOL_NAME,
    JIRA_UPDATE_SPRINT_TOOL_NAME,
    JIRA_ADD_ISSUES_TO_SPRINT_TOOL_NAME,
    JIRA_GET_SERVICE_DESK_FOR_PROJECT_TOOL_NAME,
    JIRA_LIST_SERVICE_DESK_QUEUES_TOOL_NAME,
    JIRA_LIST_SERVICE_DESK_QUEUE_ISSUES_TOOL_NAME,
    JIRA_LIST_ISSUE_FORMS_TOOL_NAME,
    JIRA_GET_ISSUE_FORM_TOOL_NAME,
    JIRA_UPDATE_ISSUE_FORM_ANSWERS_TOOL_NAME,
    JIRA_GET_ISSUE_TIMELINE_TOOL_NAME,
    JIRA_GET_ISSUE_SLA_METRICS_TOOL_NAME,
    JIRA_GET_ISSUE_DEVELOPMENT_TOOL_NAME,
    JIRA_GET_ISSUES_DEVELOPMENT_TOOL_NAME,
];

#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct JiraGetIssueArgs {
    pub issue_key: String,
    #[serde(default)]
    #[schemars(schema_with = "string_list_or_string_schema")]
    pub fields: Option<Value>,
    #[serde(default)]
    #[schemars(schema_with = "string_list_or_string_schema")]
    pub expand: Option<Value>,
    #[serde(default)]
    pub comment_limit: Option<u64>,
    #[serde(default)]
    #[schemars(schema_with = "string_list_or_string_schema")]
    pub properties: Option<Value>,
    #[serde(default)]
    pub update_history: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct JiraSearchArgs {
    pub jql: String,
    #[serde(default)]
    #[schemars(description = "Issue fields to return, as a comma-separated string or array.")]
    #[schemars(schema_with = "string_list_or_string_schema")]
    pub fields: Option<Value>,
    #[serde(default)]
    #[schemars(description = "Maximum number of issues to return for this page.")]
    pub limit: Option<u64>,
    #[serde(default)]
    #[schemars(
        description = "Offset pagination start index for Server/Data Center and older Jira Cloud search paths."
    )]
    pub start_at: Option<u64>,
    #[serde(default)]
    #[schemars(
        description = "Optional project keys to scope search results, as a comma-separated string or array."
    )]
    #[schemars(schema_with = "string_list_or_string_schema")]
    pub projects_filter: Option<Value>,
    #[serde(default)]
    #[schemars(schema_with = "string_list_or_string_schema")]
    pub expand: Option<Value>,
    #[serde(default)]
    #[schemars(
        description = "Jira Cloud next-page cursor returned by a previous search response; do not combine with start_at."
    )]
    pub page_token: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct JiraGetProjectIssuesArgs {
    pub project_key: String,
    #[serde(default)]
    pub limit: Option<u64>,
    #[serde(default)]
    pub start_at: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct JiraSearchFieldsArgs {
    #[serde(default)]
    pub keyword: Option<String>,
    #[serde(default)]
    pub limit: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct JiraGetFieldOptionsArgs {
    pub field_id: String,
    #[serde(default)]
    #[schemars(
        description = "Jira Cloud field-context id. Use this when the custom field has multiple contexts."
    )]
    pub context_id: Option<String>,
    #[serde(default)]
    #[schemars(
        description = "Project key used to resolve field options when context_id is not available."
    )]
    pub project_key: Option<String>,
    #[serde(default)]
    #[schemars(
        description = "Issue type used with project_key for Server/Data Center field-option lookups."
    )]
    pub issue_type: Option<String>,
    #[serde(default)]
    pub contains: Option<String>,
    #[serde(default)]
    #[schemars(description = "Maximum number of field options to return.")]
    pub return_limit: Option<u64>,
    #[serde(default)]
    pub values_only: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct JiraAddCommentArgs {
    pub issue_key: String,
    pub body: String,
    #[serde(default)]
    #[schemars(schema_with = "object_schema")]
    pub visibility: Option<Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct JiraEditCommentArgs {
    pub issue_key: String,
    pub comment_id: String,
    pub body: String,
    #[serde(default)]
    #[schemars(schema_with = "object_schema")]
    pub visibility: Option<Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct JiraGetTransitionsArgs {
    pub issue_key: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct JiraTransitionIssueArgs {
    pub issue_key: String,
    #[schemars(description = "Workflow transition id returned by jira_list_issue_transitions.")]
    pub transition_id: String,
    #[serde(default)]
    #[schemars(
        description = "Optional transition field values; Jira validates these against the selected transition screen."
    )]
    #[schemars(schema_with = "object_schema")]
    pub fields: Option<Value>,
    #[serde(default)]
    #[schemars(description = "Optional comment to add while applying the transition.")]
    pub comment: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct JiraCreateIssueArgs {
    pub project_key: String,
    pub summary: String,
    pub issue_type: String,
    #[serde(default)]
    pub assignee: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    #[schemars(description = "Component names or ids, as a comma-separated string or array.")]
    #[schemars(schema_with = "string_list_or_string_schema")]
    pub components: Option<Value>,
    #[serde(default)]
    #[schemars(
        description = "Additional Jira field values keyed by field id or field name, such as customfield_10000."
    )]
    #[schemars(schema_with = "object_schema")]
    pub additional_fields: Option<Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct JiraCreateIssuesArgs {
    #[schemars(
        description = "Issue creation objects. Each item must contain Jira create-issue fields such as project, issuetype, and summary."
    )]
    #[schemars(schema_with = "object_list_schema")]
    pub issues: Value,
    #[serde(default)]
    #[schemars(description = "When true, validate all issue payloads without creating issues.")]
    pub validate_only: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct JiraGetIssueChangelogsArgs {
    #[schemars(
        description = "Issue ids or keys to fetch changelogs for, as a comma-separated string or array."
    )]
    #[schemars(schema_with = "string_list_or_string_schema")]
    pub issue_ids_or_keys: Value,
    #[serde(default)]
    #[schemars(description = "Changelog fields to include, as a comma-separated string or array.")]
    #[schemars(schema_with = "string_list_or_string_schema")]
    pub fields: Option<Value>,
    #[serde(default)]
    #[schemars(
        description = "Maximum changelog items to request from the Jira Cloud bulk changelog endpoint."
    )]
    pub limit: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct JiraUpdateIssueArgs {
    pub issue_key: String,
    #[schemars(
        description = "Jira fields to update, keyed by field id or field name. This mutates existing issue data."
    )]
    #[schemars(schema_with = "object_schema")]
    pub fields: Value,
    #[serde(default)]
    #[schemars(description = "Additional update payload merged into the Jira edit request.")]
    #[schemars(schema_with = "object_schema")]
    pub additional_fields: Option<Value>,
    #[serde(default)]
    #[schemars(
        description = "Replacement component names or ids, as a comma-separated string or array."
    )]
    #[schemars(schema_with = "string_list_or_string_schema")]
    pub components: Option<Value>,
    #[serde(default)]
    #[schemars(
        description = "Set false to suppress Jira user notifications for this issue update when supported."
    )]
    pub notify_users: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct JiraDeleteIssueArgs {
    pub issue_key: String,
    #[serde(default)]
    #[schemars(description = "When true, delete the issue's subtasks along with the issue.")]
    pub delete_subtasks: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct JiraListProjectsArgs {
    #[serde(default)]
    pub include_archived: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct JiraProjectKeyArgs {
    pub project_key: String,
}

pub type JiraListProjectVersionsArgs = JiraProjectKeyArgs;
pub type JiraListProjectComponentsArgs = JiraProjectKeyArgs;

#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct JiraCreateProjectVersionArgs {
    pub project_key: String,
    pub name: String,
    #[serde(default)]
    pub start_date: Option<String>,
    #[serde(default)]
    pub release_date: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct JiraCreateProjectVersionsArgs {
    pub project_key: String,
    #[schemars(
        description = "Version creation objects for one project. Partial success is possible across items."
    )]
    #[schemars(schema_with = "object_list_schema")]
    pub versions: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct JiraGetUserArgs {
    #[schemars(
        description = "Use me/currentuser(), a Jira Cloud accountId, or a Server/Data Center username."
    )]
    pub user_identifier: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct JiraListIssueWatchersArgs {
    pub issue_key: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct JiraWatcherMutationArgs {
    pub issue_key: String,
    #[schemars(
        description = "Watcher identity. Jira Cloud expects an accountId; Server/Data Center expects a username."
    )]
    pub user_identifier: String,
}

pub type JiraAddWatcherArgs = JiraWatcherMutationArgs;
pub type JiraRemoveWatcherArgs = JiraWatcherMutationArgs;

#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct JiraListIssueWorklogsArgs {
    pub issue_key: String,
    #[serde(default)]
    #[schemars(description = "Offset pagination start index for issue worklogs.")]
    pub start_at: Option<u64>,
    #[serde(default)]
    #[schemars(description = "Maximum number of worklogs to return.")]
    pub limit: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct JiraAddWorklogArgs {
    pub issue_key: String,
    pub time_spent: String,
    #[serde(default)]
    #[schemars(
        description = "Worklog start timestamp in Jira's expected date-time format, including timezone offset."
    )]
    pub started: Option<String>,
    #[serde(default)]
    pub comment: Option<String>,
    #[serde(default)]
    #[schemars(
        description = "Optional Jira visibility object that restricts who can see the worklog."
    )]
    #[schemars(schema_with = "object_schema")]
    pub visibility: Option<Value>,
    #[serde(default)]
    #[schemars(
        description = "Jira remaining-estimate adjustment mode, such as auto, new, manual, or leave."
    )]
    pub adjust_estimate: Option<String>,
    #[serde(default)]
    #[schemars(
        description = "New remaining estimate when adjust_estimate requests a replacement estimate."
    )]
    pub new_estimate: Option<String>,
    #[serde(default)]
    #[schemars(
        description = "Amount to reduce the remaining estimate when using manual reduction."
    )]
    pub reduce_by: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct JiraListIssueLinkTypesArgs {
    #[serde(default)]
    pub name_filter: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct JiraSetIssueParentArgs {
    pub issue_key: String,
    pub epic_key: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct JiraCreateIssueLinkArgs {
    pub link_type: String,
    pub inward_issue_key: String,
    pub outward_issue_key: String,
    #[serde(default)]
    #[schemars(description = "Optional comment body to add when creating the issue link.")]
    pub comment: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct JiraCreateRemoteIssueLinkArgs {
    pub issue_key: String,
    #[schemars(description = "External URL to attach to the Jira issue.")]
    pub url: String,
    pub title: String,
    #[serde(default)]
    #[schemars(
        description = "Stable remote-link global id used by Jira to identify or replace an existing remote link."
    )]
    pub global_id: Option<String>,
    #[serde(default)]
    pub summary: Option<String>,
    #[serde(default)]
    pub relationship: Option<String>,
    #[serde(default)]
    #[schemars(description = "Icon URL shown by Jira for the remote link.")]
    pub icon_url: Option<String>,
    #[serde(default)]
    #[schemars(description = "Optional Jira remote-link status object.")]
    #[schemars(schema_with = "object_schema")]
    pub status: Option<Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct JiraDeleteIssueLinkArgs {
    pub link_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct JiraGetIssueAttachmentsArgs {
    pub issue_key: String,
    #[serde(default)]
    #[schemars(
        description = "Optional attachment ids to restrict the response, as a comma-separated string or array."
    )]
    #[schemars(schema_with = "string_list_or_string_schema")]
    pub attachment_ids: Option<Value>,
    #[serde(default)]
    #[schemars(
        description = "When true, include bounded inline attachment content in structuredContent."
    )]
    pub include_content: Option<bool>,
    #[serde(default)]
    #[schemars(description = "Maximum bytes of inline content to include per attachment.")]
    pub max_bytes: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct JiraGetIssueImagesArgs {
    pub issue_key: String,
    #[serde(default)]
    #[schemars(
        description = "When true, include bounded inline image content in structuredContent."
    )]
    pub include_content: Option<bool>,
    #[serde(default)]
    #[schemars(description = "Maximum bytes of inline content to include per image attachment.")]
    pub max_bytes: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct JiraListAgileBoardsArgs {
    #[serde(default)]
    pub project_key: Option<String>,
    #[serde(default)]
    #[schemars(description = "Optional Jira Software board type filter, such as scrum or kanban.")]
    pub board_type: Option<String>,
    #[serde(default)]
    #[schemars(description = "Offset pagination start index for Jira Software boards.")]
    pub start_at: Option<u64>,
    #[serde(default)]
    #[schemars(description = "Maximum number of Jira Software boards to return.")]
    pub limit: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct JiraListBoardIssuesArgs {
    pub board_id: u64,
    #[serde(default)]
    pub jql: Option<String>,
    #[serde(default)]
    #[schemars(
        description = "Issue fields to return from the board query, as a comma-separated string or array."
    )]
    #[schemars(schema_with = "string_list_or_string_schema")]
    pub fields: Option<Value>,
    #[serde(default)]
    #[schemars(description = "Offset pagination start index for board issues.")]
    pub start_at: Option<u64>,
    #[serde(default)]
    #[schemars(description = "Maximum number of board issues to return.")]
    pub limit: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct JiraListBoardSprintsArgs {
    pub board_id: u64,
    #[serde(default)]
    #[schemars(
        description = "Sprint states to include, as a comma-separated string or array; common values are active, future, and closed."
    )]
    #[schemars(schema_with = "string_list_or_string_schema")]
    pub state: Option<Value>,
    #[serde(default)]
    #[schemars(description = "Offset pagination start index for board sprints.")]
    pub start_at: Option<u64>,
    #[serde(default)]
    #[schemars(description = "Maximum number of board sprints to return.")]
    pub limit: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct JiraListSprintIssuesArgs {
    pub sprint_id: u64,
    #[serde(default)]
    #[schemars(
        description = "Issue fields to return from the sprint query, as a comma-separated string or array."
    )]
    #[schemars(schema_with = "string_list_or_string_schema")]
    pub fields: Option<Value>,
    #[serde(default)]
    #[schemars(description = "Offset pagination start index for sprint issues.")]
    pub start_at: Option<u64>,
    #[serde(default)]
    #[schemars(description = "Maximum number of sprint issues to return.")]
    pub limit: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct JiraCreateSprintArgs {
    pub name: String,
    #[schemars(description = "Jira Software board id that will own the new sprint.")]
    pub origin_board_id: u64,
    #[serde(default)]
    #[schemars(
        description = "Sprint start date in Jira Software's expected ISO-8601 date-time format."
    )]
    pub start_date: Option<String>,
    #[serde(default)]
    #[schemars(
        description = "Sprint end date in Jira Software's expected ISO-8601 date-time format."
    )]
    pub end_date: Option<String>,
    #[serde(default)]
    pub goal: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct JiraUpdateSprintArgs {
    pub sprint_id: u64,
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    #[schemars(
        description = "New sprint state. Jira Software commonly accepts future, active, or closed depending on transition rules."
    )]
    pub state: Option<String>,
    #[serde(default)]
    #[schemars(
        description = "Updated sprint start date in Jira Software's expected ISO-8601 date-time format."
    )]
    pub start_date: Option<String>,
    #[serde(default)]
    #[schemars(
        description = "Updated sprint end date in Jira Software's expected ISO-8601 date-time format."
    )]
    pub end_date: Option<String>,
    #[serde(default)]
    pub goal: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct JiraAddIssuesToSprintArgs {
    pub sprint_id: u64,
    #[schemars(
        description = "Issue keys to move into the sprint, as a comma-separated string or array."
    )]
    #[schemars(schema_with = "string_list_or_string_schema")]
    pub issue_keys: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct JiraGetServiceDeskForProjectArgs {
    pub project_key: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct JiraListServiceDeskQueuesArgs {
    #[schemars(description = "Jira Service Management service desk id.")]
    pub service_desk_id: String,
    #[serde(default)]
    #[schemars(description = "Offset pagination start index for service desk queues.")]
    pub start_at: Option<u64>,
    #[serde(default)]
    #[schemars(description = "Maximum number of service desk queues to return.")]
    pub limit: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct JiraListServiceDeskQueueIssuesArgs {
    #[schemars(description = "Jira Service Management service desk id.")]
    pub service_desk_id: String,
    #[schemars(description = "Jira Service Management queue id within the service desk.")]
    pub queue_id: String,
    #[serde(default)]
    #[schemars(description = "Offset pagination start index for service desk queue issues.")]
    pub start_at: Option<u64>,
    #[serde(default)]
    #[schemars(description = "Maximum number of service desk queue issues to return.")]
    pub limit: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct JiraListIssueFormsArgs {
    pub issue_key: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct JiraGetIssueFormArgs {
    pub issue_key: String,
    #[schemars(description = "Jira Forms or ProForma form id attached to the issue.")]
    pub form_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct JiraUpdateIssueFormAnswersArgs {
    pub issue_key: String,
    #[schemars(description = "Jira Forms or ProForma form id attached to the issue.")]
    pub form_id: String,
    #[schemars(
        description = "Answer objects to update on the form. The required keys depend on Jira Forms or ProForma field definitions."
    )]
    #[schemars(schema_with = "object_list_schema")]
    pub answers: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct JiraGetIssueTimelineArgs {
    pub issue_key: String,
    #[serde(default)]
    pub include_status_changes: Option<bool>,
    #[serde(default)]
    pub include_status_summary: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct JiraGetIssueSlaMetricsArgs {
    pub issue_key: String,
    #[serde(default)]
    #[schemars(
        description = "SLA metric names or field ids to include, as a comma-separated string or array."
    )]
    #[schemars(schema_with = "string_list_or_string_schema")]
    pub metrics: Option<Value>,
    #[serde(default)]
    #[schemars(
        description = "When true, include raw Jira Service Management SLA date fields in the response."
    )]
    pub include_raw_dates: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct JiraGetIssueDevelopmentArgs {
    pub issue_key: String,
    #[serde(default)]
    #[schemars(
        description = "Jira Software development application type filter, such as bitbucket or github."
    )]
    pub application_type: Option<String>,
    #[serde(default)]
    #[schemars(
        description = "Jira Software development data type selector, such as branch, commit, pullrequest, build, or deployment."
    )]
    pub data_type: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct JiraGetIssuesDevelopmentArgs {
    #[schemars(
        description = "Issue keys to fetch Jira Software development information for, as a comma-separated string or array."
    )]
    #[schemars(schema_with = "string_list_or_string_schema")]
    pub issue_keys: Value,
    #[serde(default)]
    #[schemars(
        description = "Jira Software development application type filter, such as bitbucket or github."
    )]
    pub application_type: Option<String>,
    #[serde(default)]
    #[schemars(
        description = "Jira Software development data type selector, such as branch, commit, pullrequest, build, or deployment."
    )]
    pub data_type: Option<String>,
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;

    use super::*;

    #[test]
    fn jira_extension_tool_names_are_complete_and_unique() {
        let unique = JIRA_EXTENSION_TOOL_NAMES.iter().collect::<BTreeSet<_>>();

        assert_eq!(JIRA_EXTENSION_TOOL_NAMES.len(), 40);
        assert_eq!(unique.len(), 40);
        assert!(unique.contains(&&JIRA_CREATE_ISSUE_TOOL_NAME));
        assert!(unique.contains(&&JIRA_GET_ISSUES_DEVELOPMENT_TOOL_NAME));
    }

    #[test]
    fn jira_core_tool_names_use_canonical_action_names() {
        assert_eq!(JIRA_GET_ISSUE_TOOL_NAME, "jira_get_issue");
        assert_eq!(JIRA_SEARCH_TOOL_NAME, "jira_search_issues");
        assert_eq!(
            JIRA_GET_PROJECT_ISSUES_TOOL_NAME,
            "jira_list_project_issues"
        );
        assert_eq!(JIRA_SEARCH_FIELDS_TOOL_NAME, "jira_search_fields");
        assert_eq!(JIRA_GET_FIELD_OPTIONS_TOOL_NAME, "jira_list_field_options");
        assert_eq!(JIRA_ADD_COMMENT_TOOL_NAME, "jira_add_issue_comment");
        assert_eq!(JIRA_EDIT_COMMENT_TOOL_NAME, "jira_update_issue_comment");
        assert_eq!(
            JIRA_GET_TRANSITIONS_TOOL_NAME,
            "jira_list_issue_transitions"
        );
        assert_eq!(JIRA_TRANSITION_ISSUE_TOOL_NAME, "jira_transition_issue");
    }

    #[test]
    fn string_list_schema_allows_comma_separated_string_or_array() {
        let mut generator = schemars::SchemaGenerator::default();
        let schema = string_list_or_string_schema(&mut generator);
        let value = serde_json::to_value(schema).unwrap();
        let one_of = value["oneOf"].as_array().unwrap();

        assert_eq!(one_of[0]["type"], "string");
        assert_eq!(one_of[1]["type"], "array");
        assert_eq!(one_of[1]["items"]["type"], "string");
    }

    #[test]
    fn jira_extension_args_accept_python_style_json_inputs() {
        let args: JiraCreateIssuesArgs = serde_json::from_value(serde_json::json!({
            "issues": "[{\"project_key\":\"ABC\",\"summary\":\"Demo\",\"issue_type\":\"Task\"}]",
            "validate_only": true
        }))
        .unwrap();

        assert_eq!(args.validate_only, Some(true));
        assert!(args.issues.is_string());
    }
}

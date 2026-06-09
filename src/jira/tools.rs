use rmcp::schemars;
use serde::{Deserialize, Serialize};
use serde_json::Value;

fn string_list_or_string_schema(_: &mut schemars::SchemaGenerator) -> schemars::Schema {
    schemars::json_schema!({
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
        "additionalProperties": true
    })
}

fn object_list_schema(_: &mut schemars::SchemaGenerator) -> schemars::Schema {
    schemars::json_schema!({
        "type": "array",
        "items": {
            "type": "object",
            "additionalProperties": true
        }
    })
}

pub const JIRA_GET_ISSUE_TOOL_NAME: &str = "jira_get_issue";
pub const JIRA_SEARCH_TOOL_NAME: &str = "jira_search";
pub const JIRA_GET_PROJECT_ISSUES_TOOL_NAME: &str = "jira_get_project_issues";
pub const JIRA_SEARCH_FIELDS_TOOL_NAME: &str = "jira_search_fields";
pub const JIRA_GET_FIELD_OPTIONS_TOOL_NAME: &str = "jira_get_field_options";
pub const JIRA_ADD_COMMENT_TOOL_NAME: &str = "jira_add_comment";
pub const JIRA_EDIT_COMMENT_TOOL_NAME: &str = "jira_edit_comment";
pub const JIRA_GET_TRANSITIONS_TOOL_NAME: &str = "jira_get_transitions";
pub const JIRA_TRANSITION_ISSUE_TOOL_NAME: &str = "jira_transition_issue";
pub const JIRA_CREATE_ISSUE_TOOL_NAME: &str = "jira_create_issue";
pub const JIRA_BATCH_CREATE_ISSUES_TOOL_NAME: &str = "jira_batch_create_issues";
pub const JIRA_BATCH_GET_CHANGELOGS_TOOL_NAME: &str = "jira_batch_get_changelogs";
pub const JIRA_UPDATE_ISSUE_TOOL_NAME: &str = "jira_update_issue";
pub const JIRA_DELETE_ISSUE_TOOL_NAME: &str = "jira_delete_issue";
pub const JIRA_GET_ALL_PROJECTS_TOOL_NAME: &str = "jira_get_all_projects";
pub const JIRA_GET_PROJECT_VERSIONS_TOOL_NAME: &str = "jira_get_project_versions";
pub const JIRA_GET_PROJECT_COMPONENTS_TOOL_NAME: &str = "jira_get_project_components";
pub const JIRA_CREATE_VERSION_TOOL_NAME: &str = "jira_create_version";
pub const JIRA_BATCH_CREATE_VERSIONS_TOOL_NAME: &str = "jira_batch_create_versions";
pub const JIRA_GET_USER_PROFILE_TOOL_NAME: &str = "jira_get_user_profile";
pub const JIRA_GET_ISSUE_WATCHERS_TOOL_NAME: &str = "jira_get_issue_watchers";
pub const JIRA_ADD_WATCHER_TOOL_NAME: &str = "jira_add_watcher";
pub const JIRA_REMOVE_WATCHER_TOOL_NAME: &str = "jira_remove_watcher";
pub const JIRA_GET_WORKLOG_TOOL_NAME: &str = "jira_get_worklog";
pub const JIRA_ADD_WORKLOG_TOOL_NAME: &str = "jira_add_worklog";
pub const JIRA_GET_LINK_TYPES_TOOL_NAME: &str = "jira_get_link_types";
pub const JIRA_LINK_TO_EPIC_TOOL_NAME: &str = "jira_link_to_epic";
pub const JIRA_CREATE_ISSUE_LINK_TOOL_NAME: &str = "jira_create_issue_link";
pub const JIRA_CREATE_REMOTE_ISSUE_LINK_TOOL_NAME: &str = "jira_create_remote_issue_link";
pub const JIRA_REMOVE_ISSUE_LINK_TOOL_NAME: &str = "jira_remove_issue_link";
pub const JIRA_DOWNLOAD_ATTACHMENTS_TOOL_NAME: &str = "jira_download_attachments";
pub const JIRA_GET_ISSUE_IMAGES_TOOL_NAME: &str = "jira_get_issue_images";
pub const JIRA_GET_AGILE_BOARDS_TOOL_NAME: &str = "jira_get_agile_boards";
pub const JIRA_GET_BOARD_ISSUES_TOOL_NAME: &str = "jira_get_board_issues";
pub const JIRA_GET_SPRINTS_FROM_BOARD_TOOL_NAME: &str = "jira_get_sprints_from_board";
pub const JIRA_GET_SPRINT_ISSUES_TOOL_NAME: &str = "jira_get_sprint_issues";
pub const JIRA_CREATE_SPRINT_TOOL_NAME: &str = "jira_create_sprint";
pub const JIRA_UPDATE_SPRINT_TOOL_NAME: &str = "jira_update_sprint";
pub const JIRA_ADD_ISSUES_TO_SPRINT_TOOL_NAME: &str = "jira_add_issues_to_sprint";
pub const JIRA_GET_SERVICE_DESK_FOR_PROJECT_TOOL_NAME: &str = "jira_get_service_desk_for_project";
pub const JIRA_GET_SERVICE_DESK_QUEUES_TOOL_NAME: &str = "jira_get_service_desk_queues";
pub const JIRA_GET_QUEUE_ISSUES_TOOL_NAME: &str = "jira_get_queue_issues";
pub const JIRA_GET_ISSUE_PROFORMA_FORMS_TOOL_NAME: &str = "jira_get_issue_proforma_forms";
pub const JIRA_GET_PROFORMA_FORM_DETAILS_TOOL_NAME: &str = "jira_get_proforma_form_details";
pub const JIRA_UPDATE_PROFORMA_FORM_ANSWERS_TOOL_NAME: &str = "jira_update_proforma_form_answers";
pub const JIRA_GET_ISSUE_DATES_TOOL_NAME: &str = "jira_get_issue_dates";
pub const JIRA_GET_ISSUE_SLA_TOOL_NAME: &str = "jira_get_issue_sla";
pub const JIRA_GET_ISSUE_DEVELOPMENT_INFO_TOOL_NAME: &str = "jira_get_issue_development_info";
pub const JIRA_GET_ISSUES_DEVELOPMENT_INFO_TOOL_NAME: &str = "jira_get_issues_development_info";

#[cfg(test)]
pub const STAGE3_JIRA_TOOL_NAMES: &[&str] = &[
    JIRA_CREATE_ISSUE_TOOL_NAME,
    JIRA_BATCH_CREATE_ISSUES_TOOL_NAME,
    JIRA_BATCH_GET_CHANGELOGS_TOOL_NAME,
    JIRA_UPDATE_ISSUE_TOOL_NAME,
    JIRA_DELETE_ISSUE_TOOL_NAME,
    JIRA_GET_ALL_PROJECTS_TOOL_NAME,
    JIRA_GET_PROJECT_VERSIONS_TOOL_NAME,
    JIRA_GET_PROJECT_COMPONENTS_TOOL_NAME,
    JIRA_CREATE_VERSION_TOOL_NAME,
    JIRA_BATCH_CREATE_VERSIONS_TOOL_NAME,
    JIRA_GET_USER_PROFILE_TOOL_NAME,
    JIRA_GET_ISSUE_WATCHERS_TOOL_NAME,
    JIRA_ADD_WATCHER_TOOL_NAME,
    JIRA_REMOVE_WATCHER_TOOL_NAME,
    JIRA_GET_WORKLOG_TOOL_NAME,
    JIRA_ADD_WORKLOG_TOOL_NAME,
    JIRA_GET_LINK_TYPES_TOOL_NAME,
    JIRA_LINK_TO_EPIC_TOOL_NAME,
    JIRA_CREATE_ISSUE_LINK_TOOL_NAME,
    JIRA_CREATE_REMOTE_ISSUE_LINK_TOOL_NAME,
    JIRA_REMOVE_ISSUE_LINK_TOOL_NAME,
    JIRA_DOWNLOAD_ATTACHMENTS_TOOL_NAME,
    JIRA_GET_ISSUE_IMAGES_TOOL_NAME,
    JIRA_GET_AGILE_BOARDS_TOOL_NAME,
    JIRA_GET_BOARD_ISSUES_TOOL_NAME,
    JIRA_GET_SPRINTS_FROM_BOARD_TOOL_NAME,
    JIRA_GET_SPRINT_ISSUES_TOOL_NAME,
    JIRA_CREATE_SPRINT_TOOL_NAME,
    JIRA_UPDATE_SPRINT_TOOL_NAME,
    JIRA_ADD_ISSUES_TO_SPRINT_TOOL_NAME,
    JIRA_GET_SERVICE_DESK_FOR_PROJECT_TOOL_NAME,
    JIRA_GET_SERVICE_DESK_QUEUES_TOOL_NAME,
    JIRA_GET_QUEUE_ISSUES_TOOL_NAME,
    JIRA_GET_ISSUE_PROFORMA_FORMS_TOOL_NAME,
    JIRA_GET_PROFORMA_FORM_DETAILS_TOOL_NAME,
    JIRA_UPDATE_PROFORMA_FORM_ANSWERS_TOOL_NAME,
    JIRA_GET_ISSUE_DATES_TOOL_NAME,
    JIRA_GET_ISSUE_SLA_TOOL_NAME,
    JIRA_GET_ISSUE_DEVELOPMENT_INFO_TOOL_NAME,
    JIRA_GET_ISSUES_DEVELOPMENT_INFO_TOOL_NAME,
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
    #[schemars(schema_with = "string_list_or_string_schema")]
    pub fields: Option<Value>,
    #[serde(default)]
    pub limit: Option<u64>,
    #[serde(default)]
    pub start_at: Option<u64>,
    #[serde(default)]
    #[schemars(schema_with = "string_list_or_string_schema")]
    pub projects_filter: Option<Value>,
    #[serde(default)]
    #[schemars(schema_with = "string_list_or_string_schema")]
    pub expand: Option<Value>,
    #[serde(default)]
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
    pub context_id: Option<String>,
    #[serde(default)]
    pub project_key: Option<String>,
    #[serde(default)]
    pub issue_type: Option<String>,
    #[serde(default)]
    pub contains: Option<String>,
    #[serde(default)]
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
    pub transition_id: String,
    #[serde(default)]
    #[schemars(schema_with = "object_schema")]
    pub fields: Option<Value>,
    #[serde(default)]
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
    #[schemars(schema_with = "string_list_or_string_schema")]
    pub components: Option<Value>,
    #[serde(default)]
    #[schemars(schema_with = "object_schema")]
    pub additional_fields: Option<Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct JiraBatchCreateIssuesArgs {
    #[schemars(schema_with = "object_list_schema")]
    pub issues: Value,
    #[serde(default)]
    pub validate_only: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct JiraBatchGetChangelogsArgs {
    #[schemars(schema_with = "string_list_or_string_schema")]
    pub issue_ids_or_keys: Value,
    #[serde(default)]
    #[schemars(schema_with = "string_list_or_string_schema")]
    pub fields: Option<Value>,
    #[serde(default)]
    pub limit: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct JiraUpdateIssueArgs {
    pub issue_key: String,
    #[schemars(schema_with = "object_schema")]
    pub fields: Value,
    #[serde(default)]
    #[schemars(schema_with = "object_schema")]
    pub additional_fields: Option<Value>,
    #[serde(default)]
    #[schemars(schema_with = "string_list_or_string_schema")]
    pub components: Option<Value>,
    #[serde(default)]
    pub notify_users: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct JiraDeleteIssueArgs {
    pub issue_key: String,
    #[serde(default)]
    pub delete_subtasks: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct JiraGetAllProjectsArgs {
    #[serde(default)]
    pub include_archived: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct JiraProjectKeyArgs {
    pub project_key: String,
}

pub type JiraGetProjectVersionsArgs = JiraProjectKeyArgs;
pub type JiraGetProjectComponentsArgs = JiraProjectKeyArgs;

#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct JiraCreateVersionArgs {
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
pub struct JiraBatchCreateVersionsArgs {
    pub project_key: String,
    #[schemars(schema_with = "object_list_schema")]
    pub versions: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct JiraGetUserProfileArgs {
    pub user_identifier: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct JiraGetIssueWatchersArgs {
    pub issue_key: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct JiraWatcherMutationArgs {
    pub issue_key: String,
    pub user_identifier: String,
}

pub type JiraAddWatcherArgs = JiraWatcherMutationArgs;
pub type JiraRemoveWatcherArgs = JiraWatcherMutationArgs;

#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct JiraGetWorklogArgs {
    pub issue_key: String,
    #[serde(default)]
    pub start_at: Option<u64>,
    #[serde(default)]
    pub limit: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct JiraAddWorklogArgs {
    pub issue_key: String,
    pub time_spent: String,
    #[serde(default)]
    pub started: Option<String>,
    #[serde(default)]
    pub comment: Option<String>,
    #[serde(default)]
    #[schemars(schema_with = "object_schema")]
    pub visibility: Option<Value>,
    #[serde(default)]
    pub adjust_estimate: Option<String>,
    #[serde(default)]
    pub new_estimate: Option<String>,
    #[serde(default)]
    pub reduce_by: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct JiraGetLinkTypesArgs {
    #[serde(default)]
    pub name_filter: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct JiraLinkToEpicArgs {
    pub issue_key: String,
    pub epic_key: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct JiraCreateIssueLinkArgs {
    pub link_type: String,
    pub inward_issue_key: String,
    pub outward_issue_key: String,
    #[serde(default)]
    pub comment: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct JiraCreateRemoteIssueLinkArgs {
    pub issue_key: String,
    pub url: String,
    pub title: String,
    #[serde(default)]
    pub global_id: Option<String>,
    #[serde(default)]
    pub summary: Option<String>,
    #[serde(default)]
    pub relationship: Option<String>,
    #[serde(default)]
    pub icon_url: Option<String>,
    #[serde(default)]
    #[schemars(schema_with = "object_schema")]
    pub status: Option<Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct JiraRemoveIssueLinkArgs {
    pub link_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct JiraDownloadAttachmentsArgs {
    pub issue_key: String,
    #[serde(default)]
    #[schemars(schema_with = "string_list_or_string_schema")]
    pub attachment_ids: Option<Value>,
    #[serde(default)]
    pub include_content: Option<bool>,
    #[serde(default)]
    pub max_bytes: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct JiraGetIssueImagesArgs {
    pub issue_key: String,
    #[serde(default)]
    pub include_content: Option<bool>,
    #[serde(default)]
    pub max_bytes: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct JiraGetAgileBoardsArgs {
    #[serde(default)]
    pub project_key: Option<String>,
    #[serde(default)]
    pub board_type: Option<String>,
    #[serde(default)]
    pub start_at: Option<u64>,
    #[serde(default)]
    pub limit: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct JiraGetBoardIssuesArgs {
    pub board_id: u64,
    #[serde(default)]
    pub jql: Option<String>,
    #[serde(default)]
    #[schemars(schema_with = "string_list_or_string_schema")]
    pub fields: Option<Value>,
    #[serde(default)]
    pub start_at: Option<u64>,
    #[serde(default)]
    pub limit: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct JiraGetSprintsFromBoardArgs {
    pub board_id: u64,
    #[serde(default)]
    #[schemars(schema_with = "string_list_or_string_schema")]
    pub state: Option<Value>,
    #[serde(default)]
    pub start_at: Option<u64>,
    #[serde(default)]
    pub limit: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct JiraGetSprintIssuesArgs {
    pub sprint_id: u64,
    #[serde(default)]
    #[schemars(schema_with = "string_list_or_string_schema")]
    pub fields: Option<Value>,
    #[serde(default)]
    pub start_at: Option<u64>,
    #[serde(default)]
    pub limit: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct JiraCreateSprintArgs {
    pub name: String,
    pub origin_board_id: u64,
    #[serde(default)]
    pub start_date: Option<String>,
    #[serde(default)]
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
    pub state: Option<String>,
    #[serde(default)]
    pub start_date: Option<String>,
    #[serde(default)]
    pub end_date: Option<String>,
    #[serde(default)]
    pub goal: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct JiraAddIssuesToSprintArgs {
    pub sprint_id: u64,
    #[schemars(schema_with = "string_list_or_string_schema")]
    pub issue_keys: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct JiraGetServiceDeskForProjectArgs {
    pub project_key: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct JiraGetServiceDeskQueuesArgs {
    pub service_desk_id: String,
    #[serde(default)]
    pub start_at: Option<u64>,
    #[serde(default)]
    pub limit: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct JiraGetQueueIssuesArgs {
    pub service_desk_id: String,
    pub queue_id: String,
    #[serde(default)]
    pub start_at: Option<u64>,
    #[serde(default)]
    pub limit: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct JiraGetIssueProformaFormsArgs {
    pub issue_key: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct JiraGetProformaFormDetailsArgs {
    pub issue_key: String,
    pub form_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct JiraUpdateProformaFormAnswersArgs {
    pub issue_key: String,
    pub form_id: String,
    #[schemars(schema_with = "object_list_schema")]
    pub answers: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct JiraGetIssueDatesArgs {
    pub issue_key: String,
    #[serde(default)]
    pub include_status_changes: Option<bool>,
    #[serde(default)]
    pub include_status_summary: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct JiraGetIssueSlaArgs {
    pub issue_key: String,
    #[serde(default)]
    #[schemars(schema_with = "string_list_or_string_schema")]
    pub metrics: Option<Value>,
    #[serde(default)]
    pub include_raw_dates: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct JiraGetIssueDevelopmentInfoArgs {
    pub issue_key: String,
    #[serde(default)]
    pub application_type: Option<String>,
    #[serde(default)]
    pub data_type: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct JiraGetIssuesDevelopmentInfoArgs {
    #[schemars(schema_with = "string_list_or_string_schema")]
    pub issue_keys: Value,
    #[serde(default)]
    pub application_type: Option<String>,
    #[serde(default)]
    pub data_type: Option<String>,
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;

    use super::*;

    #[test]
    fn stage_three_tool_names_are_complete_and_unique() {
        let unique = STAGE3_JIRA_TOOL_NAMES.iter().collect::<BTreeSet<_>>();

        assert_eq!(STAGE3_JIRA_TOOL_NAMES.len(), 40);
        assert_eq!(unique.len(), 40);
        assert!(unique.contains(&&JIRA_CREATE_ISSUE_TOOL_NAME));
        assert!(unique.contains(&&JIRA_GET_ISSUES_DEVELOPMENT_INFO_TOOL_NAME));
    }

    #[test]
    fn stage_two_tool_names_remain_unchanged() {
        assert_eq!(JIRA_GET_ISSUE_TOOL_NAME, "jira_get_issue");
        assert_eq!(JIRA_SEARCH_TOOL_NAME, "jira_search");
        assert_eq!(JIRA_GET_PROJECT_ISSUES_TOOL_NAME, "jira_get_project_issues");
        assert_eq!(JIRA_SEARCH_FIELDS_TOOL_NAME, "jira_search_fields");
        assert_eq!(JIRA_GET_FIELD_OPTIONS_TOOL_NAME, "jira_get_field_options");
        assert_eq!(JIRA_ADD_COMMENT_TOOL_NAME, "jira_add_comment");
        assert_eq!(JIRA_EDIT_COMMENT_TOOL_NAME, "jira_edit_comment");
        assert_eq!(JIRA_GET_TRANSITIONS_TOOL_NAME, "jira_get_transitions");
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
    fn stage_three_args_accept_python_style_json_inputs() {
        let args: JiraBatchCreateIssuesArgs = serde_json::from_value(serde_json::json!({
            "issues": "[{\"project_key\":\"ABC\",\"summary\":\"Demo\",\"issue_type\":\"Task\"}]",
            "validate_only": true
        }))
        .unwrap();

        assert_eq!(args.validate_only, Some(true));
        assert!(args.issues.is_string());
    }
}

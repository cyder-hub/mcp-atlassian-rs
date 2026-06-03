use rmcp::schemars;
use serde::{Deserialize, Serialize};
use serde_json::Value;

pub const JIRA_GET_ISSUE_TOOL_NAME: &str = "jira_get_issue";
pub const JIRA_SEARCH_TOOL_NAME: &str = "jira_search";
pub const JIRA_GET_PROJECT_ISSUES_TOOL_NAME: &str = "jira_get_project_issues";
pub const JIRA_SEARCH_FIELDS_TOOL_NAME: &str = "jira_search_fields";
pub const JIRA_GET_FIELD_OPTIONS_TOOL_NAME: &str = "jira_get_field_options";
pub const JIRA_ADD_COMMENT_TOOL_NAME: &str = "jira_add_comment";
pub const JIRA_EDIT_COMMENT_TOOL_NAME: &str = "jira_edit_comment";
pub const JIRA_GET_TRANSITIONS_TOOL_NAME: &str = "jira_get_transitions";
pub const JIRA_TRANSITION_ISSUE_TOOL_NAME: &str = "jira_transition_issue";

#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct JiraGetIssueArgs {
    pub issue_key: String,
    #[serde(default)]
    pub fields: Option<Value>,
    #[serde(default)]
    pub expand: Option<Value>,
    #[serde(default)]
    pub comment_limit: Option<u64>,
    #[serde(default)]
    pub properties: Option<Value>,
    #[serde(default)]
    pub update_history: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct JiraSearchArgs {
    pub jql: String,
    #[serde(default)]
    pub fields: Option<Value>,
    #[serde(default)]
    pub limit: Option<u64>,
    #[serde(default)]
    pub start_at: Option<u64>,
    #[serde(default)]
    pub projects_filter: Option<Value>,
    #[serde(default)]
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
    #[serde(default)]
    pub refresh: Option<bool>,
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
    pub visibility: Option<Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct JiraEditCommentArgs {
    pub issue_key: String,
    pub comment_id: String,
    pub body: String,
    #[serde(default)]
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
    pub fields: Option<Value>,
    #[serde(default)]
    pub comment: Option<String>,
}

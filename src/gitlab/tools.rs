use rmcp::schemars;
use serde::{Deserialize, Deserializer, Serialize, de::Error as _};
use serde_json::Value;

pub const GITLAB_GET_CURRENT_USER_TOOL_NAME: &str = "gitlab_get_current_user";
pub const GITLAB_GET_PROJECT_TOOL_NAME: &str = "gitlab_get_project";
pub const GITLAB_LIST_MERGE_REQUESTS_TOOL_NAME: &str = "gitlab_list_merge_requests";
pub const GITLAB_GET_MERGE_REQUEST_TOOL_NAME: &str = "gitlab_get_merge_request";
pub const GITLAB_LIST_MERGE_REQUEST_COMMITS_TOOL_NAME: &str = "gitlab_list_merge_request_commits";
pub const GITLAB_LIST_MERGE_REQUEST_DIFFS_TOOL_NAME: &str = "gitlab_list_merge_request_diffs";
pub const GITLAB_LIST_MERGE_REQUEST_PIPELINES_TOOL_NAME: &str =
    "gitlab_list_merge_request_pipelines";
pub const GITLAB_CREATE_MERGE_REQUEST_TOOL_NAME: &str = "gitlab_create_merge_request";
pub const GITLAB_UPDATE_MERGE_REQUEST_TOOL_NAME: &str = "gitlab_update_merge_request";
pub const GITLAB_ADD_MERGE_REQUEST_NOTE_TOOL_NAME: &str = "gitlab_add_merge_request_note";
pub const GITLAB_REPLY_MERGE_REQUEST_DISCUSSION_TOOL_NAME: &str =
    "gitlab_reply_merge_request_discussion";
pub const GITLAB_RESOLVE_MERGE_REQUEST_DISCUSSION_TOOL_NAME: &str =
    "gitlab_resolve_merge_request_discussion";
pub const GITLAB_GET_MERGE_REQUEST_APPROVAL_STATE_TOOL_NAME: &str =
    "gitlab_get_merge_request_approval_state";
pub const GITLAB_SET_MERGE_REQUEST_APPROVAL_TOOL_NAME: &str = "gitlab_set_merge_request_approval";
pub const GITLAB_ACCEPT_MERGE_REQUEST_TOOL_NAME: &str = "gitlab_accept_merge_request";

#[cfg(test)]
pub const GITLAB_TOOL_NAMES: &[&str] = &[
    GITLAB_GET_CURRENT_USER_TOOL_NAME,
    GITLAB_GET_PROJECT_TOOL_NAME,
    GITLAB_LIST_MERGE_REQUESTS_TOOL_NAME,
    GITLAB_GET_MERGE_REQUEST_TOOL_NAME,
    GITLAB_LIST_MERGE_REQUEST_COMMITS_TOOL_NAME,
    GITLAB_LIST_MERGE_REQUEST_DIFFS_TOOL_NAME,
    GITLAB_LIST_MERGE_REQUEST_PIPELINES_TOOL_NAME,
    GITLAB_CREATE_MERGE_REQUEST_TOOL_NAME,
    GITLAB_UPDATE_MERGE_REQUEST_TOOL_NAME,
    GITLAB_ADD_MERGE_REQUEST_NOTE_TOOL_NAME,
    GITLAB_REPLY_MERGE_REQUEST_DISCUSSION_TOOL_NAME,
    GITLAB_RESOLVE_MERGE_REQUEST_DISCUSSION_TOOL_NAME,
    GITLAB_GET_MERGE_REQUEST_APPROVAL_STATE_TOOL_NAME,
    GITLAB_SET_MERGE_REQUEST_APPROVAL_TOOL_NAME,
    GITLAB_ACCEPT_MERGE_REQUEST_TOOL_NAME,
];

fn string_list_or_string_schema(_: &mut schemars::SchemaGenerator) -> schemars::Schema {
    schemars::json_schema!({
        "description": "Accepts either a comma-separated string or an array of strings.",
        "oneOf": [
            { "type": "string", "description": "Comma-separated list of strings" },
            { "type": "array", "items": { "type": "string" } }
        ]
    })
}

fn deserialize_optional_string_list<'de, D>(
    deserializer: D,
) -> Result<Option<Vec<String>>, D::Error>
where
    D: Deserializer<'de>,
{
    let Some(value) = Option::<Value>::deserialize(deserializer)? else {
        return Ok(None);
    };

    match value {
        Value::Null => Ok(None),
        Value::String(value) => Ok(Some(split_string_list(&value))),
        Value::Array(values) => values
            .into_iter()
            .map(|value| {
                value
                    .as_str()
                    .map(|value| value.trim().to_string())
                    .ok_or_else(|| D::Error::custom("expected labels array to contain strings"))
            })
            .filter(|value| value.as_ref().map_or(true, |value| !value.is_empty()))
            .collect::<Result<Vec<_>, _>>()
            .map(Some),
        _ => Err(D::Error::custom(
            "expected labels to be a comma-separated string or an array of strings",
        )),
    }
}

fn split_string_list(value: &str) -> Vec<String> {
    value
        .split(',')
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToString::to_string)
        .collect()
}

#[derive(Debug, Clone, Default, Deserialize, Serialize, schemars::JsonSchema)]
pub struct GitlabGetCurrentUserArgs {}

#[derive(Debug, Clone, Deserialize, Serialize, schemars::JsonSchema)]
pub struct GitlabGetProjectArgs {
    pub project: String,
}

#[derive(Debug, Clone, Default, Deserialize, Serialize, schemars::JsonSchema)]
pub struct GitlabListMergeRequestsArgs {
    pub project: String,
    pub state: Option<String>,
    pub author_username: Option<String>,
    pub reviewer_username: Option<String>,
    pub source_branch: Option<String>,
    pub target_branch: Option<String>,
    #[serde(default, deserialize_with = "deserialize_optional_string_list")]
    #[schemars(schema_with = "string_list_or_string_schema")]
    pub labels: Option<Vec<String>>,
    pub page: Option<u64>,
    pub per_page: Option<u64>,
}

#[derive(Debug, Clone, Deserialize, Serialize, schemars::JsonSchema)]
pub struct GitlabGetMergeRequestArgs {
    pub project: String,
    pub merge_request_iid: u64,
    pub include_diverged_commits_count: Option<bool>,
    pub include_rebase_in_progress: Option<bool>,
}

#[derive(Debug, Clone, Deserialize, Serialize, schemars::JsonSchema)]
pub struct GitlabMergeRequestRefArgs {
    pub project: String,
    pub merge_request_iid: u64,
}

#[derive(Debug, Clone, Deserialize, Serialize, schemars::JsonSchema)]
pub struct GitlabListMergeRequestCommitsArgs {
    pub project: String,
    pub merge_request_iid: u64,
    #[schemars(description = "1-based GitLab pagination page. Defaults to 1.")]
    pub page: Option<u64>,
    #[schemars(description = "GitLab page size from 1 to 100. Defaults to 20.")]
    pub per_page: Option<u64>,
}

#[derive(Debug, Clone, Deserialize, Serialize, schemars::JsonSchema)]
pub struct GitlabListMergeRequestDiffsArgs {
    pub project: String,
    pub merge_request_iid: u64,
    pub max_diff_bytes: Option<u64>,
    #[schemars(description = "1-based GitLab pagination page. Defaults to 1.")]
    pub page: Option<u64>,
    #[schemars(description = "GitLab page size from 1 to 100. Defaults to 20.")]
    pub per_page: Option<u64>,
}

#[derive(Debug, Clone, Deserialize, Serialize, schemars::JsonSchema)]
pub struct GitlabListMergeRequestPipelinesArgs {
    pub project: String,
    pub merge_request_iid: u64,
    #[schemars(description = "1-based GitLab pagination page. Defaults to 1.")]
    pub page: Option<u64>,
    #[schemars(description = "GitLab page size from 1 to 100. Defaults to 20.")]
    pub per_page: Option<u64>,
}

#[derive(Debug, Clone, Deserialize, Serialize, schemars::JsonSchema)]
pub struct GitlabCreateMergeRequestArgs {
    pub project: String,
    pub source_branch: String,
    pub target_branch: String,
    pub title: String,
    pub description: Option<String>,
    pub remove_source_branch: Option<bool>,
    pub squash: Option<bool>,
    pub assignee_ids: Option<Vec<u64>>,
    pub reviewer_ids: Option<Vec<u64>>,
    pub labels: Option<Vec<String>>,
}

#[derive(Debug, Clone, Deserialize, Serialize, schemars::JsonSchema)]
pub struct GitlabUpdateMergeRequestArgs {
    pub project: String,
    pub merge_request_iid: u64,
    pub title: Option<String>,
    pub description: Option<String>,
    pub state_event: Option<String>,
    pub labels: Option<Vec<String>>,
    pub add_labels: Option<Vec<String>>,
    pub remove_labels: Option<Vec<String>>,
    pub reviewer_ids: Option<Vec<u64>>,
    pub assignee_ids: Option<Vec<u64>>,
    pub target_branch: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize, schemars::JsonSchema)]
pub struct GitlabAddMergeRequestNoteArgs {
    pub project: String,
    pub merge_request_iid: u64,
    pub body: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, schemars::JsonSchema)]
pub struct GitlabReplyMergeRequestDiscussionArgs {
    pub project: String,
    pub merge_request_iid: u64,
    pub discussion_id: String,
    pub body: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, schemars::JsonSchema)]
pub struct GitlabResolveMergeRequestDiscussionArgs {
    pub project: String,
    pub merge_request_iid: u64,
    pub discussion_id: String,
    pub resolved: bool,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, schemars::JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum GitlabMergeRequestApprovalAction {
    Approve,
    Unapprove,
}

#[derive(Debug, Clone, Deserialize, Serialize, schemars::JsonSchema)]
pub struct GitlabSetMergeRequestApprovalArgs {
    pub project: String,
    pub merge_request_iid: u64,
    #[schemars(description = "Approval action to apply to the merge request.")]
    pub action: GitlabMergeRequestApprovalAction,
}

#[derive(Debug, Clone, Deserialize, Serialize, schemars::JsonSchema)]
pub struct GitlabAcceptMergeRequestArgs {
    pub project: String,
    pub merge_request_iid: u64,
    pub sha: String,
    pub auto_merge: Option<bool>,
    pub squash: Option<bool>,
    pub should_remove_source_branch: Option<bool>,
    pub merge_commit_message: Option<String>,
    pub squash_commit_message: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn list_merge_requests_accepts_comma_separated_labels() {
        let args: GitlabListMergeRequestsArgs = serde_json::from_value(serde_json::json!({
            "project": "group/project",
            "labels": "bug, backend, ,ready"
        }))
        .unwrap();

        assert_eq!(
            args.labels,
            Some(vec![
                "bug".to_string(),
                "backend".to_string(),
                "ready".to_string()
            ])
        );
    }

    #[test]
    fn list_merge_requests_accepts_label_array() {
        let args: GitlabListMergeRequestsArgs = serde_json::from_value(serde_json::json!({
            "project": "group/project",
            "labels": ["bug", " backend ", ""]
        }))
        .unwrap();

        assert_eq!(
            args.labels,
            Some(vec!["bug".to_string(), "backend".to_string()])
        );
    }
}

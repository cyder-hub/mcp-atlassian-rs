use crate::gitlab::tools;

use super::{ToolAccess, ToolAnnotationMetadata, ToolMetadata, ToolService};

gitlab_metadata!(
    GITLAB_GET_CURRENT_USER_METADATA,
    tools::GITLAB_GET_CURRENT_USER_TOOL_NAME,
    Read,
    "gitlab_projects_read",
    read_only,
    "Get GitLab current user",
    "Get the GitLab user associated with the configured token."
);
gitlab_metadata!(
    GITLAB_GET_PROJECT_METADATA,
    tools::GITLAB_GET_PROJECT_TOOL_NAME,
    Read,
    "gitlab_projects_read",
    read_only,
    "Get GitLab project",
    "Get a GitLab project by numeric id or full path, subject to GITLAB_PROJECTS_FILTER."
);
gitlab_metadata!(
    GITLAB_LIST_MERGE_REQUESTS_METADATA,
    tools::GITLAB_LIST_MERGE_REQUESTS_TOOL_NAME,
    Read,
    "gitlab_merge_requests_read",
    read_only,
    "List GitLab merge requests",
    "List merge requests for a GitLab project with bounded pagination and optional filters."
);
gitlab_metadata!(
    GITLAB_GET_MERGE_REQUEST_METADATA,
    tools::GITLAB_GET_MERGE_REQUEST_TOOL_NAME,
    Read,
    "gitlab_merge_requests_read",
    read_only,
    "Get GitLab merge request",
    "Get one GitLab merge request by project and merge request IID."
);
gitlab_metadata!(
    GITLAB_LIST_MERGE_REQUEST_COMMITS_METADATA,
    tools::GITLAB_LIST_MERGE_REQUEST_COMMITS_TOOL_NAME,
    Read,
    "gitlab_merge_requests_read",
    read_only,
    "List GitLab merge request commits",
    "List commits for a GitLab merge request with bounded pagination."
);
gitlab_metadata!(
    GITLAB_LIST_MERGE_REQUEST_DIFFS_METADATA,
    tools::GITLAB_LIST_MERGE_REQUEST_DIFFS_TOOL_NAME,
    Read,
    "gitlab_merge_requests_read",
    read_only,
    "List GitLab merge request diffs",
    "List paginated, bounded diff data for a GitLab merge request and report diff truncation status."
);
gitlab_metadata!(
    GITLAB_LIST_MERGE_REQUEST_PIPELINES_METADATA,
    tools::GITLAB_LIST_MERGE_REQUEST_PIPELINES_TOOL_NAME,
    Read,
    "gitlab_merge_requests_read",
    read_only,
    "List GitLab merge request pipelines",
    "List pipelines associated with a GitLab merge request with bounded pagination."
);
gitlab_metadata!(
    GITLAB_CREATE_MERGE_REQUEST_METADATA,
    tools::GITLAB_CREATE_MERGE_REQUEST_TOOL_NAME,
    Write,
    "gitlab_merge_requests_write",
    additive_write,
    "Create GitLab merge request",
    "Create a GitLab merge request from source branch, target branch, and title."
);
gitlab_metadata!(
    GITLAB_UPDATE_MERGE_REQUEST_METADATA,
    tools::GITLAB_UPDATE_MERGE_REQUEST_TOOL_NAME,
    Write,
    "gitlab_merge_requests_write",
    destructive_write,
    "Update GitLab merge request",
    "Update mutable GitLab merge request fields such as title, description, labels, reviewers, assignees, state, or target branch."
);
gitlab_metadata!(
    GITLAB_ADD_MERGE_REQUEST_NOTE_METADATA,
    tools::GITLAB_ADD_MERGE_REQUEST_NOTE_TOOL_NAME,
    Write,
    "gitlab_merge_requests_write",
    additive_write,
    "Add GitLab merge request note",
    "Add a regular note to a GitLab merge request."
);
gitlab_metadata!(
    GITLAB_REPLY_MERGE_REQUEST_DISCUSSION_METADATA,
    tools::GITLAB_REPLY_MERGE_REQUEST_DISCUSSION_TOOL_NAME,
    Write,
    "gitlab_merge_requests_write",
    additive_write,
    "Reply to GitLab merge request discussion",
    "Reply to an existing GitLab merge request discussion."
);
gitlab_metadata!(
    GITLAB_RESOLVE_MERGE_REQUEST_DISCUSSION_METADATA,
    tools::GITLAB_RESOLVE_MERGE_REQUEST_DISCUSSION_TOOL_NAME,
    Write,
    "gitlab_merge_requests_write",
    destructive_write,
    "Resolve GitLab merge request discussion",
    "Set the resolved state of an existing GitLab merge request discussion."
);
gitlab_metadata!(
    GITLAB_GET_MERGE_REQUEST_APPROVAL_STATE_METADATA,
    tools::GITLAB_GET_MERGE_REQUEST_APPROVAL_STATE_TOOL_NAME,
    Read,
    "gitlab_merge_requests_read",
    read_only,
    "Get GitLab merge request approval state",
    "Get approval state for a GitLab merge request; availability can depend on GitLab tier and permissions."
);
gitlab_metadata!(
    GITLAB_SET_MERGE_REQUEST_APPROVAL_METADATA,
    tools::GITLAB_SET_MERGE_REQUEST_APPROVAL_TOOL_NAME,
    Write,
    "gitlab_merge_requests_write",
    destructive_write,
    "Set GitLab merge request approval",
    "Approve or unapprove a GitLab merge request as the current user."
);
gitlab_metadata!(
    GITLAB_ACCEPT_MERGE_REQUEST_METADATA,
    tools::GITLAB_ACCEPT_MERGE_REQUEST_TOOL_NAME,
    Write,
    "gitlab_merge_requests_merge",
    destructive_write,
    "Accept GitLab merge request",
    "Merge a GitLab merge request only when the required SHA matches the reviewed head."
);

pub const TOOLS: &[ToolMetadata] = &[
    GITLAB_GET_CURRENT_USER_METADATA,
    GITLAB_GET_PROJECT_METADATA,
    GITLAB_LIST_MERGE_REQUESTS_METADATA,
    GITLAB_GET_MERGE_REQUEST_METADATA,
    GITLAB_LIST_MERGE_REQUEST_COMMITS_METADATA,
    GITLAB_LIST_MERGE_REQUEST_DIFFS_METADATA,
    GITLAB_LIST_MERGE_REQUEST_PIPELINES_METADATA,
    GITLAB_CREATE_MERGE_REQUEST_METADATA,
    GITLAB_UPDATE_MERGE_REQUEST_METADATA,
    GITLAB_ADD_MERGE_REQUEST_NOTE_METADATA,
    GITLAB_REPLY_MERGE_REQUEST_DISCUSSION_METADATA,
    GITLAB_RESOLVE_MERGE_REQUEST_DISCUSSION_METADATA,
    GITLAB_GET_MERGE_REQUEST_APPROVAL_STATE_METADATA,
    GITLAB_SET_MERGE_REQUEST_APPROVAL_METADATA,
    GITLAB_ACCEPT_MERGE_REQUEST_METADATA,
];

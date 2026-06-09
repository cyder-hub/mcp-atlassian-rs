use crate::jira::tools;

use super::{ToolAccess, ToolAnnotationMetadata, ToolMetadata, ToolOutputSchema, ToolService};

pub const JIRA_GET_ISSUE_METADATA: ToolMetadata = ToolMetadata {
    name: tools::JIRA_GET_ISSUE_TOOL_NAME,
    service: ToolService::Jira,
    access: ToolAccess::Read,
    toolset: Some("jira_issues_read"),
    annotations: ToolAnnotationMetadata::read_only(),
    output_schema: None,
    title: "Get Jira issue",
    description: "Get one Jira issue by key, with optional fields, expands, properties, comments, and history controls.",
};

pub const JIRA_SEARCH_METADATA: ToolMetadata = ToolMetadata {
    name: tools::JIRA_SEARCH_TOOL_NAME,
    service: ToolService::Jira,
    access: ToolAccess::Read,
    toolset: Some("jira_issues_read"),
    annotations: ToolAnnotationMetadata::read_only(),
    output_schema: None,
    title: "Search Jira issues",
    description: "Search Jira issues with JQL. On Jira Cloud, continue paginated results with page_token instead of start_at.",
};

pub const JIRA_GET_PROJECT_ISSUES_METADATA: ToolMetadata = ToolMetadata {
    name: tools::JIRA_GET_PROJECT_ISSUES_TOOL_NAME,
    service: ToolService::Jira,
    access: ToolAccess::Read,
    toolset: Some("jira_issues_read"),
    annotations: ToolAnnotationMetadata::read_only(),
    output_schema: None,
    title: "List Jira project issues",
    description: "List issues in a Jira project with optional offset pagination.",
};

pub const JIRA_SEARCH_FIELDS_METADATA: ToolMetadata = ToolMetadata {
    name: tools::JIRA_SEARCH_FIELDS_TOOL_NAME,
    service: ToolService::Jira,
    access: ToolAccess::Read,
    toolset: Some("jira_fields_read"),
    annotations: ToolAnnotationMetadata::read_only(),
    output_schema: None,
    title: "Search Jira fields",
    description: "Search Jira fields by name, key, or id.",
};

pub const JIRA_GET_FIELD_OPTIONS_METADATA: ToolMetadata = ToolMetadata {
    name: tools::JIRA_GET_FIELD_OPTIONS_TOOL_NAME,
    service: ToolService::Jira,
    access: ToolAccess::Read,
    toolset: Some("jira_fields_read"),
    annotations: ToolAnnotationMetadata::read_only(),
    output_schema: None,
    title: "List Jira field options",
    description: "List custom-field options. Cloud requires context_id or project_key plus issue_type; Server/Data Center requires project_key and issue_type.",
};

pub const JIRA_ADD_COMMENT_METADATA: ToolMetadata = ToolMetadata {
    name: tools::JIRA_ADD_COMMENT_TOOL_NAME,
    service: ToolService::Jira,
    access: ToolAccess::Write,
    toolset: Some("jira_issue_comments_write"),
    annotations: ToolAnnotationMetadata::additive_write(),
    output_schema: None,
    title: "Add Jira issue comment",
    description: "Add a comment to a Jira issue, optionally with visibility restrictions.",
};

pub const JIRA_EDIT_COMMENT_METADATA: ToolMetadata = ToolMetadata {
    name: tools::JIRA_EDIT_COMMENT_TOOL_NAME,
    service: ToolService::Jira,
    access: ToolAccess::Write,
    toolset: Some("jira_issue_comments_update"),
    annotations: ToolAnnotationMetadata::destructive_write(),
    output_schema: None,
    title: "Update Jira issue comment",
    description: "Update an existing Jira issue comment, optionally with visibility restrictions.",
};

pub const JIRA_GET_TRANSITIONS_METADATA: ToolMetadata = ToolMetadata {
    name: tools::JIRA_GET_TRANSITIONS_TOOL_NAME,
    service: ToolService::Jira,
    access: ToolAccess::Read,
    toolset: Some("jira_issue_workflows_read"),
    annotations: ToolAnnotationMetadata::read_only(),
    output_schema: None,
    title: "List Jira issue transitions",
    description: "List workflow transitions currently available for a Jira issue.",
};

pub const JIRA_TRANSITION_ISSUE_METADATA: ToolMetadata = ToolMetadata {
    name: tools::JIRA_TRANSITION_ISSUE_TOOL_NAME,
    service: ToolService::Jira,
    access: ToolAccess::Write,
    toolset: Some("jira_issue_workflows_write"),
    annotations: ToolAnnotationMetadata::destructive_write(),
    output_schema: None,
    title: "Transition Jira issue",
    description: "Apply a workflow transition to a Jira issue, optionally with transition fields and a comment.",
};

jira_metadata!(
    JIRA_CREATE_ISSUE_METADATA,
    tools::JIRA_CREATE_ISSUE_TOOL_NAME,
    Write,
    "jira_issues_write",
    additive_write,
    "Create Jira issue",
    "Create a Jira issue from project, issue type, summary, and optional field values."
);
jira_metadata!(
    JIRA_CREATE_ISSUES_METADATA,
    tools::JIRA_CREATE_ISSUES_TOOL_NAME,
    Write,
    "jira_issues_bulk_write",
    additive_write,
    "Create Jira issues",
    "Create multiple Jira issues in one bulk request; validate_only can dry-run validation.",
    JiraCreateIssuesResult
);
jira_metadata!(
    JIRA_GET_ISSUE_CHANGELOGS_METADATA,
    tools::JIRA_GET_ISSUE_CHANGELOGS_TOOL_NAME,
    Read,
    "jira_issues_history_read",
    read_only,
    "Get Jira issue changelogs",
    "Fetch changelogs for multiple Jira issues using Jira Cloud's bulk changelog endpoint."
);
jira_metadata!(
    JIRA_UPDATE_ISSUE_METADATA,
    tools::JIRA_UPDATE_ISSUE_TOOL_NAME,
    Write,
    "jira_issues_update",
    destructive_write,
    "Update Jira issue",
    "Update fields on one Jira issue, with optional notification control.",
    JiraMutationResult
);
jira_metadata!(
    JIRA_DELETE_ISSUE_METADATA,
    tools::JIRA_DELETE_ISSUE_TOOL_NAME,
    Write,
    "jira_issues_delete",
    destructive_write,
    "Delete Jira issue",
    "Delete one Jira issue, optionally deleting subtasks.",
    JiraMutationResult
);
jira_metadata!(
    JIRA_LIST_PROJECTS_METADATA,
    tools::JIRA_LIST_PROJECTS_TOOL_NAME,
    Read,
    "jira_projects_read",
    read_only,
    "List Jira projects",
    "List Jira projects visible to the current user."
);
jira_metadata!(
    JIRA_LIST_PROJECT_VERSIONS_METADATA,
    tools::JIRA_LIST_PROJECT_VERSIONS_TOOL_NAME,
    Read,
    "jira_projects_metadata_read",
    read_only,
    "List Jira project versions",
    "List versions for a Jira project."
);
jira_metadata!(
    JIRA_LIST_PROJECT_COMPONENTS_METADATA,
    tools::JIRA_LIST_PROJECT_COMPONENTS_TOOL_NAME,
    Read,
    "jira_projects_metadata_read",
    read_only,
    "List Jira project components",
    "List components for a Jira project."
);
jira_metadata!(
    JIRA_CREATE_PROJECT_VERSION_METADATA,
    tools::JIRA_CREATE_PROJECT_VERSION_TOOL_NAME,
    Write,
    "jira_project_versions_write",
    additive_write,
    "Create Jira project version",
    "Create a Jira project version."
);
jira_metadata!(
    JIRA_CREATE_PROJECT_VERSIONS_METADATA,
    tools::JIRA_CREATE_PROJECT_VERSIONS_TOOL_NAME,
    Write,
    "jira_project_versions_write",
    additive_write,
    "Create Jira project versions",
    "Create multiple versions in one Jira project.",
    JiraPartitionedVersionsResult
);
jira_metadata!(
    JIRA_GET_USER_METADATA,
    tools::JIRA_GET_USER_TOOL_NAME,
    Read,
    "jira_users_read",
    read_only,
    "Get Jira user",
    "Get the current or specified Jira user. Use me/currentuser(), Cloud accountId, or Server/Data Center username."
);
jira_metadata!(
    JIRA_LIST_ISSUE_WATCHERS_METADATA,
    tools::JIRA_LIST_ISSUE_WATCHERS_TOOL_NAME,
    Read,
    "jira_issue_watchers_read",
    read_only,
    "List Jira issue watchers",
    "List watchers for a Jira issue."
);
jira_metadata!(
    JIRA_ADD_WATCHER_METADATA,
    tools::JIRA_ADD_WATCHER_TOOL_NAME,
    Write,
    "jira_issue_watchers_write",
    additive_write,
    "Add Jira issue watcher",
    "Add a watcher to a Jira issue. Cloud expects accountId; Server/Data Center expects username."
);
jira_metadata!(
    JIRA_REMOVE_WATCHER_METADATA,
    tools::JIRA_REMOVE_WATCHER_TOOL_NAME,
    Write,
    "jira_issue_watchers_delete",
    destructive_write,
    "Remove Jira issue watcher",
    "Remove a watcher from a Jira issue. Cloud expects accountId; Server/Data Center expects username."
);
jira_metadata!(
    JIRA_LIST_ISSUE_WORKLOGS_METADATA,
    tools::JIRA_LIST_ISSUE_WORKLOGS_TOOL_NAME,
    Read,
    "jira_issue_worklogs_read",
    read_only,
    "List Jira issue worklogs",
    "List worklogs for a Jira issue."
);
jira_metadata!(
    JIRA_ADD_WORKLOG_METADATA,
    tools::JIRA_ADD_WORKLOG_TOOL_NAME,
    Write,
    "jira_issue_worklogs_write",
    additive_write,
    "Add Jira issue worklog",
    "Add a worklog entry to a Jira issue."
);
jira_metadata!(
    JIRA_LIST_ISSUE_LINK_TYPES_METADATA,
    tools::JIRA_LIST_ISSUE_LINK_TYPES_TOOL_NAME,
    Read,
    "jira_issue_links_read",
    read_only,
    "List Jira issue link types",
    "List Jira issue link types."
);
jira_metadata!(
    JIRA_SET_ISSUE_PARENT_METADATA,
    tools::JIRA_SET_ISSUE_PARENT_TOOL_NAME,
    Write,
    "jira_issue_links_write",
    destructive_write,
    "Set Jira issue parent",
    "Set a Jira issue parent or epic using the parent key field."
);
jira_metadata!(
    JIRA_CREATE_ISSUE_LINK_METADATA,
    tools::JIRA_CREATE_ISSUE_LINK_TOOL_NAME,
    Write,
    "jira_issue_links_write",
    additive_write,
    "Create Jira issue link",
    "Create a typed relationship between two Jira issues."
);
jira_metadata!(
    JIRA_CREATE_REMOTE_ISSUE_LINK_METADATA,
    tools::JIRA_CREATE_REMOTE_ISSUE_LINK_TOOL_NAME,
    Write,
    "jira_issue_links_write",
    additive_write,
    "Create Jira remote issue link",
    "Create an external remote link on a Jira issue."
);
jira_metadata!(
    JIRA_DELETE_ISSUE_LINK_METADATA,
    tools::JIRA_DELETE_ISSUE_LINK_TOOL_NAME,
    Write,
    "jira_issue_links_delete",
    destructive_write,
    "Delete Jira issue link",
    "Delete a Jira issue link by link id."
);
jira_metadata!(
    JIRA_GET_ISSUE_ATTACHMENTS_METADATA,
    tools::JIRA_GET_ISSUE_ATTACHMENTS_TOOL_NAME,
    Read,
    "jira_issue_attachments_read",
    read_only,
    "Get Jira issue attachments",
    "List Jira issue attachments and optionally include bounded inline content.",
    JiraIssueAttachmentsResult
);
jira_metadata!(
    JIRA_GET_ISSUE_IMAGES_METADATA,
    tools::JIRA_GET_ISSUE_IMAGES_TOOL_NAME,
    Read,
    "jira_issue_attachments_read",
    read_only,
    "Get Jira issue image attachments",
    "List image attachments for a Jira issue and optionally include bounded inline content.",
    JiraIssueImageAttachmentsResult
);
jira_metadata!(
    JIRA_LIST_AGILE_BOARDS_METADATA,
    tools::JIRA_LIST_AGILE_BOARDS_TOOL_NAME,
    Read,
    "jira_agile_boards_read",
    read_only,
    "List Jira agile boards",
    "List Jira Software agile boards.",
    JiraProductDependencyResult
);
jira_metadata!(
    JIRA_LIST_BOARD_ISSUES_METADATA,
    tools::JIRA_LIST_BOARD_ISSUES_TOOL_NAME,
    Read,
    "jira_agile_boards_read",
    read_only,
    "List Jira board issues",
    "List issues from a Jira Software board.",
    JiraProductDependencyResult
);
jira_metadata!(
    JIRA_LIST_BOARD_SPRINTS_METADATA,
    tools::JIRA_LIST_BOARD_SPRINTS_TOOL_NAME,
    Read,
    "jira_sprints_read",
    read_only,
    "List Jira board sprints",
    "List sprints from a Jira Software board.",
    JiraProductDependencyResult
);
jira_metadata!(
    JIRA_LIST_SPRINT_ISSUES_METADATA,
    tools::JIRA_LIST_SPRINT_ISSUES_TOOL_NAME,
    Read,
    "jira_sprints_read",
    read_only,
    "List Jira sprint issues",
    "List issues from a Jira Software sprint.",
    JiraProductDependencyResult
);
jira_metadata!(
    JIRA_CREATE_SPRINT_METADATA,
    tools::JIRA_CREATE_SPRINT_TOOL_NAME,
    Write,
    "jira_sprints_write",
    additive_write,
    "Create Jira sprint",
    "Create a Jira Software sprint."
);
jira_metadata!(
    JIRA_UPDATE_SPRINT_METADATA,
    tools::JIRA_UPDATE_SPRINT_TOOL_NAME,
    Write,
    "jira_sprints_write",
    destructive_write,
    "Update Jira sprint",
    "Update a Jira Software sprint."
);
jira_metadata!(
    JIRA_ADD_ISSUES_TO_SPRINT_METADATA,
    tools::JIRA_ADD_ISSUES_TO_SPRINT_TOOL_NAME,
    Write,
    "jira_sprint_membership_write",
    destructive_write,
    "Add Jira issues to sprint",
    "Move Jira issues into a Jira Software sprint."
);
jira_metadata!(
    JIRA_GET_SERVICE_DESK_FOR_PROJECT_METADATA,
    tools::JIRA_GET_SERVICE_DESK_FOR_PROJECT_TOOL_NAME,
    Read,
    "jira_service_desks_read",
    read_only,
    "Get Jira project service desk",
    "Get the Jira Service Management service desk associated with a Jira project.",
    JiraProductDependencyResult
);
jira_metadata!(
    JIRA_LIST_SERVICE_DESK_QUEUES_METADATA,
    tools::JIRA_LIST_SERVICE_DESK_QUEUES_TOOL_NAME,
    Read,
    "jira_service_desks_read",
    read_only,
    "List Jira service desk queues",
    "List queues for a Jira Service Management service desk.",
    JiraProductDependencyResult
);
jira_metadata!(
    JIRA_LIST_SERVICE_DESK_QUEUE_ISSUES_METADATA,
    tools::JIRA_LIST_SERVICE_DESK_QUEUE_ISSUES_TOOL_NAME,
    Read,
    "jira_service_desks_read",
    read_only,
    "List Jira service desk queue issues",
    "List issues in a Jira Service Management queue.",
    JiraProductDependencyResult
);
jira_metadata!(
    JIRA_LIST_ISSUE_FORMS_METADATA,
    tools::JIRA_LIST_ISSUE_FORMS_TOOL_NAME,
    Read,
    "jira_issue_forms_read",
    read_only,
    "List Jira issue forms",
    "List Jira Forms or ProForma forms attached to an issue; requires Forms API availability and a Cloud ID.",
    JiraProductDependencyResult
);
jira_metadata!(
    JIRA_GET_ISSUE_FORM_METADATA,
    tools::JIRA_GET_ISSUE_FORM_TOOL_NAME,
    Read,
    "jira_issue_forms_read",
    read_only,
    "Get Jira issue form",
    "Get details and answers for one Jira Form or ProForma form; requires Forms API availability and a Cloud ID.",
    JiraProductDependencyResult
);
jira_metadata!(
    JIRA_UPDATE_ISSUE_FORM_ANSWERS_METADATA,
    tools::JIRA_UPDATE_ISSUE_FORM_ANSWERS_TOOL_NAME,
    Write,
    "jira_issue_forms_write",
    destructive_write,
    "Update Jira issue form answers",
    "Update answers on one Jira Form or ProForma form; requires Forms API availability and a Cloud ID.",
    JiraProductDependencyResult
);
jira_metadata!(
    JIRA_GET_ISSUE_TIMELINE_METADATA,
    tools::JIRA_GET_ISSUE_TIMELINE_TOOL_NAME,
    Read,
    "jira_issue_metrics_read",
    read_only,
    "Get Jira issue timeline",
    "Get Jira issue dates and optional status-change timeline derived from issue fields and changelog."
);
jira_metadata!(
    JIRA_GET_ISSUE_SLA_METRICS_METADATA,
    tools::JIRA_GET_ISSUE_SLA_METRICS_TOOL_NAME,
    Read,
    "jira_issue_metrics_read",
    read_only,
    "Get Jira issue SLA metrics",
    "Parse Jira Service Management SLA metrics from issue fields; does not recompute timers or working-hours calendars.",
    JiraProductDependencyResult
);
jira_metadata!(
    JIRA_GET_ISSUE_DEVELOPMENT_METADATA,
    tools::JIRA_GET_ISSUE_DEVELOPMENT_TOOL_NAME,
    Read,
    "jira_issue_development_read",
    read_only,
    "Get Jira issue development",
    "Get Jira Software development-status information for one issue.",
    JiraProductDependencyResult
);
jira_metadata!(
    JIRA_GET_ISSUES_DEVELOPMENT_METADATA,
    tools::JIRA_GET_ISSUES_DEVELOPMENT_TOOL_NAME,
    Read,
    "jira_issue_development_read",
    read_only,
    "Get Jira issues development",
    "Get Jira Software development-status information for multiple issues.",
    JiraProductDependencyResult
);

pub(super) const TOOLS: &[ToolMetadata] = &[
    JIRA_GET_ISSUE_METADATA,
    JIRA_SEARCH_METADATA,
    JIRA_GET_PROJECT_ISSUES_METADATA,
    JIRA_SEARCH_FIELDS_METADATA,
    JIRA_GET_FIELD_OPTIONS_METADATA,
    JIRA_ADD_COMMENT_METADATA,
    JIRA_EDIT_COMMENT_METADATA,
    JIRA_GET_TRANSITIONS_METADATA,
    JIRA_TRANSITION_ISSUE_METADATA,
    JIRA_CREATE_ISSUE_METADATA,
    JIRA_CREATE_ISSUES_METADATA,
    JIRA_GET_ISSUE_CHANGELOGS_METADATA,
    JIRA_UPDATE_ISSUE_METADATA,
    JIRA_DELETE_ISSUE_METADATA,
    JIRA_LIST_PROJECTS_METADATA,
    JIRA_LIST_PROJECT_VERSIONS_METADATA,
    JIRA_LIST_PROJECT_COMPONENTS_METADATA,
    JIRA_CREATE_PROJECT_VERSION_METADATA,
    JIRA_CREATE_PROJECT_VERSIONS_METADATA,
    JIRA_GET_USER_METADATA,
    JIRA_LIST_ISSUE_WATCHERS_METADATA,
    JIRA_ADD_WATCHER_METADATA,
    JIRA_REMOVE_WATCHER_METADATA,
    JIRA_LIST_ISSUE_WORKLOGS_METADATA,
    JIRA_ADD_WORKLOG_METADATA,
    JIRA_LIST_ISSUE_LINK_TYPES_METADATA,
    JIRA_SET_ISSUE_PARENT_METADATA,
    JIRA_CREATE_ISSUE_LINK_METADATA,
    JIRA_CREATE_REMOTE_ISSUE_LINK_METADATA,
    JIRA_DELETE_ISSUE_LINK_METADATA,
    JIRA_GET_ISSUE_ATTACHMENTS_METADATA,
    JIRA_GET_ISSUE_IMAGES_METADATA,
    JIRA_LIST_AGILE_BOARDS_METADATA,
    JIRA_LIST_BOARD_ISSUES_METADATA,
    JIRA_LIST_BOARD_SPRINTS_METADATA,
    JIRA_LIST_SPRINT_ISSUES_METADATA,
    JIRA_CREATE_SPRINT_METADATA,
    JIRA_UPDATE_SPRINT_METADATA,
    JIRA_ADD_ISSUES_TO_SPRINT_METADATA,
    JIRA_GET_SERVICE_DESK_FOR_PROJECT_METADATA,
    JIRA_LIST_SERVICE_DESK_QUEUES_METADATA,
    JIRA_LIST_SERVICE_DESK_QUEUE_ISSUES_METADATA,
    JIRA_LIST_ISSUE_FORMS_METADATA,
    JIRA_GET_ISSUE_FORM_METADATA,
    JIRA_UPDATE_ISSUE_FORM_ANSWERS_METADATA,
    JIRA_GET_ISSUE_TIMELINE_METADATA,
    JIRA_GET_ISSUE_SLA_METRICS_METADATA,
    JIRA_GET_ISSUE_DEVELOPMENT_METADATA,
    JIRA_GET_ISSUES_DEVELOPMENT_METADATA,
];

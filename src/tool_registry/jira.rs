use crate::jira::tools;

use super::{ToolAccess, ToolMetadata, ToolService};

pub const JIRA_GET_ISSUE_METADATA: ToolMetadata = ToolMetadata {
    name: tools::JIRA_GET_ISSUE_TOOL_NAME,
    service: ToolService::Jira,
    access: ToolAccess::Read,
    toolset: Some("jira_issues"),
    title: "Get Jira issue",
    description: "Get a Jira issue by key.",
};

pub const JIRA_SEARCH_METADATA: ToolMetadata = ToolMetadata {
    name: tools::JIRA_SEARCH_TOOL_NAME,
    service: ToolService::Jira,
    access: ToolAccess::Read,
    toolset: Some("jira_issues"),
    title: "Search Jira issues",
    description: "Search Jira issues with JQL.",
};

pub const JIRA_GET_PROJECT_ISSUES_METADATA: ToolMetadata = ToolMetadata {
    name: tools::JIRA_GET_PROJECT_ISSUES_TOOL_NAME,
    service: ToolService::Jira,
    access: ToolAccess::Read,
    toolset: Some("jira_issues"),
    title: "Get Jira project issues",
    description: "List Jira issues for a project.",
};

pub const JIRA_SEARCH_FIELDS_METADATA: ToolMetadata = ToolMetadata {
    name: tools::JIRA_SEARCH_FIELDS_TOOL_NAME,
    service: ToolService::Jira,
    access: ToolAccess::Read,
    toolset: Some("jira_fields"),
    title: "Search Jira fields",
    description: "Search Jira fields by keyword.",
};

pub const JIRA_GET_FIELD_OPTIONS_METADATA: ToolMetadata = ToolMetadata {
    name: tools::JIRA_GET_FIELD_OPTIONS_TOOL_NAME,
    service: ToolService::Jira,
    access: ToolAccess::Read,
    toolset: Some("jira_fields"),
    title: "Get Jira field options",
    description: "Get options for a Jira field.",
};

pub const JIRA_ADD_COMMENT_METADATA: ToolMetadata = ToolMetadata {
    name: tools::JIRA_ADD_COMMENT_TOOL_NAME,
    service: ToolService::Jira,
    access: ToolAccess::Write,
    toolset: Some("jira_comments"),
    title: "Add Jira comment",
    description: "Add a comment to a Jira issue.",
};

pub const JIRA_EDIT_COMMENT_METADATA: ToolMetadata = ToolMetadata {
    name: tools::JIRA_EDIT_COMMENT_TOOL_NAME,
    service: ToolService::Jira,
    access: ToolAccess::Write,
    toolset: Some("jira_comments"),
    title: "Edit Jira comment",
    description: "Edit a Jira issue comment.",
};

pub const JIRA_GET_TRANSITIONS_METADATA: ToolMetadata = ToolMetadata {
    name: tools::JIRA_GET_TRANSITIONS_TOOL_NAME,
    service: ToolService::Jira,
    access: ToolAccess::Read,
    toolset: Some("jira_transitions"),
    title: "Get Jira transitions",
    description: "Get available transitions for a Jira issue.",
};

pub const JIRA_TRANSITION_ISSUE_METADATA: ToolMetadata = ToolMetadata {
    name: tools::JIRA_TRANSITION_ISSUE_TOOL_NAME,
    service: ToolService::Jira,
    access: ToolAccess::Write,
    toolset: Some("jira_transitions"),
    title: "Transition Jira issue",
    description: "Transition a Jira issue.",
};

jira_metadata!(
    JIRA_CREATE_ISSUE_METADATA,
    tools::JIRA_CREATE_ISSUE_TOOL_NAME,
    Write,
    "jira_issues",
    "Create Jira issue",
    "Create a Jira issue."
);
jira_metadata!(
    JIRA_BATCH_CREATE_ISSUES_METADATA,
    tools::JIRA_BATCH_CREATE_ISSUES_TOOL_NAME,
    Write,
    "jira_issues",
    "Batch create Jira issues",
    "Create multiple Jira issues."
);
jira_metadata!(
    JIRA_BATCH_GET_CHANGELOGS_METADATA,
    tools::JIRA_BATCH_GET_CHANGELOGS_TOOL_NAME,
    Read,
    "jira_issues",
    "Batch get Jira changelogs",
    "Get changelogs for multiple Jira issues."
);
jira_metadata!(
    JIRA_UPDATE_ISSUE_METADATA,
    tools::JIRA_UPDATE_ISSUE_TOOL_NAME,
    Write,
    "jira_issues",
    "Update Jira issue",
    "Update fields on a Jira issue."
);
jira_metadata!(
    JIRA_DELETE_ISSUE_METADATA,
    tools::JIRA_DELETE_ISSUE_TOOL_NAME,
    Write,
    "jira_issues",
    "Delete Jira issue",
    "Delete a Jira issue."
);
jira_metadata!(
    JIRA_GET_ALL_PROJECTS_METADATA,
    tools::JIRA_GET_ALL_PROJECTS_TOOL_NAME,
    Read,
    "jira_projects",
    "Get all Jira projects",
    "List Jira projects visible to the current user."
);
jira_metadata!(
    JIRA_GET_PROJECT_VERSIONS_METADATA,
    tools::JIRA_GET_PROJECT_VERSIONS_TOOL_NAME,
    Read,
    "jira_projects",
    "Get Jira project versions",
    "List versions for a Jira project."
);
jira_metadata!(
    JIRA_GET_PROJECT_COMPONENTS_METADATA,
    tools::JIRA_GET_PROJECT_COMPONENTS_TOOL_NAME,
    Read,
    "jira_projects",
    "Get Jira project components",
    "List components for a Jira project."
);
jira_metadata!(
    JIRA_CREATE_VERSION_METADATA,
    tools::JIRA_CREATE_VERSION_TOOL_NAME,
    Write,
    "jira_projects",
    "Create Jira version",
    "Create a Jira project version."
);
jira_metadata!(
    JIRA_BATCH_CREATE_VERSIONS_METADATA,
    tools::JIRA_BATCH_CREATE_VERSIONS_TOOL_NAME,
    Write,
    "jira_projects",
    "Batch create Jira versions",
    "Create multiple Jira project versions."
);
jira_metadata!(
    JIRA_GET_USER_PROFILE_METADATA,
    tools::JIRA_GET_USER_PROFILE_TOOL_NAME,
    Read,
    "jira_users",
    "Get Jira user profile",
    "Retrieve a Jira user profile."
);
jira_metadata!(
    JIRA_GET_ISSUE_WATCHERS_METADATA,
    tools::JIRA_GET_ISSUE_WATCHERS_TOOL_NAME,
    Read,
    "jira_watchers",
    "Get Jira issue watchers",
    "List watchers for a Jira issue."
);
jira_metadata!(
    JIRA_ADD_WATCHER_METADATA,
    tools::JIRA_ADD_WATCHER_TOOL_NAME,
    Write,
    "jira_watchers",
    "Add Jira watcher",
    "Add a watcher to a Jira issue."
);
jira_metadata!(
    JIRA_REMOVE_WATCHER_METADATA,
    tools::JIRA_REMOVE_WATCHER_TOOL_NAME,
    Write,
    "jira_watchers",
    "Remove Jira watcher",
    "Remove a watcher from a Jira issue."
);
jira_metadata!(
    JIRA_GET_WORKLOG_METADATA,
    tools::JIRA_GET_WORKLOG_TOOL_NAME,
    Read,
    "jira_worklog",
    "Get Jira worklog",
    "List worklogs for a Jira issue."
);
jira_metadata!(
    JIRA_ADD_WORKLOG_METADATA,
    tools::JIRA_ADD_WORKLOG_TOOL_NAME,
    Write,
    "jira_worklog",
    "Add Jira worklog",
    "Add a worklog entry to a Jira issue."
);
jira_metadata!(
    JIRA_GET_LINK_TYPES_METADATA,
    tools::JIRA_GET_LINK_TYPES_TOOL_NAME,
    Read,
    "jira_links",
    "Get Jira link types",
    "List Jira issue link types."
);
jira_metadata!(
    JIRA_LINK_TO_EPIC_METADATA,
    tools::JIRA_LINK_TO_EPIC_TOOL_NAME,
    Write,
    "jira_links",
    "Link Jira issue to epic",
    "Link a Jira issue to an epic."
);
jira_metadata!(
    JIRA_CREATE_ISSUE_LINK_METADATA,
    tools::JIRA_CREATE_ISSUE_LINK_TOOL_NAME,
    Write,
    "jira_links",
    "Create Jira issue link",
    "Create a Jira issue link."
);
jira_metadata!(
    JIRA_CREATE_REMOTE_ISSUE_LINK_METADATA,
    tools::JIRA_CREATE_REMOTE_ISSUE_LINK_TOOL_NAME,
    Write,
    "jira_links",
    "Create Jira remote issue link",
    "Create a remote link on a Jira issue."
);
jira_metadata!(
    JIRA_REMOVE_ISSUE_LINK_METADATA,
    tools::JIRA_REMOVE_ISSUE_LINK_TOOL_NAME,
    Write,
    "jira_links",
    "Remove Jira issue link",
    "Remove a Jira issue link."
);
jira_metadata!(
    JIRA_DOWNLOAD_ATTACHMENTS_METADATA,
    tools::JIRA_DOWNLOAD_ATTACHMENTS_TOOL_NAME,
    Read,
    "jira_attachments",
    "Download Jira attachments",
    "Fetch Jira attachment metadata and bounded content."
);
jira_metadata!(
    JIRA_GET_ISSUE_IMAGES_METADATA,
    tools::JIRA_GET_ISSUE_IMAGES_TOOL_NAME,
    Read,
    "jira_attachments",
    "Get Jira issue images",
    "Fetch image attachments for a Jira issue."
);
jira_metadata!(
    JIRA_GET_AGILE_BOARDS_METADATA,
    tools::JIRA_GET_AGILE_BOARDS_TOOL_NAME,
    Read,
    "jira_agile",
    "Get Jira agile boards",
    "List Jira Software agile boards."
);
jira_metadata!(
    JIRA_GET_BOARD_ISSUES_METADATA,
    tools::JIRA_GET_BOARD_ISSUES_TOOL_NAME,
    Read,
    "jira_agile",
    "Get Jira board issues",
    "List issues from a Jira Software board."
);
jira_metadata!(
    JIRA_GET_SPRINTS_FROM_BOARD_METADATA,
    tools::JIRA_GET_SPRINTS_FROM_BOARD_TOOL_NAME,
    Read,
    "jira_agile",
    "Get Jira board sprints",
    "List sprints from a Jira Software board."
);
jira_metadata!(
    JIRA_GET_SPRINT_ISSUES_METADATA,
    tools::JIRA_GET_SPRINT_ISSUES_TOOL_NAME,
    Read,
    "jira_agile",
    "Get Jira sprint issues",
    "List issues from a Jira Software sprint."
);
jira_metadata!(
    JIRA_CREATE_SPRINT_METADATA,
    tools::JIRA_CREATE_SPRINT_TOOL_NAME,
    Write,
    "jira_agile",
    "Create Jira sprint",
    "Create a Jira Software sprint."
);
jira_metadata!(
    JIRA_UPDATE_SPRINT_METADATA,
    tools::JIRA_UPDATE_SPRINT_TOOL_NAME,
    Write,
    "jira_agile",
    "Update Jira sprint",
    "Update a Jira Software sprint."
);
jira_metadata!(
    JIRA_ADD_ISSUES_TO_SPRINT_METADATA,
    tools::JIRA_ADD_ISSUES_TO_SPRINT_TOOL_NAME,
    Write,
    "jira_agile",
    "Add Jira issues to sprint",
    "Add Jira issues to a sprint."
);
jira_metadata!(
    JIRA_GET_SERVICE_DESK_FOR_PROJECT_METADATA,
    tools::JIRA_GET_SERVICE_DESK_FOR_PROJECT_TOOL_NAME,
    Read,
    "jira_service_desk",
    "Get service desk for Jira project",
    "Get the Jira Service Management desk for a project."
);
jira_metadata!(
    JIRA_GET_SERVICE_DESK_QUEUES_METADATA,
    tools::JIRA_GET_SERVICE_DESK_QUEUES_TOOL_NAME,
    Read,
    "jira_service_desk",
    "Get Jira service desk queues",
    "List queues for a Jira Service Management desk."
);
jira_metadata!(
    JIRA_GET_QUEUE_ISSUES_METADATA,
    tools::JIRA_GET_QUEUE_ISSUES_TOOL_NAME,
    Read,
    "jira_service_desk",
    "Get Jira service desk queue issues",
    "List issues in a Jira Service Management queue."
);
jira_metadata!(
    JIRA_GET_ISSUE_PROFORMA_FORMS_METADATA,
    tools::JIRA_GET_ISSUE_PROFORMA_FORMS_TOOL_NAME,
    Read,
    "jira_forms",
    "Get Jira issue forms",
    "List Jira Forms or ProForma forms for an issue."
);
jira_metadata!(
    JIRA_GET_PROFORMA_FORM_DETAILS_METADATA,
    tools::JIRA_GET_PROFORMA_FORM_DETAILS_TOOL_NAME,
    Read,
    "jira_forms",
    "Get Jira form details",
    "Get details for a Jira Form or ProForma form."
);
jira_metadata!(
    JIRA_UPDATE_PROFORMA_FORM_ANSWERS_METADATA,
    tools::JIRA_UPDATE_PROFORMA_FORM_ANSWERS_TOOL_NAME,
    Write,
    "jira_forms",
    "Update Jira form answers",
    "Update answers on a Jira Form or ProForma form."
);
jira_metadata!(
    JIRA_GET_ISSUE_DATES_METADATA,
    tools::JIRA_GET_ISSUE_DATES_TOOL_NAME,
    Read,
    "jira_metrics",
    "Get Jira issue dates",
    "Get Jira issue date and status timing information."
);
jira_metadata!(
    JIRA_GET_ISSUE_SLA_METADATA,
    tools::JIRA_GET_ISSUE_SLA_TOOL_NAME,
    Read,
    "jira_metrics",
    "Get Jira issue SLA",
    "Get SLA metrics for a Jira issue."
);
jira_metadata!(
    JIRA_GET_ISSUE_DEVELOPMENT_INFO_METADATA,
    tools::JIRA_GET_ISSUE_DEVELOPMENT_INFO_TOOL_NAME,
    Read,
    "jira_development",
    "Get Jira issue development info",
    "Get development information for a Jira issue."
);
jira_metadata!(
    JIRA_GET_ISSUES_DEVELOPMENT_INFO_METADATA,
    tools::JIRA_GET_ISSUES_DEVELOPMENT_INFO_TOOL_NAME,
    Read,
    "jira_development",
    "Get Jira issues development info",
    "Get development information for multiple Jira issues."
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
    JIRA_BATCH_CREATE_ISSUES_METADATA,
    JIRA_BATCH_GET_CHANGELOGS_METADATA,
    JIRA_UPDATE_ISSUE_METADATA,
    JIRA_DELETE_ISSUE_METADATA,
    JIRA_GET_ALL_PROJECTS_METADATA,
    JIRA_GET_PROJECT_VERSIONS_METADATA,
    JIRA_GET_PROJECT_COMPONENTS_METADATA,
    JIRA_CREATE_VERSION_METADATA,
    JIRA_BATCH_CREATE_VERSIONS_METADATA,
    JIRA_GET_USER_PROFILE_METADATA,
    JIRA_GET_ISSUE_WATCHERS_METADATA,
    JIRA_ADD_WATCHER_METADATA,
    JIRA_REMOVE_WATCHER_METADATA,
    JIRA_GET_WORKLOG_METADATA,
    JIRA_ADD_WORKLOG_METADATA,
    JIRA_GET_LINK_TYPES_METADATA,
    JIRA_LINK_TO_EPIC_METADATA,
    JIRA_CREATE_ISSUE_LINK_METADATA,
    JIRA_CREATE_REMOTE_ISSUE_LINK_METADATA,
    JIRA_REMOVE_ISSUE_LINK_METADATA,
    JIRA_DOWNLOAD_ATTACHMENTS_METADATA,
    JIRA_GET_ISSUE_IMAGES_METADATA,
    JIRA_GET_AGILE_BOARDS_METADATA,
    JIRA_GET_BOARD_ISSUES_METADATA,
    JIRA_GET_SPRINTS_FROM_BOARD_METADATA,
    JIRA_GET_SPRINT_ISSUES_METADATA,
    JIRA_CREATE_SPRINT_METADATA,
    JIRA_UPDATE_SPRINT_METADATA,
    JIRA_ADD_ISSUES_TO_SPRINT_METADATA,
    JIRA_GET_SERVICE_DESK_FOR_PROJECT_METADATA,
    JIRA_GET_SERVICE_DESK_QUEUES_METADATA,
    JIRA_GET_QUEUE_ISSUES_METADATA,
    JIRA_GET_ISSUE_PROFORMA_FORMS_METADATA,
    JIRA_GET_PROFORMA_FORM_DETAILS_METADATA,
    JIRA_UPDATE_PROFORMA_FORM_ANSWERS_METADATA,
    JIRA_GET_ISSUE_DATES_METADATA,
    JIRA_GET_ISSUE_SLA_METADATA,
    JIRA_GET_ISSUE_DEVELOPMENT_INFO_METADATA,
    JIRA_GET_ISSUES_DEVELOPMENT_INFO_METADATA,
];

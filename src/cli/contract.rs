use crate::{confluence::tools as confluence_tools, gitlab::tools as gitlab_tools, jira::tools};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CliCapability {
    pub provider: &'static str,
    pub capability: &'static str,
    pub command_path: &'static str,
    pub tool_name: &'static str,
}

pub const JIRA_CAPABILITIES: &[CliCapability] = &[
    cap(
        "jira",
        "Get issue",
        "jira issue get",
        tools::JIRA_GET_ISSUE_TOOL_NAME,
    ),
    cap(
        "jira",
        "Search issues",
        "jira issue search",
        tools::JIRA_SEARCH_TOOL_NAME,
    ),
    cap(
        "jira",
        "List project issues",
        "jira project issues",
        tools::JIRA_GET_PROJECT_ISSUES_TOOL_NAME,
    ),
    cap(
        "jira",
        "Search fields",
        "jira field search",
        tools::JIRA_SEARCH_FIELDS_TOOL_NAME,
    ),
    cap(
        "jira",
        "List field options",
        "jira field options",
        tools::JIRA_GET_FIELD_OPTIONS_TOOL_NAME,
    ),
    cap(
        "jira",
        "Add issue comment",
        "jira issue comment add",
        tools::JIRA_ADD_COMMENT_TOOL_NAME,
    ),
    cap(
        "jira",
        "Update issue comment",
        "jira issue comment update",
        tools::JIRA_EDIT_COMMENT_TOOL_NAME,
    ),
    cap(
        "jira",
        "List transitions",
        "jira issue transition list",
        tools::JIRA_GET_TRANSITIONS_TOOL_NAME,
    ),
    cap(
        "jira",
        "Transition issue",
        "jira issue transition apply",
        tools::JIRA_TRANSITION_ISSUE_TOOL_NAME,
    ),
    cap(
        "jira",
        "Create issue",
        "jira issue create",
        tools::JIRA_CREATE_ISSUE_TOOL_NAME,
    ),
    cap(
        "jira",
        "Create issues",
        "jira issue create-batch",
        tools::JIRA_CREATE_ISSUES_TOOL_NAME,
    ),
    cap(
        "jira",
        "Get changelogs",
        "jira issue changelog batch",
        tools::JIRA_GET_ISSUE_CHANGELOGS_TOOL_NAME,
    ),
    cap(
        "jira",
        "Update issue",
        "jira issue update",
        tools::JIRA_UPDATE_ISSUE_TOOL_NAME,
    ),
    cap(
        "jira",
        "Delete issue",
        "jira issue delete",
        tools::JIRA_DELETE_ISSUE_TOOL_NAME,
    ),
    cap(
        "jira",
        "List projects",
        "jira project list",
        tools::JIRA_LIST_PROJECTS_TOOL_NAME,
    ),
    cap(
        "jira",
        "List versions",
        "jira project version list",
        tools::JIRA_LIST_PROJECT_VERSIONS_TOOL_NAME,
    ),
    cap(
        "jira",
        "List components",
        "jira project component list",
        tools::JIRA_LIST_PROJECT_COMPONENTS_TOOL_NAME,
    ),
    cap(
        "jira",
        "Create version",
        "jira project version create",
        tools::JIRA_CREATE_PROJECT_VERSION_TOOL_NAME,
    ),
    cap(
        "jira",
        "Create versions",
        "jira project version create-batch",
        tools::JIRA_CREATE_PROJECT_VERSIONS_TOOL_NAME,
    ),
    cap(
        "jira",
        "Get user",
        "jira user get",
        tools::JIRA_GET_USER_TOOL_NAME,
    ),
    cap(
        "jira",
        "List watchers",
        "jira issue watcher list",
        tools::JIRA_LIST_ISSUE_WATCHERS_TOOL_NAME,
    ),
    cap(
        "jira",
        "Add watcher",
        "jira issue watcher add",
        tools::JIRA_ADD_WATCHER_TOOL_NAME,
    ),
    cap(
        "jira",
        "Remove watcher",
        "jira issue watcher remove",
        tools::JIRA_REMOVE_WATCHER_TOOL_NAME,
    ),
    cap(
        "jira",
        "List worklogs",
        "jira issue worklog list",
        tools::JIRA_LIST_ISSUE_WORKLOGS_TOOL_NAME,
    ),
    cap(
        "jira",
        "Add worklog",
        "jira issue worklog add",
        tools::JIRA_ADD_WORKLOG_TOOL_NAME,
    ),
    cap(
        "jira",
        "List link types",
        "jira issue link-type list",
        tools::JIRA_LIST_ISSUE_LINK_TYPES_TOOL_NAME,
    ),
    cap(
        "jira",
        "Set parent",
        "jira issue parent set",
        tools::JIRA_SET_ISSUE_PARENT_TOOL_NAME,
    ),
    cap(
        "jira",
        "Create issue link",
        "jira issue link create",
        tools::JIRA_CREATE_ISSUE_LINK_TOOL_NAME,
    ),
    cap(
        "jira",
        "Create remote link",
        "jira issue remote-link create",
        tools::JIRA_CREATE_REMOTE_ISSUE_LINK_TOOL_NAME,
    ),
    cap(
        "jira",
        "Delete issue link",
        "jira issue link delete",
        tools::JIRA_DELETE_ISSUE_LINK_TOOL_NAME,
    ),
    cap(
        "jira",
        "Get attachments",
        "jira issue attachment list",
        tools::JIRA_GET_ISSUE_ATTACHMENTS_TOOL_NAME,
    ),
    cap(
        "jira",
        "Get image attachments",
        "jira issue attachment images",
        tools::JIRA_GET_ISSUE_IMAGES_TOOL_NAME,
    ),
    cap(
        "jira",
        "List agile boards",
        "jira agile board list",
        tools::JIRA_LIST_AGILE_BOARDS_TOOL_NAME,
    ),
    cap(
        "jira",
        "List board issues",
        "jira agile board issues",
        tools::JIRA_LIST_BOARD_ISSUES_TOOL_NAME,
    ),
    cap(
        "jira",
        "List board sprints",
        "jira agile board sprints",
        tools::JIRA_LIST_BOARD_SPRINTS_TOOL_NAME,
    ),
    cap(
        "jira",
        "List sprint issues",
        "jira agile sprint issues",
        tools::JIRA_LIST_SPRINT_ISSUES_TOOL_NAME,
    ),
    cap(
        "jira",
        "Create sprint",
        "jira agile sprint create",
        tools::JIRA_CREATE_SPRINT_TOOL_NAME,
    ),
    cap(
        "jira",
        "Update sprint",
        "jira agile sprint update",
        tools::JIRA_UPDATE_SPRINT_TOOL_NAME,
    ),
    cap(
        "jira",
        "Add issues to sprint",
        "jira agile sprint add-issues",
        tools::JIRA_ADD_ISSUES_TO_SPRINT_TOOL_NAME,
    ),
    cap(
        "jira",
        "Get project service desk",
        "jira service-desk project",
        tools::JIRA_GET_SERVICE_DESK_FOR_PROJECT_TOOL_NAME,
    ),
    cap(
        "jira",
        "List service desk queues",
        "jira service-desk queue list",
        tools::JIRA_LIST_SERVICE_DESK_QUEUES_TOOL_NAME,
    ),
    cap(
        "jira",
        "List service desk queue issues",
        "jira service-desk queue issues",
        tools::JIRA_LIST_SERVICE_DESK_QUEUE_ISSUES_TOOL_NAME,
    ),
    cap(
        "jira",
        "Get issue timeline",
        "jira issue timeline",
        tools::JIRA_GET_ISSUE_TIMELINE_TOOL_NAME,
    ),
    cap(
        "jira",
        "Get SLA metrics",
        "jira issue sla",
        tools::JIRA_GET_ISSUE_SLA_METRICS_TOOL_NAME,
    ),
    cap(
        "jira",
        "Get development",
        "jira issue development",
        tools::JIRA_GET_ISSUE_DEVELOPMENT_TOOL_NAME,
    ),
    cap(
        "jira",
        "Get batch development",
        "jira issue development-batch",
        tools::JIRA_GET_ISSUES_DEVELOPMENT_TOOL_NAME,
    ),
];

pub const CONFLUENCE_CAPABILITIES: &[CliCapability] = &[
    cap(
        "confluence",
        "Search content",
        "confluence content search",
        confluence_tools::CONFLUENCE_SEARCH_TOOL_NAME,
    ),
    cap(
        "confluence",
        "Get page",
        "confluence page get",
        confluence_tools::CONFLUENCE_GET_PAGE_TOOL_NAME,
    ),
    cap(
        "confluence",
        "List page children",
        "confluence page children",
        confluence_tools::CONFLUENCE_LIST_PAGE_CHILDREN_TOOL_NAME,
    ),
    cap(
        "confluence",
        "Get space page tree",
        "confluence page tree",
        confluence_tools::CONFLUENCE_GET_SPACE_PAGE_TREE_TOOL_NAME,
    ),
    cap(
        "confluence",
        "Create page",
        "confluence page create",
        confluence_tools::CONFLUENCE_CREATE_PAGE_TOOL_NAME,
    ),
    cap(
        "confluence",
        "Update page",
        "confluence page update",
        confluence_tools::CONFLUENCE_UPDATE_PAGE_TOOL_NAME,
    ),
    cap(
        "confluence",
        "Delete page",
        "confluence page delete",
        confluence_tools::CONFLUENCE_DELETE_PAGE_TOOL_NAME,
    ),
    cap(
        "confluence",
        "Move page",
        "confluence page move",
        confluence_tools::CONFLUENCE_MOVE_PAGE_TOOL_NAME,
    ),
    cap(
        "confluence",
        "List comments",
        "confluence page comment list",
        confluence_tools::CONFLUENCE_LIST_PAGE_COMMENTS_TOOL_NAME,
    ),
    cap(
        "confluence",
        "Add comment",
        "confluence page comment add",
        confluence_tools::CONFLUENCE_ADD_COMMENT_TOOL_NAME,
    ),
    cap(
        "confluence",
        "Reply to comment",
        "confluence page comment reply",
        confluence_tools::CONFLUENCE_REPLY_TO_COMMENT_TOOL_NAME,
    ),
    cap(
        "confluence",
        "List labels",
        "confluence content label list",
        confluence_tools::CONFLUENCE_LIST_CONTENT_LABELS_TOOL_NAME,
    ),
    cap(
        "confluence",
        "Add label",
        "confluence content label add",
        confluence_tools::CONFLUENCE_ADD_LABEL_TOOL_NAME,
    ),
    cap(
        "confluence",
        "Search users",
        "confluence user search",
        confluence_tools::CONFLUENCE_SEARCH_USER_TOOL_NAME,
    ),
    cap(
        "confluence",
        "Get page version",
        "confluence page version get",
        confluence_tools::CONFLUENCE_GET_PAGE_VERSION_TOOL_NAME,
    ),
    cap(
        "confluence",
        "Get page diff",
        "confluence page version diff",
        confluence_tools::CONFLUENCE_GET_PAGE_DIFF_TOOL_NAME,
    ),
    cap(
        "confluence",
        "Get page analytics",
        "confluence page analytics views",
        confluence_tools::CONFLUENCE_GET_PAGE_VIEW_ANALYTICS_TOOL_NAME,
    ),
    cap(
        "confluence",
        "Upload attachment",
        "confluence attachment upload",
        confluence_tools::CONFLUENCE_UPLOAD_CONTENT_ATTACHMENT_TOOL_NAME,
    ),
    cap(
        "confluence",
        "Upload attachments",
        "confluence attachment upload-batch",
        confluence_tools::CONFLUENCE_UPLOAD_CONTENT_ATTACHMENTS_TOOL_NAME,
    ),
    cap(
        "confluence",
        "List attachments",
        "confluence attachment list",
        confluence_tools::CONFLUENCE_LIST_CONTENT_ATTACHMENTS_TOOL_NAME,
    ),
    cap(
        "confluence",
        "Download attachment",
        "confluence attachment download",
        confluence_tools::CONFLUENCE_DOWNLOAD_ATTACHMENT_TOOL_NAME,
    ),
    cap(
        "confluence",
        "Download content attachments",
        "confluence attachment download-content",
        confluence_tools::CONFLUENCE_DOWNLOAD_CONTENT_ATTACHMENTS_TOOL_NAME,
    ),
    cap(
        "confluence",
        "Delete attachment",
        "confluence attachment delete",
        confluence_tools::CONFLUENCE_DELETE_ATTACHMENT_TOOL_NAME,
    ),
    cap(
        "confluence",
        "Get image attachments",
        "confluence attachment images",
        confluence_tools::CONFLUENCE_GET_CONTENT_IMAGE_ATTACHMENTS_TOOL_NAME,
    ),
];

pub const GITLAB_CAPABILITIES: &[CliCapability] = &[
    cap(
        "gitlab",
        "Get current user",
        "gitlab user current",
        gitlab_tools::GITLAB_GET_CURRENT_USER_TOOL_NAME,
    ),
    cap(
        "gitlab",
        "Get project",
        "gitlab project get",
        gitlab_tools::GITLAB_GET_PROJECT_TOOL_NAME,
    ),
    cap(
        "gitlab",
        "List merge requests",
        "gitlab mr list",
        gitlab_tools::GITLAB_LIST_MERGE_REQUESTS_TOOL_NAME,
    ),
    cap(
        "gitlab",
        "Get merge request",
        "gitlab mr get",
        gitlab_tools::GITLAB_GET_MERGE_REQUEST_TOOL_NAME,
    ),
    cap(
        "gitlab",
        "List commits",
        "gitlab mr commits",
        gitlab_tools::GITLAB_LIST_MERGE_REQUEST_COMMITS_TOOL_NAME,
    ),
    cap(
        "gitlab",
        "List diffs",
        "gitlab mr diffs",
        gitlab_tools::GITLAB_LIST_MERGE_REQUEST_DIFFS_TOOL_NAME,
    ),
    cap(
        "gitlab",
        "List pipelines",
        "gitlab mr pipelines",
        gitlab_tools::GITLAB_LIST_MERGE_REQUEST_PIPELINES_TOOL_NAME,
    ),
    cap(
        "gitlab",
        "Create MR",
        "gitlab mr create",
        gitlab_tools::GITLAB_CREATE_MERGE_REQUEST_TOOL_NAME,
    ),
    cap(
        "gitlab",
        "Update MR",
        "gitlab mr update",
        gitlab_tools::GITLAB_UPDATE_MERGE_REQUEST_TOOL_NAME,
    ),
    cap(
        "gitlab",
        "Add note",
        "gitlab mr note add",
        gitlab_tools::GITLAB_ADD_MERGE_REQUEST_NOTE_TOOL_NAME,
    ),
    cap(
        "gitlab",
        "Reply discussion",
        "gitlab mr discussion reply",
        gitlab_tools::GITLAB_REPLY_MERGE_REQUEST_DISCUSSION_TOOL_NAME,
    ),
    cap(
        "gitlab",
        "Resolve discussion",
        "gitlab mr discussion resolve",
        gitlab_tools::GITLAB_RESOLVE_MERGE_REQUEST_DISCUSSION_TOOL_NAME,
    ),
    cap(
        "gitlab",
        "Get approval state",
        "gitlab mr approval get",
        gitlab_tools::GITLAB_GET_MERGE_REQUEST_APPROVAL_STATE_TOOL_NAME,
    ),
    cap(
        "gitlab",
        "Set approval",
        "gitlab mr approval set",
        gitlab_tools::GITLAB_SET_MERGE_REQUEST_APPROVAL_TOOL_NAME,
    ),
    cap(
        "gitlab",
        "Accept MR",
        "gitlab mr merge",
        gitlab_tools::GITLAB_ACCEPT_MERGE_REQUEST_TOOL_NAME,
    ),
];

pub const SHARED_PARAMETER_FLAGS: &[&str] = &[
    "priority",
    "labels",
    "fix-versions",
    "released",
    "archived",
    "filename-contains",
    "media-type",
    "max-bytes",
    "name",
    "include-counts",
    "context-lines",
    "include-title",
];

pub const CLI_ONLY_PARAMETER_FLAGS: &[&str] = &[
    "env-file",
    "json",
    "pretty",
    "body-file",
    "body-stdin",
    "comment-file",
    "comment-stdin",
    "description-file",
    "description-stdin",
    "content-file",
    "content-stdin",
    "issues-file",
    "versions-file",
    "files",
];

const fn cap(
    provider: &'static str,
    capability: &'static str,
    command_path: &'static str,
    tool_name: &'static str,
) -> CliCapability {
    CliCapability {
        provider,
        capability,
        command_path,
        tool_name,
    }
}

pub fn all_capabilities() -> impl Iterator<Item = CliCapability> {
    JIRA_CAPABILITIES
        .iter()
        .chain(CONFLUENCE_CAPABILITIES)
        .chain(GITLAB_CAPABILITIES)
        .copied()
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;

    use crate::cli;

    use super::*;

    #[test]
    fn cli_contract_covers_all_public_business_capabilities() {
        assert_eq!(JIRA_CAPABILITIES.len(), 46);
        assert_eq!(
            CONFLUENCE_CAPABILITIES.len(),
            confluence_tools::CONFLUENCE_TOOL_NAMES.len()
        );
        assert_eq!(
            GITLAB_CAPABILITIES.len(),
            gitlab_tools::GITLAB_TOOL_NAMES.len()
        );
        assert_eq!(all_capabilities().count(), 85);
    }

    #[test]
    fn cli_contract_paths_are_unique_and_resource_oriented() {
        let mut paths = BTreeSet::new();

        for capability in all_capabilities() {
            assert!(
                paths.insert(capability.command_path),
                "duplicate CLI path {}",
                capability.command_path
            );
            assert!(
                !capability.command_path.contains(" call ")
                    && !capability.command_path.contains(" schema ")
                    && !capability.command_path.contains(" tools "),
                "raw fallback path leaked into CLI contract: {}",
                capability.command_path
            );
            assert!(
                !capability.command_path.contains(capability.tool_name),
                "MCP tool name leaked into CLI path: {}",
                capability.command_path
            );
        }
    }

    #[test]
    fn cli_contract_tracks_shared_and_cli_only_parameter_boundaries() {
        for flag in [
            "priority",
            "labels",
            "fix-versions",
            "released",
            "archived",
            "filename-contains",
            "media-type",
            "max-bytes",
            "name",
            "include-counts",
            "context-lines",
            "include-title",
        ] {
            assert!(SHARED_PARAMETER_FLAGS.contains(&flag));
        }

        for flag in CLI_ONLY_PARAMETER_FLAGS {
            assert!(
                flag.ends_with("-file")
                    || flag.ends_with("-stdin")
                    || matches!(*flag, "env-file" | "json" | "pretty" | "files"),
                "CLI-only flag must be limited to input/output medium: {flag}"
            );
        }
    }

    #[test]
    fn cli_help_exposes_providers_and_not_raw_fallbacks() {
        let help = cli::cli_command().render_help().to_string();

        for expected in [
            "jira",
            "confluence",
            "gitlab",
            "--env-file",
            "--json",
            "--pretty",
        ] {
            assert!(help.contains(expected), "missing {expected} from CLI help");
        }
        for forbidden in [
            " call",
            " schema",
            " tools",
            "--token",
            "--password",
            "--url",
        ] {
            assert!(
                !help.contains(forbidden),
                "forbidden CLI help fragment leaked: {forbidden}"
            );
        }
    }
}

# Jira Commands

This module covers `workhub cli jira ...` commands.

When `jira issue search` omits `--fields`, the default fields are `key,summary,status,assignee,reporter,issuetype,priority,project`.

## Example Input Files

Description file `/tmp/workhub-jira-description.md`:

```markdown
Implement the CLI example.

- Add validation
- Update the user guide
```

Field update file `/tmp/workhub-jira-fields.json`:

```json
{
  "summary": "Updated summary",
  "description": "Updated plain text description",
  "assignee": "account-id-or-username"
}
```

Additional fields file `/tmp/workhub-jira-additional-fields.json`:

```json
{
  "customfield_10000": "Example value",
  "duedate": "2026-07-01"
}
```

Visibility file `/tmp/workhub-jira-visibility.json`:

```json
{
  "type": "role",
  "value": "Developers"
}
```

Transition fields file `/tmp/workhub-jira-transition-fields.json`:

```json
{
  "resolution": {
    "name": "Done"
  }
}
```

Batch issue create file `/tmp/workhub-jira-issues.json`:

```json
[
  {
    "project_key": "ABC",
    "issue_type": "Task",
    "summary": "First task",
    "description": "Create the first task",
    "priority": "Medium",
    "labels": ["cli", "example"]
  },
  {
    "project_key": "ABC",
    "issue_type": "Bug",
    "summary": "Second issue",
    "customfield_10000": "Extra value"
  }
]
```

Batch versions file `/tmp/workhub-jira-versions.json`:

```json
[
  {
    "name": "2026.07",
    "description": "July release",
    "releaseDate": "2026-07-31",
    "released": false
  }
]
```

Remote-link status file `/tmp/workhub-jira-remote-status.json`:

```json
{
  "resolved": false,
  "icon": {
    "url16x16": "https://example.invalid/icon.png",
    "title": "External status"
  }
}
```

## Issue Reads

| Command | Use | Example | Returns |
| --- | --- | --- | --- |
| `cli jira issue get <issue-key> [--fields <csv>] [--expand <csv>] [--comment-limit <n>] [--properties <csv>] [--update-history <bool>]` | Get one issue by key. | `workhub cli jira issue get ABC-1 --fields summary,status,assignee --comment-limit 5` | Issue fields by default; issue object with `--json`. |
| `cli jira issue search --jql <jql> [--fields <csv>] [--limit <n>] [--start-at <n>] [--projects <csv>] [--expand <csv>] [--page-token <token>]` | Search issues with JQL. | `workhub cli jira issue search --jql 'project = ABC ORDER BY updated DESC' --limit 10` | Issue table and paging fields by default; search result object with `--json`. |
| `cli jira project issues <project-key> [--limit <n>] [--start-at <n>]` | List issues for a project. | `workhub cli jira project issues ABC --limit 25` | Issue table by default; issue list and paging fields with `--json`. |

## Issue Comments

Comment body input uses one source at a time: `--body`, `--body-file`, or `--body-stdin`.

| Command | Use | Example | Returns |
| --- | --- | --- | --- |
| `cli jira issue comment add <issue-key> (--body <text> \| --body-file <path> \| --body-stdin) [--visibility-json <json> \| --visibility-file <path>]` | Add an issue comment. | `workhub cli jira issue comment add ABC-1 --body-file /tmp/comment.md --visibility-file /tmp/workhub-jira-visibility.json` | Created comment summary by default; comment object with `--json`. |
| `cli jira issue comment update <issue-key> <comment-id> (--body <text> \| --body-file <path> \| --body-stdin) [--visibility-json <json> \| --visibility-file <path>]` | Update an existing issue comment. | `workhub cli jira issue comment update ABC-1 10010 --body 'Updated comment'` | Updated comment summary by default; comment object with `--json`. |

## Issue Transitions

| Command | Use | Example | Returns |
| --- | --- | --- | --- |
| `cli jira issue transition list <issue-key>` | List available transitions for an issue. | `workhub cli jira issue transition list ABC-1` | Transition table by default; transition list with `--json`. |
| `cli jira issue transition apply <issue-key> <transition-id> [--fields-json <json> \| --fields-file <path>] [--comment <text> \| --comment-file <path> \| --comment-stdin]` | Apply a transition, optionally with fields and a comment. | `workhub cli jira issue transition apply ABC-1 31 --fields-file /tmp/workhub-jira-transition-fields.json --comment 'Moving to Done'` | Mutation summary by default; operation result with `--json`. |

## Issue Writes

| Command | Use | Example | Returns |
| --- | --- | --- | --- |
| `cli jira issue create --project <key> --issue-type <name> --summary <text> [--description <text> \| --description-file <path> \| --description-stdin] [--assignee <id-or-name>] [--priority <name>] [--labels <csv>] [--components <csv>] [--fix-versions <csv>] [--additional-fields-json <json> \| --additional-fields-file <path>]` | Create one issue. | `workhub cli jira issue create --project ABC --issue-type Task --summary 'Add CLI example' --description-file /tmp/workhub-jira-description.md --priority Medium --labels cli,docs --additional-fields-file /tmp/workhub-jira-additional-fields.json` | Created issue summary by default, usually including key and id; create result with `--json`. |
| `cli jira issue create-batch --issues-file <path> [--validate-only]` | Create issues from a JSON file. With `--validate-only`, validate without creating. | `workhub cli jira issue create-batch --issues-file /tmp/workhub-jira-issues.json --validate-only` | Per-issue create or validation results by default; batch result object with `--json`. |
| `cli jira issue update <issue-key> (--fields-json <json> \| --fields-file <path>) [--notify-users <bool>]` | Update issue fields. | `workhub cli jira issue update ABC-1 --fields-file /tmp/workhub-jira-fields.json --notify-users false` | Mutation summary by default; update result with `--json`. |
| `cli jira issue delete <issue-key> [--delete-subtasks]` | Delete an issue, optionally including subtasks. | `workhub cli jira issue delete ABC-99 --delete-subtasks` | Delete summary by default; operation result with `--json`. |

## History, Watchers, And Worklogs

| Command | Use | Example | Returns |
| --- | --- | --- | --- |
| `cli jira issue changelog batch --issue-ids <csv> [--limit <n>] [--field-ids <csv>]` | Get changelogs for multiple issues. | `workhub cli jira issue changelog batch --issue-ids ABC-1,ABC-2 --field-ids status,assignee --limit 20` | Changelog summary by issue by default; changelog object with `--json`. |
| `cli jira issue watcher list <issue-key>` | List issue watchers. | `workhub cli jira issue watcher list ABC-1` | Watcher table by default; watcher list with `--json`. |
| `cli jira issue watcher add <issue-key> <user>` | Add an issue watcher. | `workhub cli jira issue watcher add ABC-1 account-id-or-username` | Operation summary by default; operation result with `--json`. |
| `cli jira issue watcher remove <issue-key> <user>` | Remove an issue watcher. | `workhub cli jira issue watcher remove ABC-1 account-id-or-username` | Operation summary by default; operation result with `--json`. |
| `cli jira issue worklog list <issue-key> [--start-at <n>] [--limit <n>]` | List issue worklogs. | `workhub cli jira issue worklog list ABC-1 --limit 10` | Worklog table by default; worklog list and paging fields with `--json`. |
| `cli jira issue worklog add <issue-key> --time-spent <text> [--started <timestamp>] [--comment <text> \| --comment-file <path> \| --comment-stdin] [--visibility-json <json> \| --visibility-file <path>] [--adjust-estimate <mode>] [--new-estimate <text>] [--reduce-by <text>]` | Add an issue worklog. | `workhub cli jira issue worklog add ABC-1 --time-spent 1h --started 2026-06-12T09:00:00.000+0000 --comment 'Investigation'` | New worklog summary by default; worklog object with `--json`. |

## Links And Parent Relationships

| Command | Use | Example | Returns |
| --- | --- | --- | --- |
| `cli jira issue link-type list [--name <name>]` | List issue link types, optionally filtered by name. | `workhub cli jira issue link-type list --name Blocks` | Link type table by default; type list with `--json`. |
| `cli jira issue parent set <issue-key> <parent-key>` | Set an issue parent. | `workhub cli jira issue parent set ABC-2 ABC-1` | Operation summary by default; operation result with `--json`. |
| `cli jira issue link create --type <name> --inward <issue-key> --outward <issue-key> [--comment <text> \| --comment-file <path> \| --comment-stdin]` | Create an issue link. | `workhub cli jira issue link create --type Blocks --inward ABC-1 --outward ABC-2 --comment 'Blocks release'` | Operation summary by default; link result with `--json`. |
| `cli jira issue link delete <link-id>` | Delete an issue link. | `workhub cli jira issue link delete 10001` | Delete summary by default; operation result with `--json`. |
| `cli jira issue remote-link create <issue-key> --url <url> --title <title> [--global-id <id>] [--relationship <text>] [--summary <text>] [--icon-url <url>] [--icon-title <title>] [--status-json <json> \| --status-file <path>]` | Add a remote link to an issue. | `workhub cli jira issue remote-link create ABC-1 --url https://example.invalid/external/42 --title 'External item 42' --relationship 'relates to' --status-file /tmp/workhub-jira-remote-status.json` | Remote-link summary by default; remote-link object with `--json`. |

## Attachments, Timeline, SLA, And Related Data

| Command | Use | Example | Returns |
| --- | --- | --- | --- |
| `cli jira issue attachment list <issue-key> [--attachment-ids <csv>] [--filename-contains <text>] [--media-type <type>] [--include-content] [--max-bytes <n>]` | List issue attachments and optionally include content. | `workhub cli jira issue attachment list ABC-1 --filename-contains log --media-type text/plain --include-content --max-bytes 65536` | Attachment table by default; attachment object with `--json`. Content fields are bounded by `--max-bytes`. |
| `cli jira issue attachment images <issue-key> [--include-content] [--max-bytes <n>]` | List image attachments and optionally include content. | `workhub cli jira issue attachment images ABC-1 --include-content --max-bytes 1048576` | Image attachment table by default; image attachment object with `--json`. |
| `cli jira issue timeline <issue-key> [--include-status-changes <bool>] [--include-status-summary <bool>]` | Get issue timeline data. | `workhub cli jira issue timeline ABC-1 --include-status-changes true --include-status-summary true` | Timeline summary by default; timeline object with `--json`. |
| `cli jira issue sla <issue-key> [--metrics <csv>] [--include-raw-dates]` | Get Jira Service Management SLA metrics. | `workhub cli jira issue sla ABC-1 --metrics time_to_resolution --include-raw-dates` | SLA metrics by default; metrics object with `--json`. |
| `cli jira issue development <issue-key> [--data-type <type>]` | Get linked branch, commit, pull request, or similar issue data. | `workhub cli jira issue development ABC-1 --data-type branch` | Related-data summary by default; related-data object with `--json`. |
| `cli jira issue development-batch --issues <csv> [--data-type <type>]` | Get linked data for multiple issues. | `workhub cli jira issue development-batch --issues ABC-1,ABC-2 --data-type pullrequest` | Related-data summary by issue by default; batch object with `--json`. |

## Projects, Versions, And Components

| Command | Use | Example | Returns |
| --- | --- | --- | --- |
| `cli jira project list [--include-archived]` | List projects. | `workhub cli jira project list --include-archived` | Project table by default; project list with `--json`. |
| `cli jira project version list <project-key>` | List project versions. | `workhub cli jira project version list ABC` | Version table by default; version list with `--json`. |
| `cli jira project version create <project-key> --name <name> [--description <text>] [--start-date <date>] [--release-date <date>] [--released <bool>] [--archived <bool>]` | Create a project version. | `workhub cli jira project version create ABC --name 2026.07 --description 'July release' --release-date 2026-07-31 --released false` | Created version summary by default; version object with `--json`. |
| `cli jira project version create-batch <project-key> --versions-file <path>` | Create project versions from a JSON file. | `workhub cli jira project version create-batch ABC --versions-file /tmp/workhub-jira-versions.json` | Per-version create results by default; batch result object with `--json`. |
| `cli jira project component list <project-key>` | List project components. | `workhub cli jira project component list ABC` | Component table by default; component list with `--json`. |

## Fields And Users

| Command | Use | Example | Returns |
| --- | --- | --- | --- |
| `cli jira field search [--keyword <text>] [--limit <n>]` | Search Jira fields. | `workhub cli jira field search --keyword sprint --limit 20` | Field table by default; field list with `--json`. |
| `cli jira field options <field-id> [--context-id <id>] [--project <key>] [--issue-type <name>] [--contains <text>] [--return-limit <n>] [--values-only]` | List field options. | `workhub cli jira field options customfield_10000 --project ABC --issue-type Task --contains backend --values-only` | Option table by default; option object with `--json`. With `--values-only`, output focuses on option values. |
| `cli jira user get [--user <id-or-name>]` | Get a user. Without `--user`, get the current identity. | `workhub cli jira user get --user account-id-or-username` | User fields by default; user object with `--json`. |

## Agile And Sprints

| Command | Use | Example | Returns |
| --- | --- | --- | --- |
| `cli jira agile board list [--project <key>] [--type <type>] [--name <text>] [--start-at <n>] [--limit <n>]` | List Agile boards. | `workhub cli jira agile board list --project ABC --type scrum --name Team --limit 20` | Board table by default; board list and paging fields with `--json`. |
| `cli jira agile board issues <board-id> [--jql <jql>] [--fields <csv>] [--start-at <n>] [--limit <n>]` | List issues on a board. | `workhub cli jira agile board issues 12 --jql 'status != Done' --fields summary,status --limit 20` | Issue table by default; issue list with `--json`. |
| `cli jira agile board sprints <board-id> [--state <csv>] [--start-at <n>] [--limit <n>]` | List board sprints. | `workhub cli jira agile board sprints 12 --state active,future --limit 20` | Sprint table by default; sprint list with `--json`. |
| `cli jira agile sprint issues <sprint-id> [--fields <csv>] [--start-at <n>] [--limit <n>]` | List issues in a sprint. | `workhub cli jira agile sprint issues 42 --fields summary,status --limit 20` | Issue table by default; sprint issue list with `--json`. |
| `cli jira agile sprint create --board-id <id> --name <name> [--start-date <timestamp>] [--end-date <timestamp>] [--goal <text>]` | Create a sprint. | `workhub cli jira agile sprint create --board-id 12 --name 'Sprint 42' --start-date 2026-06-12T09:00:00.000Z --end-date 2026-06-26T09:00:00.000Z --goal 'Ship CLI'` | Created sprint summary by default; sprint object with `--json`. |
| `cli jira agile sprint update <sprint-id> [--name <name>] [--state <state>] [--start-date <timestamp>] [--end-date <timestamp>] [--goal <text>]` | Update a sprint. | `workhub cli jira agile sprint update 42 --state active --goal 'Stabilize CLI'` | Updated sprint summary by default; sprint or operation result with `--json`. |
| `cli jira agile sprint add-issues <sprint-id> --issues <csv>` | Add issues to a sprint. | `workhub cli jira agile sprint add-issues 42 --issues ABC-1,ABC-2` | Operation summary by default; operation result with `--json`. |

## Service Desk

| Command | Use | Example | Returns |
| --- | --- | --- | --- |
| `cli jira service-desk project <project-key>` | Get the service desk for a Jira project. | `workhub cli jira service-desk project ABC` | Service desk fields by default; service desk object with `--json`. |
| `cli jira service-desk queue list <service-desk-id> [--include-counts] [--start <n>] [--limit <n>]` | List service desk queues. | `workhub cli jira service-desk queue list 5 --include-counts --limit 20` | Queue table by default; queue list and paging fields with `--json`. |
| `cli jira service-desk queue issues <service-desk-id> <queue-id> [--start <n>] [--limit <n>]` | List issues in a service desk queue. | `workhub cli jira service-desk queue issues 5 12 --limit 20` | Issue table by default; queue issue list with `--json`. |

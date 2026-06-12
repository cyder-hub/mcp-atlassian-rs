# CLI Reference

`workhub cli ...` is the production resource-oriented command surface for Jira, Confluence, and GitLab. It uses the same service credentials, service clients, project/space filters, redaction, proxy, TLS, mTLS, redirect, bounded attachment, and bounded diff behavior as the MCP tools. It ignores MCP tool visibility controls such as `MCP_TOOL_PROFILE`, `MCP_TOOLSETS`, `MCP_ENABLED_TOOLS`, and `MCP_DISABLED_TOOLS`.

The CLI does not expose raw MCP tool calls, schema dumping, or tool-name fallback commands.

## Runtime

Run commands as:

```bash
workhub cli [--env-file <path>] [--json] [--pretty] <provider> <resource> <action> ...
```

Global flags:

| Flag | Behavior |
| --- | --- |
| `--env-file <path>` | Load a dotenv file before reading runtime configuration. Takes precedence over `ENV_FILE`. |
| `--json` | Print successful results as compact JSON to stdout. Errors are JSON on stderr. |
| `--pretty` | Pretty-print JSON. Requires `--json`. |

Environment loading:

- `stdio` does not load `.env`, `ENV_FILE`, or `--env-file`.
- `streamhttp` and `cli` load environment in this order: explicit `--env-file`, `ENV_FILE`, current directory `.env`.
- Missing default `.env` is ignored. An explicit env file that cannot be read is a configuration error.

Credentials and service endpoint URLs are configured only through environment variables or env files. The CLI intentionally has no provider endpoint, token, password, proxy, custom-header, TLS, or mTLS override flags.

Output and exits:

| Case | stdout | stderr | Exit |
| --- | --- | --- | --- |
| Success, default | Compact text | Empty | `0` |
| Success, `--json` | Result JSON | Empty | `0` |
| Usage error | Empty | Usage/error text | `2` |
| Invalid input | Empty | Operation error | `2` |
| Missing service, unavailable service, project/space filter rejection | Empty | Operation error | `3` |
| Upstream HTTP/transport/decode/shape error | Empty | Operation error | `4` |
| Business structured error surfaced as CLI failure | Empty | Operation error | `5` |

Default text output renders scalar object fields as `key: value`. Object arrays inside results, such as `issues`, `values`, `results`, or `items`, are expanded into tabular sections with inferred key columns. Empty cells are shown as `-` so missing values remain visible in plain terminal output. Use `--json` when a caller needs the exact structured response.

Long text fields use mutually exclusive inline, file, or stdin inputs. File paths are read by the process running `workhub cli`; for remote shells or containers, this means the path must exist on that host/container.

## Jira

`jira issue search` defaults to key issue list fields when `--fields` is omitted: `key`, `summary`, `status`, `assignee`, `reporter`, `issuetype`, `priority`, and `project`. Pass `--fields <csv>` to request a different field set.

| Capability | Command |
| --- | --- |
| Get issue | `workhub cli jira issue get <issue-key> [--fields <csv>] [--expand <csv>] [--comment-limit <n>] [--properties <csv>] [--update-history <bool>]` |
| Search issues | `workhub cli jira issue search --jql <jql> [--fields <csv>] [--limit <n>] [--start-at <n>] [--projects <csv>] [--expand <csv>] [--page-token <token>]` |
| List project issues | `workhub cli jira project issues <project-key> [--limit <n>] [--start-at <n>]` |
| Create issue | `workhub cli jira issue create --project <key> --issue-type <type> --summary <text> [--description <text>|--description-file <path>|--description-stdin] [--assignee <user>] [--priority <name>] [--labels <csv>] [--components <csv>] [--fix-versions <csv>] [--additional-fields-json <json>|--additional-fields-file <path>]` |
| Create issues | `workhub cli jira issue create-batch --issues-file <path> [--validate-only]` |
| Get changelogs | `workhub cli jira issue changelog batch --issue-ids <csv> [--limit <n>] [--field-ids <csv>]` |
| Update issue | `workhub cli jira issue update <issue-key> (--fields-json <json>|--fields-file <path>) [--notify-users <bool>]` |
| Delete issue | `workhub cli jira issue delete <issue-key> [--delete-subtasks]` |
| Search fields | `workhub cli jira field search [--keyword <text>] [--limit <n>]` |
| List field options | `workhub cli jira field options <field-id> [--context-id <id>] [--project <key>] [--issue-type <type>] [--contains <text>] [--return-limit <n>] [--values-only]` |
| Add comment | `workhub cli jira issue comment add <issue-key> (--body <text>|--body-file <path>|--body-stdin) [--visibility-json <json>|--visibility-file <path>]` |
| Update comment | `workhub cli jira issue comment update <issue-key> <comment-id> (--body <text>|--body-file <path>|--body-stdin) [--visibility-json <json>|--visibility-file <path>]` |
| List transitions | `workhub cli jira issue transition list <issue-key>` |
| Apply transition | `workhub cli jira issue transition apply <issue-key> <transition-id> [--fields-json <json>|--fields-file <path>] [--comment <text>|--comment-file <path>|--comment-stdin]` |
| List projects | `workhub cli jira project list [--include-archived]` |
| List versions | `workhub cli jira project version list <project-key>` |
| Create version | `workhub cli jira project version create <project-key> --name <name> [--description <text>] [--start-date <date>] [--release-date <date>] [--released <bool>] [--archived <bool>]` |
| Create versions | `workhub cli jira project version create-batch <project-key> --versions-file <path>` |
| List components | `workhub cli jira project component list <project-key>` |
| Get user | `workhub cli jira user get [--user <id-or-name>]` |
| List watchers | `workhub cli jira issue watcher list <issue-key>` |
| Add watcher | `workhub cli jira issue watcher add <issue-key> <user>` |
| Remove watcher | `workhub cli jira issue watcher remove <issue-key> <user>` |
| List worklogs | `workhub cli jira issue worklog list <issue-key> [--start-at <n>] [--limit <n>]` |
| Add worklog | `workhub cli jira issue worklog add <issue-key> --time-spent <text> [--started <timestamp>] [--comment <text>|--comment-file <path>|--comment-stdin] [--visibility-json <json>|--visibility-file <path>] [--adjust-estimate <mode>] [--new-estimate <text>] [--reduce-by <text>]` |
| List link types | `workhub cli jira issue link-type list [--name <name>]` |
| Set parent | `workhub cli jira issue parent set <issue-key> <parent-key>` |
| Create issue link | `workhub cli jira issue link create --type <name> --inward <issue-key> --outward <issue-key> [--comment <text>|--comment-file <path>|--comment-stdin]` |
| Delete issue link | `workhub cli jira issue link delete <link-id>` |
| Create remote link | `workhub cli jira issue remote-link create <issue-key> --url <url> --title <title> [--global-id <id>] [--relationship <text>] [--summary <text>] [--icon-url <url>] [--icon-title <title>] [--status-json <json>|--status-file <path>]` |
| Get attachments | `workhub cli jira issue attachment list <issue-key> [--attachment-ids <csv>] [--filename-contains <text>] [--media-type <type>] [--include-content] [--max-bytes <n>]` |
| Get image attachments | `workhub cli jira issue attachment images <issue-key> [--include-content] [--max-bytes <n>]` |
| List agile boards | `workhub cli jira agile board list [--project <key>] [--type <type>] [--name <text>] [--start-at <n>] [--limit <n>]` |
| List board issues | `workhub cli jira agile board issues <board-id> [--jql <jql>] [--fields <csv>] [--start-at <n>] [--limit <n>]` |
| List board sprints | `workhub cli jira agile board sprints <board-id> [--state <csv>] [--start-at <n>] [--limit <n>]` |
| List sprint issues | `workhub cli jira agile sprint issues <sprint-id> [--fields <csv>] [--start-at <n>] [--limit <n>]` |
| Create sprint | `workhub cli jira agile sprint create --board-id <id> --name <name> [--start-date <timestamp>] [--end-date <timestamp>] [--goal <text>]` |
| Update sprint | `workhub cli jira agile sprint update <sprint-id> [--name <name>] [--state <state>] [--start-date <timestamp>] [--end-date <timestamp>] [--goal <text>]` |
| Add issues to sprint | `workhub cli jira agile sprint add-issues <sprint-id> --issues <csv>` |
| Get project service desk | `workhub cli jira service-desk project <project-key>` |
| List service desk queues | `workhub cli jira service-desk queue list <service-desk-id> [--include-counts] [--start <n>] [--limit <n>]` |
| List service desk queue issues | `workhub cli jira service-desk queue issues <service-desk-id> <queue-id> [--start <n>] [--limit <n>]` |
| Get issue timeline | `workhub cli jira issue timeline <issue-key> [--include-status-changes <bool>] [--include-status-summary <bool>]` |
| Get SLA metrics | `workhub cli jira issue sla <issue-key> [--metrics <csv>] [--include-raw-dates]` |
| Get development | `workhub cli jira issue development <issue-key> [--data-type <type>]` |
| Get batch development | `workhub cli jira issue development-batch --issues <csv> [--data-type <type>]` |

## Confluence

| Capability | Command |
| --- | --- |
| Search content | `workhub cli confluence content search --query <text> [--limit <n>] [--spaces <csv>]` |
| Get page | `workhub cli confluence page get (--id <page-id>|--space <key> --title <title>) [--include-metadata <bool>] [--markdown <bool>]` |
| List page children | `workhub cli confluence page children <parent-id> [--expand <csv>] [--limit <n>] [--include-content] [--markdown <bool>] [--start <n>] [--include-folders <bool>]` |
| Get space page tree | `workhub cli confluence page tree <space-key> [--limit <n>]` |
| Create page | `workhub cli confluence page create --space <key> --title <title> (--content <text>|--content-file <path>|--content-stdin) [--parent-id <id>] [--format <markdown|wiki|storage>] [--include-content] [--emoji <emoji>]` |
| Update page | `workhub cli confluence page update <page-id> --title <title> (--content <text>|--content-file <path>|--content-stdin) [--minor-edit <bool>] [--version-comment <text>] [--parent-id <id>] [--format <markdown|wiki|storage>] [--include-content] [--emoji <emoji>]` |
| Delete page | `workhub cli confluence page delete <page-id>` |
| Move page | `workhub cli confluence page move <page-id> [--target-parent-id <id>] [--target-space <key>] [--position <append|before|after>]` |
| List comments | `workhub cli confluence page comment list <page-id>` |
| Add comment | `workhub cli confluence page comment add <page-id> (--body <text>|--body-file <path>|--body-stdin)` |
| Reply to comment | `workhub cli confluence page comment reply <page-id> <comment-id> (--body <text>|--body-file <path>|--body-stdin)` |
| List labels | `workhub cli confluence content label list <content-id>` |
| Add label | `workhub cli confluence content label add <content-id> <label>` |
| Search users | `workhub cli confluence user search --query <text> [--limit <n>] [--group <name>]` |
| Get page version | `workhub cli confluence page version get <page-id> <version> [--markdown <bool>]` |
| Get page diff | `workhub cli confluence page version diff <page-id> --from <version> --to <version> [--context-lines <n>]` |
| Get page analytics | `workhub cli confluence page analytics views <page-id> [--from <date>] [--to <date>] [--include-title <bool>]` |
| Upload attachment | `workhub cli confluence attachment upload <content-id> <file-path> [--comment <text>] [--minor-edit <bool>]` |
| Upload attachments | `workhub cli confluence attachment upload-batch <content-id> --files <csv> [--comment <text>] [--minor-edit <bool>]` |
| List attachments | `workhub cli confluence attachment list <content-id> [--filename-contains <text>] [--media-type <type>] [--start <n>] [--limit <n>]` |
| Download attachment | `workhub cli confluence attachment download <attachment-id> [--max-bytes <n>]` |
| Download content attachments | `workhub cli confluence attachment download-content <content-id> [--filename-contains <text>] [--media-type <type>] [--max-bytes <n>] [--limit <n>]` |
| Delete attachment | `workhub cli confluence attachment delete <attachment-id>` |
| Get image attachments | `workhub cli confluence attachment images <content-id> [--max-bytes <n>]` |

## GitLab

| Capability | Command |
| --- | --- |
| Get current user | `workhub cli gitlab user current` |
| Get project | `workhub cli gitlab project get <project>` |
| List merge requests | `workhub cli gitlab mr list <project> [--state <state>] [--author <username>] [--reviewer <username>] [--source-branch <branch>] [--target-branch <branch>] [--labels <csv>] [--page <n>] [--per-page <n>]` |
| Get merge request | `workhub cli gitlab mr get <project> <iid> [--include-diverged-commits-count <bool>] [--include-rebase-in-progress <bool>]` |
| List commits | `workhub cli gitlab mr commits <project> <iid> [--page <n>] [--per-page <n>]` |
| List diffs | `workhub cli gitlab mr diffs <project> <iid> [--max-diff-bytes <n>] [--page <n>] [--per-page <n>]` |
| List pipelines | `workhub cli gitlab mr pipelines <project> <iid> [--page <n>] [--per-page <n>]` |
| Create MR | `workhub cli gitlab mr create <project> --source <branch> --target <branch> --title <title> [--description <text>|--description-file <path>|--description-stdin] [--remove-source-branch <bool>] [--squash <bool>] [--assignee-ids <csv>] [--reviewer-ids <csv>] [--labels <csv>]` |
| Update MR | `workhub cli gitlab mr update <project> <iid> [--title <title>] [--description <text>|--description-file <path>|--description-stdin] [--state-event <event>] [--labels <csv>] [--add-labels <csv>] [--remove-labels <csv>] [--reviewer-ids <csv>] [--assignee-ids <csv>] [--target-branch <branch>]` |
| Add note | `workhub cli gitlab mr note add <project> <iid> (--body <text>|--body-file <path>|--body-stdin)` |
| Reply discussion | `workhub cli gitlab mr discussion reply <project> <iid> <discussion-id> (--body <text>|--body-file <path>|--body-stdin)` |
| Resolve discussion | `workhub cli gitlab mr discussion resolve <project> <iid> <discussion-id> --resolved <bool>` |
| Get approval state | `workhub cli gitlab mr approval get <project> <iid>` |
| Set approval | `workhub cli gitlab mr approval set <project> <iid> --action <approve|unapprove>` |
| Merge MR | `workhub cli gitlab mr merge <project> <iid> --sha <sha> [--auto-merge <bool>] [--squash <bool>] [--remove-source-branch <bool>] [--merge-commit-message <text>] [--squash-commit-message <text>]` |

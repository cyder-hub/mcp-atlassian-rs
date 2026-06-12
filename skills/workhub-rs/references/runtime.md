# CLI Runtime

This module covers configuration, global flags, input forms, output, and error handling for `workhub cli ...`.

If the binary is not on `PATH`, replace `workhub` with its absolute path.

## Command Shape

```bash
workhub cli [--env-file <path>] [--json] [--pretty] <provider> <resource> <action> ...
```

Provider entry points:

```bash
workhub cli jira ...
workhub cli confluence ...
workhub cli gitlab ...
```

Global flags:

| Flag | Use | Return behavior |
| --- | --- | --- |
| `--env-file <path>` | Load credentials and runtime settings from a dotenv-style file for this command. | On success, the command continues normally. If the file cannot be loaded, stderr reports a configuration error and the exit code is `3`. |
| `--json` | Return command results as JSON. | Success JSON is written to stdout. Failure JSON is written to stderr. |
| `--pretty` | Format JSON with indentation. Requires `--json`. | Same as `--json`, but formatted for reading. |

## Environment Loading

The CLI loads environment variables in this priority order:

1. Explicit `--env-file <path>`.
2. `ENV_FILE`.
3. `.env` in the current directory.

Missing default `.env` is ignored. A missing or unreadable explicit env file is a configuration error.

Minimal env file example:

```bash
JIRA_URL=https://your-company.atlassian.net
JIRA_USERNAME=user@example.com
JIRA_API_TOKEN=<api-token>

CONFLUENCE_URL=https://your-company.atlassian.net/wiki
CONFLUENCE_USERNAME=user@example.com
CONFLUENCE_API_TOKEN=<api-token>

GITLAB_URL=https://gitlab.example.com
GITLAB_TOKEN=<token>

MCP_TOOL_PROFILE=basic
```

## Service Credentials

Configure one provider or any combination of providers. Commands for unconfigured services fail before sending upstream requests.

| Service | Minimal variables | Notes |
| --- | --- | --- |
| Jira Cloud | `JIRA_URL`, `JIRA_USERNAME`, `JIRA_API_TOKEN` | Uses Atlassian API-token Basic auth. |
| Jira Server/Data Center | `JIRA_URL`, `JIRA_PERSONAL_TOKEN` | If the instance still allows Basic auth, `JIRA_USERNAME` and `JIRA_PASSWORD` can be used instead. |
| Confluence Cloud | `CONFLUENCE_URL`, `CONFLUENCE_USERNAME`, `CONFLUENCE_API_TOKEN` | Cloud URLs commonly end with `/wiki`. |
| Confluence Server/Data Center | `CONFLUENCE_URL`, `CONFLUENCE_PERSONAL_TOKEN` | If the instance still allows Basic auth, `CONFLUENCE_USERNAME` and `CONFLUENCE_PASSWORD` can be used instead. |
| GitLab | `GITLAB_URL`, `GITLAB_TOKEN` | `GITLAB_URL` is the instance root. The token is sent as `PRIVATE-TOKEN`. |

Jira and Confluence can also use shared `ATLASSIAN_USERNAME`, `ATLASSIAN_API_TOKEN`, `ATLASSIAN_PASSWORD`, or `ATLASSIAN_PERSONAL_TOKEN` fallbacks when service-specific credentials are unset.

## MCP Access Controls

These controls affect MCP tool discovery and MCP tool calls only. `workhub cli ...` ignores them and exposes its full command surface for configured services.

| Variable | Default | Use |
| --- | --- | --- |
| `MCP_TOOL_PROFILE` | `basic` | Selects a capability baseline: `basic`, `developer`, `manager`, `full`, or `custom`. |
| `MCP_TOOLSETS` | Profile defaults | Adds comma-separated toolsets. `all` enables every toolset. |
| `MCP_ENABLED_TOOLS` | Unset | Adds exact MCP tool names. |
| `MCP_DISABLED_TOOLS` | Unset | Removes exact MCP tool names. This takes precedence over profile, toolset, and enabled-tool inclusion. |
| `JIRA_PROJECTS_FILTER` | Unset | Restricts Jira project scope. |
| `CONFLUENCE_SPACES_FILTER` | Unset | Restricts Confluence space scope. |
| `GITLAB_PROJECTS_FILTER` | Unset | Restricts GitLab project IDs or full paths. |

If a command is blocked by missing service configuration, service availability, or a project/space filter, the CLI exits with code `3` and does not send the upstream request.

## Output

Default success output is compact text:

```text
is_last: true

issues:
key     id      summary     status   project assignee
ABC-1   10001   First       Done     ABC     Ada
ABC-2   10002   Second      To Do    ABC     -
```

Rules:

- Object fields render as `key: value`.
- Array fields such as `issues`, `values`, `results`, or `items` render as table sections.
- Empty strings, nulls, and missing table cells render as `-`.
- Nested objects prefer readable identifiers such as `name`, `title`, `key`, `username`, or `id`.

Use JSON for automation:

```bash
workhub cli --json jira issue search --jql 'project = ABC' --limit 10
workhub cli --json --pretty confluence page get --id 123456 --markdown true
```

## Errors

Failures leave stdout empty and write stderr. Exit codes:

| Code | Meaning |
| --- | --- |
| `0` | Success. |
| `2` | Usage error, missing input, invalid argument, unreadable file input, or invalid JSON input. |
| `3` | Missing configuration, unavailable service, or project/space filter rejection. |
| `4` | Upstream HTTP, transport, decode, or unexpected response-shape error. |
| `5` | Business structured error or output rendering failure. |

With `--json`, stderr has this shape:

```json
{
  "success": false,
  "error": {
    "category": "config",
    "message": "Jira is not configured"
  }
}
```

## Input Forms

Text body triads are mutually exclusive:

```bash
workhub cli jira issue comment add ABC-1 --body 'Short note'
workhub cli jira issue comment add ABC-1 --body-file /tmp/comment.md
printf 'Short note\n' | workhub cli jira issue comment add ABC-1 --body-stdin
```

JSON inline/file pairs are mutually exclusive:

```bash
workhub cli jira issue update ABC-1 --fields-json '{"summary":"New summary"}'
workhub cli jira issue update ABC-1 --fields-file /tmp/jira-fields.json
```

CSV flags use comma-separated values:

```bash
workhub cli gitlab mr list group/project --labels backend,urgent --per-page 20
workhub cli jira agile sprint add-issues 42 --issues ABC-1,ABC-2
```

Boolean flags have two forms:

- Presence flags: `--include-content`, `--values-only`, `--delete-subtasks`, `--validate-only`, `--include-raw-dates`.
- Explicit booleans: `--minor-edit true`, `--resolved false`, `--released true`.

## Example Input Files

Jira field update:

```json
{
  "summary": "Updated summary",
  "description": "Plain text description",
  "assignee": "account-id-or-username"
}
```

Jira visibility:

```json
{
  "type": "role",
  "value": "Developers"
}
```

Confluence Markdown page:

```markdown
# Roadmap

Status: draft

- Milestone A
- Milestone B
```

GitLab MR note:

```markdown
Reviewed the change. Please document the empty assignee case.
```

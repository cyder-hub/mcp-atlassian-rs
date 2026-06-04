# mcp-atlassian-rs

Rust migration workspace for MCP Atlassian.

This repository is migrating the Python `mcp-atlassian` Jira and Confluence MCP server to a Rust-native implementation. The Rust binary currently has the shared MCP runtime/control plane, the Stage 2 Jira core tool loop, and all Stage 3 Jira extension tools implemented with local mock REST coverage. Real Jira validation for product-dependent extensions, Confluence, and production auth/security remain later stages.

## Current Status

Implemented in the Rust root project:

- Package, binary, server name, Docker image, compose service, and CI image identity use `mcp-atlassian-rs`.
- MCP server runs over `stdio` and streamable HTTP at `/mcp`.
- Logging is configured to stderr so stdio MCP stdout remains protocol-only.
- Runtime control-plane config parses `READ_ONLY_MODE`, `ENABLED_TOOLS`, `TOOLSETS`, `CONFLUENCE_URL`, `ATLASSIAN_OAUTH_CLOUD_ID`, `MCP_HTTP_HOST`, `MCP_HTTP_PORT`, and `MCP_HTTP_PATH`.
- Jira config parses `JIRA_URL`, `JIRA_USERNAME`, `JIRA_API_TOKEN`, `JIRA_PERSONAL_TOKEN`, `JIRA_SSL_VERIFY`, `JIRA_PROJECTS_FILTER`, and `JIRA_TIMEOUT`.
- Jira Cloud uses username/API token auth for `*.atlassian.net`; Jira Server/Data Center uses PAT auth.
- Shared Atlassian HTTP/auth/error helpers and Jira models/client/tool handlers are implemented for the Stage 2 core tools and the completed Stage 3 Jira extensions.
- Tool registry metadata, service availability filtering, toolset filtering, enabled-tools filtering, and read-only write guards are in place for migrated tools.
- Stage 3 Jira extension tools are implemented for local mock validation: create/update/delete issue, batch create, changelog bulk fetch, projects, versions, users, watchers, worklog, links, attachment download, issue image retrieval, agile boards/sprints, service desk queues, Forms/ProForma, metrics/SLA, and development information.
- Streamable HTTP exposes `GET /healthz`.
- Local stdio, streamable HTTP, and read-only smoke commands validate MCP initialization, Jira tool discovery, mock Jira read calls, `/healthz`, and write-tool blocking.
- The temporary MCP tool `migration_status` reports the migration state.

Deferred:

- Confluence config, auth, client, models, and MCP tools.
- Real Jira validation for Stage 3 product-dependent Jira extensions. Local mock coverage is complete, but Jira Software, Jira Service Management, Forms/ProForma, SLA, and dev-status behavior remain Stage 4 validation gates.
- OAuth, BYOT, per-request HTTP header auth, SSRF protections, allowed domains, proxy/custom headers, mTLS, and full production security hardening.
- Real Jira and Confluence smoke tests. Real Jira validation is a Stage 4 gate.
- Release, Docker, compose, Helm, and parity audit gates beyond the local mock checks.

## Requirements

- Rust 1.94 or newer
- just
- Python 3 when running local smoke scripts
- curl for manual HTTP checks
- Docker when validating container or compose behavior in later gates
- An MCP client or MCP inspector for manual transport checks

## Quick Start

Run over stdio:

```bash
just dev
```

Run over streamable HTTP:

```bash
just dev-http
```

The streamable HTTP endpoint is:

```text
http://127.0.0.1:8000/mcp
```

Direct binary usage:

```bash
cargo run -- stdio
cargo run -- streamhttp --host 127.0.0.1 --port 8000 --path /mcp
```

When no command is provided, the binary defaults to `stdio`.

## Jira Configuration

Jira tools are discoverable only when Jira service configuration and authentication are complete.

Jira Cloud:

```bash
export JIRA_URL="https://your-company.atlassian.net"
export JIRA_USERNAME="user@example.com"
export JIRA_API_TOKEN="<jira-api-token>"
cargo run -- stdio
```

Jira Server/Data Center:

```bash
export JIRA_URL="https://jira.example.com"
export JIRA_PERSONAL_TOKEN="<jira-personal-access-token>"
cargo run -- stdio
```

Optional Jira variables:

| Variable | Default | Behavior |
| --- | --- | --- |
| `JIRA_SSL_VERIFY` | `true` | Set `false`, `0`, `no`, or `off` to disable TLS certificate verification for Jira requests. |
| `JIRA_PROJECTS_FILTER` | unset | Comma-separated project keys. Filters `jira_get_issue` by issue key prefix and injects a project filter into JQL search. |
| `JIRA_TIMEOUT` | `75` | Jira HTTP request timeout in seconds. Must be a positive integer. |
| `ATLASSIAN_OAUTH_CLOUD_ID` | unset | Optional Cloud ID used by Jira Forms/ProForma helpers. Missing values return a structured product-dependency response. |

Stage 3 does not implement OAuth, BYOT, or per-request Authorization overrides.

## Runtime Control Plane

| Variable | Default | Behavior |
| --- | --- | --- |
| `READ_ONLY_MODE` | `false` | Truthy values are `true`, `1`, `yes`, `y`, and `on`. Write tools are hidden from discovery and blocked on direct call when enabled. |
| `ENABLED_TOOLS` | unset | Comma-separated tool names. Empty or unset means no name filtering. |
| `TOOLSETS` | all toolsets | Supports `all`, `default`, or comma-separated toolset names. Unknown-only values fail closed. `migration_status` is not part of Jira or Confluence toolsets. |
| `CONFLUENCE_URL` | unset | Reserved for Confluence service availability filtering in a later stage. |
| `MCP_HTTP_HOST` | `127.0.0.1` | Streamable HTTP host when not overridden by CLI. |
| `MCP_HTTP_PORT` | `8000` | Streamable HTTP port when not overridden by CLI. |
| `MCP_HTTP_PATH` | `/mcp` | Streamable HTTP MCP path when not overridden by CLI. A missing leading slash is normalized. |

The health endpoint is always:

```text
GET http://127.0.0.1:8000/healthz
```

## MCP Tools

The Rust server exposes these Stage 2 Jira core tools when Jira is configured:

| Tool | Access | Toolset |
| --- | --- | --- |
| `jira_get_issue` | read | `jira_issues` |
| `jira_search` | read | `jira_issues` |
| `jira_get_project_issues` | read | `jira_issues` |
| `jira_search_fields` | read | `jira_fields` |
| `jira_get_field_options` | read | `jira_fields` |
| `jira_add_comment` | write | `jira_comments` |
| `jira_edit_comment` | write | `jira_comments` |
| `jira_get_transitions` | read | `jira_transitions` |
| `jira_transition_issue` | write | `jira_transitions` |

The Rust server also exposes these Stage 3 Jira extension tools when Jira is configured. These are locally validated with mock Jira; real Jira acceptance remains deferred to Stage 4.

| Tool | Access | Toolset |
| --- | --- | --- |
| `jira_create_issue` | write | `jira_issues` |
| `jira_batch_create_issues` | write | `jira_issues` |
| `jira_batch_get_changelogs` | read | `jira_issues` |
| `jira_update_issue` | write | `jira_issues` |
| `jira_delete_issue` | write | `jira_issues` |
| `jira_get_all_projects` | read | `jira_projects` |
| `jira_get_project_versions` | read | `jira_projects` |
| `jira_get_project_components` | read | `jira_projects` |
| `jira_create_version` | write | `jira_projects` |
| `jira_batch_create_versions` | write | `jira_projects` |
| `jira_get_user_profile` | read | `jira_users` |
| `jira_get_issue_watchers` | read | `jira_watchers` |
| `jira_add_watcher` | write | `jira_watchers` |
| `jira_remove_watcher` | write | `jira_watchers` |
| `jira_get_worklog` | read | `jira_worklog` |
| `jira_add_worklog` | write | `jira_worklog` |
| `jira_get_link_types` | read | `jira_links` |
| `jira_link_to_epic` | write | `jira_links` |
| `jira_create_issue_link` | write | `jira_links` |
| `jira_create_remote_issue_link` | write | `jira_links` |
| `jira_remove_issue_link` | write | `jira_links` |
| `jira_download_attachments` | read | `jira_attachments` |
| `jira_get_issue_images` | read | `jira_attachments` |
| `jira_get_agile_boards` | read | `jira_agile` |
| `jira_get_board_issues` | read | `jira_agile` |
| `jira_get_sprints_from_board` | read | `jira_agile` |
| `jira_get_sprint_issues` | read | `jira_agile` |
| `jira_create_sprint` | write | `jira_agile` |
| `jira_update_sprint` | write | `jira_agile` |
| `jira_add_issues_to_sprint` | write | `jira_agile` |
| `jira_get_service_desk_for_project` | read | `jira_service_desk` |
| `jira_get_service_desk_queues` | read | `jira_service_desk` |
| `jira_get_queue_issues` | read | `jira_service_desk` |
| `jira_get_issue_proforma_forms` | read | `jira_forms` |
| `jira_get_proforma_form_details` | read | `jira_forms` |
| `jira_update_proforma_form_answers` | write | `jira_forms` |
| `jira_get_issue_dates` | read | `jira_metrics` |
| `jira_get_issue_sla` | read | `jira_metrics` |
| `jira_get_issue_development_info` | read | `jira_development` |
| `jira_get_issues_development_info` | read | `jira_development` |

The Rust server also exposes one migration utility tool:

- `migration_status`: reports the Rust migration state.

`migration_status` is not a Jira or Confluence business tool and is not counted as Atlassian tool parity.

## Commands

Run `just --list` to see the local command surface.

```bash
just dev           # run stdio transport
just dev-http      # run streamable HTTP transport on 127.0.0.1:8000
just smoke-stdio   # validate stdio MCP initialize, tools/list, and mock Jira jira_get_issue
just smoke-http    # validate /healthz, HTTP MCP tools/list, and mock Jira jira_get_issue
just smoke-jira    # validate read-only Jira write-tool hiding and blocking
just smoke         # run all local smoke checks
just build         # build the release binary
just test          # run tests
just check         # fmt, check, and tests
just docker-build  # local Docker image build
```

## Docker And Compose

Build the local image:

```bash
just docker-build
```

Equivalent direct Docker command:

```bash
docker build -t mcp-atlassian-rs:local -f Dockerfile .
```

Run the image:

```bash
docker run --rm -p 8000:8000 mcp-atlassian-rs:local
```

The image runs:

```bash
mcp-atlassian-rs streamhttp --host 0.0.0.0 --port 8000
```

Run with compose:

```bash
docker compose up --build
```

Set `MCP_PORT` to change the host port used by compose.

## Verification

Local checks:

```bash
cargo fmt --check
cargo check
cargo test
just check
just smoke-stdio
just smoke-http
just smoke-jira
just smoke
```

The smoke commands start a local mock Jira server and do not require real Jira credentials. Real Jira validation is intentionally deferred to the Stage 4 acceptance gate.

## License

Licensed under the MIT License. See [LICENSE](LICENSE).

# mcp-atlassian-rs

Rust-native MCP server for Atlassian Jira, Confluence, and GitLab merge requests.

The Rust binary has the shared MCP runtime/control plane, 49 Jira business tools, 24 Confluence business tools, and 15 GitLab merge-request tools implemented with local mock REST and MCP coverage. Integrated acceptance has validated representative real Jira, real Confluence, dual-service MCP, release, Docker, and compose paths. GitLab support is locally validated only; real GitLab acceptance is not part of the current release claim. The current release includes production safety support for redaction, request-scoped streamable HTTP auth, SSRF/allowed-domain checks, redirect protection, BYOT access tokens, Bearer disambiguation, Cloud API gateway base rewrite, HTTP/HTTPS proxy, NO_PROXY, custom outbound headers, and mTLS client cert/key.

The final support matrix is in [`docs/support-matrix.md`](docs/support-matrix.md). It covers all 88 Jira, Confluence, and GitLab business tools, local/MCP coverage, real acceptance status, blocker/local-only notes, and the runtime/auth/transport/network support boundaries.

## Current Status

Implemented in the Rust root project:

- Package, binary, server name, Docker image, compose service, and CI image identity use `mcp-atlassian-rs`.
- MCP server runs over `stdio` and streamable HTTP at `/mcp`.
- Logging is configured to stderr so stdio MCP stdout remains protocol-only.
- Runtime control-plane config parses `TOOL_PROFILE`, `TOOLSETS`, `ENABLED_TOOLS`, `DISABLED_TOOLS`, `ATLASSIAN_OAUTH_CLOUD_ID`, `ATLASSIAN_OAUTH_ENABLE`, `MCP_ALLOWED_URL_DOMAINS`, and `IGNORE_HEADER_AUTH`. Streamable HTTP additionally parses `MCP_HTTP_HOST`, `MCP_HTTP_PORT`, and `MCP_HTTP_PATH`.
- Jira config parses `JIRA_URL`, `JIRA_USERNAME`, `JIRA_API_TOKEN`, `JIRA_PASSWORD`, `JIRA_PERSONAL_TOKEN`, `ATLASSIAN_USERNAME`, `ATLASSIAN_API_TOKEN`, `ATLASSIAN_PASSWORD`, `ATLASSIAN_PERSONAL_TOKEN`, `ATLASSIAN_OAUTH_ACCESS_TOKEN`, `JIRA_OAUTH_ACCESS_TOKEN`, `JIRA_SSL_VERIFY`, `ATLASSIAN_SSL_VERIFY`, `JIRA_PROJECTS_FILTER`, `JIRA_TIMEOUT`, `ATLASSIAN_TIMEOUT`, `JIRA_HTTP_PROXY`, `JIRA_HTTPS_PROXY`, `JIRA_NO_PROXY`, `ATLASSIAN_HTTP_PROXY`, `ATLASSIAN_HTTPS_PROXY`, `ATLASSIAN_NO_PROXY`, `JIRA_CUSTOM_HEADERS`, `ATLASSIAN_CUSTOM_HEADERS`, `JIRA_CLIENT_CERT`, `JIRA_CLIENT_KEY`, `ATLASSIAN_CLIENT_CERT`, and `ATLASSIAN_CLIENT_KEY`.
- Confluence config parses `CONFLUENCE_URL`, `CONFLUENCE_USERNAME`, `CONFLUENCE_API_TOKEN`, `CONFLUENCE_PASSWORD`, `CONFLUENCE_PERSONAL_TOKEN`, `ATLASSIAN_USERNAME`, `ATLASSIAN_API_TOKEN`, `ATLASSIAN_PASSWORD`, `ATLASSIAN_PERSONAL_TOKEN`, `ATLASSIAN_OAUTH_ACCESS_TOKEN`, `CONFLUENCE_OAUTH_ACCESS_TOKEN`, `CONFLUENCE_SSL_VERIFY`, `ATLASSIAN_SSL_VERIFY`, `CONFLUENCE_SPACES_FILTER`, `CONFLUENCE_TIMEOUT`, `ATLASSIAN_TIMEOUT`, `CONFLUENCE_HTTP_PROXY`, `CONFLUENCE_HTTPS_PROXY`, `CONFLUENCE_NO_PROXY`, `ATLASSIAN_HTTP_PROXY`, `ATLASSIAN_HTTPS_PROXY`, `ATLASSIAN_NO_PROXY`, `CONFLUENCE_CUSTOM_HEADERS`, `ATLASSIAN_CUSTOM_HEADERS`, `CONFLUENCE_CLIENT_CERT`, `CONFLUENCE_CLIENT_KEY`, `ATLASSIAN_CLIENT_CERT`, and `ATLASSIAN_CLIENT_KEY`.
- GitLab config parses `GITLAB_URL`, `GITLAB_TOKEN`, `GITLAB_PERSONAL_TOKEN`, `GITLAB_OAUTH_ACCESS_TOKEN`, `GITLAB_PROJECTS_FILTER`, `GITLAB_SSL_VERIFY`, `GITLAB_TIMEOUT`, `GITLAB_HTTP_PROXY`, `GITLAB_HTTPS_PROXY`, `GITLAB_NO_PROXY`, `GITLAB_CUSTOM_HEADERS`, `GITLAB_CLIENT_CERT`, and `GITLAB_CLIENT_KEY`.
- Jira Cloud uses username/API token auth for `*.atlassian.net`; Jira Server/Data Center uses PAT, BYOT access-token, or username/password auth.
- Confluence Cloud uses username/API token auth for `*.atlassian.net`; Confluence Server/Data Center uses PAT, BYOT access-token, or username/password auth.
- GitLab uses `PRIVATE-TOKEN` for `GITLAB_TOKEN` or `GITLAB_PERSONAL_TOKEN`, and `Authorization: Bearer` for `GITLAB_OAUTH_ACCESS_TOKEN`. Username/password API auth is not supported.
- Cloud BYOT access tokens use `https://api.atlassian.com/ex/jira/{cloud_id}` for Jira and `https://api.atlassian.com/ex/confluence/{cloud_id}/wiki` for Confluence. Server/Data Center BYOT keeps the configured service base URL.
- Service network config supports typed HTTP/HTTPS proxy, NO_PROXY, custom outbound headers, and mTLS client cert/key.
- Shared upstream HTTP/auth/error helpers and Jira models/client/tool handlers are implemented for Jira core and extended tools.
- Tool registry metadata, service availability filtering, profile filtering, toolset filtering, enabled-tools inclusion, and disabled-tools exclusion are in place for Jira, Confluence, and GitLab tools.
- Jira extended tools are implemented for local mock validation: create/update/delete issue, batch create, changelog bulk fetch, projects, versions, users, watchers, worklog, links, attachment download, issue image retrieval, agile boards/sprints, service desk queues, Forms/ProForma, metrics/SLA, and development information.
- Confluence implementation has local mock coverage for config/auth/client/models and all 24 Confluence tools, including pages/comments/labels/users/history/diff/analytics/attachments.
- GitLab implementation has local mock coverage for config/auth/client/handlers and all 15 GitLab MR tools, including project lookup, MR reads, commits, bounded diffs, pipelines, create/update, notes, discussions, approval state, approval changes, and SHA-gated merge.
- Streamable HTTP exposes `GET /healthz`.
- Security is implemented for unified token/header/error redaction, request-scoped streamable HTTP auth, header-provided service URL SSRF checks, allowed domains, same-origin redirect policy, and MCP session auth fingerprint stability.
- Local stdio, streamable HTTP, and restricted smoke commands validate MCP initialization, Jira, Confluence, and GitLab tool discovery, mock read calls, `/healthz`, and write-tool blocking. GitLab handler paths are also covered by local Rust mock tests.
- Local validation passed `cargo fmt --check`, `cargo check`, `cargo test`, local stdio/HTTP/Jira/Confluence/GitLab smokes, release build, Docker build, compose config, and compose `/healthz` smoke.
- Real acceptance passed Jira core read paths, Jira Agile board lookup, SLA read, development-info single/batch paths, Confluence page/comment/label/analytics/attachment representative paths, and dual-service MCP stdio/HTTP representative calls.
- Local auth/network validation passed `cargo fmt --check`, `cargo check`, `cargo test`, local stdio/HTTP/Jira/Confluence/GitLab smokes, and aggregate smoke.

Deferred:

- `confluence_get_page_view_analytics` is Cloud-only. Confluence Server/Data Center returns a structured unavailable response; real acceptance validated the Cloud representative path.
- Jira Service Management and Forms/ProForma remain objectively blocked in the test tenant: JSM service desk lookup returned 403, and the current Forms client path did not receive an effective Forms API response. These toolsets are implemented with local mock/product-dependency coverage but are not documented as real-accepted.
- `confluence_search_users` is implemented with local mock coverage. Real acceptance did not include a dedicated user-search row, so it remains local-validated only.
- GitLab MR tools are implemented with local mock coverage. Real GitLab acceptance has not been run. Approval endpoints can depend on GitLab tier and token permissions.
- OAuth Cloud 3LO, OAuth proxy/DCR, OAuth refresh/token storage, and Data Center OAuth authorization-code/refresh flows are not implemented and are fixed in the support matrix and backlog.
- SSE transport, SOCKS proxy, and system truststore injection are not implemented in the Rust server. Supported transports are `stdio` and streamable HTTP.
- Release workflow, production deployment documentation, final per-tool support matrix, configuration/auth/transport/network support matrix, fixed long-term backlog, migration-tool cleanup, zero-warning check, final release gate, and completion audit are now complete.

## Validation Result

| Area | Result |
| --- | --- |
| Local Rust regression | Passed: format, check, tests, local stdio/HTTP smoke, Jira restricted smoke, Confluence smoke, GitLab smoke, aggregate smoke. |
| Release/container | Passed: release build, Docker image build, compose config, compose startup, and `/healthz` smoke. |
| Real Jira core | Passed: issue read, JQL search, project issue search, field search/options, watchers read, and disabled-tool guards. |
| Real Jira product paths | Passed for Agile board lookup, SLA read, and development single/batch reads. JSM is blocked by 403 in the test tenant; Forms/ProForma is blocked by product/interface availability. |
| Real Confluence | Passed for search, page read, children/tree, comments, test-object create/update, add/reply comment, labels, Cloud page views, attachments list/download/content/images/upload/batch upload, and disabled-tool delete/move/delete-attachment guards. `confluence_search_users` was not separately real-executed. |
| GitLab MR | Local only: mock REST, MCP handler, discovery/profile, disabled guard, local smoke, project filter, and SHA-gated merge tests passed. Real GitLab acceptance was not run. |
| Dual-service MCP | Passed for stdio and streamable HTTP discovery and representative Jira/Confluence read calls, including `TOOL_PROFILE=basic` and `DISABLED_TOOLS` samples. |

## Requirements

- Rust 1.94 or newer
- just
- curl for manual HTTP checks
- Docker when validating container or compose behavior in later gates
- An MCP client or MCP inspector for manual transport checks

## Quick Start

Configure the Jira, Confluence, and GitLab services you want to expose. They are independent: configure any one service or any combination. `TOOL_PROFILE` defaults to `basic`, so no tool-access variable is required for the smallest setup.

Jira Cloud:

```bash
export JIRA_URL="https://your-company.atlassian.net"
export JIRA_USERNAME="user@example.com"
export JIRA_API_TOKEN="<api-token>"
```

Jira Server/Data Center:

```bash
export JIRA_URL="https://jira.example.com"
export JIRA_PERSONAL_TOKEN="<personal-access-token>"
```

Or, when the private Jira instance still allows Basic username/password auth:

```bash
export JIRA_URL="https://jira.example.com"
export JIRA_USERNAME="<username>"
export JIRA_PASSWORD="<password>"
```

Confluence Cloud:

```bash
export CONFLUENCE_URL="https://your-company.atlassian.net/wiki"
export CONFLUENCE_USERNAME="user@example.com"
export CONFLUENCE_API_TOKEN="<api-token>"
```

Confluence Server/Data Center:

```bash
export CONFLUENCE_URL="https://confluence.example.com"
export CONFLUENCE_PERSONAL_TOKEN="<personal-access-token>"
```

Or, when the private Confluence instance still allows Basic username/password auth:

```bash
export CONFLUENCE_URL="https://confluence.example.com"
export CONFLUENCE_USERNAME="<username>"
export CONFLUENCE_PASSWORD="<password>"
```

GitLab:

```bash
export GITLAB_URL="https://gitlab.example.com"
export GITLAB_TOKEN="<personal-project-or-group-access-token>"
```

For read-only GitLab tools, use a token with `read_api`. For GitLab write, approval, and merge tools, use a token with `api`. `GITLAB_PROJECTS_FILTER` is optional but recommended for production; it is an exact allowlist of numeric project IDs or full paths such as `123,group/project`.

Run over stdio:

```bash
cargo run -- stdio
```

Run over streamable HTTP:

```bash
cargo run -- streamhttp --host 127.0.0.1 --port 8000 --path /mcp
```

The streamable HTTP MCP endpoint is `http://127.0.0.1:8000/mcp`; the health endpoint is `http://127.0.0.1:8000/healthz`. When no command is provided, the binary defaults to `stdio`.

Keep real credentials in a local shell, secret manager, orchestrator secret, or uncommitted dotenv file.

## Tool Access

Most users should choose a single profile and leave lower-level tool controls unset.

| Variable | Default | Behavior |
| --- | --- | --- |
| `TOOL_PROFILE` | `basic` | Supports `basic`, `developer`, `manager`, `full`, or `custom`. Profiles expand to default toolsets; with Jira, Confluence, and GitLab configured they expose 23, 47, 85, 88, or 0 tools respectively. Service availability filters out tools for unconfigured services. Unknown values fail startup. |

Profiles are ordered from least to most capable:

| Profile | Intended use |
| --- | --- |
| `basic` | 23 common Jira, Confluence, and GitLab tools: issue/page/project/MR reads, Jira issue creation, Jira comments, field/project reads, Confluence content/comment/label reads, and GitLab MR read context. No destructive updates or deletes. |
| `developer` | 47 tools: `basic` plus workflow transitions, Agile board/sprint reads, sprint membership changes, development info, issue attachments, issue timeline metrics, Confluence page versions, Confluence attachment reads, and GitLab MR write/approval/merge tools. |
| `manager` | 85 tools: `developer` plus issue updates/deletes, bulk issue creation, issue history, project metadata/version writes, sprint management, worklogs, links, users, watchers, JSM, SLA metrics, Forms, Confluence writes, analytics, and attachment uploads. |
| `full` | All 88 registered Jira, Confluence, and GitLab tools, including Confluence destructive delete toolsets and user search. |
| `custom` | No profile baseline; use `TOOLSETS` and/or exact tool variables. |

Advanced tool overrides:

| Variable | Default | Behavior |
| --- | --- | --- |
| `TOOLSETS` | profile defaults | Comma-separated registered Jira, Confluence, and GitLab toolsets to add to the selected profile. `all` enables every toolset. Unknown names fail startup. |
| `ENABLED_TOOLS` | unset | Comma-separated tool names to add exactly, even when their toolset is not enabled. |
| `DISABLED_TOOLS` | unset | Comma-separated tool names to remove exactly. This takes precedence over profile, toolset, and enabled-tool inclusion. |

## HTTP Deployment

| Variable | Default | Behavior |
| --- | --- | --- |
| `MCP_HTTP_HOST` | `127.0.0.1` | Streamable HTTP host when not overridden by CLI. |
| `MCP_HTTP_PORT` | `8000` | Streamable HTTP port when not overridden by CLI. |
| `MCP_HTTP_PATH` | `/mcp` | Streamable HTTP MCP path when not overridden by CLI. A missing leading slash is normalized. |
| `ENV_FILE` | unset | Optional dotenv file loaded at startup. The `--env-file` CLI argument takes precedence. |
| `MCP_ALLOWED_URL_DOMAINS` | unset | Optional comma-separated domain allowlist for header-provided Jira/Confluence service URLs. Exact domain and subdomain matches are accepted; URL values, IP literals, localhost, and metadata hostnames are rejected. |
| `IGNORE_HEADER_AUTH` | `false` | Truthy values are `true`, `1`, `yes`, `y`, and `on`. When enabled, streamable HTTP ignores all request-scoped auth/service headers and uses only global env service config. |

Docker Compose also supports `MCP_PORT` for host-to-container port mapping. `MCP_PORT` is a compose wrapper variable, not a Rust runtime variable.

Supported transports are `stdio` and streamable HTTP. SSE is not implemented.

## Advanced Atlassian Auth

BYOT access tokens:

- Cloud Jira with `ATLASSIAN_OAUTH_ACCESS_TOKEN` or `JIRA_OAUTH_ACCESS_TOKEN` requires `ATLASSIAN_OAUTH_CLOUD_ID` and uses `https://api.atlassian.com/ex/jira/{cloud_id}`.
- Cloud Confluence with `ATLASSIAN_OAUTH_ACCESS_TOKEN` or `CONFLUENCE_OAUTH_ACCESS_TOKEN` requires `ATLASSIAN_OAUTH_CLOUD_ID` and uses `https://api.atlassian.com/ex/confluence/{cloud_id}/wiki`.
- Server/Data Center PAT takes precedence over BYOT access-token auth. When no PAT is set, Server/Data Center can use the BYOT access token against the configured service URL. When neither PAT nor BYOT is set, Server/Data Center can use Basic username/password auth.

| Variable | Default | Behavior |
| --- | --- | --- |
| `ATLASSIAN_OAUTH_ACCESS_TOKEN` | unset | Shared BYOT/OAuth access token fallback for Jira and Confluence. |
| `JIRA_OAUTH_ACCESS_TOKEN` | unset | Jira-specific BYOT/OAuth access token. Takes precedence over `ATLASSIAN_OAUTH_ACCESS_TOKEN` for Jira. |
| `CONFLUENCE_OAUTH_ACCESS_TOKEN` | unset | Confluence-specific BYOT/OAuth access token. Takes precedence over `ATLASSIAN_OAUTH_ACCESS_TOKEN` for Confluence. |
| `ATLASSIAN_OAUTH_CLOUD_ID` | unset | Required for Cloud BYOT access-token auth. Also used by Jira Forms/ProForma helpers. |
| `ATLASSIAN_OAUTH_ENABLE` | `false` | Truthy values interpret streamable HTTP `Authorization: Bearer` as BYOT/OAuth access-token auth instead of PAT-compatible auth. |
| `ATLASSIAN_USERNAME` / `ATLASSIAN_API_TOKEN` | unset | Shared username/API-token fallback for Jira and Confluence when service-specific values are unset. |
| `ATLASSIAN_USERNAME` / `ATLASSIAN_PASSWORD` | unset | Shared Server/Data Center username/password fallback when service-specific username/password values are unset. |
| `ATLASSIAN_PERSONAL_TOKEN` | unset | Shared PAT fallback for Jira and Confluence Server/Data Center when service-specific PAT values are unset. |

Full OAuth Cloud 3LO, OAuth proxy/DCR, refresh/token storage, and Data Center OAuth authorization-code/refresh flows are not implemented.

## GitLab Auth

`GITLAB_URL` must be the GitLab instance root, such as `https://gitlab.example.com`. If it ends in `/api/v4`, the server normalizes it back to the instance root before building REST API v4 paths.

| Variable | Default | Behavior |
| --- | --- | --- |
| `GITLAB_TOKEN` | unset | Highest-priority GitLab token. Sent as `PRIVATE-TOKEN`. |
| `GITLAB_PERSONAL_TOKEN` | unset | Fallback token. Sent as `PRIVATE-TOKEN`. |
| `GITLAB_OAUTH_ACCESS_TOKEN` | unset | Fallback OAuth access token. Sent as `Authorization: Bearer`. |

GitLab token precedence is `GITLAB_TOKEN`, then `GITLAB_PERSONAL_TOKEN`, then `GITLAB_OAUTH_ACCESS_TOKEN`. Use `read_api` for read-only GitLab tools. Use `api` for create/update/note/discussion/approval/merge tools. GitLab username/password API auth is not supported.

## Service Options

Jira:

| Variable | Default | Behavior |
| --- | --- | --- |
| `JIRA_SSL_VERIFY` | `true` | Set `false`, `0`, `no`, or `off` to disable TLS certificate verification for Jira requests. |
| `JIRA_PROJECTS_FILTER` | unset | Comma-separated project keys. Filters `jira_get_issue` by issue key prefix and injects a project filter into JQL search. |
| `JIRA_TIMEOUT` | `75` | Jira HTTP request timeout in seconds. Must be a positive integer. |

Confluence:

| Variable | Default | Behavior |
| --- | --- | --- |
| `CONFLUENCE_SSL_VERIFY` | `true` | Set `false`, `0`, `no`, or `off` to disable TLS certificate verification for Confluence requests. |
| `CONFLUENCE_SPACES_FILTER` | unset | Comma-separated space keys. Applies to Confluence search when the tool call does not provide `spaces_filter`; an explicit empty `spaces_filter` disables the env filter. |
| `CONFLUENCE_TIMEOUT` | `75` | Confluence HTTP request timeout in seconds. Must be a positive integer. |

GitLab:

| Variable | Default | Behavior |
| --- | --- | --- |
| `GITLAB_SSL_VERIFY` | `true` | Set `false`, `0`, `no`, or `off` to disable TLS certificate verification for GitLab requests. |
| `GITLAB_PROJECTS_FILTER` | unset | Optional exact allowlist of numeric project IDs or full paths. When set, every project-scoped GitLab tool rejects projects outside this set before sending HTTP. |
| `GITLAB_TIMEOUT` | `75` | GitLab HTTP request timeout in seconds. Must be a positive integer. |

Shared fallback options:

| Variable | Default | Behavior |
| --- | --- | --- |
| `ATLASSIAN_SSL_VERIFY` | `true` | Shared TLS verification fallback when `JIRA_SSL_VERIFY` or `CONFLUENCE_SSL_VERIFY` is unset. |
| `ATLASSIAN_TIMEOUT` | `75` | Shared positive timeout fallback when `JIRA_TIMEOUT` or `CONFLUENCE_TIMEOUT` is unset. |

## Network And TLS

Proxy and custom headers:

| Variable | Behavior |
| --- | --- |
| `HTTP_PROXY` / `HTTPS_PROXY` / `NO_PROXY` | Standard proxy fallback when service-specific and `ATLASSIAN_*` proxy variables are unset. |
| `ATLASSIAN_HTTP_PROXY` / `ATLASSIAN_HTTPS_PROXY` / `ATLASSIAN_NO_PROXY` | Shared Atlassian proxy fallback for Jira and Confluence. Takes precedence over standard proxy variables. |
| `JIRA_HTTP_PROXY` / `JIRA_HTTPS_PROXY` / `JIRA_NO_PROXY` | Jira-specific proxy config. |
| `CONFLUENCE_HTTP_PROXY` / `CONFLUENCE_HTTPS_PROXY` / `CONFLUENCE_NO_PROXY` | Confluence-specific proxy config. |
| `GITLAB_HTTP_PROXY` / `GITLAB_HTTPS_PROXY` / `GITLAB_NO_PROXY` | GitLab-specific proxy config. Falls back to standard `HTTP_PROXY`, `HTTPS_PROXY`, and `NO_PROXY`, not `ATLASSIAN_*`. |
| `JIRA_CUSTOM_HEADERS` | Jira outbound headers as comma-separated `Name=value` pairs. |
| `CONFLUENCE_CUSTOM_HEADERS` | Confluence outbound headers as comma-separated `Name=value` pairs. |
| `GITLAB_CUSTOM_HEADERS` | GitLab outbound headers as comma-separated `Name=value` pairs. Does not use `ATLASSIAN_CUSTOM_HEADERS`. |
| `ATLASSIAN_CUSTOM_HEADERS` | Shared custom outbound headers fallback when service-specific custom headers are unset. |

HTTP/HTTPS proxy URLs must use `http` or `https`. Reserved custom header names are rejected: `Authorization`, `Cookie`, `Set-Cookie`, `Proxy-Authorization`, `Host`, `Content-Type`, `Content-Length`, `Transfer-Encoding`, `Connection`, `Private-Token`, `Job-Token`, `X-Atlassian-Jira-Personal-Token`, `X-Atlassian-Confluence-Personal-Token`, `X-Atlassian-Jira-Url`, `X-Atlassian-Confluence-Url`, and `X-Atlassian-Cloud-Id`.

mTLS:

| Variable | Behavior |
| --- | --- |
| `JIRA_CLIENT_CERT` / `JIRA_CLIENT_KEY` | Jira PEM client certificate and key paths. Must be set together. |
| `CONFLUENCE_CLIENT_CERT` / `CONFLUENCE_CLIENT_KEY` | Confluence PEM client certificate and key paths. Must be set together. |
| `GITLAB_CLIENT_CERT` / `GITLAB_CLIENT_KEY` | GitLab PEM client certificate and key paths. Must be set together. Does not use `ATLASSIAN_CLIENT_CERT` / `ATLASSIAN_CLIENT_KEY`. |
| `ATLASSIAN_CLIENT_CERT` / `ATLASSIAN_CLIENT_KEY` | Shared mTLS fallback when service-specific mTLS variables are unset. Must be set together. |

SOCKS proxy support is not compiled in.

## Diagnostics

| Variable | Default | Behavior |
| --- | --- | --- |
| `MCP_TOOL_CALL_DEBUG` | `false` | Enables MCP tool-call diagnostics when `RUST_LOG` is unset. Arguments are redacted and truncated, but can still contain business data. |
| `RUST_LOG` | unset | Advanced tracing filter. Takes precedence over `MCP_TOOL_CALL_DEBUG`. |

## Confluence Content Conversion Boundary

The Confluence implementation uses a deterministic minimal Markdown to Confluence storage conversion for local mock validation. It covers headings, paragraphs, unordered lists, simple inline links, fenced code blocks, line breaks, and HTML escaping.

The Rust implementation does not claim full `md2conf` feature parity. Mermaid rendering, macro rendering, and full heading anchor parity remain outside the current local Confluence loop.

## Security And Request Auth

Security is active for the streamable HTTP MCP endpoint. Request-scoped Bearer handling supports Atlassian BYOT access tokens. These request-scoped headers do not affect `stdio` global-env behavior except for shared redaction and outbound redirect policy. GitLab request-scoped service headers are not implemented; configure GitLab through global `GITLAB_*` environment variables.

Supported request-scoped headers:

| Header | Behavior |
| --- | --- |
| `Authorization: Basic <base64(email:api_token)>` or `Authorization: Basic <base64(username:password)>` | Overrides credentials for already configured Jira/Confluence services in the current HTTP request or MCP session. |
| `Authorization: Token <pat>` | Uses the token as a PAT-compatible credential for already configured services. |
| `Authorization: Bearer <token>` | Uses the token as a BYOT/OAuth access token when `X-Atlassian-Cloud-Id` is present or global `ATLASSIAN_OAUTH_ENABLE=true`; otherwise keeps the PAT-compatible behavior. |
| `X-Atlassian-Jira-Url` + `X-Atlassian-Jira-Personal-Token` | Creates or overrides the Jira config for the current request/session after URL validation. |
| `X-Atlassian-Confluence-Url` + `X-Atlassian-Confluence-Personal-Token` | Creates or overrides the Confluence config for the current request/session after URL validation. |
| `X-Atlassian-Cloud-Id` | Sets request-scoped Cloud ID context and is a BYOT signal for `Authorization: Bearer`. |

Security behavior:

- Header-provided service URL/token values must be paired. Missing pairs are rejected.
- Header-provided service URLs must use `http` or `https`, have a hostname, and cannot target localhost, metadata hostnames, private, loopback, link-local, multicast, unspecified, documentation, or DNS-resolved non-global IP addresses.
- When `MCP_ALLOWED_URL_DOMAINS` is set, header-provided service URLs must match an allowed domain or subdomain.
- Outbound upstream HTTP redirects are limited to same-origin `http`/`https` redirects, maximum 3 hops.
- Request-scoped BYOT, service URL, and credential overrides apply only to the current streamable HTTP request or MCP session and do not mutate global env service config.
- Request auth fingerprint is bound to `Mcp-Session-Id`; changing request auth or token type within the same MCP session is rejected.
- Logs, MCP debug argument output, acceptance compact errors, HTTP status summaries, and URL query values are redacted for secret-looking header, token, cookie, password, key, signature, and env secret values.

## MCP Tools

The Rust server exposes these Jira core tools when Jira is configured:

| Tool | Access | Toolset |
| --- | --- | --- |
| `jira_get_issue` | read | `jira_issues_read` |
| `jira_search_issues` | read | `jira_issues_read` |
| `jira_list_project_issues` | read | `jira_issues_read` |
| `jira_search_fields` | read | `jira_fields_read` |
| `jira_list_field_options` | read | `jira_fields_read` |
| `jira_add_issue_comment` | write | `jira_issue_comments_write` |
| `jira_update_issue_comment` | write | `jira_issue_comments_update` |
| `jira_list_issue_transitions` | read | `jira_issue_workflows_read` |
| `jira_transition_issue` | write | `jira_issue_workflows_write` |

The Rust server also exposes these Jira extended tools when Jira is configured. These are locally validated with mock Jira. Real acceptance passed representative Jira core, Agile, SLA, and development paths; Jira Service Management and Forms/ProForma remain objectively blocked as described above.

| Tool | Access | Toolset |
| --- | --- | --- |
| `jira_create_issue` | write | `jira_issues_write` |
| `jira_create_issues` | write | `jira_issues_bulk_write` |
| `jira_get_issue_changelogs` | read | `jira_issues_history_read` |
| `jira_update_issue` | write | `jira_issues_update` |
| `jira_delete_issue` | write | `jira_issues_delete` |
| `jira_list_projects` | read | `jira_projects_read` |
| `jira_list_project_versions` | read | `jira_projects_metadata_read` |
| `jira_list_project_components` | read | `jira_projects_metadata_read` |
| `jira_create_project_version` | write | `jira_project_versions_write` |
| `jira_create_project_versions` | write | `jira_project_versions_write` |
| `jira_get_user` | read | `jira_users_read` |
| `jira_list_issue_watchers` | read | `jira_issue_watchers_read` |
| `jira_add_issue_watcher` | write | `jira_issue_watchers_write` |
| `jira_remove_issue_watcher` | write | `jira_issue_watchers_delete` |
| `jira_list_issue_worklogs` | read | `jira_issue_worklogs_read` |
| `jira_add_issue_worklog` | write | `jira_issue_worklogs_write` |
| `jira_list_issue_link_types` | read | `jira_issue_links_read` |
| `jira_set_issue_parent` | write | `jira_issue_links_write` |
| `jira_create_issue_link` | write | `jira_issue_links_write` |
| `jira_create_remote_issue_link` | write | `jira_issue_links_write` |
| `jira_delete_issue_link` | write | `jira_issue_links_delete` |
| `jira_get_issue_attachments` | read | `jira_issue_attachments_read` |
| `jira_get_issue_image_attachments` | read | `jira_issue_attachments_read` |
| `jira_list_agile_boards` | read | `jira_agile_boards_read` |
| `jira_list_board_issues` | read | `jira_agile_boards_read` |
| `jira_list_board_sprints` | read | `jira_sprints_read` |
| `jira_list_sprint_issues` | read | `jira_sprints_read` |
| `jira_create_sprint` | write | `jira_sprints_write` |
| `jira_update_sprint` | write | `jira_sprints_write` |
| `jira_add_issues_to_sprint` | write | `jira_sprint_membership_write` |
| `jira_get_project_service_desk` | read | `jira_service_desks_read` |
| `jira_list_service_desk_queues` | read | `jira_service_desks_read` |
| `jira_list_service_desk_queue_issues` | read | `jira_service_desks_read` |
| `jira_list_issue_forms` | read | `jira_issue_forms_read` |
| `jira_get_issue_form` | read | `jira_issue_forms_read` |
| `jira_update_issue_form_answers` | write | `jira_issue_forms_write` |
| `jira_get_issue_timeline` | read | `jira_issue_metrics_read` |
| `jira_get_issue_sla_metrics` | read | `jira_issue_sla_read` |
| `jira_get_issue_development` | read | `jira_issue_development_read` |
| `jira_get_issues_development` | read | `jira_issue_development_read` |

`jira_get_issue_sla_metrics` parses SLA values from Jira/JSM issue fields and returns `parsing_limitations`; it does not apply a local working-hours calendar or recompute SLA timers.

The Rust server also exposes these Confluence tools when Confluence is configured. These are locally validated with mock Confluence. Real acceptance passed representative pages, comments, labels, analytics, and attachments paths on test objects; `confluence_search_users` remains local-validated only.

| Tool | Access | Toolset |
| --- | --- | --- |
| `confluence_search_content` | read | `confluence_content_read` |
| `confluence_get_page` | read | `confluence_content_read` |
| `confluence_list_page_children` | read | `confluence_content_read` |
| `confluence_get_space_page_tree` | read | `confluence_content_read` |
| `confluence_create_page` | write | `confluence_content_write` |
| `confluence_update_page` | write | `confluence_content_update` |
| `confluence_delete_page` | write | `confluence_content_delete` |
| `confluence_move_page` | write | `confluence_content_update` |
| `confluence_list_page_comments` | read | `confluence_page_comments_read` |
| `confluence_add_page_comment` | write | `confluence_page_comments_write` |
| `confluence_reply_to_comment` | write | `confluence_page_comments_write` |
| `confluence_list_content_labels` | read | `confluence_content_labels_read` |
| `confluence_add_content_label` | write | `confluence_content_labels_write` |
| `confluence_search_users` | read | `confluence_users_read` |
| `confluence_get_page_version` | read | `confluence_page_versions_read` |
| `confluence_get_page_diff` | read | `confluence_page_versions_read` |
| `confluence_get_page_view_analytics` | read | `confluence_page_analytics_read` |
| `confluence_upload_content_attachment` | write | `confluence_attachments_write` |
| `confluence_upload_content_attachments` | write | `confluence_attachments_write` |
| `confluence_list_content_attachments` | read | `confluence_attachments_read` |
| `confluence_download_attachment` | read | `confluence_attachments_read` |
| `confluence_download_content_attachments` | read | `confluence_attachments_read` |
| `confluence_delete_attachment` | write | `confluence_attachments_delete` |
| `confluence_get_content_image_attachments` | read | `confluence_attachments_read` |

`confluence_list_page_children` applies the requested limit to the combined page/folder result set and returns page/folder query statistics. `confluence_create_page` and `confluence_update_page` return `emoji_status` for the optional emoji sub-operation. `confluence_get_page_view_analytics` is Cloud-only. Attachment download/image tools return bounded structured content; the current inline content limit is 1 MiB per attachment. `confluence_download_content_attachments` paginates attachment listings up to 10 pages and returns `has_more`, `next_start`, `pages_fetched`, and `limit_applied` in the summary. Attachment upload tools accept explicit local file paths readable by the server process, reject files larger than 10 MiB before reading them, and do not implement directory allowlists or remote URL upload in this release.

The Rust server also exposes these GitLab merge-request tools when GitLab is configured. These are locally validated with mock GitLab REST and MCP handler tests. Real GitLab acceptance was not run; approval endpoints can depend on GitLab tier and token permissions.

| Tool | Access | Toolset |
| --- | --- | --- |
| `gitlab_get_current_user` | read | `gitlab_projects_read` |
| `gitlab_get_project` | read | `gitlab_projects_read` |
| `gitlab_list_merge_requests` | read | `gitlab_merge_requests_read` |
| `gitlab_get_merge_request` | read | `gitlab_merge_requests_read` |
| `gitlab_list_merge_request_commits` | read | `gitlab_merge_requests_read` |
| `gitlab_list_merge_request_diffs` | read | `gitlab_merge_requests_read` |
| `gitlab_list_merge_request_pipelines` | read | `gitlab_merge_requests_read` |
| `gitlab_create_merge_request` | write | `gitlab_merge_requests_write` |
| `gitlab_update_merge_request` | write | `gitlab_merge_requests_write` |
| `gitlab_add_merge_request_note` | write | `gitlab_merge_requests_write` |
| `gitlab_reply_merge_request_discussion` | write | `gitlab_merge_requests_write` |
| `gitlab_resolve_merge_request_discussion` | write | `gitlab_merge_requests_write` |
| `gitlab_get_merge_request_approval_state` | read | `gitlab_merge_requests_read` |
| `gitlab_set_merge_request_approval` | write | `gitlab_merge_requests_write` |
| `gitlab_accept_merge_request` | write | `gitlab_merge_requests_merge` |

GitLab project arguments accept numeric project IDs or full paths such as `group/project`; full paths are encoded for REST API v4. MR commit, diff, and pipeline list tools accept `page`/`per_page`; `gitlab_list_merge_request_diffs` returns bounded diff content for the selected page with truncation metadata. `gitlab_accept_merge_request` requires a non-empty `sha` and sends it to GitLab's merge endpoint to guard against merging an unreviewed head.

## Commands

Run `just --list` to see the local command surface.

```bash
just dev           # run stdio transport
just dev-http      # run streamable HTTP transport on 127.0.0.1:8000
just smoke             # run all local smoke checks
just smoke-gitlab      # GitLab stdio, HTTP, and restricted smoke with a local mock GitLab
just acceptance-jira        # run real Jira acceptance using .env.dev by default
just acceptance-confluence  # run real Confluence acceptance using .env.dev by default
just acceptance-mcp         # run real dual-service MCP acceptance using .env.dev by default
just build             # build the release binary
just test              # run tests
just check             # fmt, check, and tests
just docker-build      # local Docker image build
```

Detailed development-only smoke, acceptance, `xtask`, and test-object variable documentation lives in [`docs/development-tools.md`](docs/development-tools.md).

## Release Artifacts

Releases are tag-driven. A release tag must use the form `vX.Y.Z`, match `Cargo.toml` `package.version = "X.Y.Z"`, and have a matching `## X.Y.Z` entry in `CHANGELOG.md`.

The release workflow builds a Linux x86_64 release binary archive:

```text
mcp-atlassian-rs-linux-x86_64.tar.gz
mcp-atlassian-rs-linux-x86_64.tar.gz.sha256
```

The current release process does not publish to crates.io, GHCR, Docker Hub, or any other external registry.

## Production Deployment

See [`docs/deployment.md`](docs/deployment.md) for the supported stdio, streamable HTTP, Docker, compose, authentication, network, TLS, security, and unsupported-capability deployment guidance.

See [`docs/support-matrix.md`](docs/support-matrix.md) and [`docs/backlog.md`](docs/backlog.md) for current support status and fixed future work. The support matrix includes the final 49-tool Jira, 24-tool Confluence, and 16-tool GitLab per-tool release status plus runtime, auth, transport, network, TLS, and security support boundaries. The backlog covers OAuth flow gaps, SSE, SOCKS, system truststore, Helm, external registry publishing, crates.io publishing, real-acceptance follow-up, and Confluence content-conversion parity.

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

Set `MCP_PORT` to change the host port used by compose. The compose file passes through runtime control variables plus Jira, Confluence, GitLab, shared `ATLASSIAN_*`, and proxy variables from the shell or compose environment.

The compose service includes a `/healthz` healthcheck and runs the binary as the non-root `app` user from the image. Keep secrets in your deployment secret manager or local shell environment; do not commit dotenv files with real Atlassian credentials.

Tool-call diagnostics can be enabled in release binaries, Docker, or compose with `MCP_TOOL_CALL_DEBUG=true`. This prints MCP tool names, elapsed time, failures, and redacted JSON arguments to stderr when `RUST_LOG` is unset. `RUST_LOG` remains the advanced override; the equivalent filter is `RUST_LOG=mcp_atlassian_rs::mcp=debug,mcp_atlassian_rs=info,rmcp=info`. Diagnostic arguments are redacted and truncated, but can still contain business data such as JQL, issue keys, page IDs, summaries, or descriptions, so enable them only during troubleshooting.

Release validation covered `just docker-build`, compose config, compose startup, and `GET /healthz` with `MCP_PORT=18080`. The current release includes release artifact policy, deployment documentation, Docker/compose health validation, and final release gate; Helm and external registry publishing remain out of scope.

## Verification

Local checks:

```bash
cargo fmt --check
cargo check
cargo test
just check
just smoke
```

The smoke commands start local mock Jira, Confluence, and GitLab servers and do not require real Atlassian or GitLab credentials. Real acceptance commands require disposable test objects and development-only credentials; see [`docs/development-tools.md`](docs/development-tools.md).

## License

Licensed under the MIT License. See [LICENSE](LICENSE).

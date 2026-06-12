# Configuration

This document is the detailed configuration reference for `workhub-rs`. For deployment shapes and operational guidance, see [deployment.md](deployment.md). For per-tool support status, see [support-matrix.md](support-matrix.md).

## Service Credentials

Configure any one provider or any combination. Tools for unconfigured services are filtered out automatically.

### Jira

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

If the private Jira instance still allows Basic username/password auth:

```bash
export JIRA_URL="https://jira.example.com"
export JIRA_USERNAME="<username>"
export JIRA_PASSWORD="<password>"
```

### Confluence

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

If the private Confluence instance still allows Basic username/password auth:

```bash
export CONFLUENCE_URL="https://confluence.example.com"
export CONFLUENCE_USERNAME="<username>"
export CONFLUENCE_PASSWORD="<password>"
```

### GitLab

```bash
export GITLAB_URL="https://gitlab.example.com"
export GITLAB_TOKEN="<personal-project-or-group-access-token>"
```

`GITLAB_URL` must be the GitLab instance root. If it ends in `/api/v4`, the server normalizes it back to the instance root before building REST API v4 paths.

| Variable | Default | Behavior |
| --- | --- | --- |
| `GITLAB_TOKEN` | unset | Highest-priority GitLab token. Sent as `PRIVATE-TOKEN`. |
| `GITLAB_PERSONAL_TOKEN` | unset | Fallback token. Sent as `PRIVATE-TOKEN`. |
| `GITLAB_PROJECTS_FILTER` | unset | Optional exact allowlist of numeric project IDs or full paths. Every project-scoped GitLab tool rejects projects outside this set before sending HTTP. |

GitLab token precedence is `GITLAB_TOKEN`, then `GITLAB_PERSONAL_TOKEN`. Use `read_api` for read-only tools and `api` for writes, approvals, and merge. GitLab username/password API auth is not supported.

## MCP Tool Access

Most users should choose a single MCP profile and leave lower-level controls unset. These variables only affect MCP tool discovery and MCP tool calls. The resource CLI ignores them and exposes its full command surface for configured services.

| Variable | Default | Behavior |
| --- | --- | --- |
| `MCP_TOOL_PROFILE` | `basic` | Supports `basic`, `developer`, `manager`, `full`, or `custom`. With Jira, Confluence, and GitLab configured, profiles expose 23, 47, 82, 85, or 0 tools respectively. Service availability filters out tools for unconfigured services. |
| `MCP_TOOLSETS` | profile defaults | Adds comma-separated registered toolsets to the selected profile. `all` enables every toolset. Unknown names fail startup. |
| `MCP_ENABLED_TOOLS` | unset | Adds comma-separated exact MCP tool names, even when their toolset is not enabled. |
| `MCP_DISABLED_TOOLS` | unset | Removes comma-separated exact MCP tool names. This takes precedence over profile, toolset, and enabled-tool inclusion. |

Profiles are ordered from least to most capable:

| Profile | Intended use |
| --- | --- |
| `basic` | Common Jira, Confluence, and GitLab reads plus limited safe writes such as Jira issue creation/comments. |
| `developer` | Adds workflow, Agile, attachment, development-info, Confluence version/attachment reads, and GitLab MR write/approval/merge tools. |
| `manager` | Adds most Jira project, sprint, worklog, link, JSM, SLA, Forms, Confluence write, analytics, and attachment upload tools. |
| `full` | All registered tools, including destructive Confluence delete toolsets. |
| `custom` | No profile baseline; use `MCP_TOOLSETS` and/or exact tool variables. |

## Runtime And HTTP

| Variable | Default | Behavior |
| --- | --- | --- |
| `MCP_HTTP_HOST` | `127.0.0.1` | Streamable HTTP host when not overridden by CLI. |
| `MCP_HTTP_PORT` | `8000` | Streamable HTTP port when not overridden by CLI. |
| `MCP_HTTP_PATH` | `/mcp` | Streamable HTTP MCP path when not overridden by CLI. A missing leading slash is normalized. |
| `ENV_FILE` | unset | Optional dotenv file loaded by `streamhttp` and `cli` startup. The explicit `--env-file <path>` argument takes precedence. |

Docker Compose also supports `MCP_PORT` for host-to-container port mapping. `MCP_PORT` is a compose wrapper variable, not a Rust runtime variable.

Supported MCP transports are `stdio` and streamable HTTP. SSE is not implemented.

The production resource CLI is available through:

```bash
workhub cli [--env-file <path>] [--json] [--pretty] <provider> ...
```

Startup dotenv behavior is intentionally mode-specific:

| Mode | Dotenv behavior |
| --- | --- |
| `stdio` | Does not load `.env`, `ENV_FILE`, or `--env-file`; stdout remains protocol-only. |
| `streamhttp` | Loads explicit `--env-file`, then `ENV_FILE`, then current directory `.env`; missing default `.env` is ignored. |
| `cli` | Uses the same dotenv priority as `streamhttp`. |

The CLI has no provider URL, token, password, proxy, custom-header, TLS, or mTLS override flags. Use environment variables, `ENV_FILE`, or `--env-file` so credentials do not enter shell history or process arguments.

## Atlassian Auth

Jira and Confluence read service-specific credential variables first, then shared `ATLASSIAN_*` fallback variables. Cloud deployments use username/API-token Basic Auth. Server/Data Center deployments use PAT first, then username/password Basic Auth.

| Variable | Default | Behavior |
| --- | --- | --- |
| `ATLASSIAN_USERNAME` / `ATLASSIAN_API_TOKEN` | unset | Shared username/API-token fallback for Jira and Confluence when service-specific values are unset. |
| `ATLASSIAN_USERNAME` / `ATLASSIAN_PASSWORD` | unset | Shared Server/Data Center username/password fallback when service-specific username/password values are unset. |
| `ATLASSIAN_PERSONAL_TOKEN` | unset | Shared PAT fallback for Jira and Confluence Server/Data Center when service-specific PAT values are unset. |

Full OAuth Cloud 3LO, OAuth proxy/DCR, refresh/token storage, and Data Center OAuth authorization-code/refresh flows are not implemented.

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

HTTP/HTTPS proxy URLs must use `http` or `https`. Reserved custom header names are rejected: `Authorization`, `Cookie`, `Set-Cookie`, `Proxy-Authorization`, `Host`, `Content-Type`, `Content-Length`, `Transfer-Encoding`, `Connection`, `Private-Token`, and `Job-Token`.

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

The equivalent diagnostic filter is `workhub_rs::mcp=debug,workhub_rs=info,rmcp=info`.

## Streamable HTTP Endpoint

The streamable HTTP MCP endpoint uses the same process-wide service configuration as stdio. Incoming HTTP headers are handled by the MCP transport and do not change Jira, Confluence, or GitLab upstream identity.

Security behavior:

- Protect public HTTP deployments with a fronting gateway or network boundary appropriate for your environment.
- Outbound upstream HTTP redirects are limited to same-origin `http`/`https` redirects, maximum 3 hops.
- Logs, MCP debug argument output, acceptance compact errors, HTTP status summaries, and URL query values are redacted for secret-looking header, token, cookie, password, key, signature, and env secret values.

## Confluence Content Conversion

The Confluence implementation uses deterministic minimal Markdown-to-Confluence storage conversion for local mock validation. It covers headings, paragraphs, unordered lists, simple inline links, fenced code blocks, line breaks, and HTML escaping.

The Rust implementation does not claim full `md2conf` feature parity. Mermaid rendering, macro rendering, and full heading anchor parity remain outside the current local Confluence loop.

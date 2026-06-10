# Production Deployment

This guide describes the supported runtime shapes for `mcp-atlassian-rs`.

## Deployment Checklist

- Choose one supported transport: `stdio` for local MCP clients or streamable HTTP for server deployments.
- Configure only the Jira, Confluence, and GitLab services you want exposed.
- Restrict exposed tools with `TOOL_PROFILE`, `TOOLSETS`, `ENABLED_TOOLS`, or `DISABLED_TOOLS` when the client should not see every configured tool.
- Set `MCP_ALLOWED_URL_DOMAINS` before accepting request-scoped Atlassian service URLs over streamable HTTP.
- Keep Atlassian and GitLab credentials in a secret manager, shell environment, or orchestrator secret. Do not commit dotenv files with real credentials.
- Check `GET /healthz` for streamable HTTP deployments.
- Review `SECURITY.md` before exposing the HTTP endpoint beyond localhost.

## Stdio

Use stdio when an MCP client starts the server process directly:

```bash
mcp-atlassian-rs stdio
```

Logs are written to stderr. Stdout is reserved for the MCP protocol.

To include MCP tool call names, elapsed time, failures, and redacted arguments in stderr logs, enable tool-call diagnostics:

```bash
MCP_TOOL_CALL_DEBUG=true mcp-atlassian-rs stdio
```

`RUST_LOG` remains the advanced logging control and takes precedence over `MCP_TOOL_CALL_DEBUG` when set.

## Streamable HTTP

Use streamable HTTP for server deployments:

```bash
mcp-atlassian-rs streamhttp --host 0.0.0.0 --port 8000 --path /mcp
```

The health endpoint is:

```text
GET /healthz
```

The MCP endpoint path defaults to `/mcp` and can be set with `MCP_HTTP_PATH` or `--path`.

To enable tool-call diagnostics for streamable HTTP:

```bash
MCP_TOOL_CALL_DEBUG=true mcp-atlassian-rs streamhttp --host 0.0.0.0 --port 8000 --path /mcp
```

## Docker And Compose

Build the local image:

```bash
docker build -t mcp-atlassian-rs:local -f Dockerfile .
```

Run the image:

```bash
docker run --rm -p 8000:8000 mcp-atlassian-rs:local
```

Run the image with tool-call diagnostics:

```bash
docker run --rm -e MCP_TOOL_CALL_DEBUG=true -p 8000:8000 mcp-atlassian-rs:local
```

Run with compose:

```bash
docker compose up --build
```

The image runs as a non-root `app` user. The compose service includes a `/healthz` healthcheck and maps `${MCP_PORT:-8000}` on the host to container port `8000`.

Compose passes through runtime control variables plus Jira, Confluence, GitLab, shared `ATLASSIAN_*`, and proxy variables. For example:

```bash
MCP_TOOL_CALL_DEBUG=true docker compose up --build
```

## Runtime Controls

| Variable | Deployment use |
| --- | --- |
| `TOOL_PROFILE` | Set `basic`, `developer`, `manager`, `full`, or `custom`. Defaults to `basic`. With Jira, Confluence, and GitLab configured, profiles expose 23, 47, 85, 88, or 0 tools respectively. Unknown values fail startup. |
| `TOOLSETS` | Add comma-separated toolset names to the selected profile. `all` enables every toolset. Unknown names fail startup. |
| `ENABLED_TOOLS` | Add comma-separated exact tool names. |
| `DISABLED_TOOLS` | Remove comma-separated exact tool names. Takes precedence over profile/toolset inclusion. |
| `MCP_HTTP_HOST` / `MCP_HTTP_PORT` / `MCP_HTTP_PATH` | Configure streamable HTTP when CLI flags are not used. Ignored by stdio startup. |
| `MCP_PORT` | Compose-only host port mapping. Does not configure the Rust process itself. |
| `ENV_FILE` | Optional dotenv file loaded at startup. The `--env-file` CLI argument takes precedence. |
| `IGNORE_HEADER_AUTH` | Set `true` to ignore request-scoped auth/service headers and use only global environment config. |
| `MCP_ALLOWED_URL_DOMAINS` | Restrict header-provided Jira/Confluence service URLs to exact domains or subdomains. |
| `MCP_TOOL_CALL_DEBUG` | Set `true` to enable MCP tool-call diagnostics when `RUST_LOG` is unset. Uses `mcp_atlassian_rs::mcp=debug,mcp_atlassian_rs=info,rmcp=info`. |
| `RUST_LOG` | Advanced tracing filter. Takes precedence over `MCP_TOOL_CALL_DEBUG`. |

## Jira, Confluence, And GitLab Auth

Supported global auth:

- Jira Cloud: `JIRA_URL`, `JIRA_USERNAME`, `JIRA_API_TOKEN`.
- Jira Server/Data Center: `JIRA_URL`, `JIRA_PERSONAL_TOKEN`, or `JIRA_URL`, `JIRA_USERNAME`, `JIRA_PASSWORD`.
- Confluence Cloud: `CONFLUENCE_URL`, `CONFLUENCE_USERNAME`, `CONFLUENCE_API_TOKEN`.
- Confluence Server/Data Center: `CONFLUENCE_URL`, `CONFLUENCE_PERSONAL_TOKEN`, or `CONFLUENCE_URL`, `CONFLUENCE_USERNAME`, `CONFLUENCE_PASSWORD`.
- GitLab: `GITLAB_URL` plus `GITLAB_TOKEN`, `GITLAB_PERSONAL_TOKEN`, or `GITLAB_OAUTH_ACCESS_TOKEN`.
- Shared auth fallbacks: `ATLASSIAN_USERNAME`, `ATLASSIAN_API_TOKEN`, `ATLASSIAN_PASSWORD`, `ATLASSIAN_PERSONAL_TOKEN`.
- BYOT access token: `ATLASSIAN_OAUTH_ACCESS_TOKEN`, or service-specific `JIRA_OAUTH_ACCESS_TOKEN` / `CONFLUENCE_OAUTH_ACCESS_TOKEN`.

Cloud BYOT access-token auth requires `ATLASSIAN_OAUTH_CLOUD_ID`. Jira Cloud BYOT uses `https://api.atlassian.com/ex/jira/{cloud_id}`. Confluence Cloud BYOT uses `https://api.atlassian.com/ex/confluence/{cloud_id}/wiki`.

GitLab token precedence is `GITLAB_TOKEN`, then `GITLAB_PERSONAL_TOKEN`, then `GITLAB_OAUTH_ACCESS_TOKEN`. `GITLAB_TOKEN` and `GITLAB_PERSONAL_TOKEN` are sent as `PRIVATE-TOKEN`; `GITLAB_OAUTH_ACCESS_TOKEN` is sent as `Authorization: Bearer`. Use `read_api` for read-only GitLab tools and `api` for GitLab writes, approvals, and merge. GitLab username/password API auth is not supported.

`GITLAB_URL` should be the instance root, such as `https://gitlab.example.com`. A value ending in `/api/v4` is normalized back to the instance root. Set `GITLAB_PROJECTS_FILTER` to an exact comma-separated allowlist of numeric project IDs or full paths, such as `123,group/project`, to prevent project-scoped GitLab tools from reaching other projects.

Supported streamable HTTP request-scoped auth:

- `Authorization: Basic <base64(email:api_token)>` or `Authorization: Basic <base64(username:password)>`.
- `Authorization: Token <pat>`.
- `Authorization: Bearer <token>`.
- `X-Atlassian-Jira-Url` plus `X-Atlassian-Jira-Personal-Token`.
- `X-Atlassian-Confluence-Url` plus `X-Atlassian-Confluence-Personal-Token`.
- `X-Atlassian-Cloud-Id`.

Bearer tokens are interpreted as BYOT/OAuth access tokens when `X-Atlassian-Cloud-Id` is present or `ATLASSIAN_OAUTH_ENABLE=true`. Otherwise Bearer keeps the PAT-compatible behavior. Server/Data Center global env auth precedence is PAT, then BYOT access token, then username/password Basic Auth. GitLab does not implement request-scoped service/auth override headers; configure GitLab through global `GITLAB_*` environment variables.

## Network And TLS

Supported outbound network controls:

- Jira proxy: `JIRA_HTTP_PROXY`, `JIRA_HTTPS_PROXY`, `JIRA_NO_PROXY`.
- Confluence proxy: `CONFLUENCE_HTTP_PROXY`, `CONFLUENCE_HTTPS_PROXY`, `CONFLUENCE_NO_PROXY`.
- GitLab proxy: `GITLAB_HTTP_PROXY`, `GITLAB_HTTPS_PROXY`, `GITLAB_NO_PROXY`.
- Shared Atlassian proxy fallback: `ATLASSIAN_HTTP_PROXY`, `ATLASSIAN_HTTPS_PROXY`, `ATLASSIAN_NO_PROXY`.
- Global proxy fallback: `HTTP_PROXY`, `HTTPS_PROXY`, `NO_PROXY`.
- Jira custom outbound headers: `JIRA_CUSTOM_HEADERS`.
- Confluence custom outbound headers: `CONFLUENCE_CUSTOM_HEADERS`.
- GitLab custom outbound headers: `GITLAB_CUSTOM_HEADERS`.
- Shared custom outbound headers fallback: `ATLASSIAN_CUSTOM_HEADERS`.
- Jira mTLS: `JIRA_CLIENT_CERT`, `JIRA_CLIENT_KEY`.
- Confluence mTLS: `CONFLUENCE_CLIENT_CERT`, `CONFLUENCE_CLIENT_KEY`.
- GitLab mTLS: `GITLAB_CLIENT_CERT`, `GITLAB_CLIENT_KEY`.
- Shared mTLS fallback: `ATLASSIAN_CLIENT_CERT`, `ATLASSIAN_CLIENT_KEY`.
- TLS verification toggles: `JIRA_SSL_VERIFY`, `CONFLUENCE_SSL_VERIFY`, `GITLAB_SSL_VERIFY`, `ATLASSIAN_SSL_VERIFY`.
- Timeout controls: `JIRA_TIMEOUT`, `CONFLUENCE_TIMEOUT`, `GITLAB_TIMEOUT`, `ATLASSIAN_TIMEOUT`.

Reserved auth, cookie, host, content, proxy, connection, GitLab token headers, and request-scoped Atlassian headers are rejected in custom outbound headers. GitLab does not use the shared `ATLASSIAN_*` fallback variables for proxy, custom headers, mTLS, TLS verification, or timeout.

## Security Behavior

- Secret-looking values are redacted from logs, MCP debug output, development acceptance output, URL query values, and upstream error summaries.
- MCP tool-call diagnostics include redacted JSON arguments. They can still include business data such as JQL, issue keys, page IDs, summaries, or descriptions, so enable them only while troubleshooting.
- Header-provided service URLs are validated for scheme, hostname, blocked hostnames, IP ranges, DNS results, and optional allowed domains.
- Outbound upstream redirects are same-origin only and limited to 3 hops.
- Request-scoped auth applies only to the current streamable HTTP request or MCP session and does not mutate global environment config.
- `Mcp-Session-Id` is bound to the request auth fingerprint. Changing auth or token type within the same session is rejected.

## Unsupported In The Current Rust Release

- OAuth Cloud 3LO authorization-code flow.
- OAuth proxy/DCR.
- OAuth refresh/token storage.
- Data Center OAuth authorization-code/refresh.
- SSE transport.
- SOCKS proxy.
- Helm chart.
- External registry publishing.

# Support Matrix

This matrix is the current release support reference for the Rust MCP Atlassian server. It is based on the current Rust registry, the Python 73-business-tool baseline, and the real acceptance record.

## Status Terms

| Term | Meaning |
| --- | --- |
| Implemented | Exposed by the Rust MCP server with registry metadata, schema, handler, service filtering, profile filtering, toolset filtering, enabled-tool inclusion, and disabled-tool exclusion. |
| Local/MCP covered | Covered by local mock REST tests, MCP discovery/call tests, smoke tests, or filtering tests. |
| Real accepted | The path was executed against real Atlassian test services. For writes, this only means real write execution when the note explicitly says so. |
| Local only | Implemented and locally validated, but no dedicated real acceptance row was executed. |
| Disabled-tool guard only | Real MCP/disabled-tool mode blocks the destructive write; the destructive operation itself was not executed on a real object. |
| Product blocked | Implemented locally, but real acceptance was blocked by product availability, permission, or interface behavior in the test tenant. |
| Cloud only | Supported for Atlassian Cloud; Server/Data Center returns a structured unavailable response where documented. |

## Tool Count Summary

| Service | Python baseline | Rust business tools | Release status |
| --- | ---: | ---: | --- |
| Jira | 49 | 49 | Implemented |
| Confluence | 24 | 24 | Implemented |
| Total Atlassian business tools | 73 | 73 | Implemented |
| Migration utility | Not part of parity | 0 exposed | No migration utility tool is exposed in the production MCP tool surface. |

## Jira Tools

All Jira rows below are implemented in Rust and are registry-managed business tools.

| Tool | Access | Toolset | Local/MCP status | Real acceptance status |
| --- | --- | --- | --- | --- |
| `jira_get_issue` | read | `jira_issue_read` | Local mock REST and MCP handler covered. | Real accepted for issue read. |
| `jira_search` | read | `jira_issue_read` | Local mock REST and MCP handler covered. | Real accepted for JQL search. |
| `jira_get_project_issues` | read | `jira_issue_read` | Local mock REST and MCP handler covered. | Real accepted for project issue search. |
| `jira_create_issue` | write | `jira_issue_write` | Local mock REST, schema, and disabled-tool guard covered. | Disabled-tool guard only; real create was not executed. |
| `jira_batch_create_issues` | write | `jira_issue_bulk_write` | Local mock REST, schema, and disabled-tool guard covered. | Local only; real batch create was not executed. |
| `jira_batch_get_changelogs` | read | `jira_issue_history_read` | Local mock REST coverage for bulk changelog fetch. | Local only. |
| `jira_update_issue` | write | `jira_issue_write` | Local mock REST, schema, and disabled-tool guard covered. | Local only; real update was not executed. |
| `jira_delete_issue` | write | `jira_issue_delete` | Local mock REST, schema, and disabled-tool guard covered. | Local only; real delete was not executed. |
| `jira_search_fields` | read | `jira_fields_read` | Local mock REST and MCP handler covered. | Real accepted for field search. |
| `jira_get_field_options` | read | `jira_fields_read` | Local mock REST and MCP handler covered. | Real accepted for field option lookup. |
| `jira_add_comment` | write | `jira_comments_write` | Local mock REST, schema, and disabled-tool guard covered. | Local only; no dedicated real comment write row. |
| `jira_edit_comment` | write | `jira_comments_write` | Local mock REST, schema, and disabled-tool guard covered. | Local only; no dedicated real edit row. |
| `jira_get_transitions` | read | `jira_workflow_read` | Local mock REST coverage for transition listing. | Local only. |
| `jira_transition_issue` | write | `jira_workflow_write` | Local mock REST, schema, and disabled-tool guard covered. | Local only; real transition was not executed. |
| `jira_get_all_projects` | read | `jira_project_read` | Local mock REST and MCP handler covered. | Local only. |
| `jira_get_project_versions` | read | `jira_project_metadata_read` | Local mock REST coverage for project versions. | Local only. |
| `jira_get_project_components` | read | `jira_project_metadata_read` | Local mock REST coverage for project components. | Local only. |
| `jira_create_version` | write | `jira_project_write` | Local mock REST, schema, and disabled-tool guard covered. | Local only; real version create was not executed. |
| `jira_batch_create_versions` | write | `jira_project_write` | Local mock REST, schema, and disabled-tool guard covered. | Local only; real batch version create was not executed. |
| `jira_get_user_profile` | read | `jira_users` | Local mock REST coverage for user profile lookup. | Local only. |
| `jira_get_issue_watchers` | read | `jira_watchers` | Local mock REST coverage for watcher listing. | Real accepted for watcher read. |
| `jira_add_watcher` | write | `jira_watchers` | Local mock REST, schema, and disabled-tool guard covered. | Local only; real watcher add was not executed. |
| `jira_remove_watcher` | write | `jira_watchers` | Local mock REST, schema, and disabled-tool guard covered. | Disabled-tool guard only; real watcher removal was not executed. |
| `jira_get_worklog` | read | `jira_worklog` | Local mock REST and smoke coverage. | Local only. |
| `jira_add_worklog` | write | `jira_worklog` | Local mock REST, schema, and disabled-tool guard covered. | Local only; real worklog add was not executed. |
| `jira_get_link_types` | read | `jira_links` | Local mock REST coverage for link type listing. | Local only. |
| `jira_link_to_epic` | write | `jira_links` | Local mock REST, schema, and disabled-tool guard covered. | Local only; real link mutation was not executed. |
| `jira_create_issue_link` | write | `jira_links` | Local mock REST, schema, and disabled-tool guard covered. | Local only; real issue link creation was not executed. |
| `jira_create_remote_issue_link` | write | `jira_links` | Local mock REST, schema, and disabled-tool guard covered. | Local only; real remote link creation was not executed. |
| `jira_remove_issue_link` | write | `jira_links` | Local mock REST, schema, and disabled-tool guard covered. | Local only; real link removal was not executed. |
| `jira_download_attachments` | read | `jira_attachments_read` | Local mock REST coverage for bounded attachment output. | Local only. |
| `jira_get_issue_images` | read | `jira_attachments_read` | Local mock REST coverage for issue image retrieval. | Local only. |
| `jira_get_agile_boards` | read | `jira_agile_read` | Local mock REST and smoke coverage. | Real accepted for Agile board lookup. |
| `jira_get_board_issues` | read | `jira_agile_read` | Local mock REST coverage for board issue listing. | Local only. |
| `jira_get_sprints_from_board` | read | `jira_agile_read` | Local mock REST coverage for sprint listing. | Local only. |
| `jira_get_sprint_issues` | read | `jira_agile_read` | Local mock REST coverage for sprint issue listing. | Local only. |
| `jira_create_sprint` | write | `jira_sprint_manage` | Local mock REST, schema, and disabled-tool guard covered. | Local only; real sprint create was not executed. |
| `jira_update_sprint` | write | `jira_sprint_manage` | Local mock REST, schema, and disabled-tool guard covered. | Local only; real sprint update was not executed. |
| `jira_add_issues_to_sprint` | write | `jira_sprint_planning` | Local mock REST, schema, and disabled-tool guard covered. | Local only; real sprint membership write was not executed. |
| `jira_get_service_desk_for_project` | read | `jira_service_desk` | Local mock REST coverage for Jira Service Management lookup. | Product blocked; service desk lookup returned 403 in the test tenant. |
| `jira_get_service_desk_queues` | read | `jira_service_desk` | Local mock REST coverage for queue listing. | Product blocked by the same JSM permission/product boundary. |
| `jira_get_queue_issues` | read | `jira_service_desk` | Local mock REST coverage for queue issue listing. | Product blocked by the same JSM permission/product boundary. |
| `jira_get_issue_proforma_forms` | read | `jira_forms` | Local mock REST coverage for Forms/ProForma listing. | Product blocked; real acceptance did not receive an effective Forms API response. |
| `jira_get_proforma_form_details` | read | `jira_forms` | Local mock REST coverage for form details. | Product blocked; no valid real form ID/interface was available. |
| `jira_update_proforma_form_answers` | write | `jira_forms` | Local mock REST, schema, and disabled-tool guard covered. | Product blocked; real answer update was not executed. |
| `jira_get_issue_dates` | read | `jira_metrics_read` | Local mock REST coverage for issue date/status timing. | Local only. |
| `jira_get_issue_sla` | read | `jira_metrics_read` | Local mock REST coverage for SLA extraction and parsing limitation output; no local working-hours filtering. | Real accepted for issue SLA read. |
| `jira_get_issue_development_info` | read | `jira_development_read` | Local mock REST coverage for single issue development info. | Real accepted for single development-info read. |
| `jira_get_issues_development_info` | read | `jira_development_read` | Local mock REST coverage for batch development info. | Real accepted for batch development-info read. |

## Confluence Tools

All Confluence rows below are implemented in Rust and are registry-managed business tools.

| Tool | Access | Toolset | Local/MCP status | Real acceptance status |
| --- | --- | --- | --- | --- |
| `confluence_search` | read | `confluence_content_read` | Local mock REST and MCP smoke covered. | Real accepted for search. |
| `confluence_get_page` | read | `confluence_content_read` | Local mock REST and MCP smoke covered. | Real accepted for page read. |
| `confluence_get_page_children` | read | `confluence_content_read` | Local mock REST coverage for combined page/folder limit handling and query statistics. | Real accepted for children listing. |
| `confluence_get_space_page_tree` | read | `confluence_content_read` | Local mock REST coverage for page tree. | Real accepted for space tree. |
| `confluence_create_page` | write | `confluence_content_write` | Local mock REST, schema, disabled-tool guard, and `emoji_status` success output covered. | Real accepted on a test object. |
| `confluence_update_page` | write | `confluence_content_write` | Local mock REST, schema, disabled-tool guard, and `emoji_status` success/failure output covered. | Real accepted on a test object. |
| `confluence_delete_page` | write | `confluence_content_delete` | Local mock REST, schema, and disabled-tool guard covered. | Disabled-tool guard only; real delete was not executed. |
| `confluence_move_page` | write | `confluence_content_delete` | Local mock REST, schema, and disabled-tool guard covered. | Disabled-tool guard only; real move was not executed. |
| `confluence_get_comments` | read | `confluence_comments_read` | Local mock REST coverage for page comments. | Real accepted for comment read. |
| `confluence_add_comment` | write | `confluence_comments_write` | Local mock REST, schema, and disabled-tool guard covered. | Real accepted on a test object. |
| `confluence_reply_to_comment` | write | `confluence_comments_write` | Local mock REST, schema, and disabled-tool guard covered. | Real accepted on a test comment. |
| `confluence_get_labels` | read | `confluence_labels_read` | Local mock REST coverage for labels. | Real accepted for label read. |
| `confluence_add_label` | write | `confluence_labels_write` | Local mock REST, schema, and disabled-tool guard covered. | Real accepted on a test object. |
| `confluence_search_user` | read | `confluence_users_read` | Local mock REST coverage for Cloud CQL and Server/Data Center fallback behavior. | Local only; real acceptance did not execute a dedicated user-search row. |
| `confluence_get_page_history` | read | `confluence_versions_read` | Local mock REST coverage for page history. | Local only. |
| `confluence_get_page_diff` | read | `confluence_versions_read` | Local mock REST coverage for page version diff. | Local only. |
| `confluence_get_page_views` | read | `confluence_analytics_read` | Local mock REST coverage for Cloud success and Server/Data Center unavailable behavior. | Real accepted for Cloud page views; Server/Data Center is unavailable by design. |
| `confluence_upload_attachment` | write | `confluence_attachments_write` | Local mock REST, schema, disabled-tool guard, and 10 MiB pre-read size limit covered. | Real accepted on a test object. |
| `confluence_upload_attachments` | write | `confluence_attachments_write` | Local mock REST coverage for batch partial success/failure summaries, including oversized file failures. | Real accepted for batch upload. |
| `confluence_get_attachments` | read | `confluence_attachments_read` | Local mock REST coverage for attachment listing. | Real accepted for attachment listing. |
| `confluence_download_attachment` | read | `confluence_attachments_read` | Local mock REST coverage for same-origin bounded attachment download. | Real accepted for single attachment download. |
| `confluence_download_content_attachments` | read | `confluence_attachments_read` | Local mock REST coverage for paginated bounded multi-attachment download, partial failures, and page protection summaries. | Real accepted for content attachment download. |
| `confluence_delete_attachment` | write | `confluence_attachments_delete` | Local mock REST, schema, and disabled-tool guard covered. | Disabled-tool guard only; real attachment delete was not executed. |
| `confluence_get_page_images` | read | `confluence_attachments_read` | Local mock REST coverage for page image extraction. | Real accepted for page images. |

## Acceptance Boundaries

| Area | Current release status | Unblock condition for stronger claim |
| --- | --- | --- |
| Jira Service Management | Product/permission blocked. Service desk lookup returned 403 in the test tenant, so `jira_service_desk` is not real-accepted. | Run acceptance in a tenant/project where the test identity can resolve a service desk and queue, then record real service desk, queue, and queue issue reads. |
| Jira Forms/ProForma | Product/interface blocked. Real acceptance did not receive an effective Forms API response and did not have a valid real form ID for details/update. | Run acceptance in a tenant with Jira Forms/ProForma enabled, a test issue with a form, and a safe test answer update target. |
| Confluence user search | Local only for `confluence_search_user`. Real acceptance did not execute a dedicated user-search row. | Add and run a dedicated real Confluence user-search acceptance row with a non-sensitive test query and expected account result shape. |
| Confluence destructive writes | Disabled-tool guard only for `confluence_delete_page`, `confluence_move_page`, and `confluence_delete_attachment`. Real acceptance did not delete, move, or delete an attachment on a real object. | Run destructive acceptance only against disposable test pages/attachments with cleanup and explicit object isolation. |

## Runtime Configuration

| Capability | Env or CLI surface | Rust status | Notes |
| --- | --- | --- | --- |
| Tool profile | `TOOL_PROFILE` | Supported | Supports `basic`, `developer`, `manager`, `full`, and `custom`; defaults to `basic`. |
| Toolset filtering | `TOOLSETS` | Supported | Adds comma-separated registered toolsets to the selected profile. `all` enables every toolset. |
| Exact tool inclusion | `ENABLED_TOOLS` | Supported | Comma-separated MCP tool names to add exactly. |
| Exact tool exclusion | `DISABLED_TOOLS` | Supported | Comma-separated MCP tool names to remove exactly. Takes precedence over profile/toolset inclusion. |
| Streamable HTTP binding | `MCP_HTTP_HOST`, `MCP_HTTP_PORT`, `MCP_HTTP_PATH`, `streamhttp --host/--port/--path` | Supported | Default MCP path is `/mcp`; missing leading slash is normalized. |
| Health endpoint | `GET /healthz` | Supported | Available for streamable HTTP deployments and compose healthchecks. |
| Request-scoped auth bypass | `IGNORE_HEADER_AUTH` | Supported | Truthy values ignore request-scoped auth/service headers and use only global environment config. |
| Header URL allowlist | `MCP_ALLOWED_URL_DOMAINS` | Supported | Restricts header-provided Jira/Confluence service URLs to exact domains or subdomains. |
| Global Cloud ID | `ATLASSIAN_OAUTH_CLOUD_ID` | Supported | Used by Cloud BYOT auth and request-scoped Bearer disambiguation. |
| Bearer BYOT switch | `ATLASSIAN_OAUTH_ENABLE` | Supported | Truthy values interpret streamable HTTP `Authorization: Bearer` as BYOT/OAuth access-token auth. |

## Authentication

| Capability | Env or request surface | Rust status | Notes |
| --- | --- | --- | --- |
| Jira Cloud basic/API token | `JIRA_URL`, `JIRA_USERNAME`, `JIRA_API_TOKEN` | Supported | Used for Jira Cloud URLs ending in `.atlassian.net`. |
| Jira Server/Data Center PAT | `JIRA_URL`, `JIRA_PERSONAL_TOKEN` | Supported | Used for non-Cloud Jira URLs. |
| Confluence Cloud basic/API token | `CONFLUENCE_URL`, `CONFLUENCE_USERNAME`, `CONFLUENCE_API_TOKEN` | Supported | Used for Confluence Cloud URLs ending in `.atlassian.net`. |
| Confluence Server/Data Center PAT | `CONFLUENCE_URL`, `CONFLUENCE_PERSONAL_TOKEN` | Supported | Used for non-Cloud Confluence URLs. |
| Shared BYOT access token | `ATLASSIAN_OAUTH_ACCESS_TOKEN` | Supported | Fallback access token for Jira and Confluence. Cloud mode requires `ATLASSIAN_OAUTH_CLOUD_ID`. |
| Jira-specific BYOT access token | `JIRA_OAUTH_ACCESS_TOKEN` | Supported | Takes precedence over the shared access token for Jira. Jira Cloud uses `https://api.atlassian.com/ex/jira/{cloud_id}`. |
| Confluence-specific BYOT access token | `CONFLUENCE_OAUTH_ACCESS_TOKEN` | Supported | Takes precedence over the shared access token for Confluence. Confluence Cloud uses `https://api.atlassian.com/ex/confluence/{cloud_id}/wiki`. |
| Server/Data Center BYOT fallback | BYOT env token without service PAT | Supported | Server/Data Center keeps the configured service base URL when PAT is absent. |
| Request-scoped Basic auth | `Authorization: Basic <base64(email:api_token)>` | Supported | Streamable HTTP only; scoped to the request/session. |
| Request-scoped Token auth | `Authorization: Token <pat>` | Supported | Streamable HTTP only; PAT-compatible token auth. |
| Request-scoped Bearer PAT-compatible auth | `Authorization: Bearer <token>` | Supported | Used when no BYOT signal is present. |
| Request-scoped Bearer BYOT auth | `Authorization: Bearer <token>` plus `X-Atlassian-Cloud-Id` or `ATLASSIAN_OAUTH_ENABLE=true` | Supported | Uses BYOT/OAuth access-token semantics and token-type-aware session fingerprinting. |
| Request-scoped service URL/PAT headers | `X-Atlassian-Jira-Url`, `X-Atlassian-Jira-Personal-Token`, `X-Atlassian-Confluence-Url`, `X-Atlassian-Confluence-Personal-Token` | Supported | Header-provided URLs are validated by the SSRF boundary. |
| Request-scoped Cloud ID | `X-Atlassian-Cloud-Id` | Supported | Also disambiguates Bearer as BYOT access-token auth. |
| OAuth Cloud 3LO authorization-code flow | OAuth app flow | Not supported in the current Rust release | Fixed backlog item; not implemented in the current release. |
| OAuth proxy/DCR | Dynamic client registration/proxy flow | Not supported in the current Rust release | Fixed backlog item; not implemented in the current release. |
| OAuth refresh/token storage | Refresh tokens and token persistence | Not supported in the current Rust release | Fixed backlog item; BYOT access tokens are accepted but not refreshed or stored. |
| Data Center OAuth authorization-code/refresh | Data Center OAuth app flow | Not supported in the current Rust release | Fixed backlog item; PAT and BYOT fallback are supported instead. |

## Transport

| Capability | Surface | Rust status | Notes |
| --- | --- | --- | --- |
| stdio transport | `mcp-atlassian-rs stdio` | Supported | Logs go to stderr; stdout remains MCP protocol-only. |
| Streamable HTTP transport | `mcp-atlassian-rs streamhttp` | Supported | Default endpoint is `/mcp`; path is configurable. |
| Streamable HTTP request auth | HTTP headers listed above | Supported | Request-scoped auth applies only to the current request/session and preserves service/tool filtering. |
| Health check | `GET /healthz` | Supported | Returns a simple status response for HTTP deployments. |
| SSE transport | SSE server mode | Not supported in the current Rust release | Fixed unsupported/backlog item; supported transports are stdio and streamable HTTP. |

## Network, TLS, And Security

| Capability | Env or behavior | Rust status | Notes |
| --- | --- | --- | --- |
| Jira HTTP/HTTPS proxy | `JIRA_HTTP_PROXY`, `JIRA_HTTPS_PROXY`, `JIRA_NO_PROXY` | Supported | Service-specific values take precedence over global proxy fallback. |
| Confluence HTTP/HTTPS proxy | `CONFLUENCE_HTTP_PROXY`, `CONFLUENCE_HTTPS_PROXY`, `CONFLUENCE_NO_PROXY` | Supported | Service-specific values take precedence over global proxy fallback. |
| Global HTTP/HTTPS proxy fallback | `HTTP_PROXY`, `HTTPS_PROXY`, `NO_PROXY` | Supported | Used when service-specific proxy variables are unset. |
| Jira custom outbound headers | `JIRA_CUSTOM_HEADERS` | Supported | Validated comma-separated `Name=value` pairs. Reserved auth, cookie, proxy, host, content, connection, and request-scoped Atlassian headers are rejected. |
| Confluence custom outbound headers | `CONFLUENCE_CUSTOM_HEADERS` | Supported | Same validation policy as Jira custom headers. |
| Jira mTLS client cert/key | `JIRA_CLIENT_CERT`, `JIRA_CLIENT_KEY` | Supported | PEM certificate and key paths; both must be set together. |
| Confluence mTLS client cert/key | `CONFLUENCE_CLIENT_CERT`, `CONFLUENCE_CLIENT_KEY` | Supported | PEM certificate and key paths; both must be set together. |
| Jira TLS verification toggle | `JIRA_SSL_VERIFY` | Supported | `false`, `0`, `no`, and `off` disable verification. |
| Confluence TLS verification toggle | `CONFLUENCE_SSL_VERIFY` | Supported | `false`, `0`, `no`, and `off` disable verification. |
| SSRF protection for header URLs | URL validation and optional domain allowlist | Supported | Validates scheme, hostname, blocked hostnames, non-global IP literals, DNS results, and `MCP_ALLOWED_URL_DOMAINS`. |
| Outbound redirect policy | Atlassian HTTP client behavior | Supported | Redirects are same-origin only and limited to 3 hops. |
| Session auth fingerprint | `Mcp-Session-Id` with auth fingerprint | Supported | Changing auth or token type within a streamable HTTP MCP session is rejected. |
| Redaction | Logs, debug output, compact errors, env/config/error text | Supported | Redacts auth fragments, access tokens, proxy credentials, custom header values, sensitive headers, query values, and Atlassian error summaries. |

## Release And Deployment

| Capability | Rust status | Notes |
| --- | --- | --- |
| Linux release binary artifact | Supported | Tag-driven workflow builds `mcp-atlassian-rs-linux-x86_64.tar.gz` and checksum. |
| Docker image build | Supported | Local and CI builds use `Dockerfile`; external image publishing is not part of the current Rust release. |
| Docker Compose | Supported | `docker-compose.yml` includes streamable HTTP startup and `/healthz` healthcheck. |
| Helm chart | Not supported in the current Rust release | The Python reference has a Helm chart. Rust Helm requires a dedicated future task and remains in `docs/backlog.md`. |
| External registry publishing | Not supported in the current Rust release | No crates.io, GHCR, Docker Hub, or other external registry publishing is performed. |

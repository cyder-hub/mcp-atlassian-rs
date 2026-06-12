# Support Matrix

This matrix is the current release support reference for the Rust MCP server. It is based on the current Rust registry, local mock coverage, and the real Jira/Confluence acceptance record.

## Status Terms

| Term | Meaning |
| --- | --- |
| Implemented | Exposed by the Rust MCP server with registry metadata, schema, handler, service filtering, profile filtering, toolset filtering, enabled-tool inclusion, and disabled-tool exclusion. |
| Local/MCP/CLI covered | Covered by local mock REST tests, MCP discovery/call tests, production CLI tests, smoke tests, or filtering tests. |
| Real accepted | The path was executed against real upstream test services. For writes, this only means real write execution when the note explicitly says so. |
| Local only | Implemented and locally validated, but no dedicated real acceptance row was executed. |
| Disabled-tool guard only | Real MCP/disabled-tool mode blocks the destructive write; the destructive operation itself was not executed on a real object. |
| Product blocked | Implemented locally, but real acceptance was blocked by product availability, permission, or interface behavior in the test tenant. |
| Cloud only | Supported for Atlassian Cloud; Server/Data Center returns a structured unavailable response where documented. |

## Tool Count Summary

| Service | Expected business tools | Rust business tools | Release status |
| --- | ---: | ---: | --- |
| Jira | 46 | 46 | Implemented |
| Confluence | 24 | 24 | Implemented |
| GitLab | 15 | 15 | Implemented |
| Total business tools | 85 | 85 | Implemented |
| Production CLI commands | 85 | 85 | Resource-oriented CLI commands call the same shared operation layer as MCP handlers. |
| Utility tools | 0 | 0 exposed | No utility tool is exposed in the production MCP or CLI surface. |

## Jira Tools

All Jira rows below are implemented in Rust and are registry-managed business tools.

| Tool | Access | Toolset | Local/MCP status | Real acceptance status |
| --- | --- | --- | --- | --- |
| `jira_get_issue` | read | `jira_issues_read` | Local mock REST and MCP handler covered. | Real accepted for issue read. |
| `jira_search_issues` | read | `jira_issues_read` | Local mock REST and MCP handler covered. | Real accepted for JQL search. |
| `jira_list_project_issues` | read | `jira_issues_read` | Local mock REST and MCP handler covered. | Real accepted for project issue search. |
| `jira_create_issue` | write | `jira_issues_write` | Local mock REST, schema, and disabled-tool guard covered. | Disabled-tool guard only; real create was not executed. |
| `jira_create_issues` | write | `jira_issues_bulk_write` | Local mock REST, schema, and disabled-tool guard covered. | Local only; real batch create was not executed. |
| `jira_get_issue_changelogs` | read | `jira_issues_history_read` | Local mock REST coverage for bulk changelog fetch. | Local only. |
| `jira_update_issue` | write | `jira_issues_update` | Local mock REST, schema, and disabled-tool guard covered. | Local only; real update was not executed. |
| `jira_delete_issue` | write | `jira_issues_delete` | Local mock REST, schema, and disabled-tool guard covered. | Local only; real delete was not executed. |
| `jira_search_fields` | read | `jira_fields_read` | Local mock REST and MCP handler covered. | Real accepted for field search. |
| `jira_list_field_options` | read | `jira_fields_read` | Local mock REST and MCP handler covered. | Real accepted for field option lookup. |
| `jira_add_issue_comment` | write | `jira_issue_comments_write` | Local mock REST, schema, and disabled-tool guard covered. | Local only; no dedicated real comment write row. |
| `jira_update_issue_comment` | write | `jira_issue_comments_update` | Local mock REST, schema, and disabled-tool guard covered. | Local only; no dedicated real edit row. |
| `jira_list_issue_transitions` | read | `jira_issue_workflows_read` | Local mock REST coverage for transition listing. | Local only. |
| `jira_transition_issue` | write | `jira_issue_workflows_write` | Local mock REST, schema, and disabled-tool guard covered. | Local only; real transition was not executed. |
| `jira_list_projects` | read | `jira_projects_read` | Local mock REST and MCP handler covered. | Local only. |
| `jira_list_project_versions` | read | `jira_projects_metadata_read` | Local mock REST coverage for project versions. | Local only. |
| `jira_list_project_components` | read | `jira_projects_metadata_read` | Local mock REST coverage for project components. | Local only. |
| `jira_create_project_version` | write | `jira_project_versions_write` | Local mock REST, schema, and disabled-tool guard covered. | Local only; real version create was not executed. |
| `jira_create_project_versions` | write | `jira_project_versions_write` | Local mock REST, schema, and disabled-tool guard covered. | Local only; real batch version create was not executed. |
| `jira_get_user` | read | `jira_users_read` | Local mock REST coverage for user profile lookup. | Local only. |
| `jira_list_issue_watchers` | read | `jira_issue_watchers_read` | Local mock REST coverage for watcher listing. | Real accepted for watcher read. |
| `jira_add_issue_watcher` | write | `jira_issue_watchers_write` | Local mock REST, schema, and disabled-tool guard covered. | Local only; real watcher add was not executed. |
| `jira_remove_issue_watcher` | write | `jira_issue_watchers_delete` | Local mock REST, schema, and disabled-tool guard covered. | Disabled-tool guard only; real watcher removal was not executed. |
| `jira_list_issue_worklogs` | read | `jira_issue_worklogs_read` | Local mock REST and smoke coverage. | Local only. |
| `jira_add_issue_worklog` | write | `jira_issue_worklogs_write` | Local mock REST, schema, and disabled-tool guard covered. | Local only; real worklog add was not executed. |
| `jira_list_issue_link_types` | read | `jira_issue_links_read` | Local mock REST coverage for link type listing. | Local only. |
| `jira_set_issue_parent` | write | `jira_issue_links_write` | Local mock REST, schema, and disabled-tool guard covered. | Local only; real link mutation was not executed. |
| `jira_create_issue_link` | write | `jira_issue_links_write` | Local mock REST, schema, and disabled-tool guard covered. | Local only; real issue link creation was not executed. |
| `jira_create_remote_issue_link` | write | `jira_issue_links_write` | Local mock REST, schema, and disabled-tool guard covered. | Local only; real remote link creation was not executed. |
| `jira_delete_issue_link` | write | `jira_issue_links_delete` | Local mock REST, schema, and disabled-tool guard covered. | Local only; real link removal was not executed. |
| `jira_get_issue_attachments` | read | `jira_issue_attachments_read` | Local mock REST coverage for bounded attachment output. | Local only. |
| `jira_get_issue_image_attachments` | read | `jira_issue_attachments_read` | Local mock REST coverage for issue image retrieval. | Local only. |
| `jira_list_agile_boards` | read | `jira_agile_boards_read` | Local mock REST and smoke coverage. | Real accepted for Agile board lookup. |
| `jira_list_board_issues` | read | `jira_agile_boards_read` | Local mock REST coverage for board issue listing. | Local only. |
| `jira_list_board_sprints` | read | `jira_sprints_read` | Local mock REST coverage for sprint listing. | Local only. |
| `jira_list_sprint_issues` | read | `jira_sprints_read` | Local mock REST coverage for sprint issue listing. | Local only. |
| `jira_create_sprint` | write | `jira_sprints_write` | Local mock REST, schema, and disabled-tool guard covered. | Local only; real sprint create was not executed. |
| `jira_update_sprint` | write | `jira_sprints_write` | Local mock REST, schema, and disabled-tool guard covered. | Local only; real sprint update was not executed. |
| `jira_add_issues_to_sprint` | write | `jira_sprint_membership_write` | Local mock REST, schema, and disabled-tool guard covered. | Local only; real sprint membership write was not executed. |
| `jira_get_project_service_desk` | read | `jira_service_desks_read` | Local mock REST coverage for Jira Service Management lookup. | Product blocked; service desk lookup returned 403 in the test tenant. |
| `jira_list_service_desk_queues` | read | `jira_service_desks_read` | Local mock REST coverage for queue listing. | Product blocked by the same JSM permission/product boundary. |
| `jira_list_service_desk_queue_issues` | read | `jira_service_desks_read` | Local mock REST coverage for queue issue listing. | Product blocked by the same JSM permission/product boundary. |
| `jira_get_issue_timeline` | read | `jira_issue_metrics_read` | Local mock REST coverage for issue date/status timing. | Local only. |
| `jira_get_issue_sla_metrics` | read | `jira_issue_sla_read` | Local mock REST coverage for SLA extraction and parsing limitation output; no local working-hours filtering. | Real accepted for issue SLA read. |
| `jira_get_issue_development` | read | `jira_issue_development_read` | Local mock REST coverage for single issue development info. | Real accepted for single development-info read. |
| `jira_get_issues_development` | read | `jira_issue_development_read` | Local mock REST coverage for batch development info. | Real accepted for batch development-info read. |

## Confluence Tools

All Confluence rows below are implemented in Rust and are registry-managed business tools.

| Tool | Access | Toolset | Local/MCP status | Real acceptance status |
| --- | --- | --- | --- | --- |
| `confluence_search_content` | read | `confluence_content_read` | Local mock REST and MCP smoke covered. | Real accepted for search. |
| `confluence_get_page` | read | `confluence_content_read` | Local mock REST and MCP smoke covered. | Real accepted for page read. |
| `confluence_list_page_children` | read | `confluence_content_read` | Local mock REST coverage for combined page/folder limit handling and query statistics. | Real accepted for children listing. |
| `confluence_get_space_page_tree` | read | `confluence_content_read` | Local mock REST coverage for page tree. | Real accepted for space tree. |
| `confluence_create_page` | write | `confluence_content_write` | Local mock REST, schema, disabled-tool guard, and `emoji_status` success output covered. | Real accepted on a test object. |
| `confluence_update_page` | write | `confluence_content_update` | Local mock REST, schema, disabled-tool guard, and `emoji_status` success/failure output covered. | Real accepted on a test object. |
| `confluence_delete_page` | write | `confluence_content_delete` | Local mock REST, schema, and disabled-tool guard covered. | Disabled-tool guard only; real delete was not executed. |
| `confluence_move_page` | write | `confluence_content_update` | Local mock REST, schema, and disabled-tool guard covered. | Disabled-tool guard only; real move was not executed. |
| `confluence_list_page_comments` | read | `confluence_page_comments_read` | Local mock REST coverage for page comments. | Real accepted for comment read. |
| `confluence_add_page_comment` | write | `confluence_page_comments_write` | Local mock REST, schema, and disabled-tool guard covered. | Real accepted on a test object. |
| `confluence_reply_to_comment` | write | `confluence_page_comments_write` | Local mock REST, schema, and disabled-tool guard covered. | Real accepted on a test comment. |
| `confluence_list_content_labels` | read | `confluence_content_labels_read` | Local mock REST coverage for labels. | Real accepted for label read. |
| `confluence_add_content_label` | write | `confluence_content_labels_write` | Local mock REST, schema, and disabled-tool guard covered. | Real accepted on a test object. |
| `confluence_search_users` | read | `confluence_users_read` | Local mock REST coverage for Cloud CQL and Server/Data Center fallback behavior. | Local only; real acceptance did not execute a dedicated user-search row. |
| `confluence_get_page_version` | read | `confluence_page_versions_read` | Local mock REST coverage for page history. | Local only. |
| `confluence_get_page_diff` | read | `confluence_page_versions_read` | Local mock REST coverage for page version diff. | Local only. |
| `confluence_get_page_view_analytics` | read | `confluence_page_analytics_read` | Local mock REST coverage for Cloud success and Server/Data Center unavailable behavior. | Real accepted for Cloud page views; Server/Data Center is unavailable by design. |
| `confluence_upload_content_attachment` | write | `confluence_attachments_write` | Local mock REST, schema, disabled-tool guard, and 10 MiB pre-read size limit covered. | Real accepted on a test object. |
| `confluence_upload_content_attachments` | write | `confluence_attachments_write` | Local mock REST coverage for batch partial success/failure summaries, including oversized file failures. | Real accepted for batch upload. |
| `confluence_list_content_attachments` | read | `confluence_attachments_read` | Local mock REST coverage for attachment listing. | Real accepted for attachment listing. |
| `confluence_download_attachment` | read | `confluence_attachments_read` | Local mock REST coverage for same-origin bounded attachment download. | Real accepted for single attachment download. |
| `confluence_download_content_attachments` | read | `confluence_attachments_read` | Local mock REST coverage for paginated bounded multi-attachment download, partial failures, and page protection summaries. | Real accepted for content attachment download. |
| `confluence_delete_attachment` | write | `confluence_attachments_delete` | Local mock REST, schema, and disabled-tool guard covered. | Disabled-tool guard only; real attachment delete was not executed. |
| `confluence_get_content_image_attachments` | read | `confluence_attachments_read` | Local mock REST coverage for page image extraction. | Real accepted for page images. |

## GitLab Tools

All GitLab rows below are implemented in Rust and are registry-managed business tools. GitLab support is local/mock validated only; real GitLab acceptance has not been run. The local GitLab smoke covers current user, project, MR list, streamable HTTP `/healthz`, production CLI text/JSON output, MCP restricted create-MR guard behavior, and CLI isolation from MCP tool visibility controls.

| Tool | Access | Toolset | Local/MCP status | Real acceptance status |
| --- | --- | --- | --- | --- |
| `gitlab_get_current_user` | read | `gitlab_projects_read` | Local mock REST and MCP handler covered with `PRIVATE-TOKEN` auth. | Local only; real GitLab acceptance was not run. |
| `gitlab_get_project` | read | `gitlab_projects_read` | Local mock REST, full-path project encoding, project filter, and MCP handler covered. | Local only; real GitLab acceptance was not run. |
| `gitlab_list_merge_requests` | read | `gitlab_merge_requests_read` | Local mock REST coverage for filters, labels, bounded pagination, and MCP handler output. | Local only; real GitLab acceptance was not run. |
| `gitlab_get_merge_request` | read | `gitlab_merge_requests_read` | Local mock REST coverage for MR IID lookup and optional include flags. | Local only; real GitLab acceptance was not run. |
| `gitlab_list_merge_request_commits` | read | `gitlab_merge_requests_read` | Local mock REST coverage for MR commit listing with bounded pagination. | Local only; real GitLab acceptance was not run. |
| `gitlab_list_merge_request_diffs` | read | `gitlab_merge_requests_read` | Local mock REST coverage for paginated bounded diff output and truncation metadata. | Local only; real GitLab acceptance was not run. |
| `gitlab_list_merge_request_pipelines` | read | `gitlab_merge_requests_read` | Local mock REST coverage for MR pipeline listing with bounded pagination. | Local only; real GitLab acceptance was not run. |
| `gitlab_create_merge_request` | write | `gitlab_merge_requests_write` | Local mock REST and MCP handler coverage for create payload fields. | Local only; real GitLab write acceptance was not run. |
| `gitlab_update_merge_request` | write | `gitlab_merge_requests_write` | Local mock REST and MCP handler coverage for mutable MR fields, explicit empty clear values, labels, reviewers, assignees, state, and target branch. | Local only; real GitLab write acceptance was not run. |
| `gitlab_add_merge_request_note` | write | `gitlab_merge_requests_write` | Local mock REST and MCP handler coverage for regular MR notes and empty-body rejection. | Local only; real GitLab write acceptance was not run. |
| `gitlab_reply_merge_request_discussion` | write | `gitlab_merge_requests_write` | Local mock REST and MCP handler coverage for discussion reply payloads and discussion ID encoding. | Local only; real GitLab write acceptance was not run. |
| `gitlab_resolve_merge_request_discussion` | write | `gitlab_merge_requests_write` | Local mock REST and MCP handler coverage for discussion resolved-state updates. | Local only; real GitLab write acceptance was not run. |
| `gitlab_get_merge_request_approval_state` | read | `gitlab_merge_requests_read` | Local mock REST and MCP handler coverage for approval state. | Local only; availability can depend on GitLab tier and token permissions. |
| `gitlab_set_merge_request_approval` | write | `gitlab_merge_requests_write` | Local mock REST and MCP handler coverage for approve and unapprove endpoints. | Local only; availability can depend on GitLab tier and token permissions. |
| `gitlab_accept_merge_request` | write | `gitlab_merge_requests_merge` | Local mock REST and MCP handler coverage for SHA-gated merge payload and 409 upstream error mapping. | Local only; real GitLab merge acceptance was not run. |

## Acceptance Boundaries

| Area | Current release status | Unblock condition for stronger claim |
| --- | --- | --- |
| Jira Service Management | Product/permission blocked. Service desk lookup returned 403 in the test tenant, so `jira_service_desks_read` is not real-accepted. | Run acceptance in a tenant/project where the test identity can resolve a service desk and queue, then record real service desk, queue, and queue issue reads. |
| Confluence user search | Local only for `confluence_search_users`. Real acceptance did not execute a dedicated user-search row. | Add and run a dedicated real Confluence user-search acceptance row with a non-sensitive test query and expected account result shape. |
| Confluence destructive writes | Disabled-tool guard only for `confluence_delete_page`, `confluence_move_page`, and `confluence_delete_attachment`. Real acceptance did not delete, move, or delete an attachment on a real object. | Run destructive acceptance only against disposable test pages/attachments with cleanup and explicit object isolation. |
| GitLab MR tools | Local/mock only. Real GitLab acceptance was not run for reads, writes, approvals, or merge. Approval endpoints can depend on GitLab tier and token permissions. | Add a separate real GitLab acceptance suite with disposable projects/MRs and explicit cleanup before recording real GitLab status. |

## Runtime Configuration

| Capability | Env or CLI surface | Rust status | Notes |
| --- | --- | --- | --- |
| Tool profile | `MCP_TOOL_PROFILE` | Supported | MCP-only. Supports `basic`, `developer`, `manager`, `full`, and `custom`; defaults to `basic`. With Jira, Confluence, and GitLab configured, profiles expose 23, 47, 82, 85, and 0 tools respectively. Service availability filters out tools for unconfigured services. Unknown values fail startup. |
| Toolset filtering | `MCP_TOOLSETS` | Supported | Adds comma-separated registered toolsets to the selected profile. `all` enables every toolset; unknown names fail startup. |
| Exact tool inclusion | `MCP_ENABLED_TOOLS` | Supported | Comma-separated MCP tool names to add exactly. |
| Exact tool exclusion | `MCP_DISABLED_TOOLS` | Supported | Comma-separated MCP tool names to remove exactly. Takes precedence over profile/toolset inclusion. |
| Streamable HTTP binding | `MCP_HTTP_HOST`, `MCP_HTTP_PORT`, `MCP_HTTP_PATH`, `streamhttp --host/--port/--path` | Supported | Parsed only for streamable HTTP startup. Default MCP path is `/mcp`; missing leading slash is normalized. |
| Resource CLI | `workhub cli ...` | Supported | Covers all 85 business capabilities as resource-oriented commands for configured services. It ignores `MCP_TOOL_PROFILE`, `MCP_TOOLSETS`, `MCP_ENABLED_TOOLS`, and `MCP_DISABLED_TOOLS`; no raw MCP tool-call, schema, or tools-list fallback is exposed. |
| CLI JSON output | `workhub cli --json ...` and `--pretty` | Supported | Successful JSON goes to stdout; errors go to stderr. `--pretty` requires `--json`. |
| CLI env file loading | `workhub cli --env-file <path>`, `ENV_FILE`, `.env` | Supported | Same dotenv priority as `streamhttp`; `stdio` intentionally does not load dotenv files. |
| Health endpoint | `GET /healthz` | Supported | Available for streamable HTTP deployments and compose healthchecks. |
| GitLab project allowlist | `GITLAB_PROJECTS_FILTER` | Supported | Optional exact allowlist of numeric project IDs or full paths. Project-scoped GitLab tools reject unlisted projects before sending HTTP. |

## Authentication

| Capability | Env or request surface | Rust status | Notes |
| --- | --- | --- | --- |
| Jira Cloud basic/API token | `JIRA_URL`, `JIRA_USERNAME`, `JIRA_API_TOKEN` | Supported | Used for Jira Cloud URLs ending in `.atlassian.net`. |
| Jira Server/Data Center PAT | `JIRA_URL`, `JIRA_PERSONAL_TOKEN` | Supported | Used for non-Cloud Jira URLs. |
| Jira Server/Data Center basic password | `JIRA_URL`, `JIRA_USERNAME`, `JIRA_PASSWORD` | Supported | Used for non-Cloud Jira URLs when PAT is unset. |
| Confluence Cloud basic/API token | `CONFLUENCE_URL`, `CONFLUENCE_USERNAME`, `CONFLUENCE_API_TOKEN` | Supported | Used for Confluence Cloud URLs ending in `.atlassian.net`. |
| Confluence Server/Data Center PAT | `CONFLUENCE_URL`, `CONFLUENCE_PERSONAL_TOKEN` | Supported | Used for non-Cloud Confluence URLs. |
| Confluence Server/Data Center basic password | `CONFLUENCE_URL`, `CONFLUENCE_USERNAME`, `CONFLUENCE_PASSWORD` | Supported | Used for non-Cloud Confluence URLs when PAT is unset. |
| Shared basic/API token fallback | `ATLASSIAN_USERNAME`, `ATLASSIAN_API_TOKEN` | Supported | Used when service-specific username/API-token variables are unset. Service-specific values take precedence. |
| Shared Server/Data Center basic password fallback | `ATLASSIAN_USERNAME`, `ATLASSIAN_PASSWORD` | Supported | Used for non-Cloud Jira and Confluence when service-specific username/password variables and PAT are unset. |
| Shared Server/Data Center PAT fallback | `ATLASSIAN_PERSONAL_TOKEN` | Supported | Used when service-specific PAT variables are unset. Service-specific values take precedence. |
| GitLab token header auth | `GITLAB_URL`, `GITLAB_TOKEN` or `GITLAB_PERSONAL_TOKEN` | Supported | Token is sent as `PRIVATE-TOKEN`. Use `read_api` for read-only tools and `api` for writes, approvals, and merge. |
| GitLab username/password auth | `GITLAB_USERNAME`, `GITLAB_PASSWORD` | Not supported in the current Rust release | GitLab API auth is token-only in this implementation. |
| OAuth Cloud 3LO authorization-code flow | OAuth app flow | Not supported in the current Rust release | Fixed backlog item; not implemented in the current release. |
| OAuth proxy/DCR | Dynamic client registration/proxy flow | Not supported in the current Rust release | Fixed backlog item; not implemented in the current release. |
| OAuth refresh/token storage | Refresh tokens and token persistence | Not supported in the current Rust release | Fixed backlog item; not implemented in the current release. |
| Data Center OAuth authorization-code/refresh | Data Center OAuth app flow | Not supported in the current Rust release | Fixed backlog item; PAT and username/password Basic Auth are supported instead. |

## Transport

| Capability | Surface | Rust status | Notes |
| --- | --- | --- | --- |
| stdio transport | `workhub stdio` | Supported | Logs go to stderr; stdout remains MCP protocol-only. |
| Streamable HTTP transport | `workhub streamhttp` | Supported | Default endpoint is `/mcp`; path is configurable. |
| Production CLI | `workhub cli` | Supported | One-shot command surface for Jira, Confluence, and GitLab using the shared operation layer. |
| Health check | `GET /healthz` | Supported | Returns a simple status response for HTTP deployments. |
| SSE transport | SSE server mode | Not supported in the current Rust release | Fixed unsupported/backlog item; supported transports are stdio and streamable HTTP. |

## Network, TLS, And Security

| Capability | Env or behavior | Rust status | Notes |
| --- | --- | --- | --- |
| Jira HTTP/HTTPS proxy | `JIRA_HTTP_PROXY`, `JIRA_HTTPS_PROXY`, `JIRA_NO_PROXY` | Supported | Service-specific values take precedence over global proxy fallback. |
| Confluence HTTP/HTTPS proxy | `CONFLUENCE_HTTP_PROXY`, `CONFLUENCE_HTTPS_PROXY`, `CONFLUENCE_NO_PROXY` | Supported | Service-specific values take precedence over global proxy fallback. |
| GitLab HTTP/HTTPS proxy | `GITLAB_HTTP_PROXY`, `GITLAB_HTTPS_PROXY`, `GITLAB_NO_PROXY` | Supported | Service-specific values take precedence over standard `HTTP_PROXY`, `HTTPS_PROXY`, and `NO_PROXY`; `ATLASSIAN_*` proxy fallback does not apply. |
| Shared Atlassian proxy fallback | `ATLASSIAN_HTTP_PROXY`, `ATLASSIAN_HTTPS_PROXY`, `ATLASSIAN_NO_PROXY` | Supported | Used when service-specific proxy variables are unset; takes precedence over standard proxy variables. |
| Global HTTP/HTTPS proxy fallback | `HTTP_PROXY`, `HTTPS_PROXY`, `NO_PROXY` | Supported | Used when service-specific proxy variables and any applicable shared fallback are unset. |
| Jira custom outbound headers | `JIRA_CUSTOM_HEADERS` | Supported | Validated comma-separated `Name=value` pairs. Reserved auth, cookie, proxy, host, content, and connection headers are rejected. |
| Confluence custom outbound headers | `CONFLUENCE_CUSTOM_HEADERS` | Supported | Same validation policy as Jira custom headers. |
| GitLab custom outbound headers | `GITLAB_CUSTOM_HEADERS` | Supported | Same validation policy as Jira/Confluence custom headers, plus GitLab auth headers such as `Private-Token` and `Job-Token` are reserved. Does not use `ATLASSIAN_CUSTOM_HEADERS`. |
| Shared custom outbound headers fallback | `ATLASSIAN_CUSTOM_HEADERS` | Supported | Used when service-specific custom headers are unset. |
| Jira mTLS client cert/key | `JIRA_CLIENT_CERT`, `JIRA_CLIENT_KEY` | Supported | PEM certificate and key paths; both must be set together. |
| Confluence mTLS client cert/key | `CONFLUENCE_CLIENT_CERT`, `CONFLUENCE_CLIENT_KEY` | Supported | PEM certificate and key paths; both must be set together. |
| GitLab mTLS client cert/key | `GITLAB_CLIENT_CERT`, `GITLAB_CLIENT_KEY` | Supported | PEM certificate and key paths; both must be set together. Does not use shared `ATLASSIAN_*` mTLS fallback. |
| Shared mTLS client cert/key fallback | `ATLASSIAN_CLIENT_CERT`, `ATLASSIAN_CLIENT_KEY` | Supported | Used when service-specific mTLS variables are unset; both must be set together. |
| Jira TLS verification toggle | `JIRA_SSL_VERIFY` | Supported | `false`, `0`, `no`, and `off` disable verification. Service-specific value takes precedence over `ATLASSIAN_SSL_VERIFY`. |
| Confluence TLS verification toggle | `CONFLUENCE_SSL_VERIFY` | Supported | `false`, `0`, `no`, and `off` disable verification. Service-specific value takes precedence over `ATLASSIAN_SSL_VERIFY`. |
| GitLab TLS verification toggle | `GITLAB_SSL_VERIFY` | Supported | `false`, `0`, `no`, and `off` disable verification. Does not use shared `ATLASSIAN_SSL_VERIFY`. |
| Shared TLS verification fallback | `ATLASSIAN_SSL_VERIFY` | Supported | Shared fallback for Jira and Confluence TLS verification. |
| Service timeout controls | `JIRA_TIMEOUT`, `CONFLUENCE_TIMEOUT`, `ATLASSIAN_TIMEOUT`, `GITLAB_TIMEOUT` | Supported | Positive integer seconds. Jira/Confluence can use shared `ATLASSIAN_TIMEOUT`; GitLab uses only `GITLAB_TIMEOUT` or its 75 second default. |
| Outbound redirect policy | Upstream HTTP client behavior | Supported | Redirects are same-origin only and limited to 3 hops. |
| Redaction | Logs, debug output, compact errors, env/config/error text | Supported | Redacts auth fragments, access tokens, proxy credentials, custom header values, sensitive headers including GitLab `Private-Token`/`Job-Token`, query values, and upstream error summaries. |

## Release And Deployment

| Capability | Rust status | Notes |
| --- | --- | --- |
| Multi-platform release binary artifacts | Supported | Tag-driven workflow builds Linux, macOS, and Windows `workhub-*` binaries and checksums. |
| Docker image build | Supported | Local and CI builds use `Dockerfile`; external image publishing is not part of the current Rust release. |
| Docker Compose | Supported | `docker-compose.yml` includes streamable HTTP startup and `/healthz` healthcheck. |
| Helm chart | Not supported in the current Rust release | Rust Helm packaging requires a dedicated future task and remains in `docs/backlog.md`. |
| External registry publishing | Not supported in the current Rust release | No crates.io, GHCR, Docker Hub, or other external registry publishing is performed. |

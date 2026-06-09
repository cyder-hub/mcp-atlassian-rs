# Changelog

## 0.1.2 - 2026-06-08

### Added
- feat: Added `MCP_TOOL_CALL_DEBUG` to enable MCP tool-call diagnostics when `RUST_LOG` is unset, including tool names, elapsed time, failures, and redacted arguments.
- docs: Added repository contributor guidelines in `AGENTS.md`.

### Fixed
- fix(jira): Fixed Server/Data Center search expand array payload handling.
- fix(jira): Resolved `currentUser()` and `me` Jira user profile lookups.

## 0.1.1 - 2026-06-05

### Added
- feat: Support loading environment variables from an `.env` file via the `--env-file` argument or `ENV_FILE` environment variable when running in `streamhttp` mode.

### Fixed
- fix: Resolved Jira Cloud "unbounded JQL" rejection errors by automatically injecting `issuekey IS NOT EMPTY` when queries are unbounded or contain only an `ORDER BY` clause.
- fix: Corrected `ORDER BY` clause positioning when a Jira project filter is applied.
- fix: Prevented MCP schema validation errors (e.g. in Opencode) by automatically wrapping raw JSON array responses into an `items` record (`{ "items": [...] }`) for all tools.

### Chores
- chore(deps): Bumped GitHub Actions `upload-artifact` (v4 to v7) and `download-artifact` (v4 to v8).

## 0.1.0 - 2026-06-05

- Implemented the Rust MCP Atlassian server with 49 Jira business tools and 24 Confluence business tools.
- Added stdio and streamable HTTP transports, `/healthz`, tool filtering, toolset filtering, service availability filtering, and write-tool guards.
- Added Jira and Confluence Cloud Basic/API-token auth, Server/Data Center PAT auth, BYOT access-token support, request-scoped streamable HTTP auth, SSRF and allowed-domain checks, same-origin redirect protection, proxy/custom outbound headers, and mTLS client cert/key support.
- Added local mock REST and MCP smoke coverage, plus real acceptance records for representative Jira and Confluence paths.
- Deferred full OAuth flows, SSE transport, SOCKS proxy, system truststore injection, Helm, external registry publishing, and blocked or local-only product acceptance items to the published backlog.

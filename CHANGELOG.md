# Changelog

## Unreleased

## 0.4.0 - 2026-06-12

### Added
- Added the production `workhub cli` command surface for resource-oriented Jira, Confluence, and GitLab operations.
- Added structured CLI output modes with text output by default, `--json`, `--pretty`, stdout-only success output, stderr-only errors, and documented exit-code categories.
- Added CLI dotenv loading via `--env-file`, `ENV_FILE`, or default `.env` discovery for CLI and streamable HTTP modes.
- Added shared operation dispatch and output rendering layers reused by MCP handlers and the CLI.
- Added CLI documentation, support-matrix coverage, development tooling notes, smoke coverage, and a Workhub skill reference.

### Changed
- Removed legacy multi-user request-auth and session-auth paths; runtime credentials are now sourced from global service configuration only.
- Updated Docker, Compose, CI, release, and smoke tooling to include the new CLI workflows and artifact checks.

### Fixed
- Kept successful env-file-backed CLI commands stdout-only by suppressing dotenv success diagnostics in CLI mode.
- Rendered CLI startup configuration failures through the CLI error contract, including JSON formatting and config exit code `3`.

## 0.3.0 - 2026-06-10

### Added
- Added `mcp-workhub-rs -v` to print only the package version and exit.

### Changed
- Renamed the released package, binary, and runtime identity from `mcp-atlassian-rs` to `mcp-workhub-rs`, including MCP server metadata, Docker image and Compose service names, release artifact names, `just` shortcuts, and smoke/acceptance tooling paths.
- Updated the default tracing target namespace from `mcp_atlassian_rs` to `mcp_workhub_rs`; custom `RUST_LOG` filters should use the new module path.
- Repositioned the project documentation around a provider-neutral Workhub server for Jira, Confluence, and GitLab, with the README focused on quick start, MCP client setup, architecture, Docker, and release usage.
- Moved the detailed configuration reference into `docs/configuration.md`, with deployment and support documentation updated to use the new binary, Docker, Compose, and artifact names.

## 0.2.0 - 2026-06-10

### Added
- feat(gitlab): Added GitLab merge request tool support.
- feat(toolsets): Added profile-based access controls for tool exposure.
- feat(auth): Added Jira and Confluence Server/Data Center username/password Basic Auth via `JIRA_PASSWORD`, `CONFLUENCE_PASSWORD`, and shared `ATLASSIAN_PASSWORD` fallbacks.

### Fixed
- fix(mcp): Corrected advertised output schemas.

### Changed
- refactor: Split Atlassian modules for clearer shared HTTP, auth, security, and service-specific boundaries.
- chore: Migrated validation tooling to `xtask`.

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

- Implemented the Rust MCP server with 49 Jira business tools and 24 Confluence business tools.
- Added stdio and streamable HTTP transports, `/healthz`, tool filtering, toolset filtering, service availability filtering, and write-tool guards.
- Added Jira and Confluence Cloud Basic/API-token auth, Server/Data Center PAT auth, username/password fallback for private instances, same-origin redirect protection, proxy/custom outbound headers, and mTLS client cert/key support.
- Added local mock REST and MCP smoke coverage, plus real acceptance records for representative Jira and Confluence paths.
- Deferred full OAuth flows, SSE transport, SOCKS proxy, system truststore injection, Helm, external registry publishing, and blocked or local-only product acceptance items to the published backlog.

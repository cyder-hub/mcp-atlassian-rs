# Repository Guidelines

## Project Structure & Module Organization

This is a Rust 1.94 / edition 2024 binary crate for `mcp-workhub-rs`. `src/main.rs` owns startup, production CLI parsing, tracing, transports, and `/healthz`. `src/mcp.rs` owns RMCP handlers, tool dispatch, and schema handling. Runtime config/context live in `src/config.rs` and `src/context.rs`.

Jira, Confluence, and GitLab code live under `src/jira/`, `src/confluence/`, and `src/gitlab/`. Atlassian-specific compatibility lives in `src/atlassian/`; provider-agnostic auth, HTTP, proxy, mTLS, custom headers, redaction, SSRF, and redirect logic live in `src/upstream/`. `src/tool_registry.rs` centralizes metadata, discovery, `MCP_TOOL_PROFILE`, `MCP_TOOLSETS`, `MCP_ENABLED_TOOLS`, and `MCP_DISABLED_TOOLS`. Development smoke and acceptance tooling lives in `xtask/`; public docs are in `README.md` and `docs/`.

## Build, Test, and Development Commands

- `just dev`: run the MCP server over stdio.
- `just dev-http`: run streamable HTTP on `127.0.0.1:8000`.
- `cargo fmt --check`: verify Rust formatting.
- `cargo test`: run unit and async tests.
- `just check`: run format, check, and tests.
- `just smoke`: run all local MCP smoke checks.

## Coding Style & Naming Conventions

Use `cargo fmt`; `.editorconfig` sets 4-space Rust/TOML indentation and LF endings. Keep docs in English. Follow existing patterns before adding abstractions. Tool names use snake_case, such as `jira_get_issue`; argument structs use PascalCase, such as `JiraGetIssueArgs`. Keep stdio stdout protocol-only.

## MCP Tool Changes

When adding or changing a tool, update constants and argument structs in `src/jira/tools.rs`, `src/confluence/tools.rs`, or `src/gitlab/tools.rs`, the service client, the handler in `src/mcp/`, and metadata in `src/tool_registry.rs`. Preserve service availability, profile filtering, tool filtering, toolsets, and disabled-tool behavior. Update README or `docs/support-matrix.md` when the public tool surface changes.

## Testing Guidelines

Most tests are inline `#[cfg(test)]` modules; async paths use `#[tokio::test]`. Add focused tests next to changed code. For tool or transport changes, verify metadata, discovery filtering, disabled-tool blocking, mock REST behavior, and the relevant `cargo xtask smoke` or `just smoke` path. Real acceptance commands need real test credentials; run them only when explicitly requested or confirmed ready.

## Commit & Pull Request Guidelines

History uses Conventional Commit-style messages such as `fix(jira): ...`, `docs: ...`, and `chore: ...`. Keep commits focused and scoped when helpful.

Pull requests should include a concise summary, verification commands run, and follow-up work. Do not commit, read aloud, or print real credentials from `.env*`, logs, shell env, or task notes. Update docs or changelog when behavior, support status, commands, deployment, or tools change.

## Security & Configuration Tips

Preserve redaction, same-origin redirect limits, custom-header reserved-name checks, proxy/TLS/mTLS behavior, and global credential handling. Do not claim unsupported features unless implemented and validated.

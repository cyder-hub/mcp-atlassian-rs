# Development Tools

This document covers development-only validation commands and variables for `workhub-rs`. These commands are not production server commands and are intentionally exposed through `just` and `cargo xtask`, not through the `workhub` binary.

Production runtime commands include:

```bash
workhub stdio
workhub streamhttp --host 127.0.0.1 --port 8000 --path /mcp
workhub cli jira issue get ABC-1
```

## Local Shortcuts

`justfile` is the friendly local entry point. It delegates development validation to `cargo xtask`.

```bash
just smoke-stdio       # Jira stdio MCP smoke with a local mock Jira
just smoke-http        # Jira streamable HTTP MCP smoke with a local mock Jira
just smoke-jira        # Jira restricted-profile guard smoke
just smoke-confluence  # Confluence stdio, HTTP, and restricted smoke with a local mock Confluence
just smoke-gitlab      # GitLab stdio, HTTP, and restricted smoke with a local mock GitLab
just smoke-cli         # production CLI smoke against local mock upstreams
just smoke             # all local smoke checks
```

GitLab also has local Rust mock tests for client, handler, and discovery coverage. No real GitLab acceptance shortcut is currently provided:

```bash
cargo test gitlab
cargo test mcp::tests::gitlab_handlers
cargo test mcp::tests::discovery
```

Real acceptance shortcuts build `target/debug/workhub`, run preflight, then run the selected acceptance suite against that binary:

```bash
just acceptance-jira
just acceptance-confluence
just acceptance-mcp
```

Real acceptance must use disposable test objects and development-only credentials. Do not store token values in committed files, logs, task records, or screenshots.

## xtask CLI

Run local mock smoke checks directly:

```bash
cargo xtask smoke
cargo xtask smoke jira all
cargo xtask smoke jira stdio
cargo xtask smoke jira http
cargo xtask smoke jira restricted
cargo xtask smoke confluence all
cargo xtask smoke confluence stdio
cargo xtask smoke confluence http
cargo xtask smoke confluence restricted
cargo xtask smoke gitlab all
cargo xtask smoke gitlab stdio
cargo xtask smoke gitlab http
cargo xtask smoke gitlab restricted
cargo xtask smoke cli all
cargo xtask smoke cli jira
cargo xtask smoke cli confluence
cargo xtask smoke cli gitlab
```

Smoke options:

- `--port <port>`: streamable HTTP server port for HTTP smoke.
- `--path <path>`: streamable HTTP MCP path for HTTP smoke. Defaults to `/mcp`.

`cargo xtask smoke cli all` runs the production `workhub cli ...` command surface against local mock upstreams. It verifies default text stdout, `--json` stdout, and disabled-tool stderr behavior for Jira, Confluence, and GitLab.

Run real acceptance preflight or full acceptance:

```bash
cargo xtask acceptance jira --preflight
cargo xtask acceptance confluence --preflight
cargo xtask acceptance mcp --preflight
cargo xtask acceptance jira --run target/debug/workhub
cargo xtask acceptance confluence --run target/debug/workhub
cargo xtask acceptance mcp --run target/debug/workhub
```

Acceptance options:

- `--env-file <path>`: load a dotenv file for this acceptance invocation.
- `--run <binary>`: run the acceptance suite against the specified `workhub` binary.
- `--preflight`: check required configuration without running the acceptance suite.

## Acceptance Environment

Acceptance reads environment values in this priority order:

1. Process environment.
2. `--env-file <path>` when provided.
3. `ACCEPTANCE_ENV_FILE` when set.
4. `.env.dev` in the current working directory.

The common service configuration variables are the same as production development configuration:

- Jira Cloud: `JIRA_URL`, `JIRA_USERNAME`, `JIRA_API_TOKEN`.
- Jira Server/Data Center: `JIRA_URL`, `JIRA_PERSONAL_TOKEN`, or `JIRA_URL`, `JIRA_USERNAME`, `JIRA_PASSWORD`.
- Confluence Cloud: `CONFLUENCE_URL`, `CONFLUENCE_USERNAME`, `CONFLUENCE_API_TOKEN`.
- Confluence Server/Data Center: `CONFLUENCE_URL`, `CONFLUENCE_PERSONAL_TOKEN`, or `CONFLUENCE_URL`, `CONFLUENCE_USERNAME`, `CONFLUENCE_PASSWORD`.
- GitLab local smoke/mock tests: `GITLAB_URL`, `GITLAB_TOKEN` or `GITLAB_PERSONAL_TOKEN`, and `GITLAB_PROJECTS_FILTER` mirror production config, but no real GitLab acceptance suite is currently provided.

Common Jira test-object variables:

- `JIRA_READ_ISSUE`
- `JIRA_PROJECT_KEY`
- `JIRA_FIELD_ID`
- `JIRA_FIELD_CONTEXT_ID`
- `JIRA_SERVICE_DESK_ID`
- `JIRA_QUEUE_ID`

Common Confluence test-object variables:

- `CONFLUENCE_SEARCH_QUERY`
- `CONFLUENCE_PAGE_ID`
- `CONFLUENCE_SPACE_KEY`
- `CONFLUENCE_TEST_PAGE_PREFIX`
- `CONFLUENCE_MUTATION_PAGE_ID`
- `CONFLUENCE_COMMENT_ID`
- `CONFLUENCE_ATTACHMENT_ID`
- `CONFLUENCE_ATTACHMENT_FILE`
- `CONFLUENCE_LABEL_NAME`

Use only disposable pages, issues, attachments, labels, and comments. Real acceptance output redacts secret-looking values, but it can still include business object identifiers and test content.

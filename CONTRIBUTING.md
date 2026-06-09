# Contributing

This repository contains the Rust implementation of the MCP Atlassian Jira and Confluence server. Contributions should keep the released binary, Docker image, MCP tool surface, support matrix, security posture, and documentation consistent.

## Local Setup

Install the versions listed in `README.md`, then use `just --list` to inspect the local command surface.

## Verification

Run the checks that match your change. For service code, dependency, CI, Docker, release, support-matrix, or deployment documentation changes, run the full local path:

```bash
cargo fmt --check
RUSTFLAGS="-Dwarnings" cargo check
cargo test
cargo build --release
just smoke
docker compose -f docker-compose.yml config
docker build -t mcp-atlassian-rs:ci -f Dockerfile .
```

Also run a container `/healthz` smoke when changing `Dockerfile`, `docker-compose.yml`, transport startup, health handling, or release packaging.

The shorter project shortcut remains:

```bash
just check
```

`just check` covers Rust formatting, compilation checks, and tests. It does not replace the release, `just smoke`, Docker, compose, or support-matrix checks required for broader changes.

## Pull Requests

Create a branch from the current `master` branch and keep the pull request focused on one change. Include a concise summary, the verification commands you ran, and any follow-up work that remains.

Before opening a pull request:

- Keep generated and local files out of the commit, including `target/`, `.env`, logs, task scratch files, or credentials.
- Do not commit real credentials, tokens, private endpoints, or machine-specific config.
- Keep the main README in English.
- Do not add claims for unsupported capabilities, including full OAuth flows, SSE transport, SOCKS proxy, system truststore injection, Helm, external registry publishing, or real acceptance of blocked product paths.
- Update README, support matrix, backlog, release notes, and task documents when behavior, commands, support status, naming, or deployment automation changes.

## Code Style

Use `cargo fmt` for Rust formatting. Prefer the project patterns already present in `src/`, `Dockerfile`, `docker-compose.yml`, `.github/`, and `docs/`.

## Dependency Updates

Dependency update pull requests should include the relevant lockfile changes and should not bundle unrelated refactors. Before merging dependency updates, run the release validation set:

```bash
cargo fmt --check
RUSTFLAGS="-Dwarnings" cargo check
cargo test
cargo build --release
just smoke
docker compose -f docker-compose.yml config
docker build -t mcp-atlassian-rs:ci -f Dockerfile .
```

Record any Docker or real-environment checks that were not run and why.

default: list

# Show available local shortcuts.
list:
	@just --list

# Run the MCP server over stdio.
dev:
	cd '{{justfile_directory()}}' && cargo run -- stdio

# Run the MCP server over streamable HTTP.
dev-http host="127.0.0.1" port="8000":
	cd '{{justfile_directory()}}' && cargo run -- streamhttp --host "{{host}}" --port "{{port}}"

# Run the streamable HTTP server with MCP tool-call diagnostics enabled.
dev-http-debug host="127.0.0.1" port="8000":
	cd '{{justfile_directory()}}' && RUST_LOG="${RUST_LOG:-workhub_rs::mcp=debug,workhub_rs=info,rmcp=info}" cargo run -- streamhttp --host "{{host}}" --port "{{port}}"

# Run the stdio MCP smoke check.
smoke-stdio:
	cd '{{justfile_directory()}}' && cargo xtask smoke jira stdio

# Run the streamable HTTP MCP smoke check.
smoke-http:
	cd '{{justfile_directory()}}' && cargo xtask smoke jira http

# Run the Jira restricted MCP smoke check against a local mock Jira.
smoke-jira:
	cd '{{justfile_directory()}}' && cargo xtask smoke jira restricted

# Run the Confluence MCP smoke check against a local mock Confluence.
smoke-confluence:
	cd '{{justfile_directory()}}' && cargo xtask smoke confluence all

# Run the GitLab MCP smoke check against a local mock GitLab.
smoke-gitlab:
	cd '{{justfile_directory()}}' && cargo xtask smoke gitlab all

# Run production CLI smoke checks against local mock upstreams.
smoke-cli:
	cd '{{justfile_directory()}}' && cargo xtask smoke cli all

# Run all local smoke checks.
smoke: smoke-stdio smoke-http smoke-jira smoke-confluence smoke-gitlab smoke-cli

# Run real Jira acceptance checks.
acceptance-jira:
	cd '{{justfile_directory()}}' && cargo build --quiet --bin workhub
	cd '{{justfile_directory()}}' && cargo xtask acceptance jira --preflight
	cd '{{justfile_directory()}}' && cargo xtask acceptance jira --run target/debug/workhub

# Run real Confluence acceptance checks.
acceptance-confluence:
	cd '{{justfile_directory()}}' && cargo build --quiet --bin workhub
	cd '{{justfile_directory()}}' && cargo xtask acceptance confluence --preflight
	cd '{{justfile_directory()}}' && cargo xtask acceptance confluence --run target/debug/workhub

# Run real Jira+Confluence MCP acceptance checks.
acceptance-mcp:
	cd '{{justfile_directory()}}' && cargo build --quiet --bin workhub
	cd '{{justfile_directory()}}' && cargo xtask acceptance mcp --preflight
	cd '{{justfile_directory()}}' && cargo xtask acceptance mcp --run target/debug/workhub

# Build the release binary.
build:
	cd '{{justfile_directory()}}' && cargo build --release

# Run tests.
test:
	cd '{{justfile_directory()}}' && cargo test

# Run the local aggregate verification suite.
check: fmt-check check-code test

# Check compilation without producing release artifacts.
check-code:
	cd '{{justfile_directory()}}' && cargo check

# Format Rust sources.
fmt:
	cd '{{justfile_directory()}}' && cargo fmt

# Check Rust formatting without writing changes.
fmt-check:
	cd '{{justfile_directory()}}' && cargo fmt --check

# Build the local Docker image.
docker-build image="workhub-rs:local":
	cd '{{justfile_directory()}}' && docker build -t "{{image}}" -f Dockerfile .

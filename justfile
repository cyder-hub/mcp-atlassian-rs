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

# Run the stdio MCP smoke check.
smoke-stdio:
	cd '{{justfile_directory()}}' && bash scripts/smoke-stdio.sh

# Run the streamable HTTP MCP smoke check.
smoke-http:
	cd '{{justfile_directory()}}' && bash scripts/smoke-http.sh

# Run the Jira read-only MCP smoke check against a local mock Jira.
smoke-jira:
	cd '{{justfile_directory()}}' && bash scripts/smoke-jira.sh

# Run all local MCP smoke checks.
smoke: smoke-stdio smoke-http smoke-jira

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
docker-build image="mcp-atlassian-rs:local":
	cd '{{justfile_directory()}}' && docker build -t "{{image}}" -f Dockerfile .

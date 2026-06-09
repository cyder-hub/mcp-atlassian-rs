#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT_DIR"

MODE="${1:-all}"
case "$MODE" in
    all | stdio | http | read-only) ;;
    *)
        echo "usage: $0 [all|stdio|http|read-only]" >&2
        exit 2
        ;;
esac

cargo build --quiet --bin mcp-atlassian-rs

args=(smoke confluence "$MODE" --path "${MCP_SMOKE_PATH:-/mcp}")
if [[ -n "${MCP_SMOKE_PORT:-}" ]]; then
    args+=(--port "$MCP_SMOKE_PORT")
fi

"$ROOT_DIR/target/debug/mcp-atlassian-rs" "${args[@]}"

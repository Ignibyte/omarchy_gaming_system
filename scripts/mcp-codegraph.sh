#!/usr/bin/env bash
set -euo pipefail

ogs_root=$(git rev-parse --show-toplevel 2>/dev/null) || {
  echo "CodeGraph MCP must start inside the Omarchy Gaming System Git worktree." >&2
  exit 1
}
ogs_binary="$ogs_root/.dev/pipeline-tools/codegraph/node_modules/.bin/codegraph"

[[ -x "$ogs_binary" ]] || {
  echo "CodeGraph is not prepared. Run scripts/setup-pipeline-tools.sh." >&2
  exit 1
}

export CODEGRAPH_TELEMETRY=0
export DO_NOT_TRACK=1
exec "$ogs_binary" serve --mcp

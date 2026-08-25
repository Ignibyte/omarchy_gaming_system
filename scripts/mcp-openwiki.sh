#!/usr/bin/env bash
set -euo pipefail

ogs_root=$(git rev-parse --show-toplevel 2>/dev/null) || {
  echo "OpenWiki MCP must start inside the Omarchy Gaming System Git worktree." >&2
  exit 1
}
ogs_entrypoint="$ogs_root/.dev/pipeline-tools/openwiki/dist/cli/cli.js"

[[ -f "$ogs_entrypoint" ]] || {
  echo "OpenWiki is not prepared. Run scripts/setup-pipeline-tools.sh." >&2
  exit 1
}

"$ogs_root/scripts/check-pipeline-tools.sh" >/dev/null || {
  echo "OpenWiki install or build provenance is invalid. Run scripts/setup-pipeline-tools.sh." >&2
  exit 1
}

export DO_NOT_TRACK=1
export OPENWIKI_TELEMETRY_DISABLED=1
exec node "$ogs_entrypoint" mcp --host codex

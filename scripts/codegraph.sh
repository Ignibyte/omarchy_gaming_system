#!/usr/bin/env bash
set -euo pipefail

ogs_root=$(git rev-parse --show-toplevel 2>/dev/null) || {
  echo "CodeGraph must run inside the Omarchy Gaming System Git worktree." >&2
  exit 1
}
ogs_binary="$ogs_root/.dev/pipeline-tools/codegraph/node_modules/.bin/codegraph"

[[ -x "$ogs_binary" ]] || {
  echo "CodeGraph is not prepared. Run scripts/setup-pipeline-tools.sh." >&2
  exit 1
}

export CODEGRAPH_TELEMETRY=0
export DO_NOT_TRACK=1

ogs_status=0
"$ogs_binary" "$@" || ogs_status=$?

if ((ogs_status == 0)) && [[ "${1:-}" == "explore" ]]; then
  # shellcheck source=.codex/hooks/lib-hook-helpers.sh
  source "$ogs_root/.codex/hooks/lib-hook-helpers.sh"
  ogs_pipeline_id=$(active_pipeline_id)
  ogs_slot=$(codegraph_receipt_slot)
  if [[ -n "$ogs_pipeline_id" && -n "$ogs_slot" ]]; then
    write_pipeline_tool_receipt "$ogs_slot" "$ogs_pipeline_id" "CodeGraph CLI"
  fi
fi

exit "$ogs_status"

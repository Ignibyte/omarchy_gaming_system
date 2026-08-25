#!/usr/bin/env bash
set -euo pipefail

ogs_input=$(cat)
command -v jq >/dev/null 2>&1 || exit 0

# shellcheck source=.codex/hooks/lib-hook-helpers.sh
source "$(dirname "$0")/lib-hook-helpers.sh"

ogs_pipeline_id=$(active_pipeline_id)
[[ -n "$ogs_pipeline_id" ]] || exit 0

ogs_tool=$(jq -r '.tool_name // empty' <<<"$ogs_input")
ogs_slot=""

case "$ogs_tool" in
  mcp__codegraph__codegraph_explore)
    hook_tool_succeeded "$ogs_input" || exit 0
    ogs_slot=$(codegraph_receipt_slot)
    ;;
  mcp__openwiki__openwiki_finish)
    grep -q '^Phase 4' <<<"$(active_pipeline_status)" || exit 0
    hook_tool_succeeded "$ogs_input" || exit 0
    jq -e '
      .tool_response.structuredContent.status == "complete" or
      ([.tool_response.content[]?.text?] | any(test("\\\"status\\\"[[:space:]]*:[[:space:]]*\\\"complete\\\"")))
    ' <<<"$ogs_input" >/dev/null || exit 0
    ogs_slot="complete"
    ;;
esac

[[ -n "$ogs_slot" ]] || exit 0
write_pipeline_tool_receipt "$ogs_slot" "$ogs_pipeline_id" "$ogs_tool"

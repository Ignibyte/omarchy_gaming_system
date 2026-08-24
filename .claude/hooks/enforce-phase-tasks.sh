#!/usr/bin/env bash
set -euo pipefail

bbs_input=$(cat)
command -v jq >/dev/null 2>&1 || exit 0

# shellcheck source=.claude/hooks/lib-hook-helpers.sh
source "$(dirname "$0")/lib-hook-helpers.sh"

bbs_transcript=$(jq -r '.transcript_path // empty' <<<"$bbs_input")
[[ -f "$bbs_transcript" ]] || exit 0
[[ -n "$(get_active_pipeline_doc)" ]] || exit 0
bbs_phase=$(detect_active_command "$bbs_transcript")
[[ -n "$bbs_phase" ]] || exit 0

if ! transcript_readable "$bbs_transcript"; then
  echo "PHASE-TASKS CHECK DEGRADED — transcript unreadable: $bbs_transcript" >&2
  exit 1
fi

bbs_index=$(index_of_latest_phase_advance "$bbs_transcript") || exit 1
[[ -n "$bbs_index" ]] || exit 0
bbs_created=$(count_tool_uses_after_index "$bbs_transcript" "$bbs_index" "TaskCreate") || exit 1
bbs_resolved=$(count_resolved_tasks_after_index "$bbs_transcript" "$bbs_index") || exit 1

if ((bbs_created == 0)); then
  printf '\nSTOP BLOCKED — /pipeline:%s created no tasks. Create the phase checklist first.\n' \
    "$bbs_phase" >&2
  exit 2
fi

if ((bbs_resolved < bbs_created)); then
  printf '\nSTOP BLOCKED — /pipeline:%s has unresolved tasks (%s/%s resolved).\n' \
    "$bbs_phase" "$bbs_resolved" "$bbs_created" >&2
  exit 2
fi

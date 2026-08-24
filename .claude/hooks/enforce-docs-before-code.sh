#!/usr/bin/env bash
set -euo pipefail

bbs_input=$(cat)
command -v jq >/dev/null 2>&1 || exit 0

# shellcheck source=.claude/hooks/lib-hook-helpers.sh
source "$(dirname "$0")/lib-hook-helpers.sh"

bbs_file_path=$(jq -r '.tool_input.file_path // empty' <<<"$bbs_input")
[[ -n "$bbs_file_path" ]] || exit 0
bbs_file_path=$(normalize_path "$bbs_file_path")
bbs_is_gated_path "$bbs_file_path" || exit 0

bbs_transcript=$(jq -r '.transcript_path // empty' <<<"$bbs_input")
[[ -f "$bbs_transcript" ]] || exit 0
is_pipeline_session "$bbs_transcript" || exit 0

bbs_phase=$(detect_active_command "$bbs_transcript")
case "$bbs_phase" in
  implement | inspect | validate) ;;
  *) exit 0 ;;
esac

bbs_recall_result=0
knowledge_recall_seen "$bbs_transcript" || bbs_recall_result=$?
[[ "$bbs_recall_result" == 2 ]] && exit 0
if [[ "$bbs_recall_result" != 0 ]]; then
  printf '\nDOCS-BEFORE-CODE BLOCKED — recall the knowledge register, prior pipeline notes, or architecture docs before writing %s (§18).\n' \
    "$bbs_file_path" >&2
  exit 2
fi

if [[ "$bbs_phase" == "implement" ]]; then
  bbs_recall_result=0
  architecture_recall_seen "$bbs_transcript" || bbs_recall_result=$?
  [[ "$bbs_recall_result" == 2 ]] && exit 0
  if [[ "$bbs_recall_result" != 0 ]]; then
    printf '\nDOCS-BEFORE-CODE BLOCKED — implementation requires a relevant docs/architecture read before writing %s.\n' \
      "$bbs_file_path" >&2
    exit 2
  fi
fi

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

bbs_missing=""
case "$bbs_phase" in
  plan)
    find "$PROJECT_ROOT/docs/planning/tickets/open" -maxdepth 1 -name 'TICKET-*.md' \
      -print -quit | grep -q . || bbs_missing=" open ticket"
    [[ -n "$(get_aar_for_active_pipeline)" ]] || bbs_missing+=" active AAR"
    ;;
  design)
    knowledge_recall_seen "$bbs_transcript" || bbs_missing=" knowledge recall"
    architecture_recall_seen "$bbs_transcript" || bbs_missing+=" architecture read"
    ;;
  implement | inspect)
    knowledge_recall_seen "$bbs_transcript" || bbs_missing=" knowledge recall"
    ;;
  validate)
    bbs_commands=$(extract_bash_commands "$bbs_transcript") || exit 1
    [[ -n "$bbs_commands" ]] || bbs_missing=" real validation command"
    ;;
  complete)
    bbs_aar=$(get_aar_for_active_pipeline)
    [[ -n "$bbs_aar" && "$(aar_status "$bbs_aar")" == "submitted" ]] \
      || bbs_missing=" submitted AAR"
    ;;
esac

if [[ -n "$bbs_missing" ]]; then
  printf '\nCOMPLETION BLOCKED — /pipeline:%s is missing:%s (§18/§19).\n' \
    "$bbs_phase" "$bbs_missing" >&2
  exit 2
fi

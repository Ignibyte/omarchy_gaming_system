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

bbs_doc=$(get_active_pipeline_doc)
[[ -n "$bbs_doc" ]] || exit 0
bbs_phase=$(detect_active_command "$bbs_transcript")

block_phase() {
  printf '\nPHASE GATE BLOCKED — %s\nSpec: %s\nFile: %s\n' \
    "$1" "$bbs_doc" "$bbs_file_path" >&2
  exit 2
}

[[ -n "$bbs_phase" ]] \
  || block_phase "a gated write requires a resolvable /pipeline phase (§3)."

phase_passed() {
  local bbs_number
  local bbs_status
  bbs_number=$(sed 's/\./\\./g' <<<"$1")
  bbs_status=$(sed -n 's/^status:[[:space:]]*//p' "$bbs_doc" | head -1)
  grep -qE "Phase[[:space:]]+${bbs_number}[[:space:]][^0-9;]*PASS" <<<"$bbs_status"
}

case "$bbs_phase" in
  plan) block_phase "planning may write pipeline documents, not application code." ;;
  design) block_phase "design may write notes and architecture documents, not application code." ;;
  implement) phase_passed 2 || block_phase "Phase 2 (Design) is not PASS." ;;
  inspect) phase_passed 3 || block_phase "Phase 3 (Implement) is not PASS." ;;
  validate) phase_passed 3.5 || block_phase "Phase 3.5 (Inspect) is not PASS." ;;
  complete) phase_passed 4 || block_phase "Phase 4 (Validate) is not PASS." ;;
esac

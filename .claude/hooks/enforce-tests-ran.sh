#!/usr/bin/env bash
set -euo pipefail

bbs_input=$(cat)
command -v jq >/dev/null 2>&1 || exit 0

# shellcheck source=.claude/hooks/lib-hook-helpers.sh
source "$(dirname "$0")/lib-hook-helpers.sh"

bbs_transcript=$(jq -r '.transcript_path // empty' <<<"$bbs_input")
[[ -f "$bbs_transcript" ]] || exit 0
[[ "$(latest_pipeline_command "$bbs_transcript")" == "pipeline:validate" ]] \
  || [[ "$(latest_pipeline_command "$bbs_transcript")" == "pipeline-validate" ]] \
  || exit 0

if ! transcript_readable "$bbs_transcript"; then
  echo "TESTS-RAN CHECK DEGRADED — transcript unreadable: $bbs_transcript" >&2
  exit 1
fi

bbs_commands=$(extract_bash_commands "$bbs_transcript") || exit 1
if ! grep -qE 'cargo[[:space:]]+(test|nextest)|bin/gate\.sh' <<<"$bbs_commands"; then
  echo "TESTS-RAN BLOCKED — validate must run cargo test or bin/gate.sh and report its real output (§7/§15)." >&2
  exit 2
fi

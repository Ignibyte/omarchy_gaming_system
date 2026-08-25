#!/usr/bin/env bash
set -euo pipefail

ogs_input=$(cat)
if ! command -v jq >/dev/null 2>&1; then
  grep -q 'commit' <<<"$ogs_input" \
    && echo "COMMIT BLOCKED — jq is required to inspect commit commands." >&2 \
    && exit 2
  exit 0
fi

# shellcheck source=.codex/hooks/lib-hook-helpers.sh
source "$(dirname "$0")/lib-hook-helpers.sh"

ogs_command=$(jq -r '.tool_input.command // empty' <<<"$ogs_input")
[[ -n "$ogs_command" ]] || exit 0
ogs_command=${ogs_command//$'\\\n'/}
ogs_command=${ogs_command//$'\n'/;}

grep -qE '(^|[^[:alnum:]_.-])git[[:space:]]+([^;&|]*[[:space:]])?commit([[:space:]]|$)' \
  <<<"$ogs_command" || exit 0

# Only an exact standalone non-mutating inquiry is exempt. A help or dry-run
# token elsewhere in a compound command must not exempt a real commit.
if grep -qE \
  '^[[:space:]]*git[[:space:]]+commit[[:space:]]+(--dry-run|--help|-h)[[:space:]]*$' \
  <<<"$ogs_command"; then
  exit 0
fi

ogs_active=$(get_active_pipeline_doc)
if [[ -n "$ogs_active" ]]; then
  printf '\nCOMMIT BLOCKED — pipeline work is still active.\nComplete and archive %s before delivery (§3/§19).\n' \
    "$ogs_active" >&2
  exit 2
fi

cd "$PROJECT_ROOT"
ogs_gated_change=0
while IFS= read -r -d '' ogs_file; do
  if ogs_is_gated_path "$ogs_file"; then
    ogs_gated_change=1
    break
  fi
done < <(
  {
    git diff --name-only -z HEAD 2>/dev/null
    git ls-files -z --others --exclude-standard 2>/dev/null
  } | LC_ALL=C sort -zu
)

((ogs_gated_change == 1)) || exit 0

matching_gate_receipt_exists && exit 0

printf '\nCOMMIT BLOCKED — gated files lack a matching delivery receipt.\nRun bin/gate.sh --diff after the last gated edit, fix every red, then commit (§0/§15).\n' >&2
exit 2

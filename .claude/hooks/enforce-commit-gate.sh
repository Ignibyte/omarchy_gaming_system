#!/usr/bin/env bash
set -euo pipefail

bbs_input=$(cat)
if ! command -v jq >/dev/null 2>&1; then
  grep -q 'commit' <<<"$bbs_input" \
    && echo "COMMIT BLOCKED — jq is required to inspect commit commands." >&2 \
    && exit 2
  exit 0
fi

# shellcheck source=.claude/hooks/lib-hook-helpers.sh
source "$(dirname "$0")/lib-hook-helpers.sh"

bbs_command=$(jq -r '.tool_input.command // empty' <<<"$bbs_input")
[[ -n "$bbs_command" ]] || exit 0
bbs_command=${bbs_command//$'\\\n'/}
bbs_command=${bbs_command//$'\n'/;}

grep -qE '(^|[^[:alnum:]_.-])git[[:space:]]+([^;&|]*[[:space:]])?commit([[:space:]]|$)' \
  <<<"$bbs_command" || exit 0

if grep -qE -- '(^|[[:space:]])(--dry-run|--help|-h)([[:space:]]|$)' \
  <<<"$bbs_command"; then
  exit 0
fi

cd "$PROJECT_ROOT"
bbs_changed=$(
  {
    git diff --name-only HEAD 2>/dev/null
    git ls-files --others --exclude-standard 2>/dev/null
  } | LC_ALL=C sort -u
)
bbs_gated_change=0
while IFS= read -r bbs_file; do
  if bbs_is_gated_path "$bbs_file"; then
    bbs_gated_change=1
    break
  fi
done <<<"$bbs_changed"

((bbs_gated_change == 1)) || exit 0

bbs_receipt=$(bbs_gate_receipt_path) || true
if [[ -n "$bbs_receipt" && -f "$bbs_receipt" ]] \
  && [[ "$(<"$bbs_receipt")" == "$(bbs_gate_state_hash)" ]]; then
  exit 0
fi

printf '\nCOMMIT BLOCKED — gated files lack a matching delivery receipt.\nRun bin/gate.sh --diff after the last gated edit, fix every red, then commit (§0/§15).\n' >&2
exit 2

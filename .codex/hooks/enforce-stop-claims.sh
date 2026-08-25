#!/usr/bin/env bash
set -euo pipefail

ogs_input=$(cat)
command -v jq >/dev/null 2>&1 || exit 0

# shellcheck source=.codex/hooks/lib-hook-helpers.sh
source "$(dirname "$0")/lib-hook-helpers.sh"

ogs_message=$(jq -r '.last_assistant_message // empty' <<<"$ogs_input")
[[ -n "$ogs_message" ]] || exit 0
ogs_active=$(get_active_pipeline_doc)

if [[ -n "$ogs_active" ]] \
  && grep -qiE 'Phase 2([[:space:]]+—[[:space:]]+Design)? PASS' <<<"$ogs_message" \
  && ! matching_pipeline_tool_receipt_exists design; then
  echo "DESIGN CLAIM BLOCKED — Phase 2 success requires a CodeGraph explore receipt matching this pipeline and gated worktree. Use CodeGraph after planning and before claiming the design gate." >&2
  exit 2
fi

if [[ -n "$ogs_active" ]] \
  && grep -qiE 'Phase 3[.]5([[:space:]]+—[[:space:]]+Inspect)? PASS' <<<"$ogs_message" \
  && ! matching_pipeline_tool_receipt_exists inspect; then
  echo "INSPECTION CLAIM BLOCKED — Phase 3.5 success requires a fresh CodeGraph explore receipt matching the post-implementation gated worktree." >&2
  exit 2
fi

if grep -qiE 'Phase 4 PASS|GATE GREEN \[(diff|full)\]' <<<"$ogs_message" \
  && ! matching_gate_receipt_exists; then
  echo "VALIDATION CLAIM BLOCKED — Phase 4/full-gate success requires a receipt matching the current gated worktree (§0/§15). Run bin/gate.sh --diff after the last gated edit." >&2
  exit 2
fi

if grep -qiE 'Phase 5 PASS|pipeline (is )?complete' <<<"$ogs_message"; then
  if ! matching_pipeline_tool_receipt_exists complete; then
    echo "COMPLETION CLAIM BLOCKED — Phase 5 success requires a completed OpenWiki lifecycle receipt matching this pipeline and gated worktree." >&2
    exit 2
  fi
  if [[ -n "$ogs_active" ]]; then
    printf 'COMPLETION CLAIM BLOCKED — the pipeline is still active at %s. Finish the AC audit, AAR, ticket closure, and archive (§19).\n' \
      "$ogs_active" >&2
    exit 2
  fi
fi

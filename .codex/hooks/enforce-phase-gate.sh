#!/usr/bin/env bash
set -euo pipefail

ogs_input=$(cat)
command -v jq >/dev/null 2>&1 || exit 0

# shellcheck source=.codex/hooks/lib-hook-helpers.sh
source "$(dirname "$0")/lib-hook-helpers.sh"

ogs_doc=$(get_active_pipeline_doc)
[[ -n "$ogs_doc" ]] || exit 0
design_is_passed && exit 0

ogs_gated_paths=""
while IFS= read -r ogs_file; do
  [[ -n "$ogs_file" ]] || continue
  ogs_original_file="$ogs_file"
  if ! ogs_file=$(normalize_path "$ogs_file"); then
    ogs_gated_paths+=" $ogs_original_file (outside or unresolved)"
    continue
  fi
  if ogs_is_gated_path "$ogs_file"; then
    ogs_gated_paths+=" $ogs_file"
  fi
done < <(extract_edit_paths "$ogs_input")

[[ -n "$ogs_gated_paths" ]] || exit 0

printf '\nPHASE GATE BLOCKED — Design must be PASS before editing gated files:%s\nSpec: %s\n' \
  "$ogs_gated_paths" "$ogs_doc" >&2
exit 2

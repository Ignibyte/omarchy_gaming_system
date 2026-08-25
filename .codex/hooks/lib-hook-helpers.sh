#!/usr/bin/env bash

# Shared, worktree-based helpers for Codex lifecycle hooks.

PROJECT_ROOT=$(git rev-parse --show-toplevel 2>/dev/null || pwd)
OGS_PROJECT_ROOT="$PROJECT_ROOT"

# shellcheck source=bin/lib-gate.sh
source "$PROJECT_ROOT/bin/lib-gate.sh"

normalize_path() {
  local ogs_input_path="$1"
  local ogs_absolute_path
  local ogs_canonical_root

  command -v realpath >/dev/null 2>&1 || return 1
  ogs_canonical_root=$(realpath -m -- "$PROJECT_ROOT") || return 1
  case "$ogs_input_path" in
    /*) ogs_absolute_path=$(realpath -m -- "$ogs_input_path") || return 1 ;;
    *) ogs_absolute_path=$(realpath -m -- "$ogs_canonical_root/$ogs_input_path") || return 1 ;;
  esac

  case "$ogs_absolute_path" in
    "$ogs_canonical_root") printf '.\n' ;;
    "$ogs_canonical_root"/*) printf '%s\n' "${ogs_absolute_path#"$ogs_canonical_root"/}" ;;
    *) return 1 ;;
  esac
}

get_active_pipeline_doc() {
  local ogs_file

  for ogs_file in "$PROJECT_ROOT"/docs/planning/pipeline/active/*.spec.md; do
    [[ -f "$ogs_file" ]] && {
      printf '%s\n' "$ogs_file"
      return 0
    }
  done
  return 0
}

active_pipeline_status() {
  local ogs_doc

  ogs_doc=$(get_active_pipeline_doc)
  [[ -n "$ogs_doc" ]] || return 0
  sed -n 's/^status:[[:space:]]*//p' "$ogs_doc" | head -1
}

active_pipeline_id() {
  local ogs_doc

  ogs_doc=$(get_active_pipeline_doc)
  [[ -n "$ogs_doc" ]] || return 0
  sed -n 's/^pipeline_id:[[:space:]]*//p' "$ogs_doc" | head -1
}

codegraph_receipt_slot() {
  local ogs_status

  ogs_status=$(active_pipeline_status)
  case "$ogs_status" in
    "Phase 1"*) printf 'design\n' ;;
    "Phase 3"*) printf 'inspect\n' ;;
  esac
}

pipeline_tool_receipt_dir() {
  local ogs_git_dir

  ogs_git_dir=$(git -C "$PROJECT_ROOT" rev-parse --git-dir 2>/dev/null) || return 1
  case "$ogs_git_dir" in
    /*) printf '%s/omarchy-gaming-system-pipeline-tools\n' "$ogs_git_dir" ;;
    *) printf '%s/%s/omarchy-gaming-system-pipeline-tools\n' "$PROJECT_ROOT" "$ogs_git_dir" ;;
  esac
}

pipeline_tool_receipt_path() {
  local ogs_slot="$1"
  local ogs_dir

  case "$ogs_slot" in
    design | inspect | complete) ;;
    *) return 1 ;;
  esac
  ogs_dir=$(pipeline_tool_receipt_dir) || return 1
  printf '%s/%s.receipt\n' "$ogs_dir" "$ogs_slot"
}

hook_tool_succeeded() {
  local ogs_input="$1"

  jq -e '
    (.tool_response.isError? // false) != true and
    (.tool_response.exit_code? // .tool_response.exitCode? // 0) == 0
  ' <<<"$ogs_input" >/dev/null
}

write_pipeline_tool_receipt() {
  local ogs_slot="$1"
  local ogs_pipeline_id="$2"
  local ogs_tool="$3"
  local ogs_receipt
  local ogs_dir
  local ogs_temp

  ogs_receipt=$(pipeline_tool_receipt_path "$ogs_slot") || return 1
  ogs_dir=$(dirname "$ogs_receipt")
  mkdir -p "$ogs_dir"
  ogs_temp=$(mktemp "$ogs_dir/.${ogs_slot}.XXXXXX")
  {
    printf 'version=1\n'
    printf 'pipeline_id=%s\n' "$ogs_pipeline_id"
    printf 'state_hash=%s\n' "$(ogs_gate_state_hash)"
    printf 'tool=%s\n' "$ogs_tool"
  } >"$ogs_temp"
  mv "$ogs_temp" "$ogs_receipt"
}

matching_pipeline_tool_receipt_exists() {
  local ogs_slot="$1"
  local ogs_receipt
  local ogs_pipeline_id
  local ogs_receipt_id
  local ogs_receipt_hash
  local ogs_completed

  ogs_receipt=$(pipeline_tool_receipt_path "$ogs_slot") || return 1
  [[ -f "$ogs_receipt" ]] || return 1
  ogs_receipt_id=$(sed -n 's/^pipeline_id=//p' "$ogs_receipt" | head -1)
  ogs_receipt_hash=$(sed -n 's/^state_hash=//p' "$ogs_receipt" | head -1)
  [[ -n "$ogs_receipt_id" && "$ogs_receipt_hash" == "$(ogs_gate_state_hash)" ]] || return 1

  ogs_pipeline_id=$(active_pipeline_id)
  if [[ -n "$ogs_pipeline_id" ]]; then
    [[ "$ogs_receipt_id" == "$ogs_pipeline_id" ]]
    return
  fi

  [[ "$ogs_slot" == "complete" ]] || return 1
  for ogs_completed in "$PROJECT_ROOT"/docs/planning/pipeline/completed/*.spec.md; do
    [[ -f "$ogs_completed" ]] || continue
    grep -qx "pipeline_id: $ogs_receipt_id" "$ogs_completed" \
      && grep -qx 'status: Phase 5 — Complete PASS' "$ogs_completed" \
      && return 0
  done
  return 1
}

design_is_passed() {
  local ogs_status

  ogs_status=$(active_pipeline_status)
  grep -qE '^Phase (2|3|3\.5|4|5)[[:space:]].*PASS' <<<"$ogs_status"
}

extract_edit_paths() {
  local ogs_input="$1"
  local ogs_command
  local ogs_path

  jq -r '.tool_input.file_path // .tool_input.path // empty' <<<"$ogs_input"
  ogs_command=$(jq -r '.tool_input.command // empty' <<<"$ogs_input")
  [[ -n "$ogs_command" ]] || return 0

  while IFS= read -r ogs_path; do
    printf '%s\n' "$ogs_path"
  done < <(
    sed -nE \
      -e 's/^\*\*\* (Add|Update|Delete) File: (.*)$/\2/p' \
      -e 's/^\*\*\* Move to: (.*)$/\1/p' \
      <<<"$ogs_command"
  )
}

matching_gate_receipt_exists() {
  local ogs_receipt

  ogs_receipt=$(ogs_gate_receipt_path) || return 1
  [[ -f "$ogs_receipt" ]] || return 1
  [[ "$(<"$ogs_receipt")" == "$(ogs_gate_state_hash)" ]]
}

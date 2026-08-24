#!/usr/bin/env bash

# Shared transcript and pipeline helpers for Claude Code hooks.

PROJECT_ROOT=$(git rev-parse --show-toplevel 2>/dev/null || pwd)
BBS_PROJECT_ROOT="$PROJECT_ROOT"

# shellcheck source=bin/lib-gate.sh
source "$PROJECT_ROOT/bin/lib-gate.sh"

normalize_path() {
  printf '%s\n' "$1" \
    | sed "s|^${PROJECT_ROOT}/||" \
    | sed 's|^\.claude/worktrees/[A-Za-z0-9._-]\+/||'
}

transcript_readable() {
  [[ -f "$1" ]] || return 1
  head -c 1 "$1" >/dev/null 2>&1
}

_JSONL_PRELUDE='[inputs | (fromjson? // {}) | if type == "object" then . else {} end]'
_PIPELINE_RE='(pipeline[-:](plan|design|implement|inspect|validate|complete)|commit|work)'
_KNOWLEDGE_RE='CONSTITUTION|docs/architecture|docs/planning/(knowledge|pipeline|tickets)|ROADMAP'

extract_read_targets() {
  local bbs_transcript="$1"
  [[ -f "$bbs_transcript" ]] || return 0
  jq -nRr "$_JSONL_PRELUDE"' |
    [.[] | ((.message // .) | if type == "object" then . else {} end) |
      select(.role == "assistant") | (.content // [])[]? | objects |
      select(.type == "tool_use") |
      select(.name == "Read" or .name == "Grep" or .name == "Glob") |
      (.input.file_path? // empty), (.input.path? // empty),
      (.input.pattern? // empty), (.input.glob? // empty)] | .[]' \
    "$bbs_transcript" 2>/dev/null
}

extract_write_paths() {
  local bbs_transcript="$1"
  [[ -f "$bbs_transcript" ]] || return 0
  jq -nRr "$_JSONL_PRELUDE"' |
    [.[] | ((.message // .) | if type == "object" then . else {} end) |
      select(.role == "assistant") | (.content // [])[]? | objects |
      select(.type == "tool_use") |
      select(.name == "Write" or .name == "Edit" or .name == "NotebookEdit") |
      .input.file_path? // empty] | .[]' "$bbs_transcript" 2>/dev/null
}

extract_bash_commands() {
  local bbs_transcript="$1"
  [[ -f "$bbs_transcript" ]] || return 0
  jq -nRr "$_JSONL_PRELUDE"' |
    [.[] | ((.message // .) | if type == "object" then . else {} end) |
      select(.role == "assistant") | (.content // [])[]? | objects |
      select(.type == "tool_use" and .name == "Bash") |
      .input.command? // empty] | .[]' "$bbs_transcript" 2>/dev/null
}

latest_pipeline_command() {
  local bbs_transcript="$1"
  [[ -f "$bbs_transcript" ]] || return 0
  jq -nRr --arg bbs_re "^\\s*/?$_PIPELINE_RE\\b" "$_JSONL_PRELUDE"' |
    [.[] | ((.message // .) | if type == "object" then . else {} end) |
      if .role == "user" then
        ((.content // "") |
          if type == "array" then
            ([.[]? | objects | select(.type == "text") | .text? // empty] | join("\n"))
          elif type == "string" then . else "" end)
      else "" end] |
    map(select(test($bbs_re))) | last // empty' "$bbs_transcript" 2>/dev/null \
    | grep -oE "/?$_PIPELINE_RE" | tail -1 | sed 's#^/##' || true
}

detect_active_command() {
  latest_pipeline_command "$1" \
    | sed -nE 's/^pipeline[-:](plan|design|implement|inspect|validate|complete)$/\1/p'
}

get_active_pipeline_doc() {
  local bbs_file
  for bbs_file in "$PROJECT_ROOT"/docs/planning/pipeline/active/*.spec.md; do
    [[ -f "$bbs_file" ]] && {
      printf '%s\n' "$bbs_file"
      return 0
    }
  done
  return 0
}

is_pipeline_session() {
  [[ -n "$(get_active_pipeline_doc)" ]] || [[ -n "$(detect_active_command "$1")" ]]
}

knowledge_recall_seen() {
  local bbs_targets
  bbs_targets=$(extract_read_targets "$1") || return 2
  grep -qE "$_KNOWLEDGE_RE|(PR|BF|AD)-omarchy-bbs" <<<"$bbs_targets"
}

architecture_recall_seen() {
  local bbs_targets
  bbs_targets=$(extract_read_targets "$1") || return 2
  grep -qE 'docs/architecture' <<<"$bbs_targets"
}

index_of_latest_phase_advance() {
  local bbs_transcript="$1"
  [[ -f "$bbs_transcript" ]] || return 0
  jq -nRr --arg bbs_re "^\\s*/?pipeline[-:](plan|design|implement|inspect|validate|complete)\\b" "$_JSONL_PRELUDE"' |
    [.[] | ((.message // .) | if type == "object" then . else {} end) |
      if .role == "user" then
        ((.content // "") |
          if type == "array" then
            ([.[]? | objects | select(.type == "text") | .text? // empty] | join("\n"))
          elif type == "string" then . else "" end)
      else "" end] |
    to_entries | map(select(.value | test($bbs_re))) | last // empty |
    if . == "" then "" else (.key | tostring) end' "$bbs_transcript" 2>/dev/null
}

count_tool_uses_after_index() {
  local bbs_transcript="$1"
  local bbs_index="${2:--1}"
  local bbs_tool="$3"
  jq -nRr --argjson bbs_index "$bbs_index" --arg bbs_tool "$bbs_tool" "$_JSONL_PRELUDE"' |
    [.[] | ((.message // .) | if type == "object" then . else {} end) |
      if .role == "assistant" then
        ([.content // [] | .[]? | objects |
          select(.type == "tool_use" and .name == $bbs_tool)] | length)
      else 0 end] |
    to_entries | map(select(.key > $bbs_index) | .value) | add // 0' \
    "$bbs_transcript" 2>/dev/null
}

count_resolved_tasks_after_index() {
  local bbs_transcript="$1"
  local bbs_index="${2:--1}"
  jq -nRr --argjson bbs_index "$bbs_index" "$_JSONL_PRELUDE"' |
    [.[] | ((.message // .) | if type == "object" then . else {} end) |
      if .role == "assistant" then
        ([.content // [] | .[]? | objects |
          select(.type == "tool_use" and .name == "TaskUpdate") |
          select((.input.status? // "") == "completed" or
                 (.input.status? // "") == "deleted")] | length)
      else 0 end] |
    to_entries | map(select(.key > $bbs_index) | .value) | add // 0' \
    "$bbs_transcript" 2>/dev/null
}

get_aar_for_active_pipeline() {
  local bbs_doc bbs_slug bbs_file
  bbs_doc=$(get_active_pipeline_doc)
  [[ -n "$bbs_doc" ]] || return 0
  bbs_slug=$(basename "$bbs_doc" .spec.md)

  for bbs_file in "$PROJECT_ROOT"/docs/planning/knowledge/aar/AAR-*.md; do
    [[ -f "$bbs_file" ]] || continue
    grep -qE "^pipeline:[[:space:]]*${bbs_slug}[[:space:]]*$" "$bbs_file" \
      && {
        printf '%s\n' "$bbs_file"
        return 0
      }
  done
  return 0
}

aar_status() {
  [[ -f "$1" ]] || return 0
  sed -n 's/^status:[[:space:]]*//p' "$1" | head -1
}

#!/usr/bin/env bash

# Shared, side-effect-free helpers for the delivery gate and Codex commit hook.

OGS_PROJECT_ROOT="${OGS_PROJECT_ROOT:-$(git rev-parse --show-toplevel 2>/dev/null || pwd)}"

ogs_is_gated_path() {
  local ogs_path="$1"

  git -C "$OGS_PROJECT_ROOT" check-ignore -q "$ogs_path" 2>/dev/null && return 1

  case "$ogs_path" in
    crates/* | client/* | examples/first-party-door-legends/* | migrations/* | \
      scripts/*.sh | bin/*.sh | \
      Cargo.toml | Cargo.lock | compose.yaml | mise.toml | \
      .github/workflows/* | .codex/config.toml | .codex/hooks.json | \
      .codex/hooks/*.sh | openwiki/* | \
      .agents/skills/* | AGENTS.md | CONSTITUTION.md)
      return 0
      ;;
  esac

  return 1
}

ogs_gated_file_list_nul() {
  {
    git -C "$OGS_PROJECT_ROOT" ls-files -z 2>/dev/null
    git -C "$OGS_PROJECT_ROOT" ls-files -z --others --exclude-standard 2>/dev/null
  } | LC_ALL=C sort -zu | while IFS= read -r -d '' ogs_file; do
    if ogs_is_gated_path "$ogs_file" && [[ -f "$OGS_PROJECT_ROOT/$ogs_file" ]]; then
      printf '%s\0' "$ogs_file"
    fi
  done
}

ogs_gate_state_hash() {
  if ! command -v sha256sum >/dev/null 2>&1; then
    printf 'sha256sum-missing-%s\n' "$$"
    return 0
  fi

  {
    git -C "$OGS_PROJECT_ROOT" rev-parse HEAD 2>/dev/null || printf 'no-head\n'
    ogs_gated_file_list_nul | while IFS= read -r -d '' ogs_file; do
      sha256sum --zero -- "$OGS_PROJECT_ROOT/$ogs_file"
    done
  } | sha256sum | awk '{print $1}'
}

ogs_gate_receipt_path() {
  local ogs_git_dir
  ogs_git_dir=$(git -C "$OGS_PROJECT_ROOT" rev-parse --git-dir 2>/dev/null) || return 1

  case "$ogs_git_dir" in
    /*) printf '%s/omarchy-gaming-system-gate-receipt\n' "$ogs_git_dir" ;;
    *) printf '%s/%s/omarchy-gaming-system-gate-receipt\n' "$OGS_PROJECT_ROOT" "$ogs_git_dir" ;;
  esac
}

#!/usr/bin/env bash

# Shared, side-effect-free helpers for the delivery gate and Claude commit hook.

BBS_PROJECT_ROOT="${BBS_PROJECT_ROOT:-$(git rev-parse --show-toplevel 2>/dev/null || pwd)}"

bbs_is_gated_path() {
  local bbs_path="$1"

  git -C "$BBS_PROJECT_ROOT" check-ignore -q "$bbs_path" 2>/dev/null && return 1

  case "$bbs_path" in
    crates/* | client/* | migrations/* | scripts/*.sh | bin/*.sh | \
      Cargo.toml | Cargo.lock | compose.yaml | mise.toml | \
      .github/workflows/* | .claude/settings.json | .claude/hooks/*.sh)
      return 0
      ;;
  esac

  return 1
}

bbs_gated_file_list() {
  {
    git -C "$BBS_PROJECT_ROOT" ls-files 2>/dev/null
    git -C "$BBS_PROJECT_ROOT" ls-files --others --exclude-standard 2>/dev/null
  } | LC_ALL=C sort -u | while IFS= read -r bbs_file; do
    if bbs_is_gated_path "$bbs_file" && [[ -f "$BBS_PROJECT_ROOT/$bbs_file" ]]; then
      printf '%s\n' "$bbs_file"
    fi
  done
}

bbs_gate_state_hash() {
  if ! command -v sha256sum >/dev/null 2>&1; then
    printf 'sha256sum-missing-%s\n' "$$"
    return 0
  fi

  {
    git -C "$BBS_PROJECT_ROOT" rev-parse HEAD 2>/dev/null || printf 'no-head\n'
    bbs_gated_file_list | while IFS= read -r bbs_file; do
      sha256sum "$BBS_PROJECT_ROOT/$bbs_file"
    done
  } | sha256sum | awk '{print $1}'
}

bbs_gate_receipt_path() {
  local bbs_git_dir
  bbs_git_dir=$(git -C "$BBS_PROJECT_ROOT" rev-parse --git-dir 2>/dev/null) || return 1

  case "$bbs_git_dir" in
    /*) printf '%s/omarchy-bbs-gate-receipt\n' "$bbs_git_dir" ;;
    *) printf '%s/%s/omarchy-bbs-gate-receipt\n' "$BBS_PROJECT_ROOT" "$bbs_git_dir" ;;
  esac
}

#!/usr/bin/env bash
set -uo pipefail

bbs_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
bbs_mode="full"
bbs_failures=0

case "${1:-}" in
  "") ;;
  --full) bbs_mode="full" ;;
  --diff) bbs_mode="diff" ;;
  --fast) bbs_mode="fast" ;;
  *)
    echo "Usage: $0 [--fast|--diff|--full]" >&2
    exit 2
    ;;
esac

cd "$bbs_root"

# shellcheck source=bin/lib-gate.sh
source "$bbs_root/bin/lib-gate.sh"

run_gate() {
  local bbs_number="$1"
  local bbs_name="$2"
  shift 2

  printf '\n[%s] %s\n' "$bbs_number" "$bbs_name"
  if "$@"; then
    printf '[%s] PASS\n' "$bbs_number"
  else
    printf '[%s] FAIL\n' "$bbs_number" >&2
    bbs_failures=$((bbs_failures + 1))
  fi
}

check_shell_syntax() {
  local bbs_script

  while IFS= read -r bbs_script; do
    bash -n "$bbs_script" || return 1
  done < <(
    find bin scripts .claude/hooks -type f -name '*.sh' -print | LC_ALL=C sort
  )
}

check_changed_secrets() {
  printf '{}\n' | .claude/hooks/enforce-secrets.sh
}

check_whitespace() {
  local bbs_file
  local bbs_output
  local bbs_whitespace_failures=0

  git diff --check || bbs_whitespace_failures=1
  git diff --cached --check || bbs_whitespace_failures=1

  while IFS= read -r bbs_file; do
    [[ -f "$bbs_file" ]] || continue
    bbs_output=$(git diff --no-index --check -- /dev/null "$bbs_file" 2>&1 || true)
    if [[ -n "$bbs_output" ]]; then
      printf '%s\n' "$bbs_output" >&2
      bbs_whitespace_failures=1
    fi
  done < <(git ls-files --others --exclude-standard)

  ((bbs_whitespace_failures == 0))
}

run_gate 1 "rustfmt" cargo fmt --all --check
run_gate 2 "clippy" cargo clippy --workspace --all-targets -- -D warnings
run_gate 3 "tests" cargo test --workspace
run_gate 4 "rustdoc" env RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps
run_gate 5 "compose" docker compose config --quiet
run_gate 6 "shell syntax" check_shell_syntax
run_gate 7 "pipeline structure" ./scripts/check-pipeline.sh
run_gate 8 "changed-file secret scan" check_changed_secrets
run_gate 9 "Claude hook self-tests" ./scripts/selftest-hooks.sh
run_gate 10 "whitespace errors" check_whitespace

if [[ "$bbs_mode" != "fast" ]]; then
  run_gate 11 "PostgreSQL + Rust API + QML smoke" ./scripts/dev.sh --smoke-test
fi

if ((bbs_failures > 0)); then
  printf '\nGATE RED [%s] — %d check(s) failed\n' "$bbs_mode" "$bbs_failures" >&2
  exit 1
fi

if [[ "$bbs_mode" != "fast" ]]; then
  bbs_receipt=$(bbs_gate_receipt_path) || {
    echo "GATE RED [$bbs_mode] — could not resolve receipt path" >&2
    exit 1
  }
  bbs_gate_state_hash >"$bbs_receipt"
fi

printf '\nGATE GREEN [%s]\n' "$bbs_mode"

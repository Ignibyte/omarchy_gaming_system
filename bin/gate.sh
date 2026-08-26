#!/usr/bin/env bash
set -uo pipefail

ogs_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
ogs_mode="full"
ogs_failures=0

case "${1:-}" in
  "") ;;
  --full) ogs_mode="full" ;;
  --diff) ogs_mode="diff" ;;
  --fast) ogs_mode="fast" ;;
  *)
    echo "Usage: $0 [--fast|--diff|--full]" >&2
    exit 2
    ;;
esac

cd "$ogs_root"

# shellcheck source=bin/lib-gate.sh
source "$ogs_root/bin/lib-gate.sh"

run_gate() {
  local ogs_number="$1"
  local ogs_name="$2"
  shift 2

  printf '\n[%s] %s\n' "$ogs_number" "$ogs_name"
  if "$@"; then
    printf '[%s] PASS\n' "$ogs_number"
  else
    printf '[%s] FAIL\n' "$ogs_number" >&2
    ogs_failures=$((ogs_failures + 1))
  fi
}

check_shell_syntax() {
  local ogs_script

  while IFS= read -r ogs_script; do
    bash -n "$ogs_script" || return 1
  done < <(
    find bin scripts .codex/hooks -type f -name '*.sh' -print | LC_ALL=C sort
  )
  bash -n packaging/arch/omarchygs
}

check_changed_secrets() {
  printf '{}\n' | .codex/hooks/enforce-secrets.sh
}

check_whitespace() {
  local ogs_file
  local ogs_output
  local ogs_whitespace_failures=0

  git diff --check || ogs_whitespace_failures=1
  git diff --cached --check || ogs_whitespace_failures=1

  while IFS= read -r ogs_file; do
    [[ -f "$ogs_file" ]] || continue
    ogs_output=$(git diff --no-index --check -- /dev/null "$ogs_file" 2>&1 || true)
    if [[ -n "$ogs_output" ]]; then
      printf '%s\n' "$ogs_output" >&2
      ogs_whitespace_failures=1
    fi
  done < <(git ls-files --others --exclude-standard)

  ((ogs_whitespace_failures == 0))
}

run_gate 1 "rustfmt" cargo fmt --all --check
run_gate 2 "clippy" cargo clippy --workspace --all-targets -- -D warnings
run_gate 3 "tests" cargo test --workspace
run_gate 4 "rustdoc" env RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps
run_gate 5 "compose" docker compose config --quiet
run_gate 6 "shell syntax" check_shell_syntax
run_gate 7 "pipeline structure" ./scripts/check-pipeline.sh
run_gate 8 "changed-file secret scan" check_changed_secrets
run_gate 9 "Codex hook self-tests" ./scripts/selftest-hooks.sh
run_gate 10 "whitespace errors" check_whitespace
run_gate 11 "production Game Cartridge contract" ./scripts/test-game-cartridge.sh
run_gate 12 "trusted Game Cartridge renderer" ./scripts/test-game-cartridge-renderer.sh
run_gate 13 "Game Cartridge SDK release" ./scripts/test-game-cartridge-sdk.sh
run_gate 14 "Game Cartridge architecture proof" ./scripts/test-game-cartridge-spike.sh
run_gate 15 "native Omarchy client package source" ./scripts/check-client-package-source.sh

if [[ "$ogs_mode" != "fast" ]]; then
  run_gate 16 "native Omarchy client package" ./scripts/test-client-package.sh
  run_gate 17 "PostgreSQL integration tests" ./scripts/test-database.sh
  run_gate 18 "PostgreSQL + Rust API + QML smoke" ./scripts/dev.sh --smoke-test
  run_gate 19 "remote provider security conformance" ./scripts/test-provider-conformance.sh
  run_gate 20 "first-party remote-provider authority pilot" ./scripts/test-provider-authority-pilot.sh
  run_gate 21 "platform operator backup and restore drill" ./scripts/test-operator-recovery.sh
fi

if ((ogs_failures > 0)); then
  printf '\nGATE RED [%s] — %d check(s) failed\n' "$ogs_mode" "$ogs_failures" >&2
  exit 1
fi

if [[ "$ogs_mode" != "fast" ]]; then
  ogs_receipt=$(ogs_gate_receipt_path) || {
    echo "GATE RED [$ogs_mode] — could not resolve receipt path" >&2
    exit 1
  }
  ogs_gate_state_hash >"$ogs_receipt"
fi

printf '\nGATE GREEN [%s]\n' "$ogs_mode"

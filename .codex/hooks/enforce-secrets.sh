#!/usr/bin/env bash
set -euo pipefail

cat >/dev/null 2>&1 || true

# shellcheck source=.codex/hooks/lib-hook-helpers.sh
source "$(dirname "$0")/lib-hook-helpers.sh"

cd "$PROJECT_ROOT"
ogs_secret_re='gho_[A-Za-z0-9]{20,}|github_pat_[A-Za-z0-9_]{20,}|sk-(proj|svcacct)-[A-Za-z0-9_-]{20,}|sk-ant-[A-Za-z0-9_-]{20,}|AKIA[0-9A-Z]{16}|xox[baprs]-[A-Za-z0-9-]{10,}|-----BEGIN[A-Z ]*PRIVATE KEY-----'
ogs_hits=()

while IFS= read -r -d '' ogs_file; do
  [[ -f "$ogs_file" && ! -L "$ogs_file" ]] || continue
  [[ "$ogs_file" == ".codex/hooks/enforce-secrets.sh" ]] && continue
  if grep -aEq "$ogs_secret_re" -- "$ogs_file" 2>/dev/null; then
    ogs_hits+=("$ogs_file")
  fi
done < <(
  {
    git diff --name-only -z HEAD 2>/dev/null
    git ls-files -z --others --exclude-standard 2>/dev/null
  } | LC_ALL=C sort -zu
)

if ((${#ogs_hits[@]} > 0)); then
  printf '\nSECRETS BLOCKED — high-signal credential material appears in:\n' >&2
  printf '  %q\n' "${ogs_hits[@]}" >&2
  printf 'Remove it and use a gitignored local secret store (§14).\n' >&2
  exit 2
fi

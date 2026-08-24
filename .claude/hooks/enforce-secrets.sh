#!/usr/bin/env bash
set -euo pipefail

cat >/dev/null 2>&1 || true

# shellcheck source=.claude/hooks/lib-hook-helpers.sh
source "$(dirname "$0")/lib-hook-helpers.sh"

cd "$PROJECT_ROOT"
bbs_changed=$(
  {
    git diff --name-only HEAD 2>/dev/null
    git ls-files --others --exclude-standard 2>/dev/null
  } | LC_ALL=C sort -u
)
[[ -n "$bbs_changed" ]] || exit 0

bbs_secret_re='gho_[A-Za-z0-9]{20,}|github_pat_[A-Za-z0-9_]{20,}|sk-ant-[A-Za-z0-9_-]{20,}|AKIA[0-9A-Z]{16}|xox[baprs]-[A-Za-z0-9-]{10,}|-----BEGIN[A-Z ]*PRIVATE KEY-----'
bbs_hits=""

while IFS= read -r bbs_file; do
  [[ -f "$bbs_file" && ! -L "$bbs_file" ]] || continue
  [[ "$bbs_file" == ".claude/hooks/enforce-secrets.sh" ]] && continue
  if grep -aEq "$bbs_secret_re" "$bbs_file" 2>/dev/null; then
    bbs_hits+=" $bbs_file"
  fi
done <<<"$bbs_changed"

if [[ -n "$bbs_hits" ]]; then
  printf '\nSECRETS BLOCKED — high-signal credential material appears in:%s\nRemove it and use a gitignored local secret store (§14).\n' \
    "$bbs_hits" >&2
  exit 2
fi

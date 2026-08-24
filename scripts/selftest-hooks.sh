#!/usr/bin/env bash
set -Eeuo pipefail

bbs_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
bbs_sandbox=$(mktemp -d)

cleanup() {
  [[ -n "$bbs_sandbox" && "$bbs_sandbox" == /tmp/* ]] \
    && rm -rf -- "$bbs_sandbox"
}
trap cleanup EXIT

fail() {
  echo "Hook self-test failed: $1" >&2
  exit 1
}

expect_hook_exit() {
  local bbs_expected="$1"
  local bbs_hook="$2"
  local bbs_input="$3"
  local bbs_actual=0

  printf '%s\n' "$bbs_input" | bash "$bbs_hook" >/dev/null 2>&1 \
    || bbs_actual=$?
  [[ "$bbs_actual" == "$bbs_expected" ]] \
    || fail "$(basename "$bbs_hook") expected $bbs_expected, got $bbs_actual"
}

mkdir -p \
  "$bbs_sandbox/.claude/hooks" \
  "$bbs_sandbox/bin" \
  "$bbs_sandbox/crates" \
  "$bbs_sandbox/docs/architecture" \
  "$bbs_sandbox/docs/planning/knowledge/aar" \
  "$bbs_sandbox/docs/planning/pipeline/active"

cp "$bbs_root"/.claude/hooks/*.sh "$bbs_sandbox/.claude/hooks/"
cp "$bbs_root/bin/lib-gate.sh" "$bbs_sandbox/bin/"
printf 'fn main() {}\n' >"$bbs_sandbox/crates/app.rs"

git -C "$bbs_sandbox" init -q
git -C "$bbs_sandbox" config user.name "Hook Self-Test"
git -C "$bbs_sandbox" config user.email "hooks@example.invalid"
git -C "$bbs_sandbox" add .
git -C "$bbs_sandbox" commit -qm "fixture"

printf 'fn main() { println!("changed"); }\n' >"$bbs_sandbox/crates/app.rs"
bbs_commit_input='{"tool_input":{"command":"git commit -m test"}}'
(
  cd "$bbs_sandbox"
  expect_hook_exit 2 \
    "$bbs_sandbox/.claude/hooks/enforce-commit-gate.sh" \
    "$bbs_commit_input"
)

(
  cd "$bbs_sandbox"
  BBS_PROJECT_ROOT="$bbs_sandbox"
  # shellcheck source=bin/lib-gate.sh
  source "$bbs_sandbox/bin/lib-gate.sh"
  bbs_receipt=$(bbs_gate_receipt_path)
  bbs_gate_state_hash >"$bbs_receipt"
)
(
  cd "$bbs_sandbox"
  expect_hook_exit 0 \
    "$bbs_sandbox/.claude/hooks/enforce-commit-gate.sh" \
    "$bbs_commit_input"
)

git -C "$bbs_sandbox" add crates/app.rs
(
  cd "$bbs_sandbox"
  expect_hook_exit 0 \
    "$bbs_sandbox/.claude/hooks/enforce-commit-gate.sh" \
    "$bbs_commit_input"
)

printf 'fn main() { println!("stale"); }\n' >"$bbs_sandbox/crates/app.rs"
(
  cd "$bbs_sandbox"
  expect_hook_exit 2 \
    "$bbs_sandbox/.claude/hooks/enforce-commit-gate.sh" \
    "$bbs_commit_input"
)

cat >"$bbs_sandbox/docs/planning/pipeline/active/test.spec.md" <<'EOF'
---
status: Phase 1 — Plan PASS; ready for Phase 2 — Design
---
EOF
cat >"$bbs_sandbox/design.jsonl" <<'EOF'
{"message":{"role":"user","content":"/pipeline:design test"}}
EOF
bbs_edit_input=$(printf '{"transcript_path":"%s","tool_input":{"file_path":"%s/crates/app.rs"}}' \
  "$bbs_sandbox/design.jsonl" "$bbs_sandbox")
(
  cd "$bbs_sandbox"
  expect_hook_exit 2 \
    "$bbs_sandbox/.claude/hooks/enforce-phase-gate.sh" \
    "$bbs_edit_input"
)

cat >"$bbs_sandbox/docs/planning/pipeline/active/test.spec.md" <<'EOF'
---
status: Phase 2 — Design PASS; ready for Phase 3 — Implement
---
EOF
cat >"$bbs_sandbox/implement.jsonl" <<'EOF'
{"message":{"role":"user","content":"/pipeline:implement test"}}
{"message":{"role":"assistant","content":[{"type":"tool_use","name":"Read","input":{"file_path":"docs/planning/knowledge/INDEX.md"}},{"type":"tool_use","name":"Read","input":{"file_path":"docs/architecture/system-overview.md"}}]}}
EOF
bbs_edit_input=$(printf '{"transcript_path":"%s","tool_input":{"file_path":"%s/crates/app.rs"}}' \
  "$bbs_sandbox/implement.jsonl" "$bbs_sandbox")
(
  cd "$bbs_sandbox"
  expect_hook_exit 0 \
    "$bbs_sandbox/.claude/hooks/enforce-phase-gate.sh" \
    "$bbs_edit_input"
  expect_hook_exit 0 \
    "$bbs_sandbox/.claude/hooks/enforce-docs-before-code.sh" \
    "$bbs_edit_input"
)

cat >"$bbs_sandbox/validate.jsonl" <<'EOF'
{"message":{"role":"user","content":"/pipeline:validate test"}}
EOF
bbs_stop_input=$(printf '{"transcript_path":"%s"}' "$bbs_sandbox/validate.jsonl")
(
  cd "$bbs_sandbox"
  expect_hook_exit 2 \
    "$bbs_sandbox/.claude/hooks/enforce-tests-ran.sh" \
    "$bbs_stop_input"
)
cat >>"$bbs_sandbox/validate.jsonl" <<'EOF'
{"message":{"role":"assistant","content":[{"type":"tool_use","name":"Bash","input":{"command":"cargo test --workspace"}}]}}
EOF
(
  cd "$bbs_sandbox"
  expect_hook_exit 0 \
    "$bbs_sandbox/.claude/hooks/enforce-tests-ran.sh" \
    "$bbs_stop_input"
)

cat >"$bbs_sandbox/tasks.jsonl" <<'EOF'
{"message":{"role":"user","content":"/pipeline:implement test"}}
{"message":{"role":"assistant","content":[{"type":"tool_use","name":"TaskCreate","input":{"subject":"implement fixture"}}]}}
EOF
bbs_stop_input=$(printf '{"transcript_path":"%s"}' "$bbs_sandbox/tasks.jsonl")
(
  cd "$bbs_sandbox"
  expect_hook_exit 2 \
    "$bbs_sandbox/.claude/hooks/enforce-phase-tasks.sh" \
    "$bbs_stop_input"
)
cat >>"$bbs_sandbox/tasks.jsonl" <<'EOF'
{"message":{"role":"assistant","content":[{"type":"tool_use","name":"TaskUpdate","input":{"taskId":"1","status":"completed"}}]}}
EOF
(
  cd "$bbs_sandbox"
  expect_hook_exit 0 \
    "$bbs_sandbox/.claude/hooks/enforce-phase-tasks.sh" \
    "$bbs_stop_input"
)

cat >"$bbs_sandbox/docs/planning/knowledge/aar/AAR-001-test.md" <<'EOF'
---
pipeline: test
status: open
---
EOF
cat >"$bbs_sandbox/complete.jsonl" <<'EOF'
{"message":{"role":"user","content":"/pipeline:complete test"}}
EOF
bbs_stop_input=$(printf '{"transcript_path":"%s"}' "$bbs_sandbox/complete.jsonl")
(
  cd "$bbs_sandbox"
  expect_hook_exit 2 \
    "$bbs_sandbox/.claude/hooks/enforce-completion.sh" \
    "$bbs_stop_input"
)
sed -i 's/status: open/status: submitted/' \
  "$bbs_sandbox/docs/planning/knowledge/aar/AAR-001-test.md"
(
  cd "$bbs_sandbox"
  expect_hook_exit 0 \
    "$bbs_sandbox/.claude/hooks/enforce-completion.sh" \
    "$bbs_stop_input"
)

printf 'gho_%s\n' '12345678901234567890' >"$bbs_sandbox/secret.txt"
(
  cd "$bbs_sandbox"
  expect_hook_exit 2 \
    "$bbs_sandbox/.claude/hooks/enforce-secrets.sh" \
    '{}'
)
rm -f "$bbs_sandbox/secret.txt"
(
  cd "$bbs_sandbox"
  expect_hook_exit 0 \
    "$bbs_sandbox/.claude/hooks/enforce-secrets.sh" \
    '{}'
)

echo "Hook self-tests passed"

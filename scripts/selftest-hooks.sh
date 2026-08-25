#!/usr/bin/env bash
set -Eeuo pipefail

ogs_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
ogs_sandbox=$(mktemp -d)

cleanup() {
  [[ -n "$ogs_sandbox" && "$ogs_sandbox" == /tmp/* ]] \
    && rm -rf -- "$ogs_sandbox"
}
trap cleanup EXIT

fail() {
  echo "Hook self-test failed: $1" >&2
  exit 1
}

expect_hook_exit() {
  local ogs_expected="$1"
  local ogs_hook="$2"
  local ogs_input="$3"
  local ogs_actual=0

  printf '%s\n' "$ogs_input" | bash "$ogs_hook" >/dev/null 2>&1 \
    || ogs_actual=$?
  [[ "$ogs_actual" == "$ogs_expected" ]] \
    || fail "$(basename "$ogs_hook") expected $ogs_expected, got $ogs_actual"
}

mkdir -p \
  "$ogs_sandbox/.codex/hooks" \
  "$ogs_sandbox/bin" \
  "$ogs_sandbox/crates" \
  "$ogs_sandbox/docs/planning/pipeline/active" \
  "$ogs_sandbox/docs/planning/pipeline/completed"

cp "$ogs_root"/.codex/hooks/*.sh "$ogs_sandbox/.codex/hooks/"
cp "$ogs_root/bin/lib-gate.sh" "$ogs_sandbox/bin/"
printf 'fn main() {}\n' >"$ogs_sandbox/crates/app.rs"

git -C "$ogs_sandbox" init -q
git -C "$ogs_sandbox" config user.name "Hook Self-Test"
git -C "$ogs_sandbox" config user.email "hooks@example.invalid"
git -C "$ogs_sandbox" add .
git -C "$ogs_sandbox" commit -qm "fixture"

printf 'fn main() { println!("changed"); }\n' >"$ogs_sandbox/crates/app.rs"
ogs_commit_input='{"tool_input":{"command":"git commit -m test"}}'
(
  cd "$ogs_sandbox"
  expect_hook_exit 2 \
    "$ogs_sandbox/.codex/hooks/enforce-commit-gate.sh" \
    "$ogs_commit_input"
)

write_receipt() {
  (
    cd "$ogs_sandbox"
    OGS_PROJECT_ROOT="$ogs_sandbox"
    # shellcheck source=bin/lib-gate.sh
    source "$ogs_sandbox/bin/lib-gate.sh"
    ogs_receipt=$(ogs_gate_receipt_path)
    ogs_gate_state_hash >"$ogs_receipt"
  )
}

write_receipt
(
  cd "$ogs_sandbox"
  expect_hook_exit 0 \
    "$ogs_sandbox/.codex/hooks/enforce-commit-gate.sh" \
    "$ogs_commit_input"
)

git -C "$ogs_sandbox" add crates/app.rs
(
  cd "$ogs_sandbox"
  expect_hook_exit 0 \
    "$ogs_sandbox/.codex/hooks/enforce-commit-gate.sh" \
    "$ogs_commit_input"
)

printf 'fn main() { println!("stale"); }\n' >"$ogs_sandbox/crates/app.rs"
(
  cd "$ogs_sandbox"
  expect_hook_exit 2 \
    "$ogs_sandbox/.codex/hooks/enforce-commit-gate.sh" \
    "$ogs_commit_input"
)

ogs_newline_path=$'crates/newline\ncommand.rs'
printf 'fn newline_path() {}\n' >"$ogs_sandbox/$ogs_newline_path"
write_receipt
printf 'fn newline_path() { println!("changed"); }\n' \
  >"$ogs_sandbox/$ogs_newline_path"
(
  cd "$ogs_sandbox"
  expect_hook_exit 2 \
    "$ogs_sandbox/.codex/hooks/enforce-commit-gate.sh" \
    "$ogs_commit_input"
)
rm -f -- "$ogs_sandbox/$ogs_newline_path"

printf '%s\n' \
  '---' \
  'pipeline_id: 11111111-1111-4111-8111-111111111111' \
  'status: Phase 1 — Plan PASS; ready for Phase 2 — Design' \
  '---' \
  >"$ogs_sandbox/docs/planning/pipeline/active/test.spec.md"
for ogs_nonmutating_commit in \
  'git commit --help' \
  'git commit -h' \
  'git commit --dry-run'; do
  ogs_nonmutating_input=$(jq -cn --arg ogs_command "$ogs_nonmutating_commit" \
    '{tool_input:{command:$ogs_command}}')
  (
    cd "$ogs_sandbox"
    expect_hook_exit 0 \
      "$ogs_sandbox/.codex/hooks/enforce-commit-gate.sh" \
      "$ogs_nonmutating_input"
  )
done
for ogs_compound_commit in \
  'true --help ; git commit -m test' \
  'git commit --dry-run ; git commit -m test' \
  $'true -h\ngit commit -m test'; do
  ogs_compound_input=$(jq -cn --arg ogs_command "$ogs_compound_commit" \
    '{tool_input:{command:$ogs_command}}')
  (
    cd "$ogs_sandbox"
    expect_hook_exit 2 \
      "$ogs_sandbox/.codex/hooks/enforce-commit-gate.sh" \
      "$ogs_compound_input"
  )
done
ogs_patch=$'*** Begin Patch\n*** Update File: crates/app.rs\n@@\n-old\n+new\n*** End Patch'
ogs_edit_input=$(jq -cn --arg ogs_patch "$ogs_patch" \
  '{tool_input:{command:$ogs_patch}}')
(
  cd "$ogs_sandbox"
  expect_hook_exit 2 \
    "$ogs_sandbox/.codex/hooks/enforce-phase-gate.sh" \
    "$ogs_edit_input"
)

ogs_traversal_patch=$'*** Begin Patch\n*** Update File: docs/../crates/app.rs\n@@\n-old\n+new\n*** End Patch'
ogs_traversal_input=$(jq -cn --arg ogs_patch "$ogs_traversal_patch" \
  '{tool_input:{command:$ogs_patch}}')
ogs_absolute_patch=$(printf '%s\n' \
  '*** Begin Patch' \
  "*** Update File: $ogs_sandbox/docs/../crates/app.rs" \
  '@@' \
  '-old' \
  '+new' \
  '*** End Patch')
ogs_absolute_input=$(jq -cn --arg ogs_patch "$ogs_absolute_patch" \
  '{tool_input:{command:$ogs_patch}}')
ln -s ../crates "$ogs_sandbox/docs/crates-link"
ogs_symlink_patch=$'*** Begin Patch\n*** Update File: docs/crates-link/app.rs\n@@\n-old\n+new\n*** End Patch'
ogs_symlink_input=$(jq -cn --arg ogs_patch "$ogs_symlink_patch" \
  '{tool_input:{command:$ogs_patch}}')
for ogs_alias_input in \
  "$ogs_traversal_input" \
  "$ogs_absolute_input" \
  "$ogs_symlink_input"; do
  (
    cd "$ogs_sandbox"
    expect_hook_exit 2 \
      "$ogs_sandbox/.codex/hooks/enforce-phase-gate.sh" \
      "$ogs_alias_input"
  )
done

ogs_docs_patch=$'*** Begin Patch\n*** Add File: docs/guide.md\n+documentation only\n*** End Patch'
ogs_docs_input=$(jq -cn --arg ogs_patch "$ogs_docs_patch" \
  '{tool_input:{command:$ogs_patch}}')
(
  cd "$ogs_sandbox"
  expect_hook_exit 0 \
    "$ogs_sandbox/.codex/hooks/enforce-phase-gate.sh" \
    "$ogs_docs_input"
)

ogs_stop_input='{"stop_hook_active":false,"last_assistant_message":"Phase 2 PASS"}'
(
  cd "$ogs_sandbox"
  expect_hook_exit 2 \
    "$ogs_sandbox/.codex/hooks/enforce-stop-claims.sh" \
    "$ogs_stop_input"
  expect_hook_exit 2 \
    "$ogs_sandbox/.codex/hooks/enforce-stop-claims.sh" \
    '{"stop_hook_active":true,"last_assistant_message":"Phase 3.5 PASS"}'
)

ogs_codegraph_error='{"tool_name":"mcp__codegraph__codegraph_explore","tool_response":{"isError":true}}'
(
  cd "$ogs_sandbox"
  expect_hook_exit 0 \
    "$ogs_sandbox/.codex/hooks/record-pipeline-tool-use.sh" \
    "$ogs_codegraph_error"
  expect_hook_exit 2 \
    "$ogs_sandbox/.codex/hooks/enforce-stop-claims.sh" \
    "$ogs_stop_input"
)

ogs_codegraph_success='{"tool_name":"mcp__codegraph__codegraph_explore","tool_response":{"isError":false,"content":[{"type":"text","text":"current source and blast radius"}]}}'
(
  cd "$ogs_sandbox"
  expect_hook_exit 0 \
    "$ogs_sandbox/.codex/hooks/record-pipeline-tool-use.sh" \
    "$ogs_codegraph_success"
  expect_hook_exit 0 \
    "$ogs_sandbox/.codex/hooks/enforce-stop-claims.sh" \
    "$ogs_stop_input"
)

write_receipt
(
  cd "$ogs_sandbox"
  expect_hook_exit 2 \
    "$ogs_sandbox/.codex/hooks/enforce-commit-gate.sh" \
    "$ogs_commit_input"
)

sed -i \
  's/Phase 1 — Plan PASS; ready for Phase 2 — Design/Phase 2 — Design PASS; ready for Phase 3 — Implement/' \
  "$ogs_sandbox/docs/planning/pipeline/active/test.spec.md"
(
  cd "$ogs_sandbox"
  expect_hook_exit 0 \
    "$ogs_sandbox/.codex/hooks/enforce-phase-gate.sh" \
    "$ogs_edit_input"
)

printf 'fn main() { println!("validation changed"); }\n' \
  >"$ogs_sandbox/crates/app.rs"
sed -i \
  's/Phase 2 — Design PASS; ready for Phase 3 — Implement/Phase 3 — Implement PASS; ready for Phase 3.5 — Inspect/' \
  "$ogs_sandbox/docs/planning/pipeline/active/test.spec.md"
ogs_stop_input='{"stop_hook_active":false,"last_assistant_message":"Phase 3.5 PASS"}'
(
  cd "$ogs_sandbox"
  expect_hook_exit 2 \
    "$ogs_sandbox/.codex/hooks/enforce-stop-claims.sh" \
    "$ogs_stop_input"
  expect_hook_exit 0 \
    "$ogs_sandbox/.codex/hooks/record-pipeline-tool-use.sh" \
    '{"tool_name":"Bash","tool_input":{"command":"echo codegraph explore"},"tool_response":{"exit_code":0}}'
  expect_hook_exit 2 \
    "$ogs_sandbox/.codex/hooks/enforce-stop-claims.sh" \
    "$ogs_stop_input"
  expect_hook_exit 0 \
    "$ogs_sandbox/.codex/hooks/record-pipeline-tool-use.sh" \
    "$ogs_codegraph_success"
  expect_hook_exit 0 \
    "$ogs_sandbox/.codex/hooks/enforce-stop-claims.sh" \
    "$ogs_stop_input"
)

ogs_stop_input='{"stop_hook_active":false,"last_assistant_message":"Phase 4 PASS"}'
(
  cd "$ogs_sandbox"
  expect_hook_exit 2 \
    "$ogs_sandbox/.codex/hooks/enforce-stop-claims.sh" \
    "$ogs_stop_input"
)

write_receipt
(
  cd "$ogs_sandbox"
  expect_hook_exit 0 \
    "$ogs_sandbox/.codex/hooks/enforce-stop-claims.sh" \
    "$ogs_stop_input"
)

sed -i \
  's/Phase 3 — Implement PASS; ready for Phase 3.5 — Inspect/Phase 4 — Validate PASS; ready for Phase 5 — Complete/' \
  "$ogs_sandbox/docs/planning/pipeline/active/test.spec.md"
ogs_stop_input='{"stop_hook_active":false,"last_assistant_message":"Phase 5 PASS; pipeline complete"}'
(
  cd "$ogs_sandbox"
  expect_hook_exit 2 \
    "$ogs_sandbox/.codex/hooks/enforce-stop-claims.sh" \
    "$ogs_stop_input"
)

ogs_openwiki_error='{"tool_name":"mcp__openwiki__openwiki_finish","tool_response":{"isError":true,"content":[{"type":"text","text":"validation failed"}]}}'
(
  cd "$ogs_sandbox"
  expect_hook_exit 0 \
    "$ogs_sandbox/.codex/hooks/record-pipeline-tool-use.sh" \
    "$ogs_openwiki_error"
  expect_hook_exit 2 \
    "$ogs_sandbox/.codex/hooks/enforce-stop-claims.sh" \
    "$ogs_stop_input"
)

ogs_openwiki_success='{"tool_name":"mcp__openwiki__openwiki_finish","tool_response":{"isError":false,"structuredContent":{"status":"complete"},"content":[{"type":"text","text":"complete"}]}}'
(
  cd "$ogs_sandbox"
  expect_hook_exit 0 \
    "$ogs_sandbox/.codex/hooks/record-pipeline-tool-use.sh" \
    "$ogs_openwiki_success"
  expect_hook_exit 2 \
    "$ogs_sandbox/.codex/hooks/enforce-stop-claims.sh" \
    "$ogs_stop_input"
)

sed \
  's/Phase 4 — Validate PASS; ready for Phase 5 — Complete/Phase 5 — Complete PASS/' \
  "$ogs_sandbox/docs/planning/pipeline/active/test.spec.md" \
  >"$ogs_sandbox/docs/planning/pipeline/completed/test.spec.md"
rm -f "$ogs_sandbox/docs/planning/pipeline/active/test.spec.md"
(
  cd "$ogs_sandbox"
  expect_hook_exit 0 \
    "$ogs_sandbox/.codex/hooks/enforce-stop-claims.sh" \
    "$ogs_stop_input"
)

printf 'gho_%s\n' '12345678901234567890' >"$ogs_sandbox/secret.txt"
(
  cd "$ogs_sandbox"
  expect_hook_exit 2 \
    "$ogs_sandbox/.codex/hooks/enforce-secrets.sh" \
    '{}'
)
rm -f "$ogs_sandbox/secret.txt"
(
  cd "$ogs_sandbox"
  expect_hook_exit 0 \
    "$ogs_sandbox/.codex/hooks/enforce-secrets.sh" \
    '{}'
)

printf 'sk-proj-%s\n' '12345678901234567890' \
  >"$ogs_sandbox/openai-project-key.txt"
(
  cd "$ogs_sandbox"
  expect_hook_exit 2 \
    "$ogs_sandbox/.codex/hooks/enforce-secrets.sh" \
    '{}'
)
rm -f "$ogs_sandbox/openai-project-key.txt"

printf 'sk-svcacct-%s\n' '12345678901234567890' \
  >"$ogs_sandbox/openai-service-account-key.txt"
(
  cd "$ogs_sandbox"
  expect_hook_exit 2 \
    "$ogs_sandbox/.codex/hooks/enforce-secrets.sh" \
    '{}'
)
rm -f "$ogs_sandbox/openai-service-account-key.txt"

printf 'gho_%s\n' '12345678901234567890' >"$ogs_sandbox/-q"
(
  cd "$ogs_sandbox"
  expect_hook_exit 2 \
    "$ogs_sandbox/.codex/hooks/enforce-secrets.sh" \
    '{}'
)
rm -f "$ogs_sandbox/-q"

ogs_newline_secret=$'secret\nkey.txt'
printf 'sk-proj-%s\n' '12345678901234567890' \
  >"$ogs_sandbox/$ogs_newline_secret"
(
  cd "$ogs_sandbox"
  expect_hook_exit 2 \
    "$ogs_sandbox/.codex/hooks/enforce-secrets.sh" \
    '{}'
)
rm -f -- "$ogs_sandbox/$ogs_newline_secret"

printf 'gho_%s\n' '12345678901234567890' \
  >"$ogs_sandbox/secret key.txt"
(
  cd "$ogs_sandbox"
  expect_hook_exit 2 \
    "$ogs_sandbox/.codex/hooks/enforce-secrets.sh" \
    '{}'
)
rm -f -- "$ogs_sandbox/secret key.txt"

printf 'sk-proj-too-short\n' >"$ogs_sandbox/near-miss.txt"
(
  cd "$ogs_sandbox"
  expect_hook_exit 0 \
    "$ogs_sandbox/.codex/hooks/enforce-secrets.sh" \
    '{}'
)

echo "Hook self-tests passed"

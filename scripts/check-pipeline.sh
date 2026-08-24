#!/usr/bin/env bash
set -Eeuo pipefail

bbs_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$bbs_root"

required_files=(
  CLAUDE.md
  CONSTITUTION.md
  .claude/settings.json
  .claude/commands/work.md
  .claude/commands/brainstorm.md
  .claude/commands/commit.md
  .claude/commands/pipeline/plan.md
  .claude/commands/pipeline/design.md
  .claude/commands/pipeline/implement.md
  .claude/commands/pipeline/inspect.md
  .claude/commands/pipeline/validate.md
  .claude/commands/pipeline/complete.md
  docs/planning/tickets/INDEX.md
  docs/planning/knowledge/INDEX.md
  docs/planning/knowledge/aar/TEMPLATE.md
  docs/planning/bulletins/INDEX.md
  docs/planning/pipeline/_templates/spec.md
  docs/planning/pipeline/_templates/notes.md
  docs/planning/_templates/ticket.md
  docs/planning/_templates/intake.md
)

for bbs_file in "${required_files[@]}"; do
  [[ -f "$bbs_file" ]] || {
    echo "Pipeline check failed: missing $bbs_file" >&2
    exit 1
  }
done

jq -e '.hooks.PreToolUse and .hooks.Stop' .claude/settings.json >/dev/null

for bbs_hook in \
  enforce-commit-gate.sh \
  enforce-phase-gate.sh \
  enforce-docs-before-code.sh \
  enforce-phase-tasks.sh \
  enforce-tests-ran.sh \
  enforce-completion.sh \
  enforce-secrets.sh; do
  [[ -x ".claude/hooks/$bbs_hook" ]] || {
    echo "Pipeline check failed: hook is missing or not executable: $bbs_hook" >&2
    exit 1
  }
  jq -e --arg bbs_hook "$bbs_hook" \
    '.. | strings | select(contains($bbs_hook))' \
    .claude/settings.json >/dev/null || {
      echo "Pipeline check failed: hook is not wired in settings: $bbs_hook" >&2
      exit 1
    }
done

bbs_active_count=$(find docs/planning/pipeline/active -maxdepth 1 \
  -name '*.spec.md' -print | wc -l)
if ((bbs_active_count > 1)); then
  echo "Pipeline check failed: more than one active spec" >&2
  exit 1
fi

for bbs_spec in docs/planning/pipeline/active/*.spec.md; do
  [[ -f "$bbs_spec" ]] || continue
  bbs_stem=${bbs_spec%.spec.md}
  [[ -f "$bbs_stem.notes.md" ]] || {
    echo "Pipeline check failed: missing notes pair for $bbs_spec" >&2
    exit 1
  }

  bbs_ticket=$(sed -n 's/^ticket_doc:[[:space:]]*//p' "$bbs_spec" | head -1)
  bbs_aar=$(sed -n 's/^aar:[[:space:]]*//p' "$bbs_spec" | head -1)
  [[ -n "$bbs_ticket" && -f "$bbs_ticket" ]] || {
    echo "Pipeline check failed: active spec ticket is missing: $bbs_ticket" >&2
    exit 1
  }
  [[ -n "$bbs_aar" && -f "$bbs_aar" ]] || {
    echo "Pipeline check failed: active spec AAR is missing: $bbs_aar" >&2
    exit 1
  }
done

for bbs_spec in docs/planning/pipeline/completed/*.spec.md; do
  [[ -f "$bbs_spec" ]] || continue
  bbs_stem=${bbs_spec%.spec.md}
  [[ -f "$bbs_stem.notes.md" ]] || {
    echo "Pipeline check failed: missing completed notes pair for $bbs_spec" >&2
    exit 1
  }
done

echo "Pipeline structure check passed"

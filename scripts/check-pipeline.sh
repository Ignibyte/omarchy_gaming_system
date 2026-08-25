#!/usr/bin/env bash
set -Eeuo pipefail

ogs_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ogs_root"

required_files=(
  AGENTS.md
  CONSTITUTION.md
  .codex/config.toml
  .codex/hooks.json
  .agents/skills/omarchy-workflow/SKILL.md
  .agents/skills/omarchy-workflow/references/phases.md
  .agents/skills/omarchy-brainstorm/SKILL.md
  .agents/skills/openwiki/SKILL.md
  scripts/codegraph.sh
  scripts/mcp-codegraph.sh
  scripts/mcp-openwiki.sh
  scripts/setup-pipeline-tools.sh
  scripts/check-pipeline-tools.sh
  docs/planning/tickets/INDEX.md
  docs/planning/knowledge/INDEX.md
  docs/planning/knowledge/aar/TEMPLATE.md
  docs/planning/bulletins/INDEX.md
  docs/planning/pipeline/_templates/spec.md
  docs/planning/pipeline/_templates/notes.md
  docs/planning/_templates/ticket.md
  docs/planning/_templates/intake.md
)

for ogs_file in "${required_files[@]}"; do
  [[ -f "$ogs_file" ]] || {
    echo "Pipeline check failed: missing $ogs_file" >&2
    exit 1
  }
done

jq -e '.hooks.PreToolUse and .hooks.PostToolUse and .hooks.Stop' \
  .codex/hooks.json >/dev/null

for ogs_hook in \
  enforce-commit-gate.sh \
  enforce-phase-gate.sh \
  record-pipeline-tool-use.sh \
  enforce-stop-claims.sh \
  enforce-secrets.sh; do
  [[ -x ".codex/hooks/$ogs_hook" ]] || {
    echo "Pipeline check failed: hook is missing or not executable: $ogs_hook" >&2
    exit 1
  }
  jq -e --arg ogs_hook "$ogs_hook" \
    '.. | strings | select(contains($ogs_hook))' \
    .codex/hooks.json >/dev/null || {
      echo "Pipeline check failed: hook is not wired in settings: $ogs_hook" >&2
      exit 1
    }
done

for ogs_script in \
  scripts/codegraph.sh \
  scripts/mcp-codegraph.sh \
  scripts/mcp-openwiki.sh \
  scripts/setup-pipeline-tools.sh \
  scripts/check-pipeline-tools.sh; do
  [[ -x "$ogs_script" ]] || {
    echo "Pipeline check failed: script is missing or not executable: $ogs_script" >&2
    exit 1
  }
done

grep -Fq '[mcp_servers.codegraph]' .codex/config.toml \
  && grep -Fq 'scripts/mcp-codegraph.sh' .codex/config.toml \
  && grep -Fq '[mcp_servers.openwiki]' .codex/config.toml \
  && grep -Fq 'scripts/mcp-openwiki.sh' .codex/config.toml || {
  echo "Pipeline check failed: Codex MCP wiring is incomplete" >&2
  exit 1
}

for ogs_skill in \
  .agents/skills/omarchy-workflow/SKILL.md \
  .agents/skills/omarchy-brainstorm/SKILL.md \
  .agents/skills/openwiki/SKILL.md; do
  sed -n '1,/^---$/p' "$ogs_skill" | grep -q '^name: ' || {
    echo "Pipeline check failed: skill name is missing: $ogs_skill" >&2
    exit 1
  }
  sed -n '1,/^---$/p' "$ogs_skill" | grep -q '^description: ' || {
    echo "Pipeline check failed: skill description is missing: $ogs_skill" >&2
    exit 1
  }
done

ogs_obsolete_agent=$(printf '%s%s' 'clau' 'de')
ogs_obsolete_doc=$(printf '%s%s.md' 'CLAU' 'DE')
ogs_obsolete_dir=$(printf '.%s%s' 'clau' 'de')
ogs_obsolete_pattern="${ogs_obsolete_agent}|[.]${ogs_obsolete_agent}"
if [[ -e "$ogs_obsolete_doc" ]] \
  || find "$ogs_obsolete_dir" -type f -print -quit 2>/dev/null | grep -q . \
  || grep -RIlE --exclude-dir=.git --exclude-dir=.dev \
    --exclude-dir=.codegraph --exclude-dir=target \
    "$ogs_obsolete_pattern" . >/dev/null 2>&1; then
  echo "Pipeline check failed: obsolete agent integration remains" >&2
  exit 1
fi

ogs_active_count=$(find docs/planning/pipeline/active -maxdepth 1 \
  -name '*.spec.md' -print | wc -l)
if ((ogs_active_count > 1)); then
  echo "Pipeline check failed: more than one active spec" >&2
  exit 1
fi

for ogs_spec in docs/planning/pipeline/active/*.spec.md; do
  [[ -f "$ogs_spec" ]] || continue
  ogs_stem=${ogs_spec%.spec.md}
  [[ -f "$ogs_stem.notes.md" ]] || {
    echo "Pipeline check failed: missing notes pair for $ogs_spec" >&2
    exit 1
  }

  ogs_ticket=$(sed -n 's/^ticket_doc:[[:space:]]*//p' "$ogs_spec" | head -1)
  ogs_aar=$(sed -n 's/^aar:[[:space:]]*//p' "$ogs_spec" | head -1)
  [[ -n "$ogs_ticket" && -f "$ogs_ticket" ]] || {
    echo "Pipeline check failed: active spec ticket is missing: $ogs_ticket" >&2
    exit 1
  }
  [[ -n "$ogs_aar" && -f "$ogs_aar" ]] || {
    echo "Pipeline check failed: active spec AAR is missing: $ogs_aar" >&2
    exit 1
  }
done

for ogs_spec in docs/planning/pipeline/completed/*.spec.md; do
  [[ -f "$ogs_spec" ]] || continue
  ogs_stem=${ogs_spec%.spec.md}
  [[ -f "$ogs_stem.notes.md" ]] || {
    echo "Pipeline check failed: missing completed notes pair for $ogs_spec" >&2
    exit 1
  }

  ogs_ticket=$(sed -n 's/^ticket_doc:[[:space:]]*//p' "$ogs_spec" | head -1)
  ogs_aar=$(sed -n 's/^aar:[[:space:]]*//p' "$ogs_spec" | head -1)
  ogs_status=$(sed -n 's/^status:[[:space:]]*//p' "$ogs_spec" | head -1)
  [[ "$ogs_status" == "Phase 5 — Complete PASS" ]] || {
    echo "Pipeline check failed: completed spec status is invalid: $ogs_spec" >&2
    exit 1
  }
  [[ -n "$ogs_ticket" && -f "$ogs_ticket" ]] || {
    echo "Pipeline check failed: completed spec ticket is missing: $ogs_ticket" >&2
    exit 1
  }
  [[ -n "$ogs_aar" && -f "$ogs_aar" ]] || {
    echo "Pipeline check failed: completed spec AAR is missing: $ogs_aar" >&2
    exit 1
  }
  grep -q '^status:[[:space:]]*submitted[[:space:]]*$' "$ogs_aar" || {
    echo "Pipeline check failed: completed spec AAR is not submitted: $ogs_aar" >&2
    exit 1
  }
done

echo "Pipeline structure check passed"

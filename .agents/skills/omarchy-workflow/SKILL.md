---
name: omarchy-workflow
description: Run or resume the evidence-based Omarchy Gaming System workflow for non-trivial feature, fix, migration, infrastructure, or workflow changes. Covers ticketed planning, EARS design, implementation, inspection, validation, completion, and authorized delivery; do not use for read-only questions or rough product brainstorming.
---

# Omarchy workflow

Read `AGENTS.md` and `CONSTITUTION.md`, then read
[references/phases.md](references/phases.md) completely before changing files.

## Route the request

1. Inspect `docs/planning/pipeline/active/`.
2. If an active spec exists, resume it from its recorded status. Do not open a
   second pipeline.
3. If the request is a rough idea rather than approved work, use
   `$omarchy-brainstorm` and do not write application code.
4. If the user explicitly waives ceremony for a small change, record the
   waiver in the final response and use the ordinary implementation loop. The
   constitution's quality, testing, secrets, and receipt rules still apply.
5. Otherwise, initialize Phase 1 and continue through the phases appropriate to
   the user's requested outcome.

Before Phase 2, run `scripts/check-pipeline-tools.sh`. If local state is absent,
run `scripts/setup-pipeline-tools.sh`. A newly configured MCP server requires a
Codex restart; until then, `scripts/codegraph.sh explore ...` is the permitted
CodeGraph fallback. OpenWiki completion always uses its MCP lifecycle.

Use the plan tool for multi-step work when available. Phase labels in the
active spec are the durable source of truth; do not infer phase completion from
chat history. Pause only for a missing decision that would materially change
scope or when the user requested phase-by-phase review.

Never claim a gate or test passed unless it actually ran. Never commit, push,
or open a pull request without explicit authorization for that delivery action.
Never manufacture or edit tool receipts under `.git`; lifecycle hooks and the
successful CodeGraph fallback wrapper own them.

At handoff, report the active or completed phase, changed surfaces, tests and
gate results, unresolved requirements, and delivery status.

# ADR-0001: Local agent work pipeline

- Status: accepted
- Date: 2026-08-23
- Knowledge ID: `AD-omarchy-bbs-agent-work-pipeline-001`

## Context

Omarchy Gaming System will span Rust services, PostgreSQL migrations, public API
contracts, QML behavior, and deterministic games. Agent-authored changes need
scope control, durable project memory, and evidence that crosses those layers.
The owner's Rustal project already demonstrates a disciplined agent workflow.

## Decision

Adopt Rustal's core progression:

```text
work → plan → design → implement → inspect → validate → complete → commit
```

Keep tickets, pipeline narratives, architecture decisions, bulletins, and AARs
inside this repository. Require EARS acceptance criteria, one active pipeline,
an adversarial inspect checkpoint, and a worktree-bound green-gate receipt
before Codex may commit gated files.

Use Codex's repository-native discovery surfaces: `AGENTS.md` for always-on
project guidance, `.agents/skills/` for the workflow playbooks, and trusted
`.codex/hooks.json` lifecycle hooks for deterministic guardrails. Hook decisions
must derive from repository state and documented hook input, not an unstable
chat-transcript format.

Use pinned, project-local CodeGraph and OpenWiki integrations through Codex MCP
configuration. CodeGraph is mandatory in design and post-implementation inspect
to expose structural flows and blast radius. OpenWiki is mandatory during
completion to reconcile claims-backed engineering memory. PostToolUse hooks
bind successful use to the active pipeline and gated worktree; Stop hooks reject
unsupported phase claims. Generated dependency/index state remains ignored and
telemetry is disabled.

Adapt the gate to this project's present risks: Rust formatting/lints/tests and
docs, Compose validation, shell syntax, and a real PostgreSQL→HTTP→QML smoke
path. Add coverage, mutation, broader security tooling, and richer QML/game
tests as the corresponding code surfaces arrive.

## Consequences

- Work carries more documentation than an ad-hoc coding loop.
- The active spec makes scope and evidence reviewable across sessions.
- Cross-layer integration failures are caught before delivery.
- The gate can ratchet upward without redesigning the workflow.
- Codex project hooks and MCP tools assist discipline, while `bin/gate.sh`
  remains usable locally by Codex and humans.
- Hosted CI/CD workflow definitions are prohibited. The local gate and its
  worktree-bound receipt are the sole delivery-quality evidence.
- A fresh clone needs one local tool bootstrap and a Codex restart before the
  first non-trivial pipeline can complete.
- The pinned OpenWiki revision needs a narrow generated-state patch until its
  upstream lifecycle can be configured to maintain only this repository's
  Codex guide and omit provider-driven scheduling.

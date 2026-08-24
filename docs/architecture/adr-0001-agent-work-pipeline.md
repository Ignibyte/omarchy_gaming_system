# ADR-0001: Local agent work pipeline

- Status: accepted
- Date: 2026-08-23
- Knowledge ID: `AD-omarchy-bbs-agent-work-pipeline-001`

## Context

Omarchy BBS will span Rust services, PostgreSQL migrations, public API
contracts, QML behavior, and deterministic games. Agent-authored changes need
scope control, durable project memory, and evidence that crosses those layers.
The owner's Rustal project already demonstrates a disciplined Claude workflow.

## Decision

Adopt Rustal's core progression:

```text
work → plan → design → implement → inspect → validate → complete → commit
```

Keep tickets, pipeline narratives, architecture decisions, bulletins, and AARs
inside this repository. Require EARS acceptance criteria, one active pipeline,
an adversarial inspect checkpoint, and a worktree-bound green-gate receipt
before Claude may commit gated files.

Adapt the gate to this project's present risks: Rust formatting/lints/tests and
docs, Compose validation, shell syntax, and a real PostgreSQL→HTTP→QML smoke
path. Add coverage, mutation, broader security tooling, and richer QML/game
tests as the corresponding code surfaces arrive.

## Consequences

- Work carries more documentation than an ad-hoc coding loop.
- The active spec makes scope and evidence reviewable across sessions.
- Cross-layer integration failures are caught before delivery.
- The gate can ratchet upward without redesigning the workflow.
- Claude-specific hooks assist discipline, while `bin/gate.sh` remains usable
  by humans, CI, and other agents.

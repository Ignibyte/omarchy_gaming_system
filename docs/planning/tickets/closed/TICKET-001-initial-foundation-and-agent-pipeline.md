---
title: TICKET-001-initial-foundation-and-agent-pipeline
status: done
ticket_number: 001
type: chore
created: 2026-08-23
closed: 2026-08-23
intake:
pipeline_spec: docs/planning/pipeline/completed/initial-foundation-and-agent-pipeline.spec.md
---

# TICKET-001-initial-foundation-and-agent-pipeline

## Summary

Establish the first verified Rust/PostgreSQL/QML vertical slice and adapt
Rustal's agent work pipeline for this repository.

## Why

Feature work needs a runnable foundation and a durable, evidence-based workflow
before accounts, personas, inboxes, connections, and games begin accumulating.

## EARS requirements

| ID | EARS requirement | Verification |
|---|---|---|
| REQ-001 | When the local development command starts, the system shall apply PostgreSQL migrations, serve a healthy Rust endpoint, and let the QML client consume it. | `./scripts/dev.sh --smoke-test`; migration/table inspection |
| REQ-002 | When Codex begins feature work, the repository shall provide ordered plan, design, implement, inspect, validate, complete, and delivery guidance. | Project instruction and workflow-skill validation |
| REQ-003 | When gated files are committed through Codex, the commit hook shall require a receipt matching the current gated worktree. | Hook self-test and `bin/gate.sh --diff` |
| REQ-004 | When a future session starts work, the repository shall provide local tickets, pipeline history, architecture decisions, bulletins, and recallable lessons. | Planning-tree and link audit |
| REQ-005 | When CI runs on GitHub, it shall execute the repository's canonical fast gate. | Workflow inspection; first Actions run after initial push |

## Scope

- In: local server/client/database slice, developer commands, CI, Codex
  instructions/skills/hooks, local planning and knowledge stores, canonical gate.
- Out: registration endpoints, persona APIs, inboxes, connections, game rules,
  deployment infrastructure, coverage/mutation floors.

## Links

- Pipeline spec: `docs/planning/pipeline/completed/initial-foundation-and-agent-pipeline.spec.md`
- Architecture: `docs/architecture/adr-0001-agent-work-pipeline.md`

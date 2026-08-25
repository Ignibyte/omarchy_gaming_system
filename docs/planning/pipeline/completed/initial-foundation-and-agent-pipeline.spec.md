---
title: Initial foundation and agent pipeline
pipeline_id: 99052ba2-3095-443d-a469-607158643a6c
status: Phase 5 — Complete PASS
ticket: TICKET-001
ticket_doc: docs/planning/tickets/closed/TICKET-001-initial-foundation-and-agent-pipeline.md
aar: docs/planning/knowledge/aar/AAR-001-initial-foundation-and-pipeline.md
created: 2026-08-23
---

# Initial foundation and agent pipeline — spec

## Intent

Ship the first database-backed server-to-QML connection and establish a
Rustal-inspired agent workflow that can carry subsequent product slices with
explicit scope, independent inspection, local memory, and verifiable gates.

## Scope

- In: Rust health server, identity migration, PostgreSQL Compose service, QML
  health client, one-command dev/smoke path, agent guidance/hooks, local work
  record, project-specific gate, and CI.
- Out: user-facing identity operations, messaging, game runtime, production
  deployment, Rustal's mature coverage/mutation/Playwright gates.

## Acceptance criteria (EARS)

| ID | EARS requirement | Verification |
|---|---|---|
| REQ-001 | When the local development command starts, the system shall apply PostgreSQL migrations, serve a healthy Rust endpoint, and let the QML client consume it. | `./scripts/dev.sh --smoke-test`; SQL migration/table queries |
| REQ-002 | When Codex begins feature work, the repository shall provide ordered plan, design, implement, inspect, validate, complete, and delivery guidance. | Project instruction and workflow-skill validation |
| REQ-003 | When gated files are committed through Codex, the commit hook shall require a receipt matching the current gated worktree. | Hook self-test and `bin/gate.sh --diff` |
| REQ-004 | When a future session starts work, the repository shall provide local tickets, pipeline history, architecture decisions, bulletins, and recallable lessons. | Planning-tree and link audit |
| REQ-005 | When CI runs on GitHub, it shall execute the repository's canonical fast gate. | Workflow inspection; first Actions run after initial push |

## Locked decisions

| # | Decision | Why |
|---|---|---|
| 1 | Use Rustal's phase progression and local-memory model. | It is already proven in the owner's adjacent Rust project. |
| 2 | Keep the canonical gate independent of Codex. | Humans and CI need the same truth source. |
| 3 | Start with static Rust checks plus the real DB/API/QML smoke path. | These are the current executable surfaces; later quality tiers should ratchet with real code. |
| 4 | Keep one active ticket and EARS requirements. | It controls scope and enables a requirement-by-requirement completion audit. |

## Linked artifacts

- Ticket: `docs/planning/tickets/open/TICKET-001-initial-foundation-and-agent-pipeline.md`
- Architecture: `docs/architecture/adr-0001-agent-work-pipeline.md`
- Product: `docs/product-charter.md`
- Reference inspected: local clone of `Ignibyte/rustal`

## Phase plan

| Phase | Deliverable | Exit gate |
|---|---|---|
| 1 Plan | Ticket, spec, notes, open AAR | scope recorded |
| 2 Design | Rustal comparison, architecture, file manifest, regression plan | approach recorded |
| 3 Implement | Commands, hooks, planning store, gate, CI integration | syntax/checks compile |
| 3.5 Inspect | Cross-file and enforcement review | findings dispositioned |
| 4 Validate | Hook tests and canonical gate green | matching receipt |
| 5 Complete | AC audit, submitted AAR, ticket and pair archived | no silent drops |
| Delivery | Staged review and authorized commit | receipt matches |

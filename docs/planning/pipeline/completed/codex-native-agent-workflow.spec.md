---
title: Codex-native agent workflow
pipeline_id: 2720b6c4-0b38-48be-838f-0c8a99b21ac1
status: Phase 5 — Complete PASS
ticket: TICKET-002
ticket_doc: docs/planning/tickets/closed/TICKET-002-codex-native-agent-workflow.md
aar: docs/planning/knowledge/aar/AAR-002-codex-native-agent-workflow.md
created: 2026-08-24
---

# Codex-native agent workflow — spec

## Intent

Make the repository workflow genuinely native to Codex while preserving the
existing evidence, phase, local-memory, and delivery-receipt model.

## Scope

- In: project instructions, workflow and brainstorm skills, lifecycle hooks,
  deterministic enforcement, hook self-tests, gate coverage, and docs.
- Out: global or managed Codex settings, product behavior, and remote delivery.

## Acceptance criteria (EARS)

| ID | EARS requirement | Verification |
|---|---|---|
| REQ-001 | When Codex starts in this repository, the system shall expose project guidance and the work workflow through supported Codex discovery locations. | `AGENTS.md`, repo-skill validation, and pipeline structure check |
| REQ-002 | When Codex edits gated files or attempts a commit, the system shall enforce phase readiness and a matching delivery receipt through repository Codex hooks. | Hook self-tests for phase denial and receipt denial/allowance |
| REQ-003 | When Codex finishes a turn, the system shall reject unsupported validation/completion claims and scan changed files for high-signal secrets. | Stop-hook self-tests |
| REQ-004 | When project checks run, the system shall validate only Codex workflow surfaces and contain no obsolete agent integration. | Fast gate and residual-reference audit |

## Locked decisions

| # | Decision | Why |
|---|---|---|
| 1 | Use `AGENTS.md`, `.agents/skills`, and `.codex/hooks.json`. | These are Codex's documented repository discovery surfaces. |
| 2 | Replace transcript-dependent enforcement with worktree/spec-state checks. | Codex documents transcript format as unstable; durable repository state is deterministic and testable. |
| 3 | Keep the canonical gate independent of Codex. | Humans and CI still need the same verification source. |

## Linked artifacts

- Ticket: `docs/planning/tickets/closed/TICKET-002-codex-native-agent-workflow.md`
- Architecture: `docs/architecture/adr-0001-agent-work-pipeline.md`
- Product: `docs/product-charter.md`

## Phase plan

| Phase | Deliverable | Exit gate |
|---|---|---|
| 1 Plan | Ticket, EARS spec, notes, open AAR | scope recorded |
| 2 Design | Codex surface mapping and regression plan | approach recorded |
| 3 Implement | Instructions, skills, hooks, tests, and docs | syntax/checks compile |
| 3.5 Inspect | Cross-file and enforcement review | findings dispositioned |
| 4 Validate | Skill validation, hook tests, and canonical gate | matching receipt |
| 5 Complete | AC audit, AAR, ticket and pair archived | no silent drops |
| Delivery | Staged review and commit when authorized | matching receipt |

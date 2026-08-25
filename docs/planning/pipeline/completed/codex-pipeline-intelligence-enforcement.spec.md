---
title: Codex pipeline intelligence enforcement
pipeline_id: b896c489-44bd-40e9-91c0-ff6c9ba3ecd3
status: Phase 5 — Complete PASS
ticket: TICKET-003
ticket_doc: docs/planning/tickets/closed/TICKET-003-codex-pipeline-intelligence-enforcement.md
aar: docs/planning/knowledge/aar/AAR-003-codex-pipeline-intelligence-enforcement.md
created: 2026-08-24
---

# Codex pipeline intelligence enforcement — spec

## Intent

Make CodeGraph structural analysis and OpenWiki knowledge reconciliation
required, observable parts of every non-trivial Codex pipeline.

## Scope

- In: pinned repository-local tool bootstrap, project Codex MCP wiring,
  Codex-only OpenWiki behavior, phase-bound evidence, tests, and documentation.
- Out: global configuration, hosted wiki publishing, CI network dependencies,
  application behavior, commits, pushes, and pull requests.

## Acceptance criteria (EARS)

| ID | EARS requirement | Verification |
|---|---|---|
| REQ-001 | When a developer prepares this trusted Codex project, the system shall install pinned CodeGraph and OpenWiki versions locally, disable their telemetry, configure their Codex MCP servers, and initialize the source graph without global agent configuration. | Setup smoke and configuration audit |
| REQ-002 | When a pipeline advances through design, the system shall require CodeGraph evidence bound to the current gated worktree before Codex may claim Phase 2 PASS. | Hook self-test |
| REQ-003 | When implementation is ready for validation, the system shall require a new CodeGraph evidence receipt bound to the post-edit gated worktree before Codex may claim Phase 3.5 PASS. | Hook self-test |
| REQ-004 | When a pipeline completes, the system shall require an OpenWiki lifecycle finish receipt bound to the current gated worktree before Codex may claim Phase 5 PASS. | Hook self-test |
| REQ-005 | When OpenWiki runs in this repository, the integration shall operate only through Codex surfaces and shall not create or retain obsolete agent integration files or references. | Residual audit and OpenWiki smoke |
| REQ-006 | When the repository is cloned without generated tool state, the canonical gate shall validate all committed wiring without requiring network access or globally installed third-party CLIs. | `bin/gate.sh --fast` and structure audit |

## Locked decisions

| # | Decision | Why |
|---|---|---|
| 1 | Install CodeGraph 1.5.0 and build OpenWiki 0.3.3 from inspected commit `a525ed8` under ignored `.dev/pipeline-tools`. | Avoids global state and floating dependencies; the npm artifact carrying the same OpenWiki version predates its Codex lifecycle server. |
| 2 | Configure only project-scoped Codex MCP servers. | This repository is Codex-only and project trust already gates local config. |
| 3 | Patch OpenWiki's repo setup during bootstrap to maintain only `AGENTS.md`. | Upstream 0.3.3 otherwise creates a second agent guide that violates the repository's Codex-only invariant. |
| 4 | Store evidence receipts under `.git`, keyed to the gated worktree hash. | Receipts remain local, tamper-evident for ordinary workflow use, and become stale after relevant edits. |
| 5 | Keep third-party CLI availability out of CI gates. | Clean-clone validation must remain deterministic and offline; runtime readiness is an explicit setup check. |

## Linked artifacts

- Ticket: [TICKET-003](../../tickets/open/TICKET-003-codex-pipeline-intelligence-enforcement.md)
- Architecture: [ADR-0001](../../../architecture/adr-0001-agent-work-pipeline.md)
- Intake:

## Phase plan

| Phase | Deliverable | Exit gate |
|---|---|---|
| 1 Plan | Ticket, spec, notes, open AAR | scope and decisions recorded |
| 2 Design | Tool topology, file manifest, receipt protocol, regression plan | actionable design and CodeGraph evidence |
| 3 Implement | Local bootstrap, MCP/skill wiring, hooks, docs | focused setup and syntax checks |
| 3.5 Inspect | Findings ledger and fixes | fresh post-edit CodeGraph analysis |
| 4 Validate | Hook regressions and delivery gate green | matching gate receipt |
| 5 Complete | OpenWiki reconciliation, AC audit, submitted AAR, archive | OpenWiki finish receipt and no silent drops |
| Delivery | Fresh gate, staged review, authorized commit | matching receipt |

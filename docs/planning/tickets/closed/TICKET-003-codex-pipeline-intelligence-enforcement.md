---
title: TICKET-003-codex-pipeline-intelligence-enforcement
status: done
ticket_number: 003
type: infrastructure
created: 2026-08-24
closed: 2026-08-24
intake:
pipeline_spec: docs/planning/pipeline/completed/codex-pipeline-intelligence-enforcement.spec.md
---

# TICKET-003-codex-pipeline-intelligence-enforcement

## Summary

Add pinned, project-scoped CodeGraph and OpenWiki integrations to the Codex
workflow and require fresh evidence that both were used at the appropriate
pipeline phases.

## Why

The workflow currently asks Codex to recall architecture and refresh durable
knowledge but cannot prove that structural analysis or wiki reconciliation
actually happened. The new integrations make those activities explicit and
machine-checkable without introducing another agent runtime.

## EARS requirements

| ID | EARS requirement | Verification |
|---|---|---|
| REQ-001 | When a developer prepares this trusted Codex project, the system shall install pinned CodeGraph and OpenWiki versions locally, disable their telemetry, configure their Codex MCP servers, and initialize the source graph without global agent configuration. | Setup smoke and configuration audit |
| REQ-002 | When a pipeline advances through design, the system shall require CodeGraph evidence bound to the current gated worktree before Codex may claim Phase 2 PASS. | Hook self-test |
| REQ-003 | When implementation is ready for validation, the system shall require a new CodeGraph evidence receipt bound to the post-edit gated worktree before Codex may claim Phase 3.5 PASS. | Hook self-test |
| REQ-004 | When a pipeline completes, the system shall require an OpenWiki lifecycle finish receipt bound to the current gated worktree before Codex may claim Phase 5 PASS. | Hook self-test |
| REQ-005 | When OpenWiki runs in this repository, the integration shall operate only through Codex surfaces and shall not create or retain obsolete agent integration files or references. | Residual audit and OpenWiki smoke |
| REQ-006 | When the repository is cloned without generated tool state, the canonical gate shall validate all committed wiring without requiring network access or globally installed third-party CLIs. | `bin/gate.sh --fast` and structure audit |

## Scope

- In: pinned repo-local installers, Codex MCP configuration, a repository
  OpenWiki skill, CodeGraph indexing, phase-bound tool receipts, hooks,
  self-tests, and operator documentation.
- Out: global Codex configuration, hosted wiki publication, unattended
  scheduled updates, changes to application behavior, and delivery to Git.

## Links

- Intake:
- Pipeline spec: [completed spec](../../pipeline/completed/codex-pipeline-intelligence-enforcement.spec.md)
- Architecture: [ADR-0001](../../../architecture/adr-0001-agent-work-pipeline.md)

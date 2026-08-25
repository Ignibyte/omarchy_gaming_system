---
title: TICKET-002-codex-native-agent-workflow
status: done
ticket_number: 002
type: chore
created: 2026-08-24
closed: 2026-08-24
intake:
pipeline_spec: docs/planning/pipeline/completed/codex-native-agent-workflow.spec.md
---

# TICKET-002-codex-native-agent-workflow

## Summary

Replace every previous-agent-specific workflow surface with repository-native
Codex instructions, skills, hooks, enforcement tests, and documentation.

## Why

Omarchy BBS will be developed exclusively with Codex. Its checked-in workflow
must use interfaces Codex discovers and executes rather than compatibility
artifacts for another coding agent.

## EARS requirements

| ID | EARS requirement | Verification |
|---|---|---|
| REQ-001 | When Codex starts in this repository, the system shall expose project guidance and the work workflow through supported Codex discovery locations. | `AGENTS.md`, repo-skill validation, and pipeline structure check |
| REQ-002 | When Codex edits gated files or attempts a commit, the system shall enforce phase readiness and a matching delivery receipt through repository Codex hooks. | Hook self-tests for phase denial and receipt denial/allowance |
| REQ-003 | When Codex finishes a turn, the system shall reject unsupported validation/completion claims and scan changed files for high-signal secrets. | Stop-hook self-tests |
| REQ-004 | When project checks run, the system shall validate only Codex workflow surfaces and contain no obsolete agent integration. | Fast gate and residual-reference audit |

## Scope

- In: `AGENTS.md`, repo skills, Codex hooks, hook helpers/tests, gate path
  coverage, planning validation, and all current documentation references.
- Out: user-global Codex configuration, managed enterprise policy, automatic
  hook trust decisions, product features, and GitHub delivery.

## Links

- Pipeline spec: `docs/planning/pipeline/completed/codex-native-agent-workflow.spec.md`
- Architecture: `docs/architecture/adr-0001-agent-work-pipeline.md`

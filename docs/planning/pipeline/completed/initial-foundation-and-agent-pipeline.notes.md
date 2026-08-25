---
title: Initial foundation and agent pipeline — notes
pipeline_id: 99052ba2-3095-443d-a469-607158643a6c
---

# Initial foundation and agent pipeline — running notes

## Phase 1 — Plan

- The first playable foundation is the real QML → `/health` → PostgreSQL path.
- The user selected Rust for the server, QML on Omarchy, and the existing
  `Ignibyte/omarchy_bbs` repository.
- Work expanded to adapt the agent pipeline from `Ignibyte/rustal`.

## Phase 2 — Design

- Rustal surfaces inspected: project guidance, `CONSTITUTION.md`, phase playbooks,
  settings/hooks, ticket/spec/notes/AAR templates, local knowledge register,
  and `bin/gate.sh` receipt model.
- Adopted: one active pipeline, EARS criteria, mandatory inspect, local memory,
  phase commands, and a content-bound commit receipt.
- Adapted: the gate targets Rust, PostgreSQL, Compose, and QML. Rustal's CMS,
  Playwright, mutation, coverage, Fable/Opus batch, and component-generation
  specifics remain out of scope until the project has those surfaces.
- File manifest: agent configuration, `bin/`, `CONSTITUTION.md`, project guidance, planning
  stores/templates, architecture docs, README, CI, and verification scripts.

### Regression plan

| Requirement | Evidence |
|---|---|
| REQ-001 | Live migration/API/QML smoke and table inspection |
| REQ-002 | Expected phase-guidance inventory plus valid hook configuration |
| REQ-003 | Commit hook denies missing/stale receipts and allows matching receipts |
| REQ-004 | Link/path audit over ticket, spec, AAR, architecture, bulletin, and knowledge files |
| REQ-005 | CI references the canonical gate; confirm the first Actions run after initial push |

## Phase 3 — Implement

- Added the project guide, binding constitution, work and brainstorm guidance,
  six phase playbooks, and delivery guidance.
- Added local ticket, intake, pipeline, bulletin, architecture, knowledge, and
  AAR stores with templates and the bootstrap TICKET-001 record.
- Added phase, recall, task, validation, completion, secret, and commit-receipt
  hooks wired through the project agent configuration.
- Added a canonical gate with a content-bound receipt, pipeline structure
  checks, and isolated hook self-tests. CI now calls the canonical fast gate.
- Added the system overview, pipeline ADR, roadmap, and README instructions.

## Phase 3.5 — Inspect ledger

| # | Lens | Finding | Severity | Disposition |
|---|---|---|---|---|
| 1 | Security/secrets | The secret-scanner self-test originally contained its fake `gho_` value contiguously in the source, so the scanner would flag the test file itself while it was changed. | medium | Real; construct the fixture from separate format/value fragments and retain the runtime detection test. |
| 2 | Correctness | The receipt needed explicit proof that `git add` preserves green while a later content edit invalidates it. | medium | Real coverage gap; added allow-after-stage and deny-after-edit assertions to the isolated hook test. |
| 3 | Simplification | CI and `scripts/check.sh` duplicated subsets of the quality commands. | low | Real; both now invoke the canonical `bin/gate.sh --fast`. |
| 4 | Data/state | PostgreSQL 18 migration boot and the identity tables remain covered by the real smoke path; no migration was rewritten during the pipeline adaptation. | — | No new finding. |
| 5 | QML/usability | The existing health client exposes connecting, connected, offline, protocol-error, and manual reconnect states; offscreen load and live XHR still require full-gate confirmation. | — | No code finding; retained as Phase 4 smoke evidence. |
| 6 | Delivery evidence | The first staged review found blank EOF lines in newly added files because plain `git diff --check` ignores untracked files. | medium | Real; remove the whitespace and make the canonical check scan tracked, staged, and untracked content. |

## Phase 4 — Validate

- `bin/gate.sh --fast` passed formatting, Clippy, 3 Rust tests, rustdoc,
  Compose validation, shell syntax, pipeline structure, hook self-tests, and
  whitespace checks.
- `bin/gate.sh --diff` passed all fast checks plus the real PostgreSQL + Rust
  API + QML smoke and printed `GATE GREEN [diff]`.
- PostgreSQL recorded migration `1 / identity foundation / success=true` and
  contained `_sqlx_migrations`, `accounts`, `account_sessions`, and `personas`.
- The live receipt matched `bbs_gate_state_hash`; the real commit hook accepted
  the current worktree.
- No tests or failures were skipped.

## Phase 5 — Complete

- Acceptance-criteria audit:

  | Requirement | Verdict | Evidence |
  |---|---|---|
  | REQ-001 | satisfied | Full gate check 11; SQL migration/table queries; server log showed three 200 health requests and graceful shutdown. |
  | REQ-002 | satisfied | Nine command files cover work, brainstorm, six ordered phases, and commit; seven configured enforcement hooks resolve to executable files. |
  | REQ-003 | satisfied | Isolated tests prove missing/stale receipt denial and matching/staged receipt allowance; the real worktree receipt also passed the commit hook. |
  | REQ-004 | satisfied | `scripts/check-pipeline.sh` verifies required local stores, templates, pairs, tickets, and AAR links. |
  | REQ-005 | satisfied locally | `.github/workflows/ci.yml` invokes `bin/gate.sh --fast`, the same command proven green locally; the first remote Actions run remains pending until `main` is pushed. |
- Docs updated: README, constitution, project guide, roadmap, system overview,
  ADR-0001, local ticket/knowledge/bulletin stores, and all templates.
- AAR: `docs/planning/knowledge/aar/AAR-001-initial-foundation-and-pipeline.md`
  submitted with both encountered failures and prevention rules registered.
- Ticket moved to `closed/`; spec/notes pair moved to `completed/`; active
  pipeline is empty.

## Defect and lesson ledger

| # | What happened | Root cause | Fix | Prevention |
|---|---|---|---|---|
| 1 | PostgreSQL 18 container exited on the original volume target. | PostgreSQL 18 images use a major-version-aware layout beneath `/var/lib/postgresql`. | Mount the named volume at `/var/lib/postgresql` and recreate the brand-new failed volume. | `BF-omarchy-bbs-postgres18-volume-layout-001` |
| 2 | A high-signal fake credential in a scanner test could trigger the scanner against its own changed source. | The fixture was stored as one contiguous literal rather than assembled only in the test sandbox. | Split the prefix and body in source while producing the same runtime fixture. | `PR-omarchy-bbs-secret-fixtures-must-not-match-source-001` |
| 3 | The original whitespace gate missed new files until they were staged. | `git diff --check` does not inspect untracked content. | Scan unstaged, staged, and each committable untracked file. | `PR-omarchy-bbs-quality-gates-include-untracked-001` |

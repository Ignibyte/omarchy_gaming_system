---
title: Local-only automation state reconciliation
pipeline_id: 7f356f37-bd05-43f0-a123-98a74a6c99ba
status: Phase 5 — Complete PASS
ticket: TICKET-043
ticket_doc: docs/planning/tickets/closed/TICKET-043-local-only-automation-state-reconciliation.md
aar: docs/planning/knowledge/aar/AAR-043-local-only-automation-state-reconciliation.md
created: 2026-08-30
---

# Local-only automation state reconciliation — spec

## Intent

Repair drift between the already accepted local-only delivery architecture,
the live GitHub Actions permission, and contributor guidance without changing
the canonical local quality gate or introducing a remote dependency.

## Scope

- In:
  - GitHub Actions permission disablement and empty-workflow readback;
  - lifecycle-owned `AGENTS.md` OpenWiki guidance correction at the reviewed
    pinned local generator;
  - fail-closed source/build readiness checks for that local-only patch;
  - existing local-only checker/hostile-fixture verification;
  - local CodeGraph/OpenWiki/gate evidence and delivery.
- Out:
  - application behavior or production infrastructure;
  - hosted CI/CD of any kind;
  - network calls inside the canonical local gate;
  - changes to the quality-stage inventory.

## Acceptance criteria (EARS)

| ID | EARS requirement | Verification |
|---|---|---|
| REQ-001 | When the GitHub repository automation state is inspected, Actions shall be disabled and the workflow inventory shall be empty. | `gh api` permissions and workflows readback before completion and after delivery. |
| REQ-002 | When the repository is checked locally, it shall contain no GitHub Actions or equivalent hosted CI/CD definition and shall reject a hostile hosted-workflow fixture. | `scripts/check-local-only-automation.sh`, `scripts/test-server-module-spike.sh`, tracked-file inventory, and residual path audit. |
| REQ-003 | When contributors read the project workflow guidance, it shall describe OpenWiki refresh as a local lifecycle and shall not claim that scheduled GitHub Actions exist. | Reviewed OpenWiki source patch, source/build readiness assertions, lifecycle-regenerated `AGENTS.md`, and README/Constitution/ADR documentation audit. |
| REQ-004 | When the correction completes, all validation shall run through local scripts, the canonical local diff gate shall pass, and no hosted automation dependency shall be introduced. | Focused shell/pipeline checks, local `bin/gate.sh --diff`, worktree receipt, and staged dependency review. |
| REQ-005 | When the slice completes, CodeGraph/OpenWiki evidence, an AAR, the ticket archive, and the local-only architecture record shall remain internally consistent. | CodeGraph receipts, OpenWiki completion receipt, requirement audit, and pipeline-structure check. |

## Locked decisions

| # | Decision | Why |
|---|---|---|
| 1 | Retain `bin/gate.sh` and its worktree receipt as the sole delivery-quality proof. | The Constitution and ADR-0001 already establish the accepted local architecture. |
| 2 | Keep GitHub settings readback outside the canonical gate. | A local delivery proof must not require network or GitHub availability. |
| 3 | Reuse the existing local-only checker and hostile fixture unless inspection finds a concrete coverage gap. | Ticket 039 already implemented and validated the repository enforcement. |
| 4 | Correct the managed contributor text in the reviewed OpenWiki dependency patch, not by hand-editing the generated `AGENTS.md` block. | The lifecycle owns that block and demonstrably restores the upstream hosted-workflow sentence on every begin. |
| 5 | Make no application/runtime change. | The defect is external configuration drift plus local workflow-tool integration drift. |

## Linked artifacts

- Ticket: [TICKET-043](../../tickets/closed/TICKET-043-local-only-automation-state-reconciliation.md)
- Architecture: [ADR-0001](../../../architecture/adr-0001-agent-work-pipeline.md)
- Intake:

## Phase plan

| Phase | Deliverable | Exit gate |
|---|---|---|
| 1 Plan | Ticket, spec, notes, open AAR | settled local-only correction scope |
| 2 Design | Exact external/local evidence and file manifest | CodeGraph receipt plus direct shell/doc inspection |
| 3 Implement | Disabled Actions setting and corrected lifecycle-owned guidance | focused setup/readiness and local-only checks |
| 3.5 Inspect | Drift, enforcement, documentation, and dependency review | findings disposition plus fresh CodeGraph receipt |
| 4 Validate | Focused checks and local diff gate | matching worktree receipt |
| 5 Complete | AC audit, OpenWiki, submitted AAR, archive | no silent drops |
| Delivery | Staged review, commit, and push | authorized matching delivery evidence |

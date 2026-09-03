---
title: Usurper Local-play Control Regression
pipeline_id: 963f0e95-1a0d-45e7-8519-1b6f2270188e
status: Phase 5 — Complete PASS
ticket: TICKET-065
ticket_doc: docs/planning/tickets/closed/TICKET-065-usurper-local-play-control-regression.md
aar: docs/planning/knowledge/aar/AAR-065-usurper-local-play-control-regression.md
created: 2026-09-02
---

# Usurper Local-play Control Regression — spec

## Intent

Make the provider-backed Usurper development window honestly interactive and
render each signed choice once, with tests that exercise the same pointer and
keyboard event paths a player uses.

## Scope

- In:
  - trusted QML delegate cardinality and lifecycle;
  - real pointer and keyboard activation across local loading and plan changes;
  - exact current action/revision propagation through the local provider;
  - fixture-versus-live affordance clarity and workspace-8 validation.
- Out:
  - new Usurper levels or game systems;
  - platform-owned Usurper rules, provider protocol changes, database work,
    packaging, admission, deployment, commit, push, or publication.

## Acceptance criteria (EARS)

| ID | EARS requirement | Verification |
|---|---|---|
| REQ-001 | When a valid signed render plan is accepted, the trusted QML surface shall materialize exactly one visual control for each interactive node, with no retained delegate from an earlier plan. | Delegate/node cardinality tests, unique ID/label assertions, plan-replacement test, and captured current-plan evidence. |
| REQ-002 | When local-play loading completes, each materialized control shall follow the surface's enabled state and accept pointer input without requiring a reload. | Real QML mouse-event test across the disabled-to-enabled transition. |
| REQ-003 | When a player activates one current control by pointer or keyboard, the shell shall submit exactly one signed current-screen action and one provider revision shall be confirmed. | Signal/action cardinality tests and provider-backed HTTP/QML revision proof. |
| REQ-004 | While an activation key repeats or a plan is replaced asynchronously, the trusted surface shall consume the repeat without activating the newly focused control. | Real key-event and plan-replacement regression test. |
| REQ-005 | When a signed fixture is opened instead of local play, it shall remain visibly identified as non-interactive, while the provider-backed development shell remains interactive. | Fixture/local title and action-authority tests. |
| REQ-006 | When the repaired current build is opened for review, it shall run in workspace 8 with unique controls and working one-activation/one-revision behavior, without adding Level 12, production admission, deployment, commit, push, or publication. | Workspace-8 process/window readback, visible inspection, complete relevant gates, and scope review. |

## Locked decisions

| # | Decision | Why |
|---|---|---|
| 1 | Treat current provider JSON and visible QML as separate evidence boundaries. | Unique signed nodes do not prove unique delegates or working hit-testing. |
| 2 | Reproduce with real Qt pointer and keyboard events before choosing a repair. | Direct function calls bypass the reported path and previously produced a false-green smoke. |
| 3 | Keep the signed fixture preview inert and repair only provider-backed local play. | A preview has no confirming provider; presenting its controls as live would be dishonest. |
| 4 | Preserve one phase-valid command per visible choice and one provider revision per activation. | This is the established safety and usability contract from Tickets 061–064. |

## Linked artifacts

- Ticket: [TICKET-065](../../tickets/closed/TICKET-065-usurper-local-play-control-regression.md)
- Architecture: [Game Cartridges](../../../architecture/game-cartridges.md)
- Intake: direct user report on 2026-09-02.

## Phase plan

| Phase | Deliverable | Exit gate |
|---|---|---|
| 1 Plan | Ticket, spec, notes, open AAR | concrete reproduction and acceptance contract |
| 2 Design | Event/delegate flow and exact file manifest | worktree-bound CodeGraph receipt |
| 3 Implement | Minimal QML/test/provider-harness repair | compile and self-review |
| 3.5 Inspect | Findings ledger and security review | lead disposition |
| 4 Validate | Pointer/key/provider/full gates and workspace-8 play | matching receipts |
| 5 Complete | AC audit, docs, OpenWiki, submitted AAR, archive | no silent drops |
| Delivery | Fresh gate, staged review, authorized commit | explicit user authorization only |

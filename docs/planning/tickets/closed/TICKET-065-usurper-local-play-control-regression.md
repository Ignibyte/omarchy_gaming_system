---
title: TICKET-065-usurper-local-play-control-regression
status: done
ticket_number: 065
type: bug
created: 2026-09-02
closed: 2026-09-02
intake:
pipeline_spec: docs/planning/pipeline/completed/usurper-local-play-control-regression.spec.md
---

# TICKET-065-usurper-local-play-control-regression

## Summary

Reproduce and repair the reported Usurper local-play regression in which most
controls appear twice and visible buttons do not activate, then prove the real
pointer and keyboard paths against provider revisions in workspace 8.

## Why

The signed provider payload and current conformance checks report unique nodes,
but the user-visible development client is the acceptance boundary. Existing
QML smoke calls control functions directly, so it can miss pointer hit-testing,
delegate materialization, and disabled-to-enabled transition defects.

## EARS requirements

| ID | EARS requirement | Verification |
|---|---|---|
| REQ-001 | When a valid signed render plan is accepted, the trusted QML surface shall materialize exactly one visual control for each interactive node, with no retained delegate from an earlier plan. | Delegate/node cardinality tests, unique ID/label assertions, plan-replacement test, and captured current-plan evidence. |
| REQ-002 | When local-play loading completes, each materialized control shall follow the surface's enabled state and accept pointer input without requiring a reload. | Real QML mouse-event test across the disabled-to-enabled transition. |
| REQ-003 | When a player activates one current control by pointer or keyboard, the shell shall submit exactly one signed current-screen action and one provider revision shall be confirmed. | Signal/action cardinality tests and provider-backed HTTP/QML revision proof. |
| REQ-004 | While an activation key repeats or a plan is replaced asynchronously, the trusted surface shall consume the repeat without activating the newly focused control. | Real key-event and plan-replacement regression test. |
| REQ-005 | When a signed fixture is opened instead of local play, it shall remain visibly identified as non-interactive, while the provider-backed development shell remains interactive. | Fixture/local title and action-authority tests. |
| REQ-006 | When the repaired current build is opened for review, it shall run in workspace 8 with unique controls and working one-activation/one-revision behavior, without adding Level 12, production admission, deployment, commit, push, or publication. | Workspace-8 process/window readback, visible inspection, complete relevant gates, and scope review. |

## Scope

- In:
  - current Usurper provider payload, trusted QML materialization, pointer and
    keyboard activation, busy/enable transitions, and plan replacement;
  - regression tests that send real pointer/key events instead of invoking
    control functions directly;
  - provider-backed workspace-8 validation and precise fixture/live labeling.
- Out:
  - Level 12 or additional game content;
  - platform gameplay authority, provider protocol/schema changes, migrations,
    registration, admission, deployment, commit, push, or publication.

## Links

- Intake: direct user report on 2026-09-02.
- Pipeline spec: [usurper-local-play-control-regression.spec.md](../../pipeline/completed/usurper-local-play-control-regression.spec.md)
- Architecture: [Game Cartridges](../../../architecture/game-cartridges.md)

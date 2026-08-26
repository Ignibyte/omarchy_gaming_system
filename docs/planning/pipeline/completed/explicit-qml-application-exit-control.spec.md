---
title: Explicit QML application exit control
pipeline_id: bc70b184-3b03-44f0-9435-0b263117cede
status: Phase 5 — Complete PASS
ticket: TICKET-026
ticket_doc: docs/planning/tickets/closed/TICKET-026-explicit-qml-application-exit-control.md
aar: docs/planning/knowledge/aar/AAR-026-explicit-qml-application-exit-control.md
created: 2026-08-26
---

# Explicit QML application exit control — spec

## Intent

Give every player a discoverable, keyboard-accessible way to close the QML
client through the platform-owned shell while preserving all account, persona,
social, game, cartridge, and server authority boundaries.

## Scope

- In: all three Ticket 026 requirements; production QML shell; deterministic
  keyboard, pointer, accessibility, and layout evidence; lifecycle docs and
  workflow completion.
- Out: confirmation or dirty-state policy, server/API/database changes,
  implicit logout or session revocation, installer/package implementation,
  desktop/window-manager configuration, and Git delivery.

## Acceptance criteria (EARS)

The authoritative acceptance criteria are REQ-001 through REQ-003 in
[`TICKET-026`](../../tickets/closed/TICKET-026-explicit-qml-application-exit-control.md#ears-requirements).

## Locked decisions

| # | Decision | Why |
|---|---|---|
| 1 | The shell owns one persistent EXIT button rather than duplicating it on routed screens. | Application lifetime is global and must not drift with screen implementations. |
| 2 | Activation requests a normal `ApplicationWindow.close()` and does not call controller logout or network functions. | Closing a client process and revoking a durable server session are separate user actions. |
| 3 | The control uses the existing trusted `OgsButton` contract and remains keyboard reachable without stealing routed-screen initial focus. | This preserves the established theme, accessible role, focus ring, and deterministic screen entry behavior. |
| 4 | No confirmation is added while the client has no unsaved local document state. | A modal would add complexity without preventing durable game or social data loss. |

## Linked artifacts

- Ticket: `docs/planning/tickets/closed/TICKET-026-explicit-qml-application-exit-control.md`
- Architecture: `docs/architecture/system-overview.md`
- Dependencies: Ticket 025 host-owned theme and production-root QML fixture
- Intake: none

## Phase plan

| Phase | Deliverable | Exit gate |
|---|---|---|
| 1 Plan | Ticket, EARS scope, active spec/notes, open AAR | bounded shell-only slice |
| 2 Design | shell close flow, file manifest, regression plan, CodeGraph receipt | actionable QML-only design |
| 3 Implement | persistent EXIT control and interaction coverage | focused QML suite |
| 3.5 Inspect | accessibility, lifecycle, layout, security, and blast-radius ledger | resolved findings and fresh CodeGraph receipt |
| 4 Validate | focused QML tests and canonical diff gate | matching gate receipt |
| 5 Complete | AC audit, OpenWiki, docs, submitted AAR, ticket/archive | matching completion receipt |
| Delivery | fresh gate, staged review, separately authorized commit/push | explicit delivery authorization |

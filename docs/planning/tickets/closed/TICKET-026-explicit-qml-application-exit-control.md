---
title: TICKET-026-explicit-qml-application-exit-control
status: closed
ticket_number: 026
type: feature
created: 2026-08-26
closed: 2026-08-26
intake:
pipeline_spec: docs/planning/pipeline/completed/explicit-qml-application-exit-control.spec.md
---

# TICKET-026 — Explicit QML application exit control

## Summary

Add a persistent, keyboard-accessible exit control to the platform-owned QML
shell so a player can close OmarchyGS without relying on window-manager
knowledge or an external terminal command.

## Why

The client can currently be closed through window-manager controls or by
terminating `qml6`, but the interface itself provides no discoverable exit
action. An installed private-alpha client needs an explicit application
lifecycle control.

## EARS requirements

| ID | EARS requirement | Verification |
|---|---|---|
| REQ-001 | While the QML client window is open, the system shall present one visible, enabled, shell-owned exit control on every routed screen at the supported 640×420 minimum and 920×600 default sizes. | Production-root QML interaction and geometry test |
| REQ-002 | When a player activates the exit control with Enter or a pointer, the system shall request a normal application-window close without dispatching logout, API, social, game, or cartridge actions. | Production-root keyboard/pointer close tests and source review |
| REQ-003 | When the exit control is exposed to assistive technology, the system shall provide a stable button role, descriptive accessible name, and visible keyboard focus treatment. | QML accessibility/focus assertions |

## Scope

- In: the production QML shell, explicit normal-window close behavior,
  keyboard/pointer/accessibility/compact-layout tests, and lifecycle docs.
- Out: confirmation dialogs, unsaved-work policy, server/API/database changes,
  session logout or revocation, installer/package implementation, operating
  system window-manager configuration, and Git delivery.

## Links

- Intake: none
- Pipeline spec: [completed spec](../../pipeline/completed/explicit-qml-application-exit-control.spec.md)
- Architecture: [system overview](../../../architecture/system-overview.md)

## Outcome

Closed with all three requirements passing. The production shell now exposes a
persistent keyboard- and pointer-operable EXIT button across every route. It
requests a normal window close and deliberately leaves durable server-session
authority unchanged. The final focused suite passed 40 QML cases; canonical
post-documentation delivery evidence is recorded in the completed pipeline
notes.

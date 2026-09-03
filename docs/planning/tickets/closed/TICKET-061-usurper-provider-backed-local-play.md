---
title: TICKET-061-usurper-provider-backed-local-play
status: closed
ticket_number: 061
type: feature
created: 2026-09-02
closed: 2026-09-02
intake:
pipeline_spec: docs/planning/pipeline/completed/usurper-provider-backed-local-play.spec.md
---

# TICKET-061-usurper-provider-backed-local-play

## Summary

Add a development-only local-play shell that connects the signed Usurper
cartridge and production trusted QML surface to the real provider adapter, so
visible button presses confirm deterministic state changes or signed screen
navigation. Make the existing fixture preview visibly non-interactive.

## Why

The signed fixture preview proves rendering but looks playable while discarding
every requested action. A provider-backed shell is the smallest honest way to
test the game visibly before production registration, admission, or deployment.

## EARS requirements

| ID | EARS requirement | Verification |
|---|---|---|
| REQ-001 | When a developer launches local Usurper play, the system shall start one in-memory `UsurperGame` session, compile its view from the exact signed cartridge through the platform renderer, and display it through the trusted QML surface with an explicit development-only label. | Local-play script smoke and visible workspace-8 verification. |
| REQ-002 | When the player requests a provider action declared by the currently rendered plan, the shell shall apply it through `ProviderGame`, compile the resulting view, and publish the new state only after both operations succeed. | Rust reducer/commit tests and local HTTP integration. |
| REQ-003 | When the player requests a current signed `navigate.*` action, the shell shall render its authenticated target without mutating provider state; when an action is stale, undeclared, malformed, or carries an unsupported payload, the shell shall reject it without state mutation. | Rust navigation, revision, declaration, payload, and rollback tests. |
| REQ-004 | While the local-play service is running, it shall bind only to loopback, require an unguessable session capability for state/action requests, bound request and response data, and serve only renderer-emitted assets addressed by validated tokens. | Focused HTTP integration and security inspection. |
| REQ-005 | When the signed fixture preview is opened, it shall visibly identify itself as non-interactive and shall not present enabled controls outside its automated input smoke. | QML smoke plus visible fixture review. |
| REQ-006 | When the slice is validated, existing provider, signed-cartridge, renderer, QML, and platform delivery gates shall remain green without database, protocol, admission, registration, deployment, or publication changes. | Focused suites, full external suite, platform `bin/gate.sh --diff`, and scope review. |

## Scope

- In:
  - explicit-screen support in the platform development preview CLI;
  - a separate platform-owned local-play QML shell over the trusted surface;
  - an in-memory Usurper provider driver with loopback/capability boundaries;
  - exact provider versus signed-navigation handling, revision checks, and
    atomic render-before-commit behavior;
  - automated local-play and QML smoke coverage;
  - a visibly inert fixture preview and workspace-8 live verification.
- Out:
  - platform gameplay rules or Usurper state;
  - PostgreSQL persistence for the local shell;
  - Provider SDK/protocol changes;
  - cartridge registration, admission, packaging, deployment, publication,
    shared-realm work, or Level 9 content.

## Links

- Intake:
- Pipeline spec: [usurper-provider-backed-local-play.spec.md](../../pipeline/completed/usurper-provider-backed-local-play.spec.md)
- Architecture: [game-cartridges.md](../../../architecture/game-cartridges.md)

## Outcome

Completed as a non-packaged provider-backed local-play harness over the real
Usurper adapter and signed platform renderer. Provider actions and signed
navigation now produce visible confirmed updates through trusted QML; the old
fixture viewer is explicitly inert. Focused and full external suites, the live
provider corpus twice across restart, a zero-finding security scan, the exact
forty-file production QML inventory, the full platform diff gate, OpenWiki, and
visible workspace-8 play passed. Registration, admission, persistence,
deployment, commit, push, and publication remain outside this ticket.

---
title: TICKET-023-keyboard-first-qml-connections-and-private-inbox
status: closed
ticket_number: 023
type: feature
created: 2026-08-25
closed: 2026-08-25
intake:
pipeline_spec: docs/planning/pipeline/completed/keyboard-first-qml-connections-and-private-inbox.spec.md
---

# TICKET-023-keyboard-first-qml-connections-and-private-inbox

## Summary

Extend the authenticated QML player shell with keyboard-first connection,
private block, conversation, and message screens that consume the existing
persona-scoped REST contracts without widening account or credential exposure.

## Why

Ticket 022 lets a player reach one owned persona, but the client still stops at
a placeholder home. Connections and private inboxes are the next coherent
product boundary: together they let two clean clients find one another by exact
handle, establish or end a relationship, and exchange durable private messages
before challenge and gameplay presentation are added.

## EARS requirements

| ID | EARS requirement | Verification |
|---|---|---|
| REQ-001 | When an authenticated player with a selected owned persona opens the social hub, the client shall load separate incoming requests, outgoing requests, accepted connections, and private blocks through owner-scoped APIs while exposing explicit loading, empty, ready, and retryable error states. | QML fixture inventory/state matrix and migrated live smoke |
| REQ-002 | When a player submits an exact public persona handle, the client shall resolve the public profile, reject self or malformed input locally, create the connection request without disclosing block direction or account identity, and support keyboard accept, decline, cancel, and accepted-connection removal with idempotent recovery. | Hostile fixture request/action cases and multi-account live path |
| REQ-003 | When a player blocks or unblocks another persona, the client shall use only the selected owned persona as actor, refresh private block and connection state, present generic unavailable feedback, and never infer or render another persona's block direction. | Fixture block/unblock/privacy cases and source review |
| REQ-004 | When the player opens the private inbox or a conversation, the client shall load bounded conversation inventory and ascending conversation-local history, render only exact allowlisted user and typed system-message variants as plain text, expose unread state, and support bounded older-page recovery. | Conversation/message schema corpus and keyboard UI tests |
| REQ-005 | When the player sends a valid private message or opens unread history, the client shall submit only bounded body text, clear the composer after handoff, append or reload committed history, and advance only that persona's private read position without moving it backward. | Fixture send/read/retry tests and migrated two-account smoke |
| REQ-006 | When social or inbox transport times out, is superseded, returns malformed or oversized data, loses authorization, or denies current relationship policy, the client shall keep prior valid state or fail closed, expose a bounded recoverable error, and clear all account/persona authority on `invalid_session`. | Adversarial transport/schema/401 corpus |
| REQ-007 | When any new screen is used at 640×420 or larger, every field, list item, action, tab, retry, pagination, and back path shall have an accessible name, visible focus, keyboard-only traversal/activation, plain-text output, and predictable Enter/Escape behavior. | Offscreen Qt Quick interaction matrix at both supported sizes |
| REQ-008 | When the canonical delivery gate runs, it shall execute the deterministic social/inbox QML corpus and a real migrated two-account connection/conversation/message flow before accepting the client slice. | Focused QML script, enhanced live smoke, and `bin/gate.sh --diff` |

## Scope

- In: authenticated home navigation; exact-handle public lookup; connection
  request inventory/create/accept/decline/cancel/remove; private block
  inventory/create/remove; conversation inventory; bounded message history and
  older-page recovery; user send; read acknowledgement; tagged system-message
  presentation; manual durable refresh; invalid-session cleanup; keyboard,
  accessibility, hostile fixtures, real PostgreSQL/API/QML smoke, docs,
  OpenWiki, security inspection, and AAR.
- Out: WebSocket hint consumption or background polling; game catalog,
  challenges, sessions, gameplay, cartridges, achievements, reporting;
  persona editing; token persistence; new server endpoints or migrations;
  broad visual-polish pass; Git delivery.

## Links

- Intake: none; selected from the ordered private-alpha roadmap after Ticket 022.
- Pipeline spec: `docs/planning/pipeline/completed/keyboard-first-qml-connections-and-private-inbox.spec.md`
- Architecture: `docs/architecture/system-overview.md`

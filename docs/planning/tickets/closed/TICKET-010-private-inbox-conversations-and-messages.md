---
title: TICKET-010-private-inbox-conversations-and-messages
status: closed
ticket_number: 010
type: feature
created: 2026-08-24
closed: 2026-08-24
intake:
pipeline_spec: docs/planning/pipeline/completed/private-inbox-conversations-and-messages.spec.md
---

# TICKET-010-private-inbox-conversations-and-messages

## Summary

Add one private inbox conversation per accepted persona pair, user-authored
messages, durable per-persona unread state, and server-authored typed system
messages with bounded history and explicit privacy contracts.

## Why

Connections now establish who may begin private interaction. The first playable
needs a durable inbox before game challenges and turn notifications can be
represented without making WebSockets authoritative or allowing clients to
forge system events.

## EARS requirements

| ID | EARS requirement | Verification |
|---|---|---|
| REQ-001 | When a pending request becomes an accepted connection, the system shall create at most one private conversation for that canonical persona pair and append one typed `connection_accepted` system message only for the state transition; retrying acceptance shall create neither duplicate. | Transactional router/PostgreSQL tests |
| REQ-002 | When an authenticated account lists conversations for an owned persona, the system shall return only conversations containing that persona, the other public persona profile, stable latest-message metadata, and a durable unread count without exposing account ownership or another participant's read state. | Multi-account inventory/privacy tests |
| REQ-003 | When a participant sends bounded user text while the pair is still connected and unblocked, the system shall append one server-timestamped message, mark it read for the sender, and make it unread for the other participant; clients shall not be able to submit a system type, sender, sequence, or timestamp. | Input-allowlist and PostgreSQL tests plus live smoke |
| REQ-004 | When a participant reads bounded conversation history, the system shall return a stable sequence-ordered page containing explicit user or system message shapes and only public persona data; a foreign, absent, or malformed conversation shall remain non-disclosing. | Pagination, DTO, and authorization tests |
| REQ-005 | When a participant marks through a message as read, the system shall monotonically advance only that persona's read position, update its unread count, and remain idempotent under repeats or concurrent older acknowledgements. | Read-state and concurrency PostgreSQL tests |
| REQ-006 | When a connection is removed or either participant blocks the other, existing conversation history shall remain readable to both participants but new user messages shall fail with one non-disclosing error; unblocking alone shall not restore send permission until the pair reconnects. | Lifecycle/privacy tests and live smoke |
| REQ-007 | When the delivery gate validates this slice, the system shall exercise conversation creation, typed system and user messages, owner/participant privacy, unread transitions, history persistence, connection/block send policy, and concurrent ordering through real migrations and PostgreSQL. | `bin/gate.sh --diff` |

## Scope

- In: one-to-one conversations for accepted persona pairs, acceptance-created
  typed system messages, bounded user text, conversation inventory, bounded
  history pagination, per-participant unread state, monotonic read
  acknowledgement, connection/block send authorization, stable JSON contracts,
  PostgreSQL race tests, live smoke, and documentation.
- Out: attachments, edits/deletes, reactions, group conversations, arbitrary
  client-created system messages, game-specific system payloads, notification
  cursors, WebSockets, presence/typing indicators, search, retention controls,
  moderation, encryption beyond normal transport/storage policy, and Git
  delivery.

## Links

- Intake:
- Pipeline spec: [completed spec](../../pipeline/completed/private-inbox-conversations-and-messages.spec.md)
- Architecture: [system overview](../../../architecture/system-overview.md)

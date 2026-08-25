---
title: TICKET-011-durable-persona-sync-and-websocket-notifications
status: closed
ticket_number: 011
type: feature
created: 2026-08-24
closed: 2026-08-24
intake:
pipeline_spec: docs/planning/pipeline/completed/durable-persona-sync-and-websocket-notifications.spec.md
---

# TICKET-011-durable-persona-sync-and-websocket-notifications

## Summary

Add a persona-local durable change feed and authenticated advisory WebSocket
notifications so clients can recover missed social and inbox changes through
REST after reconnect without treating the socket as authoritative.

## Why

Private conversations are durable, but a client currently has no bounded way
to learn which resource changed while it was offline. The first playable needs
one recovery cursor before challenges and turns can reuse the same notification
boundary.

## EARS requirements

| ID | EARS requirement | Verification |
|---|---|---|
| REQ-001 | When a social or inbox mutation changes an owned persona's visible state, the system shall append the appropriate typed event to that persona's monotonically ordered durable feed in the same PostgreSQL transaction; failed and idempotent no-op mutations shall append nothing. | Transaction and retry PostgreSQL tests |
| REQ-002 | When an authenticated account requests a sync baseline or reads after a retained cursor for an owned persona, the system shall return only that persona's bounded ascending events, a stable next cursor, and explicit pagination/reset state without exposing account ownership or another persona's feed. | Multi-account cursor/API tests |
| REQ-003 | When more than the retained per-persona event bound accumulates, the system shall prune only that persona's oldest events and tell a client with an expired cursor to take a new baseline and full REST snapshot before resuming incremental reads. | Retention/reset PostgreSQL tests |
| REQ-004 | When an authenticated owner upgrades the persona notification route to WebSocket, the system shall send a ready document and advisory changed hints only for that persona; absent, malformed, foreign, or invalid-session handshakes shall remain non-disclosing. | Real WebSocket and multi-account tests |
| REQ-005 | When a transaction commits a persona event, PostgreSQL notification fan-out shall wake connected server instances after commit; rollback shall produce neither a durable event nor a live hint, and lagged socket consumers shall be told to recover through REST. | Commit/rollback listener and socket tests |
| REQ-006 | When connection request, acceptance/removal/block state, inbox message, or read state changes, the system shall emit the documented minimal invalidation type for every affected persona without putting private message content, peer read state, block direction, or account identity into the event or WebSocket payload. | Event-mapping, exact-shape, and privacy tests |
| REQ-007 | When the delivery gate validates this slice, the system shall exercise migrations, durable cursor recovery, pruning/reset, transaction coupling, authenticated WebSocket delivery, reconnect recovery, and the unchanged QML health connector against real PostgreSQL. | `bin/gate.sh --diff` |

## Scope

- In: persona-local durable sequences, typed social/inbox invalidations,
  baseline and bounded incremental REST reads, bounded retention with reset,
  authenticated WebSocket ready/change/recovery hints, PostgreSQL commit-time
  notification fan-out, mutation integration, tests, live smoke, and docs.
- Out: game/challenge event payloads, browser query-string credentials,
  authoritative socket commands, guaranteed one-hint-per-event delivery,
  presence/typing, message bodies in notifications, a QML login/persona UI,
  external brokers, cross-region delivery guarantees, and Git delivery.

## Links

- Intake:
- Pipeline spec: [completed spec](../../pipeline/completed/durable-persona-sync-and-websocket-notifications.spec.md)
- Architecture: [system overview](../../../architecture/system-overview.md)

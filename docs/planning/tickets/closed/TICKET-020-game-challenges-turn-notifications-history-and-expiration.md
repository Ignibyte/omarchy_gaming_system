---
title: TICKET-020-game-challenges-turn-notifications-history-and-expiration
status: closed
ticket_number: 020
type: feature
created: 2026-08-25
closed: 2026-08-25
intake:
pipeline_spec: docs/planning/pipeline/completed/game-challenges-turn-notifications-history-and-expiration.spec.md
---

# TICKET-020-game-challenges-turn-notifications-history-and-expiration

## Summary

Add the durable two-person challenge workflow that turns an accepted inbox
challenge into one exact-version game session, preserves challenge history and
expiration, and makes challenge and turn changes reconnect-safe through the
existing REST/cursor/WebSocket model.

## Why

The platform could already connect personas, maintain private inboxes, create a
version-pinned game session inside a trusted transaction, and notify session
participants after commands. This ticket supplied the public orchestration that
joins those foundations into the first-playable invitation flow.

## Outcome

All seven requirements passed. OmarchyGS now has durable exact-version
two-person challenges, typed private lifecycle messages, participant history,
server-owned expiry and pending limits, atomic acceptance into exactly one game
session, and cursor/WebSocket notification behavior that keeps REST as truth.
The final 16-stage gate passed with all 39 PostgreSQL tests and the live
PostgreSQL/API/QML smoke.

## EARS requirements

| ID | EARS requirement | Verification |
|---|---|---|
| REQ-001 | When an owned persona challenges a connected, unblocked persona to an exact available game version, the system shall create at most one pending challenge for the idempotency key, apply bounded pending-inventory and expiration policy, append a typed private inbox event, and notify both personas in one transaction. | Multi-account PostgreSQL creation, privacy, cap, idempotency, and rollback tests |
| REQ-002 | When either participant reads challenge inventory or detail, the system shall expose only participant-authorized records, public persona projections, immutable game/version identity, direction, status, expiry, and an accepted session link, with bounded pagination and retained terminal history. | Inventory/detail allowlist, pagination, history, and foreign/absent equivalence tests |
| REQ-003 | When the challenged persona accepts a pending unexpired challenge while the pair remains connected and unblocked, the system shall create exactly one version-pinned game session through the existing trusted transaction primitive, mark the challenge accepted, append the typed inbox transition, and notify both personas atomically; a retry shall return the same accepted result. | Acceptance, retry, exact-version, rollback, and concurrent one-winner tests |
| REQ-004 | When the challenged persona declines or the challenger cancels a pending challenge, the system shall apply only the authorized terminal transition, retain it in history, append one typed inbox transition, notify both personas, and create no game session. | Directional authorization, idempotency, race, and no-session tests |
| REQ-005 | When a pending challenge reaches its server-owned expiry, any subsequent read or mutation shall resolve it to expired before returning or acting, retain the terminal record, deny acceptance, and create no game session or misleading turn state. | Controllable-clock expiration and boundary-race tests |
| REQ-006 | When a first-use game command commits, each participant shall receive the existing durable `game_session_changed` cursor event and advisory live hint exactly once; semantic replay, conflict, rejection, or rollback shall emit none, and participant-authorized REST shall remain the state source. | Game command plus sync/WebSocket integration tests |
| REQ-007 | When challenge data crosses HTTP, inbox, sync, or game-session boundaries, account ownership, credentials, block direction, private catalog state, and game snapshots shall not enter challenge or WebSocket payloads. | Response allowlists, no-store assertions, privacy equivalence, and payload inspection |

## Scope

- In: two-person challenges between accepted unblocked connections; exact
  compiled game key/version; server-owned expiry; pending caps; client
  idempotency; incoming/outgoing/history inventory and detail; accept, decline,
  and cancel; typed inbox lifecycle messages; atomic session creation; durable
  sync invalidations and advisory WebSocket hints; migration, API docs, smoke,
  architecture, OpenWiki, and AAR evidence.
- Out: playable production rules, bot personas, match completion/results,
  rematches, spectators, tournaments, public invites, more than two human
  participants, cartridge launch UI, remote providers, scheduled background
  notification delivery, email/push, and Git delivery.

## Links

- Depends on: `TICKET-009`, `TICKET-010`, `TICKET-011`, `TICKET-012`, `TICKET-013`
- Pipeline: [completed spec](../../pipeline/completed/game-challenges-turn-notifications-history-and-expiration.spec.md)
- Architecture: [system overview](../../../architecture/system-overview.md)

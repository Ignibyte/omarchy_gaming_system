---
title: Durable persona sync and WebSocket notifications
pipeline_id: 95a453b6-506c-4003-a9ea-921bafc47072
status: Phase 5 — Complete PASS
ticket: TICKET-011
ticket_doc: docs/planning/tickets/closed/TICKET-011-durable-persona-sync-and-websocket-notifications.md
aar: docs/planning/knowledge/aar/AAR-011-durable-persona-sync-and-websocket-notifications.md
created: 2026-08-24
---

# Durable persona sync and WebSocket notifications — spec

## Intent

Ship the reconnect-safe notification boundary: PostgreSQL owns persona-local
durable events, REST owns baseline and recovery, and authenticated WebSockets
only wake clients to fetch newer truth.

## Scope

- In: the seven EARS requirements in `TICKET-011`, exact event and socket
  unions, transaction integration across implemented social/inbox mutations,
  bounded retention/reset semantics, tests, live smoke, and documentation.
- Out: game-specific events, client-to-server socket commands, public socket
  credentials, presence, QML authentication UI, external brokers, commits,
  pushes, and pull requests.

## Acceptance criteria (EARS)

See the seven requirements in
[`TICKET-011`](../../tickets/closed/TICKET-011-durable-persona-sync-and-websocket-notifications.md#ears-requirements).

## Locked decisions

| # | Decision | Why |
|---|---|---|
| 1 | Each persona owns an independent positive sync cursor and at most 10,000 retained invalidation events. | Prevents cross-persona activity leakage and bounds durable storage while leaving ample reconnect history. |
| 2 | `GET /v1/personas/{persona_id}/sync` with no `after` returns an empty baseline page at the current cursor. Supplying `after` returns up to 100 ascending events; an expired cursor returns an empty page with `reset_required: true` and a new baseline cursor. | A client can read a baseline, fetch full REST snapshots, then close the race with incremental reads; the same flow repairs retention gaps. |
| 3 | The initial event union is `connection_requests_changed`, `connections_changed`, `blocks_changed`, and `conversation_changed { conversation_id }`. | Gives clients the narrowest resource invalidation without message bodies, peer read positions, account identity, or block direction. Ticket 012 may add reviewed game variants by forward migration. |
| 4 | Event insertion, per-persona sequence allocation, retention pruning, and `pg_notify` execute inside the owning mutation transaction. PostgreSQL delivers duplicate-folded persona wakeups only after commit. | Couples recovery truth to state changes, produces no rollback hint, and lets every server instance share the same advisory wakeup channel without an external broker. |
| 5 | Successful idempotent retries and no-op deletes append no event. A new request invalidates request inventories for both personas; acceptance invalidates requests, connections, and the conversation for both; relationship removal invalidates requests/connections for both; block/unblock invalidates only resources that actually changed; message sends invalidate the conversation for both; a forward read change invalidates it only for the reader. | Prevents event amplification and makes every feed item correspond to an observable resource change. |
| 6 | `GET /v1/personas/{persona_id}/sync/live` requires the normal `Authorization: Bearer` header. Query-string credentials are unsupported. | Reuses the private session boundary without leaking tokens into URLs or access logs. |
| 7 | A PostgreSQL listener feeds a process-local broadcast hub. Sockets receive only `ready { cursor }`, `changed`, or `resync_required`; broadcast lag explicitly sends the recovery hint. | WebSockets remain lossy advisory signals and cannot become a second data contract. |
| 8 | The hub permits at most five live sockets per persona, 20 per account, and 256 per process. The route limits frames and assembled messages to 1 KiB; unexpected client data is rejected while ping/close control flow remains supported. | Bounds authenticated connection exhaustion by both resource and principal and keeps the route server-to-client only. |
| 9 | `app::router` retains its test-friendly local hub constructor, while `main` creates the shared hub, establishes the PostgreSQL listener before serving, and stops it on shutdown. | Existing request tests need no background task; runtime and real WebSocket tests explicitly prove listener wiring. |
| 10 | Prepared sockets retain only account/session UUID authority, never the raw token. The serving loop revalidates without touching `last_used_at` before persona hints and every 30 seconds, closing on revocation, expiry, inactive account, or database uncertainty. | Long-lived transports must not outlive the session boundary or keep an otherwise idle session alive. |

## Linked artifacts

- Ticket: [TICKET-011](../../tickets/closed/TICKET-011-durable-persona-sync-and-websocket-notifications.md)
- Architecture: [system overview](../../../architecture/system-overview.md)
- Intake:

## Phase plan

| Phase | Deliverable | Exit gate |
|---|---|---|
| 1 Plan | Ticket, spec, notes, open AAR, recalled cursor/privacy/concurrency rules | observable recovery contract recorded |
| 2 Design | Schema, REST/socket unions, retention, transaction map, manifest, regression table, CodeGraph evidence | actionable reconnect-safe design |
| 3 Implement | Sync domain, migration, routes/socket hub, mutation hooks, tests, smoke, and docs | focused checks and self-review |
| 3.5 Inspect | Correctness/security/concurrency/privacy/operations ledger and fixes | fresh CodeGraph receipt |
| 4 Validate | Focused tests and canonical diff gate | matching delivery receipt |
| 5 Complete | EARS audit, OpenWiki lifecycle, submitted AAR, closed ticket and archive | no silent drops and matching wiki receipt |
| Delivery | Fresh gate, staged review, authorized commit | matching receipt |

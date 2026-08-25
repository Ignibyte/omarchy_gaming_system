---
aar: AAR-011-durable-persona-sync-and-websocket-notifications
ticket: TICKET-011
pipeline: durable-persona-sync-and-websocket-notifications
status: submitted
opened: 2026-08-24
submitted: 2026-08-24
effectiveness: 5
---

# AAR-011-durable-persona-sync-and-websocket-notifications

## Recalled at plan

| ID or source | How surfaced | Useful? |
|---|---|---|
| `PR-omarchy-gaming-system-scope-public-cursors-to-resource-001` | Knowledge-register search and Ticket 010 AAR | Yes — the durable cursor must be persona-local rather than a global activity oracle. |
| `PR-omarchy-gaming-system-bound-owner-inventories-at-write-001` | Knowledge-register search and Ticket 010 AAR | Yes — durable event retention needs an explicit stored-cardinality and reset contract. |
| `PR-omarchy-gaming-system-lock-social-pairs-before-state-001` | Knowledge-register search and completed social/inbox notes | Yes — event insertion must remain inside established mutation transactions without reversing root locks. |
| OpenWiki product/runtime pages and system overview | Generated evidence recall | Yes — REST is authoritative recovery and WebSockets only signal change. |

## What happened

OmarchyGS gained an owner-scoped durable recovery boundary for social and inbox
state. Each affected persona owns an independent monotonic cursor with at most
10,000 retained typed invalidations. REST supplies baselines, bounded ascending
pages, and explicit reset state; a PostgreSQL listener feeds advisory
WebSockets that carry only ready, changed, or resynchronize hints. Event append,
pruning, and notification share the state-changing transaction, so rollback and
idempotent no-op paths produce neither durable events nor live hints.

Inspection found a retention-boundary read race and three WebSocket weaknesses.
A concurrent prune could create an undetected gap between the retention check
and page fetch. Axum's default frame limits were broader than this server-only
protocol required, one account could consume process capacity through many
personas, and established sockets could outlive revoked or expired sessions.
The implementation now verifies the first fetched cursor, caps decoded frames
and messages at 1 KiB, admits at most five sockets per persona and twenty per
account within the 256-process cap, and reauthorizes UUID-only socket authority
without refreshing session idle time.

The Codex Security scan reviewed the complete frozen diff and reported one
medium and two low findings; all were fixed and covered by real TCP/PostgreSQL
tests. CodeGraph inspection traced the route, session, hub, listener, and
mutation blast radius, while direct test inspection covered its incomplete test
associations. The canonical gate passed 27 local tests, all 28 migrated
PostgreSQL tests, and the complete REST recovery plus unchanged QML smoke.

## Failures captured

| ID | Failure | Where surfaced |
|---|---|---|
| `BF-omarchy-gaming-system-sync-retention-read-race-001` | Separate `READ COMMITTED` retention and fetch snapshots could let concurrent pruning remove the expected first event and return a silently gapped page. | Correctness inspection of `sync::list_events`. |
| `BF-omarchy-gaming-system-websocket-decoder-defaults-001` | The socket rejected client data only after Axum's much larger default decoder limits had accepted it. | Codex Security resource-bound review. |
| `BF-omarchy-gaming-system-websocket-principal-exhaustion-001` | Persona and process quotas still allowed one account to spread sockets across personas and consume shared capacity. | Codex Security admission-fairness review. |
| `BF-omarchy-gaming-system-websocket-session-lifetime-001` | An upgraded socket no longer retained verifiable session authority, so revocation, expiry, or account disablement did not end it. | Codex Security session-lifecycle review. |

## Prevention rules captured

| ID | Rule | Why |
|---|---|---|
| `PR-omarchy-gaming-system-verify-retained-cursor-continuity-001` | After fetching a retained incremental page, verify that its first event is exactly the requested successor; otherwise require a baseline reset. | Snapshot-separated validation can become stale under concurrent pruning even when each query is individually correct. |
| `PR-omarchy-gaming-system-bound-live-transports-by-principal-001` | Bound long-lived transports by authenticated principal as well as resource and process, and release every counter through one lifetime-owned permit. | Resource-only limits do not prevent one principal from multiplying resource identities to monopolize shared capacity. |
| `PR-omarchy-gaming-system-reauthorize-live-transports-without-touch-001` | Retain non-secret session identity for long-lived transports and periodically reauthorize it without extending idle lifetime; close fail-closed on invalidity or uncertainty. | One-time upgrade authorization lets a connection outlive the account/session boundary, while ordinary touch authentication would keep idle sessions alive. |

## Architecture decisions

| ID | Decision | ADR |
|---|---|---|
| `AD-omarchy-gaming-system-persona-sync-boundary-001` | Use persona-local retained invalidation cursors as durable REST recovery truth and PostgreSQL-backed owner-scoped WebSockets only as lossy wakeup hints. | `docs/architecture/system-overview.md` and `openwiki/runtime-foundation.md` |

Every new ID above must also be appended to
`docs/planning/knowledge/INDEX.md`.

## Effectiveness

5/5. Recalled cursor privacy, bounded-inventory, and canonical social-lock rules
directly shaped the design. The security and concurrency passes still found
four material boundary defects that happy-path API tests would not have exposed.
Focused unit coverage, six migrated sync cases, real TCP socket tests, live REST
smoke, CodeGraph, OpenWiki, and the canonical gate supplied independent evidence.
All seven requirements and every validated finding were dispositioned; game
events, authoritative socket commands, QML authentication UI, and Git delivery
remained out of scope.

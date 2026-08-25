---
title: Game challenges, turn notifications, history, and expiration
pipeline_id: 54febbf7-107e-448b-ae18-11771fdd8ee6
status: Phase 5 — Complete PASS
ticket: TICKET-020
ticket_doc: docs/planning/tickets/closed/TICKET-020-game-challenges-turn-notifications-history-and-expiration.md
aar: docs/planning/knowledge/aar/AAR-020-game-challenges-turn-notifications-history-and-expiration.md
created: 2026-08-25
---

# Game challenges, turn notifications, history, and expiration — spec

## Intent

Ship the smallest public orchestration that lets one connected persona send an
inbox game challenge and lets the other accept it into exactly one durable,
version-pinned game session. Preserve private history, deterministic expiry,
retry/race safety, and reconnect recovery without adding playable rules or a
second notification truth.

## Scope

- In: all seven Ticket 020 EARS requirements; forward-only challenge/inbox/sync
  schema; domain and HTTP APIs; transaction reuse of the existing game-session
  primitive; exact game-version validation; connection/block authorization;
  typed inbox events; fixed server expiry; idempotency and pending caps;
  inventory/detail and terminal history; command notification proof; database,
  live smoke, architecture, OpenWiki, security, and AAR evidence.
- Out: game results/completion, bot opponent, production game definition,
  multi-party invitations, scheduled workers, external notification delivery,
  QML challenge screens, Game Cartridge launch, provider authority, and Git
  delivery.

## Acceptance criteria (EARS)

The authoritative requirements are REQ-001 through REQ-007 in
[`TICKET-020`](../../tickets/closed/TICKET-020-game-challenges-turn-notifications-history-and-expiration.md#ears-requirements).

## Phase 2 decisions

| # | Decision | Why |
|---|---|---|
| 1 | V1 challenges are exactly two-person, exact compiled game key/version invitations between an accepted unblocked pair. | It matches the first playable and existing one-conversation-per-pair model while avoiding premature multiplayer policy. |
| 2 | OmarchyGS owns one fixed expiry interval and pending-inventory cap; the client cannot choose unbounded expiry or quota. | Expiration and resource policy are server authority, not game or client input. |
| 3 | A challenge has one immutable request identity and a monotonic pending → accepted/declined/cancelled/expired lifecycle; no transition reopens or substitutes game versions. | Durable history and safe retry require terminal state to remain stable. |
| 4 | Acceptance calls the existing crate-private game-session creation function inside the same PostgreSQL transaction as the challenge transition, inbox event, and sync effects. | A challenge may never claim acceptance without its exact session, or create an orphan session without acceptance. |
| 5 | Challenge lifecycle events are typed server-authored messages in the pair's private conversation, while cursor events remain payload-minimal invalidations and WebSockets remain hints. | This makes the challenge arrive through the inbox without creating a competing durable notification system or leaking challenge details live. |
| 6 | Expiration is resolved transactionally on challenge reads and mutations in v1; no background scheduler is introduced. | The first slice can guarantee no expired acceptance and honest history without operating a worker whose delivery semantics are not yet needed. |

## Data and state contract

- `game_challenges` is the durable aggregate. It stores immutable challenger,
  challenged, game key/version, idempotency key, creation and server-owned
  expiry; its state moves once from `pending` to `accepted`, `declined`,
  `cancelled`, or `expired`.
- Accepted rows have exactly one `game_session_id`; all other states have none.
  Terminal rows have a resolution timestamp and remain queryable history.
- A challenger-scoped idempotency constraint prevents duplicate intent, while
  a partial uniqueness constraint prevents concurrent duplicate pending
  challenges for the same pair and exact game version.
- Pending policy is fixed at seven days and at most 100 outgoing plus 100
  incoming unexpired challenges per persona. These are server constants, not
  request fields.
- Inbox messages gain typed challenge references. They contain the public
  actor projection plus the challenge ID and, only for acceptance, the session
  ID. Current challenge state is fetched from the participant-authorized
  challenge endpoint rather than copied into mutable message payloads.
- Persona sync events gain `game_challenge_changed` with only a challenge ID.
  Existing `conversation_changed` and `game_session_changed` events keep their
  existing payload-minimal shapes.

## API contract

All routes are nested below `/v1/personas/{persona_id}/game-challenges`, use
bearer authentication and `Cache-Control: no-store`, and return only public
persona projections.

| Method and suffix | Meaning |
|---|---|
| `POST /` | Create an exact-version challenge from `idempotency_key`, `challenged_persona_id`, `game_key`, and `game_version`; return 201 for creation and 200 for an exact replay. |
| `GET /?limit=&before=` | Return participant-visible incoming, outgoing, pending, and terminal rows newest first with bounded opaque continuation. |
| `GET /{challenge_id}` | Return one participant-authorized challenge or the same not-found surface used for an absent row. |
| `PUT /{challenge_id}/accept` | Challenged-persona-only pending-to-accepted transition and exact session creation. |
| `PUT /{challenge_id}/decline` | Challenged-persona-only pending-to-declined transition. |
| `DELETE /{challenge_id}` | Challenger-persona-only pending-to-cancelled transition; return the terminal representation. |

The challenge representation contains ID, exact game key/version, direction,
status, both public personas, expiry/creation/update/resolution timestamps, and
an optional accepted session ID. It never returns the request idempotency key,
account IDs, relationship/block rows, game catalog internals, or game state.

## Transaction and lock contract

1. Authenticate and prove actor-persona ownership before starting a mutation.
2. Lock the two persona roots in canonical UUID order through the established
   connected-pair helper; this serializes relationship, block, duplicate, and
   per-persona pending-cap decisions.
3. Resolve due pending rows involving the locked actor(s) to `expired`, then
   lock/re-read the target challenge and evaluate the requested transition.
4. On acceptance, invoke `games::create_session` with challenger seat 0 and
   challenged seat 1 inside the same transaction, then persist the accepted
   link, typed inbox transition, challenge/conversation invalidations, and the
   session invalidations emitted by the existing game primitive.
5. On creation, decline, or cancel, persist the aggregate, one typed inbox
   message, and challenge/conversation invalidations before the same commit.
6. Exact retries return the durable result without appending another message,
   cursor event, live hint, or session. Competing transitions return a stable
   conflict; initialization, inbox, sync, or database failure rolls back all
   effects.

Inventory reads authenticate, lock the actor persona in a short transaction,
resolve participant-visible due rows, and then page the resulting history.
Expiry itself does not invent an inbox message or second notification channel;
the expiry timestamp and refreshed challenge inventory are authoritative.

## Verification contract

- Runtime unit tests cover exact manifest lookup and two-player eligibility.
- PostgreSQL tests cover creation/replay/collision, relationship and block
  privacy, pending limits, list/detail pagination, lazy expiry, every
  directional transition, exact-version session creation, rollback, and
  concurrent one-winner behavior.
- Inbox and sync tests cover every new typed payload and prove IDs remain
  participant-scoped and payload-minimal.
- Existing game-command integration tests are extended to prove one durable
  event per participant on first commit and none on replay, conflict,
  rejection, or rollback.
- The live smoke uses the production empty registry to prove a challenge to an
  unavailable game is rejected without partial rows or notification effects.

## Linked artifacts

- Ticket: `docs/planning/tickets/closed/TICKET-020-game-challenges-turn-notifications-history-and-expiration.md`
- Architecture: `docs/architecture/system-overview.md`, `docs/architecture/game-cartridges.md`
- Runtime predecessors: Tickets 009–013.

## Phase plan

| Phase | Deliverable | Exit gate |
|---|---|---|
| 1 Plan | Ticket, active spec/notes, open AAR, bounded first-playable slice | scope and exclusions fixed |
| 2 Design | Schema, state machine, APIs, transaction/lock order, file manifest, regression map | CodeGraph receipt and actionable design |
| 3 Implement | Migration, domain/routes, typed inbox/sync integration, tests/docs/smoke | focused loop green |
| 3.5 Inspect | Correctness, authorization, privacy, concurrency, expiry, and notification ledger | findings resolved plus fresh CodeGraph receipt |
| 4 Validate | Full matrix and canonical diff gate | matching delivery receipt |
| 5 Complete | EARS audit, OpenWiki, AAR, ticket archive | matching OpenWiki and delivery receipts |
| Delivery | Authorized commit/push only | explicit user authorization |

---
aar: AAR-020-game-challenges-turn-notifications-history-and-expiration
ticket: TICKET-020
pipeline: game-challenges-turn-notifications-history-and-expiration
status: submitted
opened: 2026-08-25
submitted: 2026-08-25
effectiveness: 5
---

# AAR-020-game-challenges-turn-notifications-history-and-expiration

## Recalled at plan

| ID or source | How surfaced | Useful? |
|---|---|---|
| `AD-omarchy-gaming-system-game-first-identity-001` | Knowledge register, product charter, and roadmap | Yes — challenges and matches are the primary product flow; message boards remain later. |
| `AD-omarchy-gaming-system-persona-social-pair-model-001` | Connections AAR and system overview | Yes — challenge authorization belongs to public persona pairs under private account ownership and canonical pair locking. |
| `AD-omarchy-gaming-system-private-inbox-model-001` | Inbox AAR and OpenWiki | Yes — a challenge arrives through the one durable participant-private conversation with typed server messages. |
| `AD-omarchy-gaming-system-persona-sync-boundary-001` | Sync AAR and OpenWiki | Yes — durable persona cursors recover challenge/turn changes; WebSockets remain advisory. |
| `AD-omarchy-gaming-system-version-pinned-game-session-foundation-001` | Game-session AAR and current domain code | Yes — challenge acceptance must call the exact-version transaction primitive and expose public personas only. |
| `AD-omarchy-gaming-system-idempotent-revision-command-boundary-001` | Game-command AAR and current integration tests | Yes — turn notification must occur only for a first-use committed command, never replay or rollback. |
| `BUL-001-initial-push-pending` | Bulletin preflight | Yes — work remains local and no delivery is authorized. |

## What happened

Ticket 020 joined the existing persona, connection, inbox, sync, and game
session foundations into the first public game-invitation workflow. A connected
persona can create one exact-version, two-player challenge with server-owned
expiry and bounded pending inventory. Both participants receive a typed private
inbox record and payload-minimal durable sync invalidation in the same
transaction. Participant-only inventory and detail retain terminal history
without exposing account, relationship, catalog, or snapshot state.

Acceptance calls the existing version-pinned game-session primitive in the
challenge transaction, fixes the challenger at seat 0 and challenged persona at
seat 1, and links exactly one session before committing the accepted state.
Decline, cancel, and lazy expiry remain session-free terminal states. Canonical
persona-pair locking and row locks make concurrent terminal transitions choose
one winner, while rollback tests prove initialization failure leaves no partial
challenge transition, message, sync event, or session.

Inspection found that challenge creation initially applied the current
connection and game-registry policy before consulting the durable idempotency
record. That made an exact network retry depend on mutable policy that was only
intended to admit new work. The flow now authenticates and owner-scopes the
actor, resolves the immutable challenger-scoped replay identity, and returns it
through the normal participant-authorized loader before applying current
admission checks to a new write. A regression changes both relationship and
registry state after creation and proves replay still returns the same result
without duplicate effects.

Two sealed Codex Security diff scans reviewed all 51 frozen worktree items and
reported no vulnerabilities. A durable-history churn candidate was rejected as
a diff-scoped vulnerability because the pre-existing authenticated private
message API already permits the same connected participants to append history
more directly; account throttling and retention remain future public-deployment
hardening. The final OpenWiki lifecycle reconciled the challenge architecture,
runtime, product boundary, and validation pages, while retaining one explicit
evidence-debt warning for the product-boundaries claim sidecar.

## Failures captured

| ID | Failure | Where surfaced |
|---|---|---|
| `BF-omarchy-gaming-system-challenge-replay-current-policy-order-001` | Durable challenge replay was gated by current connection and registry state, so an exact retry could fail after policy drift even though the original result remained participant-authorized and immutable. | Phase 3.5 correctness and idempotency inspection. |

## Prevention rules captured

| ID | Rule | Why |
|---|---|---|
| `PR-omarchy-gaming-system-resolve-durable-replay-before-current-admission-001` | After authenticating and owner-scoping the actor, resolve and validate a durable idempotency identity before current admission checks that apply only to new work; return it through the normal resource authorization path. | Retry semantics must replay the committed result despite later policy drift without weakening ownership, identity matching, or participant privacy. |

## Architecture decisions

| ID | Decision | ADR |
|---|---|---|
| `AD-omarchy-gaming-system-durable-game-challenge-orchestration-001` | Use a two-person, exact-version durable challenge aggregate with fixed server expiry and pending caps; deliver lifecycle records through the private inbox, recover state through REST and persona sync, keep WebSockets as hints, and create the game session atomically on acceptance. | `../../../architecture/system-overview.md`; `../../../../openwiki/runtime-foundation.md` |

Every new ID above must also be appended to
`docs/planning/knowledge/INDEX.md`.

## Effectiveness

5/5. All seven EARS requirements passed through production HTTP and PostgreSQL
boundaries, including privacy-equivalent detail, bounded history, exact replay,
directional terminal states, lazy expiry, exact-version atomic acceptance,
rollback, and a concurrent one-winner race. The 39-test database suite, live
PostgreSQL/API/QML smoke, two complete security scans, CodeGraph inspection,
OpenWiki lifecycle, and final worktree gate provide independent evidence. The
result is the smallest durable first-playable invitation flow and does not
claim playable production rules, challenge QML, background delivery, or remote
provider authority.

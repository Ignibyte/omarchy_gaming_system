---
aar: AAR-010-private-inbox-conversations-and-messages
ticket: TICKET-010
pipeline: private-inbox-conversations-and-messages
status: submitted
opened: 2026-08-24
submitted: 2026-08-24
effectiveness: 5
---

# AAR-010-private-inbox-conversations-and-messages

## Recalled at plan

| ID or source | How surfaced | Useful? |
|---|---|---|
| `PR-omarchy-bbs-owner-scope-account-resources-001` | Knowledge-register search and persona pipeline recall | Yes — acting personas and private inventories must derive ownership from the validated session. |
| `PR-omarchy-bbs-graph-coverage-is-advisory-001` | Knowledge-register search | Yes — message/unread authorization and races require direct tests plus executed PostgreSQL evidence. |
| `PR-omarchy-bbs-verify-the-vertical-slice-001` | Knowledge-register search | Yes — extend the live API path before the unchanged QML health connector. |
| `PR-omarchy-gaming-system-lock-social-pairs-before-state-001` | Ticket 009 AAR and knowledge register | Yes — sends must serialize with removal/blocking through the same canonical persona locks. |
| `AD-omarchy-gaming-system-persona-social-pair-model-001` | Ticket 009 AAR, system overview, and OpenWiki runtime page | Yes — one accepted pair can own one private conversation without exposing accounts. |

## What happened

OmarchyGS gained one durable private conversation per accepted canonical
persona pair. The actual pending-to-accepted transaction creates or reuses the
conversation and appends one server-authored `connection_accepted` message;
acceptance retry creates neither duplicate. Authenticated participants can
inventory conversations, read bounded stable history, send bounded body-only
user messages while the pair is connected and unblocked, and monotonically
acknowledge read state. History remains readable after disconnect or block,
while sends fail until the pair reconnects.

Inspection found three security issues before completion. Pending connection
requests had no stored-cardinality bound, public message sequences leaked
approximate activity in unrelated conversations, and the existing block/error
contract could still permit an indirect block-direction inference. After user
approval, request creation began enforcing 100 incoming and 100 outgoing
pending rows under the existing persona-root locks, and forward migration
`0008` converted previously applied global message identities and read/latest
cursors to conversation-local sequences. The block row and inventory remain
directly private, while the API now honestly documents that interaction denial
does not conceal every inference; inventing suppressed success state was
rejected as a larger, misleading contract.

The 59-item Codex Security scan completed with one medium and two low findings,
all dispositioned above. CodeGraph inspection traced the request, acceptance,
send, and response-layer blast radius; its automated test association remained
advisory, so direct PostgreSQL tests stayed authoritative. OpenWiki completed
after updating quickstart, runtime, product, and validation knowledge; it
reported one non-blocking pre-existing evidence-debt warning on an older
runtime claim. The final gate passed 25 local tests, all 22 migrated PostgreSQL
tests, the persistent migration upgrade, and the complete inbox/MFA/session/QML
smoke. The final gated state is
`14a54a2961767b1c253baebefe0f026f7373c8599bdae7375a49cfe4e08f3242`.

## Failures captured

| ID | Failure | Where surfaced |
|---|---|---|
| `BF-omarchy-gaming-system-unbounded-pending-inventory-001` | Stable request ordering did not bound how many durable incoming rows one persona could accumulate, so an authenticated account could amplify storage and owner-list work. | Codex Security inspection and `list_connection_requests` query review. |
| `BF-omarchy-gaming-system-global-private-message-cursor-001` | A database-global sequence was exposed as the public private-history cursor, allowing gaps to reveal approximate unrelated conversation activity. | Codex Security privacy review and migration/DTO inspection. |

## Prevention rules captured

| ID | Rule | Why |
|---|---|---|
| `PR-omarchy-gaming-system-bound-owner-inventories-at-write-001` | For an owner-scoped collection without pagination, enforce a stored-cardinality ceiling in the mutation transaction and serialize boundary races on an existing domain root. | Bounding only a read limit does not prevent accumulated storage or unbounded serialization, and a count outside the write lock is raceable. |
| `PR-omarchy-gaming-system-scope-public-cursors-to-resource-001` | Scope a public cursor or sequence to the resource whose history it orders unless cross-resource activity disclosure is an explicit documented contract. | Globally convenient ordering metadata can leak tenant or conversation activity even when message bodies and membership checks are private. |

## Architecture decisions

| ID | Decision | ADR |
|---|---|---|
| `AD-omarchy-gaming-system-private-inbox-model-001` | Give each accepted canonical persona pair one durable conversation with typed server/user messages, participant-private monotonic read positions, and conversation-local ordering; keep send authorization dependent on the live accepted unblocked relationship. | `docs/architecture/system-overview.md` and `openwiki/runtime-foundation.md` |
| `AD-omarchy-gaming-system-block-interaction-inference-policy-001` | Keep directional block storage and inventory directly owner-private, but do not claim that a generic denied interaction with a known persona conceals every indirect inference about block direction. | `docs/api.md` and `openwiki/product-boundaries.md` |

Every new ID above must also be appended to
`docs/planning/knowledge/INDEX.md`.

## Effectiveness

5/5. Prior persona-owner scoping and canonical pair-locking rules directly
shaped the design, while the security pass still found two material metadata
and availability blind spots that ordinary correctness tests had missed. User
approval cleanly separated code remediation from the explicit product-policy
tradeoff. Five migrated inbox tests, the new request-cap race case, persistent
migration smoke, CodeGraph, OpenWiki, and the canonical gate supplied distinct
evidence. All seven requirements and every validated finding were dispositioned;
WebSockets, general event cursors, and game payloads remained deferred, and no
commit, push, or pull request was performed.

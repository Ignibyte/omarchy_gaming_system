---
aar: AAR-023-keyboard-first-qml-connections-and-private-inbox
ticket: TICKET-023
pipeline: keyboard-first-qml-connections-and-private-inbox
status: submitted
opened: 2026-08-25
submitted: 2026-08-25
effectiveness: 5
---

# AAR-023-keyboard-first-qml-connections-and-private-inbox

## Recalled at plan

| ID or source | How surfaced | Useful? |
|---|---|---|
| `AD-omarchy-gaming-system-qml-onboarding-authority-boundary-001` | Ticket 022 AAR, completed notes, and current QML controllers | Yes — social screens must share the existing in-memory bearer without receiving or persisting it. |
| `PR-omarchy-bbs-owner-scope-account-resources-001` | Knowledge register and social API modules | Yes — every actor path must be the selected persona already proven owned by the bearer account. |
| `PR-omarchy-gaming-system-lock-social-pairs-before-state-001` | Knowledge register and connection/block contracts | Yes — client retries and generic errors must not invent pair state or block direction outside the serialized server result. |
| `PR-omarchy-gaming-system-scope-public-cursors-to-resource-001` | Knowledge register and inbox API | Yes — message paging and read state are conversation-local, not global activity cursors. |
| `PR-omarchy-gaming-system-mirror-authoritative-client-response-bounds-001` | Ticket 022 AAR and API documentation | Yes — every new inventory, profile, message, timestamp, and pagination validator must match the server contract exactly. |
| Product charter and roadmap | Product preflight | Yes — connections and inbox are the next usable two-person client outcome, while challenge/gameplay has a separate production-game dependency. |

## What happened

Ticket 023 extended the Ticket 022 account/persona shell into a usable
keyboard-first social and private-inbox client without widening credential or
server authority. One dedicated controller shares the bearer-owning API only
through a selected-persona request gateway. The new screens cover exact-handle
connection requests, relationship and private-block lifecycle, bounded
conversation inventory, ascending/paged history, plain-text user and typed
system messages, body-only send, unread acknowledgement, manual REST refresh,
and complete invalid-session cleanup.

The deterministic corpus grew from 19 to 24 passing QML cases and exercises the
production root at the 640×420 minimum against stateful normal and hostile
fixtures. The migrated smoke now runs real QML registration, social/inbox, and
MFA scenarios; its social path selects one of two real personas, loads their
accepted conversation, commits a private reply, and verifies the corresponding
payload-minimal sync event. The 18-stage diff gate passed, and the fixed-snapshot
Codex Security review reported no findings.

A delivery-time gate rerun exposed a timing race in the clean-clone Door
Legends replay test. The platform projection can commit before the provider
worker marks its outbox attempt delivered, so the test could requeue while the
original attempt was still completing. Waiting for the durable delivered state
before requeueing removed that ambiguity; the focused authority pilot then
proved both the original callback and exact replay. A refreshed incremental
Codex Security scan found no reportable issue in that final delta.

OpenWiki reconciled the quickstart, runtime, and validation pages. The first
completion attempt returned complete but could not replace the prior pipeline
receipt because the spec had not yet recorded Phase 4 PASS; the recalled Ticket
021 rule exposed that mismatch. After advancing the spec, a second lifecycle
finished cleanly and wrote a receipt for pipeline
`1dc98b6c-4c08-4ded-8c99-e1d58e9ac1a8`.

## Failures captured

| ID | Failure | Where surfaced |
|---|---|---|
| `BF-omarchy-gaming-system-qml-bodyless-xhr-null-payload-001` | Qt XHR serialized `send(null)` as a four-byte `null` payload for a bodyless social `PUT`; the fixture left those unread bytes on the keep-alive connection and parsed the next request as HTTP 501. | Stateful social fixture followed a successful mutation with an immediate durable refresh. |
| `BF-omarchy-gaming-system-provider-replay-requeue-race-001` | The clean-clone provider replay test requeued an outbox row after observing the platform projection but before the provider committed its first delivered status, allowing the original worker update to consume the simulated replay. | Delivery-time Stage 18 diff-gate rerun. |
| `BF-omarchy-gaming-system-openwiki-phase-receipt-sequencing-001` (recalled) | The first OpenWiki run could not bind the Ticket 023 completion receipt because the spec still said Inspect PASS. | Receipt readback after `openwiki_finish`; corrected before archive or delivery. |

## Prevention rules captured

| ID | Rule | Why |
|---|---|---|
| `PR-omarchy-gaming-system-preserve-bodyless-qml-requests-001` | When a QML request has no document, call `XMLHttpRequest.send()` with no argument; assert zero request bytes and immediately reuse the connection in a stateful fixture. | JavaScript `null` is a value, not absence, and Qt may serialize it even for methods whose API contract is bodyless. |
| `PR-omarchy-gaming-system-observe-delivery-before-requeue-001` | Before manually requeueing a durable outbox row in a replay test, wait for the producer's original attempt to reach its committed delivered state. | Downstream projection proves callback acceptance, not completion of the producer's asynchronous outbox acknowledgement. |

## Architecture decisions

| ID | Decision | ADR |
|---|---|---|
| `AD-omarchy-gaming-system-qml-social-inbox-authority-boundary-001` | Keep the shared process-memory bearer behind the onboarding authority controller; derive every social/inbox actor path from the selected owned persona, consume durable REST truth explicitly, and defer polling/WebSocket lifetime to a separate client transport slice. | `docs/architecture/system-overview.md` |

Every new ID above must also be appended to
`docs/planning/knowledge/INDEX.md`.

## Effectiveness

High. All eight EARS requirements have deterministic evidence, the real
PostgreSQL/Rust/QML path commits a two-account private message, security review
found no reportable issue, the canonical 18-stage gate is green, and the
OpenWiki completion receipt is bound to the current pipeline. Challenge,
catalog, launch, gameplay, and live-hint presentation remain explicitly outside
this slice instead of being implied by fixture-only UI.

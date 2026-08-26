---
title: Keyboard-first QML connections and private inbox — notes
pipeline_id: 1dc98b6c-4c08-4ded-8c99-e1d58e9ac1a8
---

# Keyboard-first QML connections and private inbox — running notes

Chronological evidence and decisions. If a test or command did not run, these
notes must not claim it passed.

## Phase 1 — Plan

- Recall: `main` began clean and synchronized at delivered Ticket 022 commit
  `cfcc2e2`; there is no active pipeline or critical bulletin. PostgreSQL is
  healthy, and CodeGraph 1.5.0 plus the Codex-only OpenWiki 0.3.3 integration
  passed provenance readiness.
- Recall: Ticket 022 established the process-memory-only bearer/MFA boundary,
  strict endpoint and response validation, a selected owned persona, and
  keyboard-first accessible controls. Its AAR rules require the shared XHR to
  invalidate generations before deferred abort, exact client/server bounds,
  protected secret test handoffs, production-root QML compilation, and an
  explicitly owned headless Qt environment.
- Recall: Tickets 009–011 already implement the server-side persona social
  graph, private blocks, durable private conversations, conversation-local
  message history/unread state, persona cursor recovery, and hint-only
  WebSockets. Owner scope always derives from the bearer account and selected
  acting persona; same-account and blocked targets remain generically
  unavailable, while block inventories are private.
- Recall: the API exposes exact public-handle lookup, bounded 100-entry
  request/connection/block inventories, bounded 100-conversation inventory,
  ascending message pages with `next_before`, exact tagged user/system
  variants, body-only sends, and monotonic participant-private read
  acknowledgements. Conversation history survives disconnect/block although
  sending does not.
- Recall: the product charter's first-playable outcome requires two people to
  connect and use an inbox before challenge/gameplay. The next roadmap line is
  broad, but Ticket 022 explicitly deferred social/inbox and game presentation
  as separate client authority surfaces.
- Gap recorded for the next slice: production currently advertises only
  one-human Signal Siege v1 and optional one-human Door Legends v1. The
  challenge API requires an exact game admitting two humans, so a later
  challenge/gameplay ticket must add or deliberately select a production
  two-person definition; fixture-only UI cannot prove the private-alpha
  challenged-match outcome.
- Decision: Ticket 023 completes the persona-social and private-conversation
  client boundary, including blocks and history, but not challenge or game
  screens. This is one shippable authority slice rather than a cosmetic subset.
- Decision: refresh is explicit on screen entry and player action. Automatic
  WebSocket hints or polling would introduce a second concurrent client
  transport/lifetime and remain a later reviewed ticket.

## Phase 2 — Design

- CodeGraph design exploration traced the existing connection and inbox route
  handlers through `connections.rs` and `inboxes.rs`. The affected server
  symbols have only the Axum application as callers, so this client slice does
  not change server code or migrations. Direct inspection covered the QML
  controller, shared `ApiClient`, production root, fixture harness, public API
  contract, and live-smoke orchestration that CodeGraph does not model fully.
- Architecture: `OnboardingController` remains the sole owner of `ApiClient`
  and its process-memory bearer. It exposes a session-gated `playerRequest`
  function, request-completion signal, cancellation, navigation allowlist, and
  invalid-session transition; it never exposes the token. A dedicated
  `SocialController` receives that gateway and the selected owned persona,
  derives every actor path from that persona ID, and serializes REST requests
  over the existing one-request transport. Screens receive only controller
  state and commands.
- Data flow: entering social or inbox explicitly refreshes durable REST truth.
  Social refresh chains request, connection, and private-block inventories;
  inbox refresh loads at most 100 conversations. Exact handle lookup is public
  but allowed only inside an active selected-persona session, validates the
  local handle grammar, rejects self, then issues the UUID connection command.
  Mutation success refreshes affected inventories. No timer, polling loop, or
  WebSocket is introduced.
- Inbox flow: opening a conversation resets the local history and requests an
  ascending page of at most 50 exact message variants. `next_before` is the
  only older-page cursor and older pages prepend after strict sequence
  validation. The composer hands off only `{body}`, clears after the transport
  accepts the request, and appends the validated committed response. Opening
  unread history acknowledges only the latest loaded message ID; monotonicity
  remains server-authoritative.
- Trust and privacy: public persona profiles use the exact seven-field schema;
  inventory wrappers, conversation summaries, read receipts, and tagged
  messages reject extra/missing fields, invalid UUID/timestamp/sequence/bounds,
  unknown variants, and oversized lists. Rendering is `Text.PlainText` only.
  `connection_unavailable` and `conversation_unavailable` map to generic
  policy text. No UI state models whether another persona blocked the actor.
  A valid `401 invalid_session` cancels work, clears social state, and asks the
  authority owner to clear bearer, personas, and selection.
- Failure/concurrency: each action stores an expected generation and operation;
  superseded completions are ignored. Transport/protocol failures retain the
  last validated inventory/history where safe, set a bounded retryable error,
  and never accept partial response data. A persona change, logout, or back to
  server configuration resets social state and cancels the shared request.
- File manifest: add `client/qml/SocialController.qml`,
  `screens/SocialScreen.qml`, and `screens/InboxScreen.qml`; update
  `OnboardingController.qml`, `Main.qml`, and `HomeScreen.qml`; extend the
  deterministic fixture and add social/inbox QML tests; update the focused QML
  runner and real migrated smoke; reconcile roadmap, API/client architecture,
  constitution gate text, generated OpenWiki, pipeline artifacts, and AAR.
- Regression plan: compile the production QML root; run the existing 19-case
  onboarding/transport corpus plus new keyboard social/inbox happy, empty,
  pagination, policy, invalid-session, and hostile-schema cases at 640×420 and
  920×600; run a real two-account/persona acceptance and bidirectional message
  flow through PostgreSQL/server/QML; then CodeGraph inspection, Codex Security
  diff scan, fast gate, and canonical 18-stage diff gate.
- Design receipt: worktree-bound CodeGraph receipt for pipeline
  `1dc98b6c-4c08-4ded-8c99-e1d58e9ac1a8`, state hash
  `e270380ce0f52b3d18b8efc7430bc452a22cf7c06da9bf96fa9aefae3805bf05`.

## Phase 3 — Implement

- Built: one bearer-owning QML authority gateway, a dedicated strict social
  controller, home/social/inbox navigation, exact-handle connection requests,
  request/connection/private-block inventories and actions, bounded
  conversation inventory, ascending history and older-page recovery, exact
  plain-text user/system variants, body-only sends, read acknowledgements,
  generic policy errors, and full invalid-session cleanup.
- Built: keyboard/accessibility layouts at the 640×420 minimum; a stateful
  deterministic HTTP fixture with public lookup, bearer/actor checks,
  relationship mutation, history, send/read, malformed, oversized, and 401
  cases; and migrated QML controller evidence against two real accounts,
  accepted connection, durable conversation, and committed private reply.
- Built: Stage 16 now runs 24 deterministic onboarding/social/inbox cases and
  three real QML scenarios (registration, social/inbox, MFA) before completing
  the existing API/game/sync smoke. README, architecture, roadmap, and
  constitution gate text describe the new shipped boundary.
- Deviation: the existing focused script remains named
  `test-qml-onboarding.sh` because it is already the canonical QML access-shell
  corpus; Ticket 023 extends that corpus rather than adding a second fixture
  process and lock. Its output now proves the full keyboard-first player shell.
- Focused evidence: `scripts/test-qml-onboarding.sh` passed 24/24. Live
  evidence: `scripts/dev.sh --smoke-test` passed the deterministic corpus plus
  real registration, social/inbox, and MFA QML scenarios against migrated
  PostgreSQL and the production Rust server.

## Phase 3.5 — Inspect ledger

| # | Lens | Finding | Severity | Disposition |
|---|---|---|---|---|
| 1 | Correctness and EARS coverage | The implemented controller/screens and deterministic/live evidence cover all eight requirements. Social mutation refreshes durable REST state, history pages preserve strict sequence order, and invalid sessions clear authority. | None | PASS. |
| 2 | Authentication and authorization | `ApiClient` remains the sole bearer owner; player requests require a live session and validated selected owned persona, while the Rust API remains the authorization authority for every actor/target pair. | None | PASS. |
| 3 | Input, injection, privacy, and secrets | API documents use exact bounded schemas; peer data is rendered as `Text.PlainText`; block/policy errors stay generic; live credentials use NUL-delimited stdin and a symlink-resistant mode-0600 short-lived file. | None | PASS. Codex Security fixed-snapshot diff scan completed with zero reportable findings: `/tmp/codex-security-scans/omarchy_bbs/cfcc2e2_local_20260825_6nH0mb/report.md`. The delivery-time synchronization delta was separately rescanned with zero findings: `/tmp/codex-security-scans-gVuCBs/omarchy_bbs/cfcc2e27d7267d6649506543a5ecccc6759e8909_20260826T003512Z_4b0be9bn/report.md`. |
| 4 | Concurrency and stale state | One shared XHR serializes player requests, generation+operation matching rejects stale completions, actor/session changes cancel pending work, and pagination validates both page-local and cross-page sequence order. | None | PASS. |
| 5 | Database and migration integrity | No Rust, SQL, or migration source changed. The real migrated PostgreSQL smoke exercised existing owner scope, durable connection/conversation/message state, read acknowledgement, and minimal sync invalidation. | None | PASS. |
| 6 | QML usability, keyboard, and accessibility | Home, social, and inbox entry/actions have explicit focus, Enter activation, Escape recovery, accessible names, minimum-layout scrolling, and deterministic 640×420 coverage. | None | PASS. |
| 7 | Simplification and reuse | The slice reuses the one authority gateway, existing platform controls, existing REST contracts, and the existing deterministic/live QML harness. It introduces no polling, WebSocket lifetime, duplicate bearer, or client-side policy model. | None | PASS. |
| 8 | HTTP request framing | Absent QML request documents used `send(null)`, which Qt serialized as a four-byte body and left unread bytes behind bodyless fixture commands. | Medium | RESOLVED: `ApiClient` now calls `send()` for absent documents; the fixture rejects bodies and immediately reuses the connection. |
| 9 | Blast radius | CodeGraph inspection confirmed the server route/domain graph is unchanged and the client change is confined to the shared QML authority, trusted screens, and validation harness. | None | PASS. Fresh inspect receipt state hash `7a350d5855ef59bc829d959d0ac5ca6ea4a7ccffe325ca1d3036ade300715db9`. |

## Phase 4 — Validate

- Tests run: `scripts/test-qml-onboarding.sh` passed all 24 deterministic
  account/persona/social/inbox cases; `scripts/dev.sh --smoke-test` passed the
  deterministic corpus, real QML registration, social/inbox, and MFA scenarios,
  and the complete migrated API smoke.
- Gate run: `bin/gate.sh --fast` passed. `bin/gate.sh --diff` then passed all 18
  stages, including 44 PostgreSQL tests, Stage 16 QML evidence, provider
  conformance, and the clean-clone Door Legends pilot. A delivery-time rerun
  then exposed a pre-existing race in the pilot replay setup: projection could
  become visible before the provider committed its first outbox delivery, so
  the test's manual requeue could be consumed by that original update. The
  harness now waits for the delivered outbox state before requeueing, and the
  focused clean-clone pilot passed with both original and replay callbacks.
  The exact final worktree then passed all 18 stages again. Canonical receipt
  hash: `5ebf0005e55e64f8d79b200c63bc45dc7ff991fcd6bb49f5aa3ed3c2f7081ccc`.
- Skips or pre-existing failures: none. The ordinary unit loop's
  environment-dependent ignored cases ran in their owning later gate stages
  and passed.

## Phase 5 — Complete

- Acceptance-criteria audit: REQ-001 PASS (four social inventories and explicit
  states); REQ-002 PASS (exact handle and full connection lifecycle); REQ-003
  PASS (selected actor, private blocks, generic policy); REQ-004 PASS (bounded
  conversations, ascending typed/plain history, older pages); REQ-005 PASS
  (body-only send and monotonic read); REQ-006 PASS (transport/protocol/size/
  stale/401 failures); REQ-007 PASS (keyboard, accessibility, 640×420); REQ-008
  PASS (24 deterministic cases and real migrated two-account QML message).
- Docs: README, system overview, roadmap, constitution Stage 16, and generated
  OpenWiki quickstart/runtime/validation pages now describe the shipped
  social/inbox boundary and explicit challenge/game/live-hint deferrals.
  OpenWiki ultimately finished cleanly and wrote the current-pipeline receipt;
  its first run preserved broad-page claims sidecars because unrelated stale
  evidence debt remained, which introduced no ungrounded completion claim.
- AAR: completed with two new failure IDs, two prevention rules, and one
  architecture decision; all five were appended to the knowledge register.
- Archive: Ticket 023 is closed; its spec and notes move together to completed.
  No active pipeline remains. Git delivery was explicitly authorized by the
  user on 2026-08-25.

## Defect and lesson ledger

| # | What happened | Root cause | Fix | Prevention |
|---|---|---|---|---|
| 1 | A successful bodyless QML `PUT` was followed by HTTP 501 on the keep-alive connection. | Qt's XHR serialized `send(null)` as four bytes (`null`) for the bodyless method; the small fixture correctly exposed the unread bytes before the next request line. | `ApiClient` now calls `send()` with no argument for absent documents and serializes JSON only when a document exists. | The stateful fixture rejects bodies on bodyless social commands and immediately follows each mutation with a durable refresh on the same connection. |
| 2 | A delivery-time Stage 18 rerun timed out waiting for the forced callback replay even though the first callback returned 204. | The platform result projection became visible before the provider's asynchronous worker committed its first outbox status update; the test requeued too early, and the original update consumed that pending state without creating a second attempt. | The pilot now waits for the provider outbox row to reach `delivered` before changing it back to `pending`. | Replay tests must observe the producer's durable delivery acknowledgement before mutating an outbox row to simulate redelivery. |

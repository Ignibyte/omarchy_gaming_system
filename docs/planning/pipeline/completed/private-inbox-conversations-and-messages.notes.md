---
title: Private inbox conversations and messages — notes
pipeline_id: 6ee75fc9-36ae-4660-997e-cf22d0adc11a
---

# Private inbox conversations and messages — running notes

Chronological evidence and decisions. If a test or command did not run, these
notes must not claim it passed.

## Phase 1 — Plan

- User directive: continue through the ordered five-ticket roadmap set. Ticket
  009 is archived with matching final gate and OpenWiki receipts, leaving no
  active pipeline; Ticket 010 is the next unchecked roadmap item.
- Recall: account ownership stays private; all social/game identity is persona
  scoped; owner-and-object authorization derives from a validated session;
  graph test association is advisory; shared pair state locks persona roots in
  canonical order; and the vertical slice requires migrated PostgreSQL, the
  real HTTP path, and the existing QML connector smoke.
- Nearest pipeline: Ticket 009 established a canonical accepted connection,
  private directional blocks, ordered pair locking, idempotent removal, exact
  public persona DTOs, and generic blocked-target failures. It intentionally
  emitted no inbox or notification state.
- OpenWiki recall: REST/JSON is durable truth, WebSockets remain future advisory
  notifications, and an implemented connection is the authorization foundation
  for private inboxes. Product scope calls for inbox threads and typed game
  messages but Ticket 010 must not invent the game schemas owned by Tickets
  012–013.
- Preflight: the next local ticket number was 010, the only bulletin remains a
  non-blocking warning about unconfirmed remote `main`, and pipeline tooling
  reported CodeGraph 1.5.0 plus OpenWiki 0.3.3 ready with Codex-only provenance.

## Phase 2 — Design

- API: `GET /v1/personas/{persona_id}/conversations?limit=` returns at most 100
  latest-active summaries. `GET
  /v1/personas/{persona_id}/conversations/{conversation_id}/messages?before=&limit=`
  returns an ascending bounded page and optional `next_before`. `POST` on the
  messages path accepts only `{ "body": ... }`. `PUT
  /v1/personas/{persona_id}/conversations/{conversation_id}/read/{message_id}`
  monotonically acknowledges through one message. All authenticated inbox
  responses are `Cache-Control: no-store`.
- Conversation summaries contain conversation ID, the other participant's
  seven-field public persona, unread count, optional explicit latest-message
  union, and created/updated timestamps. They expose neither account identity
  nor the peer's read position. History uses `type: user` with public sender and
  body or `type: system` with a nested typed `connection_accepted` actor.
- Persistence: migration `0007` adds one UUID-keyed conversation per canonical
  persona pair with low/high read cursors and typed messages. Forward-only
  migration `0008` converts the original database-global identity into a
  conversation-local sequence, remaps existing latest/read cursors, and binds
  latest-message integrity to `(conversation_id, message_sequence)`. Messages
  retain a public UUID, exact user/system content constraints, optional system
  actor, and server timestamp. Existing accepted rows are backfilled into
  conversations with one typed acceptance message using the retained
  addressee as actor.
- Acceptance integration: only the actual pending-to-accepted branch calls the
  inbox domain inside the existing connection transaction. It creates/reuses
  the pair conversation, serializes the conversation row, appends one system
  message, marks it read for the accepting addressee, and leaves it unread for
  the requester. An already accepted retry returns the existing connection
  without another message.
- Send concurrency: authenticate and owner-check the acting persona, resolve
  the immutable conversation pair, lock the persona roots in canonical order,
  verify the pair is accepted and unblocked, then lock the conversation and
  append. Removal and block use the same root locks, so either they commit first
  and send fails generically or send commits first and remains durable history.
- Reads do not require a live connection. Inventory/history membership derives
  from the owner-scoped actor and canonical conversation pair. Read
  acknowledgement locks only the immutable conversation row, verifies the
  target message belongs to it, and uses `GREATEST` so older/concurrent retries
  cannot move state backward.
- Errors: invalid/foreign actors share `persona_not_found`; invalid, absent, or
  foreign conversations share `conversation_not_found`; a message outside the
  conversation shares `message_not_found`; send denial after disconnect or in
  either block direction is `conversation_unavailable`; invalid body and query
  bounds are stable 422 responses. Authentication always precedes object
  disclosure.
- CodeGraph evidence: exploration traced acceptance from Axum through the
  connection transaction, canonical `lock_pair`, block check, and public DTO
  boundary. Direct inspection supplements its advisory test association. The
  design receipt for pipeline `6ee75fc9-36ae-4660-997e-cf22d0adc11a` matches
  this worktree.

### File manifest

| Path | Purpose |
|---|---|
| `migrations/0007_private_inbox.sql` | Add canonical conversations, monotonic per-participant read positions, typed user/system messages, indexes, constraints, and accepted-pair backfill. |
| `migrations/0008_conversation_local_message_sequences.sql` | Forward-migrate global identities and existing read/latest positions to unique conversation-local message sequences. |
| `crates/server/src/inboxes.rs` | Own inbox validation, membership/privacy, bounded inventory/history, unread calculations, monotonic acknowledgement, accepted-connection send checks, and acceptance-created system messages. |
| `crates/server/src/connections.rs` | Expose the canonical connected-pair lock to the inbox domain and invoke typed conversation creation only on an actual acceptance transition. |
| `crates/server/src/app.rs` | Add thin inbox routes/query/body parsing, explicit tagged DTOs, stable error mappings, and no-store responses. |
| `crates/server/src/inbox_api_tests.rs`, `main.rs` | Register and execute multi-account, exact-shape, unread, lifecycle, pagination, and concurrency tests against PostgreSQL. |
| `scripts/dev.sh` | Extend the existing two-account flow with conversation inventory, typed system/user history, unread/read state, and rejected post-removal send before QML. |
| `docs/api.md`, `README.md`, `docs/architecture/system-overview.md`, `docs/planning/ROADMAP.md` | Document inbox contracts/invariants, implemented roadmap state, and the still-future durable event/WebSocket layer. |
| Ticket/spec/notes/AAR/knowledge/OpenWiki | Preserve evidence, durable decisions, generated documentation, and completion state. |

### Regression plan

| Requirement | Evidence |
|---|---|
| REQ-001 | Acceptance/retry and migration-backed tests prove one pair conversation, one transition message, and no retry duplicate. |
| REQ-002 | Multi-persona/account inventory assertions prove participant filtering, peer public DTO, private read state, latest message, stable ordering, and no-store. |
| REQ-003 | User-message tests and live smoke prove bounded body-only input, server-owned metadata, sender-read/peer-unread behavior, and client system-field rejection. |
| REQ-004 | History tests prove 50/100 bounds, before pagination, ascending sequence, exact tagged unions, public actor/sender shapes, and non-disclosing foreign/absent access. |
| REQ-005 | Repeated, older, and concurrent acknowledgements prove monotonic read state and exact unread counts without changing the peer. |
| REQ-006 | Disconnect/block lifecycle test and live smoke prove durable readable history, generic send denial, and reconnect requirement after unblock. |
| REQ-007 | All new ignored SQLx tests run in the PostgreSQL tier and the expanded live inbox flow runs inside a final `bin/gate.sh --diff`. |

## Phase 3 — Implement

- Built: forward-only migration `0007` for canonical pair conversations,
  tagged messages, participant read cursors, accepted-pair backfill, and
  database constraints; the `inboxes` domain for inventory,
  bounded history, sends, acknowledgements, and acceptance events; thin Axum
  DTO/routes/error mappings; the connection-acceptance transaction hook; four
  multi-account PostgreSQL API tests; the live two-persona smoke lifecycle;
  and the API, architecture, README, and roadmap updates.
- Focused checks: `cargo check --workspace --all-targets` passed; `cargo test
  --workspace --all-targets` passed 25 non-database tests with 20 PostgreSQL
  tests intentionally ignored; `./scripts/test-database.sh` passed all 20
  PostgreSQL tests including the four new inbox tests; `cargo clippy
  --workspace --all-targets -- -D warnings` passed; `bash -n scripts/dev.sh`
  passed; and `./scripts/dev.sh --smoke-test` exercised acceptance, typed
  inventory/history, user send, unread acknowledgement, preserved history, and
  denied post-removal send before the QML protocol smoke exited successfully.
- Deviations: none from the approved manifest or transport boundary. The QML
  connector remains health-only and WebSockets remain explicitly owned by
  Ticket 011.

## Phase 3.5 — Inspect ledger

| # | Lens | Finding | Severity | Disposition |
|---|---|---|---|---|
| 1 | Correctness/EARS/concurrency | CodeGraph traced acceptance through the existing pair transaction into one transition-only inbox event, and traced send authorization through canonical pair locks before conversation locking. Direct migration/test inspection confirmed bounded ascending history, participant ownership, durable post-disconnect reads, `GREATEST` read cursors, and exact tagged DTOs. | None | Pass; fresh post-implementation receipt still required after any inspection fixes. |
| 2 | Security/availability | An authenticated account can manufacture many personas and durable pending requests for one victim, while `list_connection_requests` fetches and serializes the full inventory without a cardinality cap or pagination. | Medium | Fixed after user approval: new requests enforce 100 incoming and 100 outgoing pending rows under the existing persona-root locks; a PostgreSQL race test proves exactly one boundary request wins and inventories remain bounded. |
| 3 | Security/privacy | Public table-global inbox message sequences let a participant infer approximate unrelated private activity volume from gaps. | Low | Fixed after user approval: forward migration `0008` remaps existing state and new inserts allocate a sequence under the conversation lock; an isolated two-conversation test proves unrelated activity creates no gap. |
| 4 | Security/privacy | A known target's failed connection request plus the actor's own block inventory can indirectly reveal the target-to-actor block direction despite the generic error body. | Low | Accepted policy after user approval: block rows and inventories remain owner-private, but interaction denial is not claimed to prevent every inference. API and architecture docs state this residual disclosure; suppressed/fabricated request state is outside the product contract. |
| 5 | Security/cache correctness | Handler-level `no_store` covered successful inbox DTOs but Axum extractor rejections could bypass the handler. | Low | Fixed during inspection: an inbox-router response layer now applies `Cache-Control: no-store` to success, domain errors, and extractor rejections; PostgreSQL router tests cover rejected JSON and query parsing. |

- Codex Security reviewed all 59 frozen-diff workbench items with complete
  coverage. The sealed report is
  `/tmp/codex-security-scans-TFWcqD/omarchy_bbs/493749e2194df621640b229be4a5058fc872f30a_20260824T195900Z_67j2mqtp/report.md`.
  TAC display access could not be verified because its advisory connector was
  not connected; the scan itself completed normally.
- Approved remediation focused check: `cargo check --workspace --all-targets`
  passed and `./scripts/test-database.sh` passed all 22 PostgreSQL tests,
  including pending-cap races, conversation-local sequence isolation, and
  extractor-rejection no-store coverage.
- Post-fix runtime/schema evidence: `cargo test --workspace --all-targets`
  passed 25 local tests with 22 PostgreSQL tests intentionally ignored;
  warning-clean Clippy passed; and `./scripts/dev.sh --smoke-test` applied
  migration `0008` to the persistent development database before the complete
  live inbox and QML protocol smoke passed. Direct schema inspection confirmed
  the composite latest-message foreign key, per-conversation uniqueness, and
  contiguous existing local sequences.
- Fresh CodeGraph inspection traced the request cap through the sole Axum
  command caller, both system/user message insertion paths through the shared
  conversation lock/allocator, and the inbox response layer. Its automated
  test association remained incomplete, so the directly executed
  `connection_api_tests` and `inbox_api_tests` remain the authoritative
  coverage evidence. The inspect receipt matches the final gated worktree.

## Phase 4 — Validate

- Tests run: `cargo check --workspace --all-targets`, `cargo test --workspace
  --all-targets`, `cargo clippy --workspace --all-targets -- -D warnings`,
  focused `./scripts/test-database.sh`, persistent-database `./scripts/dev.sh
  --smoke-test`, direct migration/schema queries, and the canonical diff gate.
  Local tests passed 25/25 with the 22 database tests intentionally ignored;
  the PostgreSQL tier passed 22/22.
- Gate run: `bin/gate.sh --diff` printed `GATE GREEN [diff]` across all twelve
  stages, including the migrated database tests and live PostgreSQL → Rust API
  → QML smoke.
- Skips or pre-existing failures: no required check skipped and no test failed.
  The offscreen QML run emitted non-fatal `libEGL` DRI warnings, as in prior
  successful smokes.

## Phase 5 — Complete

- Acceptance-criteria audit:

  | Requirement | Final evidence | Result |
  |---|---|---|
  | REQ-001 | `acceptance_creates_one_private_conversation_and_one_typed_event` proves one canonical conversation and one transition-only system message across acceptance retry. | Satisfied |
  | REQ-002 | The same multi-account inventory test proves participant filtering, peer public DTOs, latest/unread state, private-field absence, and owner authorization. | Satisfied |
  | REQ-003 | `user_messages_are_body_only_and_drive_private_monotonic_unread_state` plus live smoke prove bounded allowlisted text, server-owned metadata, sender-read and peer-unread state. | Satisfied |
  | REQ-004 | `history_is_bounded_private_and_survives_disconnect_and_block` proves ascending local pagination, explicit tagged unions, bounded queries, and indistinguishable absent/foreign conversations. | Satisfied |
  | REQ-005 | The unread test and `concurrent_sends_serialize_and_concurrent_reads_never_move_backward` prove idempotent `GREATEST` acknowledgement and actor-only monotonic read state. | Satisfied |
  | REQ-006 | The lifecycle test and live smoke prove retained readable history, generic post-removal/block send denial, and reconnect required after unblock. | Satisfied |
  | REQ-007 | Two green canonical diff gates ran all 22 migrated PostgreSQL tests plus the full PostgreSQL → Rust API → QML smoke; the post-OpenWiki receipt matches state `14a54a2961767b1c253baebefe0f026f7373c8599bdae7375a49cfe4e08f3242`. | Satisfied |

- Docs: hand-maintained README, API, system overview, and roadmap are current.
  `$openwiki` update run `68dec5ec-32b7-4245-a1d8-d8d03cbc889e`
  reconciled quickstart, runtime, product-boundary, and validation claims and
  returned `status: complete`. It reported one non-blocking pre-existing
  evidence-debt warning for an older runtime-foundation claim; the new inbox
  claims were resolved before prose edits.
- AAR: `AAR-010` submitted on 2026-08-24 with two failures, two prevention
  rules, two architecture decisions, and matching knowledge-register entries.
- Archive: ticket closed and spec/notes moved together to `completed/`. No
  delivery action was authorized or performed.

## Defect and lesson ledger

| # | What happened | Root cause | Fix | Prevention |
|---|---|---|---|---|
| 1 | The initial design exposed a database-global message sequence in private conversation DTOs. | A convenient ordering primitive was treated as harmless public metadata. | Use per-conversation allocation and explicitly include cross-tenant metadata leakage in inspection. | Public cursors must be scoped to the resource boundary unless a broader cursor is itself part of the documented contract. |
| 2 | Pending request inventories had stable ordering but no cardinality bound. | Read bounding was reviewed for inbox history but not inherited social inventory. | Enforce race-safe incoming/outgoing caps at mutation time. | Every collection endpoint needs both query-cost and stored-cardinality analysis. |

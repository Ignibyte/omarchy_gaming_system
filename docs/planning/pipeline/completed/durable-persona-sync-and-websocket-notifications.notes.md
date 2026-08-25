---
title: Durable persona sync and WebSocket notifications — notes
pipeline_id: 95a453b6-506c-4003-a9ea-921bafc47072
---

# Durable persona sync and WebSocket notifications — running notes

Chronological evidence and decisions. If a test or command did not run, these
notes must not claim it passed.

## Phase 1 — Plan

- User directive: continue the ordered five-ticket roadmap set. Ticket 010 is
  archived with matching final gate and OpenWiki receipts, so Ticket 011 is the
  next unchecked slice and the only active pipeline.
- Knowledge recall: public cursors must stay within their resource/privacy
  boundary; owner-scoped collections need write-time bounds; persona/account
  identity stays separate; canonical persona locks serialize shared social
  state; CodeGraph coverage is advisory; and the real migration/API/QML path is
  the delivery proof.
- OpenWiki and architecture recall: REST/JSON is durable truth, WebSockets are
  advisory wakeups, and a cursor API must recover missed changes. Ticket 010
  explicitly reserves the general cross-domain cursor for this slice; its
  conversation-local message sequence must not be repurposed.
- Nearest pipeline: accepted social pairs now own durable private conversations
  with typed messages and monotonic unread state. Sends already share canonical
  persona locks with removal/blocking, while read acknowledgements lock the
  conversation only. Ticket 011 must append its event in each owning mutation
  transaction without inverting those lock orders.
- Smallest shippable boundary: persona-local invalidation events for the
  implemented connection/request/block/conversation resources; baseline plus
  bounded incremental REST; retained-history reset; and header-authenticated
  WebSocket ready/change/recovery hints. No message body or private state rides
  the feed or socket.

## Phase 2 — Design

- API: `GET /v1/personas/{persona_id}/sync?after=&limit=` authenticates and
  owner-scopes before parsing/disclosing feed state. With no `after`, it returns
  `{events: [], next_cursor, has_more: false, reset_required: false}` as the
  baseline to capture before full REST snapshots. With `after`, it returns an
  ascending page (default 50, max 100). A cursor older than retained history
  returns the current baseline with `reset_required: true`; negative/future
  cursors and invalid limits are stable 422 responses.
- Event JSON is an explicit tagged union. Social invalidations expose only
  `cursor`, `type`, and `created_at`; `conversation_changed` additionally
  exposes a participant-authorized conversation UUID. No event contains
  message text, persona/account records, read counts, block direction, or
  authentication material.
- Persistence: migration `0009` adds one `persona_sync_state` row on first use
  and a `(persona_id, event_sequence)` event table. A state-row lock allocates
  the next persona-local cursor, inserts an exactly shaped event, removes rows
  older than the newest 10,000, and calls `pg_notify` in the same transaction.
  No backfill is required: existing clients take a baseline and snapshot.
- Recovery: a client first records a baseline cursor, reads authoritative REST
  resources, then requests events after the baseline to close the snapshot
  race. It repeats that sequence when `reset_required` is true. `has_more`
  drains additional pages without skipping the last returned cursor.
- Mutation map: new request → request feeds for both; transitioned acceptance →
  request, connection, and conversation feeds for both; actual relationship
  deletion → request and connection feeds for both; new block → actor block
  feed plus pair feeds only when a relationship was deleted; actual unblock →
  actor block feed; new message → conversation feeds for both; forward read →
  actor conversation feed. No-op/idempotent branches produce no rows.
- Concurrency: social and send paths retain their persona-before-conversation
  locks, then lock sync state. Read acknowledgement retains conversation-before-
  actor-sync; it never later requests a persona root. Multi-persona mutation
  roots already serialize pair callers, and per-persona sync locks serialize
  independent conversation changes without a reverse dependency.
- WebSocket: the header-authenticated `/sync/live` route subscribes before
  reading the ready cursor, so a commit at handshake is represented by the
  baseline, a later hint, or harmlessly both. The PostgreSQL listener reconnects
  through SQLx and broadcasts only UUID payloads. Five sockets per persona,
  twenty per account, and 256 per process are admitted; lag emits
  `resync_required`; unexpected text or binary input closes the
  server-to-client-only channel.
- Operations: runtime startup establishes `LISTEN` before accepting HTTP. If
  startup cannot subscribe, the process fails rather than advertise live
  delivery. A later listener receive error is logged and retried; durable REST
  recovery remains available through transient notification loss.
- CodeGraph evidence: exploration traced all seven social/inbox mutation paths,
  their idempotent branches, persona/conversation lock order, transaction commit
  boundaries, the thin Axum callers, `AppState`, and `main`. Direct inspection
  supplements its incomplete automated test association. The design receipt for
  pipeline `95a453b6-506c-4003-a9ea-921bafc47072` matches this worktree.

### File manifest

| Path | Purpose |
|---|---|
| `migrations/0009_persona_sync_events.sql` | Add persona-local cursor state, exact typed invalidation rows, retention indexes/constraints, and the forward-extensible event discriminator. |
| `crates/server/src/sync.rs` | Own feed authorization, baseline/pagination/reset semantics, transaction-coupled append/prune/notify, the PostgreSQL listener, bounded broadcast hub, and socket loop. |
| `crates/server/src/connections.rs` | Append exact social invalidations only in state-changing transaction branches and preserve existing lock order/idempotency. |
| `crates/server/src/inboxes.rs` | Append participant conversation invalidations for acceptance, send, and forward read transitions. |
| `crates/server/src/app.rs` | Add sync REST/WebSocket routes, tagged DTOs, stable errors, no-store layer, and shared-hub router construction. |
| `crates/server/src/main.rs` | Register the sync module and test suite, establish the PostgreSQL listener, inject the shared hub, and stop its task during shutdown. |
| `Cargo.toml`, `Cargo.lock`, `crates/server/Cargo.toml` | Add the minimal stream/sink, JSON runtime, timer, and real WebSocket test dependencies. |
| `crates/server/src/sync_api_tests.rs` | Prove mutation mapping/idempotency/privacy, pagination/reset/isolation, commit/rollback hints, concurrency, socket authorization/delivery/limits, and reconnect recovery against PostgreSQL. |
| `scripts/dev.sh` | Extend live smoke with baseline, social/inbox event recovery, stable cursor progression, and retained QML health behavior. |
| `docs/api.md`, `README.md`, `docs/architecture/system-overview.md`, `docs/planning/ROADMAP.md` | Document the durable/advisory boundary, contracts, limits, operations, and implemented roadmap state. |
| Ticket/spec/notes/AAR/knowledge/OpenWiki | Preserve evidence, durable lessons, generated documentation, and completion state. |

### Regression plan

| Requirement | Evidence |
|---|---|
| REQ-001 | Mutation-map PostgreSQL test proves atomic event rows for actual transitions and zero rows for rejection/idempotent retry/no-op paths. |
| REQ-002 | Multi-account baseline/pagination test proves owner scope, local ascending cursors, exact unions, bounded pages, no-store, and absent/foreign equivalence. |
| REQ-003 | Retention fixture plus a real append proves oldest-row pruning, feed isolation, explicit reset, and baseline/snapshot recovery semantics. |
| REQ-004 | Real TCP WebSocket test proves header authentication, ready/change payloads, persona filtering, foreign/malformed denial, unexpected-input close, and connection caps. |
| REQ-005 | PostgreSQL listener test proves no pre-commit hint, commit delivery, rollback silence, and lag recovery; real socket test consumes the same listener path. |
| REQ-006 | Exact event assertions across request/accept/remove/block/unblock/send/read prove minimal payloads and every affected persona mapping. |
| REQ-007 | All new ignored SQLx/socket tests run in the PostgreSQL tier and the expanded live recovery flow runs inside a final `bin/gate.sh --diff`. |

### Alternatives rejected

- Reusing the inbox message sequence would cover only one conversation and
  repeat the cross-resource cursor mistake fixed in Ticket 010.
- Putting message bodies or full resources on WebSockets would create a second
  authorization/serialization source of truth and make missed delivery harder
  to repair.
- In-process-only broadcast would miss commits made through another server
  instance. PostgreSQL `NOTIFY` supplies a commit-coupled shared wakeup while
  the retained table remains recovery truth.
- Indefinite event retention would recreate unbounded owner storage; destructive
  pruning without an explicit reset response would silently skip changes.
- Query-string Bearer tokens would improve browser convenience by moving
  credentials into URLs, logs, and histories; a later browser client needs a
  separately reviewed credential transport.

## Phase 3 — Implement

- Built: forward-only sync state/event schema; persona-local sequence,
  retention, and transactional notify domain; owner-scoped baseline/incremental
  REST; bounded header-authenticated WebSocket hub; startup PostgreSQL listener;
  exact connection/block/inbox mutation hooks; real TCP socket and PostgreSQL
  contracts; live cursor smoke; and public API/runtime documentation.
- Focused checks: `cargo check --workspace --all-targets` passed;
  `cargo test --workspace --all-targets` passed 26 local tests with 26 database
  tests intentionally ignored; `./scripts/test-database.sh` passed all 26
  isolated PostgreSQL tests; `cargo clippy --workspace --all-targets -- -D
  warnings` passed; `bash -n scripts/dev.sh` and `git diff --check` passed; and
  `./scripts/dev.sh --smoke-test` completed the real PostgreSQL/server/cursor/QML
  path (the headless renderer printed its known non-fatal EGL warnings).
- Deviations: runtime JSON moved from a test-only dependency to the ordinary
  server dependency for socket serialization. `futures-util` and
  `tokio-tungstenite` remain test-only; the production Axum socket uses its
  inherent send/receive API. The migration intentionally does not foreign-key
  `conversation_id`, so a future conversation lifecycle cannot make retained
  historical invalidations prevent deletion.

## Phase 3.5 — Inspect ledger

| # | Lens | Finding | Severity | Disposition |
|---|---|---|---|---|
| 1 | Concurrency / recovery | Retention validation and event fetch used separate `READ COMMITTED` snapshots, so a concurrent append at the 10,000-row boundary could prune the expected first event after validation and return a silently gapped page. | correctness | Fixed with a post-fetch continuity check that returns `reset_required`; focused unit coverage exercises contiguous, missing, empty-stale, and current-empty pages. |
| 2 | Resource bounds | Axum's default WebSocket decoder accepts messages and frames far larger than this server-to-client-only protocol needs, and application rejection occurs only after decoding. | medium security | Fixed with 1 KiB frame/message decoder ceilings; the real TCP test proves an alternate binary payload above the boundary terminates before application delivery while ready/changed remains intact. |
| 3 | Admission fairness | Per-persona and process socket limits do not prevent one account from distributing connections across many personas and consuming the shared process pool. | low security | Fixed with a 20-socket account counter in the same RAII permit. Unit coverage proves cross-account fairness and release; a real route test proves HTTP 429 and post-close reacquisition. |
| 4 | Session lifecycle | An established socket retains no session authority, so revocation, expiry, or account disablement cannot terminate the transport or stop later hints. | low security | Fixed with UUID-only no-touch reauthorization before ready/hints and every 30 seconds. Real socket coverage proves no idle refresh and close on revocation, expiry, and account disablement. |
| 5 | Documentation | The block endpoint section still described general sync events and WebSockets as future work after Ticket 011 implemented them. | docs | Fixed to link state-changing social mutations to the synchronization contract below. |
| 6 | Test / operations coverage | Existing tests did not exercise route-level 429/release, decoder size rejection, or established-socket revocation, and the live smoke validates REST recovery without opening the WebSocket. | coverage | The first three gaps now have real TCP/PostgreSQL coverage. Live WebSocket smoke remains an inspection follow-up; the canonical integration tier owns the full socket matrix. |

- Codex Security scan `64f82233-d66b-43cb-b583-2cfb26f7c7e3` completed
  against frozen snapshot
  `codex-security-snapshot/v1:sha256:8f0587ada13655249282a3a79c08db10059f38b2965615e6df999bf0fb4bbc56`.
  It reviewed all 63 diff worklist rows and produced one medium and two low
  reportable findings. The message-membership oracle and developer-smoke argv
  candidates were suppressed; the cursor race was retained here as a
  non-security correctness defect.
- Post-repair focused evidence: `cargo fmt --all -- --check` passed;
  `cargo test --workspace incremental_page_detects_pruned_or_missing_events`
  passed the new continuity regression; `cargo check --workspace --all-targets`
  passed; and the real PostgreSQL
  `retention_prunes_per_persona_and_expired_clients_receive_reset` test passed
  in isolation. Final broad validation remains intentionally pending until the
  approval-gated WebSocket findings are dispositioned.
- Approval-ready security remediation design: configure both Axum decoder
  limits to 1 KiB for the server-to-client-only route; add a 20-socket
  per-account limit alongside the existing five-per-persona and 256-per-process
  limits; and retain only the authenticated account/session UUIDs so the socket
  can run a no-touch validity query before persona hints and every 30 seconds.
  The validity query must enforce revocation, absolute expiry, seven-day idle
  expiry, and active account status without advancing `last_used_at`. The
  existing RAII permit will own and release all account/persona/process counts.
  Planned regression evidence covers decoder rejection, route-level 429 and
  release, cross-account fairness, post-revocation close-before-hint, periodic
  expiry, and proof that revalidation does not refresh idle lifetime.
- User authorized continuation. Implemented that design at the shared
  boundaries: `WebSocketUpgrade` owns decoder limits, `SyncHub` accounts for
  account/persona/process admission in one permit, and `sessions` supplies a
  UUID-keyed no-touch validity query reused before ready/hints and by the
  periodic socket timer. No socket retains a raw Bearer token. Post-fix
  `cargo check --workspace --all-targets` passed; the full local suite passed
  27 tests with 28 PostgreSQL tests ignored; and the three focused real
  TCP/PostgreSQL WebSocket tests passed, including oversized binary input,
  route-level 429/release, ordinary ready/changed behavior, no-touch idle
  evidence, and close on revocation, expiry, or account disablement.
- Post-fix skeptical inspection found no sibling WebSocket route or alternate
  `SyncHub::acquire` caller that bypasses the new boundaries. CodeGraph traced
  `open_sync_socket` through owner authentication, account/persona/process
  admission, `PreparedSocket`, and the no-touch session authority loop; it
  identified one production caller for both `prepare_socket` and
  `serve_socket`. Its static test association remains incomplete, so direct
  source inspection and the real TCP/PostgreSQL tests supply the missing
  transport coverage. The worktree-bound inspection receipt was refreshed
  after all inspection edits.

## Phase 4 — Validate

- Tests run: the final pre-completion `bin/gate.sh --diff` passed Rust format,
  Clippy with warnings denied, 27 local tests, Rustdoc, Compose validation,
  shell syntax, pipeline structure, changed-file secret scanning, Codex hook
  self-tests, whitespace validation, all 28 isolated PostgreSQL tests, and the
  real PostgreSQL/Rust API/QML smoke path. The new WebSocket tests prove both
  oversized binary and text rejection, persona/account quota 429 and release,
  cross-account admission fairness, no-touch revalidation, and established
  socket closure after revocation, absolute expiry, or account disablement.
- Gate run: `bin/gate.sh --diff` returned `GATE GREEN [diff]` and wrote the
  worktree-bound validation receipt.
- Skips or pre-existing failures: none. The headless QML renderer emitted its
  known non-fatal EGL warnings while the smoke still passed.

## Phase 5 — Complete

- Acceptance-criteria audit:

  | Requirement | Final evidence | Result |
  |---|---|---|
  | REQ-001 | `mutations_emit_minimal_owner_local_events_and_noop_retries_emit_none` proves transaction-coupled event mapping for every social/inbox transition and proves idempotent retries append nothing. | Satisfied |
  | REQ-002 | The same multi-account test proves empty baselines, bounded ascending pagination, stable cursors, exact minimal DTOs, no-store responses, owner scoping, and absent/foreign equivalence. | Satisfied |
  | REQ-003 | `retention_prunes_per_persona_and_expired_clients_receive_reset` plus the post-fetch continuity unit test prove the 10,000-event persona-local bound, isolation, explicit reset, and concurrent-gap defense. | Satisfied |
  | REQ-004 | `websocket_is_header_authenticated_owner_scoped_and_hint_only` proves header-only authentication, non-disclosing foreign denial, ready/changed exact shapes, persona filtering, and bounded client-frame rejection over real TCP. | Satisfied |
  | REQ-005 | `postgres_hints_and_events_are_visible_only_after_commit` proves rollback silence and commit fan-out; the real socket test consumes the shared hint path and the hub unit test proves lag recovery. | Satisfied |
  | REQ-006 | The mutation-map assertions cover request, acceptance, removal, block/unblock, message, and read invalidations for every affected persona while excluding message bodies, account IDs, peer read state, and block direction. | Satisfied |
  | REQ-007 | The green canonical diff gate ran migration `0009`, all 28 PostgreSQL tests, durable REST recovery and reconnect semantics, the real socket matrix, live cursor smoke, and the unchanged QML health connector. | Satisfied |

- Docs: README, API, architecture, roadmap, and the four affected OpenWiki pages
  describe REST as durable truth and WebSockets as bounded advisory hints. The
  OpenWiki lifecycle completed and was reconciled again after ticket/AAR/archive
  state changed so its final receipt represents the completed worktree.
- AAR: `AAR-011` submitted at effectiveness 5/5 with four failures, three
  prevention rules, one architecture decision, and matching knowledge-register
  entries.
- Archive: TICKET-011 closed and the spec/notes pair moved together to
  `completed/`. Delivery remains separate and unauthorized: no commit, push,
  pull request, or publication was performed.

## Defect and lesson ledger

| # | What happened | Root cause | Fix | Prevention |
|---|---|---|---|---|
| 1 | A retained incremental read could validate one snapshot and fetch a later snapshot after concurrent pruning removed its expected first row. | Continuity was inferred from pre-fetch retention metadata under `READ COMMITTED`. | Verify the first returned cursor after fetch and require reset on any gap. | Validate cursor continuity on the returned page, not only against earlier bounds. |
| 2 | The initial socket boundary relied on Axum's broad default decoder limits. | Application-level rejection was mistaken for allocation-level bounding. | Set both frame and assembled-message limits to 1 KiB and prove oversized text/binary termination. | Bound transport decoders before application parsing for server-only protocols. |
| 3 | Per-persona and process quotas let one account consume the process pool through many personas. | Admission was scoped to the immediate resource but not the authenticated principal. | Add a 20-per-account counter to the same RAII permit and prove cross-account fairness/release. | Long-lived transport quotas must include principal, resource, and process dimensions. |
| 4 | Established sockets could outlive revoked/expired sessions or disabled accounts. | Authority was checked only at upgrade and the raw token was correctly discarded without retaining a revalidation key. | Retain account/session UUIDs and run no-touch checks before ready/hints and periodically, closing fail-closed. | Reauthorize long-lived transports without refreshing idle lifetime or retaining credentials. |

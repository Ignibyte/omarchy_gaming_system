---
title: Game challenges, turn notifications, history, and expiration — notes
pipeline_id: 54febbf7-107e-448b-ae18-11771fdd8ee6
---

# Game challenges, turn notifications, history, and expiration — running notes

Chronological evidence and decisions. If a test or command did not run, these
notes must not claim it passed.

## Phase 1 — Plan

- Ticket 017 is Phase 5 complete. The public cartridge SDK/release/import path
  is local tooling and does not block the first playable; compiled rules and
  PostgreSQL remain current gameplay authority.
- Recalled product and runtime boundaries: challenges belong to persona
  identity; accepted connections own one private conversation; blocks suppress
  new interaction; exact game versions and snapshots are durable; REST/cursor
  recovery is truth and WebSockets carry hints only.
- Recalled `PR-omarchy-gaming-system-lock-social-pairs-before-state-001`:
  challenge transitions that depend on relationship/block state must use the
  established canonical pair lock before reading or mutating that state.
- Recalled `PR-omarchy-gaming-system-pin-durable-game-rules-version-001` and
  `PR-omarchy-gaming-system-check-replay-before-current-revision-001`: the
  invitation pins one exact game version, and accepted command replay keeps the
  current receipt-before-revision behavior.
- Recalled `PR-omarchy-gaming-system-verify-retained-cursor-continuity-001`:
  challenge and turn changes extend the existing persona-local sync feed; they
  do not introduce payload-bearing WebSocket truth.
- Smallest useful slice: a two-person challenge sent through the existing
  private inbox, terminal challenge history with lazy transactional expiry,
  atomic acceptance into the existing session primitive, and explicit proof
  that commands notify once without creating a second notification store.
- Initial CodeGraph exploration found the expected additive blast radius:
  `games::create_session` is already a trusted transaction primitive;
  `sync::append_event` commits persona-local invalidations; app handlers own
  transport mapping; connections/inboxes own pair and conversation policy.
  CodeGraph did not return migrations or complete private helper bodies, so
  those SQL/docs/test surfaces were inspected directly as required.
- No commit, push, pull request, remote provider work, or external publication
  is authorized by this pipeline.

## Phase 2 — Design

- CodeGraph design receipt: pipeline
  `54febbf7-107e-448b-ae18-11771fdd8ee6`, tool
  `mcp__codegraph__codegraph_explore`, baseline state
  `0a70c181f456aa850ccf6b5d54444a1648d30c17c04ec93cf3121ef3e736851c`.
- Direct inspection covered migration constraints, SQL row projections, route
  and response patterns, transaction ownership, runtime manifest/player-count
  validation, test harnesses, and unsupported documentation surfaces.
- Add forward-only migration `0012_game_challenges.sql` with the challenge
  aggregate, challenger-scoped idempotency, one pending exact-game challenge
  per directed pair, participant history indexes, and state/session/timestamp
  shape constraints. Extend inbox messages with typed challenge/session
  references and persona sync events with a challenge reference plus an exact
  payload-shape constraint.
- Add `challenges.rs` as the orchestration domain. It authenticates ownership,
  uses `connections::lock_connected_pair`, applies a seven-day expiry and
  100-incoming/100-outgoing caps, resolves expiry lazily under root locks, and
  owns create/list/detail/accept/decline/cancel semantics.
- Preserve lock order as persona roots in UUID order, challenge row, then
  conversation sequence. The existing game-session primitive may re-lock the
  already locked persona roots in the same order and therefore does not invert
  the graph. Sync state locks follow domain and conversation writes.
- Add one exact registry-manifest lookup to the database-free runtime so create
  can reject absent versions or definitions that do not admit exactly two
  human players before writing. Initialization remains authoritative at
  acceptance and any later failure rolls the whole transaction back.
- Challenge messages are `game_challenge_created`,
  `game_challenge_accepted`, `game_challenge_declined`, and
  `game_challenge_cancelled`. Expiry changes durable status on the next read or
  mutation but does not fabricate a retroactive inbox message. The client
  already knows `expires_at` and re-fetches durable inventory.
- Every first mutation emits `game_challenge_changed` and
  `conversation_changed` for both participants. Acceptance additionally gets
  the existing `game_session_changed` events from `games::create_session`.
  WebSocket frames remain only `{\"type\":\"sync_required\"}` hints.
- HTTP is five participant-scoped operations under
  `/v1/personas/{persona_id}/game-challenges`: create, paginated list, detail,
  accept, decline, and cancel (the create/list pair share one route). All
  responses are `no-store`; foreign and absent challenge detail are
  indistinguishable.
- Response allowlist: challenge ID, exact public game identity, direction,
  status, public challenger/challenged personas, optional accepted session ID,
  and timestamps. Idempotency keys, account IDs, block/connection direction,
  registry internals, and game snapshots never cross the boundary.
- Regression map: runtime lookup unit tests; challenge creation, pagination,
  authorization, expiry, transition, rollback and race PostgreSQL tests; typed
  inbox and sync payload tests; command notification exact-once coverage; API
  no-store/body-limit/error-shape checks; and empty-registry live smoke.
- Phase 2 exit: design is actionable, scoped to the first playable, and backed
  by a fresh worktree-bound CodeGraph receipt. PASS.

## Phase 3 — Implement

- Added forward-only migration `0012_game_challenges.sql`. PostgreSQL now
  enforces distinct participants, canonical exact-game identity, monotonic
  status/session/resolution shape, challenger-scoped UUID idempotency, one
  equivalent pending challenge, participant-history indexes, typed inbox
  challenge/session references, and payload-minimal challenge sync events.
- Added exact registry manifest lookup without exposing compiled definitions or
  making the runtime stateful. Challenge creation admits only an existing
  exact version whose human-player bounds contain two; acceptance still calls
  the authoritative initializer.
- Added the `challenges` domain with owner authentication, canonical pair
  locking, connected/unblocked enforcement, seven-day expiry, 100-directional
  caps, private list/detail, exact creation replay, monotonic directional
  transitions, atomic session creation, and stable public errors.
- Extended the inbox with four server-authored challenge variants. Messages
  reference the durable challenge, and acceptance alone references its exact
  session. The existing conversation sequence/unread implementation remains
  authoritative and every mutation appends conversation invalidations for both
  participants.
- Extended the cursor feed with `game_challenge_changed`. The database shape,
  Rust enum, query parser, and JSON response all require only the challenge ID;
  WebSocket frames remain unchanged and payload-free.
- Added six private PostgreSQL challenge tests plus an extractor/body-bound
  test. Coverage includes exact creation/replay/collision, foreign/absent
  privacy, typed inbox parsing, minimal sync responses, exact-version seats,
  acceptance retry, initializer rollback, post-challenge block enforcement,
  server caps, terminal history/pagination, durable expiry, and an
  accept-versus-decline race.
- Updated the HTTP contract, system architecture, README status, and live smoke.
  Since production intentionally has no compiled game, live smoke challenges a
  connected peer to an unavailable version and proves HTTP 409 plus no cursor
  effect.
- Focused evidence:
  - `cargo test --workspace --no-fail-fast`: 31 server tests and all workspace
    package/integration tests passed; 37 PostgreSQL tests were intentionally
    ignored by that command.
  - `./scripts/test-database.sh`: all then-current 37 PostgreSQL tests passed.
  - focused challenge rerun after adding the remaining cases:
    `cargo test -p omarchy-gaming-system-server challenge_api_tests:: --
    --ignored --test-threads=1`: six passed.
  - `cargo clippy -p omarchy-gaming-system-server --all-targets -- -D warnings`:
    passed.
- Test correction: inbox history is deliberately returned in ascending local
  sequence after selecting the newest bounded page. The first assertion
  expected newest-first; it was corrected to the documented chronological
  order without changing production behavior.
- Phase 3 exit: implementation and focused evidence are complete. PASS.

## Phase 3.5 — Inspect ledger

| # | Lens | Finding | Severity | Disposition |
|---|---|---|---|---|
| 1 | Correctness/idempotency | Creation checked the current game registry and connected-pair policy before consulting its durable idempotency record. An exact retry could therefore stop returning the original representation after a block, disconnect, or registry change. | Medium | Fixed: authenticate/own the challenger, check the challenger-scoped immutable replay identity first, then load through the participant-authorized detail transaction. A PostgreSQL API test proves the same 200 representation and no duplicate challenge/message/event after both policy and registry changes. |
| 2 | Notification contract | Accepted inbox payloads were asserted through the HTTP projection, but declined and cancelled payload types/IDs were only covered indirectly by row counts. | Low | Fixed: the terminal-history API test now reads the private conversation and asserts both typed payloads, their exact challenge IDs, and the absence of a session ID. |
| 3 | Abuse/resource use | A reviewer proposed that create/cancel churn bypasses the pending-only cap while retaining terminal challenge and inbox history. | Low candidate | Rejected as a diff-scoped vulnerability: the unchanged authenticated private-message endpoint already permits the same connected personas to append unbounded durable history with fewer requests. Keep account throttling and retention policy as public-deployment hardening; the 100 challenge limit remains a concurrent pending-inventory bound. |
| 4 | Auth/privacy/concurrency | Early replay lookup could have weakened current block/registry checks or disclosed another participant's row. | High review priority | No issue found: the lookup follows successful authentication and actor ownership, is challenger-scoped, requires exact immutable target/key/version equality, and returns through the same participant-authorized loader already exposed by list/detail. Concurrent first creates still converge through canonical pair locking plus the in-transaction replay check. |
| 5 | Security tooling | Atlassian Rovo/TAC was not authenticated during the scans. | Advisory | Protected Atlassian context could not be checked. Local source, migration, tests, runtime evidence, and every Workbench review item remained available; no security conclusion depends on TAC. |

- Full frozen diff scan `87cfddfd-f0d8-4235-a2e9-45770417caeb`
  reviewed 51/51 items at snapshot
  `codex-security-snapshot/v1:sha256:44d168e6ee6b18b91089da436a8077d4f9f9a5151b1fdc6a3e9c684f7599bb7f`.
  It completed with zero reportable findings and explicitly rejected the
  durable-churn candidate after validation.
- Final frozen recheck `4c0737fe-ca62-4512-b844-402eec5b70f4` reviewed the
  same 51/51 inventory with priority on the replay fix at snapshot
  `codex-security-snapshot/v1:sha256:1a20996a3cb772d5b91da7e57aa6451e23972f91cc6b6e60fc50892c0ad11624`.
  It completed with zero reportable findings.
- Final CodeGraph inspection covered `create_challenge`,
  `load_owned_challenge`, `get_challenge`, `transition_challenge`, their HTTP
  callers, and the challenge API tests. The inspect receipt for pipeline
  `54febbf7-107e-448b-ae18-11771fdd8ee6` matches gated state
  `d21360703d4333419b9b18c3c51dc99d9a7e5928fdbf0ef64b12ce48c3cee299`.
- Post-fix focused evidence:
  - `cargo test -p omarchy-gaming-system-server challenge_api_tests:: --
    --ignored --test-threads=1` with the repository PostgreSQL URL: six
    passed. An immediately prior invocation without `DATABASE_URL` failed at
    SQLx harness setup before executing a test; it was corrected and rerun.
  - `cargo clippy -p omarchy-gaming-system-server --all-targets -- -D
    warnings`: passed.
  - `cargo fmt --all -- --check`: passed.
- Phase 3.5 exit: confirmed findings are fixed, false positives are
  dispositioned, both security scans are sealed, and the fresh CodeGraph
  receipt matches the post-implementation gated worktree. PASS.

## Phase 4 — Validate

- `bin/gate.sh --diff` completed every canonical stage and printed
  `GATE GREEN [diff]` against gated state
  `d21360703d4333419b9b18c3c51dc99d9a7e5928fdbf0ef64b12ce48c3cee299`.
- The gate included rustfmt, workspace clippy/tests/rustdoc, compose and shell
  checks, pipeline structure, changed-file secret scanning, Codex hook tests,
  whitespace checks, the production cartridge contract, trusted renderer,
  SDK release, cartridge architecture proof, all 39 PostgreSQL integration
  tests, and the live PostgreSQL + Rust API + visible QML smoke.
- The database suite reported 39 passed, zero failed. The ordinary server test
  pass reported 31 passed with the 39 PostgreSQL cases intentionally ignored
  there and then executed by the dedicated database stage.
- The live smoke reached `Server ready at http://127.0.0.1:8080`, exercised
  the empty production registry challenge rejection, and completed despite
  harmless headless EGL `dri2` warnings.
- Phase 4 exit: the worktree-bound delivery receipt matches the final gated
  implementation. PASS.

## Phase 5 — Complete

- EARS audit:
  - REQ-001 PASS — `creation_is_private_idempotent_and_atomically_notified`
    proves exact creation, replay, collision, connected/unblocked policy,
    private typed inbox delivery, two participant-local minimal invalidations,
    replay after relationship and registry drift, and no duplicate effects;
    `outgoing_and_incoming_pending_limits_are_server_enforced` proves both
    server-owned 100-item caps and the migration fixes seven-day expiry.
  - REQ-002 PASS — creation and terminal-history tests prove participant-only
    detail, foreign/absent equivalence, response allowlists, bounded opaque
    pagination, direction, exact game identity, expiry, terminal retention, and
    accepted-session linkage.
  - REQ-003 PASS — `acceptance_creates_one_exact_session_and_retry_has_no_effects`
    proves exact version and seats, one linked session, typed acceptance,
    participant notifications, and no retry effects;
    `acceptance_failures_roll_back_and_blocked_pairs_cannot_start_sessions` and
    the transition race prove rollback, current pair policy, and one winner.
  - REQ-004 PASS — `terminal_history_paginates_and_expiry_prevents_acceptance`
    proves authorized decline/cancel, retained terminal history, exact typed
    messages, no session reference, and no created session; directional errors
    and the accept/decline race cover unauthorized and competing transitions.
  - REQ-005 PASS — the controllable-clock terminal-history case lazily resolves
    a due row before detail/acceptance, returns retained `expired`, denies the
    transition, and leaves session and misleading inbox state absent.
  - REQ-006 PASS — the existing command transaction suite proves each first
    commit appends one `game_session_changed` event per participant and that
    semantic replay, conflict, rejection, and rollback append none; the sync
    suite proves cursor recovery, commit visibility, participant ownership, and
    payload-free `sync_required` WebSocket hints.
  - REQ-007 PASS — challenge HTTP tests assert `no-store`, bounded bodies,
    participant/absent privacy equivalence, and absence of private fields;
    typed inbox messages expose public personas and durable references only,
    challenge sync exposes only the challenge ID, and WebSockets carry no
    challenge or game state.
- OpenWiki update run `ee01b6bc-f473-457c-acc3-f04e3cec8171` reconciled
  `quickstart.md`, `runtime-foundation.md`, `product-boundaries.md`, and
  `development-and-validation.md`. The lifecycle completed with a warning that
  the `product-boundaries.md` Claims sidecar retains unresolved evidence debt.
  Its completion receipt records pipeline
  `54febbf7-107e-448b-ae18-11771fdd8ee6`, gated state
  `912b23231b84c6a1ded742e259a160dc50a406c1e36ce3d7b79a7c1829b5a509`, and
  tool `mcp__openwiki__openwiki_finish`.
- AAR-020 records one corrected replay-order failure, one standing prevention
  rule, the durable challenge orchestration decision, the rejected
  diff-scoped abuse candidate, and the public-deployment hardening boundary.
- The final post-OpenWiki `bin/gate.sh --diff` passed all 16 gates and wrote the
  matching worktree receipt
  `912b23231b84c6a1ded742e259a160dc50a406c1e36ce3d7b79a7c1829b5a509`.
  This included 31 ordinary server tests, all 39 sequential PostgreSQL tests,
  the live PostgreSQL → Rust API → visible QML smoke, and the complete
  cartridge contract, renderer, SDK release, and architecture proofs.
- Ticket 020 is closed and this spec/notes pair is archived. No commit, push,
  pull request, or external publication was performed.

## Defect and lesson ledger

| # | What happened | Root cause | Fix | Prevention |
|---|---|---|---|---|
| 1 | An exact create retry could fail after a block, disconnect, or registry change. | Current admission policy ran before durable idempotency replay lookup. | Authenticate and owner-scope, resolve the exact immutable replay first, then load it through participant authorization; apply current admission only to new work. | `PR-omarchy-gaming-system-resolve-durable-replay-before-current-admission-001` and a policy/registry-drift regression. |
| 2 | Decline and cancel inbox payloads had only indirect row-count coverage. | The initial test map inspected the accepted payload but not both non-session terminal payloads. | Assert exact message type, challenge ID, and absent session ID through the private inbox API. | Keep transport-level assertions for every new tagged-union variant. |
| 3 | One initial inbox assertion expected newest-first order. | The test overlooked the existing contract: select the newest bounded page, then return that page in ascending conversation sequence. | Corrected the expectation; production behavior was unchanged. | Read the established pagination contract before asserting new system-message order. |
| 4 | One focused PostgreSQL rerun failed before test execution. | The direct Cargo invocation omitted the repository `DATABASE_URL`. | Reran with the repository PostgreSQL URL; all six focused challenge tests passed. | Treat harness setup failures separately from executed-test failures and record both honestly. |
| 5 | OpenWiki completed with product-boundaries evidence debt. | The lifecycle could not verify every existing Claims-sidecar entry for that page from available indexed evidence. | Kept the warning explicit; no unsupported claim was silently marked verified. | Reconcile the product-boundaries sidecar in a later documentation maintenance run when supporting evidence is available. |

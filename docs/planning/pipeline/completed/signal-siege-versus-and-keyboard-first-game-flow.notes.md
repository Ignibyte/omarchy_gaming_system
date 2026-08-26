---
title: Signal Siege versus and keyboard-first game flow — notes
pipeline_id: 8d6fff91-f81f-4d9f-b0d3-302d96960781
---

# Signal Siege versus and keyboard-first game flow — running notes

Chronological evidence and decisions. If a test or command did not run, these
notes must not claim it passed.

## Phase 1 — Plan

- Recall: `main` began clean and synchronized at delivered Ticket 023 commit
  `26d592c`; no pipeline or critical bulletin is active. PostgreSQL is healthy,
  and CodeGraph 1.5.0 plus the Codex-only OpenWiki 0.3.3 integration passed
  provenance readiness.
- Recall: Tickets 020 and 021 already provide participant-private exact-version
  challenges, atomic challenge acceptance into ordered two-human sessions,
  revision/idempotency-checked commands, terminal history, and Signal Siege v1
  solo play. Durable replay precedes mutable policy/revision checks, REST is
  recovery truth, and cursor/WebSocket events contain resource UUIDs only.
- Recall: Tickets 022 and 023 established one process-memory bearer owner, a
  selected owned persona, a serialized player request gateway, strict response
  bounds, keyboard/accessibility controls, real QML authority evidence, and
  explicit manual REST refresh for social/inbox state.
- Recall: the trusted cartridge renderer already accepts only inert bounded
  render plans in repository-owned QML components and emits unconfirmed
  allowlisted actions. This ticket can reuse that client seam without loading
  publisher executable code or claiming cartridge acquisition exists.
- Recall: `PR-omarchy-gaming-system-resolve-durable-replay-before-current-admission-001`,
  `PR-omarchy-gaming-system-check-replay-before-current-revision-001`,
  `PR-omarchy-gaming-system-validate-game-state-cross-field-invariants-001`,
  `PR-omarchy-gaming-system-mirror-authoritative-client-response-bounds-001`,
  and `PR-omarchy-gaming-system-retire-qml-xhr-after-generation-invalidation-001`
  apply directly to new game rules and QML transport behavior.
- Product gap: production advertises only one-human Signal Siege v1 and the
  optional one-human Door Legends pilot, while the challenge domain requires
  an exact definition admitting two humans. A challenge-only screen would be
  knowingly unusable and cannot prove the charter's first-playable outcome.
- Decision: add immutable Signal Siege v2 with public alternating turns, then
  deliver catalog/challenge/session/gameplay QML as one end-to-end slice. V1
  remains the exact solo definition and existing durable v1 sessions do not
  change interpretation.
- Design correction: `TrustedCartridgeSurface` requires an authenticated
  publisher/game/version/archive origin. A platform-generated plan would
  counterfeit that provenance. The compiled Signal Siege presenter will use
  repository-owned inert node components under an explicit platform label,
  while installed cartridge discovery and remote-provider presentation remain
  later verified integrations.
- Delivery: the overnight continuation authorizes ongoing pipeline work but
  does not separately authorize another commit or push; delivery will remain
  pending unless the user explicitly approves it.

## Phase 2 — Design

### Runtime rules and durable state

- CodeGraph traced the exact dynamic-dispatch and persistence path:
  production registry → `GameRegistry::{catalog,initialize,apply_command}` →
  immutable `GameDefinition`; challenge acceptance →
  `challenges::accept_challenge` → `games::create_session`; and participant
  command → locked exact-version session → rules transition → snapshot,
  revision, receipt, lifecycle, and sync commit. Existing generic challenge,
  command, replay, transaction, and participant-privacy tests remain the
  regression perimeter.
- Keep `SignalSiege` and every v1 constant/state/transition unchanged. Add a
  separate `SignalSiegeVersus` definition under the same `signal_siege` key at
  version 2 with `min_human_players == max_human_players == 2`; register both
  exact definitions in production canonical order.
- V2 state is one exact object: schema/rules versions, `turn` in `0..=24`,
  `max_turns: 24`, phase `awaiting_action|completed`, nullable
  `active_seat`, exactly two ordered combatants (`seat`, core `0..8`, energy
  `0..4`, guard `0|2`), nullable last-turn evidence, and nullable terminal
  outcome. Active seat is `turn % 2`; completed state has no active seat.
- Both combatants start at eight core/two energy. Strike and guard cost one;
  charge gains two up to four. Guard installs a two-point shield through the
  opponent's next action and expires at the start of its owner's following
  turn. Strike consumes the opponent shield and applies the remaining part of
  two damage. A valid action increments one turn and either selects the other
  seat or completes by destroyed core/turn 24. Turn-limit outcome compares
  core, then energy, otherwise draws.
- V2 parsing validates structure, scalar bounds, ordered seats, active-seat/
  turn parity, phase/core/turn/outcome relationships, last-turn actor/action/
  damage/block consistency, and terminal outcome copies before any command.
  Rules receive only state, actor seat, and bounded command; no identity,
  clock, database, network, or randomness is introduced.

### API, authority, and compatibility

- No migration or route/schema change is needed. `GET /v1/games` additively
  returns Signal Siege v2 after v1. The solo-start route still admits only
  exact one-human definitions, so v1 behavior and durable replay are
  unchanged; challenge creation/acceptance now admits v2 through the existing
  exact two-human check and session primitive.
- `OnboardingController` remains the only bearer owner. Its navigation and
  player-request allowlists gain `games`, `challenges`, and `gameplay`; a new
  `GameController` receives only that request gateway and the selected owned
  persona. Every actor path is derived internally, actor/session changes
  cancel the shared request and invalidate generations, and a valid 401 clears
  the complete onboarding/social/game authority state.
- The game controller strictly validates exact catalog, connection,
  challenge-page, public persona, session, participant, provider-result, and
  command envelopes. Supported compiled state receives exact v1/v2
  cross-field validation before publication. Unknown exact games/providers may
  remain in bounded inventory but expose no raw state and no action until a
  verified presentation adapter exists.
- Fresh user intents receive UUID-shaped non-secret idempotency identities.
  An uncertain transport outcome retains the exact method/path/body for one
  explicit retry; successful/domain-terminal outcomes clear it. Revision
  conflict and other-player turns refetch participant-authorized detail rather
  than guessing current revision or state.

### QML data and presentation flow

- Entering Games chains public catalog then up-to-100 participant sessions.
  Entering Challenges chains catalog, accepted connections, then the newest
  challenge page; `next_before` is the only older-history cursor and older
  pages append after strict uniqueness/order checks. Entry and action refresh
  are explicit—there is no timer, polling, or WebSocket client.
- Catalog rows start exact one-human supported v1 or direct the player to
  challenges for exact two-human v2. Session/challenge rows can open only a
  validated participant session UUID. Challenge buttons derive the target from
  the validated accepted-connection record and game identity from the
  validated two-human catalog entry; screens never accept arbitrary actor,
  target, game, revision, or command JSON.
- `GameController` projects validated Signal Siege state into a small
  platform-owned view model. `game/SignalSiegeSurface.qml` labels its origin as
  `PLATFORM COMPILED`, uses repository-owned inert status/meter/button nodes,
  and emits only `strike|guard|charge`. It never constructs
  `omarchygs.render-plan/v1` or a cartridge digest. The authenticated cartridge
  surface therefore remains reserved for genuinely verified archives.
- V1 actions are enabled for actor seat 0 while active. V2 actions are enabled
  only when `active_seat` equals the actor's validated participant seat and
  affordability permits the action. Other turns show a refresh state;
  completed sessions show immutable result/history and no action nodes.
- Games, Challenges, and Gameplay screens keep platform chrome outside game
  presentation, render all strings as `Text.PlainText`, provide explicit
  loading/empty/offline/protocol/error states, and expose Enter/Space actions,
  Escape recovery, focus restoration, accessible names, and scrollable
  640×420 layouts.

### Exact file manifest

| File | Purpose |
|---|---|
| `crates/game-signal-siege/src/lib.rs` | Add immutable exact v2 state, alternating rules, validation, and unit matrix without changing v1. |
| `crates/server/src/main.rs` | Register v1 and v2 in the production compiled registry. |
| `crates/server/src/signal_siege_api_tests.rs` | Prove two-entry catalog, unchanged solo v1, real v2 challenge/session/command/completion/recovery. |
| `client/qml/OnboardingController.qml`, `Main.qml`, `screens/HomeScreen.qml` | Admit and wire games/challenges/gameplay navigation while preserving one authority owner. |
| `client/qml/GameController.qml` | Own strict game/challenge/session REST state, mutation retry, actor binding, validation, and presentation projection. |
| `client/qml/game/SignalSiegeSurface.qml` | Render exact supported compiled state with explicit platform provenance and inert allowlisted actions. |
| `client/qml/screens/GamesScreen.qml`, `ChallengesScreen.qml`, `GameplayScreen.qml` | Provide keyboard catalog/session, challenge lifecycle, and authoritative match flows. |
| `client/qml/tests/fixture_server.py`, `tests/fixture/tst_games.qml` | Add stateful normal/hostile game contracts and minimum-layout interaction evidence. |
| `client/qml/tests/live/tst_live_onboarding.qml`, `scripts/dev.sh` | Run two independent QML authorities through real v2 challenge, acceptance, alternating completion, and restart/refetch. |
| `README.md`, `docs/api.md`, `docs/architecture/system-overview.md`, `docs/product-charter.md`, `docs/planning/ROADMAP.md`, `CONSTITUTION.md` | Describe v2 rules, shipped QML boundary, first-playable evidence, and Stage 16 gate truth. |
| `openwiki/*`, current ticket/spec/notes/AAR/knowledge files | Reconcile durable engineering context and completion evidence in Phase 5. |

### Regression map

| Requirement | Evidence |
|---|---|
| REQ-001 | v1/v2 manifest and initialization unit tests; production catalog local test; v1 solo and v2 challenge PostgreSQL admission. |
| REQ-002 | V2 turn/guard/charge/strike/affordability/wrong-seat/malformed-state/determinism/core/limit outcome unit matrix; challenged-match PostgreSQL command loop and existing replay/conflict suite. |
| REQ-003 | QML catalog/session loading, empty/error/manual refresh, bounded/exact hostile envelope tests. |
| REQ-004 | QML v1 start/detail/history interaction and migrated live start/refetch assertions. |
| REQ-005 | QML peer challenge create/incoming accept/decline/outgoing cancel/history/older page plus real two-account acceptance. |
| REQ-006 | Platform provenance assertion, allowlisted surface actions, affordability/turn gating, conflict refetch, completion, and no-poll checks. |
| REQ-007 | Wrong/extra/missing keys, invalid IDs/timestamps/enums/state/outcome, oversized inventories, transport failure, provider-unavailable presentation, and invalid-session cleanup. |
| REQ-008 | Production-root QML interactions at 640×420 and 920×600, focus/Enter/Escape/accessibility/plain-text assertions. |
| REQ-009 | Protected-config live QML scenario with two account/session controllers, durable v2 create/accept/alternating completion, controller recreation/refetch, API readback, and canonical diff gate. |

### Risks and rejected alternatives

- Reject modifying v1 to admit two humans: exact-version durable sessions make
  delivered behavior immutable; v2 is the compatibility boundary.
- Reject hidden simultaneous choices: shared session state would disclose the
  first choice without a new private per-seat projection contract.
- Reject raw JSON or generic command entry: it exposes protocol authority to
  the screen and cannot provide bounded keyboard/accessibility behavior.
- Reject a forged render-plan origin: the cartridge surface's publisher and
  digest identify authenticated content, not a styling convenience.
- Reject automatic polling during another player's turn: it adds unbounded
  background work and competes for the single XHR; manual REST recovery is
  complete and live hints remain a separately reviewed slice.
- No database rollback plan is necessary because there is no schema change.
  Removing v2 from a later registry would make new work unavailable while
  existing v2 sessions retain honest `game_unavailable` behavior; delivery
  must therefore keep v2 registered once sessions exist.
- CodeGraph design receipt: pipeline
  `8d6fff91-f81f-4d9f-b0d3-302d96960781`, tool
  `mcp__codegraph__codegraph_explore`, gated state
  `a92b4d6c806ed12ac95450c6e8e29b90d85a4c1aa97f9fcf0a493f473b202ac0`.

## Phase 3 — Implement

- Added immutable `SignalSiegeVersus` under `signal_siege` v2 while retaining
  the v1 definition and production manifest. V2 admits exactly two humans,
  stores ordered combatants and active-seat parity, applies one deterministic
  strike/guard/charge turn, validates hostile lifecycle/last-turn/outcome
  state, and completes by destroyed core or turn 24.
- Registered v1 and v2 together in the production compiled registry. The
  public catalog test now fixes their exact canonical order, and a real
  PostgreSQL integration creates a v2 challenge, accepts it into seats 0/1,
  rejects the wrong seat, alternates commands to completion, and proves both
  participant reads converge on the same revision/state.
- Added one bearer-free `GameController` authority boundary plus Games,
  Challenges, and Gameplay screens. The controller derives actor paths,
  targets, exact games, revisions, and allowlisted commands from validated
  state; it retains an uncertain mutation identity for explicit retry and
  refetches on revision conflict. Catalog, challenge, persona, session,
  provider-result, and exact Signal Siege v1/v2 state envelopes fail closed.
- Added a platform-owned `SignalSiegeSurface` assembled from trusted inert
  status/meter/button nodes. It consumes only a derived view model, exposes
  only strike/guard/charge, gates affordability and actor seat, uses plain
  text/accessibility metadata, and never invents a signed cartridge origin,
  digest, or render-plan envelope.
- Extended the deterministic fixture with catalog, session, command, and
  challenge lifecycle state plus a production-root QML test at 640×420.
  Extended protected live configuration and `scripts/dev.sh --smoke-test`
  with two independent QML bearer/session authorities that create, accept,
  alternate, complete, recreate the controller, and refetch the same terminal
  v2 session through the production API. Sync inspection accepts only minimal
  challenge/conversation/session invalidations and rejects state/command data.
- Updated the API, cartridge boundary, system overview, product charter, and
  roadmap. `README.md` and `CONSTITUTION.md` from the design manifest required
  no edit: their current commands/boundaries remain accurate and the live gate
  expansion stays within existing Stage 16 wording.
- Cleanup: removed the generated local Python bytecode cache before review.

### Focused implementation evidence

- `cargo fmt --all --check` — PASS.
- `cargo test -p omarchy-game-signal-siege` — PASS, 10 tests.
- `DATABASE_URL=postgres://... cargo test -p omarchy-gaming-system-server challenge_api_tests::production_signal_siege_versus_alternates_and_completes_for_both_players -- --ignored --exact --test-threads=1` — PASS, real PostgreSQL.
- `./scripts/test-qml-onboarding.sh` — PASS, 29 QML tests with no warnings.
- `./scripts/dev.sh --smoke-test` — PASS after the final QML cross-field
  validator changes; deterministic fixtures plus live registration, social,
  two-authority game, and MFA scenarios all passed. The headless launch
  printed non-fatal host EGL `dri2` warnings after the successful scenarios.

## Phase 3.5 — Inspect

### Inspection ledger

| Lens | Evidence and finding | Resolution | Status |
|---|---|---|---|
| Rules/state correctness | Followed the locked participant row and database-derived seat through `GameRegistry::apply_command` into `SignalSiegeVersus`; exact serde state, active-seat parity, affordability, last-turn evidence, terminal outcome copies, and the 24-turn ceiling fail closed. Inspection caught and fixed the hostile `last_turn.actor_seat` bounds check before array indexing. | The parser rejects seats above one before indexing; the 10-test rules matrix and real PostgreSQL challenged-match test pass. | PASS |
| Authentication/privacy | Game QML receives no bearer and derives actor paths from the selected validated persona; the server independently authenticates persona ownership and joins sessions through participant membership. Unknown providers receive no actionable presenter or raw-state UI. | Hostile-provider fixture coverage proves an inert unsupported surface; a valid `invalid_session` clears onboarding and game authority. | PASS |
| Response/provenance | Direct review found that the first QML validator bounded participants but did not require unique persona IDs or exact v1/v2 cardinality before indexed presentation. It also allowed server-impossible manifest/player ceilings. | Mirrored runtime limits, rejected duplicate/self identities, required one v1 or two v2 participants, and added crafted hostile-envelope regression coverage. The platform presenter still creates no cartridge origin, digest, or render-plan envelope. | PASS |
| Idempotency/concurrency | Mutation retries preserve the same method/path/body after uncertain transport; exact server receipts bind actor, revision, and semantic command. Revision conflicts refetch REST truth, while the PostgreSQL session lock admits one revision winner. | Added deterministic timeout-identity and revision-conflict/refetch QML regressions; existing command replay/race integration remains the authoritative durability proof. | PASS |
| Availability/resource bounds | State/command/runtime body ceilings, 100-row client inventories, exact two-player arrays, finite turn count, and response-size enforcement bound work. | Malformed state and oversized/mismatched envelopes remain fail closed; no polling or new WebSocket loop exists. | PASS |
| Accessibility/layout | The original six-button home row could exceed the 640px required width. | Replaced it with a three-column keyboard-order-preserving grid and added a content-bound assertion at 640×420; normal 920×600 hostile-flow coverage also passes. | PASS |
| Developer/live evidence | Reviewed the protected NUL-delimited QML live configuration, two-controller test, terminal API readback, and minimal sync-event filter. | Credentials remain outside argv/log output; live mutation uses only production controllers and server routes. | PASS |

### Inspection receipts

- Fresh post-implementation CodeGraph exploration traced
  `production_game_registry` through participant locking, receipt/revision
  enforcement, `GameRegistry::apply_command`, and `SignalSiegeVersus`, then
  separately inspected `parse_versus_state`, `last_turn_is_valid`, and terminal
  outcome construction. Tool: `mcp__codegraph__codegraph_explore`; pipeline:
  `8d6fff91-f81f-4d9f-b0d3-302d96960781`.
- Codex Security diff scan
  `d99f4742-297d-46ae-9eb6-0cc39f63b76f` completed with zero reportable
  findings at
  `/tmp/codex-security-scans-gVuCBs/omarchy_bbs/26d592c52c47a9c3c0c7b9ca51c140eda64da4b5_20260826T014317Z_r5k7t36z/report.md`.
  Its immutable workbench inventory omitted newly created untracked QML files,
  which were inspected directly and hardened above; the scan honestly records
  partial coverage and requires one final staged delivery scan.
- `./scripts/test-qml-onboarding.sh` — PASS, 33 tests after inspection fixes,
  including hostile participant/state envelopes, inert provider presentation,
  exact uncertain mutation retention, conflict refetch, invalid-session
  cleanup, and minimum-width containment.

## Phase 4 — Validate

- `bin/gate.sh --diff` — `GATE GREEN [diff]`. The canonical run passed
  rustfmt, warning-denied Clippy, production tests and rustdoc, Compose/shell/
  pipeline/hook/secret/whitespace checks, the cartridge contract/renderer/SDK/
  architecture proofs, provider security conformance, and the independent Door
  Legends authority pilot.
- The migrated database suite passed all 45 PostgreSQL-backed cases, including
  the production Signal Siege v2 challenge, alternating-command, completion,
  and participant-convergence case.
- The keyboard-first QML fixture suite passed all 33 cases. The live smoke
  passed registration/persona, social/inbox, MFA, and two independent game
  authorities that challenged, accepted, alternated, completed, and recovered
  the same terminal v2 session. Host EGL `dri2` diagnostics occurred only after
  successful headless scenarios and were non-fatal.
- The initial Phase 4 gate receipt was written at
  `.git/omarchy-gaming-system-gate-receipt`. Phase 5 wiki and closure edits
  intentionally make that receipt stale; delivery requires one final diff gate
  against the completed worktree.

## Phase 5 — Complete

### EARS acceptance audit

| Requirement | Evidence | Result |
|---|---|---|
| REQ-001 | Production registry and catalog assertions expose immutable one-human v1 plus exact two-human v2; solo and challenge PostgreSQL cases admit only their matching cardinalities. | PASS |
| REQ-002 | Ten deterministic v1/v2 rules tests cover turn parity, affordability, malformed state, guard/strike/charge, core and turn-limit outcomes; the migrated challenged-match case proves one revision per authoritative turn and terminal convergence. | PASS |
| REQ-003 | `tst_games.qml` exercises bounded catalog/session loading, empty/error/manual-refresh paths, exact supported actions, and hostile manifest/session envelopes through the production root. | PASS |
| REQ-004 | The game controller receives no bearer, derives the selected-persona path, generates new intent IDs, starts v1 through REST, opens participant-authorized detail, and preserves/refetches durable history. | PASS |
| REQ-005 | Fixture cases cover bounded history, accepted connections, create/cancel/decline/accept direction, and accepted-session opening; the live scenario proves two real accounts create and accept v2. | PASS |
| REQ-006 | Exact state validators precede indexed presentation; `SignalSiegeSurface` exposes only strike/guard/charge, gates active seat and affordability, refetches conflict/other turns, remains non-polling, and creates no cartridge origin/digest/render plan. | PASS |
| REQ-007 | Hostile schema/size/identity/cardinality/provider/transport fixtures reject partial state, retain safe retry identity where appropriate, and prove a valid `invalid_session` clears onboarding and game authority. | PASS |
| REQ-008 | Production-root fixtures at 640×420 and 920×600 prove content containment, keyboard focus/activation/Escape recovery, accessible names, plain text, and visible loading/error/completed states. | PASS |
| REQ-009 | The protected live QML scenario uses two independent account/session controllers against migrated PostgreSQL, alternates only production commands to completion, recreates a controller, and recovers the exact terminal revision and state. | PASS |

### Durable completion evidence

- OpenWiki update run `b82ce241-ee19-42ce-87d7-6ac3b742ce25` returned
  `status: complete` after reconciling quickstart, runtime foundation, product
  boundaries, development/validation, Game Cartridges, and their grounded
  claims. A final no-new-content OpenWiki synchronization follows this closure
  ledger so the completion receipt binds the finished worktree.
- The AAR records the participant-cardinality, minimum-layout, and security-
  inventory failures plus their prevention rules and the v2/presenter
  architecture decisions. Every new ID is registered in the knowledge index.
- Ticket 024 is closed and this single active spec/notes pair is archived under
  `docs/planning/pipeline/completed/`; no active pipeline remains.
- The user's earlier commit-and-push approval authorizes delivery. Final gate,
  staged security scan, staged diff/secret review, commit, push, and remote-SHA
  readback remain delivery operations rather than missing product acceptance.

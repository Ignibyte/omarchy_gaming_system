---
title: Signal Siege compiled game and solo bot matches — notes
pipeline_id: b5b42330-4027-4e01-bad9-eaf21d858869
---

# Signal Siege compiled game and solo bot matches — running notes

Chronological evidence and decisions. If a test or command did not run, these
notes must not claim it passed.

## Phase 1 — Plan

- Baseline delivery is clean and remotely verified: GitHub `main` matches
  cleanup commit `51100466312311150df06b0cd1304c570d03dc16`; the obsolete
  initial-push bulletin is archived and there is no critical bulletin.
- The roadmap's next playable-value item is the first original asynchronous
  game with a bot. Tickets 018 and 019 remain deliberately post-alpha provider
  work and are not pulled ahead of the first-party compiled-game path.
- Recalled `AD-omarchy-gaming-system-version-pinned-game-session-foundation-001`:
  production must register one honest exact definition and stored sessions must
  remain readable even if the current registry later changes.
- Recalled `AD-omarchy-gaming-system-idempotent-revision-command-boundary-001`
  and its replay-before-revision rule: every accepted round uses the existing
  locked session receipt boundary, and the final receipt must replay after the
  session becomes terminal.
- Recalled `PR-omarchy-gaming-system-resolve-durable-replay-before-current-admission-001`:
  solo-start replay must be resolved after authentication/owner scope but
  before current registry and inventory policy that applies only to new work.
- Recalled Constitution §10 and the product charter: the game definition owns
  deterministic rules without database, network, clock, or ambient randomness;
  the server owns persistence, time, identity, permissions, and notifications;
  REST/cursor state is durable and WebSockets remain hints.
- Direct code inspection found the necessary extension points: the immutable
  `GameRegistry`, crate-private transactional `games::create_session`, locked
  idempotent `games::apply_command`, participant-private list/detail, production
  `GameRegistry::empty()` wiring, migration status constrained to `active`, and
  a live smoke that currently proves an empty catalog.
- Smallest honest slice: dedicated Signal Siege rules crate, production
  registry wiring, generic one-human start command with durable receipt/cap,
  typed active/completed transitions with final replay, public API/sync proof,
  and live production playthrough. Bot personas, multiplayer, QML gameplay,
  achievements, providers, and delivery remain outside this pipeline.
- Phase 1 exit: all seven observable requirements, the product identity, and
  the bot/authority/privacy exclusions are fixed. PASS.

## Phase 2 — Design

- CodeGraph design exploration traced `GameDefinition` → `GameRegistry` →
  `games::apply_command` → the Axum command handler and mapped the registry,
  session, response, and fixture blast radius. A second pass enumerated all
  trait implementations and command-response/replay tests. Design receipt:
  pipeline `b5b42330-4027-4e01-bad9-eaf21d858869`, tool
  `mcp__codegraph__codegraph_explore`, gated state
  `a306e8ec395e846d48218d2565fe24295e2131a46db70c33fcf5e0427fe5bd3a`.
- Direct inspection covered unsupported forward-only SQL, production `main`
  wiring, the live shell smoke, the complete PostgreSQL fixture paths, API
  documentation, the product charter, and generated/runtime architecture.

### Rules and state contract

- Signal Siege v1 (`signal_siege`, version 1, one human) is a command-paced
  simultaneous tactical duel. Both sides start with core 8 and energy 2;
  energy is capped at 4; state begins at round 0 in `awaiting_human` phase.
- A human seat-0 command is exactly
  `{"kind":"play","action":"strike|guard|charge"}`. Strike costs one
  energy and deals two damage; guard costs one and blocks two damage for the
  current round; charge gains two energy to the cap. An unaffordable action is
  rejected without mutation.
- Before applying the submitted action, the bot chooses from only the durable
  pre-command state: charge when empty; defend a low core against an energized
  human; otherwise select a fixed state/round-indexed strike/guard/charge
  policy with unavailable actions falling back to charge. It cannot react to
  the submitted action, use ambient randomness, or keep hidden state.
- Both actions pay/gain energy and apply guarded damage simultaneously. The
  state then records one bounded `last_round` object and increments the round.
  Core destruction completes immediately; otherwise round 12 completes by
  remaining core, then energy, then draw. `outcome` records winner
  (`human|bot|draw`), reason (`core_destroyed|round_limit`), both final cores
  and energies, and `rounds_played`. Active states have no outcome; completed
  states reject further rule calls.

### Runtime, persistence, and transaction design

- Replace the definition's untyped next-JSON return with a bounded
  `GameTransition { state, status }`, where status is the closed runtime enum
  `Active|Completed`. Initialization remains active-only. Update all three
  fixture implementations and preserve exact-version/bounds validation.
- Forward-only migration `0013_signal_siege_and_solo_sessions.sql` broadens
  `game_sessions.status` to `active|completed`, adds `completed_at`, and
  enforces lifecycle/timestamp shape. It adds the applied status to existing
  command receipts (backfilled `active`) and creates `game_session_starts` with
  `(persona_id,idempotency_key)` identity, immutable game key/version, a unique
  session link, participant foreign key, canonical key/version checks, and an
  active-count index.
- A new solo-start transaction authenticates/owner-scopes the actor, locks its
  persona root, resolves an exact durable receipt before current admission,
  rejects collisions, validates that the exact manifest requires one human,
  counts at most 25 active receipt-backed solo sessions, calls the existing
  `create_session` primitive for seat 0, inserts the receipt, and commits. The
  shared participant-authorized loader returns the durable current session;
  replay survives completion and current registry removal.
- Command locking no longer filters terminal rows before receipt lookup. It
  locks any participant-owned session, resolves the immutable receipt first,
  rejects a new command against `completed`, executes only active state, then
  atomically updates state/revision/status/completion time, inserts the receipt
  with applied status, and appends one invalidation. Exact final replay returns
  its stored state/status without rules, revision, or notification effects.
- No bot row is inserted into accounts, personas, participants, inbox, or sync.
  The only participant is the owned public persona in seat 0; the bot is fully
  represented by deterministic rules and bounded public match state.

### API and compatibility contract

- Add `POST /v1/personas/{persona_id}/game-sessions` with an 8 KiB body limit:
  `idempotency_key`, `game_key`, and `game_version`, with unknown fields denied.
  Return the existing full participant-private session representation with 201
  for creation and 200 for exact replay, always `Cache-Control: no-store`.
- Add `status` to the command response so the terminal command is immediately
  self-describing. This is an additive v1 field. Existing session list/detail
  retain completed rows and state without consulting the current registry.
- Invalid start identity is 422 `invalid_game_start`; a non-solo definition is
  422 `invalid_game_participants`; unavailable exact version is 409
  `game_unavailable`; a receipt collision remains 409
  `game_idempotency_conflict`; the active cap is 429
  `too_many_active_game_sessions`; new commands after completion are 409
  `game_completed`.
- The public catalog changes intentionally from empty to exactly Signal Siege
  v1. The live smoke must prove that production identity and play through a
  completed durable session before launching the unchanged QML health screen.

### Exact file manifest

| File | Purpose |
|---|---|
| `Cargo.toml`, `Cargo.lock` | Add the compiled Signal Siege workspace member and dependencies. |
| `crates/game-runtime/src/lib.rs` | Add typed command transition/session lifecycle and preserve validation. |
| `crates/game-signal-siege/Cargo.toml`, `src/lib.rs` | Implement the original deterministic v1 rules and exhaustive unit matrix. |
| `crates/server/Cargo.toml`, `src/main.rs` | Compile and install the exact production registry. |
| `migrations/0013_signal_siege_and_solo_sessions.sql` | Add completion lifecycle, receipt status, and solo-start receipts. |
| `crates/server/src/games.rs` | Add start transaction/cap/replay and terminal command persistence/replay. |
| `crates/server/src/app.rs` | Add start request/response route, command status, and stable errors. |
| `crates/server/src/game_api_tests.rs`, `challenge_api_tests.rs` | Adapt fixture transitions and preserve prior foundation/challenge coverage. |
| `crates/server/src/signal_siege_api_tests.rs`, `main.rs` test module wiring | Prove production catalog/start/play/completion/privacy/concurrency against PostgreSQL. |
| `scripts/dev.sh` | Replace empty-catalog proof with a real production launch/play/reconnect smoke. |
| `README.md`, `docs/api.md`, `docs/architecture/system-overview.md`, `docs/product-charter.md`, `docs/planning/ROADMAP.md` | Document the playable rules, start/completion contract, authority, and roadmap outcome. |
| `openwiki/*`, current AAR/knowledge/ticket/pipeline files | Phase 5 durable documentation, evidence, lessons, and archive. |

### Regression map

| Requirement | Evidence |
|---|---|
| REQ-001 | Signal Siege manifest/init/rules parsing/determinism/bounds/malformed-state/wrong-seat/action matrix unit tests; production catalog route test. |
| REQ-002 | PostgreSQL new-start ownership/seat/status/receipt/sync atomicity, active cap, concurrent cap, invalid input, and forced-failure rollback tests. |
| REQ-003 | Exact start replay, collision, concurrent same-key convergence, replay after registry removal and completion, duplicate session/event counts. |
| REQ-004 | Rules energy/simultaneous/bot-policy tests plus command commit, rejection, stale revision, semantic replay/collision, and sync exact-once database tests. |
| REQ-005 | Core and round-limit completion, database lifecycle/timestamp/receipt status, final replay, post-completion conflict, retained list/detail, registry-independent history. |
| REQ-006 | HTTP no-store/allowlist/foreign-absent equivalence, one participant, no bot/account rows, minimal sync and hint-only WebSocket assertions. |
| REQ-007 | Canonical live smoke checks catalog, idempotent launch, command loop to bounded terminal outcome, refetch/history, cursor invalidations, and visible QML health. |

### Risks and rejected alternatives

- Reject a seeded random bot: a seed would be valid but adds unnecessary
  entropy ownership and replay design. V1's state-only policy is transparent,
  deterministic, and test-exhaustible.
- Reject a queued/background bot command: it creates a second command actor,
  worker delivery/retry semantics, intermediate wait state, and reconnect race.
  One atomic human-plus-bot round is the honest asynchronous minimum.
- Reject a bot persona/account: it pollutes public identity and authorization
  with a principal that never authenticates. Bot state stays inside rules.
- Reject client-inferred completion: terminal lifecycle must gate mutation and
  make final replay/history authoritative even if client state parsing changes.
- Reject a generic arbitrary-participant public session-creation route: the
  new command is owner-only and one-human-only; human multiplayer still enters
  through connected challenge acceptance.
- Resource risk is bounded at 25 concurrent active solo sessions under the
  actor persona lock. Completed history remains durable like messages and
  challenges; account-level rate limits/retention remain deployment hardening.
- Phase 2 exit: rules, authority, schema, transactions, compatibility, files,
  risks, and every EARS proof are actionable with a matching CodeGraph design
  receipt. PASS.

## Phase 3 — Implement

- Added the `omarchy-game-signal-siege` workspace crate and registered exactly
  `signal_siege` v1 in the production server. The definition implements the
  locked three-action rules, simultaneous resolution, deterministic pre-command
  bot policy, bounded round/core/energy state, and typed active/completed
  transitions without infrastructure access.
- Migration 0013 adds the durable completed lifecycle/timestamp, applied status
  to command receipts, and owner/idempotency/session-linked solo-start receipts
  with forward-only constraints and indexes.
- Added the authenticated owner-scoped solo-start route, persona-root
  serialization, exact replay-before-registry/cap admission, 25-active cap,
  one-human manifest admission, shared participant-private session loading,
  completed inventory/history, final-command replay, and new-command terminal
  rejection. State, revision, status/timestamp, receipt, and minimal sync event
  commit in one transaction.
- Added five game-rule unit tests and four isolated PostgreSQL scenarios that
  cover the exact catalog, body bound, same-key and final-capacity races,
  ownership/collision/registry drift, invalid/multiplayer/cap admission,
  completion/history/final replay/no bot identity, and forced receipt failure
  rollback. Existing game/challenge fixtures were adapted to the typed
  transition without weakening their assertions.
- Replaced the empty-catalog smoke with a production Signal Siege launch, exact
  retry, bounded playthrough, terminal outcome, final replay, history/refetch,
  minimal cursor-event, and post-completion rejection proof before the visible
  QML connector. Updated README, API, architecture, charter, and roadmap truth.
- Focused evidence: Signal Siege rules `5 passed`; Signal Siege PostgreSQL API
  tests `4 passed`; existing game PostgreSQL API tests `5 passed`.
- Integrated evidence after the final-cap race: `cargo test --workspace
  --quiet` passed all 82 ordinary tests; `./scripts/test-database.sh` passed all
  43 isolated PostgreSQL tests; `./scripts/dev.sh --smoke-test` completed the
  live Rust/PostgreSQL/Signal-Siege/QML path; `bash -n scripts/dev.sh` passed.
- Phase 3 exit: every planned implementation surface exists and the focused,
  full workspace, full database, and live production loops are green. PASS.

## Phase 3.5 — Inspect ledger

| # | Lens | Finding | Severity | Disposition |
|---|---|---|---|---|
| 1 | Security: local smoke trust boundary | The pre-remediation diff scan dynamically confirmed that a spoofed local health responder could return JSON strings that reached Bash arithmetic in `scripts/dev.sh`; this permitted command evaluation in a developer-only smoke process. Codex Security finding `csf_35a147294f73d82fa0617775`, scan `5fca5758-1df3-45d0-8d04-aaa5b07558b1`, immutable digest `codex-security-snapshot/v1:sha256:87f94d07e55dc0b7c52a02366259f958019d99abccce6d859054044d24ee75ff`. | Low | Fixed. The smoke now accepts health only while its spawned server is alive and after that server emits its own listening log line; JSON energy/revision values must be bounded integers before arithmetic. `bash -n`, a malicious-string non-execution probe, a prebound-spoof-server rejection probe, and the live smoke all passed after remediation. |
| 2 | Game-state correctness | Structurally valid but semantically terminal or inconsistent stored JSON could reach the compiled transition function: active round 12, zero-core active state, missing/mismatched last-round evidence, or a false completed outcome. The server normally creates canonical snapshots, but strict rejection is cheaper and safer at the deterministic rules boundary. | Low | Fixed. Signal Siege now validates lifecycle/core/round/outcome consistency, exact initial combatants, and last-round action/damage semantics before applying a command. Four adversarial fixtures were added; all five rules tests pass. |
| 3 | Authentication, privacy, replay, transactions, and concurrency | No additional confirmed defect. Owner scope precedes solo-start receipt lookup; foreign and malformed identities retain absent-resource behavior; no bot identity exists; receipts precede mutable admission/revision checks; state/status/timestamp/receipt/invalidation share one transaction; persona locking serializes same-key and final-capacity starts. | None | Accepted with the HTTP/database response allowlists, exact replay/collision/rollback tests, different-key final-slot race, completion recovery test, and minimal sync assertions. |
| 4 | Complexity and blast radius | No unnecessary second bot actor, worker, random source, or duplicate session engine was introduced. The new route reuses the compiled registry, participant-private loader, session primitive, and command transaction. | None | Accepted. Fresh CodeGraph inspection traced handler → transaction → runtime/rules boundaries and reconciled all typed-transition callers and fixture implementations. Inspect receipt: pipeline `b5b42330-4027-4e01-bad9-eaf21d858869`, tool `mcp__codegraph__codegraph_explore`, gated state `3c80f672b5ff92b0d71b3519b946fbabb4e2deaea702aad17c1967ad3895756b`. |

- The complete pre-remediation Codex Security scan reviewed 13 changed source
  surfaces. Runtime, server, API, and SQL surfaces had no validated security
  findings; the one developer-smoke finding above was resolved and retested.
- Phase 3.5 exit: both confirmed low-severity findings are resolved, all other
  review lenses have concrete coverage, and the post-fix worktree has a fresh
  CodeGraph inspection receipt. PASS.

## Phase 4 — Validate

- `bin/gate.sh --diff` passed rustfmt, warning-denied Clippy, all ordinary
  workspace tests, warning-denied Rustdoc, Compose/shell/pipeline/secret/hook/
  whitespace checks, production Cartridge conformance/renderer/SDK gates, and
  the isolated provider architecture proof.
- The same canonical run passed all 43 SQLx-managed PostgreSQL tests, including
  all four Signal Siege owner-scope, replay, cap-race, completion, privacy, and
  rollback scenarios. It then passed the live PostgreSQL → Rust API →
  Signal Siege launch/play/completion/recovery → visible QML health smoke.
- The gate printed `GATE GREEN [diff]` and wrote matching pre-OpenWiki gated
  state `3c80f672b5ff92b0d71b3519b946fbabb4e2deaea702aad17c1967ad3895756b`.
  Phase 5 generated-wiki edits intentionally make that receipt stale; the
  canonical gate will run once more after completion authoring.
- Phase 4 exit: every regression-table requirement has focused or integrated
  evidence and the canonical delivery loop is green. PASS.

## Phase 5 — Complete

- EARS audit:
  - REQ-001 PASS — the five Signal Siege rule tests prove the exact one-human
    manifest, deterministic initialization/bot/transition behavior, action and
    state rejection, simultaneous effects, bounded resources, core/round
    outcomes, and twelve-round termination; the local production-catalog test
    proves exact registration.
  - REQ-002 PASS — `solo_start_is_owner_scoped_atomic_idempotent_and_registry_independent`
    proves owned seat zero, exact response/privacy shape, receipt/event
    atomicity, same-key concurrency, and foreign denial;
    `solo_start_rejects_invalid_multiplayer_and_over_cap_without_partial_state`
    proves input/manifest admission, the 25-active bound, and a different-key
    final-slot race; the forced-trigger test proves complete rollback.
  - REQ-003 PASS — the start tests prove exact replay, collision, no duplicate
    session/event, replay at the cap, and replay after registry removal and
    completion through the participant-authorized resource loader.
  - REQ-004 PASS — the rule matrix and existing command PostgreSQL suite prove
    one exact human/bot round, unaffordable/malformed/wrong-seat rejection,
    semantic receipt replay, stale/collision denial, rollback silence, one
    revision winner, and exactly one minimal participant invalidation.
  - REQ-005 PASS — `signal_siege_completes_replays_and_recovers_without_bot_identity`
    proves bounded explicit completion, stored lifecycle/timestamp/receipt
    status, exact final replay against an empty registry, new-command conflict,
    and retained list/detail/start-replay history.
  - REQ-006 PASS — the same scenario and start tests prove exact public
    allowlists, one human participant, no extra account/persona row, no account
    or receipt identity in responses, and payload-minimal sync; the unchanged
    synchronization suite proves WebSockets remain authenticated hints only.
  - REQ-007 PASS — the canonical live smoke advertises exact Signal Siege v1,
    creates and exactly replays a solo session, plays bounded integer revisions
    to completion, replays the final command, refetches detail/inventory,
    verifies minimal cursor events, rejects a new terminal command, and then
    completes the visible QML health connector.
- OpenWiki update run `92506c11-5388-44d4-89e9-39ad0912b6d7` reconciled
  `quickstart.md`, `runtime-foundation.md`, `product-boundaries.md`, and
  `development-and-validation.md`, including all shifted product-boundary
  evidence. `openwiki_finish` returned `status: complete` without warnings.
  The completion receipt names pipeline
  `b5b42330-4027-4e01-bad9-eaf21d858869`, tool
  `mcp__openwiki__openwiki_finish`, and matching gated state
  `8373a3aae1bc409863e8f84b05e747a7db20dd8a72684208d57f03cbe5587643`.
- AAR-021 is submitted with three captured failures, three standing prevention
  rules, and the Signal Siege solo lifecycle architecture decision; every new
  ID is registered in `docs/planning/knowledge/INDEX.md`.
- The final post-OpenWiki `bin/gate.sh --diff` passed all 16 stages and printed
  `GATE GREEN [diff]`. It included all ordinary workspace tests, all 43
  sequential PostgreSQL tests, the live PostgreSQL → Rust API → Signal Siege
  launch/play/completion/recovery → visible QML smoke, and every Cartridge
  contract/renderer/SDK/provider proof. The delivery, OpenWiki completion, and
  current gated hashes all match
  `8373a3aae1bc409863e8f84b05e747a7db20dd8a72684208d57f03cbe5587643`.
- Ticket 021 is closed and this spec/notes pair is archived. No requirement was
  deferred or silently dropped.

## Defect and lesson ledger

| # | What happened | Root cause | Fix | Prevention |
|---|---|---|---|---|
| 1 | The first core-destruction unit fixture did not terminate because the documented low-core bot policy chose guard. | The fixture expected only the damage rule and accidentally selected a different deterministic policy branch. | Set bot energy to zero in that fixture so the pre-command policy must charge; production rules were unchanged. | Construct game-state fixtures against the complete policy decision table, not only the transition being asserted. |
| 2 | The first formatting check reported the newly added concurrency assertions. | The manual test patch had not yet been normalized by rustfmt. | Ran `cargo fmt --all`, then reran the full database suite. | Run format immediately after structural Rust test edits before recording validation evidence. |
| 3 | A hostile local responder could feed strings into arithmetic in the live smoke helper. | The smoke trusted `/health` by port and used permissive `jq` extraction before Bash arithmetic. | Bound readiness to the spawned server/log and require bounded integer JSON before arithmetic. | Treat local smoke responses as untrusted input and validate type/range before shell evaluation. |
| 4 | The first rules parser accepted some internally inconsistent lifecycle snapshots. | Deserialization and scalar bounds did not prove the relationships among phase, round, core, last-round evidence, and outcome. | Added semantic lifecycle and last-round validation plus adversarial fixtures. | Validate cross-field game-state invariants at the compiled rules boundary, even when persistence normally emits canonical state. |
| 5 | The first completed OpenWiki run retained the prior pipeline's completion receipt. | The durable spec still recorded Phase 3.5, so the phase-gated hook correctly did not issue a completion receipt for Ticket 021. | Recorded the already-green Phase 4 evidence, reconciled all remaining Claims debt, reran OpenWiki to a warning-free completion, and read back the matching receipt. | `PR-omarchy-gaming-system-advance-durable-phase-before-phase-tools-001`. |

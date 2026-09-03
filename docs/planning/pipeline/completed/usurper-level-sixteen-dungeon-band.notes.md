---
title: Usurper Level Sixteen Dungeon Band — notes
pipeline_id: 805147ae-42f4-4f23-a1ee-fa9c8bd39498
---

# Usurper Level Sixteen Dungeon Band — running notes

Chronological evidence and decisions. If a test or command did not run, these
notes must not claim it passed.

## Phase 1 — Plan

- Recall:
  - Ticket 069 supplies exact rules/state/cartridge v20 through Level 15 plus
    matching OpenWiki/gate evidence, fresh workspace-8 play, and the exact
    17-to-18 instantiated-delegate replacement test;
  - `PR-omarchy-gaming-system-prove-legacy-branch-scope-before-porting-behavior-001`
    requires editor rows, ordinary selection, event separation, HP, retreat,
    and registration branches to be read together;
  - `PR-omarchy-gaming-system-preserve-discarded-legacy-rng-work-001` requires
    every rejected record-150 draw to remain visible in deterministic traces;
  - `PR-omarchy-gaming-system-render-one-phase-valid-command-per-visible-choice-001`,
    `PR-omarchy-gaming-system-exercise-real-qml-input-lifecycle-001`, and
    `PR-omarchy-gaming-system-count-instantiated-delegates-across-plan-replacement-001`
    keep the sixteenth control unique and inside the actual Qt input boundary.
- Source preflight:
  - authenticated source Git and archive copies of `EDMONST.PAS` remain
    byte-identical at SHA-256
    `68c461dac11a32893de7890b52c5df0ea3c35b8ec8b875cd3ab5e5a1b3ab7577`;
  - lines 4121–4220 define Level 16 records 150–159 as Mutant Bulldog, Ogre,
    Huge Ant, Ranger, Master Strangler, Grandpa Dragon, Stabbing Master, Red
    Spider, Draconian, and Elite Guard, all at base strength 20 with exact
    equipment flags;
  - authenticated Git/archive `DUNGEONC.PAS` copies match at SHA-256
    `c2db45a4fc04f9d198abf34a0e737602952724e7d5fb08cd5aacccd05438d061`;
    lines 924–955 keep events separate, spend a fight, and repeat
    `Random(level*10)` until the result exceeds `(level-1)*10`; Level 16
    therefore normally selects records 151–159 and preserves record 150 only
    as source data;
  - the unregistered guard applies only when dungeon level is greater than 89,
    so Level 16 remains on the ordinary branch;
  - authenticated Git/archive `PLVSMON.PAS` copies match at SHA-256
    `0084ff67f29f4442190459ead7abec5b3ca52f03a505c57c8a696ea063ec29ed`;
    lines 68–98 use `Random(level*10)+3` for failed-retreat damage and lines
    603–625 initialize monster HP to strength times three.
- Existing boundary fit:
  - the provider/rules reducer is generic through the implemented maximum;
  - the dungeon screen occupies the first fifteen bounded option fields, so
    Level 16 needs one new external projection field across `GameView`, schema,
    fixtures, and signed binding without changing the platform renderer;
  - the real-input suite and recursive delegate-count assertion remain
    game-neutral and will be ratcheted to the new nineteen-button surface.
- The information-only rebuild bulletin was acknowledged and
  `docs/planning/REBUILD_HANDOFF.md` read. Pipeline tools report CodeGraph
  1.5.0 and OpenWiki 0.3.3 ready; no Docker service is active.
- Baseline `env -u CARGO_TARGET_DIR TMPDIR=/tmp CARGO_NET_OFFLINE=true
  OMARCHYGS_PLATFORM_ROOT=/srv/stacks/omarchy_gaming_system
  ./scripts/test.sh` passed: formatting, strict Clippy, 106 Rust tests,
  rustdoc, authenticated upstream/provenance checks, six real-input QML cases,
  seventeen unique signed screens, and provider-backed local play.
- Decision: implement Level 16 as the next normal dungeon band and defer Level
  17, dungeon events, shared realm, and unrelated combat breadth.

## Phase 2 — Design

- Architecture and ownership:
  - `/srv/stacks/omarchygs_usurper` remains the only owner of Usurper rules,
    durable game state, random draws, monster data, and provider projections;
  - the provider validates a revision-bound signed action, maps the fixed Level
    16 action to the existing typed `EnterDungeon` command, and asks the pure
    reducer for the next state and view;
  - the signed inert cartridge binds `option_p` to one declared button node;
    the platform authenticates the package, validates the view schema and
    declared action set, lowers each accepted node once, and trusted QML
    dispatches the unconfirmed action without gaining gameplay authority;
  - state flow remains `signed button -> local revision check -> provider
    action mapping -> pure reducer -> v21 state/view -> authenticated render
    plan -> one trusted QML delegate`.
- CodeGraph design evidence:
  - `compile_render_plan` validates the authenticated schema/view and declared
    action set before lowering signed nodes; `RenderedNode::Button` carries one
    ID, label, action, and accessible label, and the Core profile admits up to
    256 nodes, so the nineteen-button dungeon view requires no production
    platform extension;
  - its one-hop blast radius identifies the client cartridge runtime, preview
    CLI, renderer integration tests, and the cartridge contract as consumers;
    none depends on Usurper-specific option names or dungeon levels;
  - the separate Usurper repository has no CodeGraph index and QML is not
    represented in the Rust call graph, so its Rust/JSON/scripts and the real
    delegate tree were inspected directly;
  - CodeGraph issued the worktree-bound design receipt for pipeline
    `805147ae-42f4-4f23-a1ee-fa9c8bd39498` at gated state
    `7fbc73560498d635a3939b3088e3bb3682ca3949d216d4f9c421adf607ae7f92`.
- Exact implementation manifest, with one purpose per surface:
  - external `crates/usurper-model/src/lib.rs`: add bounded serialized
    `option_p` to `GameView`;
  - external `crates/usurper-data/src/lib.rs`: add exact records 150–159,
    lookup routing, and source-order/strength/equipment tests;
  - external `crates/usurper-rules/src/lib.rs`: advance strict identity to
    v21, extend validation/switching/labels through Level 16, and add
    encounter, retreat, deterministic, and hostile-state evidence;
  - external `crates/usurper-provider/src/lib.rs`: map the fixed Level 16
    action and prove generic/fixed equivalence, projection, encounter, replay,
    and live profile behavior;
  - external `cartridge/manifest.json`, `cartridge/presentation.json`, and
    `cartridge/schemas/view.schema.json`: advance exact identities, declare one
    Level 16 action/button, and require bounded `option_p`;
  - external `fixtures/presentation/armor-shop.json`, `bank.json`, `chest.json`,
    `combat.json`, `create-class.json`, `create-race.json`, `dungeon.json`,
    `entry.json`, `healer.json`, `inventory.json`, `level-master.json`,
    `magic-shop.json`, `mail-news.json`, `main-street.json`, `sleep.json`,
    `status.json`, and `weapon-shop.json`: supply the exact new required field,
    with a non-empty Level 16 label only on the dungeon view and source-valid
    Level 16 facts on the combat fixture;
  - external `provenance/source-trace.json`: register the reviewed Level 16
    source records and the established selection/HP/retreat branches;
  - external `scripts/test-cartridge.sh`, `scripts/test-provider.sh`,
    `scripts/test.sh`, and `scripts/play.sh`: assert exact v21/Level 16
    identities and uniqueness, run the live profile twice across restart, and
    finish smoke play in a Level 16 encounter;
  - external `README.md`, `docs/COMPATIBILITY.md`, and
    `docs/RUST_PORT_MAP.md`: document the newly implemented band and retain all
    other visible limits;
  - platform
    `client/qml/tests/cartridge/tst_trusted_cartridge_controls.qml`: ratchet
    the game-neutral large-screen regression from 18-to-19 actual delegates;
  - platform `docs/architecture/game-cartridges.md`: reconcile the durable
    external-development boundary through rules v21/Level 16 during Phase 5;
    no platform production renderer, server, migration, Cargo, or client
    source change is required.
- Database and migration consequences: none. Provider-owned state remains
  serialized inside the external adapter, and this slice adds no platform
  persistence, table, column, migration, or PostgreSQL write path.
- API and compatibility contract:
  - state JSON remains strict and deny-unknown-fields, but exact
    `schema_version: 21` replaces v20; v20 and malformed v21 state fail before
    RNG construction or mutation;
  - the view schema adds required string `option_p` with the existing
    64-character bound; all screens provide it, and only the dungeon screen
    binds it to `enter_dungeon_level_16`;
  - the signed manifest advances `rules_version` and `cartridge_version`
    together to 21; SDK and presentation protocol ranges remain exactly 1;
  - `enter_dungeon_level_16` accepts an empty payload only and maps to the
    existing typed command. Levels 0, 17, and `u16::MAX` remain rejected
    without revision or RNG advance.
- Regression/evidence map:

  | Requirement | Planned evidence |
  |---|---|
  | REQ-001 | exact `LEVEL_SIXTEEN_MONSTERS` arrays, source-trace validation, authenticated-source hash/readback, compatibility and port-map review |
  | REQ-002 | v21 identity checks; old/missing/unknown JSON fields; Level 17, wrong-level, boundary-record, unknown-record, wrong-name, and oversized-scalar immutability tests |
  | REQ-003 | sequential levels 1–16 switch test with unchanged RNG/empty traces, visible labels, ascent/descent/remain behavior, and rejected 0/17/max inputs |
  | REQ-004 | forced rejected `Random(160)` draw followed by 151–159, exact 20 strength/10 defence/60 HP, fight decrement, and deterministic twin equality |
  | REQ-005 | exact failed-retreat `(2, 1), (160, 10)` trace and damage; existing attack, potion, spell, class-special, poison, death, reward, and full-day suites |
  | REQ-006 | fixed/generic provider equivalence, `option_p`, live Level 16 profile twice across restart, signed-screen/action uniqueness, 18-to-19 Qt delegate replacement, local-play confirmation, and workspace-8 visual/readback audit |
- Risks and controls:
  - security/input: strict schemas, identifier checks, empty payloads,
    authenticated cartridge content, loopback capability, and revision/screen
    binding continue to reject undeclared or stale actions;
  - privacy/secrets: no account/persona identity or reusable credential enters
    game state; generated local capabilities/private keys remain temporary and
    must not be logged or committed;
  - state/concurrency: the provider serializes revision-aware actions and the
    reducer validates before constructing RNG, so rejected or stale commands
    cannot partially advance state;
  - reconnect/restart: the deterministic provider corpus runs twice around a
    fresh process and compares exact output; provider-owned sessions never
    fail back to platform rules;
  - rendering: the nineteen-button dungeon plan becomes the largest trusted
    action surface. Recursive instantiated-delegate counts across a realistic
    18-to-19 replacement, signed-plan uniqueness, and real pointer/Return
    input cover duplicate or inert controls;
  - rollback: v21 artifacts can be removed before delivery without data
    migration. Published rollback remains out of scope, and no delivery action
    is authorized.
- Decisions and rejected alternatives:
  - preserve record 150 as canonical source data but not a normal encounter;
    selecting it directly would contradict the reviewed rejection loop;
  - add bounded `option_p`; overloading primary/secondary labels would blur
    phase semantics, while a grid or platform renderer change would expand
    architecture without benefit;
  - reuse the generic dungeon/combat reducer; duplicating Level 16 logic in
    provider or QML would introduce a second rules authority;
  - keep events and registration behavior excluded; neither is needed to
    prove the ordinary Level 16 band.

## Phase 3 — Implement

- Implemented the locked Level 16 manifest without changing platform
  production code, migrations, provider protocol, or database state:
  - added exact editor records 150–159 and routed lookup through record 159;
  - advanced strict rules/state identity to v21 and bounded dungeon state,
    commands, labels, encounters, and retreat behavior through Level 16;
  - added the fixed empty-payload provider action, `option_p` projection, and
    signed cartridge button/action/schema at cartridge v21;
  - updated all seventeen view fixtures, the provider conformance/live
    profile, local-play smoke path, source trace, and compatibility docs;
  - ratcheted the platform's game-neutral large-screen replacement test from
    18 to 19 instantiated controls.
- Self-review confirmed that record 150 remains present in data and rejected
  by normal encounter selection, while `Random(160)` admits only 151–159;
  Level 17 and malformed v21 states reject before RNG mutation; `option_p` is
  non-empty only on the dungeon view; and the cartridge declares exactly one
  Level 16 node/action with an empty payload.
- `cargo fmt --all`, JSON parsing, shell syntax validation, and the complete
  `option_p` fixture contract passed.
- Focused Rust verification passed with 111 tests: 18 data, 30 provider, three
  local-session, 59 rules, and one deterministic integration test. This
  includes exact source order/flags, rejected-draw trace, 60 HP, Level 16
  retreat damage, strict v21 hostile state, fixed/generic action equivalence,
  replay, and authenticated live-profile state.
- Focused strict Clippy passed with all targets/features and `-D warnings`.
- Signed cartridge/QML verification passed: six real-input/delegate lifecycle
  cases, all seventeen unique screens, exact v21 schema/action bindings, one
  populated `option_p`, and unique rendered IDs, labels, and actions.

## Phase 3.5 — Inspect

- Frozen security-diff scope: the complete cumulative external Usurper
  working-tree source patch against
  `bb31caa122de669d72a265860b19969fcd28505f`, snapshot
  `codex-security-snapshot/v1:sha256:3c483592586dda24bdfea074774ece83b75d13e08edec1ab466e28ee2056bd1b`.
  Seventeen source-like files received full-file review receipts and their
  hashes reconciled exactly against the frozen snapshot.
- Codex Security finalized a complete sealed scan with zero reportable
  findings at
  `/tmp/codex-security-scans/omarchygs_usurper/bb31caa_20260903THsGCoz/report.md`.
  Rules/model/data, provider adapter, signed cartridge/schema, local-play,
  scripts, and provenance surfaces were all accounted for with no deferred
  coverage.
- One conditional candidate was retained and explicitly dispositioned instead
  of silently discarded: `scripts/play.sh` supplies its ephemeral loopback
  capability to QML through process arguments, which may be visible to another
  OS user on permissive multi-user procfs. Static dataflow and non-secret host
  metadata supported the condition, but final policy is `ignore`: the proven
  effect is limited to one developer-only, in-memory game session and conveys
  no production grant, account, credential, database, or durable provider
  authority. Reassess if that launcher becomes a multi-user product surface or
  carries production/durable state.
- The security access advisory could not verify TAC status because its
  connector was unavailable. This did not gate the completed local scan.
- Direct implementation inspection found no duplicate Level 16 action, view,
  or data authority: the fixed action maps to literal level 16, `option_p` is
  presentation-only and populated only in dungeon state, the reducer rejects
  levels outside 1–16 before mutation, and normal selection excludes records
  150 and 160.
- Fresh platform CodeGraph inspection re-traced authenticated
  `compile_render_plan`/`RenderedNode` lowering and its runtime, preview, and
  renderer-test consumers. The platform production graph is unchanged; the
  only platform edit remains the game-neutral QML test ratchet. QML is outside
  the Rust AST graph and was covered directly by the passing recursive
  instantiated-delegate and real-input tests.
- The only post-scan correction was validation-only: the Level 16
  `#[cfg(test)]` provider profile was extended through cast, successful
  retreat, and Main Street, while `scripts/test-provider.sh` stopped rewriting
  that successful-retreat state into the Level 15 death/re-entry path. Direct
  delta inspection confirmed no production rule, action, state, authority,
  credential, or network surface changed after the frozen security review.
- Phase 3.5 exit: no unresolved correctness, architecture, security, privacy,
  state/concurrency, reconnect, rollback, or performance finding blocks full
  validation.

## Phase 4 — Validate

- Full external validation passed after the final correction with the ambient
  target directory removed and offline dependency resolution:
  `env -u CARGO_TARGET_DIR TMPDIR=/tmp CARGO_NET_OFFLINE=true
  OMARCHYGS_PLATFORM_ROOT=/srv/stacks/omarchy_gaming_system
  ./scripts/test.sh`. Evidence includes formatting, warnings-denied Clippy,
  rustdoc, source/provenance verification, 18 data tests, 30 provider tests,
  three local-session tests, 59 rules tests, one integration test, six
  trusted-control QML cases, seventeen signed screens, and provider-backed
  local-play smoke.
- The first production conformance attempt correctly rejected one impossible
  validation command with HTTP 422. Investigation showed that Level 16's
  deterministic first retreat succeeds back to the dungeon, while the corpus
  still rewrote the following Main Street command to the Level 15
  death-only `reenter` action. No game transition was wrong. The corpus now
  preserves Main Street after that successful retreat, and the authenticated
  Level 16 provider test covers cast, retreat, positive HP, cleared monster,
  and return to Main Street.
- The corrected production provider passed its fixed fifteen-case TLS,
  authentication, replay, timeout, fault, callback, reconciliation, and
  persistence corpus twice across a real process restart against the isolated
  ticket PostgreSQL 18 instance. The host PostgreSQL service was neither
  stopped nor reused.
- The complete platform `bin/gate.sh --diff` ran with `/tmp`, no ambient Cargo
  target directory, the ticket-scoped Compose database, and the process-local
  `127.0.0.1:5432` redirect. All twenty-four stages passed, including the full
  PostgreSQL/API/QML suite, reproducible native package, remote-provider
  security conformance, sidecar/authority drills, backup/restore, private-alpha
  admission, and server-module containment; it printed `GATE GREEN [diff]`.
- A fresh rules-v21 provider-backed QML process replaced the old v20 preview
  and remains mapped on Hyprland workspace 8 while workspace 1 stayed active.
  Real compositor-directed Return/Tab input advanced exactly once through
  entry, Human, Alchemist, Main Street, Dungeon, Level 16, and Look: revisions
  zero through six were strictly monotonic with no double activation.
- Live authenticated readback at revision 5 showed the Level 16 narrative and
  nineteen buttons with nineteen unique IDs and actions. Revision 6 showed a
  Level 16 Red Spider at 60/60 HP with five combat buttons, five unique IDs,
  and five unique actions. The old window alone was closed; the fresh Level 16
  combat screen remains open on workspace 8 for user observation.
- The reported duplicate/inert-button symptom was treated as a release
  blocker. The live cardinality/readback and real Return/Tab transitions agree
  with the QML suite's pointer activation, disabled-to-enabled transition,
  single-Return emission, stale-delegate removal, and exact 18-to-19 recursive
  instantiated-delegate replacement.
- Phase 4 exit: REQ-001 through REQ-006 have exact source, implementation,
  isolation, full-regression, restart, real-input, and live-runtime evidence.
  No unresolved product failure blocks completion.

## Phase 5 — Complete

- Audited REQ-001 through REQ-006 against the source trace, implementation,
  signed screens, hostile/replay coverage, provider restart corpus, QML
  delegate/input tests, and fresh workspace-8 runtime. All requirements have
  direct passing evidence and no scoped item was silently dropped.
- Updated the external compatibility/port map and platform architecture plus
  the generated OpenWiki quickstart and cartridge pages through rules v21,
  Level 16, `option_p`, boundary record 150, and the 18-to-19 delegate proof.
  The OpenWiki lifecycle completed; its pre-existing unresolved Claims debt
  warnings were retained rather than bypassed or edited manually.
- Submitted `AAR-070`, recorded the conformance terminal-phase drift failure,
  and added a prevention rule binding fixed corpus commands to focused
  next-phase assertions.
- Closed Ticket 070 and archived this spec/notes pair. Delivery remains
  unauthorized: no commit, push, pull request, packaging, registration,
  admission, deployment, or publication was performed.

---
title: Usurper one BBS day and trusted preview — notes
pipeline_id: 523ecc99-9c51-4cca-bb2f-597075f23baa
---

# Usurper one BBS day and trusted preview — running notes

Chronological evidence and decisions. If a test or command did not run, these
notes must not claim it passed.

## Phase 1 — Plan

- Recall:
  - no active pipeline or blocking bulletin existed; Ticket 048 was next;
  - Ticket 047 authenticated canonical v0.20e commit
    `6d4a100bb271c1d28a752c3e1514ba7b11c14fe2`, excluded the stale nested tree,
    historical binaries, unmarked infrastructure, and uncleared ANSI art, and
    mapped one complete BBS day as Milestone 1;
  - `AD-omarchy-gaming-system-usurper-v020e-deterministic-provider-port-001`
    requires a separate GPL rules/provider repository, independent state, an
    inert cartridge, and no compiled fallback;
  - `PR-omarchy-gaming-system-prove-provider-state-topology-fit-001` permits the
    32 KiB session starter only for the solo proof and requires a reviewed
    shared-realm seam before town/social milestones;
  - cartridge/runtime knowledge requires exact signed data, bounded views and
    actions, trusted platform QML, and real vertical-slice evidence;
  - provider knowledge requires public-artifact consumption, expected revision,
    replay-before-current checks, separate provider persistence, scoped
    identity, and conformance/fault coverage.
- Preflight:
  - `scripts/check-pipeline-tools.sh` passed;
  - `docker compose ps` found no running development services; the later live
    PostgreSQL/QML proof must start its own bounded environment;
  - the adjacent Usurper repository contains only provenance/map files plus
    ignored authenticated upstream bytes and has no commit or remote.
- Decisions:
  - ship the complete mapped Milestone 1 day and trusted preview as one vertical
    slice;
  - use the current public provider starter only for the bounded solo state;
  - keep the alias explicitly fixed for development until trusted input exists;
  - leave packaging, publication, production admission, shared realm, mature
    content policy, and historical art outside this ticket.

## Phase 2 — Design

- Architecture:
  1. `usurper-model` owns the bounded serialized session state, ordinal-stable
     races/classes/phases, commands, views, facts, and indexed RNG trace. It has
     no SQL, network, filesystem, clock, entropy, platform ID, or SDK dependency.
  2. `usurper-data` owns only reviewed Milestone 1 tables: all ten race and
     eleven class creation modifiers, the level-one encounter catalog, and
     level thresholds. Every row carries canonical file/symbol/line evidence;
     randomized editor outputs remain development fixtures rather than claimed
     original `MONSTER.DAT` bytes.
  3. `usurper-rules` owns pure launch/reduce/view functions. An explicit
     deterministic stateful RNG records `{index,bound,result}` for each draw;
     invalid commands are validated before constructing the RNG and consume no
     draws. The state carries the RNG state/index and realm day, so the provider
     trait never reaches ambient time or entropy.
  4. Character creation follows `USERHUNC.PAS`: fixed development alias,
     ordinal race/class selection, Troll/Orc Paladin rejection, class base
     values, then race modifiers and draws, followed by `maxhps := hps`.
  5. Dungeon selection follows `DUNGEONC.PAS`: one fight is consumed before a
     normal encounter and level-one selection retains the original exclusive
     lower-bound loop. Combat follows the first normal solo path in
     `PLVSMON.PAS`/`VARIOUS.PAS`, including monster HP `strength * 3`, attack
     draw order, displayed-but-not-subtracted innate defence oddity, retreat
     coin flip/back damage, source-linked XP/gold draws, death, and potion
     healing. Unsupported events, equipment drops, spells, teammates, and
     diseases do not silently execute.
  6. Level eligibility and the first raise use the canonical editor threshold
     and `VARIOUS2.PAS` default-master class gains. The Healer implements the
     source disease/cost boundary; a healthy first-day character receives the
     original no-disease result. Sleep records the dormitory location, advances
     the configured day once, resets the human player's daily dungeon-fight
     allowance, retains HP/potions, and emits bounded news before an explicit
     re-entry command. Dead-player re-entry restores HP separately.
  7. `usurper-provider` converts exact JSON envelopes to model commands and
     implements only `ProviderGame`; the existing starter remains responsible
     for TLS/protocol/grants, revisions, operation replay, PostgreSQL storage,
     callback outbox, and lifecycle. A final day-advance state emits a
     `TurnReady` callback and remains active because Usurper is persistent.
  8. The signed Core cartridge contains eleven screens and a common bounded
     view schema. It uses only terminal, status, meter, and button nodes.
     Gameplay actions travel to the provider; explicit `navigate.<screen>`
     actions are trusted local navigation. The tracked show script prepares a
     signed development cartridge/render plan and launches the repository's
     platform-owned QML preview; generated keys/archive/plan stay ignored.
  9. No platform gameplay route, database table, catalog admission, compiled
     rule, or publisher QML is added. The platform keeps identity/session/
     broker/rendering authority and the adjacent provider keeps game state.
- CodeGraph design evidence:
  - pipeline-bound exploration traced `ProviderGame::{launch,command,view,event}`
    through the public starter boundary and Relay Forge substitution pattern;
  - a second exploration traced authenticated cartridge identity and
    presentation through `VerifiedCartridge`, render-plan compilation, session
    action validation, and the trusted QML surface;
  - direct inspection supplemented unsupported Pascal, JSON schema, shell, and
    QML details. No platform gameplay implementation caller requires a change.
- Design defect discovered:
  - the public conformance runner is described as game-agnostic but hard-codes
    Relay Forge's `mine`, `charge`, and `forge` actions and a terminal final
    status. A persistent Usurper provider cannot honestly pass that corpus.
  - narrowly generalize the public runner with a bounded validated gameplay
    profile: one launch payload, one timeout/replay command, a finite
    continuation command list, and an expected final active/completed status.
    Preserve the current Relay Forge sequence as the default so existing CLI,
    receipts, and release bytes remain compatible except for the intentional
    public crate source/version digest change. Usurper supplies real first-day
    commands and ends with an active `TurnReady` event.
- File manifest — platform repository:
  - `crates/provider-conformance/src/runner.rs` — public bounded gameplay
    profile and profile-driven fixed security/fault corpus;
  - `crates/provider-conformance/src/lib.rs` — export the profile type;
  - `crates/provider-conformance/README.md` and `kit/v1/README.md` — describe
    game-specific command profiles without weakening the fixed security cases;
  - existing provider focused tests/scripts — retain default Relay Forge proof
    and add profile validation/nonterminal behavior coverage;
  - current Ticket 048 spec/notes/AAR/index and later durable architecture/wiki
    records — workflow and knowledge evidence.
- File manifest — adjacent Usurper repository:
  - `.gitignore`, `Cargo.toml`, `Cargo.lock`, `LICENSE`, `README.md` — workspace,
    GPL identity, generated-state exclusions, and developer entry points;
  - `crates/usurper-model/{Cargo.toml,src/lib.rs}` — bounded domain types,
    commands, state, views, facts, and RNG trace;
  - `crates/usurper-data/{Cargo.toml,src/lib.rs}` — source-linked race/class,
    level, and level-one monster tables;
  - `crates/usurper-rules/{Cargo.toml,src/lib.rs,tests/one_day.rs}` — pure
    reducer/view, deterministic RNG, creation/combat/maintenance behavior, and
    complete-day fixtures;
  - `crates/usurper-provider/{Cargo.toml,src/lib.rs,src/main.rs}` — public SDK
    adapter plus private-config starter process;
  - `cartridge/{manifest.json,presentation.json,schemas/view.schema.json}` —
    signed inert first-day presentation source;
  - `fixtures/presentation/*.json` and `fixtures/preferences.json` — bounded
    per-screen ready views and trusted-host preferences;
  - `docs/COMPATIBILITY.md` and `provenance/source-trace.json` — explicit parity
    status and Pascal-symbol/Rust/test traceability;
  - `scripts/{test.sh,test-provider.sh,test-cartridge.sh,show.sh}` — sequential
    external checks, packaged public SDK consumption, signed cartridge/QML
    smoke, and visible local preview; all generated material uses private
    temporary or ignored directories.
- Database and migration consequences:
  - no OmarchyGS migration or game table is added;
  - the solo provider uses the existing starter migration in a dedicated
    provider database, with state/revision/receipts/outbox owned there;
  - Usurper-specific shared-realm migrations remain deferred until the reviewed
    pre-Milestone-3 seam.
- API and compatibility:
  - platform REST, cartridge protocol v1, provider wire protocol v1, and
    `ProviderGame` remain unchanged;
  - the conformance library gains an additive Rust builder/type and defaults to
    its exact existing Relay Forge profile; receipt format and fifteen fixed
    security/fault case IDs do not change;
  - Usurper command payload is strict `{command:{action,...}}`; race/class and
    dungeon level use bounded enum/integer fields, and unknown/extra input
    rejects before state or RNG changes.
- Risks and controls:
  - licensing: only GPL-marked source logic/tables are translated; historical
    art/binaries/unmarked units remain ignored; public distribution still waits
    for explicit Provider toolkit terms;
  - determinism: source draw order is recorded, while the development RNG is
    explicitly not claimed as Borland parity until an oracle exists;
  - state bounds: serialize every transition/view and assert below the starter
    32 KiB and renderer 256 KiB limits; retain only a bounded last-operation
    trace rather than an unbounded session log;
  - replay/concurrency: reducer purity is not enough; starter PostgreSQL tests
    must prove replay, stale revision, restart, callback and timeout recovery;
  - presentation: all strings are plain bounded data and every action is
    declared; QML executes only platform-owned components;
  - persistent semantics: sleeping does not mark the provider session complete;
    day advance emits `TurnReady` and requires explicit re-entry;
  - rollback: the external source is uncommitted and independent, platform
    conformance changes are additive, and no database migration is introduced.
- Regression plan:

  | Requirement | Evidence |
  |---|---|
  | REQ-001 | workspace fmt/Clippy/test/doc; SPDX/license and tracked-file scans; upstream checksum; machine-readable source trace validation |
  | REQ-002 | all race/class tables, invalid Troll/Orc Paladin pairs, deterministic creation twin, exact initial draw bounds/order, bounded Main Street view |
  | REQ-003 | encounter lower-bound selection, fight decrement, attack victory/death, retreat success/failure/death, reward and no-draw invalid/stale fixtures |
  | REQ-004 | potion/disease healer cases, level threshold/raise gains, sleep/day receipt/replay, reset/news/re-entry full-day twin fixture |
  | REQ-005 | adapter unit tests; packaged SDK clean build; generalized fifteen-case live TLS/PostgreSQL conformance with active final status and `TurnReady`; restart/privacy/size checks |
  | REQ-006 | pack/conform; every screen compiled; ready plus fixed unavailable-state QML smoke; keyboard/action metrics; visible preview launch/capture evidence |
  | REQ-007 | Git/dependency/route/migration/cartridge inventory; CodeGraph inspection; full `bin/gate.sh --diff` |

## Phase 3 — Implement

- Built:
  - generalized the public provider conformance runner with a bounded
    `ConformanceGameplayProfile` while retaining Relay Forge as the exact
    default and preserving all fifteen receipt/security/fault case IDs;
  - replaced the platform's pilot-specific provider-view gate with the public
    SDK's bounded safe-payload validator plus the documented 64 KiB view bound;
  - created the separate GPL-2.0-or-later Rust workspace with model, source
    tables, pure reducers, provider adapter/process, complete license text,
    compatibility ledger, and machine-readable source trace;
  - implemented fixed-development-alias character creation across all ten
    races and eleven classes, the level-one normal dungeon path, combat,
    retreat/death/reward, quick healing, healer, level master, mail/news,
    sleep/day advance, and re-entry with deterministic indexed RNG evidence;
  - consumed packaged public SDK/starter crates through ignored extracted
    Cargo packages rather than platform-internal path dependencies;
  - added a standalone TLS/PostgreSQL provider process and a live conformance
    harness that runs the fixed corpus twice across provider restart;
  - added an inert signed Core cartridge with entry, race, class, Main Street,
    status, dungeon, combat, healer, level-master, mail/news, and sleep screens;
  - added cartridge packing/conformance, all-screen render-plan compilation,
    trusted QML state smoke, and a visible signed Main Street preview. The live
    preview recorded unconfirmed `status` and `navigate.status` input while the
    platform-owned QML process remained active.
- Deviations:
  - implementation source inspection corrected three design assumptions:
    v0.20e applies one innate monster-defence draw on the normal path rather
    than two; the third reviewed level threshold is `10000`; and human daily
    maintenance retains HP/potions because the nearby refill branch is for AI
    players. These corrections are explicit in the compatibility ledger.
  - the provider's `TurnReady` payload was narrowed to the platform's exact
    `{view}` schema; realm-day semantics remain inside provider-owned state and
    the view instead of creating an undeclared callback field.
  - a demonstrated second platform defect expanded the platform file manifest
    to `crates/server/src/provider_games.rs`: authenticated views were required
    to contain exactly the five Door Legends strings. The fix validates any
    non-empty, SDK-safe, at-most-64-KiB object without changing response
    authentication, revision, release, callback, or cartridge-schema checks.

## Phase 3.5 — Inspect ledger

| # | Lens | Finding | Severity | Disposition |
|---|---|---|---|---|
| 1 | Public SDK consumer fit | The supposedly game-neutral conformance runner hard-coded Relay Forge commands and terminal status. | medium | Fixed with a bounded optional gameplay profile; the original profile remains default and the exact fifteen-case inventory is unchanged. |
| 2 | Legacy source fidelity | Initial interpretation doubled the normal monster innate-defence behavior, used a stale third threshold, and treated the AI-only maintenance refill as human behavior. | medium | Corrected from direct canonical Pascal line inspection; compatibility/source-trace docs and deterministic tests now reflect the original branches. |
| 3 | Callback exactness | The first Usurper `TurnReady` event added `event` and `realm_day` fields rejected by the platform's `deny_unknown_fields` callback payload. | medium | Reduced the event to exact `{view}` and reran live callback/retry conformance twice across restart. |
| 4 | Provider projection portability | `apply_authenticated_response`, result callbacks, turn-ready callbacks, and `upsert_view` all called a validator requiring the Door Legends five-field vocabulary. Any second provider could conform yet fail platform projection. | high | Replaced the pilot schema with non-empty, 64-KiB, `validate_provider_payload`-safe object validation; direct tests cover legacy and Usurper acceptance plus hostile shapes. |
| 5 | Authority and dependency boundary | External rules/model/data crates contain no platform/server, SQL, filesystem, network, clock, or ambient-entropy dependency; only the provider adapter consumes packaged public SDK/starter crates. | informational | PASS; Cargo manifests, source scan, runtime process, and separate provider database confirm the boundary. |
| 6 | Trusted presentation | Cartridge inventory is JSON/schema only and every screen compiles through the production verifier/renderer; no publisher QML, JavaScript, executable bit, URL, native code, historical ANSI, or asset is present. | informational | PASS; signed pack/conform, eleven render plans, and ready/unavailable-state QML smoke completed. |
| 7 | Fresh structural inspection | Pipeline-bound CodeGraph traced the generalized profile and provider view gate through conformance, authenticated response, result, turn-ready, and persistence callers without finding an uncovered projection path or remaining game-specific payload. | informational | PASS for the platform repository. The adjacent repository has no `.codegraph` index, so direct source, manifest, script, test, and canonical-Pascal inspection supplied its evidence. |

## Phase 4 — Validate

- Tests run:
  - `cargo test -p omarchygs-provider-conformance --lib --bins` — PASS, five
    tests including persistent profile bounds and unchanged receipt inventory;
  - focused server binary test `provider_view_validation` — PASS, two tests;
  - adjacent `scripts/test.sh` — PASS: fmt, Clippy with warnings denied, all
    workspace/unit/integration/doc tests, canonical source hashes and clean Git
    tree, source trace, privacy scan, signed eleven-screen cartridge, and QML
    ready/loading/offline/empty/protocol-error smoke;
  - adjacent `scripts/test-provider.sh` — PASS: real PostgreSQL, separate TLS
    provider, exact fifteen-case replay/fault/callback corpus, delivered
    `TurnReady`, durable receipts, and a second complete run after restart;
  - focused visible `scripts/show.sh main-street` — PASS: signed Main Street
    plan opened in the platform-owned QML preview and remained responsive to
    action requests.
- Gate run:
  - `bin/gate.sh --fast` — GREEN after implementation and Phase 3.5 fixes.
  - `bin/gate.sh --diff` — pending Phase 5 documentation/OpenWiki completion.
- Skips or pre-existing failures:
  - no production provider registration, server admission, marketplace
    publication, external deployment, or original DOS RNG oracle was attempted;
    each remains explicitly outside Ticket 048.
  - the adjacent game repository has no CodeGraph index, as recorded above.

## Phase 5 — Complete

- Acceptance-criteria audit:
  - REQ-001 — PASS: the adjacent GPL workspace builds cleanly, retains the
    authenticated ignored upstream tree unchanged, and validates thirteen
    machine-readable source-trace entries against the canonical commit;
  - REQ-002 — PASS: table-driven tests cover all ten races, eleven classes,
    forbidden Paladin pairs, deterministic fixed-alias creation, initial draw
    traces, strict commands, revisioned state, and bounded Main Street views;
  - REQ-003 — PASS: reducer tests cover the canonical level-one selection
    bounds, fight consumption, attack victory/death, retreat outcomes, rewards,
    stale input, and no-draw rejection;
  - REQ-004 — PASS: the complete deterministic one-day fixture covers healer,
    level master, sleep, exactly-once day advance, human counter reset,
    retained HP/potions, news, callback, and re-entry;
  - REQ-005 — PASS: the adapter implements only `ProviderGame`, consumes
    packaged public crates, keeps private state in its own PostgreSQL database,
    and passes the fixed fifteen-case corpus twice across restart with bounded
    privacy-scanned state and views;
  - REQ-006 — PASS: the signed inert cartridge packs and conforms, all eleven
    screens compile, fixed QML states pass, and the visible Main Street preview
    accepted trusted host navigation requests;
  - REQ-007 — PASS: the cross-repository dependency/content review found no
    platform Usurper rule, migration, route, registration, admission,
    publication, publisher executable presentation, historical ANSI, or
    compiled fallback. The full local diff gate supplies the final repository
    delivery proof.
- Docs:
  - OpenWiki update run `f0767e8d-7944-4365-942f-c2c4f958e007` completed after
    reconciling `quickstart.md` and `game-cartridges.md`; finalization retained
    pre-existing evidence-debt warnings for both overview pages but completed
    the lifecycle;
  - durable current-state prose now distinguishes the generalized public
    contracts and local Usurper proof from production registration/admission.
- AAR:
  - submitted `AAR-048` with three failures, three prevention rules, and the
    governing existing Usurper architecture decision;
  - every new `BF-` and `PR-` ID was appended to the knowledge register.
- Archive:
  - Ticket 048 closed and the spec/notes pair moved to `pipeline/completed/`;
    no active pipeline pair remains.

## Defect and lesson ledger

| # | What happened | Root cause | Fix | Prevention |
|---|---|---|---|---|
| 1 | A persistent provider could not use the public conformance runner. | Game vocabulary and terminal status were embedded in an otherwise reusable security corpus. | Added one bounded validated gameplay profile while retaining Relay Forge as the default and preserving all fifteen cases. | `PR-omarchy-gaming-system-parameterize-gameplay-without-weakening-provider-security-corpus-001` |
| 2 | An authenticated Usurper view would fail platform projection. | The provider bridge treated Door Legends' exact view fields as a platform protocol. | Replaced the pilot gate with non-empty public safe-payload validation and a 64 KiB bound. | `PR-omarchy-gaming-system-validate-provider-views-by-public-bounds-not-pilot-vocabulary-001` |
| 3 | Three first-pass port assumptions disagreed with canonical Pascal behavior. | Nearby branches and constants were read without fully proving actor/path scope and canonical declaration. | Corrected defence draws, the level threshold, and human maintenance behavior; recorded them in compatibility tests. | `PR-omarchy-gaming-system-prove-legacy-branch-scope-before-porting-behavior-001` |

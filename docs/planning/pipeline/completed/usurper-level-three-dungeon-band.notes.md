---
title: Usurper Level-Three Dungeon Band — notes
pipeline_id: eca339a1-38b9-40e1-b844-2138df71ae1f
---

# Usurper Level-Three Dungeon Band — running notes

## Phase 1 — Recall and plan

- User direction remains to continue building the game visibly while deferring
  packaging and delivery.
- No active pipeline, open ticket, or blocking bulletin existed; Ticket 055
  was next.
- Recalled knowledge:
  - `PR-omarchy-gaming-system-prove-legacy-branch-scope-before-porting-behavior-001`
    requires retaining the level-band rejection loop and its unused boundary
    record separately;
  - `PR-omarchy-gaming-system-declare-legacy-mode-before-composing-subsystems-001`
    keeps this slice in solo non-classic normal dungeon combat;
  - `PR-omarchy-gaming-system-preserve-discarded-legacy-rng-work-001`
    applies to rejected encounter candidates because they still advance every
    later deterministic draw;
  - `PR-omarchy-gaming-system-reconcile-composite-command-drivers-001`
    requires checking the full provider profile after altered encounter draws.
- Ticket 054 supplies the current rules-v7 levels-one/two baseline, opaque
  provider state boundary, signed seventeen-screen cartridge, and complete
  combat composition.
- Canonical v0.20e source readback:
  - `EDMONST.PAS:2802-2902` stores ten level-three records at indices 20–29,
    all with reviewed base strength 13 and exact armor/weapon-user flags;
  - `DUNGEONC.PAS:579-586,748-804` initializes and changes dungeon level;
  - `DUNGEONC.PAS:937-955` spends a fight and repeats `Random(level*10)` until
    the result is greater than `(level-1)*10`;
  - `PLVSMON.PAS:616-624` initializes a loaded monster's HP to strength times
    three.
- The platform CodeGraph shows `ProviderGame` receives and returns opaque
  `serde_json::Value` gameplay state behind authenticated provider routing;
  no platform Rust, schema, migration, route, or QML change is needed.
- The separate Usurper repository has no CodeGraph index, so its Rust, JSON,
  scripts, fixtures, and tests are inspected directly as required by the tool
  fallback guidance.
- Pipeline tools are ready and PostgreSQL is healthy.
- Phase 1 exit: scope, six EARS requirements, five locked decisions, Ticket
  055, active spec/notes, and AAR are established.

## Phase 2 — Design

- Canonical roster mapping:

  | Index | Name | Base strength | Armor user | Weapon user | Normal level-three selection |
  |---:|---|---:|---|---|---|
  | 20 | Medium Troll | 13 | yes | yes | unreachable boundary record |
  | 21 | Psycho Ape | 13 | no | no | reachable |
  | 22 | Pet Dragon | 13 | yes | no | reachable |
  | 23 | Ugly Mummy | 13 | no | yes | reachable |
  | 24 | Dwarf Wrestler | 13 | yes | no | reachable |
  | 25 | Ugly Woodman | 13 | yes | yes | reachable |
  | 26 | Small Griffin | 13 | no | no | reachable |
  | 27 | Madman | 13 | yes | yes | reachable |
  | 28 | Crazy Guard | 13 | yes | yes | reachable |
  | 29 | Drunk Mutant | 13 | yes | yes | reachable |

- Architecture and data flow:
  1. The signed inert dungeon screen emits fixed action
     `enter_dungeon_level_3`; the provider adapter maps it to the already
     public game-owned `Command::EnterDungeon { level: 3 }`.
  2. `reduce` validates current v8 state and command phase before cloning state
     or creating RNG, then `enter_dungeon` switches among levels one through
     three without a draw.
  3. `Look` clears encounter-local spells, spends one fight, and preserves
     each `Random(30)` candidate until one exceeds 20; `monster_seed` resolves
     that accepted exact record and initializes strength/defence/39 HP.
  4. Existing attack, spell, class-special, potion, reward, retreat, poison,
     death, and re-entry reducers consume the level-three state unchanged;
     retreat derives its failure-damage draw bound from level as 30.
  5. The provider starter persists the strict JSON state and replay receipt;
     `view` exposes only bounded presentation facts. OmarchyGS transports the
     opaque action/state/view and trusted QML renders the signed data.
- Exact file manifest:

  | Repository/file | Purpose |
  |---|---|
  | external `crates/usurper-data/src/lib.rs` | add exact level-three records, lookup band, and table tests |
  | external `crates/usurper-rules/src/lib.rs` | advance v8, permit levels 1–3, label level three, and prove selection/state/combat bounds |
  | external `crates/usurper-provider/src/lib.rs` | decode fixed level-three action and prove generic/fixed/replay/view behavior |
  | external `cartridge/manifest.json` | pin rules/cartridge v8 |
  | external `cartridge/presentation.json` | add one inert level-three button/action |
  | external `fixtures/presentation/{dungeon,combat}.json` | visible level-three signed examples |
  | external `provenance/source-trace.json` | add exact level-three source/data proof and retarget generic selection/retreat proofs |
  | external `scripts/test.sh` | require the expanded provenance floor and identify the level-three milestone |
  | external `scripts/test-provider.sh` | pin v8 and drive a deterministic level-three profile twice across restart |
  | external `README.md`, `docs/COMPATIBILITY.md`, `docs/RUST_PORT_MAP.md` | state exact implemented and deferred compatibility scope |
  | platform ticket/spec/notes/AAR/index | retain the auditable lifecycle record |
  | platform `docs/architecture/game-cartridges.md` and generated OpenWiki pages | reconcile the development proof at completion |
- Database and migration consequences: none. The provider starter already
  persists bounded game-owned JSON and exact operation receipts in its own
  PostgreSQL database; no schema, transaction, identity, or platform database
  change is introduced.
- API and compatibility behavior:
  - game/state/rules/cartridge identity advances from v7 to v8; v7 state is
    deliberately rejected rather than silently migrated;
  - `Command::EnterDungeon { level }`, Provider SDK v1, presentation protocol
    v1, view schema, and trusted node vocabulary remain unchanged;
  - one new fixed zero-field action is strictly decoded, and all values above
    three or below one reject before state/RNG mutation;
  - no production release, admission, or compatibility promise is made.
- Regression map:

  | Requirement | Evidence |
  |---|---|
  | REQ-001 | exact ten-row data names/order/strength/flags test, canonical checksum/clean-source checks, and new source-trace entry |
  | REQ-002 | v7 rejection; zero/four/maximum level rejection; wrong monster level, boundary index 20, unknown index/name/scalar/schema JSON cases |
  | REQ-003 | Main Street/Dungeon level 1–3 transitions, narration, empty monster, exact RNG equality, and three projected labels |
  | REQ-004 | seeded rejected candidate then accepted 21–29, all bound 30, one fight spent, 13/6/39 scalars, deterministic twin |
  | REQ-005 | level-three failed retreat trace `(2, _), (30, _)`; attack plus representative spell/special/poison/potion/reward composition through existing full tests |
  | REQ-006 | generic/fixed provider equivalence, replay, restart profile, signed cartridge checks, trusted QML smoke, visible dungeon/combat preview, platform gate |
- Security, privacy, concurrency, reconnect, and rollback risks:
  - malformed persisted level/monster pairs must fail before any command or
    projection can use them;
  - added rejection draws change every later deterministic result, so the
    complete live profile must assert actual phase outcomes and restart replay;
  - the button remains inert data and cannot gain executable, network,
    filesystem, credential, account, or persona authority;
  - state remains under 32 KiB and authenticated provider operations retain
    expected revision, idempotency, replay, callback, and reconciliation;
  - rollback is removal of the uncommitted external/platform changes; v8 does
    not accept v7 state and no migration is claimed.
- Alternatives rejected:
  - directly sampling nine rows would erase rejected RNG work;
  - accepting level four with placeholder data would create false parity;
  - importing `dungeon_event` would combine a larger composite dispatcher with
    this bounded normal-encounter slice;
  - adding platform game logic or executable QML would violate the established
    single-authority and trusted-renderer boundaries.
- CodeGraph design inspection traced `ProviderGame::command`/`view`, server
  provider routing, session cartridge actions, and trusted renderer consumers.
  It confirms the new state/action stay opaque bounded JSON and require no
  platform application-code change. External files were inspected directly
  because that separate repository has no CodeGraph index.
- Phase 2 exit: the source map, file manifest, compatibility behavior, risk
  treatment, and requirement-to-evidence plan are actionable.

## Phase 3 — Implement

- External data and lookup:
  - added exact level-three indices 20–29, names, base strength 13, and source
    armor/weapon-user flags;
  - extended `monster_seed` only through index 29 and added exact data tests.
- External rules/state:
  - advanced the exact state/rules identity to v8 and accepted only dungeon
    levels one through three;
  - extended the existing draw-free level switch and dungeon view with level
    three while rejecting zero, four, and maximum unchanged;
  - retained the generic rejection loop, so level three records every
    `Random(30)` candidate through the first result greater than 20, excludes
    boundary record 20, and initializes an accepted monster to strength 13,
    defence 6, and 39 HP;
  - added exact level-three failed-retreat draw/bound/damage coverage and v8
    hostile state cases for prior schema, unsupported/wrong level, boundary,
    unknown record, wrong name, and oversized scalar.
- Provider and persistent profile:
  - decoded fixed `enter_dungeon_level_3`, added generic/fixed equivalence,
    replay, projected view, bounded-state, and return-to-level-two tests;
  - moved the permanent Gnoll Cleric live profile to level three and retained
    the actual successful-retreat/return phase across provider restart.
- Cartridge, fixtures, provenance, and docs:
  - advanced manifest rules/cartridge identity to v8;
  - added one inert level-three button and zero-field action to the existing
    signed dungeon screen;
  - changed dungeon/combat fixtures to visible level-three examples;
  - added the level-three canonical source trace, raised the checked trace floor
    to 37, and reconciled README, compatibility limits, and port map.
- Focused evidence:
  - data tests: PASS — 5;
  - rules tests: PASS — 33 unit plus 1 complete-day integration;
  - provider tests: PASS — 10;
  - `scripts/test-cartridge.sh`: PASS — seventeen signed screens and trusted
    QML state smoke;
  - `scripts/test.sh`: PASS — formatting, Clippy with warnings denied, 49 Rust
    tests, rustdoc, upstream checksums/clean source, 37-entry provenance,
    privacy scan, signed cartridge, and trusted QML;
  - `scripts/test-provider.sh`: PASS — fixed fifteen-case TLS/replay/fault/
    callback/reconciliation corpus twice across process restart.
- Implementation deviation/failure:
  - the first provider-focused run expected Ticket 054's deterministic death,
    but the level-three rejection work changed the later retreat to success;
    the source-faithful outcome was retained and both unit/live drivers now
    assert Dungeon then `main_street` rather than incorrectly issuing
    `reenter`.
- No platform application, SDK, database, migration, route, or QML source was
  changed.
- Phase 3 exit: the complete designed external file manifest is implemented
  and all focused plus external full/provider checks are green.

## Phase 3.5 — Inspect

- Fresh platform CodeGraph inspection traced the current signed-cartridge
  action boundary through `session_cartridges::translate_command`,
  `provider_games::provider_operation`/`execute_operation`, authenticated
  response projection, the 64-KiB provider-view limit, RenderPlan preparation,
  and trusted QML consumers. A fixed `enter_dungeon_level_3` action becomes the
  opaque provider-owned JSON command `{"action":"enter_dungeon_level_3"}`;
  returned state/view data stays opaque and bounded. No platform application,
  SDK, database, migration, route, renderer, or QML change is needed.
- Cross-repository inspection lenses:

  | Lens | Result |
  |---|---|
  | EARS/correctness | PASS — all six requirements have direct data, reducer, provider, signed-cartridge, replay, or preview evidence. |
  | Determinism/state integrity | PASS — v8 state is strict; level changes are draw-free; every rejected `Random(30)` draw is retained; malformed level/monster pairs reject before mutation. |
  | Simplification | PASS — level three extends the existing generic band lookup, rejection loop, combat reducers, provider adapter, and dungeon screen rather than adding a second rules path. |
  | Security/privacy | PASS — complete current-snapshot scan reviewed all 46 external files with no reportable findings after the credential-harness remediations below. |
  | QML/keyboard/visual | PASS — the added action is inert signed data in the existing keyboard-first button surface; all seventeen screens and loading/offline/empty/protocol-error QML states pass. |
  | Scope/authority | PASS — platform remains the account/session/catalog/cartridge/rendering authority; Usurper remains the only game-rules/state/view authority; packaging and publication remain deferred. |

- Finding ledger and disposition:

  | Finding | Disposition |
  |---|---|
  | Level-three rejected draws changed the live profile's later retreat from Ticket 054's death to a successful retreat. | Fixed the stale test driver and retained the source-faithful result; no rule was bent to satisfy the old expectation. |
  | Initial security scan found the administrator URL in `psql`/`jq` process arguments (low, CWE-214). | Replaced it with password-free psql URLs, a private libpq passfile, and `jq --rawfile`; full provider DSN stays in private files. |
  | Second scan found the documented password-bearing environment override inherited by descendants (low, CWE-526). | Rejected and unset `OGS_TEST_POSTGRES_ADMIN_URL` before the first child; custom credentials now use only a private file path. |
  | Third scan found an ambient exported lowercase `admin_url` collision (low, CWE-526) and retained `sslpassword` in psql argv (low, CWE-214). | Removed the shell secret entirely; Python now opens the private credential file directly. `sslpassword` and every unknown libpq query key fail closed behind a fixed non-secret allowlist. |
  | Separate pathname checks/unbounded input in the intermediate loader. | Hardened beyond the report: one `O_NOFOLLOW|O_CLOEXEC` descriptor, same-effective-UID/mode-0600/one-link/regular-file checks, 64-KiB cap, and before/after metadata stability. |
  | Ambient operator-owned `PG*` variables and an unusual short regular-file read. | Inspected and retained as explicit trusted-launcher/robustness assumptions; no in-scope attacker path or security impact exists. |

- Security evidence:
  - scan 1 report SHA-256
    `9e5e861d1ebdc7f58e44ebd265007dc90e8de9de368569e0b8ffc231e3dd684d`;
  - scan 2 report SHA-256
    `2f321234728d39a314964dd4c24db8d40af88dbab3a5e72b572035096efa0b3f`;
  - scan 3 report SHA-256
    `5ddf3f40ffcb426442f0282fa93f1d464c5adcc121692328ba1e1c089dbfbd53`;
  - final clean scan `88b4949e-e5d3-489c-bb69-507a4b7c7920`, report
    SHA-256
    `ac50fdae9684879f15afabecdd3848b3322362b0813d85e7c7717f7ec5a49bd9`,
    46/46 files reviewed and zero findings.
- Validation after the final security changes:
  - `scripts/test.sh`: PASS — 49 Rust tests, Clippy warnings denied,
    rustdoc, provenance/checksum/privacy checks, signed cartridge, and QML
    smoke;
  - `scripts/test-provider.sh`: PASS in default mode and in private-file mode
    with an ambient exported lowercase marker — each completed the fifteen-case
    TLS/replay/fault/callback corpus twice across restart;
  - an `sslpassword` credential fixture failed closed before PostgreSQL startup.
- Phase 3.5 exit: every inspection finding is fixed or source-backed as a
  non-reportable assumption, fresh CodeGraph evidence covers the platform
  boundary, and the final external security snapshot is clean.

## Phase 4 — Validate

- Acceptance evidence remained green after the final security remediation:
  - external `scripts/test.sh`: PASS — 49 Rust tests, formatting, Clippy with
    warnings denied, rustdoc, upstream checksum/clean-source validation,
    37-entry provenance, privacy checks, seventeen signed screens, and trusted
    QML state smoke;
  - external `scripts/test-provider.sh`: PASS in both default and private-file
    credential modes — the fixed fifteen-case TLS, replay, fault, callback,
    reconciliation, and restart corpus completed twice in each mode;
  - final current-snapshot security scan: PASS — all 46 external files
    reviewed, zero findings, report SHA-256
    `ac50fdae9684879f15afabecdd3848b3322362b0813d85e7c7717f7ec5a49bd9`;
  - platform `bin/gate.sh --diff`: PASS — all 24 numbered checks, including
    Rust/PostgreSQL/QML smoke, SDK release and starter kits, reproducible client
    package, remote-provider security/durability, backup/restore, and server
    module isolation; gate receipt
    `d15fb3906c995e33f42168edb6dd6b7a1b358a0620eac6a08907b8d692a549eb`.
- A fresh visible preview remains open from external run directory
  `.preview/run.DkKn3u`. Its verified render plan is
  `omarchygs.render-plan/v1`, state `ready`, cartridge/rules v8, title
  `The Dungeons`, and narrates dungeon level three. The inert signed surface
  exposes `enter_dungeon_level_1`, `enter_dungeon_level_2`, and the corrected
  `enter_dungeon_level_3` action through the trusted QML renderer.
- Requirement audit:

  | Requirement | Result |
  |---|---|
  | REQ-001 | PASS — exact source links and ten-row level-three roster are retained in data, provenance, and compatibility evidence. |
  | REQ-002 | PASS — strict v8 state validation rejects prior schema, unsupported levels, boundary/unknown records, mismatched names, and oversized scalars before RNG mutation. |
  | REQ-003 | PASS — levels one through three switch without a draw, clear encounter state, and project exact labels; all other levels reject unchanged. |
  | REQ-004 | PASS — level three preserves every `Random(30)` rejection, accepts only records 21–29, spends one fight, and initializes source-derived 13/6/39 combat state. |
  | REQ-005 | PASS — the existing attack, retreat, potion, spell, class-special, reward, and Gnoll-poison regressions compose with the exact level-three retreat bound. |
  | REQ-006 | PASS — provider replay/restart, signed cartridge, trusted QML, platform gate, and visible preview prove the bounded three-level slice without platform game logic or delivery. |

- Phase 4 exit: every EARS requirement has direct passing evidence and the
  visible signed Level 3 build is running; ready for documentation
  reconciliation and lifecycle completion.

## Phase 5 — Complete

- OpenWiki lifecycle `6886f399-9434-4e7b-afb8-d52e8e547ab8` completed. It
  reconciled `openwiki/quickstart.md` and `openwiki/game-cartridges.md` to
  Tickets 048–055, rules v8, levels one through three, exact level-three
  boundary/selection behavior, seventeen signed screens, and unchanged
  platform authority. The lifecycle reported only pre-existing unresolved
  claim debt on those broad pages and still returned `status: complete`.
- Authoritative `docs/architecture/game-cartridges.md` now records Tickets
  047–055, rules v8, levels one through three, normally unreachable boundary
  records 10/20, selectable bands 11–19 and 21–29, and unchanged
  development-only/provider-owned scope.
- AAR-055 is submitted and effective. The existing legacy branch, discarded
  RNG, composite-driver, private-file, and deterministic-provider architecture
  knowledge fully covered this slice; no new knowledge ID was necessary.
- The Phase 4 requirement matrix confirms REQ-001 through REQ-006 PASS with no
  silent drops. Ticket 055 is closed and its ticket, spec, and notes are
  archived under the completed lifecycle paths.
- Delivery remains explicitly deferred: no packaging, production registration,
  admission, commit, push, deployment, or publication was performed.
- Phase 5 exit: documentation is reconciled, the AAR is complete, all six
  acceptance requirements pass, the pipeline is archived, and the visible
  signed Level 3 preview remains running.

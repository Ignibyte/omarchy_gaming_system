---
title: Usurper Level-Six Dungeon Band — notes
pipeline_id: 57bc1563-b5c5-42ef-824f-612c500966e5
---

# Usurper Level-Six Dungeon Band — running notes

## Phase 1 — Recall and plan

- User direction remains to continue building and visibly showing Usurper
  while deferring packaging and delivery.
- No active pipeline, open ticket, or blocking bulletin existed; Ticket 058
  was next. Pipeline tools are ready, PostgreSQL is healthy, and the signed
  Level 5 preview remains open until a replacement Level 6 preview is proven.
- Recalled knowledge:
  - `PR-omarchy-gaming-system-prove-legacy-branch-scope-before-porting-behavior-001`
    requires preserving the stored boundary row independently from normal
    reachability;
  - `PR-omarchy-gaming-system-declare-legacy-mode-before-composing-subsystems-001`
    keeps the slice in solo non-classic normal dungeon combat;
  - `PR-omarchy-gaming-system-preserve-discarded-legacy-rng-work-001`
    makes every rejected `Random(60)` result observable deterministic behavior;
  - `PR-omarchy-gaming-system-reconcile-composite-command-drivers-001`
    requires replaying the complete provider profile after the new earlier RNG
    work changes later outcomes;
  - `PR-omarchy-gaming-system-validate-private-command-files-by-descriptor-001`
    remains applicable to the provider conformance credential boundary;
  - Ticket 057 supplies the rules-v10 levels-one-through-five implementation,
    seventeen-screen cartridge, clean security scan, and visible baseline.
- Canonical v0.20e readback establishes ten Level 6 editor records at indices
  50–59 and the following exact base fixture values:

  | Index | Name | Base strength | Armor user | Weapon user | Normal Level 6 selection |
  |---:|---|---:|---|---|---|
  | 50 | Troll Chief | 16 | yes | yes | unreachable boundary |
  | 51 | Lister | 16 | yes | yes | accepted |
  | 52 | Dragonsoul | 16 | no | no | accepted |
  | 53 | Lost Soul | 16 | no | yes | accepted |
  | 54 | Golden Guardian | 16 | yes | yes | accepted |
  | 55 | Infantry Orc | 16 | yes | yes | accepted |
  | 56 | Orc Sergeant | 16 | yes | yes | accepted |
  | 57 | Splatter Monk | 16 | yes | yes | accepted |
  | 58 | Large Monk | 16 | no | yes | accepted |
  | 59 | Crazy Man | 16 | yes | yes | accepted |

- Source anchors:
  - `SOURCE/EDITOR/EDMONST.PAS:3107-3207` declares the Level 6 rows;
  - `SOURCE/EDITOR/ADDMONST.PAS:43-72` distinguishes editor base strength from
    initialized-world randomization, matching the prior explicit development
    fixture policy;
  - `SOURCE/USURPER/DUNGEONC.PAS:868-955` spends a fight and repeats
    `Random(level*10)` until the candidate exceeds `(level-1)*10`;
  - `SOURCE/USURPER/PLVSMON.PAS:603-625` sets HP to strength times three;
  - `SOURCE/USURPER/PLVSMON.PAS:68-138` makes failed retreat damage use
    `Random(global_dungeonlevel*10)+3`.
- Phase 1 exit: scope, six EARS requirements, six locked decisions, Ticket 058,
  pipeline UUID `57bc1563-b5c5-42ef-824f-612c500966e5`, and open AAR are settled.

## Phase 2 — Design

- Architecture boundary:
  - `usurper-data` owns the exact immutable Level 6 editor fixtures;
  - `usurper-rules` remains the sole game authority for level switching,
    rejection-loop selection, combat, validation, and deterministic RNG;
  - `usurper-provider` only decodes the generic and fixed action forms and
    returns the projected game view;
  - the signed cartridge declares an inert `enter_dungeon_level_6` action and
    binds the already-supported `option_f` field to an ordinary button;
  - the platform and its Rust provider SDK continue transporting opaque
    provider state/actions and rendering typed presentation nodes without any
    Usurper-specific logic.
- CodeGraph design evidence traced a zero-payload cartridge command from
  `session_cartridges::translate_command` through the provider view/render-plan
  seam into QML. The trace confirms that action schemas remain cartridge-owned,
  `ProviderGame` remains the deterministic rules adapter, and the generic
  renderer already supports this button and label. Receipt:
  `.git/omarchy-gaming-system-pipeline-tools/design.receipt`, pipeline
  `57bc1563-b5c5-42ef-824f-612c500966e5`, state hash
  `60279322e51901698d1403ed2076bae87831ada5eb0a5a77363512857385f582`.
- API/state contract:
  - advance the external game state, rules, and cartridge identity from v10 to
    v11, accepting exact v11 only;
  - accept generic `enter_dungeon` levels 1–6 and fixed
    `enter_dungeon_level_1` through `enter_dungeon_level_6`;
  - reject levels 0, 7, and larger unchanged and without RNG work;
  - require active monsters to belong to the selected implemented band and to
    match their exact name and scalar seed;
  - retain record 50 in data but make only records 51–59 selectable by the
    normal Level 6 rejection loop.
- Planned implementation files in the external provider:
  - `crates/usurper-data/src/lib.rs`;
  - `crates/usurper-rules/src/lib.rs`;
  - `crates/usurper-provider/src/lib.rs`;
  - `cartridge/manifest.json`, `cartridge/presentation.json`;
  - `fixtures/presentation/dungeon.json`,
    `fixtures/presentation/combat.json`;
  - `provenance/source-trace.json`;
  - `scripts/test.sh`, `scripts/test-provider.sh`;
  - `README.md`, `docs/COMPATIBILITY.md`, and `docs/RUST_PORT_MAP.md`.
- Platform changes are limited to the Ticket 058 planning, architecture/wiki
  reconciliation, and completion evidence; no server, SDK, QML, route,
  database, migration, or renderer-vocabulary code changes are designed.
- Regression plan:
  - exact ten-row data test plus explicit record-50 boundary assertion;
  - v11 hostile-state, level-switch, rejection-trace, deterministic-twin,
    combat, retreat-bound, provider fixed/generic, view, and replay tests;
  - signed-cartridge fixtures and action declaration checks;
  - complete external workspace checks and the fixed 15-case provider corpus
    twice across restart;
  - platform diff gate, QML smoke, and visible signed Level 6 preview.
- Primary risks are profile RNG shifts, accidentally exposing record 50,
  widening validation without a schema bump, or misrepresenting the inert
  presentation action as platform-owned gameplay. The exact trace assertions,
  strict v11 validation, full provider replay, security inspection, and
  platform boundary review cover those risks.
- Phase 2 exit: the architecture, API/state contract, file manifest, regression
  strategy, and worktree-bound CodeGraph evidence are settled.

## Phase 3 — Implement

- Added exact Level 6 rows 50–59 to `usurper-data`, including source spelling,
  reviewed base strength 16, equipment flags, lookup coverage, and an explicit
  record-50 table assertion.
- Advanced the rules identity to v11; state/commands now accept levels 1–6,
  fixed level switching exposes `option_f`, and encounter selection preserves
  every rejected `Random(60)` result until records 51–59 are selected. Level 6
  monsters initialize at strength 16, defence 8, and 48 HP.
- Added Level 6 rejection-trace, deterministic-twin, boundary, draw-free
  switching, retreat-bound, hostile-state, generic/fixed provider, view, and
  replay regressions while retaining the complete lower-level suite.
- Added the fixed `enter_dungeon_level_6` provider decoder, rules/cartridge v11
  identity, signed inert dungeon button/action, Level 6 fixtures, provenance,
  test-driver expectations, and compatibility/port-map descriptions.
- Focused proof:
  - `cargo test -p usurper-data`: 8 passed;
  - `cargo test -p usurper-rules`: 39 unit + 1 integration passed;
  - `cargo test -p usurper-provider`: 13 passed;
  - `scripts/test-cartridge.sh`: signed seventeen-screen cartridge and trusted
    QML state smoke passed.
- Self-review found no new model, platform, SDK, database, route, or renderer
  change. The cartridge remains executable-free and all new controls use the
  existing provider-owned opaque-command seam.
- Phase 3 exit: the designed source-faithful Level 6 slice is implemented and
  the focused regressions pass.

## Phase 3.5 — Inspect ledger

- Canonical source was reread after implementation. The Level 6 table still
  matches `EDMONST.PAS:3107-3207`; the normal path still spends a fight and
  repeats `Random(60)` until `> 50`; HP remains strength times three; failed
  retreat still uses `Random(level*10)+3`; and the editor-base fixture caveat
  still matches `ADDMONST.PAS`.
- Static inspection confirmed rules/cartridge/conformance identity v11, fixed
  and generic Level 6 convergence, exact records 50–59, normal selection
  51–59, the `option_f`/`enter_dungeon_level_6` inert binding, forty source
  trace entries, and no stale Level 5 ceiling or v10 identity claims.
- Fresh CodeGraph inspection traced the signed action through
  `session_cartridges::translate_command`, provider transport, generic render
  plan, and QML dispatch. No platform server, SDK, QML, route, migration,
  persistence, or renderer-vocabulary change is required. Receipt:
  `.git/omarchy-gaming-system-pipeline-tools/inspect.receipt`, pipeline
  `57bc1563-b5c5-42ef-824f-612c500966e5`, state hash
  `60279322e51901698d1403ed2076bae87831ada5eb0a5a77363512857385f582`.
- Fresh Standard Codex Security scan
  `e7a0ca90-8737-4c15-b95e-11c78d847076` completed all 46 authorized files
  with zero findings. Independent baseline, architecture, and focused
  command/state/RNG/presentation/developer-workflow reviews agreed. Ignored
  upstream/build/preview/local-SDK artifacts and delivery actions remained
  explicit exclusions. Measured scan usage: 5,568,644 total tokens, including
  5,445,632 cached input tokens and 17,368 output tokens.
- External starter authentication/replay/TLS/database enforcement, external
  renderer behavior, production deployment configuration, and whether the
  platform protocol grants the full generic command schema remain documented
  boundary questions rather than game-repository findings.
- Phase 3.5 exit: no finding requires a code change; source provenance,
  architecture boundaries, test-driver reconciliation, and security coverage
  are complete for this slice.

## Phase 4 — Validate

- Pre-rebuild checkpoint validation on 2026-09-01:
  - external `scripts/test.sh` passed formatting, warning-denying Clippy, all
    workspace tests (8 data, 13 provider, 39 rules, and 1 integration),
    rustdoc, immutable upstream hashes and commit, source-trace checks, private
    data-shape scan, all seventeen signed cartridge screens, trusted QML smoke,
    and fixed failure-state QML smoke;
  - external `scripts/test-provider.sh` passed the fixed 15-case
    TLS/authentication/replay/fault/callback/reconciliation corpus twice across
    an independent provider restart and PostgreSQL-backed durable state;
  - platform `bin/gate.sh --diff` completed all 24 current delivery stages and
    printed `GATE GREEN [diff]`, including PostgreSQL, live QML, provider,
    packaging, backup/restore, admission, and process-isolation proofs.
- A newly observed visible Level 6 preview remains required before Phase 4 may
  pass. The offscreen trusted-QML smoke above is automated evidence, not a
  claim that the visible preview was observed during this delivery turn.
- Visible acceptance evidence then passed:
  - `scripts/show.sh combat` opened the signed preview and remains running from
    ignored run `.preview/run.kT7wwc` for the user;
  - `prepared/render-plan.json` is exact `omarchygs.render-plan/v1`, `ready`,
    `core`, and titled `Dungeon Combat`; it visibly projects `A level 6 Lister
    blocks your way.`, `Lister HP 48/48`, and inert attack/retreat/heal/spell/
    class-special controls through the trusted renderer;
  - direct assertions against the render plan passed while the production QML
    process remained alive.
- Requirement audit: REQ-001 through REQ-006 are satisfied by the source
  readback, hostile and deterministic reducer/provider tests, fixed live
  provider corpus, signed-cartridge/QML evidence, security scan, full platform
  gate, and visible signed preview. No requirement was narrowed or dropped.
- Scope audit: no level seven, dungeon event, shared realm, platform gameplay
  rule, migration, registration, admission, deployment, or public Usurper
  release was introduced. The separately authorized private checkpoint commit
  and push occur only after Phase 5 and a final matching gate.
- Phase 4 exits PASS with delivery-gate state hash
  `60279322e51901698d1403ed2076bae87831ada5eb0a5a77363512857385f582`.

## Phase 5 — Complete

- Acceptance criteria: REQ-001 through REQ-006 remain satisfied by exact
  source/data review, hostile and deterministic reducer/provider coverage, the
  full external suite, fixed live provider corpus twice across restart, zero-
  finding security scan, complete platform gate, signed QML smoke, and visible
  Level 6 combat preview. No criterion or exclusion was silently dropped.
- OpenWiki update `45590597-aac7-43d5-9e6e-1d6fbe72022c` finished with status
  `complete`. It reconciled `openwiki/quickstart.md` and
  `openwiki/game-cartridges.md` through rules v11, exact Level 6 boundary and
  selection behavior, and unchanged provider/cartridge/trusted-renderer
  authority. Its warnings are the two broad pages' pre-existing unresolved
  claim debt, not a Ticket 058 verification failure.
- Hand-maintained `docs/architecture/game-cartridges.md` now records Tickets
  047–058, rules v11, selectable levels one through six, boundary record 50,
  accepted records 51–59, and Level 6's 16/8/48 combat initialization.
- `docs/planning/REBUILD_HANDOFF.md`, the active rebuild bulletin, and the
  separate Usurper `docs/REBUILD_HANDOFF.md` preserve the two-repository clone,
  reconstruction, ignored-state, validation, and private-public licensing
  boundaries.
- AAR-058 is submitted and effective. The existing legacy-branch,
  discarded-RNG, composite-driver, private-command-file, and deterministic
  provider-port knowledge covered the slice; no new knowledge ID was needed.
- Ticket 058 is closed and the spec/notes pair is archived. Production
  registration, admission, deployment, shared-realm persistence, and public
  Usurper release remain unauthorized; the user separately authorized the
  private repository checkpoint commit and push.
- Phase 5 exits PASS. The signed Level 6 preview remains open in ignored run
  `.preview/run.kT7wwc` until the user closes it or the development system is
  rebuilt.

## Defect and lesson ledger

| # | What happened | Root cause | Fix | Prevention |
|---|---|---|---|---|
| 1 | The first Level 6 provider live-profile retreat survived, so the old death assertion failed. | Preserving the new `Random(60)` encounter work intentionally shifted all later deterministic draws. | Replayed the source-composed sequence and added a second encounter/cast/retreat before re-entry in both the unit test and conformance profile. | Always replay and reconcile the complete composite driver when a legacy rejection loop adds discarded RNG work. |
| 2 | The full external gate stopped on Clippy's 100-line function limit. | Adding the sixth dungeon label made `configure_labels` 101 lines. | Extracted the static dungeon-label assignments into `configure_dungeon_labels`; behavior and data are unchanged. | Run the full warning-denying gate immediately after focused tests when a branch grows near a lint threshold. |

## Pre-rebuild handoff — 2026-09-01

- The user authorized committing and pushing the complete local checkpoint
  before rebuilding the development system. This delivery authorization
  supersedes the earlier Ticket 058 delivery exclusion; Phase 4 and Phase 5
  subsequently completed on their own recorded evidence.
- The platform delivery target remains the public
  `Ignibyte/omarchy_gaming_system` `main` branch. The separate Usurper target is
  a private `Ignibyte/omarchygs_usurper` `main` branch because the Provider SDK
  still lacks the explicit compatible public copyright grant documented in the
  Usurper README.
- The durable two-repository reconstruction procedure and ignored-state
  boundary are recorded in [`docs/planning/REBUILD_HANDOFF.md`](../../REBUILD_HANDOFF.md).
- The rebuilt machine starts from this completed checkpoint. Re-run the
  complete external Usurper suite and platform diff gate before any new
  delivery; do not reopen Ticket 058 or manufacture its local receipts.

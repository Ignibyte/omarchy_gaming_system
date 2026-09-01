---
title: Usurper v0.20e provenance and Rust port map — notes
pipeline_id: 376a6f08-d054-47ec-ac17-70ad4fa36dd7
---

# Usurper v0.20e provenance and Rust port map — running notes

Chronological evidence and decisions. If a test or command did not run, these
notes must not claim it passed.

## Phase 1 — Plan

- User decision: select Usurper as the first historical BBS game, acquire the
  actual original release locally, and map it before starting the Rust build.
- Upstream recall: `usurper.info` identifies v0.20e as the last release by
  Jakob Dangarden, links the distributed `usurp020e.zip`, and links source
  commit `6d4a100bb271c1d28a752c3e1514ba7b11c14fe2`; GitHub labels that root
  commit “Original, unmodified source.” The source and representative units
  declare GPL version 2 or later.
- Repository recall: product scope rejects ports without verified source and
  asset licensing. ADR-0002 requires BBS-inspired games to live in separate
  repositories and assigns a registered provider sole rules/private-state/RNG
  authority. Tickets 044–046 provide the public Provider SDK, deterministic
  release, starter/conformance suite, second-game proof, and reviewed
  deployment profile without external self-service admission.
- Knowledge recall: independent executable source trees must participate in
  delivery evidence; provider effects retain exact identity, deterministic
  state, expected revisions, replay receipts, and independent persistence.
  The source corpus also contains third-party/Borland/SWAG-era units and ANSI
  artist credits, so repository-level GPL text does not replace a file-class
  provenance audit.
- Tool preflight: `scripts/check-pipeline-tools.sh` passed with CodeGraph 1.5.0
  and OpenWiki 0.3.3 ready. The bulletin register has no active critical or
  warning entry. The platform worktree was clean before planning artifacts.
- Locked scope: Ticket 047 delivers acquisition evidence and a build map only.
  Rust rules, provider schema, cartridge release, admission, and publication
  remain later tickets.
- Phase 1 is PASS.

## Phase 2 — Design

- Authenticated baseline:
  - the original release is
    `/home/cpeppers/Projects/omarchygs_usurper/upstream/v0.20e/usurp020e.zip`,
    3,323,989 bytes, SHA-256
    `30ec0371d4a657d7bd406a620aa439594e54e0367c45d3af10aada1e3014b9f7`;
  - the release's `Source20e.zip` is 2,085,599 bytes, SHA-256
    `0db8fbab0b5f046cce9628ef630cb945911f53654b9949e6844ece77f55128ba`;
  - the publisher-linked source is the parentless Git commit
    `6d4a100bb271c1d28a752c3e1514ba7b11c14fe2`, tree
    `51624a9b0d259ac762b4c3eb5fb0672b1226923b`, titled “Original,
    unmodified source”; and
  - archive members were rejected for traversal, links, and non-regular
    entries before extraction. The source clone is detached at the exact
    commit and clean.
- Canonical-source resolution: `Source20e.zip` contains separate game and
  editor ZIPs. The outer 132-file `USURPER_OPEN` game tree matches the Git
  commit byte-for-byte except for its extra nested directory. The nested
  131-file game copy differs in `BANK.PAS`, `DARKC.PAS`, `INVENT.PAS`,
  `LOVERS.PAS`, `RELATION.PAS`, and `USURPER.PAS`, and lacks `CHESTLO.PAS`.
  It is retained as provenance evidence but is not a port input. The nested
  67-file editor tree matches the Git commit exactly.
- Corpus and license classification:
  - the commit contains 181 Pascal, three assembly, two include, five object,
    one TPU, seven documentation/archive/support files, and `.gitignore`. The game tree has
    146,171 Pascal/include/assembly lines; the editor has 55,272;
  - 104 game and 52 editor Pascal/include/assembly files carry the Usurper
    GPL-2.0-or-later header;
  - 21 game and nine editor source/assembly units do not carry that notice.
    They include DDPlus/serial/overlay/network infrastructure, Borland help
    viewer material, SWAG-derived utilities, separately credited routines,
    and public-domain snippets. They remain `unresolved/reimplement`, even
    when the distribution-level `COPYING` file is present;
  - `USUTEXT.DAT` is a tagged container with 18 ANSI/ASCII picture pairs and
    an end marker. Historical notes credit more than one artist but provide no
    per-asset grant, so the art is reference-only until cleared or replaced;
  - DOS executables, overlays, objects, TPU, conversion tools, DD setup, and
    generated binary record files are execution/reference evidence only, not
    Rust port inputs; and
  - a logic-level Rust derivative is planned as GPL-2.0-or-later. The current
    OmarchyGS Provider SDK preview expressly supplies no public copyright
    grant, so external/public distribution also requires an explicit
    compatible OmarchyGS toolkit license from its owner.
- Original execution and ownership flow:
  1. `USURPER.PAS` initializes DDPlus/BBS identity, configuration, paths,
     clock and global RNG; detects a missing/stale `DATE.DAT`; excludes active
     nodes with `ONLINERS.DAT`/`MAINT.FLG`; and runs maintenance before entry.
  2. `USERHUNC.PAS::User_Search` associates one BBS real name with one mortal
     record or creates a character, enforcing alias uniqueness and selecting
     sex, ten races, and eleven classes before deriving class/race stats and
     randomized physical traits.
  3. `GAMEC.PAS::Game` records online presence, reads mail/status/news, and
     dispatches the Main Street to dungeon, status, healer, level masters,
     shops, bank, inn, castle, home, relations, social/combat, and sleep paths.
  4. `DUNGEONC.PAS::Dungeons` chooses the accessible level, consumes one daily
     fight, selects an event or one of the level's ten monsters, and delegates
     combat to `PLVSMON.PAS::Player_vs_Monsters`.
  5. Combat uses `VARIOUS.PAS` attack/defence/hit helpers plus class, race,
     spell, inventory, poison, disease, retreat, monster-mode, reward, death,
     and resurrection branches. `VARIOUS2.PAS` and `LEVMAST.PAS` decide and
     apply level raises.
  6. `MAINT.PAS::Maintenance` advances the world in a fixed order: create the
     date/maintenance fence, reset the bank safe and public talk, roll news,
     calculate town control, trim event logs, update every player and tax,
     announce town control, repair bounties and royal guards, reset king
     actions, age quests/drinks/relations/children, maintain gods, then run
     `NPCMAINT.PAS` last before clearing online and maintenance state.
- Authoritative legacy state: typed binary files hold users/NPCs, monsters,
  levels, mail, items, armor/weapons, bank safe, bounties, guards, king,
  market/chests/drinks, quests, relations, children, gods, news/chat/history,
  and online/maintenance fences. `INIT.PAS::UserRec` combines identity,
  progression, money, daily counters, equipment, health/disease, alignment,
  teams/kingdom, social/marriage/children, class/race/spells/skills, and
  resurrection/robbery state. The editor reset units generate the canonical
  catalog order and require NPC generation to run after items and monsters.
- Fidelity contract:
  - preserve explicit Borland scalar widths (`byte`, `word`, 16-bit
    `integer`, 32-bit `longint`), enum ordinals, integer division, range
    boundaries, case reachability, call/RNG order, default configuration,
    record-transition order, maintenance order, and CP437-visible text;
  - inject a clock and random source into pure reducers. The 1,037
    `Random(...)` calls and `Randomize` seed behavior require a recorded RNG
    tape and, before exact-RNG parity is claimed, a Borland-compatible oracle;
  - convert binary records semantically into forward-only PostgreSQL rows.
    Do not make native record padding, file locks, DOS paths, overlays, serial
    I/O, or undefined memory behavior part of the compatibility contract;
  - preserve documented oddities as named compatibility cases until evidence
    supports a correction. Examples include unreachable `case` arms after
    `Random(n)`, `Random(1)+1`, and historical configuration-line reuse; and
  - do not preserve crashes, corruption, unsafe path access, credential or
    BBS real-name disclosure, unbounded arithmetic, or stale-node behavior.
- Rust architecture and data flow:
  1. A separate `omarchygs_usurper` workspace owns GPL model types, explicit
     legacy arithmetic, pure rule reducers, canonical seed tables, provider
     persistence/runtime, compatibility fixtures, and cartridge source.
  2. Trusted QML emits only signed cartridge actions through authenticated
     OmarchyGS APIs. OmarchyGS owns account/persona identity, catalog/session
     envelopes, admission, idempotency, revision fencing, acquisition, and
     rendering. The provider receives only the opaque pairwise subject,
     platform session, exact game/release identity, and bounded command.
  3. The Usurper provider is the sole authority for realm/player state, clock
     cutover, random evolution, rules revision, and public view. Its separate
     PostgreSQL database stores realm/player/domain tables, RNG provenance,
     daily-run receipts, operation receipts, and outbox facts.
  4. Pure reducers accept `(state, command, clock, rng)` and return the exact
     next state, emitted facts, consumed RNG trace, and presentation view.
     Storage locks one realm/player aggregate, resolves exact replay before
     current revision, persists state/RNG/receipt/outbox atomically, and never
     holds a platform transaction across provider I/O.
  5. A data-only Core cartridge initially supplies terminal, status, meter,
     grid, and button screens. Original ANSI/ASCII remains excluded until
     provenance is cleared; CP437 text is converted to bounded Unicode while
     retaining a source-byte fixture.
- Current seam gaps and compatibility behavior:
  - `provider-starter::ProviderGame` is suitable for a solo proof because it
    supplies deterministic launch/command/view/event rules and durable
    protocol receipts, but its 32 KiB per-session state and database-free
    trait cannot implement the eventual shared king/market/NPC/social realm.
    The multi-user milestone therefore requires a separately reviewed
    realm-transaction seam or a conforming Usurper-owned protocol runtime;
  - provider launch currently supplies only `player_count`, and Cartridge v1
    has no trusted text-input node. Race/class selection fits buttons, but a
    faithful player-selected alias requires a bounded trusted input
    capability or an explicit temporary test alias. The first implementation
    may not silently treat this gap as original behavior;
  - the first slice remains one player and one long-lived provider session,
    with no platform schema/route change or production registration; and
  - provider-toolkit licensing, player alias input, mature-content defaults,
    and art replacement/clearance are explicit pre-publication decisions.
- Build milestones:
  0. Baseline harness: tracked provenance, Pascal-symbol inventory, explicit
     legacy scalar helpers, injected clock/RNG, and source-linked fixtures.
  1. One BBS day: deterministic fixture/new character; race/class setup;
     Main Street/status; dungeon level selection; one monster encounter;
     attack/retreat/death/reward; healer; level eligibility/raise; sleep;
     atomic next-day maintenance; re-entry with reset fights and mail/news.
  2. Equipment economy: canonical items, inventory, shops, bank, chest,
     haggling, poison, spells, and complete class/race combat effects.
  3. Persistent town: shared NPCs, news/mail, wanted list, market, quests,
     team/town control, king/castle, and maintenance ordering.
  4. Social world: PvP, challenges, online/trade, relations, love/marriage,
     children, prison, gods/immortality, and remaining locations/events.
  5. Release: cleared/redrawn presentation, full conformance/sidecar/recovery,
     reviewed registration, signed GPL source/release provenance, and only
     then optional marketplace/server admission.
- First-playable one-day invariant: from one fixed seed, clock instant,
  configuration, character fixture, and command list, two clean runs must
  produce byte-identical state/view/event/RNG traces. A repeated command must
  replay the persisted result; a stale revision must not consume RNG; one
  maintenance receipt per realm day must reset daily counters exactly once;
  and restart before/after combat or maintenance must recover the same result.
- Exact file manifest:

  | Path | Purpose |
  |---|---|
  | Adjacent `omarchygs_usurper/.gitignore` and `README.md` | Keep upstream/build/secrets out of the future game repository and state the docs-only starting boundary. |
  | Adjacent `omarchygs_usurper/provenance/v0.20e.sha256` | Track exact acquired archive/release hashes without tracking upstream bytes. |
  | Adjacent `omarchygs_usurper/docs/UPSTREAM_PROVENANCE.md` | Record origins, archive topology, canonical-source decision, corpus and license classes. |
  | Adjacent `omarchygs_usurper/docs/RUST_PORT_MAP.md` | Durable Pascal-to-Rust domain, state, flow, fidelity, milestone, and test map. |
  | `docs/planning/pipeline/active/*usurper*`, ticket, AAR, ticket index | Platform-side lifecycle evidence and architecture decision record. |

  No Cargo workspace, Rust source, SQL migration, cartridge, platform route,
  platform migration, provider registration, or upstream byte is in scope.
- Requirement-to-evidence regression plan:

  | Requirement | Evidence |
  |---|---|
  | REQ-001 | Canonical origin/commit readback, complete SHA manifest, archive safety checks, exact local paths, detached clean clone, and platform status. |
  | REQ-002 | Release/source type counts, file-header scan, unmarked-file inventory, bundled component and `USUTEXT.DAT` review, explicit unresolved markers. |
  | REQ-003 | Direct Pascal entry/user/game/dungeon/combat/level/maintenance/editor-reset review, record/file tables, RNG count, and call-order map. |
  | REQ-004 | Provider/cartridge ownership diagram, current-seam gap analysis, deterministic reducer/storage design, one-day invariant, and milestone/evidence matrix. |
  | REQ-005 | Platform diff proves documentation-only scope; route/migration/Cargo inventories unchanged; adjacent workspace status proves no Rust implementation. |
- Security/privacy/concurrency/recovery: never transmit BBS real names or
  platform persona IDs; bound all text/actions/state; lock daily maintenance
  and player mutations under one provider-owned realm domain; persist replay
  before current-policy/revision denial; persist RNG inputs/results with the
  same operation; make clock cutover explicit and timezone-pinned; keep
  provider and platform databases/processes/keys/backups separate; render no
  upstream executable or art bytes; and roll back a milestone by removing its
  provider release without converting platform state.
- Material alternatives rejected: vendoring source into the platform violates
  the separate-game boundary; line-translating DDPlus/Borland/SWAG units copies
  irrelevant and unresolved infrastructure; importing typed binary records as
  the live database preserves layout hazards; using host randomness or wall
  clock makes retries nondeterministic; placing the full realm in the starter's
  32 KiB session state blocks shared-world correctness; copying ANSI art before
  clearance overstates rights; and widening platform admission during the port
  would mix game construction with trust onboarding.
- CodeGraph design evidence traced the platform's `ProviderGame` rule seam,
  Relay Forge implementation, broker/runtime ownership, pairwise subject,
  fixed launch payload, session-cartridge action validation, and trusted
  presentation vocabulary. Its blast-radius evidence confirms the provider
  starter, provider broker, server provider bridge, cartridge renderer, and
  QML host as the relevant later integration surfaces. Pascal, DOS archives,
  shell inventories, documents, binary records, and assets were inspected
  directly because the platform CodeGraph does not index the adjacent corpus.
- Phase 2 is PASS.

## Phase 3 — Implement

- Built the separate `/home/cpeppers/Projects/omarchygs_usurper` Git workspace
  with upstream/build/secret state ignored. The actual original corpus remains
  locally available beneath ignored `upstream/v0.20e`; no upstream byte is
  tracked by either repository.
- Added a tracked twelve-artifact SHA-256 manifest covering the release,
  nested source archives, original game/editor binaries and overlays,
  presentation data, license, samples, and DD setup evidence.
- Added `docs/UPSTREAM_PROVENANCE.md` with canonical origins, exact commit/tree,
  safe archive topology, duplicate-source resolution, source/release counts,
  license/reuse classes, unmarked-file inventories, `USUTEXT.DAT` format and
  contents, and reproducible verification commands.
- Added `docs/RUST_PORT_MAP.md` with the Pascal flow/domain/state topology,
  explicit legacy-semantics policy, separate Rust workspace architecture,
  OmarchyGS/provider authority flow, current integration gaps, cartridge
  screens, five build milestones, one-day vertical slice, and regression
  matrix.
- Initialized the adjacent repository but did not add, commit, publish, or
  configure a remote. No Cargo file, Rust source, migration, cartridge,
  provider registration, or game admission was created.
- Focused check: `sha256sum -c provenance/v0.20e.sha256` verified all twelve
  tracked identities against the acquired bytes.
- Deviations: none from the locked documentation-only manifest.
- Phase 3 is PASS.

## Phase 3.5 — Inspect ledger

| # | Lens | Finding | Severity | Disposition |
|---|---|---|---|---|
| 1 | Provenance correctness | The release source archive contains a canonical top-level game tree and a stale nested tree with six changed files and one missing file; selecting by directory name alone would port the wrong baseline. | material baseline risk | Resolved: the publisher-linked root commit and its exact tree object are authoritative; byte comparison admits only the matching outer files and labels the nested copy reference-only. |
| 2 | License/reuse | Distribution-level GPL text coexists with 30 unmarked source/assembly files, separately credited DDPlus/Borland/SWAG-era material, binary-only objects, and artist-credited ANSI without per-asset grants. | public-release blocker | Resolved for this slice: each class is explicit; port logic uses GPL-marked game/editor source; infrastructure is reimplemented; binaries and art stay reference-only pending clearance. |
| 3 | Architecture/game state | The current generic starter's 32 KiB session state and database-free rules trait cannot safely own a multi-session shared Usurper realm. | later-milestone architecture blocker | Resolved in the build sequence: Milestone 1 is bounded solo state; Milestone 3 requires a separately reviewed shared-realm seam or an independently conforming Usurper runtime. No authority widening is hidden in this ticket. |
| 4 | Presentation fidelity | Cartridge v1 has trusted Button/Grid inputs but no bounded text input, while original character creation asks for a unique alias. | first-playable parity gap | Resolved honestly: race/class fit the current vocabulary; Milestone 1 uses a labeled test alias unless a separate platform capability is approved, and cannot claim alias-flow parity before then. |
| 5 | Documentation quality | Initial adjacent Markdown/checksum files ended with an extra blank line, and the support-file count ambiguously included `.gitignore`. | low | Fixed; no-index whitespace checks pass and the inventory now reports seven support files plus `.gitignore`. |

- Fresh CodeGraph inspection traced the final map against the provider starter,
  provider session/subject models, session-cartridge action admission, server
  error translation, and trusted presentation boundary. It corroborated the
  separate subject/revision ownership and the lack of a generic shared-realm or
  text-entry seam. CodeGraph's unrelated account-session match was discarded;
  direct review remains authoritative for the Pascal/DOS corpus and adjacent
  documentation.
- The matching inspect receipt is current for pipeline
  `376a6f08-d054-47ec-ac17-70ad4fa36dd7` and gated state
  `35b16035e4f2794eec4627a5508229f696ec88bc853ec6163f2d17cf9070d77d`.
- All findings within Ticket 047 are resolved or explicitly assigned to a
  later milestone prerequisite. Phase 3.5 is PASS.

## Phase 4 — Validate

- Acquisition and inventory evidence passed:
  - `sha256sum -c provenance/v0.20e.sha256` verified all twelve tracked
    identities in the adjacent workspace;
  - archive-member safety checks rejected path traversal, links, and
    non-regular members before extraction;
  - the 132 canonical game files and 67 editor files compared exactly with the
    pinned Git tree, while the divergent nested 131-file copy remained
    reference-only;
  - the detached source clone remained at parentless commit
    `6d4a100bb271c1d28a752c3e1514ba7b11c14fe2`, tree
    `51624a9b0d259ac762b4c3eb5fb0672b1226923b`, with no tracked changes;
  - source/artifact counts, tagged `USUTEXT.DAT` inventory, license-header
    coverage, and the 1,037-call random inventory matched the map; and
  - adjacent-workspace assertions proved there is no Cargo manifest, Rust
    source, SQL migration, cartridge package, staged file, commit, or remote.
- Documentation evidence passed `git diff --check`, no-index whitespace review
  in the adjacent repository, and `scripts/check-pipeline.sh`.
- `bin/gate.sh --diff` completed all 24 local stages and printed `GATE GREEN
  [diff]`. The worktree-bound receipt hash is
  `35b16035e4f2794eec4627a5508229f696ec88bc853ec6163f2d17cf9070d77d`.
  Phase 5 knowledge/archive edits intentionally require a final matching gate.
- No checks were skipped and there were no pre-existing failures. Phase 4 is
  PASS.

## Phase 5 — Complete

- Acceptance-criteria audit:

  | Requirement | Result | Completion evidence |
  |---|---|---|
  | REQ-001 | PASS | The original release path, canonical download origin, exact 3,323,989-byte size and SHA-256, source/archive identities, parentless commit/tree, detached-clean-clone state, and safe extraction checks are recorded and reproducible outside the platform repository. |
  | REQ-002 | PASS | The provenance report classifies source, assembly/includes, binaries/overlays/objects, generated records, samples/setup archives, documentation, `USUTEXT.DAT` text/art pairs, GPL-marked code, and explicitly unresolved bundled units/assets without claiming blanket clearance. |
  | REQ-003 | PASS | The map traces startup, identity/create flow, Main Street dispatch, dungeon/combat/leveling, authoritative records/files, 1,037 random calls, clock/date fences, fixed daily maintenance, terminal tags, BBS/DOS integration, and editor seed/reset order. |
  | REQ-004 | PASS | The map defines explicit legacy arithmetic, deterministic reducers, injected clock/RNG plus traces, independent provider PostgreSQL authority, platform/provider/cartridge flow, present SDK/input gaps, a one-day vertical slice, five later milestones, and a requirement/evidence matrix. |
  | REQ-005 | PASS | Platform status contains planning/knowledge/OpenWiki lifecycle files only; adjacent status contains only README/provenance/map files, while upstream is ignored. No Cargo/Rust/SQL/cartridge/admission/route/publication bytes were created, staged, committed, or pushed. |

- Docs: OpenWiki update run `3099e4c2-25ef-4d7d-bd47-bc3901eb3df0`
  completed. Its no-page-change disposition is intentional: Ticket 047 adds no
  current platform runtime fact, and adding the future Usurper design to the
  generated system wiki would misstate roadmap intent as implemented behavior.
  The completion receipt records gated state
  `bd0fe969fbd7cd2e0b8a6f0bf622f0a0aeeabc81ac42c9000b9de939d783cbe9`.
- AAR: submitted with three captured failures, three prevention rules, and one
  architecture decision; all seven IDs were appended to the knowledge
  register.
- Final post-OpenWiki gate: one intermediate full run observed a transient
  existing sidecar callback-test timeout at stage 19a. The exact owning
  `scripts/test-provider-sidecar.sh` drill then passed without a code change,
  and a complete canonical rerun passed all 24 stages and printed `GATE GREEN
  [diff]`. The gate and OpenWiki receipts both match gated state
  `bd0fe969fbd7cd2e0b8a6f0bf622f0a0aeeabc81ac42c9000b9de939d783cbe9`.
- Archive: Ticket 047 moved to `tickets/closed`; its spec and notes moved to
  `pipeline/completed`; no active pipeline remains. Phase 5 is PASS.

## Defect and lesson ledger

| # | What happened | Root cause | Fix | Prevention |
|---|---|---|---|---|
| 1 | The release carried a second nested game-source tree that looked plausible but did not match the publisher-linked baseline. | Historical archive assembly retained a divergent copy without an explicit version marker. | Pin the parentless publisher-linked Git tree and admit only byte-matching outer files; retain the nested tree as reference evidence. | `PR-omarchy-gaming-system-authenticate-duplicate-upstream-trees-001` |
| 2 | Repository-level GPL evidence did not establish per-file or per-asset provenance for all bundled material. | The DOS-era distribution mixed primary game code, third-party infrastructure, binary objects, generated data, and credited art. | Separate reuse classes; translate GPL-marked logic, reimplement infrastructure, and exclude binaries/art until cleared. | `PR-omarchy-gaming-system-classify-bundled-corpus-rights-by-artifact-001` |
| 3 | The generic provider starter looked like the natural port base but cannot represent the eventual shared Usurper realm. | Its deliberately narrow rules seam owns only bounded per-session state and no provider database transaction context. | Bound Milestone 1 to a solo proof and make a reviewed shared-realm seam or independent conforming runtime a prerequisite for Milestone 3. | `PR-omarchy-gaming-system-prove-provider-state-topology-fit-001` |

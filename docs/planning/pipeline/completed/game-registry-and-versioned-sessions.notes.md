---
title: Game registry and versioned sessions — notes
pipeline_id: 191a6334-576a-4573-844f-629a365ed8b2
---

# Game registry and versioned sessions — completed notes

Chronological evidence and decisions. If a test or command did not run, these
notes must not claim it passed.

## Phase 1 — Plan

- User directive: continue the ordered five-ticket set. Ticket 011 is archived
  with clean OpenWiki claims and a green final diff gate, so Ticket 012 is the
  sole active pipeline.
- Bulletin recall: GitHub still has no `main` branch and remote CI remains
  unconfirmed. This warning does not block local work and no delivery action is
  authorized.
- Knowledge recall: game/account identity must remain persona-scoped; public
  cursors cannot leak unrelated activity; long-lived transport limits and
  authority are already handled by the sync boundary; canonical gate evidence
  must include untracked files and the real PostgreSQL/API/QML path.
- Product and architecture recall: the server owns game state, time, randomness,
  and permissions; compiled Rust games must be deterministic and database-free;
  transactions will eventually update snapshots/revisions/events together;
  REST remains durable truth and WebSockets only wake clients.
- Nearest pipeline: Ticket 011 added `game_session_changed` as an explicitly
  reserved forward extension point, persona-local retained recovery, and a
  transaction-coupled append API. Ticket 012 must reuse it without adding game
  state to sync or socket payloads.
- Smallest honest slice: a reusable compiled registry, an intentionally empty
  production catalog until a real game exists, durable exact-version session
  persistence, trusted internal initialization with an injected test game, and
  participant-private read APIs. Public creation remains with the later
  challenge workflow.

## Phase 2 — Design

- Runtime boundary: add a workspace library crate named
  `omarchy-game-runtime`. It owns database-free `GameDefinition`, validated
  `GameManifest`, immutable `GameRegistry`, deterministic exact-version lookup,
  and bounded initial-state creation. The server owns authentication,
  persistence, transactions, sync invalidation, and transport DTOs. Production
  constructs an empty registry until the original game pipeline supplies a
  compiled definition; router tests inject compiled fixtures.
- Manifest contract: `key` is trimmed canonical lowercase ASCII, 3–32 bytes,
  starts alphanumeric, and otherwise permits alphanumerics, `_`, and `-`;
  `version` is positive; `display_name` is 1–64 control-free characters; human
  player bounds satisfy `1 <= min <= max <= 8`. Registry construction rejects
  invalid or duplicate `(key, version)` entries and stores definitions in a
  `BTreeMap`, making catalog and version enumeration stable. Exact lookup never
  falls forward to a newer version.
- Initialization contract: a definition receives only a human-player count and
  returns deterministic JSON state. The registry verifies the count against
  the pinned manifest, requires an object snapshot, and caps serialized state
  at 64 KiB before persistence. Game code receives no pool, session token,
  account ID, clock, network, or random source in this slice.
- Persistence: forward migration `0010` adds `game_sessions` with immutable
  canonical `game_key`, positive `game_version`, nonnegative `revision` default
  zero, `active` status, JSON-object state, and timestamps. The runtime applies
  the 64 KiB serialized-state bound before persistence. A
  `game_session_participants` table gives each unique persona one zero-based
  seat, caps seats below eight, cascades from the session, restricts persona
  deletion, and indexes `(persona_id, session_id)` for owner inventory. The
  database intentionally cannot foreign-key a compiled registry.
- Trusted creation flow: `games::create_session` is crate-private and accepts an
  existing PostgreSQL transaction, immutable registry, exact key/version, and
  caller-ordered participant UUIDs. It validates/initializes before writes,
  rejects duplicates, locks all persona rows in canonical UUID order, inserts
  the session plus original seat order, then appends participant sync events in
  canonical UUID order. The future challenge transaction can call this without
  losing atomicity; this ticket exposes no creation route.
- Sync evolution: migration `0010` adds nullable `game_session_id` to retained
  persona events and replaces the type/payload-shape checks so exactly
  `game_session_changed` carries that UUID, exactly `conversation_changed`
  carries `conversation_id`, and social variants carry neither. The Rust event
  union and REST DTO gain `GameSession(Uuid)`; WebSocket hints remain unchanged.
- Read API: public `GET /v1/games` returns `{games: [...]}` with flat stable
  manifest rows and no private state; production currently returns an empty
  array. Authenticated
  `GET /v1/personas/{persona_id}/game-sessions?limit=1..100` returns a default
  50 newest-first sessions, and
  `GET /v1/personas/{persona_id}/game-sessions/{session_id}` returns one.
  Both private routes authenticate first, owner-scope the acting persona, then
  require that persona to be a session participant. Responses contain the
  pinned key/version, state, revision, status, timestamps, seats, and public
  persona profiles only; malformed, missing, and foreign session IDs share 404.
  Private successes and all extraction/domain failures use `Cache-Control:
  no-store`.
- Compatibility: existing router constructors keep an empty registry default;
  a new internal constructor injects both `SyncHub` and `GameRegistry` for main
  and integration tests. No existing endpoint changes. Stored sessions are read
  directly from PostgreSQL and never relabeled through today's registry; Ticket
  013 must require the exact compiled version before accepting a command.
- Security/privacy: public catalog data is compile-time metadata. Private
  session reads reveal participant profiles and state only after owner plus
  membership checks. Initial state and participant counts are bounded before
  database writes. Canonical participant locking and sync append order prevent
  reversed-input deadlocks. Errors collapse registry initialization and
  database failures to generic server errors rather than exposing internals.
- Operations: startup must fail if a future production compiled registry is
  invalid. An empty production registry is valid now and is asserted by live
  smoke; PostgreSQL integration tests inject two fixture versions and create
  sessions through the same crate-private transaction path.
- CodeGraph evidence: design exploration identified all AppState/router
  constructors, six sync event consumers, four `append_event` callers, the
  persona/session authorization boundary, and incomplete automated associations
  for `AppState`/sync mapping. It showed that the injected registry touches
  `main`, the default/test constructors, `sync_api_tests`, the event row parser,
  and the Axum response union. Direct Cargo, migration, shell, and SQLx-test
  inspection supplements unsupported or incomplete graph coverage. The design
  receipt for pipeline `191a6334-576a-4573-844f-629a365ed8b2` exists.

### File manifest

| Path | Purpose |
|---|---|
| `Cargo.toml`, `Cargo.lock` | Add the game-runtime workspace member and shared dependencies. |
| `crates/game-runtime/Cargo.toml`, `crates/game-runtime/src/lib.rs` | Own compiled manifest validation, exact-version registry, initialization bounds, and deterministic unit fixtures. |
| `migrations/0010_game_registry_and_sessions.sql` | Add durable sessions/participants and the exact game-session sync payload variant. |
| `crates/server/Cargo.toml` | Depend on the local runtime crate. |
| `crates/server/src/games.rs` | Own trusted transactional creation, participant-private inventory/detail, public projection data, and stable errors. |
| `crates/server/src/app.rs` | Inject the registry, add catalog/session routes and DTOs, map game errors, and extend the sync DTO. |
| `crates/server/src/main.rs` | Register game modules/tests and pass the production registry into AppState. |
| `crates/server/src/sync.rs` | Persist and decode the minimal `game_session_changed` event. |
| `crates/server/src/game_api_tests.rs` | Prove catalog, registry injection, atomic version-pinned creation, participant privacy, stable reads, and sync event shape against PostgreSQL. |
| `scripts/dev.sh` | Assert the honest empty production catalog in the live PostgreSQL/server/QML path. |
| `docs/api.md`, `README.md`, `docs/architecture/system-overview.md`, `docs/planning/ROADMAP.md` | Document the registry/session foundation, current empty catalog, privacy/version rules, and implemented roadmap state. |
| Ticket/spec/notes/AAR/knowledge/OpenWiki | Preserve workflow evidence, durable lessons, and generated engineering documentation. |

### Regression plan

| Requirement | Evidence |
|---|---|
| REQ-001 | Game-runtime unit tests cover canonical validation, player bounds, duplicate rejection, exact multi-version lookup, stable order, and bounded object initialization. |
| REQ-002 | An ordinary router test proves the empty production catalog; an injected-registry router test proves exact public metadata and stable ordering; live smoke repeats the production contract. |
| REQ-003 | PostgreSQL creation tests prove exact-version state/revision/status/seats, deterministic repeated initialization, canonical participant locking, and rollback/no-row behavior for unknown version, duplicate/missing participants, invalid counts, or bad initialization. |
| REQ-004 | Multi-account router tests prove bounded inventory/detail, stable order, exact public profiles, state visibility to participants, owner checks, no-store, and identical absent/foreign responses. |
| REQ-005 | A session created under fixture version 1 remains version 1 with its original snapshot after a registry containing version 2 is used for reads; no lookup/substitution occurs. |
| REQ-006 | Creation tests inspect both participant feeds for one exact minimal game-session event, another persona's empty feed, and rollback silence. |
| REQ-007 | All unit and migrated PostgreSQL tests plus the expanded public-catalog live smoke and unchanged QML connector run in `bin/gate.sh --diff`. |

### Alternatives rejected

- Registering a placeholder production game would make an unplayable manifest
  look like shipped product. An empty catalog plus injected deterministic
  fixtures tests the architecture without lying to clients.
- A temporary public session-creation endpoint would bypass the later challenge
  and social authorization transaction. The reusable crate-private transaction
  primitive avoids that compatibility debt.
- Looking up only the newest game version when reading or commanding an old
  session would silently reinterpret durable state. Sessions pin the exact
  version and future commands must resolve it exactly.
- Putting snapshots or participant data in persona sync events or WebSockets
  would duplicate the participant authorization boundary. The minimal session
  UUID invalidation sends clients back to authenticated REST.
- Letting a game definition query PostgreSQL or receive wall-clock/randomness
  during initialization would make deterministic replay and isolated testing
  impossible. Those capabilities remain explicit server-owned inputs for later
  command design.

## Phase 3 — Implement

- Built: a database-free `omarchy-game-runtime` workspace crate with validated
  exact-version manifests, deterministic stable catalog order, bounded object
  initialization, and an honest empty registry; forward migration `0010` for
  version-pinned revision-zero sessions, ordered persona participants, and
  shaped game-session sync invalidations; crate-private transactional creation;
  participant-scoped list/detail REST; public catalog; registry injection;
  sync union evolution; focused unit/PostgreSQL tests; live empty-catalog smoke;
  and hand-maintained API/runtime documentation.
- Focused checks: `cargo check --workspace --all-targets` passed after the new
  workspace member and SQLx JSON feature resolved; `cargo test --workspace
  --all-targets` passed 29 local tests with 30 PostgreSQL tests intentionally
  ignored; both new real PostgreSQL game tests passed independently; `cargo
  clippy --workspace --all-targets -- -D warnings` passed; `bash -n
  scripts/dev.sh` and `git diff --check` passed; and `./scripts/dev.sh
  --smoke-test` completed the full PostgreSQL/server/public-catalog/QML path
  with only the known non-fatal headless EGL warnings.
- Test correction: the first runtime object-shape test panicked inside its own
  fixture because the fixture attempted object-key indexing before returning an
  intentional JSON array. The fixture now annotates `human_players` only when
  its configured state is an object, allowing the registry—not the test
  scaffold—to reject the array. The rerun passed all runtime tests.
- Deviation: the crate-private creation path has no production caller until the
  later challenge pipeline, so its narrow constants/function/error variants use
  `cfg_attr(not(test), allow(dead_code))`. The implementation remains compiled
  in production, fully exercised through the real database tests, and is not
  exposed through a temporary transport route.

## Phase 3.5 — Inspect ledger

| # | Lens | Finding | Severity | Disposition |
|---|---|---|---|---|
| 1 | Game correctness, privacy, concurrency, and state | Fresh CodeGraph exploration and the frozen security scan found no reachable gap in exact-version resolution, deterministic initialization, canonical participant locking, participant-scoped reads, transaction-coupled sync, or minimal WebSocket hints. | none | PASS; inspection receipt matches the current gated state. |
| 2 | Secret-provider coverage | The shared Codex Stop/delivery scanner omitted OpenAI project and service-account key families, so a matching credential could pass both entrypoints before an authorized Git delivery. | low | FIXED in the shared scanner with `sk-proj-`/`sk-svcacct-` regression fixtures; the original trigger and alternate family are rejected. |
| 3 | Filename option injection | A Git-derived root filename such as `-q` reached `grep` without an option terminator and could be treated as an option rather than scanned. | low | FIXED by passing `--` before the filename and adding an adversarial filename regression fixture; symlink exclusion and clean input behavior remain intact. |
| 4 | Codex transitive hook trust | The official Codex docs require trust for the exact hook definition, and current OpenAI source hashes the normalized config-derived handler identity rather than bytes of scripts referenced by its command. A later script-only change therefore does not independently invalidate persisted hook trust. | hardening follow-up | Resolved as an upstream trust-model limitation, not a Ticket 012 application defect. Keep the repository warning that hooks are guardrails and the independently executed, worktree-bound gate receipt is delivery proof; open a dedicated integrity-hardening slice after the ordered ticket set rather than broadening this game ticket. Sources: `https://learn.chatgpt.com/docs/hooks` and `https://github.com/openai/codex/blob/main/codex-rs/hooks/src/engine/discovery.rs`. |

- Codex Security scan `1d9d669c-9a58-4a57-8314-78d4a72abd1a`
  completed over all 68 frozen review items. Its game surface was clean; two
  low scanner findings were remediated after explicit user approval. The fix
  report records the exact invariant, reproductions, controls, and green fast
  gate under the scan artifact directory.
- Patch challenge: every direct scanner caller was re-inspected. The shared
  hook remains the single boundary for `.codex/hooks.json` and `bin/gate.sh`;
  no sibling `grep` filename sink exists. Synthetic project/service-account
  keys and a secret in `-q` are blocked, while short near-miss text, a clean
  invocation, and the existing symlink exclusion remain accepted.
- Verification: shell syntax, hook self-tests, diff whitespace, and
  `bin/gate.sh --fast` all passed. The fast gate also reran rustfmt, Clippy,
  workspace tests, rustdoc, Compose validation, pipeline structure, and the
  real changed-file scanner.

## Phase 4 — Validate

- `bin/gate.sh --diff` completed green and wrote a worktree-bound delivery
  receipt. All ten static/local checks passed, including the strengthened
  secret scanner and hook self-tests; all 30 PostgreSQL integration tests
  passed, including both Ticket 012 game cases; and the live PostgreSQL + Rust
  API + visible QML smoke passed with only the known non-fatal headless EGL
  warnings.
- The receipt was created only after the real database and live empty-catalog
  path passed. No Cargo process was run concurrently or terminated.

## Phase 5 — Complete

- Acceptance-criteria audit:

  | Requirement | Final evidence | Result |
  |---|---|---|
  | REQ-001 | The three `omarchy-game-runtime` unit tests prove canonical manifest validation, duplicate rejection, stable catalog order, exact-version resolution, human-player bounds, deterministic initialization, object shape, and the 64 KiB serialized-state cap. | Satisfied |
  | REQ-002 | `public_catalog_is_stable_and_production_is_honestly_empty` proves stable injected metadata and the empty production response; the live smoke independently requires `{"games":[]}`. | Satisfied |
  | REQ-003 | `creation_is_atomic_version_pinned_and_syncs_every_participant` proves exact key/version, deterministic revision-zero active state, ordered seats, canonical participant locking, and rollback/no-row behavior for unavailable versions, duplicate/missing participants, initialization failure, and caller rollback. | Satisfied |
  | REQ-004 | `session_queries_are_bounded_participant_private_and_registry_independent` proves owner plus membership scope, newest-first bounds, public participant fields, no-store responses, and identical absent/non-participant session results. | Satisfied |
  | REQ-005 | The same query test creates version-one sessions, injects a version-two-only registry, and still reads the stored version-one identity and snapshot directly from PostgreSQL. | Satisfied |
  | REQ-006 | The creation test proves one exact `game_session_changed { game_session_id }` event for each participant, no state/account data in the feed, and rollback silence because event append shares the caller's transaction. | Satisfied |
  | REQ-007 | The final canonical diff gate ran runtime and server tests, migration `0010`, all 30 PostgreSQL cases, the live empty-catalog API path, and the unchanged visible QML health connector. | Satisfied |

- Docs: README, API, system overview, roadmap, and five affected OpenWiki pages
  distinguish the implemented registry/session foundation from future public
  creation, commands, challenges, playable rules, and game UI. OpenWiki run
  `8ea42583-30b4-4c32-b7fa-18a0a445be06` returned `complete`; all affected
  Claims had zero stale or unresolved entries, and its receipt matches the
  current gated worktree.
- AAR: `AAR-012` submitted at effectiveness 5/5 with four captured failures,
  five prevention rules, one architecture decision, and matching knowledge-
  register entries. The Codex transitive hook-trust limitation remains an
  explicit dedicated hardening follow-up rather than being silently treated as
  fixed inside this game slice.
- Archive: TICKET-012 closed and the spec/notes pair moved together to
  `completed/`. Delivery remains separate and unauthorized: no commit, push,
  pull request, or publication was performed.

## Defect and lesson ledger

| # | What happened | Root cause | Fix | Prevention |
|---|---|---|---|---|
| 1 | The runtime's negative object-shape test panicked inside its fixture before the registry could reject the intended array state. | The fixture unconditionally mutated the value as an object while constructing invalid input. | Mutate the synthetic player field only when the fixture state is an object, then let malformed shapes reach the registry boundary. | Negative fixtures must remain inert enough for the system under test—not the scaffold—to produce the expected rejection. |
| 2 | OpenAI project/service-account credentials could pass the shared Codex Stop and delivery scanner. | The migrated provider-prefix expression did not include the active Codex project's own high-signal key families. | Add `sk-proj-` and `sk-svcacct-` families plus blocking and near-miss regression fixtures. | Reconcile secret families with active repository integrations and execute the actual shared hook in tests. |
| 3 | A changed root file named `-q` could be parsed as a `grep` option instead of scanned. | Git-derived filenames reached the command without an option terminator. | Pass `--` before every filename and prove the adversarial valid Git pathname is blocked. | Terminate options before repository-derived paths even when earlier code checked that the path is a regular file. |
| 4 | Codex hook trust does not transitively hash scripts referenced by an unchanged command definition. | Current Codex hashes the normalized hook configuration identity rather than referenced script bytes. | No broad integrity redesign was folded into Ticket 012; the limitation is documented and the independent worktree gate remains delivery proof. | Treat persisted hook trust as transitive local-code trust after branch or script changes and design a dedicated digest-pinned hardening slice. |

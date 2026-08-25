---
title: Gaming system rebrand — notes
pipeline_id: a1d313c9-e799-43f3-ab79-cd6e544c6308
---

# Gaming system rebrand — running notes

Chronological evidence and decisions. If a test or command did not run, these
notes must not claim it passed.

## Phase 1 — Plan

- User decision: move away from BBS as the primary identity, rename the product
  as a gaming system, and leave message boards as a possible later capability
  rather than the current focus.
- External context: `thoughtlesslabs/omarchy-bbs` is an active Omarchy 4 native
  message-board plugin with public categories, replies, reactions,
  notifications, moderation, and a hosted PHP/MySQL API. Its shipped community-
  board position makes the shared name materially confusing even though this
  project's Rust/PostgreSQL social-game architecture is distinct.
- Recall: the product charter already excludes public boards and activity feeds
  from private alpha, commits to connections/inboxes/challenges/matches, and
  requires server-authoritative deterministic games. The rename makes that
  existing boundary explicit rather than changing domain architecture.
- Recall: historical decisions require a real PostgreSQL/API/QML vertical-slice
  proof, include untracked files in gates, bound password work, and preserve
  account/persona privacy and owner scoping.
- Preflight: no active pipeline existed, the warning bulletin records that
  remote `main` is still absent, PostgreSQL-sensitive work remains local, and
  CodeGraph 1.5.0 plus OpenWiki 0.3.3 are ready with the Codex-only patch.
- Naming inventory: current branding appears in the Cargo package/log target,
  health JSON, QML title/copy, `BBS_BIND_ADDRESS`, `bbs1_` session tokens,
  Compose resources, gate and tool receipts, Codex hooks, scripts, hand-written
  docs, and generated OpenWiki pages. Historical specs/AARs and registered IDs
  also contain the old name but are immutable evidence.

## Phase 2 — Design

- Architecture: the existing modular-monolith, API, identity, and game-state
  boundaries do not change. This is a cross-cutting identity migration at four
  edges: human-facing product copy, process/configuration identifiers, opaque
  session formatting, and local workflow/development resource names. Domain
  authorization and persistence remain unchanged.
- Runtime data flow: `Config::from_environment` resolves `OGS_BIND_ADDRESS`
  first, falls back to `BBS_BIND_ADDRESS`, and otherwise uses the loopback
  default. Startup uses the new Cargo/log target and health JSON reports
  `omarchy-gaming-system`; QML renders the game-system name while consuming the
  same `/health` contract. Session creation issues `ogs1_` plus the existing 32
  random bytes, while token parsing accepts both exact prefixes before hashing
  the complete presented string. Existing stored hashes therefore continue to
  resolve without schema changes.
- Persistence and local operations: no SQL migration or table name changes are
  required. Compose moves its disposable development database, role, and named
  volume to `omarchy_gaming_system`/`ogs_postgres_data`; the prior volume is not
  removed. Test and development scripts select the new resources explicitly.
- Compatibility: HTTP route versions and response shapes stay stable except
  the intentional `/health.service` value. Existing `bbs1_` credentials and
  `BBS_BIND_ADDRESS` remain accepted but are no longer emitted or preferred.
  An older binary will not understand newly issued `ogs1_` sessions, so a code
  rollback would require reauthentication or retaining the dual-prefix parser;
  this is acceptable for the unpublished pre-alpha.
- Security/privacy: the token change preserves 256 random bits, base64url
  validation, hashing of the exact bearer value, database-only token digests,
  expiry, revocation, and owner scoping. Prefix compatibility does not widen
  accepted length or alphabet. No account/persona field or authorization rule
  changes.
- CodeGraph evidence: explored `Config::from_environment`, startup tracing,
  `HealthResponse`/`health_document`, `TOKEN_PREFIX`, `generate_token`,
  `token_digest`, `create_session`, `authenticate`, their Rust callers, and
  one-hop test blast radius on the Phase 1 worktree. The runtime change is
  bounded to `config.rs`, `main.rs`, `app.rs`, `sessions.rs`, and the two API
  test modules; config/docs/shell/QML remain directly reviewed unsupported
  surfaces.

### File manifest

| Path | Purpose |
|---|---|
| `crates/server/Cargo.toml`, `Cargo.lock` | Rename the server package and derived Rust log target. |
| `crates/server/src/config.rs`, `main.rs`, `app.rs` | Introduce new local defaults and bind-variable precedence; rename startup and health identities. |
| `crates/server/src/sessions.rs`, `session_api_tests.rs`, `persona_api_tests.rs` | Issue `ogs1_`, accept the narrow legacy prefix, and update focused/integration assertions and fixtures. |
| `client/qml/Main.qml` | Present Omarchy Gaming System and game-service connection language. |
| `compose.yaml`, `.env.example`, `scripts/dev.sh`, `scripts/test-database.sh` | Move the executable development path to the new database, package, log, bind, and token names. |
| `bin/*.sh`, `scripts/*.sh`, `.codex/hooks/*.sh`, `.codex/hooks.json` | Rename internal shell namespace plus gate/pipeline-tool receipt paths and keep the enforcement self-tests aligned. |
| `AGENTS.md`, `CONSTITUTION.md`, `.agents/skills/**` | Rename the living project guide, constitution, and skill descriptions without weakening gates. |
| `README.md`, `docs/product-charter.md`, `docs/api.md`, `docs/architecture/system-overview.md`, `docs/planning/ROADMAP.md`, living planning indexes/templates/bulletins | Establish game-first positioning, current identifiers, compatibility notes, and the future-board boundary. |
| `openwiki/INSTRUCTIONS.md`, generated `openwiki/*.md` | Rename the wiki brief now and reconcile generated engineering claims in Phase 5. |
| Ticket/spec/notes/AAR/knowledge register | Preserve scope, evidence, findings, and the new architecture decision under the gaming-system namespace. |

### Regression plan

| Requirement | Evidence |
|---|---|
| REQ-001 | Direct scoped scan of living surfaces plus product-charter, roadmap, QML, workflow, and OpenWiki review; historical directories are explicitly excluded. |
| REQ-002 | Health unit assertion, session-token unit/integration assertions, renamed Cargo build/test target, QML source check, and live smoke requiring the new health/token identifiers. |
| REQ-003 | Pure config-resolution tests cover new precedence, legacy fallback, and default; token parsing tests prove exact SHA-256 compatibility for a structurally valid legacy token. |
| REQ-004 | `docker compose config --quiet`, shell syntax, pipeline structure, hook self-tests, and receipt-path assertions exercise the renamed development/enforcement namespace. |
| REQ-005 | `git diff` review confirms migrations and completed planning/AAR evidence were not mechanically rewritten; the new AAR registers only gaming-system IDs. |
| REQ-006 | `bin/gate.sh --diff` runs all Rust tests, isolated migrated PostgreSQL tests, and the live PostgreSQL/server/QML lifecycle. |

### Risk and rollback table

| Risk | Control |
|---|---|
| Existing sessions stop authenticating | Dual-prefix parser hashes the complete original bearer string; dedicated legacy test. |
| Existing bind configuration is ignored | `OGS_BIND_ADDRESS` precedence with documented `BBS_BIND_ADDRESS` fallback and pure resolver tests. |
| Existing local database appears lost | New named volume is additive; old volume is left untouched, and no destructive Docker command is run. |
| Historical evidence or migration history is falsified by global replacement | Mechanical edits are limited to living surfaces; completed pipelines, closed tickets, AARs, registered IDs, and migrations receive direct diff review. |
| Hook receipts or CodeGraph/OpenWiki evidence silently use stale paths | Rename helpers and self-test fixtures together; rerun design/inspection/completion tools after their final gated states. |
| Marketing rename leaves runtime identifiers stale | Health, Cargo, config, token, QML, scripts, receipts, and living-doc scans are separate regression checks. |

## Phase 3 — Implement

- Built: renamed the Rust package/log target, `/health.service`, QML title and
  game-service copy, Compose database/role/volume, environment example,
  development/test commands, shell-local namespaces, gate and pipeline-tool
  receipts, Codex hook description, guides, constitution, skills, product/API/
  architecture/roadmap documentation, planning indexes/templates, and the
  OpenWiki brief. New sessions now issue `ogs1_`; the parser retains exact
  `bbs1_` compatibility. `OGS_BIND_ADDRESS` now takes precedence over the
  legacy variable through a pure tested resolver. The live smoke now asserts
  the new health identity and token prefix.
- Focused checks: `cargo fmt --all` completed; `cargo check --workspace
  --all-targets` compiled the renamed runtime; `cargo test --workspace` passed
  15 fast tests with eight PostgreSQL tests intentionally ignored; Bash syntax,
  Compose configuration, pipeline structure, and all Codex hook self-tests
  passed. The full PostgreSQL and QML tiers remain Phase 4 evidence.
- Deviations: completed records and migrations were preserved as designed.
  Generated OpenWiki pages still show the prior name until their mandatory
  Phase 5 lifecycle; only `openwiki/INSTRUCTIONS.md` changed during
  implementation. No compatibility shim was added for the Cargo package,
  health-service value, or local Compose resources because the project has not
  been published; the old Docker volume remains recoverable but detached.

## Phase 3.5 — Inspect ledger

| # | Lens | Finding | Severity | Disposition |
|---|---|---|---|---|
| 1 | Scoped living-surface branding scan | The workflow ADR retained the old product name in its current context paragraph while correctly retaining its historical `AD-omarchy-bbs-*` knowledge ID. | Low | Fixed the current product sentence; preserved the historical ID. |
| 2 | Runtime and compatibility blast radius | CodeGraph traced new token issuance, dual-prefix parsing, bearer authentication, bind-variable precedence, health identity, startup, route callers, and tests. No stale BBS runtime/package/service identity or widened authorization path remained. | None | Pass; direct review retained for QML, shell, configuration, docs, generated data, and tests not structurally covered. |
| 3 | Security diff review | The 54-file security-sensitive local-patch inventory preserved token entropy/digest-only storage, expiry, revocation, owner scoping, parameterized SQL, request bounds, and workflow path validation. | None | No reportable findings; canonical report at `/tmp/codex-security-scans/omarchy_bbs/493749e2194df621640b229be4a5058fc872f30a_20260824T145834Z/report.md`. |

- Inspection result: PASS. The final CodeGraph call wrote a matching
  worktree-bound `inspect.receipt`; the scoped branding scan contains only
  intentional compatibility identifiers, the external plugin reference, and
  preserved historical knowledge IDs.

## Phase 4 — Validate

- Tests run: the canonical diff gate passed Rust formatting, Clippy, 15 fast
  tests, Rustdoc, Compose validation, shell syntax, pipeline structure, changed-
  file secret scanning, Codex hook self-tests, whitespace checks, all eight
  isolated migrated PostgreSQL integration tests, and the live PostgreSQL →
  Rust API → account/session/persona → QML smoke path.
- Gate run: `bin/gate.sh --diff` returned `GATE GREEN [diff]` and wrote a
  matching `.git/omarchy-gaming-system-gate-receipt` for state
  `b0574c41b748863ebbcc5bb45e70008080d3efd2b2950894511ac0f75662dc9b`.
- Skips or pre-existing failures: none. The fast tier's eight ignored database
  tests were all executed and passed in the dedicated PostgreSQL tier. The QML
  offscreen run emitted non-fatal libEGL DRI2 warnings and completed normally.

## Phase 5 — Complete

- Acceptance-criteria audit:
  - REQ-001 satisfied: product charter, roadmap, system overview, guides, QML,
    workflow surfaces, and generated OpenWiki now identify Omarchy Gaming
    System as game-first and keep boards as a possible later complement.
  - REQ-002 satisfied: Cargo/log/health/QML/development paths use the
    `omarchy-gaming-system`/`ogs` namespace; unit, integration, and live smoke
    evidence require newly issued `ogs1_` sessions.
  - REQ-003 satisfied: pure configuration tests prove `OGS_BIND_ADDRESS`
    precedence and legacy fallback; token unit evidence proves exact legacy
    bearer hashing under the unchanged validity predicates.
  - REQ-004 satisfied: Compose, shell syntax, pipeline checks, hook self-tests,
    renamed worktree receipts, and the final diff gate passed. No migration or
    historical evidence was rewritten, and the old Docker volume was not
    deleted.
  - REQ-005 satisfied: prior `omarchy-bbs` knowledge IDs and completed records
    remain stable; new durable IDs use `omarchy-gaming-system`.
  - REQ-006 satisfied: the final `bin/gate.sh --diff` returned `GATE GREEN
    [diff]` after OpenWiki, with all eight PostgreSQL tests and the live QML
    path green for gated state
    `8ecbd836ba017d6c10239c645fd3569956e81ce3518e9134ab9136331ccfbb06`.
- Docs: OpenWiki update run `d34c9f36-54d2-4442-9106-4c2b5b874f4d`
  returned `status: complete`; quickstart, runtime, product boundaries,
  development/validation, claims, and navigation now describe the new identity
  and its compatibility window. Hand-maintained product/API/architecture/
  operator docs were reconciled before the lifecycle.
- AAR: submitted `AAR-007` at 5/5. Registered
  `PR-omarchy-gaming-system-separate-live-identity-from-history-001` and
  `AD-omarchy-gaming-system-game-first-identity-001` in both the AAR and
  knowledge register.
- Archive: closed TICKET-007, removed it from the open queue, and archived the
  Phase 5 PASS spec/notes pair. Delivery remains unperformed and unauthorized.

## Defect and lesson ledger

| # | What happened | Root cause | Fix | Prevention |
|---|---|---|---|---|
| 1 | One current ADR sentence retained the old product name. | The first scan excluded a file carrying a preserved historical ID without separating that identifier from the file's living prose. | Renamed only the current sentence and retained the historical ID. | `PR-omarchy-gaming-system-separate-live-identity-from-history-001`. |

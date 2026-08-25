# Omarchy Gaming System Constitution

The binding development rules for Omarchy Gaming System. Codex project instructions,
skills, and hooks cite the numbered sections below. The workflow is based on
the proven Rustal pipeline, adapted to this API-first Rust server and QML
client.

## §0 — Quality gate

`bin/gate.sh` is the canonical delivery gate.

- `bin/gate.sh --fast` runs the short static loop and writes no receipt.
- `bin/gate.sh --diff` runs every current gate, including the PostgreSQL/QML
  smoke path, and writes a receipt for the exact worktree.
- `bin/gate.sh` is the full mode. At this project stage it is identical to
  `--diff`; it is intentionally a separate mode so coverage, mutation, and
  broader game/client tests can be ratcheted in later.

The current gates are:

1. `cargo fmt --all --check`
2. Clippy across the production workspace and all targets with warnings denied
3. Production-workspace tests
4. Rust documentation with warnings denied
5. Docker Compose validation
6. Bash syntax validation for project and Codex hook scripts
7. Pipeline-structure validation
8. Changed-file secret scanning
9. Codex hook self-tests
10. Whitespace validation across tracked, staged, and untracked files
11. The production Game Cartridge contract, including deterministic packing,
    hostile archive and content verification, compatibility reporting,
    database/network/credential-isolated conformance, and atomic local
    install/revocation checks
12. The production trusted Game Cartridge renderer, including authenticated
    schema/view compilation, profile and fallback bounds, private preview
    output, allowlisted QML components, keyboard/accessibility behavior, every
    fixed failure state, and software-rendered frame/RSS profile measurements
13. The production Game Cartridge SDK/release boundary, including deterministic
    SDK export, fresh-repository reproducibility, signed provenance, public-only
    verification/import, lifecycle policy, and descriptor-relative store tests
14. The isolated Game Cartridge architecture proof, including its nested
    workspace format, Clippy, tests, binary build, rustdoc, signed package,
    broker/provider exchange, privacy assertions, and trusted QML smoke path
15. PostgreSQL integration tests (DIFF/FULL only)
16. The real PostgreSQL migration → Rust API → keyboard-first QML client smoke
    path, including deterministic hostile-fixture tests plus live registration,
    password/MFA session creation, persona creation/selection, local logout,
    and health recovery (DIFF/FULL only)
17. The production remote-provider security foundation, including PostgreSQL
    registry/lifecycle races and a separately spawned TLS provider proving
    signed grants/messages, durable replay, expected revisions, bounded
    faults, outage recovery, event deduplication, and reconciliation
    (DIFF/FULL only)
18. The first-party Door Legends remote-authority pilot, built from a clean
    clone and exercised through the real broker/player API against an
    independent provider database, including replay, conflict, callback,
    projection, outage, reconciliation, lifecycle, privacy, and restore proof
    (DIFF/FULL only)

Repository-local CodeGraph and OpenWiki are workflow instruments, not CI
dependencies. `scripts/setup-pipeline-tools.sh` prepares their pinned generated
state, while committed checks validate the wiring without network access.

Cargo commands run sequentially. Never weaken a gate, delete a test, or edit a
receipt to manufacture green. Fix the source.

## §3 — Work phases

One shippable slice moves through one active pipeline:

```text
recall → plan → design → implement → inspect → validate → complete → delivery
```

- Never keep more than one spec/notes pair in
  `docs/planning/pipeline/active/`.
- Every pipeline has a numbered ticket and an AAR.
- The spec has one `status:` field. Advancing a phase means replacing that
  field with `Phase N — <Name> PASS; ready for Phase M — <Name>`.
- Acceptance criteria use EARS: each row says what the system **shall** do and
  names the evidence that will verify it.
- Work that is not ready for commitment belongs in `docs/planning/intake/`.
- Application code includes `crates/`, `client/`, `migrations/`, runtime and
  validation scripts, Cargo manifests/lockfile, Compose, CI, and gate/hook
  code. Codex project instructions and repository skills are also gated
  workflow code. Planning documents are not application code.

## §7 — Testing

Every behavior change receives meaningful automated evidence. The Phase 2
design maps every acceptance criterion to at least one test or explicit review
check. Phase 4 must run the tests; writing them is not proof.

- Rust logic receives unit or integration tests.
- Database behavior is exercised against PostgreSQL.
- QML-visible behavior receives a QML smoke or interaction test as the client
  harness grows.
- Game rules must be deterministic and testable without a UI or network.
- Pre-existing failures are recorded, not silently claimed as green.

## §10 — Product and architecture boundaries

- The OmarchyGS server is authoritative for authentication, accounts,
  personas, social state, catalog and launch policy, platform permissions,
  provider registration, the participant-private game-session envelope,
  public result and achievement policy/projections, audit, suspension, and
  durable recovery.
- A `platform_compiled` session keeps OmarchyGS as the sole authority for game
  rules, private state, turns, game time/randomness, revision, and outcome.
  An operator-registered `registered_provider` session may instead pin one
  exact immutable provider/game/rules/cartridge release as the sole durable
  authority for those game-scoped surfaces. Every session has exactly one
  authority: OmarchyGS must not retain a writable provider gameplay snapshot,
  and a provider-owned session must never fail back to compiled rules.
- Registered provider traffic is server-to-server through the authenticated
  OmarchyGS broker. Providers receive only pairwise subjects and scoped,
  expiring grants; they receive no account identity, reusable device
  credential, platform database access, or executable frontend privilege.
  Their signed results and achievement claims have no platform effect until
  OmarchyGS atomically authenticates, deduplicates, validates pinned policy,
  and records allowlisted projections plus cursor-sync invalidations.
- Accounts and personas are separate domain identities. Social and game
  surfaces expose personas, not account ownership.
- REST/JSON is the durable command/query interface. WebSockets notify clients;
  cursor-based synchronization remains the recovery source of truth.
- Transport handlers remain thin. Domain and game logic live in testable Rust
  modules without direct UI coupling.
- First-party server game rules start as compiled Rust crates. Portable Game
  Cartridges are signed inert data rendered only through bounded,
  platform-owned components. Loading publisher native/QML/JavaScript code is
  out of scope; any future executable extension runtime requires an explicit
  sandbox decision.
- Remote authority is limited to explicitly operator-enabled exact releases
  that pass the provider pipeline and lifecycle controls. External/self-service
  provider onboarding remains unauthorized until its own architecture and
  security pipeline passes.
- The QML connector is keyboard-first and consumes only public API or trusted
  render-plan contracts.

## §14 — Code conventions

- Rust must be formatted and Clippy-clean with warnings denied.
- Avoid `unwrap`, `expect`, `todo!`, and `unimplemented!` on input-reachable
  paths; return typed errors with useful context.
- New public APIs and modules carry documentation.
- SQL changes are forward-only, versioned migrations. Never edit a migration
  that may have run outside the local disposable database.
- Commands that mutate a game session are idempotent and revision-aware.
- QML must expose clear loading, offline, empty, and error states and remain
  keyboard-operable.
- Never commit credentials, tokens, private keys, `.env`, or generated local
  state.

## §15 — Evidence and anti-circumvention

If a test or gate did not run, do not claim it passed. A DIFF/FULL green writes
`.git/omarchy-gaming-system-gate-receipt`, containing a hash of the gated worktree.
The Codex commit hook recomputes that hash before a commit touching gated
files. Any later gated edit invalidates the receipt.

Hooks are a discipline scaffold, not a security boundary. The receipt is the
load-bearing proof. Do not amend this constitution or its hooks mid-pipeline to
escape a failure.

Codex also records local pipeline-tool receipts under `.git`. Phase 2 and Phase
3.5 claims require CodeGraph evidence for the current pipeline and gated
worktree. Phase 5 claims require an OpenWiki lifecycle that finished against
that same state. Any later gated edit invalidates the corresponding receipt.

## §18 — Inspect and knowledge first

Phase 3.5 is mandatory. Review the implementation through independent lenses:

- correctness and acceptance-criteria coverage;
- authentication, authorization, input, secrets, and privacy;
- database, concurrency, game-state, and migration integrity;
- simplification and reuse;
- QML usability and keyboard behavior when client surfaces changed.

Before planning and implementing, search the local knowledge register, nearest
completed pipeline notes, generated OpenWiki, and relevant architecture
documents. Use CodeGraph during design and inspection to expose runtime flows,
callers, and blast radius, with direct review for unsupported sources. At
completion, reconcile OpenWiki and record new failures (`BF-*`), prevention
rules (`PR-*`), and architecture decisions (`AD-*`) in the AAR and in the
knowledge register.

## §19 — Local work record

The project owns its workflow and memory inside the repository:

| Surface | Location |
|---|---|
| Tickets | `docs/planning/tickets/` |
| Active/completed pipelines | `docs/planning/pipeline/` |
| Knowledge register | `docs/planning/knowledge/INDEX.md` |
| After-action reviews | `docs/planning/knowledge/aar/` |
| Architecture decisions | `docs/architecture/` |
| Cross-session notices | `docs/planning/bulletins/INDEX.md` |

An ID recorded only in an AAR is not recallable; add it to the register too.

## Amending this constitution

Name the section, proposed change, and reason. Tightening a rule is ordinary
maintenance. Loosening a rule requires an architecture decision and must not
be used to unblock the pipeline that proposes it.

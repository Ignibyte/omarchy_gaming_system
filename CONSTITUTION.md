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
2. Clippy across the workspace and all targets with warnings denied
3. Workspace tests
4. Rust documentation with warnings denied
5. Docker Compose validation
6. Bash syntax validation for project and Codex hook scripts
7. Whitespace validation across tracked, staged, and untracked files
8. The real PostgreSQL migration → Rust health API → QML health-client smoke
   path (DIFF/FULL only)

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

- The server is authoritative for authentication, game state, turns, time,
  randomness, rewards, and permissions.
- Accounts and personas are separate domain identities. Social and game
  surfaces expose personas, not account ownership.
- REST/JSON is the durable command/query interface. WebSockets notify clients;
  cursor-based synchronization remains the recovery source of truth.
- Transport handlers remain thin. Domain and game logic live in testable Rust
  modules without direct UI coupling.
- Games start as compiled Rust crates. Loading third-party native code is out of
  scope; any future extension runtime requires an explicit sandbox decision.
- The QML connector is keyboard-first and consumes only public API contracts.

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

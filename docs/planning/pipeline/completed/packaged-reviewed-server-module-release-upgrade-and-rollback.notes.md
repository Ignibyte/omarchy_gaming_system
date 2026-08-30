---
title: Packaged reviewed server-module release upgrade and rollback — notes
pipeline_id: 4f5a60a7-b2ab-4c18-a93e-76b2c047763d
---

# Packaged reviewed server-module release upgrade and rollback — running notes

Chronological evidence and decisions. If a test or command did not run, these
notes must not claim it passed.

## Phase 1 — Plan

- Delivery baseline: Ticket 041 is committed and published on `origin/main`;
  local `HEAD` and the tracking ref both equal
  `98fe4831b1a5e91dc36fe8b529e0cc49643930b9`, and the worktree was clean
  before this pipeline opened.
- Preflight: no active pipeline or blocking bulletin existed; CodeGraph 1.5.0
  and OpenWiki 0.3.3 passed `scripts/check-pipeline-tools.sh`; PostgreSQL was
  healthy; and no other Cargo or gate process was running.
- Roadmap routing: the first external two-installation acceptance run and
  hosted origins require outside operators/infrastructure. The next autonomous
  owner-server gap is reviewed module upgrade/rollback; the public Provider SDK
  remains a larger later track.
- Sequence recall: Ticket 039 selected the exact no-WASI process/WIT boundary;
  Ticket 040 shipped one packaged reviewed observation release; Ticket 041
  generalized custody/runtime/lifecycle for operator-custom releases but left
  the reviewed path fixed at release 1.0.0.
- Architecture recall: publisher integrity, project/marketplace review, server
  admission, granted capability, and measured containment are separate facts.
  This slice uses packaged first-party review only and must not manufacture a
  marketplace-vetted claim.
- Recalled rules: semantic receipt identity excludes delivery attempts;
  readiness is reauthorized under finalization locks; optional observations
  fail open with bounded gap evidence; restored modules remain disabled until
  exact review/readiness; operation UUIDs identify whole commands; retained
  request/response preimages survive outbox pruning; and effect sinks rederive
  opaque identities from authoritative roots.
- Planning decision: add one exact compatible packaged successor and reuse the
  Ticket 041 one-step candidate/snapshot lifecycle semantics without adding a
  hook, capability, public route, client action, marketplace import, or game
  authority.
- Phase 1 is PASS.

## Phase 2 — Design

### Architecture and data flow

1. The runtime crate owns a two-entry, compile-time packaged catalog. Release
   `1.0.0` remains the initial selection; release `1.1.0` has a distinct UUID,
   review UUID, component digest, version, and `state/v2` schema while retaining
   the exact WIT major, hook, capability, budgets, no-WASI host, configuration
   schema, and typed intent.
2. Configured startup registers and byte-compares both immutable release and
   provenance envelopes under the existing registry advisory lock. It inserts
   a new instance only at release `1.0.0`, then resolves an existing instance's
   selected release by exact UUID. Registration never changes that selection.
3. Dispatcher claim reconstruction uses the same exact packaged-catalog lookup
   instead of assuming release `1.0.0`. Database-custodied operator-custom and
   future marketplace-vetted claims retain the generic verification branch;
   all classes converge on the existing `ReviewedRelease` host request,
   process supervisor, response receipt, and core effect reauthorization.
4. A new database-local `reviewed-module-apply` command accepts only `upgrade`
   or `rollback`, the fixed reviewed instance, all three expected mutable
   revisions, a whole-command UUID, actor, and reason. Upgrade additionally
   requires the exact `1.1.0` release UUID and a complete bounded `state/v2`
   candidate map. Rollback accepts no target or state and consumes only the
   retained immediate `1.0.0` predecessor and its snapshot.
5. Command validation and an exact replay lookup happen before preparation.
   Preparation loads one coherent instance/namespace/release view, resolves
   only the allowed packaged transition edge, signs a candidate admission,
   and executes contained readiness outside SQL. Finalization takes the shared
   registry lock, rechecks lifecycle/configuration/state/current-release/
   activation/restore/predecessor inputs, byte-compares the target database row
   with its packaged release, and atomically inserts the fresh admission,
   predecessor snapshot, migrated/restored namespace, lifecycle/data audit,
   operation receipt, stale-work gap evidence, and instance swap.
6. Existing `module-restore` still disables every nonterminal module and clears
   its admission. Existing reviewed `module-apply recover` clears the restore
   gate; configured startup then resolves and probes the exact selected release
   before publishing a fresh admission. It never substitutes release `1.0.0`
   for a selected `1.1.0` release.
7. If configured startup cannot reproduce or execute the exact selected
   packaged release, the optional module service remains absent while the HTTP
   core starts. The application installs an unconfigured emitter in this case,
   so later reports record bounded `runtime_unconfigured` evidence instead of
   building an undrainable queue. Malformed all-or-none secret configuration
   remains a startup error at configuration parsing.
8. Server ownership remains unchanged: only the local administrator mutates
   lifecycle state; core owns admissions, state, audit, outbox, receipts, and
   effect authorization. Clients receive neither inventory nor executable
   bytes. Public discovery continues to warn only for active/degraded
   operator-custom provenance, never for packaged reviewed releases.

### Exact file manifest

| File | One purpose |
|---|---|
| `crates/server-module-runtime/fixtures/valid-v2.wat` | Supply the distinct compatible packaged successor component without new authority. |
| `crates/server-module-runtime/build.rs` | Componentize and embed the successor fixture deterministically. |
| `crates/server-module-runtime/src/lib.rs` | Define the bounded packaged catalog, successor identities, and exact UUID lookup while preserving the release-1 compatibility helpers. |
| `crates/server-module-runtime/tests/conformance.rs` | Prove catalog determinism, identity separation, compatibility, and exact lookup/rejection. |
| `migrations/0028_packaged_reviewed_server_module_releases.sql` | Add the immutable reviewed lifecycle operation/receipt ledger and constraints; do not rewrite prior migrations. |
| `crates/server/src/server_modules.rs` | Register the catalog, resolve selected packaged releases exactly, implement reviewed upgrade/rollback preparation and atomic finalization, and expose test seams. |
| `crates/server/src/server_module_tests.rs` | Exercise registration, real upgrade/rollback/replay/races/atomicity, stale work, restart, restore, absent catalog entries, and shared dispatch. |
| `crates/server/src/main.rs` | Preserve core startup and select the gap-recording emitter when an exact optional module runtime is unavailable. |
| `crates/server/src/bin/omarchygs-admin.rs` | Decode and route the new private `reviewed-module-apply` command only. |
| `crates/server/tests/operator_cli.rs` | Prove the real CLI contract, private command custody, exact JSON receipt, replay, and hostile inputs. |
| `scripts/test-server-modules.sh` | Pin the packaged catalog/local-only boundary and both reviewed/custom contained-host paths. |
| `docs/architecture/server-modules.md` | Reconcile the reviewed catalog and lifecycle architecture. |
| `docs/operators/server-modules.md` | Document release identities, upgrade/rollback/restart/downgrade/restore procedure, and failure evidence. |
| `docs/planning/ROADMAP.md` | Close only the reviewed upgrade/rollback compatibility item. |
| Ticket/spec/notes/AAR/knowledge/OpenWiki artifacts | Preserve lifecycle evidence and durable lessons during Phase 5; generated OpenWiki pages are changed only by its lifecycle. |

### Database and migration consequences

- Migration `0028` creates `server_module_reviewed_operations` with a primary
  operation UUID, canonical command digest, action, instance, previous/result
  release IDs, predecessor snapshot, every expected/resulting revision,
  lifecycle result, actor/reason, and timestamp. Foreign keys retain all
  referenced evidence; bounds fix actions to `upgrade|rollback`, require the
  built-in instance, and enforce one-step monotonic lifecycle/state revisions.
- Row and truncate immutability reuse the existing evidence-rejection trigger.
  Release, admission, instance, namespace, snapshot, lifecycle audit, data
  audit, outbox, delivery, and intent schemas need no destructive change: the
  Ticket 041 migration already generalized release provenance, state schemas,
  and one-predecessor rollback columns.
- Packaged component bytes remain absent from PostgreSQL. Exact signed envelopes
  and digests are retained there, while executable bytes remain package-owned
  and are reproduced only by exact catalog identity.
- Upgrade terminalizes all nondelivered work with `admission_replaced`, adds its
  count to saturating observation-gap evidence, and retains already terminal
  rows/receipts. Rollback does the same before consuming predecessor pointers.

### Command contract and compatibility

- New input format:
  `omarchygs.packaged-reviewed-module-lifecycle-command/v1`; new receipt format:
  `omarchygs.packaged-reviewed-module-receipt/v1`.
- Upgrade requires `target_release_id` and `candidate_state`; rollback rejects
  both. Unknown fields, nil IDs, zero revisions, noncanonical or oversized
  state, changed replay bodies, unknown release edges, custom provenance, and
  restored/retired/inactive policy conflicts fail closed.
- Existing environment names, `modules`, `module-apply`, `module-restore`,
  custom-module commands, inventory v1, HTTP/WebSocket schemas, discovery, and
  QML remain compatible. No automatic release selection or network API is
  introduced.
- A pre-Ticket-042 binary cannot know the successor catalog; operators must not
  downgrade below the first catalog-aware build after selecting `1.1.0`.
  Catalog-aware startup itself treats any missing selected release/host as an
  optional-runtime outage: no substitute executes, core stays available, and
  reports record `runtime_unconfigured` until exact package recovery.

### Regression and evidence plan

| Requirement | Evidence |
|---|---|
| REQ-001 | Runtime catalog unit tests plus PostgreSQL fresh registration, exact replay/conflict, existing-v1 selection, upgraded-v2 restart, and no-auto-upgrade assertions. |
| REQ-002 | Injected-executor PostgreSQL upgrade test and real `reviewed-module-apply` CLI test proving all revision checks, complete candidate state, fresh readiness/admission, atomic state/release publication, immutable audit/receipt, and exact replay. |
| REQ-003 | Pending/in-flight stale-work fixtures, snapshot restoration, fresh rollback admission, cleared predecessor pointers, repeated/arbitrary rollback denial, and concurrent same/stale command races. |
| REQ-004 | Unknown/changed catalog row, wrong target, changed replay, stale lifecycle/config/state, invalid candidate/schema/quota, unavailable/rejected readiness, and mutation-count/state-root invariants. |
| REQ-005 | Runtime conformance for both packaged releases; post-upgrade report dispatch and core label receipt; existing custom dispatch/discovery/QML suites; source scans proving no public route or custom warning drift. |
| REQ-006 | Selected-v2 restart, injected missing-catalog startup, optional-service emitter selection, restore reconciliation/recover/restart, and documented pre-catalog-aware downgrade floor. |
| REQ-007 | Focused runtime/server/CLI/database suites, `scripts/test-server-modules.sh`, CodeGraph inspection, security scan, OpenWiki lifecycle, documentation audit, and `bin/gate.sh --diff`. |

### Risk analysis

| Risk | Control |
|---|---|
| Package/DB identity confusion executes old bytes under a new admission | Catalog lookup binds UUID, signed envelopes, component/WIT digests, schemas, provenance, and database row before readiness and dispatch. |
| Readiness succeeds against state that races finalization | Probe occurs outside SQL, then finalization re-locks and compares every prepared mutable and immutable input before one transaction publishes. |
| Upgrade strands old in-flight work or applies an old effect | Finalization dead-letters every nonterminal old-admission event with counted gap evidence; effect commit already rechecks selected release/admission/lifecycle. |
| Rollback becomes an arbitrary downgrade or reusable snapshot | Only the fixed successor's stored immediate predecessor/snapshot is accepted, and finalization clears both pointers. |
| State migration partially mutates live data | Candidate state is complete and bounded before preparation; namespace, snapshot, admission, audits, receipt, and instance change in one transaction. |
| Optional runtime failure either stops core or silently drops observation | Startup returns no module worker, main uses the unconfigured emitter, and the report transaction records `runtime_unconfigured`. |
| Reviewed release accidentally acquires custom support warning or marketplace status | Packaged catalog fixes first-party provenance; discovery remains derived only from active/degraded `operator_custom` rows. |
| Command UUID replays changed intent | The dedicated immutable ledger compares the canonical whole-command digest before returning a stored result. |
| Private state or trust material leaks through CLI/inventory/logs | Candidate values and signed bodies are never returned or logged; receipt/inventory expose only bounded IDs, revisions, digests/counts, and lifecycle. |

### CodeGraph and direct-source evidence

- CodeGraph 1.5.0 explored `reviewed_release`, `register_and_enable`,
  `register_release_and_instance`, `release_from_claim`, startup configuration,
  `main`, the administrator CLI, custom lifecycle preparation/finalization,
  restore, and their test callers. It found the runtime-to-server production
  path concentrated in `server-module-runtime/src/lib.rs` and
  `server/src/server_modules.rs`, with the custom lifecycle's injected-probe
  PostgreSQL tests providing the nearest transition model. The matching design
  receipt is bound to this pipeline and gated worktree.
- The explored graph reported thin direct test edges for reviewed
  `register_and_enable`, registration, and claim reconstruction. The file
  manifest therefore adds direct fresh/existing/restart/package-mismatch and
  post-upgrade dispatcher tests rather than relying only on indirect report
  coverage.
- Direct inspection covered unsupported WAT, SQL migrations, shell boundary
  checks, OpenWiki server-module context, the operator runbook, the module host
  build loop, custom transition SQL, restore SQL, CLI dispatch, and current
  PostgreSQL/CLI test fixtures.

### Material alternatives rejected

- Rejected treating the successor as `marketplace_vetted`: packaged project
  review and marketplace review are separate attestations.
- Rejected automatic startup upgrade or a configuration version selector:
  installation/restart must not mutate executable selection.
- Rejected storing successor bytes in PostgreSQL: packaged reviewed custody is
  reproduced by the exact installed binary, unlike operator-custom custody.
- Rejected reusing the custom operation table: its fingerprint and explicit
  unreviewed-acknowledgement constraints are intentionally provenance-specific.
- Rejected composing legacy state-migrate and lifecycle commands: that would
  expose a partial crash boundary between state and release/admission changes.
- Rejected arbitrary graph rollback and repeated snapshot reuse: one immediate
  predecessor is the bounded recovery contract.
- Rejected adding inventory/public API/QML controls: the local command already
  has the required authority and avoids remote executable administration.

- Phase 2 is PASS.

## Phase 3 — Implement

- Built:
  - Added a deterministic two-release packaged Sentinel catalog, distinct
    successor component/release/review identities, exact UUID lookup, and
    conformance coverage for both executable components.
  - Added forward-only migration `0028` with an immutable, whole-command
    reviewed upgrade/rollback operation ledger and constrained revision,
    predecessor, actor, and receipt evidence.
  - Registered and byte-compared every packaged release while preserving
    release `1.0.0` as the only initial selection; startup and dispatcher
    reconstruction now resolve the exact selected packaged release.
  - Added canonical private `reviewed-module-apply` decoding and the upgrade /
    one-step rollback prepare-probe-finalize path with locked reauthorization,
    complete candidate state, fresh admission, immutable snapshot/audits,
    stale-work terminalization, observation-gap accounting, replay, and
    concurrency protection.
  - Kept the optional core fail-open when the selected artifact, catalog row,
    state-schema root, or contained host is unavailable; reports use the
    unconfigured emitter and record bounded gap evidence rather than queuing
    work with no worker.
  - Extended real PostgreSQL, process-host CLI, runtime boundary, restart,
    restore, atomicity, hostile replay, changed-package, and schema-aware state
    maintenance coverage; updated the architecture, roadmap, operator runbook,
    and source boundary scan.
- Deviations:
  - Self-review extended the planned schema transition guard into the existing
    disabled-state maintenance commands. State migration snapshots now record
    the live selected schema, and legacy rollback rejects cross-schema
    snapshots. This is a compatibility invariant required by the v2 release,
    not a new administrative surface.
  - Self-review also normalized packaged catalog contract conflicts to the
    existing optional-runtime `Unavailable` path during startup. Explicit
    administrator transitions still return fail-closed conflict errors.
- Focused verification:
  - `cargo test -p omarchygs-server-module-runtime --all-targets`: 6 passed,
    1 intentionally ignored.
  - `scripts/test-server-modules.sh`: passed the packaged/custom runtime,
    contained-host, docs, clippy, local-only, and source-boundary checks.
  - `cargo clippy -p omarchy-gaming-system-server --all-targets -- -D warnings`:
    passed after the final startup/schema review fixes.
  - Focused PostgreSQL tests passed for reviewed upgrade/dispatch/rollback,
    atomic failure/concurrent replay/missing package/restore, and successor
    schema-aware state maintenance.
  - The real `reviewed-module-apply` operator CLI integration passed through
    the packaged contained host, including private file custody, upgrade,
    non-secret output, replay, changed-command conflict, and rollback.
  - `scripts/test-database.sh` passed before the final schema-boundary
    hardening; the complete database suite will be rerun in Phase 4.
  - `git diff --check`: passed.

- Phase 3 is PASS.

## Phase 3.5 — Inspect ledger

| # | Lens | Finding | Severity | Disposition |
|---|---|---|---|---|
| 1 | Correctness / compatibility | Disabled-state migration still persisted the historical `state/v1` literal and could label successor state or accept a predecessor snapshot across schema boundaries. | High correctness | Fixed in Phase 3: lock the instance, namespace, and release schema roots; persist the live selected schema; reject cross-schema legacy rollback; focused PostgreSQL coverage passed. |
| 2 | Availability / recovery | A packaged catalog contract conflict during configured startup could escape as a fatal startup error instead of an optional-runtime outage. | Medium | Fixed in Phase 3: normalize packaged catalog construction/conflict/input failures to `Unavailable` only at startup; core installs the gap-recording emitter; changed-catalog and schema-mismatch tests passed. |
| 3 | Concurrency / atomicity | Transition finalization rechecked the instance schema after out-of-transaction readiness but did not independently recheck the locked namespace schema. | Medium | Fixed: select `n.state_schema` into `LockedReviewedTransition` and compare it with the prepared candidate schema before any publication. Focused reviewed PostgreSQL tests and Clippy passed. |
| 4 | Data integrity / defense in depth | Application validation restricted reviewed operations to the packaged v1/v2 edge, but the immutable database ledger's check constraint did not pin those exact release pairs. | Low | Fixed: migration `0028` now constrains upgrade to v1→v2/schema-v2 and rollback to v2→v1/schema-v1; the static module gate and migrated PostgreSQL tests passed. |
| 5 | Security | Full changed-source diff scan reviewed 11 source-like files across artifact construction, signature/admission verification, Wasm containment, local command custody, startup, locked transition publication, SQL evidence, and tests. No candidate survived the reportability gates. | Informational | Complete; sealed report at `/tmp/codex-security-scans/omarchy_bbs/98fe4831_local_IqbY9LyP/report.md`, with complete coverage and zero reportable findings. |

- Fresh post-fix CodeGraph inspection read the current transition source and
  traced `apply_reviewed_release_command_with_executor` through preparation and
  `finalize_reviewed_transition`. It confirmed the namespace-schema root is in
  the locked final comparison and identified the expected callers/tests; the
  runtime and PostgreSQL suites provide the behavioral evidence where the
  graph cannot infer async integration coverage.
- Post-inspection verification: `cargo fmt --all -- --check`, server Clippy,
  the focused `reviewed_` PostgreSQL/CLI group (5 tests), and
  `scripts/test-server-modules.sh` all passed. One preliminary focused test
  invocation omitted `DATABASE_URL` and failed before test setup; rerunning
  the exact command with the repository database URL passed.

- Phase 3.5 is PASS.

## Phase 4 — Validate

- Tests run:
  - `cargo fmt --all -- --check`: passed.
  - `cargo clippy -p omarchy-gaming-system-server --all-targets -- -D warnings`:
    passed.
  - Focused `reviewed_` PostgreSQL/CLI group: 5 passed after rerunning with
    `DATABASE_URL`; the preliminary invocation without that required variable
    failed before database test setup and is not counted as product evidence.
  - `scripts/test-server-modules.sh`: passed all 6 ordinary conformance tests,
    the separately invoked real containment test, docs/clippy checks, and the
    local-only automation boundary.
  - `scripts/test-database.sh`: passed the historical marketplace migration,
    8 library database tests, 66 server integration tests, 5 administrator
    tests, 7 real CLI tests, and doc tests with zero failures.
- Gate run: `bin/gate.sh --diff` passed all 24 stages and wrote worktree-bound
  receipt `c5b33d8c03efece46e9058ba7945dfe9bbe513b0d8c856819f4194df47918d51`.
  The gate included its own full PostgreSQL integration rerun, native package
  reproducibility, live Rust/QML smoke, remote-provider security and authority
  pilots, backup/restore and invitation drills, server-module architecture
  proof, and real production module-host containment.
- Skips or pre-existing failures: none; the runtime's containment test is
  intentionally ignored in the ordinary test binary and then run separately
  by `scripts/test-server-modules.sh`, where it passed.

- Phase 4 is PASS.

## Phase 5 — Complete

- Acceptance-criteria audit:

  | Requirement | Disposition and evidence |
  |---|---|
  | REQ-001 | PASS — runtime conformance and PostgreSQL startup cases prove the two-entry catalog is deterministic and byte-exact, new instances select `1.0.0`, existing selections remain unchanged, and selected `1.1.0` restarts without substitution. |
  | REQ-002 | PASS — the real private CLI and PostgreSQL transition cases prove all three revision guards, complete `state/v2` candidate validation, contained readiness, fresh admission, one atomic state/release swap, immutable whole-command evidence, exact replay, and changed-body conflict. |
  | REQ-003 | PASS — upgrade/dispatch/rollback tests prove immediate-predecessor snapshot restoration as a new revision, fresh admission, stale-work terminalization and gap evidence, pointer consumption, repeated rollback denial, and concurrent identical-command replay. |
  | REQ-004 | PASS — hostile catalog, package absence, changed manifest, namespace-schema mismatch, stale command, readiness failure, concurrency, and incompatible state cases leave live roots unchanged or keep only the optional worker unavailable. |
  | REQ-005 | PASS — both packaged releases and operator-custom fixtures pass the shared WIT, signature/admission, budget, no-import, real containment, dispatcher, receipt, and core-effect checks; source and full-gate QML/API coverage confirms no public administration, warning drift, executable delivery, or gameplay authority. |
  | REQ-006 | PASS — exact successor restart, missing/changed package behavior, gap-recording emitter, audited restore reconciliation, explicit recovery, and operator downgrade guidance prove no substitute execution and preserved core availability. |
  | REQ-007 | PASS — architecture, operator, roadmap, OpenWiki, AAR, and knowledge register are reconciled; focused validation, complete database runs, security scan, fresh CodeGraph inspection, OpenWiki completion, and all 24 diff-gate stages passed. |

- Docs:
  - Hand-maintained server-module architecture and operator guidance now
    document both exact packaged releases, explicit upgrade/rollback commands,
    package downgrade/recovery behavior, schema boundaries, and unchanged
    public/client authority.
  - OpenWiki update run `b3df606d-d86a-4e0a-8363-768c407910a2` completed after
    reconciling `server-modules.md` and `quickstart.md`. It reported a
    non-blocking warning for unrelated pre-existing quickstart Claims evidence
    debt; the Ticket 042 claims were resolved before authoring.
  - The completion receipt names pipeline
    `4f5a60a7-b2ab-4c18-a93e-76b2c047763d`, tool
    `mcp__openwiki__openwiki_finish`, and gated state
    `1d74a6241e750cb75d5e395d5b45477eaba075f493141e03c83a5dca59c0ef3a`.
- AAR:
  - AAR-042 is submitted with five captured failures, five standing prevention
    rules, and the packaged reviewed release-lifecycle decision. Every new ID
    is appended to `docs/planning/knowledge/INDEX.md`.
- Archive:
  - Ticket 042 is closed and this spec/notes pair is archived. The final
    post-completion `bin/gate.sh --diff` passed all 24 stages; its worktree-bound
    gate receipt and the OpenWiki completion receipt both match gated state
    `1d74a6241e750cb75d5e395d5b45477eaba075f493141e03c83a5dca59c0ef3a`.
    Staged review and explicit user authorization remain required before any
    commit or push.

- Phase 5 is PASS.

## Defect and lesson ledger

| # | What happened | Root cause | Fix | Prevention |
|---|---|---|---|---|
| 1 | The first real CLI fixture was rejected by canonical decoding. | The test serialized an ad hoc JSON object whose key order did not match the canonical typed command representation. | Serialize the actual command structure before writing the mode-0600 command file. | Build canonical-command fixtures from production types and retain changed-body rejection coverage. |
| 2 | Self-review found legacy state migration snapshots still labeled every state root as `state/v1`. | Ticket 040's single-release invariant had been encoded as a literal and became invalid once the selected release could use `state/v2`. | Bind the coherent live schema into snapshots, compare instance/namespace/release schemas under lock, and reject cross-schema legacy rollback. | Add the successor-schema maintenance test and review every persisted schema literal when introducing a new schema version. |
| 3 | A changed packaged catalog row could surface as a fatal startup conflict. | Registration errors predated the exact-artifact optional-runtime recovery rule. | Map only packaged catalog contract/conflict failures at startup to `Unavailable`; retain database/internal failures and administrator conflicts as fatal/fail-closed. | Test changed-catalog, missing-release, and state-schema mismatch startup independently from administrator transitions. |
| 4 | Inspection found readiness finalization compared the instance state schema but not the separately locked namespace schema. | The preparation query enforced their equality, but the final race check modeled the schema as one root even though two rows can change independently. | Carry `namespace_state_schema` in the locked row and require it to equal the prepared candidate schema. | Model every independently mutable database root explicitly in post-readiness reauthorization and keep final lock queries visible in CodeGraph inspection. |
| 5 | The reviewed-operation table allowed any UUID edge satisfying broad action/revision checks if a privileged writer bypassed application code. | Exact release-edge authority existed only in Rust validation, while the ledger was intended to be self-describing immutable evidence. | Add an exact action/release/schema edge check to migration `0028` and pin it in the module boundary script. | When an audit table has a finite operation vocabulary, encode the allowed state graph in both application validation and database constraints. |

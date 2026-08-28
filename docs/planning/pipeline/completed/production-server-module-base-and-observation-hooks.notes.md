---
title: Production server-module base and observation hooks — notes
pipeline_id: 49248bf8-87d9-4cfe-886c-492133c4a89c
---

# Production server-module base and observation hooks — running notes

Chronological evidence and decisions. If a test or command did not run, these
notes must not claim it passed.

## Phase 1 — Plan

- Delivery baseline: Ticket 039 is closed and delivered at
  `48b83c089b5e95eaa152f4de08c03654efc1dde2`; GitHub `main` matched that SHA
  and the worktree was clean before Ticket 040 opened.
- Preflight: no active pipeline or blocking bulletin existed; CodeGraph 1.5.0
  and OpenWiki 0.3.3 passed `scripts/check-pipeline-tools.sh`; PostgreSQL was
  healthy; and no Cargo process was running.
- Sequence recall: Ticket 039 selected and proved the process-isolated no-WASI
  Component Model boundary. Ticket 040 is the bounded observation-only
  production base. Ticket 041 separately gates operator-custom installation
  and provenance.
- Architecture recall: authoritative domain transactions must append their
  allowlisted module observation in the same PostgreSQL commit. A dispatcher
  invokes the exact admitted release only after commit, and a core domain
  service independently reauthorizes any typed intent and commits the effect
  with an immutable receipt.
- Authority recall: cartridges remain inert presentation, compiled games
  remain reviewed core code, registered providers remain the sole remote
  gameplay authority for their pinned sessions, and server modules cannot
  receive database handles, account ownership, reusable credentials, raw
  arbitrary URLs/paths, provider grants, or executable client authority.
- Recalled production rules include exact authority binding, bounds during
  streaming/read, stable-root receipt serialization, empty partition pruning,
  one release per containment process, supported-layer resource enforcement,
  and worktree-bound gating for every independently executable source tree.
- The slice is locked to one checked-in first-party fixture, one safe
  post-commit observation hook, and one bounded typed intent. Arbitrary package
  upload, admission hooks, custom provenance, and generalized egress remain
  out of scope.
- Phase 1 is PASS.

## Phase 2 — Design

- Current topology and blast radius:
  - CodeGraph traced `app::create_report` into `reports::create_report`, its
    owner-scoped report transaction and replay branch, `AppState` construction,
    `main` runtime startup/shutdown, the immutable `server_identity`, and the
    provider registry's stable-root receipt pattern. The report domain has two
    callers and an existing PostgreSQL/API corpus; the new emitter parameter
    is therefore explicit and optional rather than global state.
  - The report insert is the first safe observation point. On first delivery,
    its transaction will append one metadata-only module event before commit.
    Exact report replay returns before enqueue and therefore cannot duplicate
    the event. The dispatcher claims and executes only after the transaction
    has committed and never retains a SQL transaction across host execution.
  - Production startup currently owns optional provider and cartridge
    runtimes. The module runtime follows that pattern but adds a background
    dispatcher handle. With module configuration absent, `AppState` receives
    no emitter, no host is started, the router inventory is byte-for-byte
    unchanged, and report creation executes its existing SQL path.

### Production architecture and data flow

1. The production `omarchygs-server-module-runtime` crate owns the exact v1
   WIT, canonical release/provenance/admission/hook/response contracts, the
   checked-in first-party Sentinel fixture source, deterministic component
   generation, pinned Wasmtime runtime, bounded framing, and the native
   `omarchygs-module-host` binary. It is a normal workspace crate; the Ticket
   039 nested spike remains independent historical proof.
2. The only configurable release is the compile-time allowlisted
   `ignibyte.sentinel` v1 descriptor and component digest. Enabling requires an
   exact boolean selector, a 32-byte core admission signing seed, and a
   separate 32-byte pairwise-subject secret. No component path, package URL,
   publisher key, hook name, capability, or host binary path comes from the
   environment or database.
3. Startup verifies the separately signed fixture release and reviewed-fixture
   provenance against compiled authorities and exact allowlisted identity,
   WIT, component digest, requested hooks/capabilities, and budgets. It inserts
   immutable release inventory while keeping requested power ungranted, probes
   the sibling packaged host under the complete sandbox, then creates a
   server-specific signed admission containing only the explicit
   `persona_reported` and `moderation_add_label` grants.
4. `reports::create_report` accepts an optional `ModuleEmitter`. After the
   report row is inserted and before commit, an enabled emitter locks the
   stable instance root, checks the current exact admission and a hard
   undelivered-row quota, derives an HMAC pairwise subject for the reported
   persona, and inserts one event. The serialized hook contains report UUID,
   bounded category, pairwise subject, current configuration/state snapshots,
   and revisions. It excludes reporter persona/account, username, report
   detail, token/MFA material, database data, path, URL, provider grant, and
   arbitrary JSON.
5. One PostgreSQL dispatcher claims only the oldest eligible event for each
   `(release, hook, pairwise subject)` partition with a lease and
   `FOR UPDATE SKIP LOCKED`. A global batch ceiling and bounded worker count
   cap memory. Later partition rows cannot pass an earlier pending/retry row.
   The claim transaction commits before the host is invoked.
6. A host invocation is one exact release in one fresh dedicated process.
   The launcher uses reviewed absolute helper/binary paths, `prlimit`,
   Bubblewrap namespaces/capability/environment/filesystem policy, and
   systemd scope limits when supported. The embedded component is read-only,
   the host links no WASI/import, creates a fresh fuel/memory-limited Store,
   emits bounded readiness, handles one bounded request, and exits. Parent
   startup, execution, and exit deadlines are independently enforced.
7. Sentinel may return no-op or propose only `moderation_add_label` with a
   numeric allowlisted label and expected revision. The module does not choose
   a target: core derives the report and persona from the durable source event,
   locks the stable instance/event/report roots, rechecks current lifecycle,
   admission, hook, capability, subject, open-report policy, expected revision,
   response context/digest, and idempotency, then inserts a core-owned report
   label plus immutable intent and delivery receipts in one transaction.
8. Failure records a stable code, releases the lease into bounded exponential
   retry, and dead-letters after the configured maximum. Consecutive failures
   trip the instance to `degraded`; no fresh delivery is claimed until explicit
   recovery. Disable, emergency suspension, retirement, startup reconciliation,
   and restore reset/stop work through expected-revision, same-transaction
   lifecycle audit. Host failure never changes the already committed report.
9. Core-owned configuration and state use one bounded JSON-object namespace
   per instance/schema with entry/value/total-byte quotas and CAS revision.
   Migration snapshots the live namespace, applies explicit typed set/remove
   operations to an isolated candidate, validates schema/quota, then swaps in
   one transaction. Failed migration is atomic; rollback restores the retained
   snapshot. Restore always leaves the instance disabled, clears stale leases,
   requeues in-flight work, and requires artifact/readiness verification before
   a new active admission.

### Database and compatibility consequences

- Forward-only migration `0025_server_modules.sql` adds immutable release and
  admission inventories; a stable instance root; insert-only lifecycle audit,
  delivery receipt, intent receipt, report-label, and state-snapshot tables; a
  mutable bounded outbox; and one live namespaced state row. Database checks
  constrain every format/status/hook/capability/digest/size/revision/attempt,
  and immutable tables reject update/delete/truncate.
- Existing tables, columns, constraints, public routes, response schemas,
  discovery capability inventory, client behavior, cartridge/provider
  authority, and default environment contract remain compatible. The report
  route adds only an internal optional emitter. Because this first hook is
  optional observation, module inactivity or saturation commits the core
  report and increments bounded aggregate gap evidence in the same transaction
  without adding queue rows.
- No public module route is added. Database-local `omarchygs-admin` gains
  bounded module inventory and expected-revision lifecycle/restore commands;
  it cannot install bytes, change grants, provide a host path, or admit a
  custom release.

### File manifest

| Path | Purpose |
|---|---|
| `Cargo.toml`, `Cargo.lock` | Add the production runtime crate and pinned dependencies to the canonical workspace. |
| `crates/server-module-runtime/Cargo.toml`, `build.rs` | Production contract/runtime package and deterministic reviewed-fixture component build. |
| `crates/server-module-runtime/wit/omarchygs-module.wit` | Exact v1 typed component world shared by host and conformance fixture. |
| `crates/server-module-runtime/fixtures/*.wat` | Checked-in valid/no-op/unauthorized/trap/loop/memory/import/interface fixture sources; never runtime-supplied packages. |
| `crates/server-module-runtime/src/lib.rs` | Strict signed contracts, compiled allowlist, verification, bounded frames, Wasmtime execution, and contained process supervisor. |
| `crates/server-module-runtime/src/bin/omarchygs-module-host.rs` | Single-release, single-request native host with readiness and no secret/network/WASI authority. |
| `crates/server-module-runtime/tests/conformance.rs` | Determinism, strict contract, runtime, framing, and hostile process evidence. |
| `migrations/0025_server_modules.sql`, `0026_server_module_observation_evidence.sql` | Registry/admission/instance, outbox/receipts, report label, state/snapshot, lifecycle audit, quotas, immutability, aggregate observation gaps, and retained request evidence. |
| `crates/server/src/server_modules.rs` | PostgreSQL registration, emitter, claim/dispatch, core intent authorization, retry/circuit, lifecycle, state migration/rollback, restore, inventory, and telemetry. |
| `crates/server/src/reports.rs`, `app.rs`, `config.rs`, `main.rs`, `lib.rs` | Optional same-transaction hook insertion, disabled-by-default runtime wiring, exact configuration, worker ownership, and public library surface for local admin. |
| `crates/server/src/server_module_tests.rs`, existing report/config tests | PostgreSQL transaction, privacy, ordering/replay/race/fault/state/lifecycle/restore/default-compatibility evidence. |
| `crates/server/src/bin/omarchygs-admin.rs`, `crates/server/tests/operator_cli.rs` | Database-local bounded inventory, disable/suspend/restore operations, exact errors, and no secret output. |
| `scripts/test-server-modules.sh`, `scripts/test-database.sh`, `bin/gate.sh` | Focused deterministic conformance/process/PostgreSQL proof and canonical local gate integration. |
| Architecture/operator/roadmap/OpenWiki/pipeline artifacts | Production status, config, operations, failures, recovery, evidence, and durable lessons. |

### Requirement-to-evidence regression map

| Requirement | Evidence |
|---|---|
| REQ-001 | Config absent/partial/complete unit tests; default router and report SQL inventory; process-spawn probe asserting zero host; complete existing workspace, database, QML, provider, and cartridge suites. |
| REQ-002 | Strict fixture release/provenance/admission verification; immutable PostgreSQL registration/replay/conflict tests; requested-versus-granted subset and exact component/WIT/digest/authority hostile corpus. |
| REQ-003 | Report commit/rollback/replay tests proving exactly one same-transaction event; exact serialized payload allowlist and sensitive-value absence; inactive/saturated commit plus aggregate-gap tests; row/frame/snapshot quotas. |
| REQ-004 | Oldest-per-partition concurrent claim tests, cross-partition progress, exact receipt replay/conflict and retained request/response preimages, lease recovery, retries/backoff/dead-letter/circuit, bounded batches, timeout, pruning retention, and a lock-duration test that mutates the report while host execution is blocked. |
| REQ-005 | Host-response context/capability/target/current-lifecycle/current-report/expected-revision/idempotency hostile tests plus simultaneous commit proving one label and one immutable receipt. |
| REQ-006 | Real sibling host under OS containment for valid/no-op/unauthorized/trap/loop/memory/import/interface/tamper/exit/hang/restart cases; readiness/resource/secret/network assertions; config-absent process inventory and emergency suspension. |
| REQ-007 | State quota/CAS/stale-write, migration success/failure atomicity, rollback retention, lifecycle expected-state race, backup inventory, restore-disabled, stale-lease reconciliation, and explicit re-enable readiness tests. |
| REQ-008 | Deterministic WIT/fixture builds, clean workspace conformance script, production route/config/authority prohibitions, operator docs, CodeGraph/security/OpenWiki evidence, and complete local `bin/gate.sh --diff`. |

### Security, privacy, concurrency, and recovery risks

- The compile-time fixture signing authority is never sufficient admission:
  core also requires exact built-in identity/digest allowlisting and a
  server-specific signed grant. No generic registry method accepts caller-
  selected executable bytes in this ticket.
- The pairwise secret is purpose-specific and never reused as the admission
  signing seed. Core keeps real persona/report foreign keys in PostgreSQL for
  authorization, while only the derived subject crosses the host boundary.
- Claim, execution, and commit are distinct phases. Lease expiry makes crash
  recovery deterministic; the stable instance root serializes first receipt
  creation; response replay cannot re-run an effect.
- The outbox hard cap never lets an optional extension control core platform
  availability. Inactive/saturated observations increment a saturating
  aggregate counter with stable reason/time in the report transaction, making
  the known evidence gap visible without unbounded per-event growth.
- Database mutation, arbitrary egress, admission hooks, raw private report
  detail, provider/game authority, custom package custody, and client
  executable content remain rejected rather than latent configuration flags.

- The worktree-bound CodeGraph design receipt for pipeline
  `49248bf8-87d9-4cfe-886c-492133c4a89c` covers the report producer,
  transport caller, startup/runtime injection, stable server identity,
  provider receipt precedent, callers, tests, and blast radius.
- Phase 2 is PASS. The design is actionable and remains inside Ticket 040.

## Phase 3 — Implement

- Built:
  - Added the normal-workspace `omarchygs-server-module-runtime` crate with the
    exact `module-production` WIT, deterministic compiled fixture catalog,
    strict signed release/provenance/admission/event/response contracts,
    pinned no-import Wasmtime runtime, bounded canonical framing, fixed sibling
    supervisor, and single-request host binary.
  - Added migration `0025_server_modules.sql` for the exact immutable release
    and admission inventories, lifecycle root/audit, bounded outbox, delivery
    and intent receipts, core-owned report labels, namespaced state, retained
    snapshots, data audit, checks, indexes, and immutability triggers. Added
    migration `0026_server_module_observation_evidence.sql` for aggregate gap
    telemetry plus retained canonical request bodies and target binding on all
    new delivery receipts while identifying legacy incomplete rows honestly.
  - Added the disabled-by-default server service, same-transaction report
    emitter, pairwise privacy derivation, partition-ordered leased dispatcher,
    attempt-normalized replay binding, retry/dead-letter/circuit behavior,
    core reauthorization, lifecycle/restore reconciliation, state CAS,
    migration/rollback, configuration CAS, and safe inventory.
  - Added exact all-or-none environment parsing, synchronous module stop at the
    HTTP-drain edge, inactive-module core startup with a gap-counting emitter,
    and database-local `modules`, `module-apply`, and `module-restore`
    administrator commands without adding a public route or artifact input.
    Every file-backed mutation uses one no-follow, owner/mode/link/stability
    checked bounded reader.
  - Added deterministic/runtime/real-process conformance, PostgreSQL
    transaction/privacy/order/replay/lock/fault/gap/state/lifecycle/restore
    tests, administrator CLI privacy tests, a configured-inactive restored
    server drill, gate stage 24, and current architecture/operator/product/
    roadmap documentation.
- Focused evidence:
  - `cargo check -p omarchy-gaming-system-server --all-targets` and targeted
    `cargo clippy ... -- -D warnings` passed after inspection fixes.
  - `scripts/test-server-modules.sh` passed, including the real
    systemd-user-scope + Bubblewrap + prlimit host crash/hang/restart drill.
  - All six ignored `server_module_tests` passed serially against real
    PostgreSQL migrations, including fail-open gaps, readiness CAS races,
    pruning retention, and upgrade-era receipt semantics.
  - The complete ignored `operator_cli` suite passed against real PostgreSQL
    migrations; `scripts/test-operator-recovery.sh` passed with module config
    present while the restored module remained disabled/pending review.
  - Both direct shutdown/startup-policy tests and the package all-targets unit
    suite passed.
- Deviations:
  - The fixed selector is the exact token
    `OGS_FIRST_PARTY_REPORT_MODULE=enabled`, not a permissively parsed boolean.
  - The first dispatcher uses one worker. This is the smallest bounded worker
    count and preserves every partition order; expansion remains an evidence-
    gated optimization.
  - Delivery receipts normalize only the changing attempt field before hashing
    stable request facts. The host still receives the real one-based attempt.
    This was required for correct at-least-once receipt reconciliation.
- Phase 3 is PASS.

## Phase 3.5 — Inspect ledger

| # | Lens | Finding | Severity | Disposition |
|---|---|---|---|---|
| 1 | Security / availability | Optional observation inactivity and saturation could reject the authoritative report transaction. | Low | Confirmed and fixed: both paths now commit the report and atomically increment bounded `module_inactive`/`queue_saturated` evidence without queue growth. |
| 2 | Security / shutdown | HTTP graceful drain could begin before the module dispatcher was told to stop claiming work. | Low | Confirmed and fixed: a direct cloneable watch trigger is invoked synchronously by the graceful-shutdown future; service shutdown then awaits the bounded worker. |
| 3 | Security / local input | Administrator command files were bounded but could be symlinked, shared, permissively readable, or replaced across the read. | Low | Confirmed and fixed: every file-backed mutation shares `O_NOFOLLOW`, regular-file, effective-owner, 0600, single-link, bounded, and pre/post descriptor-stability validation. |
| 4 | Security / audit | Delivery receipts retained response bytes but only a request digest, preventing independent reconstruction after outbox pruning. | Low | Confirmed and fixed: migration 0026 retains bounded attempt-normalized request bytes and target report for every new receipt; a `NOT VALID` constraint enforces future rows without fabricating legacy evidence. |
| 5 | Concurrency | Host readiness ran outside SQL and finalization initially did not compare every configuration/state revision used by the signed admission. | High correctness | Confirmed and fixed: finalization locks both roots and compares lifecycle, instance config/state, namespace revision, activation/restore flags, and signed admission revisions before any admission/audit insert. |
| 6 | Recovery | A raw PostgreSQL restore cannot intrinsically know it is a restore, so automatic startup reconciliation alone cannot distinguish copied production state. | High operations | Constraint made explicit: the canonical drill runs an audited 0600 `module-restore` command before any restored startup; docs state the raw limitation and require this ordering. |
| 7 | Independent post-patch review | Mapping persisted `Denied` to fatal startup still took down the core server in degraded/suspended/restored states. | High availability | Confirmed and fixed: only `Denied` becomes an inactive optional service while retaining a gap-counting emitter; other host/config faults remain fatal. The configured restored-server drill proves health without activation. |
| 8 | Independent post-patch review | An intermediate oneshot allowed the graceful future to finish before the spawned receiver updated the module watch flag. | High concurrency | Confirmed and fixed by removing the intermediary task/oneshot and invoking the watch trigger synchronously before the graceful future returns. |
| 9 | Independent post-patch review | `system-overview.md` still described saturation as rejecting reports. | Documentation | Fixed to match fail-open aggregate-gap behavior. |

- The single-pass Codex Security review inspected all 18 changed items. Scan
  `0046db3a-80b5-4e25-86e0-5e0fdeee32ee` reported four low-severity findings;
  all were confirmed and fixed above. Its report is retained at
  `/tmp/codex-security-scans-gVuCBs/omarchy_bbs/48b83c089b5e95eaa152f4de08c03654efc1dde2_20260827T232946Z_6enbiizm/report.md`.
- One fresh pre-patch investigator established each finding's source-to-sink
  path, and one independent post-patch read-only reviewer found rows 7–9. The
  fixes were validated locally without opening a second review cycle.
- Fresh CodeGraph inspection traced configuration/startup, report emission,
  dispatcher/receipt persistence, shutdown, callers, tests, and blast radius
  on the final implementation. The worktree-bound inspection receipt matches
  pipeline `49248bf8-87d9-4cfe-886c-492133c4a89c`.
- Phase 3.5 is PASS.

## Phase 4 — Validate

- Tests run:
  - The focused implementation evidence recorded in Phase 3 remained green:
    Rust check/Clippy/unit coverage, six serial real-PostgreSQL module tests,
    the complete real-PostgreSQL operator CLI suite, production host
    conformance, configured-restored-server recovery, and direct startup and
    shutdown-policy tests.
  - `bin/gate.sh --diff` reran the complete canonical local validation surface:
    formatting, warnings-denied workspace lint/tests/docs, Compose and shell
    validation, workflow/secrets/hooks/whitespace, every cartridge and
    marketplace proof, two byte-identical native client packages, 62 serial
    PostgreSQL application tests plus operator suites, the live migrated
    PostgreSQL → Rust API → keyboard-first QML smoke, provider/authority/
    recovery/private-alpha drills, the isolation spike, and production module
    conformance.
- Gate run: all 24 stages passed and the command printed
  `GATE GREEN [diff]`. The delivery receipt, fresh inspection receipt, and
  current gated state all matched
  `1c2208608bdb1d7e2987b4b17b82c909966999dd251e9e238dbb02e824890ee4`
  immediately before the durable phase advance.
- Skips or pre-existing failures: none. Tests marked ignored in the ordinary
  workspace pass were invoked explicitly by their real PostgreSQL, process,
  provider, or recovery gate stages. Cargo emitted the existing advisory that
  transitive `chacha20` 0.10.1 is yanked while packaging the clean provider
  proof; the locked build and every required test still passed.
- Phase 4 is PASS.

## Phase 5 — Complete

- Acceptance-criteria audit:

  | Requirement | Disposition and evidence |
  |---|---|
  | REQ-001 | PASS — default/partial/exact configuration tests, startup-policy tests, unchanged route/discovery inventories, the complete workspace/database/QML gate, and the packaged-host process checks prove disabled-by-default compatibility and no host startup without configuration. |
  | REQ-002 | PASS — strict canonical release, provenance, admission, WIT, component digest, authority, requested/granted subset, immutable registration replay/conflict, and hostile fixture tests bind the exact reviewed Sentinel release without implicit capability grants. |
  | REQ-003 | PASS — real PostgreSQL report tests prove one same-transaction metadata-only event on first commit, no event on rollback/replay, pairwise privacy, hard queue bounds, and fail-open inactive/saturated aggregate gap evidence without rejecting the core report. |
  | REQ-004 | PASS — partition-order, concurrent claim, lock-release, lease/retry/dead-letter/circuit, timeout, replay/conflict, pruning, and migration-era tests prove bounded post-commit execution and retained request/response/target receipt evidence. |
  | REQ-005 | PASS — core apply re-derives subject/target and rechecks exact release, admission, lifecycle, hook, capability, report policy, expected revision, context, and idempotency under PostgreSQL serialization before atomically recording one label and immutable receipts. |
  | REQ-006 | PASS — the real packaged sibling host executes one no-WASI release under systemd-user-scope, Bubblewrap, prlimit, fuel, memory, task, file, frame, startup, execution, and exit bounds; hostile import/interface/memory/loop/trap/exit/restart and emergency-stop cases remain contained. |
  | REQ-007 | PASS — configuration/state quota and CAS tests, readiness finalization races, atomic migration/rollback, lifecycle expected-revision operations, persistent activation stops, audited pre-start restore reconciliation, stale-lease recovery, and configured inactive startup prove the complete state/recovery contract. |
  | REQ-008 | PASS — deterministic author/runtime fixtures, the production module script, database/operator/recovery evidence, architecture/operator documentation, security inspection, fresh CodeGraph inspection, warning-free OpenWiki completion, and all 24 canonical local gate stages are present; source/route checks retain every custom-installation and authority prohibition. |

- Docs:
  - Hand-maintained ADR, server-module, system, cartridge, product, roadmap,
    owner-server, safety/recovery, and dedicated module-operator documentation
    describe the implemented slice and its limits.
  - OpenWiki update run `47e01057-f83b-42d9-b57d-61e63685b557`
    reconciled the server-module, quickstart, product-boundary, and validation
    pages; it corrected stage/database/QML counts, removed duplicate claims,
    and returned `status: complete` without warnings.
  - The completion receipt names pipeline
    `49248bf8-87d9-4cfe-886c-492133c4a89c`, tool
    `mcp__openwiki__openwiki_finish`, and gated state
    `600694fa6cc1fca6e0e395b75477ddf9e69621e54d14c902c940129d1399e80c`.
- AAR:
  - AAR-040 is submitted with twelve failures, eleven standing prevention
    rules, and the observation-only production-module decision. Every new ID
    is present in `docs/planning/knowledge/INDEX.md`.
- Archive:
  - Ticket 040 is closed, the open queue now begins with Ticket 041, and this
    spec/notes pair is archived. The final post-OpenWiki local gate, staged
    review, commit, push, and remote readback remain the authorized delivery
    phase.
- Phase 5 is PASS.

## Defect and lesson ledger

| # | What happened | Root cause | Fix | Prevention |
|---|---|---|---|---|
| 1 | A simulated second delivery dead-lettered instead of reconciling its immutable receipt. | The request receipt digest included the one-based delivery attempt, so legitimate retries had a different digest. | Hash a canonical clone with only `attempt` normalized to zero while delivering the real attempt to the host; retain a regression test that resets a delivered row. | Receipt identities must bind stable semantic facts and explicitly exclude transport-attempt metadata. |
| 2 | A degraded module could have reactivated on process restart without an operator recovery command. | Circuit degradation changed lifecycle but left `activation_allowed` true. | Degradation now atomically clears activation permission; the fault test proves startup is denied until expected-revision recovery. | Every circuit/restore stop state must carry a persistent activation gate, not rely only on lifecycle naming. |
| 3 | Core reauthorization trusted the stored pairwise partition captured at claim time without recomputing it from the authoritative report subject. | The dispatcher initially did not receive the pairwise derivation secret. | Pass a purpose-specific secret to core apply and compare a fresh derivation before any intent receipt/effect. | Re-derive opaque identifiers from authoritative roots at the protected sink. |
| 4 | Invalid host readiness could return without explicitly terminating the child. | `?` propagated readiness validation before cleanup. | Terminate and reap the process on readiness rejection; keep the hostile process drill. | Every post-spawn error edge must own explicit child cleanup. |
| 5 | One data operation UUID could be reused across different action kinds. | The audit uniqueness and replay lookup were scoped by action. | Make operation identity unique per instance and compare the stored action plus command digest on replay. | Idempotency UUIDs identify the whole command, never a method-local namespace. |

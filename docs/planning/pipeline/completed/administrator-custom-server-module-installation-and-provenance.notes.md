---
title: Administrator custom server-module installation and provenance — notes
pipeline_id: e07910b9-995b-4767-b464-c86ba883bd5a
---

# Administrator custom server-module installation and provenance — running notes

Chronological evidence and decisions. If a test or command did not run, these
notes must not claim it passed.

## Phase 1 — Plan

- Delivery baseline: Tickets 039 and 040 are committed and published on
  `origin/main`; local `HEAD`, the tracking ref, and GitHub readback matched
  `626f2264f64d9051df9c0e2ed2ed0dd3d4a3366e`, and the worktree was clean
  before this pipeline opened.
- Preflight: no active pipeline or blocking bulletin existed; CodeGraph 1.5.0
  and OpenWiki 0.3.3 passed `scripts/check-pipeline-tools.sh`; PostgreSQL was
  healthy; and the final Ticket 040 gate had completed before new work began.
- Sequence recall: Ticket 039 selected the process-isolated no-WASI Component
  Model boundary. Ticket 040 implemented one fixed reviewed observation
  component, durable delivery/state/lifecycle/restore machinery, and the
  local-only gate. Ticket 041 opens only the separately gated operator-custom
  custody, provenance, lifecycle, and disclosure path.
- Architecture recall: release integrity, provenance, core admission, and
  measured containment are independent. Components receive privacy-minimized
  typed snapshots, can propose only granted typed effects, and never receive
  database, server secrets, provider grants, egress, native execution, or
  client-code authority.
- Durable rules recalled from AAR-039/040 include out-of-band trust roots,
  server-bound custom provenance, artifact bounds during descriptor reads,
  absolute containment helpers, stable semantic receipt identity, persistent
  stop state, effect-sink identity derivation, owned child cleanup, whole-
  command UUIDs, fail-open optional observations, private command-file
  validation, retained request preimages, finalization reauthorization, and
  pre-start restore reconciliation.
- Product recall: owner-operated servers are independent trust domains. A
  custom decision changes provenance and player consent, not gameplay or
  trusted-client authority. Public disclosure must be aggregate, bounded,
  stable-server-bound, and explicit without becoming a module inventory or
  artifact distribution API.
- Planning decisions: component and signed-document bytes will be immutable
  PostgreSQL evidence; the core runtime will materialize only verified bytes
  into its own private temporary custody. Upgrade/rollback is one-step and
  snapshot-backed; removal is terminal but evidence-retaining. The initial
  hook/capability vocabulary does not expand beyond Ticket 040.
- Phase 1 is PASS.

## Phase 2 — Design

- Architecture and trust boundary:
  - The only new control plane is the database-local `omarchygs-admin
    custom-module-import` and `custom-module-apply` commands using an
    absolute, owner-private, non-symlink command file. No HTTP route, WebSocket
    message, discovery field, QML action, environment response, or cartridge
    gains module-import authority.
  - Import reads a strict
    `omarchygs.operator-custom-module-import-command/v1` descriptor and bounded
    owner-private files for the publisher-signed release, publisher public key,
    binary Wasm component, and operator-custom provenance signing key. Reads
    use the existing descriptor-safe open pattern: absolute paths, regular
    files, no symlink traversal, stable metadata before/after read, mode 0600
    for the command/private key, and independent byte ceilings before parsing.
  - The release signature proves publisher integrity only. The CLI verifies the
    exact release envelope against the supplied publisher public key, hashes
    the exact component bytes, checks the existing WIT/package/world/major,
    no-WASI Component Model shape, budget ceilings, requested capability/hook
    vocabulary, and then creates a separately signed
    `operator_custom` provenance statement bound to the database server UUID.
    The operator must repeat the publisher-key fingerprint, choose an explicit
    granted subset, and supply the exact acknowledgement
    `I understand this module is unreviewed and unsupported by OmarchyGS.`
  - Release integrity, operator trust, core admission, and runtime containment
    stay four independent proofs. Publisher and provenance public keys are
    immutable database evidence; their private keys are never stored. The core
    admission signer is read only by the local CLI/server process and is never
    written to PostgreSQL or a child request.
  - The runtime contract becomes provenance-neutral. A verified release bundle
    carries the exact signed release/provenance, out-of-band public keys,
    component bytes, and provenance class. Both reviewed and custom bundles
    traverse the same exact WIT, capability, state, dispatcher, receipt,
    resource, readiness, and sandbox code. Provenance affects verification and
    disclosure, never available hostcalls or effects.
  - Custom component bytes are retained immutably in PostgreSQL. For each host
    invocation the parent creates a mode-0600 private temporary artifact from
    the already verified bytes and read-only binds it at
    `/module/component.wasm` inside the existing systemd-user-scope,
    bubblewrap, and prlimit boundary. The host receives public trust roots as
    parent-controlled arguments, independently re-verifies the request and
    artifact, and exposes none of those arguments, server environment, paths,
    credentials, or networking to the guest.
  - The service supports at most eight operator-custom module identities. One
    stable instance exists per module ID, releases are immutable, and only one
    release/admission can be active for an instance. The fixed first-party
    fixture remains compatible and can coexist with custom instances.
  - Report creation queries the bounded set of active subscribed instances in
    deterministic instance-ID order inside the authoritative report
    transaction. It derives a module-scoped pairwise subject and appends a
    separate immutable event for each exact admission. A configured-but-
    unavailable runtime records `runtime_unconfigured`; per-instance queue
    saturation and inactive fixed-fixture behavior remain fail-open observation
    gaps. Report creation itself is never rolled back because an optional
    module cannot run.
  - The dispatcher claims any active exact instance/release/admission, loads
    its immutable stored trust/artifact material, independently rebuilds and
    verifies the request, launches one fresh contained child, and reauthorizes
    lifecycle, admission, report identity, pairwise subject, capability, and
    target revision again in the effect transaction. No stale admission can
    apply an effect.
- Database design (`0027_operator_custom_server_modules.sql`):
  - Drop only Ticket 040's fixture-only check constraints and replace them with
    exact generic bounds for identifiers, WIT identity, sorted allowlisted
    capability/hook arrays, budgets, schemas, and provenance-class shape.
    Existing first-party rows remain valid and immutable.
  - Extend releases with nullable reviewed/custom evidence columns:
    `component_bytes`, `artifact_custody`, publisher/provenance key IDs and
    public keys/fingerprints, and `provenance_server_id`. The packaged
    first-party fixture uses `packaged_reviewed_fixture` with null database
    component bytes; `operator_custom` requires all database-custody and
    server-binding fields. `review_id` becomes nullable only for custom
    provenance and remains required for reviewed provenance.
  - Extend instances with `previous_release_id`,
    `rollback_snapshot_id`, and `state_disposition`; retain one module-ID
    instance and exact current admission. State/schema constraints become
    bounded generic identifiers rather than Sentinel-specific values.
  - Add immutable `server_module_custom_operations`, keyed globally by
    operation UUID, containing action, canonical command digest, exact
    instance/release result, publisher/provenance fingerprints, requested and
    granted arrays, acknowledgement, actor/reason, and resulting revisions.
    Same UUID plus same digest returns the stored result; any body/action reuse
    conflicts. Existing lifecycle/data audits remain the detailed state trail.
  - Expand lifecycle audit actions for `import`, `upgrade`, `rollback`, and
    `remove`; expand gap reasons for `runtime_unconfigured`,
    `admission_replaced`, and `module_removed`; keep all release, admission,
    audit, receipt, snapshot, and custom-operation rows immutable and
    undeletable.
  - Terminalizing stale nonterminal outbox work is explicit evidence: upgrade,
    rollback, or removal moves affected rows to dead letter with the matching
    stable reason and increments the instance gap count. Nothing executes
    under an admission after its atomic replacement.
- Lifecycle, concurrency, and recovery:
  - `import` verifies everything before mutation, takes the module-registry
    advisory lock, rechecks the exact server identity and global operation UUID,
    inserts the immutable release and trust evidence, and creates a disabled
    instance plus empty namespace only for the module's first release. Later
    releases remain staged without changing the selected release.
  - `enable` accepts only a disabled or degraded selected release with matching
    lifecycle/config/state revisions and activation policy. It constructs the
    exact signed admission and candidate request, runs readiness outside the
    transaction, then takes the advisory lock and atomically rechecks every
    input before publishing the admission and active lifecycle.
  - `upgrade` requires an active/disabled instance, a staged release of the
    same module, exact expected lifecycle/state revisions, and a bounded
    explicit state candidate matching the target schema. Readiness runs against
    the candidate. Finalization atomically retains the old namespace snapshot,
    stores the immediate predecessor pointers, publishes the new namespace,
    release, and admission, and terminalizes old work.
  - `rollback` is available only for the retained immediate predecessor and
    snapshot. It performs the same readiness/CAS sequence, atomically restores
    the snapshot and predecessor admission, then clears the one-step rollback
    pointers so rollback cannot become an arbitrary downgrade graph.
  - `disable`, `suspend`, and `recover` preserve Ticket 040 semantics but accept
    exact instance identity and a whole-command digest. `remove` is terminal:
    lifecycle becomes `retired`, admission clears, state disposition is
    `retain_for_audit`, queued work is terminalized, and all artifact, state,
    snapshot, receipt, provenance, and audit evidence remains.
  - Restore starts core availability first, reconciles active custom rows
    against server identity and immutable evidence, marks restored instances
    pending operator review, and records observation gaps rather than executing
    uncertain code. The local recover command is the only route back to active.
- Public discovery and trusted-client contract:
  - When and only when at least one custom module is active/degraded, discovery
    adds sorted capability `server.operator-custom-modules.v1` and the exact
    top-level aggregate:

    ```json
    {
      "operator_custom_modules": {
        "format": "omarchygs.operator-custom-modules-disclosure/v1",
        "server_id": "<same stable server UUID>",
        "active_count": 1,
        "behavior_capabilities": ["moderation_labels"],
        "warning": "This server runs operator-custom code not reviewed or supported by OmarchyGS.",
        "support_boundary": "Security, privacy, availability, and support are the server operator's responsibility."
      }
    }
    ```

  - `active_count` is 1..8 and the behavior list is sorted, unique, and limited
    to public aggregate vocabulary. The response never includes module/release
    IDs, names, versions, component/config/state bytes, operator identity,
    filesystem paths, key material, signing authority, or private inventory.
  - `OnboardingController.qml` and `ServerProfiles.qml` exact-validate, copy,
    persist, and identity-bind the optional aggregate. `Main.qml` renders one
    persistent plain-text accessible warning bar between the brand and screen
    loader before and after sign-in; it remains contained at 640x420 and does
    not provide an acknowledgement bypass.
- API and compatibility:
  - Existing public API routes and request bodies are unchanged. Discovery is
    additively extended only when custom executable behavior is active; exact
    old documents remain valid. The fixed reviewed module selector and old
    lifecycle commands remain supported.
  - Runtime secret parsing is separated from the first-party selector. Both
    `OGS_MODULE_ADMISSION_SIGNING_SEED` and `OGS_MODULE_PAIRWISE_SECRET` enable
    the generic dispatcher; neither is valid alone. The optional
    `OGS_FIRST_PARTY_REPORT_MODULE=enabled` additionally registers/selects the
    packaged fixture and requires both secrets. With no secrets, core still
    starts with a gap-recording emitter and no module worker.
  - There is no marketplace module installer in this ticket. The generic
    verifier and shared conformance matrix accept a marketplace-reviewed trust
    class, but public marketplace approval/onboarding remains separately gated.
- File manifest:
  - Migration: `migrations/0027_operator_custom_server_modules.sql`.
  - Contracts/runtime: `crates/server-module-runtime/src/lib.rs`, its host
    binary/build fixtures, and runtime conformance tests.
  - Server/control plane: `crates/server/src/server_modules.rs`,
    `crates/server/src/config.rs`, `crates/server/src/main.rs`,
    `crates/server/src/server_discovery.rs`,
    `crates/server/src/bin/omarchygs-admin.rs`, report/app wiring, and focused
    PostgreSQL/config/API/admin tests.
  - Client: `client/qml/OnboardingController.qml`,
    `client/qml/ServerProfiles.qml`, `client/qml/Main.qml`, fixture server,
    onboarding/transport/accessibility tests, and real smoke expectations.
  - Operations/docs: custom-module operator runbook, server-module architecture
    and ADR, product charter/system overview/roadmap/API docs, local module and
    recovery drills, and Stage 24 production-boundary checks. No hosted CI file
    is introduced.
- Security and failure analysis:
  - Path substitution, symlink/TOCTOU reads, oversized artifacts, malformed or
    duplicate JSON, key confusion, forged publisher/provenance signatures,
    wrong-server provenance, digest mismatch, unknown WIT/imports/capabilities,
    request/grant escalation, signing-key persistence, stale admission effects,
    operation UUID body reuse, lifecycle races, migration failure, readiness
    failure, host crash/hang/resource exhaustion, queue saturation, restart,
    restore identity mismatch, disclosure omission/over-disclosure, hostile
    discovery, and small-window/accessibility regressions each receive a named
    negative test or drill.
  - Imported code is deliberately treated as hostile. It has no WASI, socket,
    filesystem, environment, clock, randomness, database, server-secret,
    provider, gameplay-authority, QML, JavaScript, or native-library surface.
    The only result is one typed intent that core may deny after reauthorization.
- Regression plan:
  - REQ-001: contract unit tests plus admin/PostgreSQL import tests for valid,
    replay, changed replay, descriptor/path/mode/symlink/race/size, signatures,
    keys/fingerprints, acknowledgement, server binding, WIT/component, sorted
    requested/granted subset, inventory ceiling, and immutable re-import.
  - REQ-002: PostgreSQL lifecycle matrix and concurrent CAS tests covering all
    states, readiness failure, upgrade migration commit/abort, stale work,
    immediate rollback once, terminal removal, crash boundaries, restart,
    backup/restore pending review, and recovery.
  - REQ-003: exact discovery API and QML fixture/transport/profile/accessibility
    tests for absent/present disclosure, stable identity, active-count bounds,
    hostile extra/private fields, persistence, warning visibility before/after
    login, keyboard behavior, and 640x420 containment.
  - REQ-004: one shared runtime/conformance matrix runs the reviewed fixture and
    operator-custom artifact through identical WIT, grants, budgets, sandbox,
    dispatcher, state, intent, and receipt assertions while checking distinct
    attestations/warnings.
  - REQ-005: route/source/client inventory, secret scan, forbidden-import and
    real bubblewrap/systemd containment drills, arbitrary-intent denial,
    admission replacement race, no-client-payload proof, and gameplay/provider
    authority regression suites.
  - REQ-006: operator-document audit, backup/restore drill, CodeGraph design and
    inspection receipts, Codex security diff inspection, OpenWiki lifecycle,
    focused commands, and final worktree-bound `bin/gate.sh --diff` receipt.
- Alternatives rejected:
  - Remote admin/upload APIs expose an unnecessary executable-ingress surface.
  - Operator-selected runtime paths permit post-review substitution.
  - Storing private trust keys in PostgreSQL or passing them to the host merges
    trust and execution boundaries.
  - In-process Wasm, native plugins, package QML/JavaScript, and per-module
    network access violate the selected containment/client authority model.
  - One hard-coded custom fixture would not satisfy portable owner-installed
    modules; unlimited instances would make same-transaction fan-out and
    recovery unbounded.
  - Deleting on removal destroys the evidence needed for audit and recovery;
    arbitrary rollback graphs multiply trust and migration ambiguity.
- CodeGraph design evidence: the final explore identified the report
  transaction emitter, dispatcher claim/request reconstruction, effect
  reauthorization, lifecycle command adapter, runtime verifier/supervisor,
  server configuration/startup, discovery builder, application state, and QML
  profile boundary as the implementation blast radius. It also found direct
  test coverage for `ModuleEmitter`, lifecycle commands, and service startup,
  while `claim_next_event`, `request_from_claim`, and `apply_host_response`
  require explicit new focused cases rather than relying on indirect coverage.
  The design receipt for pipeline
  `e07910b9-995b-4767-b464-c86ba883bd5a` matches gated state
  `22c029f91dabe05e47507201cdee1646afbc9249fee7660d7822bd651bbf83f3`.
- Phase 2 is PASS and the design is actionable.

## Phase 3 — Implement

- Built:
  - Migration 0027 generalizes the Ticket 040 registry without weakening its
    immutable evidence, adds database-custodied custom component/public-key
    material, server-bound provenance, rollback pointers/snapshots, bounded
    custom operation receipts, generic lifecycle actions, and explicit stale-
    admission gap reasons.
  - The local admin binary now accepts strict canonical owner-private import
    and lifecycle descriptors. Import independently verifies publisher
    integrity, provenance signing authority, fingerprints, stable server UUID,
    the exact component/WIT/budgets, requested/granted subsets, acknowledgement,
    and contained readiness. Identical operation replay is stable; changed
    intent conflicts; eight custom identities is the hard application ceiling.
  - Custom lifecycle supports enable, disable, suspend, recover, staged atomic
    upgrade, one-step snapshot rollback, and terminal evidence-retaining
    removal with all three mutable revision guards. Restore leaves retired
    instances terminal and returns every nonterminal module to explicit review.
  - Reviewed and operator-custom releases use one provenance-neutral runtime,
    generic dispatcher, fresh contained host process, no-WASI WIT, exact typed
    effect, state, receipt, retry/dead-letter, core reauthorization, and
    admission replacement rules. Database component materialization is private,
    bounded, and read-only inside the existing systemd/Bubblewrap/prlimit host.
  - Report transactions fan out deterministically across the bounded active
    module set. With runtime keys absent, core availability is preserved and a
    bounded `runtime_unconfigured` observation gap records the skipped custom
    execution.
  - Discovery emits only the server-bound custom-module count, public behavior
    class, warning, and support boundary while custom behavior is active or
    degraded. Trusted QML validates, identity-binds, persists, and continuously
    renders that aggregate without an acknowledgement bypass or private
    inventory.
  - The operator runbook, server-module architecture/ADR, API, product charter,
    system overview, owner-operated guide, README, roadmap, Constitution Stage
    24, and local conformance gates now describe and enforce the implemented
    custody, responsibility, disclosure, and no-public-admin boundaries.
  - Focused evidence is green: custom import/lifecycle/restore/concurrency/
    dispatcher PostgreSQL tests; actual custom CLI import/replay/enable; runtime
    conformance including real reviewed/custom containment; server-module
    script including Clippy/docs; QML policy and all 55 tests; server Clippy;
    and `bin/gate.sh --fast` before the final documentation/path hardening.
- Deviations:
  - The design's first draft abbreviated the import document name and described
    the custom verbs as a `modules` subcommand; the implemented stable formats
    are the explicit `custom-module-import` and `custom-module-apply` verbs now
    recorded above and in the runbook.
  - Private custody was tightened beyond final-component `O_NOFOLLOW`: both
    administrator descriptors and imported referenced files now use Linux
    `openat2` with `RESOLVE_NO_SYMLINKS|RESOLVE_NO_MAGICLINKS`, and custom
    command paths must be absolute. A nested-parent-symlink regression test
    proves the stronger design claim.
  - The latest path-hardening edit passed focused unit coverage, formatting,
    and server-wide deny-warnings Clippy; the complete local diff gate remains
    Phase 4 evidence and will be rerun after inspection fixes.

## Phase 3.5 — Inspect ledger

| # | Lens | Finding | Severity | Disposition |
|---|---|---|---|---|
| 1 | Private executable ingress | The first implementation rejected a symlink at the final artifact path but did not independently prevent a symlink in an ancestor directory. An attacker able to replace an operator-selected parent path could therefore race the privileged local import even though final inode metadata was checked. | Medium | Fixed before security finalization: all administrator descriptors and referenced custom artifacts now use Linux `openat2` with `RESOLVE_NO_SYMLINKS\|RESOLVE_NO_MAGICLINKS`, absolute paths, `O_NOFOLLOW`, owner/mode/link checks, and stable pre/post-read metadata. The nested-parent-symlink unit test and exact private-reader test pass. |
| 2 | CodeGraph test reachability | CodeGraph reported no direct test edge for the thin production wrappers `import_custom_module` and `apply_custom_lifecycle`. | Low | Accepted as an analyzer limitation, not an untested behavior: PostgreSQL tests exercise the injected-probe implementations containing the complete state machine, while `operator_cli` exercises the real production wrappers, environment configuration, contained probe, and CLI boundary. No duplicate wrapper-only test was added. |
| 3 | CodeGraph blast radius | The final graph reaches the report emitter, generic claim/rebuild/apply dispatcher, lifecycle finalizer, runtime supervisor/host, configuration/startup, discovery, and QML persistence/display surfaces. | Info | All reached surfaces were directly inspected and mapped to focused Rust, PostgreSQL, containment, CLI, discovery, and QML tests. No silent caller or alternate executable-ingress route was found. |
| 4 | Codex Security diff scan | Scan `01748081-1101-4b55-87ca-722cb46c217a` reviewed 17 workbench items plus manually accounted QML, scripts, migration, docs, planning, CLI tests, and untracked custom sources. No plausible candidate survived discovery. | Info | PASS with zero reportable findings and no deferred security work. Sealed report: `/tmp/codex-security-scans-gVuCBs/omarchy_bbs/626f2264f64d9051df9c0e2ed2ed0dd3d4a3366e_20260828T030138Z_s668lzvi/report.md`. TAC context was unavailable because its advisory connector was not configured; this did not reduce source coverage. |
| 5 | Trust and authority audit | Publisher integrity, operator-custom provenance, core admission, and process containment remain distinct proofs. Public discovery exposes only a bounded aggregate, and neither HTTP/QML nor game-provider/cartridge paths gained module administration or executable authority. | Info | PASS. Runtime re-verifies exact release/provenance/component material, effect apply reauthorizes current admission under locks, and source/gate checks reject public custom-module routes or client-delivered executable content. |

- Phase 3.5 is PASS. The fresh worktree-bound CodeGraph inspection receipt for
  pipeline `e07910b9-995b-4767-b464-c86ba883bd5a` matches gated state
  `e8509d8cb48fde3cb854deb275bb72df4c66b993b60fc011befd6bab26ae1752`.

## Phase 4 — Validate

- Tests run:
  - `./scripts/test-database.sh` passed all 82 purpose-specific PostgreSQL
    cases: 8 library module/custom-module tests, 63 complete server database
    tests, 5 administrator-domain tests, and 6 real database-local CLI tests.
  - `./scripts/test-qml-onboarding.sh` passed its 33-file policy check, all 55
    QML fixture tests, and the live QML smoke.
  - `./scripts/test-server-modules.sh` passed the shared reviewed/custom
    conformance suite and real systemd-user-scope, Bubblewrap, and prlimit
    containment/recovery proof.
  - Codex Security diff scan
    `01748081-1101-4b55-87ca-722cb46c217a` completed with zero reportable
    findings and no deferred security work.
- Gate run:
  - A fresh, uninterrupted `bin/gate.sh --diff` rerun on 2026-08-29 passed
    all 24 stages and printed `GATE GREEN [diff]`.
  - The gate covered formatting, deny-warnings Clippy, Rust tests and docs,
    Compose/script/pipeline/secret/hook/whitespace policy, cartridge/render/SDK
    proofs, two byte-identical native-client packages with SHA-256
    `d810b6db45e267189af3c81e9214b4355bb1fe7c59fa38c0866979d2f9028df6`,
    PostgreSQL/QML/provider/restore/invitation coverage, and server-module
    architecture plus production-boundary conformance.
- Skips or pre-existing failures:
  - No purpose-specific validation failed or was skipped. Tests ignored by the
    ordinary unit runner because they require PostgreSQL or real containment
    were executed by the dedicated gate stages instead.
  - Cargo emitted its existing informational notice that `chacha20` 0.10.1 is
    yanked while packaging; dependency resolution, compilation, tests, and both
    reproducible package builds still passed.
- Phase 4 is PASS. Phase 5 completion edits will intentionally invalidate this
  intermediate gate receipt; delivery requires another complete diff gate over
  the archived pipeline and finalized OpenWiki/AAR artifacts.

### Reboot checkpoint — 2026-08-27

- Durable status: Phase 3.5 is PASS and the active spec is ready for Phase 4.
  The implementation, inspection ledger, security review, operator/product/API
  documentation, and matching post-implementation CodeGraph receipt are
  present in the uncommitted worktree at baseline HEAD
  `626f2264f64d9051df9c0e2ed2ed0dd3d4a3366e`.
- Completed Phase 4 evidence before reboot:
  - `./scripts/test-database.sh` passed: 8 custom/module database tests, 63
    complete server database tests, 5 admin database tests, and 6 real
    database-local CLI tests, with zero failures.
  - `./scripts/test-qml-onboarding.sh` passed its policy check and all 55 QML
    tests, including hostile custom disclosure, persistence, accessibility,
    and compact-layout coverage.
  - `./scripts/test-server-modules.sh` passed the shared reviewed/custom
    conformance suite and the real systemd-user-scope + Bubblewrap + prlimit
    containment/recovery drill.
  - Codex Security diff scan
    `01748081-1101-4b55-87ca-722cb46c217a` is sealed with zero reportable
    findings and no deferred security work.
- Interrupted command: `bin/gate.sh --diff` was still running when the reboot
  was requested. Stages 1 through 15 had printed PASS. Stage 16 produced two
  byte-identical native-client packages with SHA-256
  `6db158e3813f18fa53fc5d7e4f611ef774fb998fbe713864b552aa0a0378aa2a`,
  but the overall command had **not** printed `GATE GREEN [diff]`. Therefore
  this partial run is not a Phase 4 pass and no receipt should be claimed from
  it.
- Exact resume action after reboot: confirm no prior Cargo/gate process remains,
  then rerun `bin/gate.sh --diff` from the beginning. After it prints
  `GATE GREEN [diff]` and the worktree receipt matches, record Phase 4 PASS and
  continue with the required OpenWiki update, acceptance-criteria audit, AAR
  and knowledge-register submission, ticket closure, pipeline archival, final
  local diff gate, and the already authorized commit/push/readback.
- Hosted GitHub CI/CD remains prohibited and absent; all quality evidence is
  local. No commit or push has been made for T041 yet.

## Phase 5 — Complete

- Acceptance-criteria audit:

  | Requirement | Disposition and evidence |
  |---|---|
  | REQ-001 | PASS — the private import/CLI corpus proves absolute owner-private descriptors, ancestor and final-symlink rejection, stable bounded reads, exact publisher signature/key fingerprint/component digest, WIT and no-WASI compatibility, explicit acknowledgement, sorted requested/granted subset review, server-bound provenance, immutable staging, actor/reason, eight-identity ceiling, and stable replay versus changed-body conflict. |
  | REQ-002 | PASS — PostgreSQL lifecycle, concurrency, CLI, and restore cases prove three-revision CAS, contained readiness, isolated candidate migration, atomic upgrade, retained immediate-predecessor rollback once, stale-admission terminalization, disable/suspend/recover, terminal evidence-retaining removal, immutable audit, restart policy, restore review, and no effect after admission replacement. |
  | REQ-003 | PASS — discovery API privacy cases and all 55 QML fixtures prove absent/present aggregate disclosure, exact stable server binding, count and behavior bounds, rejection of hostile or private fields, profile persistence, accessible warning visibility before and after sign-in, keyboard behavior, and 640x420 containment without component bytes or inventory. |
  | REQ-004 | PASS — the shared runtime and production conformance matrix runs reviewed and operator-custom artifacts through the same exact WIT, grants, budgets, no-WASI host, systemd/Bubblewrap/prlimit containment, dispatcher, state, typed intent, core reauthorization, and receipt rules while separately verifying their provenance and support claims. |
  | REQ-005 | PASS — source/route/client inventories, hostile contracts, arbitrary-intent denial, admission-replacement races, real containment, gate stages 23/24, and the zero-finding security diff scan prove no remote administration, server/database credentials, native/QML/JavaScript delivery, direct protected-state mutation, arbitrary hostcall/egress, or game-provider substitution. |
  | REQ-006 | PASS — operator terms, privacy/telemetry, security contact, patching, backup/restore, incident, and responsibility guidance are present; the focused evidence, matching CodeGraph inspection, zero-finding security scan, clean OpenWiki lifecycle, and all 24 local diff-gate stages passed. |

- Docs:
  - Hand-maintained ADR, server-module, system, API, product, roadmap,
    owner-operated-server, and dedicated operator documentation now describes
    the implemented custom custody/lifecycle/disclosure slice and its explicit
    support and authority limits.
  - OpenWiki update run `2cdea8d3-d402-434f-bdfe-0e11a52ac67d` completed
    cleanly after reconciling the server-module, runtime, quickstart, product-
    boundary, and validation pages in the preceding lifecycle.
  - The completion receipt names pipeline
    `e07910b9-995b-4767-b464-c86ba883bd5a`, tool
    `mcp__openwiki__openwiki_finish`, and gated state
    `8c92ddac11c0e6b6cb0eb2213865eb6c629ed926b0908474ab1609974dc2af94`.
- AAR:
  - AAR-041 is submitted with one captured failure, one standing prevention
    rule, and the operator-custom server-module boundary decision. Every new
    ID is present in `docs/planning/knowledge/INDEX.md`.
- Archive:
  - Ticket 041 is closed and this spec/notes pair is archived. The final
    post-completion `bin/gate.sh --diff` rerun passed all 24 stages and printed
    `GATE GREEN [diff]`; its worktree-bound receipt and the Ticket 041 OpenWiki
    completion receipt both match gated state
    `8c92ddac11c0e6b6cb0eb2213865eb6c629ed926b0908474ab1609974dc2af94`.
    Staged review, the authorized commit/push, and remote readback remain the
    delivery phase.
- Phase 5 is PASS.

## Defect and lesson ledger

| # | What happened | Root cause | Fix | Prevention |
|---|---|---|---|---|
| 1 | A private artifact path could pass final-inode checks while an ancestor directory was a symlink. | `O_NOFOLLOW` applies to the final component and does not reject a redirected parent chain. | Use Linux `openat2` with `RESOLVE_NO_SYMLINKS\|RESOLVE_NO_MAGICLINKS` for administrator descriptors and referenced artifacts, require absolute paths, and retain final no-follow plus descriptor checks. | Reject symlinked ancestors at the OS resolution boundary and keep the nested-parent-symlink regression. |
| 2 | The first OpenWiki finish did not issue a Ticket 041 receipt. | The durable spec still recorded Phase 3.5, so the completion hook correctly ignored the finish for the active pipeline. | Record Phase 4 PASS first, rerun the lifecycle, and read back the exact pipeline ID and gated-state hash. | Apply the existing `PR-omarchy-gaming-system-advance-durable-phase-before-phase-tools-001` at every Phase 4/5 transition. |

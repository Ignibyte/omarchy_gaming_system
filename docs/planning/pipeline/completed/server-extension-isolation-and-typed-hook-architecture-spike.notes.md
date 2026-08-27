---
title: Server extension isolation and typed-hook architecture spike — notes
pipeline_id: 144295f2-9300-4fcc-96e0-2e25d910f99e
---

# Server extension isolation and typed-hook architecture spike — running notes

Chronological evidence and decisions. If a test or command did not run, these
notes must not claim it passed.

## Phase 1 — Plan

- Delivery baseline: Ticket 038 is closed at commit
  `85a9a070c96b576c01d8208511b27ec9cfb74801`; local `HEAD`, fetched
  `origin/main`, and GitHub `main` matched, and the worktree was clean before
  Ticket 039 opened.
- Preflight: no active pipeline or open ticket existed; no active bulletin
  blocked work; CodeGraph 1.5.0 and OpenWiki 0.3.3 passed
  `scripts/check-pipeline-tools.sh`; PostgreSQL was healthy; and no Cargo
  process was running.
- Pipeline validation initially failed because Ticket 038's otherwise complete
  AAR used the ad hoc state `complete` instead of the validator's established
  terminal state `submitted`. Corrected that planning-only metadata before
  design and retained the failure in this pipeline's lesson ledger.
- Roadmap recall: the operator-custom inert-cartridge path is implemented by
  Ticket 038. The next owner-operated ecosystem gate is the server extension
  isolation spike, followed by the production module base and only then
  administrator-controlled executable module installation.
- Architecture recall: ADR-0003 defines modules as a third extension family.
  Game Cartridges remain inert frontend data, portable game rules use the
  brokered Provider SDK, and a general module may only observe allowlisted
  events or submit typed intents through core authorization.
- Current-state recall: `GameRegistry` is an in-process first-party compiled
  game seam; `ProviderRuntime`/`ProviderBroker` are an optional exact-release
  remote game-authority seam; the server has no general module manifest,
  registry, hook dispatcher, configuration namespace, state store, or loader.
- Security recall: modules may affect the confidentiality, integrity,
  availability, moderation, and correctness of an owner-operated server, but
  they must not receive account ownership, reusable player credentials,
  unrestricted PostgreSQL access, arbitrary client execution, or a shortcut
  around provider identity/replay/lifecycle controls.
- Recalled rules:
  `PR-omarchy-gaming-system-bind-provider-messages-to-registered-identity-001`,
  `PR-omarchy-gaming-system-enforce-bounds-during-streaming-001`,
  `PR-omarchy-gaming-system-gate-every-nested-workspace-proof-001`,
  `PR-omarchy-gaming-system-gate-independent-executable-source-trees-001`,
  `PR-omarchy-gaming-system-serialize-dedupe-on-existing-root-001`, and
  `PR-omarchy-gaming-system-separate-publisher-marketplace-server-attestations-001`.
- Local environment recall: no `wasmtime`, `wasmer`, `wasm-tools`, or OCI
  runtime is installed; Bubblewrap and systemd-run are present. Tool presence
  is evidence about current operator friction, not a reason to preselect the
  architecture. The proof must install/pin or isolate any runtime it needs and
  put every executable source and dependency decision inside the gated state.
- Phase 1 decision: run one decision-and-proof slice. It will compare external
  process RPC, Wasm, statically linked modules, and justified hybrids; define
  the complete contract; exercise the selected trust units under hostile
  conditions; and leave production loading absent. Provider SDK publication
  and production module implementation remain separate tickets.
- Phase 1 is PASS.

## Phase 2 — Design

- Current topology and insertion points:
  - CodeGraph traced the compiled `GameRegistry`, optional
    `ProviderRuntime`/`ProviderBroker`, `AppState`/router construction,
    `SyncEventKind`/`append_event`, PostgreSQL notification listener, social,
    inbox, challenge, game, report, and operator audit paths. The graph found
    no general module registry, hook dispatcher, module host, loader, or
    production configuration surface.
  - Domain transactions already own authorization and durable mutations.
    `sync::append_event` is called from eight domain paths inside those
    transactions, while `SyncHub` only wakes clients after PostgreSQL commit.
    The module system must add its own durable domain-event outbox beside the
    authoritative mutation, not reinterpret WebSocket notifications or call
    transport handlers.
  - Compiled games remain reviewed core code and registered providers remain
    the only portable game-rules authority. Module events may observe game
    envelopes or public projections but cannot invoke `GameDefinition`,
    register `ProviderRuntime`, edit provider receipts, or become a second
    rules owner.
- Primary-source/runtime evidence:
  - The Rust Reference states that the Rust ABI has no stability guarantees;
    a dynamic Rust trait/dylib boundary would also share the server's address
    space and panic/allocator/runtime assumptions. It is not an operator
    extension ABI.
  - The WebAssembly Component Model defines WIT as a language-neutral typed
    interface contract and the Canonical ABI validates component boundaries.
    WIT packages carry semantic versions and feature gates, but current
    composition tooling is still described as early and WASI 0.3/native async
    is a June 2026 developer-preview generation. OmarchyGS therefore pins an
    exact supported interface major plus WIT SHA-256 and does not infer safety
    from permissive minor-version linking.
  - Wasmtime's security guide confirms that guest code has no syscall or I/O
    access except explicit imports. Version 48.0.1 supports Component Model,
    typed bindings, store resource limits, fuel, and epoch interruption on the
    repository's Rust 1.98 toolchain. Its 2026 security advisories also include
    sandbox escape, host resource exhaustion, and component transcoding
    defects. The runtime is a maintained security boundary, not a reason to
    place untrusted components in the core server process.
  - Bubblewrap 0.11.2 and systemd 261 are installed locally. A minimal
    Bubblewrap network/filesystem namespace ran successfully, and a transient
    user systemd scope with `MemoryMax` and `TasksMax` ran successfully.
    Bubblewrap's own documentation says it constructs rather than defines a
    complete sandbox; the product contract must own the exact policy.
  - No Wasmtime CLI/runtime was preinstalled. The selected host embeds pinned
    Wasmtime 48.0.1 instead of trusting an ambient binary. `cargo info`
    confirmed Rust 1.95 minimum support. The module host remains versioned and
    patched with OmarchyGS even though module artifacts release independently.

### Isolation decision matrix

| Model | Containment and authority | Compatibility/portability | Lifecycle and operations | Decision |
|---|---|---|---|---|
| Dynamic in-process Rust library | No meaningful compromise/crash boundary; unstable Rust ABI and shared allocator/runtime | Rust/toolchain/target coupled | Hot replacement and safe unload are not credible | Rejected by ADR-0003 and this spike. |
| Statically compiled reviewed module | Rust type safety and review, but full core process authority | Stable only through a complete OmarchyGS rebuild | Excellent startup simplicity; no independent operator install/rollback | Retain for first-party core features, not the module ecosystem. |
| Native external-process RPC | Strong crash/address-space separation when paired with a complete OS sandbox; native code still reaches every syscall allowed by that sandbox | Language-neutral protocol but per-architecture packaging and parser burden | Independent health/restart/upgrade is natural; sandbox profiles vary by host | Valid specialized deployment boundary, but too much ambient-native surface for the baseline artifact. |
| In-process Wasm component | WIT type checking, linear-memory isolation, no ambient I/O, fuel/memory limits | Strong cross-language artifact contract | Simple embedding, but runtime escape or abort reaches the core server | Rejected as the sole containment boundary. |
| Native OCI/container module | Process/kernel isolation and established deployment tools | Per-architecture images; protocol remains language neutral | Highest image, daemon, policy, patch, and operator burden | Optional future deployment wrapper, not the portable module contract. |
| **Dedicated process hosting one no-WASI Wasm component release** | WIT/Wasmtime contains guest behavior; systemd/cgroup/filesystem/network/process policy contains runtime compromise; core sees only bounded RPC | Portable component plus exact WIT major/digest; host is a normal OmarchyGS native package | Per-release health, restart, disable, upgrade, rollback, resource telemetry, and patching are explicit | **Selected baseline.** Defense in depth is worth the extra host process because modules are operator-installed executable code. |

### Selected architecture and data flow

1. A future admin command snapshots one bounded package and verifies separate
   publisher integrity, optional marketplace-review or operator-custom
   provenance, component SHA-256, canonical manifest, supported WIT identity,
   and requested configuration/state schemas. Those facts do not grant a
   capability.
2. An exact `ModuleAdmissionV1` separately binds the stable server UUID,
   module/release identity, component and WIT digests, granted capability
   subset, subscribed hook subset, per-hook failure class, resource budgets,
   configuration revision, state schema, and lifecycle revision. Expected
   state and idempotency are mandatory for every future admin transition.
3. One exact release runs in one `omarchygs-module-host` process. The production
   deployment uses a dedicated systemd service identity and cgroup with
   `NoNewPrivileges`, private network/devices/tmp, protected system/home,
   `AF_UNIX` only, bounded memory/CPU/tasks/files, read-only exact component,
   and no server environment or PostgreSQL credential. The spike uses
   Bubblewrap plus a transient user scope to exercise the same trust units.
4. The host embeds pinned Wasmtime, enables Component Model, disables threads,
   memory64, and every WASI interface, and links no guest import. It verifies
   artifact bytes before compilation, never deserializes a publisher-supplied
   native/AOT cache, and creates a fresh limited Store/instance per invocation.
   Fuel bounds deterministic guest work; an outer RPC deadline and cgroup
   remain independent containment.
5. Core domain transactions append an immutable `ModuleHookEventV1` to a
   durable outbox with the protected mutation. Events contain a pairwise or
   public domain subject, exact event/revision/idempotency identity, bounded
   allowlisted data, an immutable configuration snapshot, and only the
   module's namespaced state projection. They contain no account ownership,
   session token, credential, raw private message beyond an explicitly granted
   content hook, database row, arbitrary URL, or filesystem path.
6. A dispatcher partitions events by `(module release, hook, subject)` and
   preserves sequence within that partition; different partitions may run
   concurrently. Delivery is at least once. Exact replay returns the original
   receipt, changed replay conflicts, and bounded retry/backoff/dead-letter
   state is durable. Queue saturation cannot grow memory without bound.
7. The component's one WIT export receives a typed event record and returns a
   bounded typed intent list. It has no clock, randomness, filesystem, socket,
   process, environment, or state hostcall. Configuration/state are input
   snapshots; state compare-and-set, moderation annotations, notifications,
   and allowlisted integration delivery are typed proposed intents.
8. The module host authenticates its exact admission/release context and the
   core revalidates every returned intent against the current grant, event,
   target, expected revision, lifecycle, and policy. Core services alone
   authorize and transactionally commit an intent plus immutable receipt.
   Modules cannot call a transport handler or write protected tables.
9. Observation hooks run only after the original core commit and are fail-open
   with respect to that completed operation; missed work remains in the durable
   outbox. Admission hooks operate on an immutable pre-commit snapshot outside
   any database transaction, may be configured required/fail-closed only by
   explicit operator admission, and must re-lock/revalidate current state
   before core commit. The first production slice should enable observation
   hooks before any admission hook.
10. State belongs to a core-owned namespace keyed by stable server, module, and
    state schema. Values and total bytes are bounded; revisions are
    compare-and-set. Upgrade stages a copy, applies forward migrations in an
    isolated candidate namespace, proves readiness, then atomically changes
    admission. Rollback restores the retained pre-upgrade namespace snapshot;
    uninstall retains audit/tombstone and requires explicit data disposition.

### Contracts and compatibility

- `omarchygs.server-module-release/v1` is canonical JSON with strict fields:
  immutable module/publisher/release/semantic version, component SHA-256,
  exact WIT package/world/major/SHA-256, requested sorted-unique hooks and
  capabilities, proof-bounded budgets, configuration and state schema
  identities, entrypoint, and provenance-reference identity. Publisher
  signature, marketplace/operator provenance, server admission, capability
  grant, and runtime containment remain separately verified facts.
- `omarchygs.server-module-hook/v1` binds event UUID, delivery attempt,
  server/module/release/admission identities, hook kind/version, subject,
  causal platform revision, deadline, bounded configuration/state revisions,
  and exact payload. The local RPC frame is a four-byte big-endian length plus
  canonical JSON with a 64 KiB proof ceiling enforced before allocation.
- `omarchygs.server-module-intents/v1` returns zero or more fixed intent
  variants. Each binds the source event, ordinal, capability, target, expected
  revision, and bounded arguments. The derived receipt identity is
  `(module release, event UUID, ordinal)`; the component does not choose an
  arbitrary idempotency key.
- WIT v1 uses records/enums/variants rather than JSON inside the Wasm boundary.
  The proof's first exact world exposes one typed record input and one typed
  record output. Production additions are new versioned hooks/capabilities;
  incompatible type changes require a new major. Host and artifact must match
  the supported major and exact WIT digest declared by admission.
- Marketplace-vetted and operator-custom module provenance use the same
  component and conformance contract but different attestations/warnings.
  Neither provenance class changes requested/granted capabilities or sandbox
  policy. A future player discovery flag may disclose custom server behavior;
  it never transfers executable client code or module inventory secrets.

### Lifecycle, failure, and recovery

- Lifecycle is `staged → disabled → enabling → active → degraded/suspended →
  disabled → retired`, with immutable release bytes and monotonic admission
  revision. Retirement is terminal. A candidate upgrade cannot receive live
  events before readiness and state migration both succeed.
- Hook definitions classify timeout/trap/malformed/host-unavailable outcomes.
  Observation hooks preserve the original commit, record failure, and retry
  within bounded policy. Required admission hooks deny the pending operation
  with a stable generic error. Optional admission hooks continue only when the
  operator admission explicitly says fail-open; this decision is never chosen
  by the component response.
- Repeated traps/timeouts/crashes trip a core-owned circuit breaker to
  `degraded`; fresh deliveries pause but durable events remain. Disable and
  suspension stop new work before process termination. Recovery re-verifies
  exact bytes/admission/state and reconciles receipts/outbox before active.
- Backups contain manifest/provenance/admission/audit/outbox/receipt and
  namespaced state, never an executable process image or JIT cache. Restore
  starts every module disabled until exact artifacts and host compatibility
  are reverified; no module is silently activated from database state alone.

### Proof budgets and hostile cases

- Proof ceilings, measured rather than claimed as final production defaults:
  1 MiB component/manifest input, 64 KiB framed request/response, 4 MiB linear
  memory, one instance/memory/table, 100,000 fuel, 500 ms outer deadline,
  256 MiB host cgroup memory, 50% of one CPU, 16 tasks, and 64 file descriptors.
- The selected-model proof will cover: valid typed event → one allowlisted
  core-authorized intent; no-op; exact replay; changed replay; unknown hook;
  undeclared capability; forged module/release/admission context; manifest and
  component tamper; protocol/WIT mismatch; malformed/oversized frame and
  component; memory request denial; infinite loop/fuel trap; component trap;
  host hang/outer kill; host crash; restart; configuration/state stale write,
  quota, migration failure, activation, upgrade, rollback, disable, retirement,
  and backup/restore; no WASI import; loopback-only network namespace; absent
  home/server environment; and unchanged production route/config inventory.

### File manifest

| Path | Purpose |
|---|---|
| `crates/server-module-spike/Cargo.toml`, `Cargo.lock`, `README.md` | Isolated pinned proof workspace and decision limits; never linked by the production workspace. |
| `crates/server-module-spike/wit/omarchygs-module.wit` | Exact language-neutral typed proof world and version identity. |
| `crates/server-module-spike/src/lib.rs` | Canonical release/provenance/admission/hook/intent contracts, signatures, validation, framing, core authorization, state/lifecycle/replay model, and stable proof errors. |
| `crates/server-module-spike/src/bin/module-host.rs` | Separate Wasmtime component host with no WASI/imports, exact byte verification, Store/fuel limits, and bounded single-request RPC. |
| `crates/server-module-spike/src/bin/supervisor.rs` | Core-side proof supervisor that launches the host under systemd/Bubblewrap, applies an outer deadline, authenticates the response, and commits only allowlisted proof intents. |
| `crates/server-module-spike/fixtures/components/*.wat`, `fixtures/*.json` | Valid, no-op, unauthorized, trap, loop, memory, import, and compatibility fixtures with no native executable payload. |
| `crates/server-module-spike/tests/*.rs` | Canonical/hostile contracts, signatures/provenance separation, state/lifecycle/recovery, framing, selected runtime, and supervisor containment evidence. |
| `scripts/test-server-module-spike.sh`, `bin/gate.sh`, `.gitignore` | Sequential isolated proof build/test, sandbox cross-process smoke/metrics, canonical gate stage, and ignored build products. |
| `docs/architecture/adr-0004-process-isolated-wasm-server-modules.md` | Accepted isolation decision, alternatives, risks, and implementation authorization boundary. |
| `docs/architecture/server-modules.md` | Manifest, typed hooks/intents, capabilities, state, lifecycle, failure, audit, compatibility, sandbox, operations, and rollout design. |
| `docs/architecture/system-overview.md`, `docs/architecture/game-cartridges.md`, `docs/operators/owner-operated-servers.md`, `docs/planning/ROADMAP.md` | Reconcile selected future direction, extension-family split, operator responsibility, completed spike, and sequenced implementation. |
| `.github/workflows/ci.yml`, `scripts/check-local-only-automation.sh`, `scripts/check-pipeline.sh`, `AGENTS.md`, `CONSTITUTION.md`, `README.md`, `docs/architecture/adr-0001-agent-work-pipeline.md` | Remove GitHub Actions, reject hosted automation definitions locally, and make local gate receipts the sole delivery-quality evidence. |
| Ticket/spec/notes/AAR/knowledge/OpenWiki | Evidence, durable lessons, generated navigation, acceptance audit, and archive. |

### Requirement-to-evidence map

| Requirement | Evidence |
|---|---|
| REQ-001 | CodeGraph topology plus authority matrix and production-loader absence checks. |
| REQ-002 | Primary-source decision matrix, local tool/runtime checks, selected proof startup/RSS/latency/fuel metrics, and alternatives in ADR-0004. |
| REQ-003 | Strict canonical release/provenance/admission types and malformed/duplicate/order/digest/version tests. |
| REQ-004 | WIT records, exact capability grant/intent enums, sensitive-field/import absence tests, and core authorizer tests. |
| REQ-005 | Partition/order/replay state model plus timeout, trap, crash, saturation, malformed, unauthorized, and fail-policy tests. |
| REQ-006 | Namespaced revisioned state/config model with quota, stale/CAS, migration, rollback, removal, and serialized recovery tests. |
| REQ-007 | Lifecycle state machine, exact admin-operation receipts/audit model, concurrent transition tests, readiness and player-provenance review. |
| REQ-008 | Systemd/Bubblewrap process-hosted Wasmtime smoke, hostile WAT fixtures, sandbox probes, and production source/config/route absence checks. |
| REQ-009 | Exact WIT identity/version matrix and both provenance fixture classes through one conformance corpus. |
| REQ-010 | ADR/docs/follow-up tickets, security and CodeGraph inspection, OpenWiki lifecycle, gate stage, and final `bin/gate.sh --diff`. |
| REQ-011 | Disabled GitHub workflow readback, deleted workflow definition, local-only checker positive/hostile cases, residual audit, and local diff gate. |

### Risks and rejected shortcuts

- A Wasm sandbox escape is plausible security debt, not an impossible event;
  one release per separate OS service prevents the runtime from sharing core or
  another module's address space and requires prompt Wasmtime patching.
- Process separation without filesystem/network/cgroup policy is insufficient;
  the service policy and no-WASI/no-import linker are mandatory, while
  Bubblewrap remains only the portable proof implementation.
- Component compilation itself is hostile work. Size/digest checks precede it,
  compilation happens inside the bounded host, and only trusted host-generated
  caches may be reused. Unsafe deserialization of publisher AOT/native bytes is
  prohibited.
- Durable events cannot be emitted only after commit from best-effort memory;
  the future implementation must use a transactional outbox and bound retained
  work. WebSocket/cursor events remain player synchronization, not a module
  queue.
- Synchronous module calls inside database transactions were rejected because
  timeouts would hold locks and make external failure part of transaction
  liveness. Admission snapshots are evaluated outside and revalidated under
  the eventual core transaction.
- A generic `serde_json::Value` hook or arbitrary SQL/HTTP hostcall was rejected.
  New power requires a named versioned event/intent/capability and core-owned
  destination or target policy.
- Reusing the provider broker was rejected: providers own one game's rules and
  speak a signed network protocol; modules observe general server domains and
  never become gameplay authority.
- Running many modules in one Wasmtime process was rejected for the baseline
  because one runtime escape or abort would cross module trust/provenance
  domains and make independent lifecycle/resource attribution unreliable.

- User-directed scope amendment on 2026-08-27: remove and disable GitHub CI/CD
  and require equivalent quality enforcement to remain local. Because Ticket
  039 was already the sole active pipeline, the requirement was added here
  instead of opening an impermissible second active spec. GitHub readback
  showed the only workflow, `CI`, had no active run and was
  `disabled_manually` before the committed definition was removed. Direct
  shell/document review is authoritative for these CodeGraph-unsupported
  workflow surfaces.

- Phase 2 is PASS. The design is actionable and the worktree-bound CodeGraph
  design exploration covers the real startup/domain/sync/provider seams for
  pipeline `144295f2-9300-4fcc-96e0-2e25d910f99e`.

## Phase 3 — Implement

- Added the isolated `crates/server-module-spike` workspace without linking it
  into the production workspace. It pins Wasmtime 48.0.1 and Component Model
  tooling, carries the exact versioned WIT source, and has no production
  server, route, migration, Compose, or client integration.
- Implemented strict bounded canonical release, provenance, admission, event,
  response, and state/lifecycle contracts. Publisher, provenance authority,
  and core signatures are separately verified against host-provisioned proof
  authorities; their public keys cannot be self-selected by the request.
  Admission binds exact component/WIT/release/provenance digests and grants
  only sorted hook/capability/budget subsets.
- Implemented a completed-binary-only Component Model runtime with no linker
  import/WASI, fresh limited Store/instance per call, 4 MiB linear memory,
  100,000 fuel, stable rejection codes, typed record input/output, and host-
  plus-core intent checks. The proof core binds receipts to exact release and
  event, returns exact replay, conflicts changed replay, and alone commits the
  bounded moderation-label fixture intent.
- Implemented bounded canonical four-byte framed RPC, a dedicated one-request
  module host, and a supervisor that starts it through systemd user-scope
  memory/CPU/task limits, inherited file-descriptor `prlimit`, and Bubblewrap
  user/process/network/filesystem/capability/environment containment. Startup,
  execution, and post-response exit all have independent outer deadlines.
  Host readiness measures its own RSS and proves absent home/password file,
  server environment, and non-loopback network interface.
- Implemented deterministic WIT-driven fixture componentization. Checked-in
  inert core WAT is converted twice to byte-identical completed components for
  valid/no-op/unauthorized/trap/loop/memory cases; raw hostile components cover
  forbidden imports and a wrong root interface. The runtime never performs
  this conversion for publisher input.
- Implemented contract/runtime/state/lifecycle tests plus the end-to-end
  `scripts/test-server-module-spike.sh` gate. The suite covers strict/unknown/
  duplicate/downgrade/digest/signature/trust-root cases, sensitive-field
  absence, frame bounds, typed intent/no-op, exact and changed replay, queue
  order/backpressure, namespace CAS/quota/backup/restore, atomic migration and
  rollback, lifecycle activation/replay/retirement, capability denial, tamper,
  wrong interface, forbidden import, memory, trap, loop/fuel, process exit,
  timeout, and clean restart.
- The first full `cargo test --manifest-path
  crates/server-module-spike/Cargo.toml` compiled every target. Eight contract
  tests passed, while all five runtime tests failed because the hand-written
  components directly exported named record types without the Component Model
  type-export shim. Replaced that shortcut with deterministic
  `wit-component` metadata/encoding. The next full focused run passed 17 tests
  and rustdoc tests.
- The first real systemd/Bubblewrap smoke rejected `LimitNOFILE` as a transient
  user-scope property. Moved that independently enforced ceiling to
  `/usr/bin/prlimit` outside Bubblewrap; systemd retains memory/CPU/task
  ceilings. The valid process smoke then committed one allowlisted intent with
  startup under 100 ms and execution under 20 ms.
- Every planned process scenario passed individually under
  `systemd-user-scope+bubblewrap`: valid, no-op, unauthorized, trap, loop,
  memory-hog, forbidden-import, wrong-interface, component tamper, forged
  context, host exit, host hang, and restart. The 500 ms outer deadline killed
  the intentional hang at 501 ms. An early supervisor metric observed the
  `systemd-run` launcher rather than the contained host; replaced it with the
  host's own `/proc/self/status` readiness measurement before final evidence.
- `scripts/test-server-module-spike.sh` passed formatting, Clippy, 17 tests,
  binary build, warnings-denied rustdoc, deterministic component builds, every
  process scenario, current local-only automation policy, a hostile GitHub
  Actions reintroduction fixture, and production-loader absence. Subsequent
  host-RSS/trust-root/exit-deadline hardening intentionally invalidated that
  focused result; a fresh run remains required before Phase 3 exits.
- Removed `.github/workflows/ci.yml`, remotely disabled the only GitHub
  workflow, and added local enforcement that rejects GitHub Actions plus
  common equivalent hosted automation definitions. `bin/gate.sh --diff` and
  its worktree receipt remain the sole delivery-quality proof.
- Added ADR-0004 and `docs/architecture/server-modules.md`; reconciled system,
  cartridge, operator, roadmap, README, Constitution, and workflow guidance.
  Opened sequenced Ticket 040 for the observation-only production base and
  Ticket 041 for later administrator-custom module installation/provenance.
- File-manifest deviation: added `src/bin/fixture-builder.rs`. The initially
  planned static component fixtures could not correctly hand-maintain named
  WIT type-export shims; the small deterministic builder makes the exact WIT
  source load-bearing, emits only ignored proof artifacts, and is itself
  formatted, linted, tested, built, documented, and gated.
- Fresh hardened focused evidence:
  `./scripts/test-server-module-spike.sh` passed formatting, warnings-denied
  Clippy, 19 tests, all binaries, warnings-denied rustdoc, two byte-identical
  component builds, all 13 separate-process scenarios including restart,
  production-loader absence, and positive/hostile local-only automation
  checks. A subsequent valid measurement under
  `systemd-user-scope+bubblewrap` reported 24 ms startup, 17 ms execution, and
  29,892 KiB host-self RSS; the intentional host hang was killed at 500 ms and
  a fresh host succeeded afterward. These are proof measurements, not
  production service objectives.
- `scripts/check-pipeline.sh` and `git diff --check` passed after the final
  Phase 3 changes.
- Phase 3.5 security discovery reviewed every changed executable/interface
  source row plus the deleted workflow. It found four proof-only hardening
  gaps: operator-custom provenance was not bound to the admission server,
  component files were allocated before an exact read-time bound, popped queue
  partitions remained allocated, and systemd helper selection used `PATH`.
  None reached production because the loader remains absent, but all four were
  corrected before accepting the architecture evidence.
- Added a single-handle `MAX+1` artifact reader, exact operator provenance
  server binding and non-nil identity checks, empty-partition pruning, and
  absolute `/usr/bin/systemctl`/`systemd-run` selection. Added hostile contract
  and state regressions. The refreshed focused proof passed formatting,
  warnings-denied Clippy, **21 tests**, all binaries, warnings-denied rustdoc,
  deterministic fixtures, all process scenarios, production-loader absence,
  and local-only automation checks.
- Phase 3 is PASS. The implementation and focused evidence are ready for
  independent inspection.

## Phase 3.5 — Inspect ledger

- The initial exact-snapshot security review covered all 21 changed executable
  and interface rows plus supporting workflow, lockfile, and documentation
  evidence. Its four non-production candidates were resolved and
  regression-tested before the final scan.
- The corrected exact-snapshot Codex Security scan was finalized and validated
  at
  `/tmp/codex-security-scans/omarchy_bbs/85a9a07_worktree_20260827T212953Z`.
  It reports zero validated findings and complete changed-source coverage for
  snapshot
  `codex-security-snapshot/v1:sha256:85838401d096af9cad1539b9c9753f68d8fdba8428e56992dc0ba07e8ff8bab0`.
  TAC connector status could not be verified because no TAC connection is
  configured; the repository scan contract and local artifacts were still
  completed and validated.
- Final CodeGraph inspection re-traced the bounded artifact reader, provenance
  admission binding, Wasmtime runtime, proof-core reauthorization, partitioned
  dispatch, separate host/supervisor boundary, local-only automation checker,
  gate integration, production-loader absence, callers, and blast radius.
  The worktree-bound inspection receipt matches this pipeline state.
- Phase 3.5 is PASS with no unresolved correctness, security, containment,
  lifecycle, operations, or simplification findings.

## Phase 4 — Validate

- The first full `bin/gate.sh --diff` exercised every local stage. Twenty-one
  substantive stages passed, including PostgreSQL, QML, provider, backup,
  admission, and the new module-isolation proof. It correctly remained RED
  because `git diff --check` found extra terminal blank lines in two new
  WIT/WAT files and the native-client source-fixture copy included the nested
  spike `target/` tree, exhausting the temporary-directory quota.
- Removed the two whitespace errors and changed the package-source fixture to
  stream only source trees through `bsdtar` while excluding every nested
  `target/` path before extraction. Archive-list inspection proved the
  exclusion, `git diff --check` passed, and a focused
  `./scripts/test-client-package.sh` rerun built two byte-identical native
  packages and passed its hostile source cases without copying build products.
- The fresh complete rerun passed every local stage and printed
  `GATE GREEN [diff]`. It covered Rust formatting/lints/tests/docs, Compose and
  shell contracts, pipeline/local-only/secret/hook/whitespace checks, cartridge
  and marketplace suites, two byte-identical native client packages,
  PostgreSQL integration, live QML/API smoke, remote-provider security and
  clean-clone authority, backup/restore, invite-only admission, and the 21-test
  process-isolated server-module proof.
- Phase 4 is PASS. A matching local worktree gate receipt existed at the end of
  validation; Phase 5 evidence changes will require one final complete rerun.

## Phase 5 — Complete

- Acceptance-criteria audit:

  | Requirement | Disposition and evidence |
  |---|---|
  | REQ-001 | PASS — CodeGraph and direct source inspection distinguish cartridges, compiled games, registered providers, and general modules and preserve every core authority boundary. |
  | REQ-002 | PASS — the decision matrix compares dynamic/static Rust, native RPC, in-process Wasm, OCI, and the selected process/Wasm hybrid using primary-source and measured local evidence. |
  | REQ-003 | PASS — strict canonical release, provenance, and admission documents bind identity, exact WIT/component digests, requested/granted subsets, budgets, schemas, and lifecycle. |
  | REQ-004 | PASS — exact WIT records and typed intents expose only bounded allowlisted data; sensitive fields, direct mutations, arbitrary destinations, and forbidden imports are rejected. |
  | REQ-005 | PASS — deterministic replay/conflict, partition ordering, bounded queues, timeout/trap/crash/malformed outcomes, retry policy, and core-owned commit receipts are designed and exercised. |
  | REQ-006 | PASS — core-owned revisioned namespaces prove quota, CAS, migration failure atomicity, rollback, removal, backup, restore, and disabled-on-restore semantics. |
  | REQ-007 | PASS — exact expected-state lifecycle operations, immutable audit/receipts, readiness, recovery, provenance classes, and player disclosure requirements are specified and tested. |
  | REQ-008 | PASS — 13 contained process scenarios accept one allowlisted flow and reject capability, identity, digest, interface, import, memory, fuel, trap, exit, timeout, and restart hazards without production activation. |
  | REQ-009 | PASS — one exact WIT/proof corpus validates marketplace and operator provenance as separate claims under identical capability and containment rules. |
  | REQ-010 | PASS — ADR-0004, architecture/operator/roadmap docs, Tickets 040–041, CodeGraph/security inspection, OpenWiki, focused proof, and the complete local gate are present; the production-loader absence check passes. |
  | REQ-011 | PASS — GitHub Actions is disabled, its workflow definition is removed, local-only automation enforcement rejects a hostile workflow fixture, and the canonical diff gate remains local. |

- OpenWiki:
  - Added the dedicated server-module boundary page and reconciled quickstart,
    product, cartridge, validation, Codex workflow, and navigation pages.
  - Initial run `1c5c2478-6e78-41bf-818d-a15274776895` preserved unresolved
    evidence debt rather than claiming it resolved. Follow-up run
    `7156dcfc-08c2-4548-a173-0bfadd870a64` inspected and reconciled every
    affected claim and completed without warnings.
- AAR and knowledge:
  - AAR-039 records eight failures, nine prevention rules, and ADR-0004's
    process-isolated Wasm decision. All 18 new IDs are registered in the
    knowledge index.
- Follow-up sequence:
  - Ticket 040 is the bounded production module base plus safe observation
    hooks. Ticket 041 separately gates administrator custom module installation
    and provenance. Provider SDK publication and additional hook classes remain
    independent roadmap work.
- Archive:
  - Ticket 039 is closed and the spec/notes are archived. The final delivery
    security scan, local diff gate, staged review, commit, push, and remote
    readback remain the separate authorized delivery phase.
- Phase 5 is PASS.

## Defect and lesson ledger

| # | What happened | Root cause | Fix | Prevention |
|---|---|---|---|---|
| 1 | Opening Ticket 039 made `scripts/check-pipeline.sh` reject completed Ticket 038 because its AAR was not in the exact terminal state. | Ticket 038 used the intuitive but unsupported AAR state `complete` instead of the repository contract `submitted`. | Changed only AAR-038 frontmatter to `status: submitted` and reran the validator. | Treat workflow frontmatter values as exact schemas and validate the completed archive before delivery. |
| 2 | The first full diff gate rejected two new interface fixtures for whitespace at EOF. | Focused proof checks did not include `git diff --check`; the complete delivery gate did. | Removed the extra terminal blank lines and reran `git diff --check`. | Preserve the complete gate as the authoritative result even when focused implementation checks are green. |
| 3 | Native-client packaging fixtures copied the nested module-spike `target/` tree and exhausted temporary-directory quota. | The fixture copied the complete `crates/` directory; before this ticket no nested workspace under it had a large independent build directory. | Stream source trees through `bsdtar` with nested `target/` exclusions and prove the archive inventory before extraction. | Source-fixture copies must explicitly exclude generated build trees rather than relying on the repository's prior directory shape. |

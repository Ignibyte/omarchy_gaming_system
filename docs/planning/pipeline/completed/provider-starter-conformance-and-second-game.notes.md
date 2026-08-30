---
title: Provider starter, conformance kit, and second clean-room game — notes
pipeline_id: 956c841d-af4b-4e55-a13f-e6a9d143a231
---

# Provider starter, conformance kit, and second clean-room game — completed notes

Chronological evidence and decisions. If a test or command did not run, these
notes must not claim it passed.

## Phase 1 — Plan

- Recall: Ticket 044 delivered `omarchygs-provider-sdk`, exact-v1/four-
  capability authenticated compatibility, final locked trust admission, one
  aggregate broker deadline, exact durable replay compatibility, and a
  deterministic public-only preview release at commit `2bd38b0`.
- Delivery readback: local `HEAD`, `origin/main`, and direct remote readback
  matched `2bd38b01e8ecee9526e85269a99dc1c990f0b98c`; the worktree was clean and
  the commit contained zero hosted workflow files.
- Routing: external private-alpha acceptance and official marketplace
  operations remain blocked on real people, systems, accounts, custody, and
  operating proof. The starter/conformance/second-game slice is the next
  locally actionable roadmap work and does not counterfeit those prerequisites.
- Recall: provider identity binding, replay-before-current-policy, serialized
  deduplication, first-callback disposition, post-authentication quota, clean
  source-tree coverage, locked final authority, aggregate deadline, native
  artifact inventory, durable wire-preimage, and legacy-duplicate-only rules
  all constrain this slice.
- Current gap: the protocol is public, but Door Legends still combines its own
  HTTP/TLS configuration, grant verification, PostgreSQL receipt/state tables,
  callback outbox, recovery, and game rules in one game-specific binary. The
  existing conformance suite is platform-owned and cannot be run as a bounded
  public developer tool.
- Decision: select Relay Forge as the second clean-room game. Its deterministic
  resource-building commands and terminal target are intentionally distinct
  from Door Legends and small enough to expose starter coupling without adding
  UI or product behavior.
- Decision: package the starter and conformance kit as public-only local
  preview artifacts, build Relay Forge from clean clones, and admit it only in
  ephemeral conformance databases. No route, production catalog entry,
  provider onboarding, registry publication, or hosted workflow is added.
- `scripts/check-pipeline-tools.sh` passed with CodeGraph 1.5.0, OpenWiki 0.3.3,
  verified pnpm, reviewed Codex-only patch/build provenance, and local tools
  ready. No critical bulletin blocks work. Phase 1 is PASS.

## Phase 2 — Design

- Public package boundary:
  1. Keep `omarchygs-provider-sdk` as the dependency-light owner of protocol,
     model, exact-byte verification, and SDK release identity. Add separate
     `omarchygs-provider-starter` and `omarchygs-provider-conformance` public
     preview packages; neither may depend on `omarchy-game-provider`, the
     OmarchyGS server, platform migrations, registry, broker, egress, operator
     commands, or client code.
  2. The starter owns provider-side HTTP/TLS routing, exact SDK verification,
     a separate PostgreSQL schema, whole-operation receipts, consumed-grant
     replay checks, session state, callback outbox/delivery, bounded config,
     and shutdown. A narrow synchronous `ProviderGame` trait owns only initial
     state, deterministic command transition, bounded view, and optional
     terminal event facts. It receives no keys, database handle, HTTP values,
     callback target, account/persona identifier, or platform authority.
  3. The conformance package owns an exact loopback-only TLS test client,
     deterministic platform test keys/grants/messages, a callback sink, the
     published fault inventory, and a bounded machine-readable receipt. Its
     conformance transport cannot become a production/private-network egress
     exemption and carries no registration operation.
- Starter persistence and operation flow:
  1. One starter database contains generic `provider_starter_sessions`,
     `provider_starter_consumed_grants`, `provider_starter_operation_receipts`,
     and `provider_starter_event_outbox` tables. A configured database is one
     provider-release/game authority; identity metadata is pinned in every
     durable row and startup rejects mixed identity.
  2. Compatibility remains state-free and is authenticated exactly as SDK v1.
     For an operation, the starter bounds the body, verifies Host/signature,
     parses the strict request, matches every configured identity and
     operation path, verifies the grant, canonicalizes a stable intent
     preimage, and only then opens the mutation transaction.
  3. Under one transaction, lock a guaranteed session or identity root before
     first-delivery decisions, validate a repeated grant by exact request
     digest, resolve an existing operation receipt before current revision,
     and invoke rules only for new work. Launch creates revision zero; command
     returns the current state with `revision_conflict` or atomically advances
     one revision; reconcile is read-only. The response preimage and any
     terminal event/outbox row commit with the state change.
  4. Retried exact operations return the persisted response identity. A
     configured conformance-only post-commit delay proves timeout/unknown-
     outcome recovery. Callback delivery signs exact persisted bytes, treats
     204 or authenticated duplicate acceptance as delivered, and retains a
     bounded exponential retry schedule across process restart.
- Rules seam and Relay Forge:
  1. `ProviderGame` exposes immutable identity plus `launch`, `command`,
     `view`, and `event` methods over bounded JSON state. The starter owns
     revision, subject/session association, status, protocol envelopes,
     validation, persistence, and errors; rule errors map only to stable public
     provider errors.
  2. Relay Forge starts at zero ore/energy and accepts `mine`, `charge`, and
     `forge`; forging requires two ore and one energy and completes the
     session. Rule-only tests prove deterministic transitions, invalid command
     rejection, terminal immutability, and state/view bounds. A second test
     rules implementation proves substitution without copying Relay behavior.
  3. `examples/provider-relay-forge` contains only the rules, bounded config
     bootstrap, migrations/runtime invocation, and tests. It consumes packaged
     SDK/starter artifacts from two Git clones and owns a database, keys, TLS
     identity, and process distinct from Door Legends and OmarchyGS.
- Portable conformance and real integration:
  1. The public runner executes exact compatibility, launch, command,
     reconcile, and callback flow, then named faults for request replay,
     changed intent, stale revision, commit-then-timeout, outage/restart,
     signature, digest, authority/context, malformed/oversized bodies,
     callback replay, and recovery. Every case has a stable identifier and
     expected disposition; the receipt is sorted, finite, secret-free, and
     fails if any required case is missing.
  2. A local orchestration script packages all public crates, builds Relay
     Forge from two no-hardlink clean clones using command-line Cargo patches,
     starts its separate PostgreSQL database and TLS process, runs the public
     conformance CLI, and restarts the same durable database for recovery.
  3. A dedicated `omarchy-game-provider` integration test registers Relay
     Forge only in a fresh ephemeral conformance database, uses the real
     `ProviderBroker` with its exact loopback test socket, exercises launch,
     command, timeout retry, reconcile, event authentication/deduplication,
     and verifies platform/provider databases are distinct. It adds no
     product route, catalog row, discovery item, or production pilot authority.
- Deterministic developer-kit release:
  1. Package SDK, starter, and conformance crates independently through Cargo
     and inspect exact archive inventories/dependency trees. A conformance-
     owned release helper creates an existing-empty-directory developer kit
     containing those exact `.crate` archives, README/notice, fault inventory,
     config/receipt schemas, a canonical file lock, checksums, and a domain-
     separated Ed25519 provenance envelope binding all package digests, SDK
     compatibility, source revision, and builder digest.
  2. Produce every package and kit twice, compare bytes, verify signatures and
     inventories, build the two clean Relay Forge clones, and scan packages,
     exports, dependency trees, and binary strings for repository paths,
     secrets, platform-only crates/modules, or private integration hooks.
- Security, privacy, and failure controls: config is an exact bounded private
  file, secret material is decoded into zeroizing or signing-key types and
  never emitted, TLS keys remain caller-owned files, database errors and remote
  bodies map to stable codes, callback networking pins one configured HTTPS
  origin/root/socket in conformance, database pools are finite, requests and
  JSON depth/value/string sizes reuse SDK ceilings, session/receipt/outbox
  inventories are bounded at write/read, and shutdown terminates/reaps the
  callback worker. Provider payloads carry only pairwise subjects and game
  state—never local persona/account IDs, tokens, platform database fields, or
  reusable credentials.
- Compatibility and rollback: the protocol wire format and platform schema do
  not change. Door Legends stays on its proven persisted layout and remains the
  sole production provider. Removing the new packages, gate stage, ephemeral
  test registration, and Relay Forge example restores Ticket 044 behavior with
  no migration rollback or provider data conversion.
- Exact file manifest:

  | Path | Purpose |
  |---|---|
  | `Cargo.toml`, `Cargo.lock` | Add public starter and conformance workspace packages. |
  | `crates/provider-starter/{Cargo.toml,src/**,migrations/**,README.md}` | Generic rules seam, exact HTTP/TLS runtime, PostgreSQL receipts/state/outbox, callbacks, config, and tests. |
  | `crates/provider-conformance/{Cargo.toml,src/**,kit/v1/**,README.md}` | Loopback TLS runner/CLI, fault inventory, receipt/schema contract, and signed developer-kit release. |
  | `examples/provider-relay-forge/**` | Clean-room rules, minimal runtime bootstrap, tests, and independent package source. |
  | `examples/provider-kit-consumer/**` | Public-only deterministic kit export/verification consumer. |
  | `crates/game-provider/tests/starter_integration.rs` | Ephemeral registry plus real-broker Relay Forge proof. |
  | `scripts/test-provider-developer-kit.sh`, `scripts/test-provider-conformance.sh`, `bin/gate.sh` | Package/clone/kit/conformance/integration proof and local gate stage 13b/19 coverage. |
  | README, architecture, operator, product, roadmap/intake, OpenWiki, ticket artifacts | Current boundary, developer workflow, evidence, and remaining sidecar/onboarding work. |

  No platform migration, player/operator API route, QML surface, provider
  discovery field, production provider registration, hosted workflow, or
  external publication action is in scope.
- Requirement-to-evidence map:

  | Requirement | Evidence |
  |---|---|
  | REQ-001 | Starter transaction/unit/PostgreSQL tests, commit-timeout replay, callback restart, and real broker run. |
  | REQ-002 | Trait/API review, rule-only tests, alternate rules substitution, forbidden-dependency scan. |
  | REQ-003 | Public runner required-case inventory, endpoint suite, callback sink, outage/restart orchestration, exact receipt validation. |
  | REQ-004 | Two clean clones, packaged dependency patches, real TLS/broker/database test, path/private-import scan. |
  | REQ-005 | Double Cargo packages and kit exports, byte comparison, signed lock/provenance verification, clean builds. |
  | REQ-006 | README/schema assertions, serialized traffic scan, public route/catalog diff, sole-pilot regression, full gate. |
- Material alternatives rejected: putting SQLx/Axum into the core SDK weakens
  its dependency-light contract; copying Door Legends creates a second
  game-specific framework; migrating Door Legends persistence adds production
  compatibility risk unrelated to proving reuse; an in-memory starter cannot
  prove restart/idempotency; a conformance library without a CLI or real
  endpoint does not prove portability; direct server-route integration would
  create product authority; a general loopback/private-network exemption
  pre-decides the sidecar threat model; and publishing to crates.io or adding
  hosted CI exceeds the owner-authorized local delivery boundary.
- CodeGraph design evidence traced `ProviderBroker::execute` through exact
  compatibility, final admission, grant, attempt, response, callback, and
  replay flows; traced Door Legends HTTP verification, generic receipt/session/
  outbox work, and game-specific launch/command/view/event functions; and
  inspected the SDK export inventory and dependents. Its generic `execute`
  expansion included unrelated SQL callers and was discarded. Direct review
  covered Cargo, migrations, shell orchestration, docs, package resources, and
  test fixtures that CodeGraph cannot fully model. The worktree-bound design
  receipt matches pipeline `956c841d-af4b-4e55-a13f-e6a9d143a231` and gated
  state `169baaddf43af54264773ee504106788e97e18e48847ea3f885a145d4cb8f67a`.
- Phase 2 is PASS.

## Phase 3 — Implement

- Added `omarchygs-provider-starter` as a public-only Axum/PostgreSQL runtime
  with four fixed routes, exact Host/signature/grant/context admission, a
  narrow `ProviderGame` rules seam, provider-owned identity/session/grant/
  receipt/outbox persistence, receipt-before-revision replay, atomic terminal
  callbacks, bounded retry, and a conformance-only post-commit delay.
- Added `omarchygs-provider-conformance` with a loopback/pinned-root TLS
  client, authenticated callback sink, fixed 15-case corpus, secret-free
  receipt, deterministic signed developer-kit export/verifier, schemas,
  fixtures, and a bounded key helper.
- Added standalone Relay Forge and clean developer-kit consumer examples.
  Relay Forge supplies independent mine/charge/forge rules, private mode-0600
  config loading, distinct keys/process/database, and consumes only packaged
  public crates in clean-clone validation.
- Added ignored real-PostgreSQL starter persistence and real-`ProviderBroker`
  integration tests plus local packaging/conformance scripts and gate stage
  13b. No platform migration, route, registry/catalog entry, hosted workflow,
  or production provider authority was added.
- Focused implementation evidence passed: starter/conformance compilation and
  unit tests, durable PostgreSQL replay across restart, real broker timeout
  recovery and callback deduplication, two complete live TLS conformance runs,
  deterministic double package/export verification, and two clean Relay Forge
  clone builds.
- Phase 3 is PASS.

## Phase 3.5 — Inspect ledger

| # | Lens | Finding | Severity | Disposition |
|---|---|---|---|---|
| 1 | Destination binding | The local conformance clients checked loopback IP but did not require the URL port to match the claimed exact socket, and accepted IP-literal URLs that could bypass DNS overrides. | low, test-only | Fixed: target fields are immutable, endpoint/authority/domain/socket ports must match, starter callback overrides enforce the same rule, and focused negative tests cover port and host-literal rejection. |
| 2 | Callback evidence | The conformance sink authenticated provider/release/message but did not bind an observed duplicate to game/rules/cartridge/subject/session or stable body bytes. | low, evidence integrity | Fixed: sink configuration binds the full release/game/subject identity; observations bind session, revision, and body SHA-256; the runner waits for the exact exercised session/revision. |
| 3 | Secret lifetime | Raw private config buffers and base64 seed/database strings lived longer than their decoded signing objects required. | defense in depth | Fixed: private buffers use `Zeroizing`, seed strings are zeroized immediately after decoding, the Relay Forge database URL is zeroized after connection, and the key helper clears raw input/seed strings. |
| 4 | Security diff | Complete frozen Codex Security review validated the two conformance candidates above but rejected both as reportable vulnerabilities because only the local test operator controls the targets and receipts create no admission or platform authority. | none reportable | Scan `aeb84657-2e73-4a6f-b651-49d428941a3d` completed with full coverage and zero findings; quality fixes were applied anyway and focused suites reran green. |
| 5 | Existing pilot transport | CodeGraph showed the unchanged Door Legends conformance callback resolver has the earlier port-binding shape. | pre-existing, test-only | Not a TICKET-045 regression or release path; retained as an explicit transport-hardening input for TICKET-046 rather than silently broadening this ticket's locked file manifest. |

- Fresh CodeGraph inspection traced the final provider runtime → exact
  authentication → transactional starter store → rule seam → persisted
  response/callback path, the conformance request and callback-evidence flow,
  Relay Forge implementations, platform broker consumers, and one-hop
  dependents. Its automatic test association does not fully model ignored
  PostgreSQL integration tests or standalone-workspace tests, so the direct
  test/script evidence in Phase 4 remains authoritative. Cargo manifests,
  migrations, shell orchestration, kit fixtures/schemas, and documentation
  were inspected directly because CodeGraph does not fully model them.
- All TICKET-045 findings are resolved. Phase 3.5 is PASS.

## Phase 4 — Validate

- Focused final-state evidence:
  - `cargo test -p omarchygs-provider-conformance --lib` — 3 passed;
  - `cargo test -p omarchygs-provider-starter --all-features --lib` — 1
    passed;
  - `cargo clippy -p omarchygs-provider-starter --all-features --all-targets
    -- -D warnings` — passed;
  - `cargo clippy -p omarchygs-provider-conformance --all-targets -- -D
    warnings` — passed;
  - `scripts/test-provider-developer-kit.sh` — deterministic packages/export
    and two clean Relay Forge builds passed;
  - `scripts/test-provider-starter-conformance.sh` — durable provider restart,
    real broker flow, timeout replay, exact callback retry/deduplication, and
    two live TLS conformance passes succeeded.
- `bin/gate.sh --diff` completed every local stage and printed `GATE GREEN
  [diff]`. The worktree-bound receipt hash was
  `b42537c314a69fdcd86967467120e4832caa2100cb46bc29fd26cb2c8156e175`.
  Phase 5 documentation/archive edits will intentionally require one final
  matching rerun before delivery.
- Phase 4 is PASS.

## Phase 5 — Complete

- Acceptance-criteria audit:

  | Requirement | Result | Completion evidence |
  |---|---|---|
  | REQ-001 | PASS | Exact Host/signature/context/grant admission precedes game rules; the serialized PostgreSQL transaction resolves receipts before revision, persists provider sessions and callback outbox, and passed timeout/replay/restart, callback, reconcile, real-broker, and platform-database separation tests. |
  | REQ-002 | PASS | `ProviderGame` exposes only identity plus deterministic launch/command/view/event methods; Relay Forge and the starter test game substitute behind it while transport, authentication, storage, callback delivery, configuration, and lifecycle stay in the starter. |
  | REQ-003 | PASS | The standalone CLI emitted complete bounded receipts for all fifteen fixed valid/fault cases twice across provider restart, including exact context/signature/digest/input, replay/revision, timeout/outage, reconcile, callback dedupe, and recovery behavior. |
  | REQ-004 | PASS | Two clean Relay Forge clones consumed packaged public crates without repository paths or private platform dependencies; its distinct rules/process/keys/database passed deterministic tests, public TLS conformance, and the real broker integration. |
  | REQ-005 | PASS | SDK, starter, and conformance packages and two signed developer-kit exports compared byte-for-byte; finite inventories, provenance, clean builds, source-path/credential/private-key scans, and package verification passed. |
  | REQ-006 | PASS | README, crate guides, ADR/operator docs, OpenWiki, public dependency scans, serialized receipt scans, and route/authority inspection preserve pairwise scoped identity and deny account/persona, reusable credential, platform database, egress, client-executable, direct-client, registration, admission, discovery, trust, and publication authority. |

- Docs: OpenWiki update run `b2f7cb9b-ec51-4df8-82c6-8359bc1e7295`
  completed and reconciled quickstart, cartridge, product-boundary, and
  validation pages. Its completion receipt matches gated state
  `fdd7951fc9f673ef29a433b90c6cb398421227ab85f1d42b804e02350ab9c2f3`.
  Existing evidence-debt warnings on the four long-lived page claim sets did
  not invalidate the lifecycle or the resolved Ticket 045 claims. README,
  architecture, operator, roadmap, and intake docs now record the implemented
  developer kit and retain sidecar/onboarding as future work.
- AAR: submitted with two captured failures, two prevention rules, and one
  architecture decision; every new ID was appended to the knowledge register.
- Roadmap/intake: marked only the starter/conformance/Relay Forge sub-slice
  complete. The sidecar/operations profile remains locally actionable next;
  external acceptance, marketplace operations, and reviewed provider onboarding
  remain blocked on their stated real-world prerequisites.
- Archive: ticket moved to `tickets/closed`; spec and notes moved to
  `pipeline/completed`; no active pipeline remains. Phase 5 is PASS.
- Final delivery gate: `bin/gate.sh --diff` completed every local stage after
  OpenWiki and archival edits and printed `GATE GREEN [diff]`. The gate and
  OpenWiki completion receipts both match gated state
  `fdd7951fc9f673ef29a433b90c6cb398421227ab85f1d42b804e02350ab9c2f3`.

## Defect and lesson ledger

| # | What happened | Root cause | Fix | Prevention |
|---|---|---|---|---|
| 1 | An outage probe initially reached the healthy provider despite a different resolver socket. | Reqwest preserves the URL port when applying a DNS socket override, while the target constructor treated the override as a complete address pin. | Bind URL, canonical authority, DNS host, and socket port together and construct the outage URL/client as one matching pair. | Keep exact-transport negative unit tests and live outage recovery in the public conformance corpus. |
| 2 | Callback duplicate evidence could have been satisfied by a validly signed event unrelated to the exercised session. | The sink reused transport-authentication checks as semantic test-evidence checks. | Bind full immutable identity plus session/revision/body digest at observation and wait time. | Treat test receipts as security-relevant claims and validate every fact they attest independently of production admission. |

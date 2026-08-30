---
title: Public Provider SDK contract, negotiation, and release — notes
pipeline_id: fb5cf56b-6421-482c-badf-fc3e3b02a92e
---

# Public Provider SDK contract, negotiation, and release — running notes

Chronological evidence and decisions. If a test or command did not run, these
notes must not claim it passed.

## Phase 1 — Plan

- Recall: Ticket 043 was delivered at `0075287`; local and `origin/main`
  matched, GitHub Actions were disabled with zero workflows, the worktree was
  clean, and there was no active pipeline or open ticket.
- Recall: the owner explicitly authorized autonomous implementation, commits,
  and pushes until the project is finished, while requiring every quality and
  delivery gate to remain local.
- Routing: the earlier private-alpha installation intake is blocked on two
  real external people and clean installations, and official marketplace
  operations are blocked on accounts, budget, named custodians, and an
  observation window. Promotion rules therefore select the first locally
  actionable Public Provider SDK slice and do not counterfeit external proof.
- Recall: Tickets 018 and 019 provide the production registry, fixed v1
  protocol, grant/message security, guarded broker, separate TLS provider,
  durable replay/quota/audit state, and sole Door Legends authority pilot.
- Recall: Ticket 017's cartridge SDK established the exact export pattern:
  compiled-owned bytes, canonical lock identity, deterministic two-export
  comparison, signed revision/tool provenance, fresh Git clones, and no source
  path dependency.
- Recall: `PR-omarchy-gaming-system-bind-provider-messages-to-registered-identity-001`
  requires provider/release/game/rules/cartridge/session/subject/scope/expiry
  binding; replay, callback disposition, quota, lifecycle, lock-order, clean
  clone, and independent-source-tree prevention rules remain applicable.
- Current gap: `omarchy-game-provider` with default features disabled compiles
  a useful protocol surface, but its package still carries platform model and
  implementation sources; the protocol documents compatibility as fixed and
  implicit v1; Door Legends depends on that internal package; no standalone
  SDK lock or release provenance exists.
- Decision: the first slice is a dedicated public-only crate, authenticated
  exact-v1/four-capability negotiation bound through grants/messages, and a
  deterministic locally signed SDK export. Starter/conformance/second-game and
  sidecar work stay separate.
- Decision: the project owns the preview artifact and signing authority; no
  external publication, registration, activation, discovery, or support claim
  is created. Existing repository terms remain unchanged and the export shall
  state its preview/no-license-grant status rather than inventing a license.
- `scripts/check-pipeline-tools.sh` passed with CodeGraph 1.5.0, OpenWiki 0.3.3,
  verified pnpm, reviewed patch/build provenance, and Codex-only tooling ready.
- No critical bulletin blocks work. Phase 1 is PASS.

## Phase 2 — Design

- Public boundary:
  1. Add `omarchygs-provider-sdk` as a no-default-feature workspace crate. It
     owns `ProviderError`, `ProviderScope`, compatibility types, grant/message
     types, pairwise identity, bounded payload validation, exact-byte signing,
     verification, and deterministic SDK release APIs. It has no SQLx, Tokio,
     Reqwest, URL, tracing, Axum, registry, broker, egress, operator command,
     migration, or platform database surface.
  2. Move the existing protocol implementation into that crate and keep
     `omarchy-game-provider::{protocol, model::ProviderScope, ProviderError,
     Result}` as source-compatible re-exports. The platform crate continues to
     own every registry, policy, quota, durable receipt, egress, and operator
     implementation. Door Legends changes its dependency/imports to the SDK;
     its own game persistence remains outside both crates.
  3. The SDK export is an exact existing-empty-directory write. Compiled-owned
     Cargo/source/docs/schema/fixture/notice bytes plus a canonical lock are
     read-only and exhaustively inventoried. A domain-separated Ed25519
     release envelope signs authority, key, SDK/protocol identity, lock digest,
     reviewed source revision, and builder digest. Verification reopens every
     bounded regular file, rejects symlinks/unknowns/drift, reconstructs the
     lock, and verifies the release signature. The notice records project
     preview ownership and no license grant; this ticket does not invent legal
     distribution terms.
- Compatibility protocol:
  1. SDK v1 defines one exact profile: protocol version `1` and the sorted
     required `game.launch`, `game.command`, `game.reconcile`, and `game.event`
     capabilities. Offers are bounded nonempty unique profile lists. A
     selection carries exactly one supported profile and binds the offer's
     provider, release, and message identity. Unknown, empty, duplicate,
     partial, extra, ambiguous, or non-highest/downgrade selections reject.
  2. Before a new broker attempt receives a grant or operation request, the
     broker charges normal request admission/concurrency, sends a signed
     compatibility offer to the fixed `compatibility` path, verifies the
     provider-signed exact selection and originating request context, and only
     then issues a compatibility-bound grant. A failed preflight releases the
     lease, records no provider operation attempt, and cannot mutate game state.
  3. The selected profile is a required field in grant claims, operation
     requests, responses, and events. Provider verification checks the signed
     request, exact current selection, grant identity/scope/expiry/replay, and
     request-to-grant equality before persistence or rules execution. Platform
     response/callback verification checks the provider signature and exact
     current selection before receipt/projection work. Schema v1 gains the
     mandatory field before its first public release; no optional/default
     legacy interpretation exists.
  4. Door Legends and the conformance provider expose only the fixed preflight
     route in addition to their current three operation routes. Preflight reads
     no gameplay state and returns a signed SDK selection. Events copy the
     already verified request selection. There is no general discovery route.
- Release and clean-source proof:
  1. Package the SDK through `cargo package`; inspect its exact file list and
     dependency tree for forbidden platform/runtime surfaces and repository
     paths. A focused shell harness extracts that package, creates one
     deterministic consumer repository, clones it twice without hardlinks,
     and supplies the package only through a command-line Cargo patch rather
     than a committed path dependency.
  2. Both clones build the package, call the public export and verification
     APIs with identical reviewed inputs, and produce byte-identical directory
     snapshots and release signatures. The harness rejects OmarchyGS source
     paths, secret material, platform module names, unexpected files, and
     differing hashes. The local gate runs this proof; it publishes nothing.
- Exact file manifest:

  | Path | Purpose |
  |---|---|
  | `Cargo.toml`, `Cargo.lock`, `crates/provider-sdk/Cargo.toml` | Add the public-only package and pin its dependency graph. |
  | `crates/provider-sdk/src/{lib,model,protocol,release}.rs` | Public errors/scopes, negotiation and exact-byte protocol, deterministic export, and signed provenance. |
  | `crates/provider-sdk/sdk/v1/**` | Compiled export manifest, README/notice, JSON Schemas, and valid/hostile compatibility fixtures. |
  | `crates/provider-sdk/tests/**` | Public contract, negotiation, exact export/provenance, and hostile-vector tests. |
  | `crates/game-provider/{Cargo.toml,src/lib.rs,src/model.rs,src/broker.rs,src/registry.rs,tests/**,src/bin/**}` | Depend/re-export the SDK and perform authenticated preflight in the production broker/fixture. |
  | `examples/first-party-door-legends/provider/**` | Consume the SDK directly and perform preflight before any durable effect. |
  | `scripts/test-provider-sdk.sh`, `scripts/test-provider-conformance.sh`, `scripts/test-provider-authority-pilot.sh`, `bin/gate.sh` | Two-clone SDK proof and integration into existing provider gates. |
  | Architecture/operator/README/OpenWiki and ticket artifacts | Publish the exact current contract, limitations, evidence, and remaining slices. |

  No migration, public API route, QML surface, server catalog response, provider
  registration input, external setting, or hosted workflow changes.
- Requirement-to-evidence map:

  | Requirement | Evidence |
  |---|---|
  | REQ-001 | Package/export inventories, dependency and forbidden-name scans, SDK unit tests, and two-clone harness. |
  | REQ-002 | Compatibility offer/selection matrix, signed-context and stripping vectors, broker fixture preflight, and Door Legends authority pilot. |
  | REQ-003 | Existing plus migrated protocol tests for every identity, grant/signature time, replay/body/schema/payload bound, and selected-profile mismatch. |
  | REQ-004 | Exact two-export comparison, release-envelope verification/tamper cases, packaged clean clones, lock/source/builder binding, and path-leak rejection. |
  | REQ-005 | Workspace checks, provider conformance/authority scripts, server route/catalog diff, operator-boundary inspection, and full diff gate. |
- Security and failure controls: negotiation uses the existing public-only TLS
  endpoint, registered provider/release headers, short-lived RFC 9421 profile,
  exact provider message keys, guarded DNS/TLS/redirect/body/time policy, and
  the already charged request lease. It exposes no persona, account, device,
  secret, database, endpoint discovery, or administrator data. Repeated
  preflight is idempotent and state-free. A timeout or invalid selection is
  provider-unavailable/protocol-rejected before an attempt is durable or game
  authority is invoked.
- Compatibility/rollback: internal users retain re-export paths, but serialized
  pre-public v1 messages intentionally become strict mandatory-compatibility
  documents. Both shipped providers and the broker update atomically in this
  repository. Rollback reverts the crate extraction and mandatory field before
  any external provider is admitted; persisted game/session schemas are
  unchanged.
- Material alternatives rejected: publishing the current platform package
  leaks dormant implementation source; a post-operation response version is
  too late; an unsigned discovery document is downgradeable; an optional
  compatibility field preserves implicit v1; registry-time persistence needs
  a migration and release-onboarding authority not needed for this slice; and
  assigning an open-source license without owner-selected legal terms exceeds
  this technical release ticket.
- CodeGraph design evidence: worktree-bound exploration traced
  `ProviderBroker::execute`, `ProviderEvent`, `SignedProviderGrant`, registry
  grant issuance, server callback consumers, and current protocol constructors.
  It confirmed the direct runtime blast radius is provider broker/registry,
  server provider-game projection, both provider processes, and their tests;
  its generic `execute` caller expansion included unrelated SQL execution and
  was discarded. Direct review covered Cargo, shell, docs, examples, and export
  resources that CodeGraph cannot model. The design receipt matches pipeline
  `fb5cf56b-6421-482c-badf-fc3e3b02a92e` and gated state
  `8bda12e05a1a053270fe28ac240de7fc415e81d509eaa0210835ab6562f2f00d`.
- Phase 2 is PASS.

## Phase 3 — Implement

- Built: added `omarchygs-provider-sdk` as a no-default-feature workspace crate
  and moved the complete provider-facing protocol into it. The crate owns
  bounded errors/scopes, pairwise subjects, grants, exact-byte RFC 9421/9530
  signatures, strict payloads, compatibility models, and deterministic release
  APIs. `omarchy-game-provider` depends on and source-compatibly re-exports that
  public contract while retaining all registry, broker, egress, database,
  operator, quota, lifecycle, and receipt implementations.
- Built: protocol v1 now has one exact compatibility profile with launch,
  command, reconcile, and event capabilities. The broker charges ordinary
  request admission, sends a signed preflight offer to the fixed compatibility
  path, verifies the provider-signed response and originating request, and only
  then issues a grant. Selection is mandatory in grant claims, operation
  requests/responses, and events. The conformance fixture and Door Legends
  verify the offer before effects; hostile stripping/downgrade/unknown/
  ambiguity vectors reject.
- Built: the broker releases its concurrency lease and records a safe failure
  audit when preflight fails. The real separate-process conformance test makes
  the provider return a signed stripped selection and proves rejection creates
  zero grants, zero provider attempts, and no provider state file before a
  correct restart succeeds.
- Built: Door Legends now depends directly on packaged
  `omarchygs-provider-sdk`, exposes the state-free signed preflight route, binds
  the verified selection into its durable request intent/responses/outbox
  events, and retains its independent database and existing authority model.
- Built: exact SDK export writes a standalone Cargo crate, source, README,
  conservative licensing notice, six schemas, and three compatibility fixtures
  plus a canonical file/digest lock and domain-separated Ed25519 release
  envelope binding project authority, key, source revision, and builder digest.
  Verification rejects symlinks, missing/unknown files, byte drift,
  noncanonical envelopes, wrong keys/authority, tampering, and provenance
  mismatch.
- Built: `scripts/test-provider-sdk.sh` packages only the public crate, rejects
  platform-only paths/dependencies and repository path dependencies, builds one
  deterministic consumer from two fresh Git clones, compares byte-identical
  signed exports, verifies every artifact, and rejects source-path leaks. Local
  gate 13a makes the proof mandatory. The existing Door Legends authority gate
  now packages this SDK instead of the platform crate.
- Built: README, ADR-0003, cartridge/system architecture, product charter,
  provider security/pilot runbooks, owner-operated guidance, roadmap, intake,
  and planning artifacts distinguish the delivered preview from the remaining
  starter/conformance, second-game, sidecar, operations, and onboarding work.
- Focused evidence so far: SDK 7 unit + 3 release tests passed; warnings-denied
  workspace Clippy passed; the deterministic two-clone SDK release passed;
  provider conformance passed 8 library, 3 egress, 4 public protocol, 1 admin
  CLI, 1 real TLS process, and 6 PostgreSQL registry tests; the packaged
  clean-clone Door Legends TLS/callback/restart/backup/restore authority pilot
  passed.
- Deviation: the design called the compatibility offer a bounded list to
  support future preference ordering. SDK v1 deliberately requires exactly one
  current profile instead: accepting unknown alternatives in the first public
  release would weaken the explicit unknown/ambiguous fail-closed requirement.
  A future version can introduce a new negotiation schema with a tested window.
- Deviation: no open-source license was invented. The exact export includes a
  conservative copyright/no-license-grant notice and `publish = false`; public
  here means technically separated and independently buildable preview.
  Owner-selected redistribution/production terms remain external legal policy.

## Phase 3.5 — Inspect ledger

| # | Lens | Finding | Severity | Disposition |
|---|---|---|---|---|
| 1 | Security / trust freshness | Compatibility preflight originally authenticated a registry snapshot that could become stale before grant issuance or operation dispatch. | Low | Fixed. The request binds expected configuration revision and compatibility-key identity; grant issuance and final durable-attempt creation each re-admit under provider/release locks, and the latter returns the exact material used for transport and verification. Regression proves a changed configuration creates no grant, attempt, or quota charge. |
| 2 | Concurrency / resource bounds | Two sequential provider POSTs originally each received the full timeout while the one request lease covered only one timeout plus one second. | Low | Fixed. Compatibility, grant preparation, and operation transport share one aggregate Tokio deadline shorter than the lease. Timed real-TLS conformance proves two individually-under-limit stages cannot exceed the aggregate attempt bound. |
| 3 | Release integrity / input bounds | SDK verification originally walked unbounded directory breadth before comparing the inventory. | Low | Fixed. Native file/directory allowlists now reject unknown entries during traversal with explicit depth, 64-entry, and 4 KiB aggregate-path limits; hostile broad-tree and unexpected-empty-directory cases reject. |
| 4 | Release integrity / path identity | Unix filenames containing literal backslashes could alias signed slash-separated inventory identities. | Low | Fixed. Inventory identity is native `PathBuf` with no separator rewriting or silent normalized deduplication; a literal `src\\lib.rs` hostile case rejects. |
| 5 | Compatibility / durable replay | Adding compatibility to the historical durable intent bytes would have changed completed pre-release receipt digests and broken exact retry recovery. | Medium | Fixed during independent post-patch review. Durable intent bytes retain their historical v1 shape; narrowly named persisted-v1 response/event helpers upgrade only already-authenticated local rows, while normal network parsing remains strict. |
| 6 | Authentication / callback idempotency | A callback lost before acknowledgement needed to replay its exact historic bytes, but strict mandatory compatibility would reject it before duplicate resolution. | Medium | Fixed during independent post-patch review. Current-key signature verification and exact immutable receipt identity/digest matching are required before a legacy callback can resolve only as `Duplicate`; fresh legacy-shaped callbacks and equivocation reject. Door Legends upgrades a retained row only after explicit unauthorized rejection. |
| 7 | Correctness / public boundary | Protocol extraction could accidentally expose platform registry, broker, database, operator, or runtime dependencies. | High | No defect found. Package inventory, dependency inspection, forbidden-surface scan, direct Cargo/source review, and two clean-clone consumer builds confirm the SDK owns only provider-facing code. |
| 8 | Privacy / authority | Negotiation or release might create discovery, admission, persona/account leakage, or external publication authority. | High | No defect found. The fixed path is state-free, all identities are already registered or pairwise, the export publishes nowhere, and Door Legends remains the sole admitted provider. |
| 9 | Simplification / reuse | The extracted contract might duplicate protocol implementations or leave parallel internal/public types. | Medium | No defect found. `omarchy-game-provider` source-compatibly re-exports the one SDK implementation; the platform retains only registry, broker, egress, quota, receipt, and operator ownership. |

- The specialized working-tree security scan completed with full coverage of 23
  source-like diff items and reported four high-confidence low-severity
  findings (rows 1–4). All four were fixed. Independent pre-patch and two
  post-patch reviews identified and closed the historical replay issues in rows
  5–6; no finding remains accepted or deferred. Canonical scan and remediation
  artifacts are retained in the local scan directory outside the repository.
- Fresh CodeGraph inspection traced the post-fix broker execution, locked
  registry admission, compatibility/grant/attempt flow, exact SDK inventory,
  and one-hop dependents. Direct review covered the server callback route, Door
  Legends persistence/outbox behavior, Cargo, shell, tests, and resources that
  CodeGraph does not model. The inspection receipt matches pipeline
  `fb5cf56b-6421-482c-badf-fc3e3b02a92e` and gated state
  `afd18eac1301742ef68d57c2c9769fa126bd531c70d62da1357c064f78b39d4f`.
- Phase 3.5 is PASS.

## Phase 4 — Validate

- Tests run: focused SDK, deterministic two-clone export, real TLS provider
  conformance, six PostgreSQL registry cases, clean-clone Door Legends
  authority/restart/callback/legacy-replay/backup/restore, warnings-denied
  workspace Clippy, workspace tests and Rustdoc, native package reproduction,
  full PostgreSQL API/operator coverage, QML fixture and live smoke, marketplace,
  cartridge, backup/admission, and server-module containment suites all passed.
- Gate run: `bin/gate.sh --diff` completed all 24 locally executed stages and
  printed `GATE GREEN [diff]`. Its worktree receipt matches
  `afd18eac1301742ef68d57c2c9769fa126bd531c70d62da1357c064f78b39d4f`.
- Skips or pre-existing failures: none. Tests that the ordinary workspace run
  marks ignored for PostgreSQL, real TLS processes, Bubblewrap/systemd, or live
  QML were exercised by their dedicated later gate stages and passed. Mesa
  emitted two harmless headless `libEGL` warnings during the QML smoke. The
  server-module proof intentionally created a hostile `.github/workflows`
  fixture, observed the local-only automation checker reject it, removed the
  fixture, and passed; the repository itself still has zero hosted workflows.
- Phase 4 is PASS.

## Phase 5 — Complete

- Acceptance-criteria audit:

  | Requirement | Result | Completion evidence |
  |---|---|---|
  | REQ-001 | PASS | The package and exact exported inventory contain only provider-facing sources, schemas, fixtures, notice, lock, and provenance; dependency, forbidden-name, secret, unknown-file, symlink, path-alias, and clean-clone checks passed. |
  | REQ-002 | PASS | Exact v1 and all four capabilities are authenticated before effects and bound through grant/request/response/event; hostile selection, stripping, ambiguity, trust-change, and timeout cases failed closed without a durable attempt. |
  | REQ-003 | PASS | Public verification tests cover exact provider/release/game/rules/cartridge/session/subject/scope/time/replay/request/compatibility/byte bindings plus body, schema, depth, signature, digest, and mismatch rejection. |
  | REQ-004 | PASS | Two fresh Git clones consumed the packaged crate and produced byte-identical signed SDK exports with the same lock/provenance identity and no repository path dependency or leak. |
  | REQ-005 | PASS | Workspace, server, provider, Door Legends, route/authority inspection, and the complete local diff gate passed; Door Legends remains the sole provider and the SDK exposes no admission, discovery, trust, or publication operation. |

- Docs: the OpenWiki update completed under run
  `43dd015e-659a-47d9-8cf4-502723c564bd`, reconciled quickstart, cartridge,
  product-boundary, and validation pages, removed its temporary plan, and
  wrote a matching completion receipt for gated state
  `33cd46ef6f90eda324e1a250dad186d7938e3c31da53c2957c6626cd7af308e6`.
  Existing evidence-debt warnings remained on older page claims but did not
  invalidate the completed lifecycle or the new Ticket 044 claims.
- AAR: submitted with six captured failures, five prevention rules, and one
  architecture decision; every new ID was appended to the knowledge register.
- Roadmap/intake: marked only the contract/negotiation/release sub-slice
  complete. Starter/conformance/second-game and sidecar work remain open;
  external acceptance, marketplace operations, and provider onboarding remain
  blocked on their stated real-world prerequisites.
- Archive: ticket moved to `tickets/closed`; spec and notes moved to
  `pipeline/completed`; no active pipeline remains. Phase 5 is PASS.
- Final delivery gate: the first post-OpenWiki invocation ran inside the
  restricted command sandbox and was red only where that sandbox denied
  loopback sockets, Qt/display access, Cargo index DNS, Docker, and systemd/
  Bubblewrap. After it exited normally, the identical canonical
  `bin/gate.sh --diff` command ran with the required local-machine access and
  completed every stage with `GATE GREEN [diff]`. The gate and OpenWiki
  completion receipts both match gated state
  `33cd46ef6f90eda324e1a250dad186d7938e3c31da53c2957c6626cd7af308e6`.

## Defect and lesson ledger

| # | What happened | Root cause | Fix | Prevention |
|---|---|---|---|---|
| 1 | Compatibility could outlive its trust snapshot. | Preflight and effect admission were separate authority moments. | Re-admit under locks and return exact transport/verification material. | `PR-omarchy-gaming-system-finalize-provider-effects-from-current-locked-trust-001` |
| 2 | Two provider POSTs exceeded one lease budget. | Per-call timeouts were composed sequentially. | Share one aggregate deadline shorter than the lease. | `PR-omarchy-gaming-system-budget-provider-preflight-and-operation-together-001` |
| 3 | SDK inventory verification had breadth and path-alias gaps. | Traversal preceded exact bounded native-path admission. | Enforce native allowlists and finite traversal budgets during the walk. | `PR-omarchy-gaming-system-bound-native-signed-artifact-inventory-001` |
| 4 | Mandatory compatibility changed historical replay bytes. | A wire-schema revision was reused to re-encode persisted intent. | Preserve durable v1 preimages and upgrade only authenticated local representations. | `PR-omarchy-gaming-system-preserve-durable-wire-preimages-across-upgrades-001` |
| 5 | Exact lost-ack legacy callbacks failed before deduplication. | Strict parsing preceded immutable duplicate resolution. | Authenticate with the current key and permit only exact local duplicate disposition. | `PR-omarchy-gaming-system-admit-legacy-provider-messages-as-local-duplicates-only-001` |

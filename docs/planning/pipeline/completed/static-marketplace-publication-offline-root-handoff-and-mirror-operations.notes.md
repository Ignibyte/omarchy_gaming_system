---
title: Static marketplace publication, offline-root handoff, and mirror operations — notes
pipeline_id: e02178df-dc45-4ddb-b2bd-43bc01a11e24
---

# Static marketplace publication, offline-root handoff, and mirror operations — running notes

Chronological evidence and decisions. If a command or drill did not run, these
notes must not claim it passed.

## Phase 1 — Plan

- Recall: Ticket 036 was delivered at `1f49536`; local `HEAD`, fetched
  `origin/main`, and GitHub `refs/heads/main` matched and the worktree was clean
  before Ticket 037 opened. No active pipeline or open ticket remained.
- Recall: the earliest unchecked roadmap item is the first external
  two-clean-installation acceptance run. It requires external people and
  machines and cannot be truthfully completed from this workstation, so the
  next locally actionable ordered item is marketplace/package publication
  operations.
- Recall: owner-operated servers already fetch exactly
  `snapshot.signed.json` plus each release's fixed component names beneath one
  canonical marketplace origin. The official client fetches one package-pinned
  trust document and exact package relative paths beneath an independent
  channel origin.
- Recall: Ticket 036 added deterministic root/channel primitives and a CLI, but
  its signing command consumes a hand-authored trust payload. Marketplace
  snapshot signing exists only as a library/test path, while hosted fixtures
  are assembled procedurally. There is no complete review, handoff, static
  tree, mirror, probe, or incident-response producer.
- Recall: the Game Cartridge SDK export is already released and locked. Hosted
  publication is a distribution/operations concern and must live in a new
  non-SDK crate rather than changing the public SDK identity.
- Recalled prevention rules:
  `PR-omarchy-gaming-system-separate-publisher-marketplace-server-attestations-001`,
  `PR-omarchy-gaming-system-pin-executable-artifacts-before-install-001`,
  `PR-omarchy-gaming-system-snapshot-mutable-build-inputs-before-verification-001`,
  `PR-omarchy-gaming-system-bind-current-policy-to-signed-current-snapshot-001`,
  `PR-omarchy-gaming-system-bind-fresh-enrollment-to-package-floors-001`,
  `PR-omarchy-gaming-system-preserve-ineligible-trust-as-transition-evidence-001`,
  `PR-omarchy-gaming-system-persist-authenticated-denial-before-enforcement-001`,
  and `PR-omarchy-gaming-system-serialize-monotonic-policy-transitions-001`.
- Decision: the smallest shippable slice is an immutable static publication
  set with a public online-to-offline handoff, exact local activation, strict
  verifier/probe, and deterministic mirror/incident drills. It reuses the
  current consumer paths and does not introduce a mutable marketplace service.
- Decision: online catalog signing and offline root signing are separate
  commands and workspaces. The offline command accepts only public canonical
  request bytes, reads an explicit private root key, performs no network work,
  and emits only public signed response/receipt bytes.
- Decision: real domains, cloud/CDN accounts, operator staffing, HSM/KMS or
  escrow, paging, and production keys remain external rollout requirements and
  cannot be marked proven by local fixtures.
- Phase 1 is PASS. Fifteen observable requirements define the local engineering
  and rehearsal boundary while keeping the real production custody/hosting
  rollout explicit.

## Phase 2 — Design

- CodeGraph evidence:
  - `synchronize_with_client` consumes one fixed `snapshot.signed.json`, then
    fetches `cartridge.ogsc`, `conformance.json`, and `release.signed.json`
    beneath each signed relative `release_path`. It independently verifies the
    catalog signature, publisher signature, SDK/conformance/host compatibility,
    signed policy, exact identity/digests, and active trust key before database
    publication. Ticket 037 should emit exactly this tree rather than add a new
    server protocol.
  - `ClientTrustStore` and `ClientPackageChannel` consume one fixed trust
    manifest beneath the package bootstrap's channel origin and exact package
    `relative_path` records. Root-signed package metadata already carries size,
    digest, platform, architecture, version, source revision/source digest, and
    build-provenance digest. The producer must derive bytes/digest and verify
    the remaining public provenance instead of defining a second package
    contract.
  - `verify_release_directory` and `verify_release_components` are the
    authoritative review boundary: exact three-file inventory, publisher
    signature, SDK identity, reconstructed canonical conformance, compatible
    host profile, source/builder identity, archive digest, and signed identity.
    Publication must call this boundary before policy/snapshot signing and copy
    only the bytes returned or re-read from a private owned snapshot.
  - `sign_catalog_policy` and `sign_marketplace_snapshot` provide the catalog
    signing operations, but snapshot signing currently has no production CLI.
    `sign_marketplace_trust`, `verify_marketplace_trust_bytes_at_rest`, and
    `verify_trust_transition` already own root payload validation and monotonic
    key history. A new non-SDK crate can compose these contracts without
    modifying the released Game Cartridge SDK export.
  - `GuardedChannelClient` already enforces canonical HTTPS, public DNS, no
    proxy/redirect/ambient credentials/decompression, exact content type,
    timeouts, and bounded streaming. Publication probes can reuse it for both
    channel and marketplace origins; loopback remains available only through
    its hidden conformance constructor.
  - Direct inspection covered CLI/Bash/static-layout sources that CodeGraph
    does not model reliably. The existing channel CLI signs a canonical payload
    but does not create one; the cartridge CLI signs policies but not snapshots;
    and the root-channel shell fixture hand-assembles payload JSON. No current
    producer creates or atomically activates a complete hosted tree.

- Architecture and ownership:
  - Add a private-workspace, non-SDK crate
    `omarchygs-marketplace-publisher`. It owns canonical publication plans,
    prepared online state, offline requests/responses, static publication
    manifests/receipts, descriptor-bound version storage, local activation,
    local verification, and guarded remote probes. It depends on the existing
    Game Cartridge and marketplace-trust crates and does not enter the exported
    SDK identity.
  - The online `prepare` phase receives one canonical public plan, a private
    mode-0700 input root, the supported SDK, an explicit absolute mode-0600
    catalog private key, a public root key, and optional previous signed trust.
    It copies every input once into a private owned staging directory, verifies
    releases/packages/keys/paths/versions, signs each lifecycle policy and the
    exact marketplace snapshot, then emits public prepared files plus one
    canonical offline request. It never reads the root private key.
  - The `offline-sign` phase receives only the canonical offline request, an
    explicit absolute mode-0600 root private key, and an output path. Its binary
    has no HTTP dependency or network operation. It reconstructs and verifies
    the root public identity, previous trust, transition, keyring, package
    inventory, snapshot version ownership, request digest, and ceremony time;
    then emits one root-signed trust document plus a public request-bound
    response receipt. It does not receive the prepared workspace or catalog
    private key.
  - The online `finalize` phase re-verifies the request, prepared inventory,
    root response, prior transition, catalog snapshot, every release, and every
    package. It materializes a private temporary version tree and renames it
    into `versions/<bundle-version>-<publication-digest>` only after complete
    verification and `fsync`. Existing immutable versions are never rewritten.
  - `activate` serializes on a descriptor-bound lock, verifies the candidate and
    current version, prohibits bundle/snapshot rollback or transition-history
    discontinuity, then atomically renames one restricted relative `current`
    symlink to select the complete version. At most 16 finalized versions are
    retained; reaching the ceiling fails without deleting operator evidence.
  - A static server exposes channel origin
    `.../current/channel/` and marketplace origin
    `.../current/marketplace/`. Each subtree carries identical canonical
    `publication.json`; the channel subtree also carries `trust.signed.json`
    and exact packages, while the marketplace subtree carries
    `snapshot.signed.json` and exact release components. The only allowed
    symlink is the validated store-root `current` pointer.
  - `verify` walks the local descriptor-bound tree, rejects unlisted, missing,
    duplicate, non-regular, symlink, hardlink, wrong-mode, oversized, or
    digest-divergent content, authenticates every root/catalog/publisher claim,
    and emits a secret-free public receipt. `probe` fetches both identical
    manifests through separate guarded clients, verifies every listed object
    with its declared media type and bound, reconstructs the same authenticated
    publication identity, and compares identities across mirrors supplied by
    the operator. It does not teach consumers mirror fallback.

- Canonical contracts:
  - `omarchygs.marketplace-publication-plan/v1` binds channel/marketplace
    origins, publication and ceremony timestamps, next bundle/snapshot
    versions, validity, the complete bounded trust keyring, ordered release
    review plans, ordered native package plans, fixed output paths, and optional
    previous-trust digest. It contains relative input names only and no private
    bytes or absolute paths.
  - Each release plan binds a unique output `release_path`, relative owned-input
    directory, relative publisher-key file, policy version/status/reason,
    reviewer and review summary. Display/game/publisher/rules/cartridge and
    digest identity are derived only from the verified release.
  - Each package plan binds a relative owned-input file, unique channel
    `relative_path`, platform, architecture, package version, source revision,
    source digest, and build-provenance digest. Size and package digest are
    derived from the snapshotted exact bytes.
  - `omarchygs.marketplace-offline-request/v1` binds the plan digest, exact
    prepared public-file inventory/digest, complete trust payload, optional
    previous signed trust, and ceremony time. The offline response binds the
    exact request SHA-256 and contains only canonical root-signed trust bytes
    plus public root/trust fingerprints.
  - `omarchygs.marketplace-publication/v1` binds publication ID, bundle and
    snapshot versions, channel/marketplace origins, root/catalog fingerprints,
    trust/snapshot digests, creation time, and a sorted exact file inventory of
    namespace, relative path, media type, bytes, and SHA-256. The manifest
    excludes its two identical copies to avoid self-reference; local and remote
    verifiers require precisely those copies in addition to the listed files.
  - Public operation receipts use one common versioned envelope with operation,
    publication identity, bundle/snapshot versions, public fingerprints,
    counts, and timestamp. They omit private key bytes, credentials, absolute
    key/input paths, environment values, and free-form rich output.

- Exact file manifest:

  | File | Purpose |
  |---|---|
  | `Cargo.toml`, `Cargo.lock` | Register and lock the new non-SDK publisher crate. |
  | `crates/marketplace-publisher/Cargo.toml` | Narrow dependencies and binary/library targets. |
  | `crates/marketplace-publisher/src/lib.rs` | Canonical contracts, validation, prepare/offline/finalize/activate/verify logic, immutable store, receipts, and unit/concurrency/recovery tests. |
  | `crates/marketplace-publisher/src/probe.rs` | Guarded hosted-origin and multi-mirror verification with conformance-only loopback injection. |
  | `crates/marketplace-publisher/src/bin/omarchygs-marketplace-publisher.rs` | Exact CLI dispatch and stable JSON reporting for `prepare`, `offline-sign`, `finalize`, `activate`, `verify`, and `probe`. |
  | `crates/server/Cargo.toml`, `crates/server/src/marketplace_sync_tests.rs` | Dev-only producer dependency and PostgreSQL/TLS proof that the generated static marketplace is consumed unchanged. |
  | `scripts/test-marketplace-publication.sh` | Clean-workspace CLI, deterministic double-build, exact-tree/mirror, no-network offline, rollback, rotation/revocation, interruption, and secret-absence drill. |
  | `bin/gate.sh` | Ratchet the publication drill into the canonical gate. |
  | `docs/operators/marketplace-publication.md` | Custody roles, review procedure, static hosting layout/media types, rollout/monitoring/rollback/incident ceremony, recovery, and external production checklist. |
  | `README.md`, `docs/architecture/game-cartridges.md`, `docs/architecture/system-overview.md`, `docs/operators/owner-operated-servers.md`, `docs/planning/ROADMAP.md`, OpenWiki | Reconcile the new operational boundary and preserve real infrastructure/custody rollout as explicit remaining work. |

- Database, API, client, and compatibility consequences:
  - No migration or network API changes. PostgreSQL continues to ingest only
    the existing signed snapshot/release contracts, and the server's existing
    advisory-lock/database invariants remain authoritative.
  - No QML or companion endpoint changes. Clients consume the same package
    bootstrap, trust manifest, and exact package metadata. No mirror list or
    alternate root enters discovery, QML, or selected-server data.
  - The released Game Cartridge SDK export and existing publisher/cartridge
    formats remain byte-identical. The new crate is operator tooling only.
  - Manual-key/no-channel deployments and hand-operated static origins remain
    compatible. Ticket 037 tooling is additive and produces the current exact
    consumer layout rather than requiring it at server startup.

- Regression plan:

  | Requirement | Evidence |
  |---|---|
  | REQ-001 | Canonical plan round-trip plus unknown/missing/duplicate/unsorted/control/path/version/count/size hostile corpus. |
  | REQ-002 | Production verifier on valid release plus publisher, SDK, conformance, attestation, compatibility, review, and input-swap failures. |
  | REQ-003 | Catalog key absolute/mode/symlink/identity cases; deterministic policy/snapshot output; generated tree consumed by server TLS sync. |
  | REQ-004 | Exact offline-request schema/digest, previous-trust/keyring/package/snapshot binding, reproducibility, and private-material absence. |
  | REQ-005 | Offline binary dependency/network inspection, root mode/path/root mismatch, transition/rollback/time/tamper, exclusive output, and deterministic signature cases. |
  | REQ-006 | Response/request/root/trust/prepared-manifest substitution, stale response, wrong transition, split identity, and preserved candidate on failure. |
  | REQ-007 | Two byte-identical prepared/final trees; exact names, media, size, hash, modes, hardlink count, extra/missing/tamper rejection. |
  | REQ-008 | Concurrent finalization/activation, partial temp tree, existing version collision, current-link swap, rollback denial, restart, and 16-version ceiling. |
  | REQ-009 | Local verifier complete authentic chain plus symlink/hardlink/mode/path/inventory/divergence corpus and exact receipt privacy. |
  | REQ-010 | Spawned TLS channel/marketplace origins with private/mixed DNS, wrong root, redirect, proxy, encoding, media, timeout, size, digest, and content tamper. |
  | REQ-011 | Two identical loopback mirrors pass; partial rollout, stale version, manifest split, artifact split, and alternate-root substitution fail. |
  | REQ-012 | Initial publication followed by higher snapshot/bundle with old-key revocation/new active key; stale activation/probe denied and historical evidence retained. |
  | REQ-013 | Exact receipt schemas across every operation plus secret scanner and absolute-path/credential/private-field absence assertions. |
  | REQ-014 | Existing trust, package, SDK, server sync, client runtime, QML, provider, recovery, and private-alpha suites remain green. |
  | REQ-015 | Clean-room shell drill, authored docs/OpenWiki reconciliation, security diff inspection, and canonical diff gate. |

- Security, concurrency, recovery, and operational risks:
  - Catalog and root private keys are distinct high-value inputs. Commands
    require explicit absolute regular owner-only files; no private path or key
    enters plans, receipts, argv echoes, static trees, logs, environment-derived
    defaults, or network requests. The offline binary intentionally omits probe
    construction from its command path and the drill audits outbound attempts.
  - The online operator controls catalog signatures by design but cannot create
    a root-authorized keyring/package channel. The offline operator must review
    the public request; finalization cannot combine a response with different
    prepared bytes or previous history.
  - Publication identity is computed over exact canonical manifest and every
    authenticated artifact. A manifest is not independently trusted until its
    complete root/catalog/publisher chain and file inventory verify.
  - Static activation uses one cross-process lock and descriptor-relative
    rename/fsync sequence. No network I/O or private-key read occurs while the
    activation lock is held. Failed staging leaves a bounded private temporary
    directory that verification ignores and a later explicit recovery can
    inspect/remove; current remains unchanged.
  - A valid current static tree can expire before a new root bundle arrives.
    Probe reports unhealthy and consumers fail closed; operations may activate
    only a higher already-complete publication, never rewrite timestamps or
    roll back to an older still-valid signature.
  - Mirrors increase availability but not authority. Every mirror must expose
    the same authenticated identity; client/runtime fallback and geographic
    routing remain deployment concerns outside this tool.
  - Same-UID local operators can alter their own workspace and keys. The tool
    prevents accidents, path substitution, partial publication, and ambiguous
    receipts; it cannot protect a production signing host from a fully
    compromised administrator. Real offline media/HSM, dual control, backup,
    and staff procedure remain external checklist items.

- Material alternatives rejected:
  - A mutable Axum marketplace service was rejected because current consumers
    need immutable files and such a service would add database/auth/admin/API
    authority unrelated to this slice.
  - Letting the online catalog process read the root key was rejected because
    routine release compromise would become package/trust-root compromise.
  - One command that prepares, signs, uploads, and activates was rejected
    because it erases custody review and makes partial failure ambiguous.
  - Copying files directly into a live document root was rejected because
    server/client requests could observe mixed bundle, snapshot, release, and
    package generations.
  - Client-side mirror fallback was rejected because it broadens trust and
    egress policy; operations can compare mirrors without changing clients.
  - Object deletion for automatic retention was rejected. The bounded store
    refuses a seventeenth version and preserves evidence until an operator uses
    a future audited archival procedure.
  - Marking the entire hosted roadmap item complete from local fixtures was
    rejected. Ticket 037 can complete the deterministic engineering and drill
    sub-slice, while real production provisioning/custody remains unchecked.
- Phase 2 is PASS. The fifteen requirements map to one additive non-SDK crate,
  an exact existing-consumer static layout, separated online/offline commands,
  descriptor-bound activation, local/remote verification, rotation incident
  evidence, and explicit external rollout limits. The matching CodeGraph design
  receipt covers the unchanged gated worktree and its producer/consumer blast
  radius.

## Phase 3 — Implement

- Added the non-SDK `omarchygs-marketplace-publisher` library and CLI with
  canonical plan, prepared-publication, offline request/response, publication
  manifest, probe-floor, and stable receipt/error envelopes. The root workspace
  and lockfile register only this operator tool; no Game Cartridge SDK export,
  server API, database migration, QML surface, or consumer protocol changed.
- `prepare` now verifies the exact SDK and each publisher release, derives and
  signs lifecycle policy plus one canonical catalog snapshot, snapshots package
  inputs through bounded streaming, validates the proposed root trust payload,
  and emits a deterministic public offline request from an owner-private
  workspace without reading the root private key.
- `offline-sign` revalidates the public request, root identity, validity,
  previous trust, and monotonic transition before writing one request-bound
  signed response. The CLI drill executes this command inside `bwrap
  --unshare-net`; the command path contains no HTTP construction.
- `finalize`, `activate`, and `verify` use an owner-private static store,
  cross-process `flock`, bounded temporary/final version names, exact file
  inventory and authenticated-chain verification, `fsync`, atomic rename, a
  restricted `current` link, rollback/transition denial, and a hard ceiling of
  sixteen retained versions without automatic evidence deletion.
- `probe` reuses the existing guarded channel transport, authenticates the
  identical channel/marketplace manifests plus every root/catalog/publisher
  artifact, streams native packages, requires caller-held bundle/snapshot and
  optional publication-digest floors, and compares every supplied mirror to
  one authenticated identity. It does not add client mirror fallback.
- Added seven integration tests covering deterministic double preparation and
  finalization, exact-tree/mode/link rejection, concurrent first finalization,
  root/catalog key modes and response identity, a real network-less CLI
  ceremony, two TLS mirrors with stale/tampered failure cases, and catalog-key
  compromise rotation/revocation with rollback denial and advancing package
  versions. Added `scripts/test-marketplace-publication.sh` as gate stage 15b.
- Added the publication/custody/mirror/incident operator runbook and reconciled
  the roadmap, system overview, ADR-0003, owner-server operations, and packaged
  client installation guidance. Local deterministic tooling/drills are marked
  complete; production domains, object hosting/CDN behavior, real root custody,
  staffing, monitoring, and incident coordination remain explicitly unchecked.
- Design deviations:
  - Store and probe logic were split into `store.rs` and `probe.rs`, with the
    complete regression harness in `tests/publication.rs`, to keep the contract
    module reviewable.
  - The proposed new server dev dependency/synchronization fixture was not
    needed. The producer emits the already-fixed consumer paths and authentic
    contracts; its exact TLS tree is proven by publisher tests while the
    unchanged workspace server-sync suites continue to prove consumption.
  - `README.md` and `docs/architecture/game-cartridges.md` did not need another
    distribution description; the narrower system overview, ADR, installation,
    owner-operator, and dedicated publication runbook own the affected boundary.
- Focused evidence after the last implementation edit:
  - `cargo fmt --all -- --check` — PASS.
  - `cargo test --locked -p omarchygs-marketplace-publisher --test publication`
    — PASS, 7 tests.
  - `cargo clippy --locked -p omarchygs-marketplace-publisher --all-targets --
    -D warnings` — PASS.
- Phase 3 is PASS; implementation and focused checks are ready for skeptical
  inspection.

## Phase 3.5 — Inspect

- Correctness/EARS review found the implementation inside the approved
  publication-only boundary: plan identities are canonical and bounded; the
  root response is request-bound; manifests authenticate exact inventory;
  finalization verifies before atomic selection; probe floors prevent global
  rollback; and all receipts are stable public-only envelopes.
- Concurrency/recovery review verified one `flock` boundary across store
  finalization and activation, convergence when concurrent processes finalize
  the same identity, failure without current-pointer mutation, highest-version
  recovery only when `current` is absent, monotonic trust transition checks,
  and refusal rather than automatic deletion at the retention ceiling.
- Codex Security exact-diff scan
  `dde7506a-580e-45bd-a78e-40483fdc67bd` reported one medium,
  high-confidence local race: `write_new_file` and streamed copies created an
  inode safely but then changed mode through the mutable pathname. A process
  able to modify shared handoff media could replace the path before `chmod`
  and cause the offline custodian to weaken another owned file's permissions.
- Fixed the finding by setting file permissions through the already-open
  `File` descriptor before `sync_all`. Directory creation now requests the
  restrictive mode at `mkdir`, reopens with `O_DIRECTORY | O_NOFOLLOW`, and
  applies the exact mode through that bound descriptor. Focused tests and
  Clippy passed after the fix.
- Post-remediation Codex Security exact-diff scan
  `44b3f354-9539-4b05-b16d-1f07116bceea`, snapshot
  `codex-security-snapshot/v1:sha256:996c9c5e8ce4f9a03fa11fb93fe7e1bc85b572f4b679f7884aea2b06aa6e0515`,
  reviewed all nine executable/configuration surfaces with complete coverage,
  no deferred work, and zero findings. TAC output access could not be verified
  because its connector was unavailable; this affected hosted report access,
  not local review coverage.
- Fresh CodeGraph inspection traced preparation, offline signing,
  finalization, activation, local verification, guarded probing, trust
  validation, CLI callers, and publication integration tests. It confirmed
  the final descriptor-bound sources and one-hop blast radius. Its automatic
  test mapping did not attach tests directly to private helpers
  `create_directory`, `copy_public_file`, or `ensure_private_directory`; direct
  inspection confirmed that the seven end-to-end integration tests exercise
  them through every prepare/finalize/activation path. The matching inspect
  receipt covers pipeline `e02178df-dc45-4ddb-b2bd-43bc01a11e24` and the final
  gated implementation state.
- No authentication/API/database/QML finding applied because the slice adds no
  request handler, migration, client state, or player-facing surface. Existing
  server and client authorities remain consumers of the unchanged signed
  contracts.
- Phase 3.5 is PASS. The confirmed finding is fixed, false/coverage warnings
  are dispositioned, and fresh CodeGraph/security evidence covers the final
  executable change set.

## Phase 4 — Validate

- `bin/gate.sh --diff` completed after the last gated implementation edit and
  printed `GATE GREEN [diff]`. Its worktree-bound receipt hash is
  `827075565dd0cd43fd785fc9320669b11b10b8c22351cdbc4717907918c12dbe`,
  matching the fresh CodeGraph inspection receipt.
- The gate passed rustfmt, workspace Clippy with warnings denied, all ordinary
  workspace tests, rustdoc warnings-as-errors, compose/tool availability,
  shell/pipeline/secret/hook/whitespace checks, production cartridge and
  renderer proofs, SDK reproducibility, the architecture spike, native client
  source and reproducible package builds, root trust-channel and static
  publication drills, PostgreSQL integration, real Rust API/QML smoke, remote
  provider security and clean-clone pilot, backup/restore, and private-alpha
  admission.
- Ticket-specific evidence included 2 contract unit tests and 7 publication
  integration tests, a real `bwrap --unshare-net` offline ceremony, two
  guarded TLS mirrors, deterministic duplicate builds, immutable store and
  concurrency checks, and the catalog-compromise/rollback drill. The existing
  PostgreSQL marketplace rotation/synchronization tests also passed against the
  unchanged consumer contract.
- No validation skip applies to the canonical diff gate; database, QML,
  provider, packaging, and recovery suites all ran in their dedicated gate
  stages.
- Phase 4 is PASS; a matching delivery receipt exists for the final executable
  worktree.

## Phase 5 — Complete

- Acceptance audit:

  | Requirement | Disposition and concrete evidence |
  |---|---|
  | REQ-001 | Satisfied by exact canonical plan parsing/validation, bounded identifiers, paths, ordering, counts, versions, and contract unit hostile cases; plans contain only public relative identities. |
  | REQ-002 | Satisfied by `prepare_publication` calling the production SDK and release verifier before policy/snapshot signing; integration cases reject symlinked, changed, incompatible, and foreign release inputs. |
  | REQ-003 | Satisfied by explicit absolute owner-only catalog-key admission, derived public-key equality, canonical snapshot output, exact existing release paths, and the unchanged server synchronization tests in the full gate. |
  | REQ-004 | Satisfied by the canonical request's plan/prepared digests, complete trust payload, previous signed transition, root identity, ceremony time, and secret-free deterministic duplicate preparation. |
  | REQ-005 | Satisfied by exact root-key mode/path/identity checks, independent request/transition validation, create-new response, stable public receipt, and a real CLI invocation inside `bwrap --unshare-net`. |
  | REQ-006 | Satisfied by request/root/trust digest equality, full root verification, payload equality, previous transition checks, prepared inventory verification, and foreign-response/tamper rejection. |
  | REQ-007 | Satisfied by byte-identical duplicate publications, fixed two-namespace manifests, exact media/size/digest inventory, read-only single-link files, owner-private directories, and extra/missing/mode/link rejection. |
  | REQ-008 | Satisfied by complete private staging, self-verification, `fsync` and atomic rename, cross-process `flock`, current/candidate monotonic verification, concurrent convergence, unchanged current on failure, recovery, and the 16-version refusal ceiling. |
  | REQ-009 | Satisfied by local verification of the root/catalog/publisher chain and every file plus exact-tree, symlink, hardlink, mode, tamper, rollback, and stable receipt cases. |
  | REQ-010 | Satisfied by the reused guarded HTTPS client and its existing DNS/TLS/proxy/redirect/decompression/media/timeout/size corpus plus publication TLS mirror tamper and bounded streaming cases. |
  | REQ-011 | Satisfied by two independent TLS channel/marketplace mirror pairs that pass only with one authenticated manifest identity; stale floor and tampered mirror cases fail. |
  | REQ-012 | Satisfied by the higher bundle/snapshot compromise drill that retires/revokes the old catalog key, activates a successor, advances package version/floors, rejects stale activation/probe, and retains prior versions. |
  | REQ-013 | Satisfied by one exact public receipt envelope for all six operations, stable JSON errors, secret/absolute-path absence assertions, and the gate secret scan. |
  | REQ-014 | Satisfied by the complete workspace, SDK, client package, PostgreSQL marketplace/server, QML, provider, recovery, and private-alpha stages in `bin/gate.sh --diff`; no migration/API/SDK identity changed. |
  | REQ-015 | Satisfied by the dedicated operator runbook, reconciled architecture/roadmap/client/operator docs, completed OpenWiki run `44c627df-5955-4feb-a06b-3a456410d35f`, clean post-remediation security scan, and canonical diff gate. |

- OpenWiki update run `44c627df-5955-4feb-a06b-3a456410d35f`
  refreshed all 84 prior stale/unresolved evidence anchors, added the static
  publication, custody, mirror, authority, and stage-15b claims, reconciled the
  affected generated pages, and returned `status: complete`. The matching
  completion receipt covers this pipeline and the post-wiki gated state.
- AAR-037 was submitted on 2026-08-27 with one recorded failure, one prevention
  rule, one architecture decision, and effectiveness marked effective. All
  three new IDs were appended to the knowledge register.
- Real production infrastructure remains intentionally outside acceptance:
  no domain, bucket/CDN, HSM/media ceremony, production key, monitoring account,
  pager, staffing, malware-review operation, or incident coordinator was
  created or claimed.
- `TICKET-037` is closed and the active spec/notes pair is archived together.
- Phase 5 is PASS.

## Defect and lesson ledger

| # | What happened | Root cause | Fix | Prevention |
|---|---|---|---|---|
| 1 | The first diff scan found that a new offline response or copied public file was followed by path-based `chmod`. | File identity was bound for writing but not for its final permission change, creating a post-create symlink race in a shared parent. | Use `File::set_permissions` on the open descriptor; create directories restrictive and reopen them with `O_DIRECTORY | O_NOFOLLOW` before exact descriptor-bound mode application. | `PR-omarchy-gaming-system-bind-permissions-to-opened-file-descriptors-001` |

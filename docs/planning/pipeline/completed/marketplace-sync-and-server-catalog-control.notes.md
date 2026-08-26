---
title: Marketplace synchronization and server catalog control — notes
pipeline_id: 48c86d57-dc5a-40bc-9779-d65b1e635b63
---

# Marketplace synchronization and server catalog control — running notes

Chronological evidence and decisions. If a test or command did not run, these
notes must not claim it passed.

## Phase 1 — Plan

- Recall: Ticket 031 shipped at `2c3c88dd8b753c7a54802bc9c75083c8b1d78413`,
  local and `origin/main` commit/tree identities matched, the worktree was
  clean, and no bulletin or active pipeline blocked Ticket 032.
- Recall: the external two-clean-installation acceptance event remains a real
  human/machine dependency, so the next independently executable roadmap
  outcome is the server-admin marketplace/catalog path.
- Recall: ADR-0003 makes marketplace review, server admission, and publisher
  integrity separate claims. It forbids marketplace publication from forcing
  admission and keeps every cartridge inert and locally rendered by trusted
  platform QML.
- Recall: Ticket 017 already supplies exact publisher release verification,
  domain-separated catalog lifecycle signatures, monotonic denial caching,
  compatibility evaluation, and descriptor-relative secure import. Ticket 032
  must compose those production boundaries rather than create a weaker
  verifier or pathname-based store.
- Recalled rules:
  `PR-omarchy-gaming-system-separate-publisher-marketplace-server-attestations-001`,
  `PR-omarchy-gaming-system-serialize-monotonic-policy-transitions-001`,
  `PR-omarchy-gaming-system-persist-authenticated-denial-before-enforcement-001`,
  `PR-omarchy-gaming-system-validate-retained-directory-authority-001`, and
  `PR-omarchy-gaming-system-distinguish-not-found-from-denial-001`.
- `scripts/check-pipeline-tools.sh` passed with CodeGraph 1.5.0, OpenWiki
  0.3.3, and Codex-only provenance active.
- Decision: keep the first production synchronization contract to one pinned
  marketplace per server. This avoids inventing cross-authority precedence
  while preserving an additive path to multiple sources later.
- Decision: use a signed monotonic bounded snapshot whose entries refer only
  to relative release locations beneath the configured HTTPS origin. The
  cartridge itself never selects a destination.
- Decision: synchronization can install immutable unreferenced content before
  the final database transaction, but a failed snapshot never publishes
  partial reviewed inventory or local admission.
- Decision: local activation is an explicit audited database transition. A
  rollback selects a prior imported digest; it does not decrement marketplace
  snapshot or lifecycle versions.
- Decision: expose an authenticated metadata-only cartridge catalog in this
  slice so server admission is observable. Package transfer and mounting stay
  in the next roadmap ticket.

## Phase 2 — Design

- Architecture and ownership:
  - `omarchygs-game-cartridge` owns the signed snapshot contract, canonical
    parsing, exact-release comparison, lifecycle verification, and secure
    content-store staging/resolution. It receives no database, account,
    operator, or network authority.
  - The server-admin process owns configuration, guarded marketplace HTTPS,
    snapshot orchestration, PostgreSQL publication, operator commands, and
    plain JSON output. The normal server process owns the authenticated
    metadata-only player catalog and never reads the cartridge filesystem for
    this ticket.
  - PostgreSQL owns the current synchronized snapshot, exact reviewed-release
    inventory, the server's one selected release per game, admission revision,
    and immutable catalog audit. Filesystem content is immutable evidence, not
    local catalog authority.
  - Marketplace policy and snapshot status can make a selected release
    ineffective. It never rewrites the administrator's selection or chooses a
    fallback. An explicit later command is required to activate another
    permitted digest.
- Data flow:
  1. `omarchygs-admin marketplace-sync` loads four checked inputs from the
     environment: canonical HTTPS origin, marketplace public-key file, DER TLS
     root file, and an already provisioned secure-store directory.
  2. A fresh client resolves the configured hostname once, rejects the whole
     answer set if any address is not global unicast, pins the accepted
     sockets, disables proxy/redirect/compression/referer, and streams the
     fixed snapshot path under a byte/time ceiling. Tests alone can construct
     a client for one exact loopback socket.
  3. The cartridge crate verifies canonical signed snapshot bytes under a new
     domain. The payload binds authority, monotonic version, bounded review
     facts, canonical relative release directories, exact identities,
     publisher keys, and signed per-release lifecycle policies. Entries are
     sorted and unique by release identity, digest, and location.
  4. Before release fetches, PostgreSQL is locked briefly to reject an older
     snapshot or an equal version with different bytes. Equal identical input
     is a safe replay.
  5. Each release directory supplies exactly the existing three production
     artifacts. The client fetches them beneath the configured origin with
     individual ceilings; `verify_release_components` reconstructs archive
     conformance and publisher provenance using the SDK identity embedded in
     the running cartridge crate. Every signed snapshot identity and policy is
     compared to the verified release.
  6. `SecureCartridgeStore::stage_reviewed_release` caches the highest
     authenticated policy first and stores immutable bytes only when new
     launches remain permitted. A denied policy may leave older immutable
     bytes but never publishes or activates them. Existing `import_release`
     composes staging with its legacy same-user active pointer so Ticket 017's
     CLI contract remains compatible.
  7. After all entries verify/stage, one PostgreSQL transaction rechecks the
     snapshot and upserts the complete reviewed inventory. Omitted rows remain
     historical but their `last_seen_snapshot_version` is stale, so they are
     ineffective. A failure before commit publishes no partial snapshot.
  8. `catalog-apply` first resolves durable operation replay, serializes by
     game, compares the caller's exact expected selection, resolves the desired
     digest from the secure store with current publisher/policy material, then
     atomically changes one database selection and appends an immutable audit
     receipt. Selecting an earlier imported permitted digest is rollback.
  9. Authenticated `GET /v1/cartridges` joins current-snapshot,
     imported-compatible, lifecycle-permitted rows to the one server
     selection. It returns no-store, bounded provenance/admission metadata and
     no acquisition location or executable document.
- Signed snapshot v1:
  - Outer document: exact canonical JSON with `algorithm`, `key_id`, base64url
    canonical payload, and Ed25519 signature under
    `omarchygs-marketplace-snapshot-v1`.
  - Payload: format, nonzero snapshot version, authority ID, marketplace
    display name, and at most 128 release entries.
  - Entry: canonical relative release directory; exact game/publisher,
    rules/cartridge versions and content identities; the complete publisher
    public key; bounded plain-text reviewer ID and review summary; and the
    existing signed catalog policy. The policy authority and exact release
    tuple must match the entry.
  - No URLs, credentials, code, markup, or arbitrary metadata are admitted in
    an entry. The administrator-configured origin is the only authority used
    to construct requests.
- Database/migration design (`0019`):
  - `marketplace_sync_state` is a checked singleton containing configured
    public identity (origin/authority/key), current version and snapshot digest,
    plus completion time. Secret or TLS-root bytes are not stored.
  - `marketplace_releases` stores exact immutable release identity and public
    publisher provenance, mutable monotonic marketplace lifecycle/review facts,
    relative mirror location, compatibility/import facts, and the last snapshot
    that contained the row. Identity mutation and deletion reject.
  - `server_cartridge_catalogs` owns one nullable selected release per unique
    game plus a monotonically increasing admission revision.
  - `cartridge_catalog_audit_events` owns a globally unique operation ID,
    catalog target, action, actor/reason, nullable previous/resulting digests,
    admission revision, and timestamp. Update/delete/truncate reject.
  - Foreign keys are `RESTRICT`; no package, account, persona, session, or
    gameplay state is deleted through catalog lifecycle changes.
- API/CLI compatibility:
  - Existing `/v1/games` and its compiled/provider gameplay contract do not
    change. New metadata is exposed at authenticated, no-store
    `GET /v1/cartridges`; client acquisition and launch integration will build
    on this versioned boundary later.
  - Existing `omarchygs-admin reports`, `invites`, and `apply` remain exact.
    New top-level actions are `marketplace-sync`, `cartridges`, and
    `catalog-apply <bounded-document>` with stable plain error codes.
  - Server discovery adds a truthful `cartridge_catalog` capability only after
    the API is present.
- File manifest:

  | Path | Purpose |
  |---|---|
  | `crates/game-cartridge/src/marketplace.rs` | Signed bounded snapshot v1 types, signer, canonical verifier, relative-location and exact-entry validation. |
  | `crates/game-cartridge/src/sdk.rs`, `src/lifecycle.rs`, `src/lib.rs` | Expose the embedded supported SDK identity, crate-internal signing primitive, and public snapshot/staging contracts. |
  | `crates/game-cartridge/src/secure_store.rs` | Separate immutable reviewed staging/exact resolution from the legacy active-pointer import. |
  | `crates/game-cartridge/tests/marketplace.rs`, `tests/sdk_release.rs` | Snapshot hostile corpus plus staging, denial, restart, no-activation, and exact-resolution regressions. |
  | `crates/game-provider/src/egress.rs`, `tests/egress.rs` | Name/export the already proven generic public-egress IP classifier while retaining provider compatibility. |
  | `crates/server/src/marketplace_egress.rs` | HTTPS-only, pinned-root, DNS-pinned, no-redirect, bounded GET client with exact test-only loopback construction. |
  | `crates/server/src/marketplace_sync.rs` | Checked operator config, snapshot/release orchestration, secure-store composition, atomic synchronization receipt. |
  | `crates/server/src/cartridge_catalog.rs` | PostgreSQL inventory, snapshot publication, effective catalog query, idempotent admission/rollback, and audit receipts. |
  | `crates/server/src/app.rs`, `src/main.rs`, `src/server_discovery.rs` | Route authenticated metadata catalog, register modules/tests, and advertise the implemented capability. |
  | `crates/server/src/cartridge_catalog_api_tests.rs`, `src/marketplace_sync_tests.rs`, `tests/operator_cli.rs` | Exact API, TLS sync, database lifecycle/race/replay, and CLI evidence. |
  | `crates/server/src/bin/omarchygs-admin.rs`, `crates/server/Cargo.toml`, `Cargo.toml`, `Cargo.lock` | Admin actions and production cartridge/network dependencies. |
  | `migrations/0019_marketplace_catalog.sql` | Forward-only synchronized inventory, server selection, immutable audit, constraints, indexes, and triggers. |
  | `.env.example`, `README.md`, `docs/api.md`, `docs/architecture/game-cartridges.md`, `docs/operators/owner-operated-servers.md` | Configuration, contracts, lifecycle, trust, rollback, and operational guidance. |
  | `scripts/test-operator-recovery.sh` | Seed/compare marketplace, admission, and audit state across isolated restore. |
  | Active Ticket 032 artifacts, AAR, roadmap, OpenWiki | Workflow evidence, durable lessons, completion, and generated knowledge reconciliation. |
- Regression matrix:

  | Requirement | Evidence |
  |---|---|
  | REQ-001 | Config and CLI tests reject missing variables, non-HTTPS/noncanonical origins, bad/symlink/oversized key and TLS files, invalid roots, unsafe store roots, and mutate nothing. |
  | REQ-002 | Cartridge snapshot unit corpus covers signature/key/authority, canonicality, unknown fields, size/count/text/path limits, sorting, duplicates, tuple mismatch, and versions. |
  | REQ-003 | Separately spawned TLS fixture covers exact success plus private-address production denial, wrong root, redirect, status, timeout, truncation, oversize, path escape, component/signature/digest/conformance/policy tamper. |
  | REQ-004 | Secure-store and PostgreSQL tests cover no active-pointer write during staging, descriptor containment, immutable bytes, all-or-nothing snapshot publication, and harmless unreferenced staged content. |
  | REQ-005 | CLI inventory exact JSON covers current/omitted/imported/effective/deprecated rows and asserts absence of private key, TLS bytes, credentials, absolute paths, and acquisition destinations. |
  | REQ-006 | Database tests cover activate/deactivate/older-release rollback, expected-state conflicts, exact replay/collision, concurrent writers, single selection, revision, and immutable audit. |
  | REQ-007 | Full lifecycle matrix rejects unavailable/incompatible/unimported/mismatched/denied rows and preserves deprecated warning. |
  | REQ-008 | Snapshot/policy downgrade, equal-version conflict, authenticated denial before enforcement, omission, restart, and explicit recovery tests. |
  | REQ-009 | API tests cover missing/bad/revoked bearer, exact active/deprecated response, denial/omission filters, no-store, sorted bounds, and sensitive-field absence. |
  | REQ-010 | Recovery drill compares snapshot identity/version/digest, release inventory, selected digest/revision, and catalog audit before/after restore. |
  | REQ-011 | Focused cartridge/server suites, database/CLI/TLS tests, fast loop, and final worktree-bound diff gate. |
- Security/privacy/concurrency risks and mitigations:
  - SSRF and DNS rebinding: only a checked admin-configured HTTPS origin is
    used; all initial DNS answers must be public, sockets are pinned, proxy and
    redirects are disabled, and response paths are canonical relative values
    from an authenticated snapshot.
  - Supply-chain substitution: publisher, marketplace, exact release,
    conformance, compatibility, and every digest are independently verified;
    database rows publish only after the whole snapshot succeeds.
  - Trust conflation: separate schema fields retain publisher integrity,
    marketplace review/lifecycle, and server admission/revision.
  - Rollback/races: snapshot, policy, and admission transitions have distinct
    locks and monotonic versions; durable replay resolves before current-state
    admission; no implicit fallback exists.
  - Filesystem authority: the server composes the retained-descriptor store;
    local paths are neither stored in public inventory nor returned by APIs.
  - Denial of service: fixed entry/file/byte/time/DNS limits and disabled
    content encoding bound work before JSON/archive/media parsing.
  - Secret/privacy exposure: no private signing material is consumed, TLS root
    and filesystem configuration remain local, and the player endpoint carries
    only public release/review/admission metadata after session authentication.
- Alternatives rejected:
  - Reusing `SecureCartridgeStore::import_release` directly would let
    marketplace review overwrite the filesystem active pointer before server
    admission; staging is separated instead.
  - An unsigned JSON feed plus per-release policies would permit snapshot
    omission/reordering ambiguity and replay; the snapshot itself is signed
    and monotonic.
  - Arbitrary absolute release URLs would turn a trusted marketplace key into
    an unrestricted egress capability; entries are relative beneath one pinned
    origin.
  - Treating a marketplace status as the server catalog would violate ADR-0003;
    local selection is a separate audited PostgreSQL decision.
  - Combining this with client package transfer/mounting would enlarge the
    network, filesystem, QML, and profile threat surfaces beyond one shippable
    server-admin slice.
- Rollback: disable marketplace configuration to stop new syncs, deactivate a
  local game or explicitly select an older currently permitted imported digest,
  and preserve all immutable bytes/audit. The forward-only schema remains;
  application rollback ignores the additive tables and endpoint.
- CodeGraph design evidence: exploration traced the production
  `verify_release_components` and `verify_catalog_policy_bytes` boundaries,
  `SecureCartridgeStore::import_release`/`resolve_active`, the 27-caller test
  router fan-out, existing `/v1/games`, session-authenticated handler pattern,
  operator transaction/replay/audit implementation, and the provider guarded
  egress classifier. The critical blast radius is the secure-store semantic
  split, app router/module registration, operator CLI dispatch, and additive
  server dependency surface. SQL, shell recovery, and exact HTTP fixture
  behavior remain direct-review surfaces.
- Phase 2 is PASS. The design preserves every existing gameplay and legacy
  cartridge CLI contract while making marketplace synchronization incapable of
  local activation by itself.

## Phase 3 — Implement

- Added the bounded canonical marketplace snapshot v1 contract to
  `omarchygs-game-cartridge`, including domain-separated strict Ed25519
  verification, exact authority/publisher/policy linkage, deterministic
  signing, sorted unique inventory, relative-path constraints, and hostile
  contract tests.
- Split secure-store staging from activation. Reviewed synchronization now
  verifies and retains immutable content without writing the legacy active
  pointer; exact digest resolution rechecks supported SDK, host compatibility,
  publisher identity, and current monotonic lifecycle policy. The existing
  Ticket 017 import contract composes the new stage and preserves compatibility.
- Added guarded marketplace egress with canonical HTTPS domain origins,
  public-only DNS-set validation and socket pinning, a single explicit DER TLS
  root, test-only exact loopback, no proxy/redirect/referer/decompression/
  connection reuse, strict status, per-request timeouts, and streaming byte
  ceilings. A real TLS test rejects a wrong root, redirect, and oversized body.
- Added forward-only migration `0019` for singleton snapshot state, immutable
  reviewed releases, one local selection per game, admission revisions, and
  update/delete/truncate-protected catalog audit.
- Added atomic synchronization and catalog domain services. Equal exact
  snapshots replay, older or changed-equal snapshots conflict, all releases
  verify before database publication, and omitted inventory remains historical
  but ineffective. Admission uses global durable replay plus per-game locking,
  exact expected/desired states, secure-store resolution, one transaction, and
  explicit activate/deactivate/upgrade/rollback receipts.
- Extended the database-local administrator CLI with `marketplace-sync`,
  `cartridges`, and bounded `catalog-apply`. Existing report/invitation actions
  retain their command surface. Added the authenticated no-store
  `GET /v1/cartridges` exact metadata response and truthful
  `games.cartridge-catalog.v1` discovery capability without changing
  `/v1/games` gameplay authority.
- Extended the recovery rehearsal to seed, dump, restore, compare, and protect
  marketplace snapshot, reviewed release, local selection, and immutable
  catalog audit state. Updated environment, API, architecture, owner-operator,
  recovery, and top-level workflow documentation with trust, lifecycle,
  rollback, and remaining client-acquisition boundaries.
- Focused implementation evidence:
  - `cargo check --workspace --all-targets` passed after the server library and
    production dependency integration.
  - `cargo test -p omarchygs-game-cartridge` passed 32 tests across unit,
    conformance, marketplace, and SDK/release suites after strict signature
    verification.
  - `cargo test -p omarchy-gaming-system-server --lib` passed four portable
    tests with the one PostgreSQL/TLS lifecycle test correctly ignored.
  - The focused PostgreSQL/TLS lifecycle test passed against the real migrated
    database, covering sync/replay, activation, collision, concurrency,
    deactivation, explicit rollback/recovery, lifecycle denial/no-fallback,
    snapshot downgrade, tamper atomicity, and immutable audit.
  - The focused authenticated API and real operator CLI PostgreSQL tests each
    passed.
  - `./scripts/test-operator-recovery.sh` passed with all application tables,
    catalog state, audit immutability, server identity, and restored session
    denial intact.
- Phase 3 is PASS. Implementation matches the designed ownership split and is
  ready for independent inspection; broad validation remains Phase 4 evidence.

## Phase 3.5 — Inspect ledger

- Fresh CodeGraph exploration traced the complete production path from
  `synchronize_with_client` through guarded DNS/TLS acquisition, strict
  snapshot verification, immutable staging, atomic `publish_snapshot`,
  expected-state `apply_catalog_command`, and authenticated
  `list_player_catalog`. The structural blast radius remains contained to the
  shared egress classifier, the new server catalog/sync modules, administrator
  dispatch, and authenticated route registration. CodeGraph's test labels do
  not index the registered in-module and integration fixtures, so executable
  coverage was confirmed directly.
- Direct inspection covered all SQL, shell recovery, exact HTTP fixtures,
  configuration, and documentation outside CodeGraph's indexed Rust flow.
- Codex Security diff scan
  `b1cf25e3-e128-43c6-8d89-020bbd1df0a7` sealed complete against working-tree
  digest
  `codex-security-snapshot/v1:sha256:4a5c85e03cc3a5ce97269176ea791deb9fd81111ee89a72285d9b6d458d4abb5`.
  It closed all 20 authoritative review rows and accounted for all 37 changed
  or untracked paths. The TAC advisory connector was unavailable, so protected
  external advisory status was unknown; repository threat modeling,
  independent discovery, validation, and attack-path calibration still
  completed.
- Inspection finding ledger:

  | Finding | Disposition | Resolution |
  |---|---|---|
  | The shared public-egress classifier admitted IANA-reserved former-6bone `3ffe::/16`, allowing a guarded marketplace connection attempt to cross the promised public-only routing boundary under narrow DNS, route, and pinned-TLS prerequisites. | Confirmed, low severity | Added the missing `/16` exclusion and a direct regression corpus case; `cargo test -p omarchy-game-provider` passed 13 unit tests plus the portable integration suites. |
  | Persisted marketplace continuity identifies the owner-selected signing authority by origin, authority ID, and key ID rather than a duplicate digest of the external public-key file. | Not applicable in the current owner-operated threat model | Every snapshot still receives strict verification by the exact configured key and independent pinned TLS. Changing that owner-controlled trust anchor requires server-administration-equivalent authority. Revisit if key-file writers become a delegated lower-privilege role. |
  | The foreground database-local CLI does not enforce mode/ownership on a caller-selected catalog command file or prevent a same-inode writer race. | Not applicable in the current execution model | No privileged daemon, setuid boundary, or untrusted command spool consumes the file. The caller already supplies database authority and the documented workflow requires a private mode-0600 file. Revisit before introducing a split-authority service. |
- Documentation inspection corrected the catalog database-failure envelope to
  `internal_error` and clarified the designed retained-selection behavior:
  switching releases remains explicit, while a newer authenticated policy may
  make the same exact still-selected digest effective again.
- A fresh post-fix CodeGraph read confirmed `3ffe::/16` is now excluded at the
  one shared classifier used by both provider and marketplace production DNS
  admission; its three callers and socket-pinning sink were included in the
  blast-radius review.
- Phase 3.5 is PASS. One confirmed finding was resolved with a focused passing
  regression; no reportable inspection finding remains open. Broad validation
  and the canonical delivery receipt remain Phase 4 evidence.

## Phase 4 — Validate

- The first complete `./scripts/test-database.sh` run exposed one stale exact
  discovery fixture after the truthful
  `games.cartridge-catalog.v1` capability was added: 51 server tests passed and
  `server_discovery_api_tests::discovery_is_exact_stable_public_and_immutable`
  failed only on the prior exact capability list. The fixture was updated to
  the implemented contract, its focused PostgreSQL rerun passed, and no
  production behavior was weakened.
- `bin/gate.sh --fast`: PASS across all 15 portable stages after the discovery
  fixture repair.
- A clean `./scripts/test-database.sh` rerun passed the separately spawned real
  TLS marketplace lifecycle test, all 52 server/PostgreSQL tests, all five
  administrator database tests, and all three real operator CLI tests.
- The first retained `bin/gate.sh --diff` rerun passed stages 1–19 and 21–22,
  but stage 20 was RED because its clean-clone compilation exhausted the
  `/tmp` tmpfs (`Disk quota exceeded`). Cargo's package-scoped cleanup removed
  20.3 GiB of rebuildable server artifacts, and two older unrelated security
  scan caches were moved from `/tmp` to the recoverable hold directory
  `/home/cpeppers/omarchygs-cleanup-hold.msUEFW`, leaving the current Ticket
  032 security evidence intact. The focused
  `./scripts/test-provider-authority-pilot.sh` rerun then passed.
- The canonical `bin/gate.sh --diff` rerun passed all 22 stages. It repeated the
  marketplace contract/staging/TLS/PostgreSQL/CLI/API paths, 52 server database
  tests, 44-case QML fixture, live API/QML smoke, deterministic client package,
  provider security and clean-clone authority proofs, catalog-aware backup and
  restore, and private-alpha admission drill. The worktree-bound receipt
  `85a9fafb9db18322a9016501ea8b406009dc51e421c9efa73600e01f094cf0d7`
  exactly matched the gated state before Phase 5 edits.
- Phase 4 is PASS. Every Ticket 032 behavior has focused evidence and the
  complete regression stack is green; completion records will receive a final
  post-edit receipt before delivery.

## Phase 5 — Complete

- EARS audit:

  | Requirement | Evidence | Result |
  |---|---|---|
  | REQ-001 | Configuration and CLI fixtures reject absent/invalid origins, authority keys, DER roots, and secure-store roots before network or database mutation. | PASS |
  | REQ-002 | Marketplace contract tests cover domain-separated strict Ed25519 verification, canonical exact schema, bounds, sorting, duplicates, tuple/policy linkage, relative paths, and monotonic snapshot identity. | PASS |
  | REQ-003 | The separately spawned TLS lifecycle plus guarded-client tests prove pinned-root success and reject wrong roots, redirects, private destinations, oversized bodies, and tampered release/policy/conformance identities. | PASS |
  | REQ-004 | Secure-store tests prove reviewed staging writes no active pointer, preserves immutable exact bytes, and resolves only the requested digest; PostgreSQL tests prove all-or-nothing snapshot publication. | PASS |
  | REQ-005 | The real `cartridges` CLI test asserts bounded exact public inventory/provenance/admission fields and absence of credentials, private keys, TLS bytes, absolute paths, acquisition destinations, and rich content. | PASS |
  | REQ-006 | The migrated lifecycle test covers activate, deactivate, older-release rollback, exact replay, changed collision, expected-state conflict, concurrent writers, one selection, monotonic revision, and immutable audit. | PASS |
  | REQ-007 | Domain/database lifecycle matrices reject absent, incompatible, unimported, mismatched, suspended, revoked, and retired releases while retaining the deprecated warning. | PASS |
  | REQ-008 | Snapshot and policy downgrade, changed-equal conflict, denial-before-enforcement, omission, restart, no-fallback, retained-selection recovery, and explicit rollback cases all pass. | PASS |
  | REQ-009 | Authenticated Axum tests prove exact no-store sorted metadata, session ownership, lifecycle/omission filtering, and absence of paths, acquisition locations, key material, signed records, code, or render documents. | PASS |
  | REQ-010 | The operator recovery rehearsal compares marketplace singleton state, reviewed inventory, exact selection/revision, lifecycle facts, and catalog audit across isolated restore and rechecks immutability. | PASS |
  | REQ-011 | `bin/gate.sh --diff` passed all 22 stages with matching pre-completion worktree receipt `85a9fafb9db18322a9016501ea8b406009dc51e421c9efa73600e01f094cf0d7`. | PASS |
- OpenWiki run `049738d3-ec77-4703-8858-fa61508bde6c` returned
  `status: complete` under pipeline
  `48c86d57-dc5a-40bc-9779-d65b1e635b63`. It confirmed the signed
  marketplace, shared egress, immutable staging, independent catalog admission,
  metadata-only player API, and remaining client-acquisition boundary; it also
  removed duplicate generated claims and corrected stale profile-roadmap
  wording.
- AAR-032 is submitted with three failure IDs, three prevention rules, and one
  architecture decision. Every new ID is appended to the knowledge register.
- The owner-operated-server marketplace/catalog roadmap outcome is checked,
  Ticket 032 is closed, and this sole active spec/notes pair is archived under
  `docs/planning/pipeline/completed/`.
- A subsequent OpenWiki completion-only pass returned `status: complete`
  against the submitted AAR, checked roadmap, closed ticket, and archived
  pipeline state.
- Phase 5 is PASS. Delivery will rerun the canonical diff gate after the final
  completion receipt so both receipts bind the exact committed tree.

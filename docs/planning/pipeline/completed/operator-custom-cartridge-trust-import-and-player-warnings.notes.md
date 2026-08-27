---
title: Operator-custom cartridge trust, import, and player warnings — notes
pipeline_id: b4f37837-c7c8-4a29-9747-fb128045c289
---

# Operator-custom cartridge trust, import, and player warnings — running notes

Chronological evidence and decisions. If a test or command did not run, these
notes must not claim it passed.

## Phase 1 — Plan

- Recall: Ticket 037 was delivered at
  `d8e6db6dc2e7b77562d99613bc944c0fb0deac4c`; local `HEAD`, fetched
  `origin/main`, and the GitHub `main` ref matched, and the worktree was clean
  before Ticket 038 opened.
- Recall: the external two-clean-installation event and official hosted
  origins/custody/staffing require outside people or infrastructure and cannot
  be claimed locally. The earliest independent roadmap outcome is the explicit
  operator-custom cartridge path.
- Recall: ADR-0003 and the cartridge architecture permit custom content only
  as a visibly separate `operator-custom` provenance class. It remains signed,
  inert, schema/media/capability bounded, and trusted-renderer-only; it cannot
  carry QML, JavaScript, native/Web code, credentials, URLs, or backend
  authority.
- Recall: publisher integrity, marketplace review, server admission, and player
  trust are independent claims. A custom release has publisher/operator
  integrity plus server admission and explicitly no marketplace-review
  attestation.
- Recall: Tickets 032–035 already provide descriptor-relative release staging,
  monotonic lifecycle policy, PostgreSQL server admission/audit, authenticated
  catalog/distribution, independent client verification/cache/mount, immutable
  session pins, and historical acquisition. Ticket 038 must compose these
  seams rather than add a weaker store or renderer.
- Recalled rules:
  `PR-omarchy-gaming-system-separate-publisher-marketplace-server-attestations-001`,
  `PR-omarchy-gaming-system-authenticate-independent-claims-outside-claiming-authority-001`,
  `PR-omarchy-gaming-system-separate-historical-provenance-from-current-use-policy-001`,
  `PR-omarchy-gaming-system-snapshot-mutable-build-inputs-before-verification-001`,
  `PR-omarchy-gaming-system-persist-authenticated-denial-before-enforcement-001`,
  `PR-omarchy-gaming-system-serialize-monotonic-policy-transitions-001`, and
  `PR-omarchy-gaming-system-bind-permissions-to-opened-file-descriptors-001`.
- CodeGraph traced admin synchronization/catalog selection, secure staging,
  player catalog/distribution, acquisition verification, client trust/cache,
  session presentation, and QML exact-shape consumers. The high-blast-radius
  seams are `CatalogSelection`, `apply_catalog_command`,
  `PlayerCartridgeRelease`, `AcquisitionServerAdmission`,
  `verify_acquisition_bytes_with_policy_key`, `ClientMarketplaceTrust`, and
  `CartridgeDistributionRuntime`.
- `scripts/check-pipeline-tools.sh` passed with CodeGraph 1.5.0, OpenWiki 0.3.3,
  and Codex-only patch/build provenance. No active pipeline or open ticket
  existed before this one.
- Decisions: use a distinct server-scoped operator attestation/acquisition,
  require explicit companion-owned per-server enrollment, preserve the exact
  marketplace response for vetted rows, keep gameplay authority unchanged,
  and fail closed rather than silently rotate a custom key.
- Phase 1 is PASS. The scope is the complete safe operator-custom cartridge
  path; executable modules and Provider SDK work remain separate roadmap
  families.

## Phase 2 — Design

- Architecture and ownership:
  - `omarchygs-game-cartridge` owns a canonical operator-custom release
    attestation, domain-separated signing/verification, a bounded custom
    acquisition document, and a verified acquisition type. It receives no
    account, database, network, QML, or implicit-trust authority. Existing
    publisher release, lifecycle, secure-store, host-profile, and renderer
    checks remain authoritative and are reused rather than duplicated.
  - The admin process alone loads the operator catalog private key. It owns
    checked local input snapshotting, signing, secure staging, PostgreSQL
    import/lifecycle publication, idempotency, and secret-free receipts. The
    normal server process loads only the matching public key and operator name,
    composes distribution from retained public evidence, and exposes a
    candidate trust identity in discovery.
  - PostgreSQL owns one immutable server/custom-authority binding, exact custom
    release provenance, monotonic signed lifecycle, explicit mixed-source
    catalog selection, append-only import/policy/selection audit, and the
    source pinned to every new session presentation. The filesystem remains
    immutable byte evidence; neither store pointer nor marketplace state
    selects a custom game.
  - The local companion owns the player's per-server custom trust decision in
    a private descriptor-relative store. QML may present the current server's
    advertised candidate and explicitly request enrollment/removal through the
    credentialed loopback API, but neither QML profile metadata nor the remote
    server is itself the trust database.
  - QML owns plain-text disclosure and explicit interaction only. It never
    verifies signatures, reads paths, receives private keys, invokes a shell,
    installs native code, or grants gameplay/provider authority.
- Trust and contract model:
  - `OperatorCustomReleasePayload` uses format
    `omarchygs.operator-custom-release/v1` and binds attestation version 1,
    canonical stable server UUID, operator authority/name, exact operator key
    fingerprint, complete publisher public key, game/publisher/rules/cartridge
    identity, archive and signed-identity digests, and one mandatory bounded
    unvetted-content warning. It contains no URL, path, reviewer, marketplace,
    support, executable, credential, or arbitrary metadata field.
  - `SignedOperatorCustomRelease` is canonical JSON with Ed25519, exact key ID,
    base64url canonical payload, and a signature under a new
    `omarchygs-operator-custom-release-v1` domain. Verification compares the
    complete payload with the independently verified publisher release and the
    expected stable server/key identity.
  - `OperatorCustomAcquisition` uses format
    `omarchygs.operator-custom-acquisition/v1` and carries exact server
    admission, operator public key, signed custom attestation, current signed
    lifecycle policy, and base64url archive/conformance/release bytes. It has
    the existing 16 MiB envelope and component ceilings and canonical
    serialization. It deliberately has no marketplace key, snapshot, root,
    reviewer, or `marketplace_vetted` field.
  - `VerifiedRemoteAcquisition` becomes an exact enum over existing
    `VerifiedAcquisition` and new `VerifiedOperatorCustomAcquisition`, exposing
    only shared verified release/policy methods plus typed provenance. Cache
    and server distribution code must match the variant rather than infer
    source from names or missing data.
  - A marketplace row continues to serialize the existing required
    `marketplace` object and no custom object. A custom row serializes one
    required `operator_custom` object and no marketplace object. Parsers
    require exactly one. The custom object binds provenance class, server UUID,
    operator/authority/key IDs, key SHA-256, lifecycle version/status, and the
    same mandatory warning returned at the release level.
- Admin configuration and flow:
  1. Custom operations require `GAMING_SYSTEM_CUSTOM_CARTRIDGE_PRIVATE_KEY_FILE`
     as an absolute non-symlink owner-regular mode-0600 file plus bounded
     `GAMING_SYSTEM_CUSTOM_CARTRIDGE_AUTHORITY_NAME`; the derived public key
     must match the optional separately configured public file when present.
     The existing absolute secure-store root is reused. No private path or key
     appears in argv, JSON receipts, PostgreSQL, discovery, logs, or API data.
  2. `omarchygs-admin custom-cartridge-import <plan>` reads one bounded exact
     plan containing operation UUID, absolute release directory and publisher
     public-key paths, actor, reason, and warning acknowledgement. It opens and
     snapshots the publisher key plus the three fixed release components once,
     then verifies only those owned bytes with `supported_sdk_identity` and
     `rich_2d_host_profile`.
  3. The command derives the operator public key/fingerprint, reads the stable
     server UUID, produces the deterministic custom attestation and active
     policy version 1, stages the verified release through
     `SecureCartridgeStore::stage_reviewed_release`, then publishes authority,
     release, policy, and one immutable import audit in a serialized database
     transaction. Failure may leave only harmless unreferenced immutable store
     bytes; it publishes no partial database row.
  4. Exact operation replay returns the original public receipt; a collision
     fails. The authority singleton is insert-once and thereafter requires
     exact server/key/name equality. A different key cannot silently coexist
     with retained custom provenance in this slice.
  5. `omarchygs-admin custom-cartridge-policy-apply <plan>` requires exact
     digest, expected policy version/status, desired higher version/status,
     actor/reason, and operation UUID. Under the per-game advisory lock it
     re-verifies the retained immutable release, signs the new policy, caches
     it in the secure store before enforcing denial, atomically advances the
     database row, and appends immutable audit. It never rewrites the original
     operator attestation or release identity.
- Database migration `0024`:
  - `operator_custom_authority` is a one-row stable-server binding containing
    server UUID, operator name, authority/key IDs, canonical public key,
    fingerprint, and creation time. Update/delete/truncate reject.
  - `operator_custom_releases` stores immutable publisher and operator
    provenance, exact release identity/display, signed custom attestation,
    custom warning, compatible/imported facts, and mutable monotonic signed
    lifecycle. Digests and identities cannot mutate or delete; policy version
    cannot regress or change bytes at an equal version.
  - `operator_custom_audit_events` records globally unique operations for
    import and lifecycle transitions with exact before/after policy facts,
    actor/reason, release identity, and time. Update/delete/truncate reject.
  - `server_cartridge_catalogs` gains nullable `active_custom_release_id`; a
    check permits at most one of the existing marketplace and new custom
    references. Its trigger verifies selected game identity across the proper
    table. `cartridge_catalog_audit_events` gains nullable previous/resulting
    provenance classes, backfilled as `marketplace_vetted` where a digest is
    present, and its transition checks require source and digest together.
  - Game-session presentation rows gain optional provenance class, custom key
    fingerprint, operator name, and warning. Existing presentation rows
    backfill as marketplace-vetted; custom fields are all-or-none and valid
    only for `operator_custom`. No gameplay/provider or participant foreign key
    changes.
- Mixed catalog/admission flow:
  - `CatalogSelection::Release` preserves the current marketplace admin JSON;
    a new `CustomRelease` variant names an exact digest. `Inactive` remains
    unchanged. Expected/desired comparisons include source plus digest, so the
    same immutable archive presented through two authorities is never
    ambiguous.
  - Selection resolves the requested source in its own table, verifies its
    current signed policy and secure-store bytes under the corresponding
    public key, and atomically writes exactly one foreign key plus revision and
    source-aware audit. It never automatically prefers custom or marketplace
    content and never selects a fallback after lifecycle denial.
  - Inventory is a bounded sorted union. Current marketplace synchronization
    remains authoritative only for marketplace rows and cannot alter custom
    rows. Custom import/lifecycle cannot alter marketplace sync state, root,
    keyring, snapshots, reviews, or native packages.
- Serving and client flow:
  1. Normal server startup treats custom configuration as absent or complete.
     Complete configuration loads an absolute checked public key and bounded
     name, verifies the immutable database authority if it exists, and extends
     `CartridgeDistributionRuntime` with public custom authority. The private
     key variable is ignored by and inaccessible to the server process.
  2. Discovery preserves its base exact fields and sorted capabilities, adding
     `games.operator-custom-cartridges.v1` and one optional exact public
     authority object only while public configuration is valid. Server profile
     persistence treats this as advertised public metadata, not trusted state.
  3. The custom acquisition route remains bearer-authenticated, participant/
     selection scoped, no-store, and bounded. Current and historical lookups
     choose a typed release source and construct only its corresponding
     acquisition envelope. Absence and denial retain the established
     not-found-versus-denied privacy behavior.
  4. The companion adds a descriptor-relative `operator-trust` directory,
     cross-process lock, and at most 16 canonical mode-0600 records keyed by
     server UUID. Each record binds canonical HTTPS-or-loopback origin, stable
     UUID, full catalog public key/fingerprint, and enrollment time. Path
     replacement, public modes, extra files, duplicates, malformed JSON, and
     cross-origin/server reuse fail closed.
  5. Credentialed bounded local endpoints report candidate/pinned status,
     enroll an exact candidate, and remove an exact expected fingerprint.
     Enrollment is idempotent for identical bytes and conflicts on any key,
     origin, or server replacement; removal does not delete cached/mounted
     evidence and those mounts immediately become untrusted/unrenderable.
  6. Remote acquisition parses the selected catalog provenance before fetching
     bytes. Marketplace uses the existing independent trust snapshot. Custom
     requires the exact local server binding, verifies the custom envelope,
     publisher release, current policy, and expected admission, then re-fetches
     the catalog/session to detect changes before mounting.
  7. New mount format `omarchygs.client-cartridge-mount/v2` carries a strict
     tagged provenance union. Legacy v1 marketplace records remain accepted
     only after full old validation and are upgraded in memory to v2; new
     writes are v2. Composite cache/render trust authorizes marketplace mounts
     through the marketplace trust snapshot and custom mounts through the
     exact per-server operator store.
  8. Session presentation carries custom provenance only for custom pins.
     Historical acquisition retains the immutable attestation but requires the
     latest signed custom policy for use. Cartridge actions continue to
     translate through the trusted screen contract and dispatch only to the
     existing session's compiled or registered-provider authority.
- QML interaction:
  - `ServerProfiles` and onboarding accept an optional exact advertised custom
    authority while preserving profiles without it. A changed advertisement
    does not modify the companion pin; status becomes mismatched.
  - A dedicated `OperatorTrustController` queries local status for the active
    server and exposes explicit `TRUST CUSTOM GAMES` and `REMOVE CUSTOM TRUST`
    actions. Enrollment requires the player to activate a keyboard-focusable
    confirmation displaying server/operator identity, the full fingerprint,
    and a fixed warning that the content is not marketplace reviewed or
    supported.
  - Catalog cards, mount inventory, challenges, and gameplay keep the custom
    label/warning visible. Marketplace trust gates vetted releases; exact
    operator trust gates custom releases. Untrusted custom metadata may be
    listed, but install, historical acquisition, render, challenge, and launch
    actions remain disabled. Removal remains available.
  - All remote strings remain `Text.PlainText`; URLs/markup are not accepted;
    no QML source gains process, shell, package-manager, filesystem, WebEngine,
    or network-to-provider authority.
- File manifest:

  | Path | Purpose |
  |---|---|
  | `crates/game-cartridge/src/operator_custom.rs`, `src/lib.rs`, `src/error.rs` | Canonical custom attestation/acquisition types, signer/verifier, fingerprints, limits, exports, and stable errors. |
  | `crates/game-cartridge/src/secure_store.rs`, `tests/operator_custom.rs`, `tests/sdk_release.rs` | Retained-release re-verification, custom contract hostile corpus, deterministic signing, and lifecycle/store evidence. |
  | `migrations/0024_operator_custom_cartridges.sql` | Stable authority, custom releases/audit, mixed selection, source-aware selection audit, and session provenance. |
  | `crates/server/src/operator_custom.rs`, `src/config.rs`, `src/bin/omarchygs-admin.rs`, `src/main.rs`, `src/lib.rs` | Admin config/input snapshot/sign/import/policy orchestration, public runtime config, CLI dispatch, and module registration. |
  | `crates/server/src/cartridge_catalog.rs`, `marketplace_sync.rs` | Bounded mixed inventory, source-aware activation/audit, custom lifecycle publication, and marketplace isolation. |
  | `crates/server/src/cartridge_distribution.rs`, `session_cartridges.rs`, `games.rs`, `challenges.rs`, `app.rs`, `server_discovery.rs` | Typed current/historical distribution, source-pinned sessions/actions, API routing, and optional public authority discovery. |
  | `crates/server/src/operator_custom_tests.rs`, `marketplace_sync_tests.rs`, `cartridge_catalog_api_tests.rs`, `game_api_tests.rs`, `server_discovery_api_tests.rs`, `tests/operator_cli.rs` | Config/sign/import/lifecycle, mixed catalog, acquisition, session authority, discovery, CLI, and PostgreSQL hostile evidence. |
  | `crates/client-cartridge-runtime/src/operator_trust.rs`, `lib.rs`, `main.rs`, `service.rs` | Private per-server trust store, local status/enroll/remove API, startup composition, and exact receipts/errors. |
  | `crates/client-cartridge-runtime/src/remote.rs`, `cache.rs`, `render.rs` | Provenance-aware acquisition verification, v1-to-v2 mounts, composite trust, and custom current/historical rendering. |
  | `client/qml/OperatorTrustController.qml`, `ServerProfiles.qml`, `OnboardingController.qml`, `CartridgeController.qml`, `GameController.qml`, `Main.qml` | Advertised metadata, explicit local trust actions, source-aware gates, and persistent warning state. |
  | `client/qml/screens/GamesScreen.qml`, `ChallengesScreen.qml`, `GameplayScreen.qml`, QML fixture/profile tests and fixture server | Keyboard/focus/warning controls plus exact hostile/current/historical user-flow proof. |
  | `.env.example`, `README.md`, `docs/api.md`, architecture/operator/client docs, roadmap | Configuration, contracts, trust ceremony, lifecycle/recovery, warnings, and honest remaining limitations. |
  | `scripts/test-operator-recovery.sh`, `scripts/test-client-package.sh`, `bin/gate.sh` if a focused stage is needed | Restored server evidence, packaged-client payload/provenance, and canonical enforcement. |
  | Active Ticket 038 artifacts, AAR, knowledge register, OpenWiki | Workflow evidence, durable lessons, and generated documentation reconciliation. |
- Regression matrix:

  | Requirement | Evidence |
  |---|---|
  | REQ-001 | Absent/partial config, unchanged discovery/marketplace JSON, rejected CLI actions, and zero mutation. |
  | REQ-002 | Absolute path, symlink, inode/type, ownership, 0600 mode, size, malformed/private/public mismatch, server identity, and output/privacy corpus. |
  | REQ-003 | Deterministic signing plus one-read input mutation, publisher/release/SDK/conformance/archive/schema/media/capability tamper and store containment cases. |
  | REQ-004 | PostgreSQL import success/replay/collision/concurrency/rollback, immutable trigger, audit, no partial publish, and no marketplace fields. |
  | REQ-005 | Five-state monotonic lifecycle, equal-version conflict, competing writers, denial-before-enforcement, restart, audit, and current/history use matrix. |
  | REQ-006 | Exact mixed admin/player inventory, marketplace shape compatibility, required custom warning, sort/count bounds, and sensitive-field absence. |
  | REQ-007 | Marketplace/custom/inactive transitions, same-digest ambiguity, expected-state conflict, replay collision, revision, and source-aware immutable audit. |
  | REQ-008 | Discovery absent/present exact shapes, sorted capability, public fingerprint, server/key mismatch, restart, and no private material. |
  | REQ-009 | Custom envelope canonicality/limits and wrong key/server/publisher/policy/admission/signature/digest/component/tamper plus route auth/privacy. |
  | REQ-010 | Companion enroll/same replay/replacement conflict/remove/restart/race plus origin/UUID/path/symlink/mode/extra/malformed/cross-server failures. |
  | REQ-011 | Spawned TLS server current/session acquisition, before/after changes, offline/truncation/oversize, cache/mount provenance, and trust removal. |
  | REQ-012 | Exact QML schemas, keyboard confirmation/removal, focus/compact layout, persistent full warning/fingerprint, source-specific enablement, hostile text, and no process spawn. |
  | REQ-013 | Compiled/provider new and historical custom session pins/actions with no backend-registration or authority change. |
  | REQ-014 | Server backup/restore comparisons plus companion trust/cache/mount restart and rollback/tamper evidence. |
  | REQ-015 | Focused cartridge/server/client/QML/package/recovery suites, security diff scan, fresh CodeGraph, OpenWiki, and full diff gate. |
- Security, privacy, concurrency, and recovery risks:
  - Trust-on-first-use phishing is explicit: server discovery supplies only a
    candidate. The player sees server/operator/fingerprint and must enroll via
    a local authenticated action. The exact pin persists and replacement
    conflicts; this slice does not claim marketplace or root-backed identity.
  - A compromised custom signing key can authorize unvetted inert content for
    players who pinned it. It still cannot bypass publisher/cartridge
    verification, trusted rendering, capability bounds, server admission, or
    backend authority. Player trust removal and operator signed revocation are
    the bounded responses; automatic key recovery is not claimed.
  - Local input and path races are controlled by one-time opened byte snapshots,
    descriptor-relative stores, restrictive creation, descriptor-bound modes,
    canonical file inventories, and no path reuse in receipts or persistence.
  - Database races serialize authority/import/policy/catalog changes with
    idempotency checks and advisory/row locks. Immutable store work occurs
    outside or before short database transactions; denials persist before use
    decisions and no implicit cross-source fallback occurs.
  - Custom strings are bounded plain text. APIs, receipts, QML profiles, mount
    records, and audits contain public identities only; private keys, paths,
    device/session credentials, TLS material, raw database details, provider
    endpoints, and local cache destinations are absent.
  - Existing old clients will reject custom catalog rows rather than mislabel
    or run them. Marketplace rows retain their established shape. The packaged
    QML and companion move together for custom support and continue accepting
    fully validated legacy marketplace mount records.
- Material alternatives rejected:
  - Treating the operator as a tiny marketplace and emitting fake reviewer/
    snapshot fields was rejected because it makes absence of review ambiguous.
  - Trusting the custom key automatically because a player selected/logged into
    the server was rejected because authentication/TLS selection and permission
    to distribute unvetted presentation are distinct decisions.
  - Storing the trust pin only in QML settings or inside a self-authenticating
    mount was rejected because remote/profile metadata must not become the
    security decision; the companion owns a private exact binding.
  - Reusing one ambiguous digest-only catalog selector was rejected because the
    same bytes can honestly have both vetted and custom provenance.
  - Letting custom import register provider endpoints, compiled rules, modules,
    QML, JavaScript, or native code was rejected as an authority expansion
    outside the cartridge roadmap item.
  - Transparent key rotation was rejected for this slice because no independent
    root exists to authorize replacement. Exact mismatch plus explicit future
    reenrollment/recovery is the honest fail-closed behavior.
- Phase 2 is PASS. The design maps every requirement to an additive authority,
  schema, server/client/QML flow, and hostile evidence set while preserving the
  marketplace and gameplay boundaries. Matching CodeGraph design receipt:
  pipeline `b4f37837-c7c8-4a29-9747-fb128045c289`, state hash
  `b9896c389d6bf9d268d4d125bb3edaf7762f916bfc98272c4667b71c19ff4991`.

## Phase 3 — Implement

- Built:
  - Added canonical domain-separated operator-custom attestation and
    acquisition contracts. They reuse publisher release, production SDK/host,
    lifecycle, archive, conformance, and trusted renderer verification while
    containing no marketplace/reviewer/root claim.
  - Added migration 0024 with immutable server-bound custom authority,
    publisher/operator release provenance, monotonic policy, append-only audit,
    mutually exclusive mixed-source catalog selection, source-aware catalog
    audit, and immutable session provenance.
  - Added admin-only key loading, exact public/private match and private-file
    owner/mode/symlink enforcement, verified import and policy commands,
    source-aware catalog apply/inventory, normal-server public-only startup,
    optional discovery, current/historical distribution, and session/action
    resolution without changing gameplay authority.
  - Added companion discovery, explicit canonical origin/server UUID/key pin,
    private restart-safe trust/profile storage, source-explicit acquisition and
    rendering, distinct custom mount provenance, key-change rejection, and
    removal only after custom mounts are removed.
  - Added exact QML discovery/profile/catalog/session validation, source-aware
    install/render gates, keyboard trust/removal actions, and persistent
    operator/fingerprint/unreviewed warnings in catalog and gameplay.
  - Added contract, PostgreSQL, CLI, remote cryptographic acquisition,
    companion restart/cache/render, QML keyboard, and non-empty backup/restore
    evidence. Updated API, architecture, product, README, environment, and
    owner/recovery guidance.
- Deviations:
  - Kept legacy marketplace mount JSON byte-shape compatibility and introduced
    a separate `omarchygs.client-operator-custom-mount/v1` profile instead of
    rewriting all mounts to the proposed tagged v2 union. The companion/API
    still expose an exact source union and never infer source from a digest.
  - Integrated the small operator-trust state machine into the existing
    `CartridgeController` rather than adding a second QML controller. The
    security decision remains in the native companion, and QML retains only
    public candidate/status data.
  - Trust removal is denied while custom mounts exist, rather than leaving
    present-but-untrusted mounts. This gives the player an explicit ordered
    cleanup ceremony and prevents ambiguous retained custom state; remote game
    state and shared immutable content remain untouched.
  - The local companion provides POST `/v1/operator-custom-trust/remove` in
    addition to DELETE because Qt's XMLHttpRequest does not reliably complete
    a DELETE carrying a JSON confirmation body. Both routes execute the same
    exact credentialed handler.
- Phase 3 implementation is complete; focused game-cartridge, server custom,
  client runtime, QML, CLI configuration, and recovery tests passed before
  inspection. The first complete PostgreSQL run found one stale exact CLI
  receipt expectation after provenance became explicit; that test contract was
  updated and its complete CLI suite passed. Inspection then returned one
  low-severity custom-policy/action linearization finding; implementation was
  reopened, the policy writer was added to the established global lifecycle
  lock, and the focused writer-first PostgreSQL regression plus the complete
  PostgreSQL suite passed.

## Phase 3.5 — Inspect ledger

| # | Lens | Finding | Severity | Disposition |
|---|---|---|---|---|
| 1 | Codex Security diff scan | `apply_custom_policy` used only the per-game advisory lock while fresh cartridge-action admission used the global lifecycle lock in shared mode. Under `READ COMMITTED`, a writer-first suspension/revocation could therefore race an old statement snapshot into one fresh durable action admission. | Low | FIXED. The policy transaction now takes the global exclusive lock before the per-game lock and holds both through policy/audit commit. A deterministic PostgreSQL regression queues the writer first, proves the later action waits, then proves it reads the committed denial. Scan `495c00fd-a80d-43f0-96b9-1a0ec48f33ac`, finding `csf_257a8332be1abedb883711cc`. |
| 2 | Independent post-fix review | Check the new lock order for residual race/deadlock and exact replay compatibility. | Informational | PASS. The reviewer found no residual security issue or reverse lock-order cycle. A follow-up confirmed the strengthened test closes its initial reader-first-only assurance gap. Exact pre-transition action replay remains before current lifecycle evaluation; changed replay still conflicts. |
| 3 | Client security review | Test whether advertised custom keys, origins, QML warnings, remote acquisition, private trust storage, or custom render provenance grant implicit/executable/cross-origin authority. | Informational | PASS. No plausible finding remained: trust is an explicit exact origin/server/key pin, acquisition is fixed-origin and independently verified, mounts are source-explicit, removal/render recheck native trust, and QML retains no signing, filesystem, process, WebEngine, provider, or credential authority. |
| 4 | Server/contract authority review | Test private-key confinement, custom/marketplace provenance separation, lifecycle persistence, authenticated routes, input snapshotting, and native-game/provider authority. | Informational | PASS except finding 1. Private-key loading is admin-only and descriptor checked; normal serving uses public evidence; custom envelopes cannot claim marketplace review or register rules/providers; denials and audits are durable and source-specific. |
| 5 | Raw command compatibility boundary | Determine whether `/commands` is a second cartridge admission route that must inherit custom lifecycle locking. | Informational | PASS. The documented raw route remains the authenticated platform/provider rules API for compatible clients. `/cartridge-actions` is explicitly the only route for intent emitted by a trusted cartridge plan, so the finding and fix are correctly scoped to cartridge action admission. |
| 6 | CodeGraph structural inspection | Re-trace `apply_custom_policy`, `admit_session_action`, `SNAPSHOT_ADVISORY_LOCK`, admin callers, catalog writers, and test coverage after the repair. | Informational | PASS. Custom policy is the only existing-release custom lifecycle writer, global-before-per-game matches marketplace writer ordering, imports/catalog selection retain narrower per-game serialization, and action replay remains ahead of current-policy resolution. |

- Codex Security completed the full working-tree diff scan with 27 scoped
  source files plus direct QML fixture inspection. It reported exactly the one
  low finding above; the fix report and independent reviewer readback record it
  as fixed. The optional TAC connector was unavailable, so no external threat
  advisory context was imported and no claim depends on it.
- Phase 3.5 is PASS subject to the matching final CodeGraph inspection receipt
  after the last notes/spec reconciliation. The final inspection receipt now
  matches pipeline `b4f37837-c7c8-4a29-9747-fb128045c289` and gated state
  `2c951597d09ecca4ba7cff6b0120326ff8c62d9d486ba2e260712c710adddefe`.

## Phase 4 — Validate

- Tests run:
  - Focused game-cartridge operator-custom contracts, server custom config/
    import/lifecycle/catalog/distribution/session tests, client trust/cache/
    remote/render/service tests, QML fixtures, operator CLI, and recovery
    drill all passed after the inspection repair.
  - The clean database gate passed 3 special tests, 56 server tests, 5 admin
    CLI tests, and 4 operator CLI tests. The QML suite passed 53 tests, the
    packaged client smoke passed 15 tests, and the package reproducibility
    check produced SHA-256
    `9766372e67b410ae55fdc758f855c8147c57fd992a820ef74afd39355f29d4c9`.
- Gate run:
  - An earlier untouched `bin/gate.sh --diff` completed all 22 stages and
    reported `GATE GREEN [diff]` after the code, security fix, and
    client-package timeout separation.
  - The canonical rerun against the final OpenWiki-reconciled gated state also
    completed all 22 stages and reported `GATE GREEN [diff]`. Its matching
    worktree receipt is
    `2c951597d09ecca4ba7cff6b0120326ff8c62d9d486ba2e260712c710adddefe`,
    identical to the final CodeGraph inspection state hash.
- Skips or pre-existing failures:
  - No required stage was skipped. One earlier full-gate attempt found two
    workspace Clippy diagnostics and an outer package-process timeout; both
    were fixed and the complete stages passed.
  - A later coverage run was invalidated only because a manual fixture-status
    probe added an unexpected request to the fixture's exact request ledger.
    No application code changed for that event; the untouched rerun passed.
  - The first post-OpenWiki canonical rerun had one live QML registration
    scenario fail closed at the access screen after account and session
    requests each consumed more than four seconds and the following personas
    request exceeded the client's five-second network timeout. The untouched
    focused live smoke immediately passed all 53 fixture tests and all four
    live scenarios, so the product timeout was retained and the untouched
    canonical rerun above established the final green receipt.
  - The OpenWiki lifecycle completed. Its four warnings were pre-existing
    unresolved evidence-debt sidecars for `quickstart`, `game-cartridges`,
    `runtime-foundation`, and `product-boundaries`; the lifecycle preserved
    those sidecars rather than claiming the evidence gaps were resolved.
- Phase 4 is PASS. The complete diff gate and final CodeGraph inspection
  receipts match the final gated worktree.

## Phase 5 — Complete

- Acceptance-criteria audit:

  | Requirement | Disposition and evidence |
  |---|---|
  | REQ-001 | PASS — absent-versus-complete server/admin configuration and unchanged marketplace/base discovery contracts are exact-tested. |
  | REQ-002 | PASS — descriptor-bound absolute private-key loading enforces regular owner mode 0600, public/private identity match, stable server identity, and admin-only secret access. |
  | REQ-003 | PASS — bounded release inputs are snapshotted once, production-verified, operator-signed, and staged from those owned bytes; tamper/substitution tests pass. |
  | REQ-004 | PASS — migration 0024 atomically persists immutable publisher/operator provenance, idempotent results, and append-only audit without review claims or partial publication. |
  | REQ-005 | PASS — monotonic signed lifecycle writers serialize global-before-game, persist denial through commit, reject rollback, and pass writer-first action-denial evidence. |
  | REQ-006 | PASS — CLI/API inventory is a bounded sorted exact source union; marketplace rows preserve their prior shape and custom rows expose only public identity/lifecycle/warning facts. |
  | REQ-007 | PASS — mixed-source selection is mutually exclusive, expected-state/idempotent, revisioned, audited, concurrency-safe, and never performs implicit source fallback. |
  | REQ-008 | PASS — discovery advertises only a bounded public candidate; client trust remains an independent exact origin/server/key pin and mismatches fail closed. |
  | REQ-009 | PASS — authenticated bounded current/historical custom acquisition binds server admission, operator attestation, publisher release, signed current policy, and immutable bytes with no marketplace evidence. |
  | REQ-010 | PASS — the companion's private descriptor-relative trust store is restart/race/symlink/mode/substitution tested and requires explicit enrollment/removal confirmation. |
  | REQ-011 | PASS — remote custom acquisition performs pre/post catalog rechecks, verifies every authority/admission/policy claim against the local pin, and mounts source-explicit content-addressed bytes. |
  | REQ-012 | PASS — QML exact-schema and keyboard tests prove persistent plain-text custom warnings, server/operator/fingerprint identity, explicit trust controls, and disabled use before current trust. |
  | REQ-013 | PASS — current/historical session presentation retains source/custom provenance while action dispatch remains limited to existing compiled or registered-provider gameplay authority. |
  | REQ-014 | PASS — database recovery and companion restart/cache/mount reconciliation preserve release, lifecycle, admission, audit, trust, and historical evidence without secrets. |
  | REQ-015 | PASS — focused/security/CodeGraph evidence passed, OpenWiki completed with preserved evidence-debt warnings, the final 22-stage diff gate is green with a matching receipt, and executable modules/review claims remain absent. |

- Docs:
  - Updated public configuration/API, Game Cartridge architecture, owner and
    recovery guidance, product charter, README, environment example, QML
    warnings, and OpenWiki pages for the implemented operator-custom boundary.
  - OpenWiki update run `38a136e8-e27c-44d0-b9ea-cb23955b2264`
    completed. Four unresolved-claim sidecars were deliberately preserved and
    reported as evidence debt rather than silently rewritten.
  - The final no-drift receipt refresh run
    `e0c7c0d8-eedf-405e-b99c-493d1f94e45d` inspected the same affected claim
    sets, required no further factual edits, and completed without warnings
    against the final OpenWiki state.
- AAR:
  - AAR-038 records the security race, separate startup/post-load timeout
    lesson, exact-request observer pollution, the new trust-boundary decision,
    and three prevention rules. Every new ID is registered in the knowledge
    index; existing exact-contract and workspace-Clippy rules were reused.
- Archive:
  - The matching final gate and OpenWiki receipts are present, Ticket 038 is
    closed, and this ticket/spec/notes set is archived in the closed/completed
    planning directories.
- Phase 5 is PASS.

## Defect and lesson ledger

| # | What happened | Root cause | Fix | Prevention |
|---|---|---|---|---|
| 1 | The first PostgreSQL run rejected a CLI receipt fixture after catalog selection gained explicit provenance. | The test still expected the legacy digest-only selection receipt. | Updated the exact CLI expectation to include the custom/marketplace source identity. | Treat additive provenance as an exact contract change and run the complete CLI suite before inspection. |
| 2 | Security inspection found custom policy publication could race a fresh cartridge action. | The new policy writer reused the per-game custom lock but did not join the established global lifecycle linearization lock used by action admission. | Acquire global exclusive before per-game, retain both through commit, and add deterministic writer-first denial/replay evidence. | Every lifecycle writer must share the same lock domain and lock order as durable use admission; test both reader-first replay and writer-first denial. |
| 3 | The first complete gate stopped on `map(...).flatten()` and a helper with too many arguments even though focused tests passed. | Focused compile/test commands did not run the workspace-wide warning-denied Clippy contract. | Replaced the flatten pattern, scoped the justified helper exception, and reran the complete gate. | Reuse `PR-omarchy-gaming-system-run-warning-denied-workspace-clippy-before-canonical-gate-001`; focused behavioral evidence does not replace workspace lint evidence. |
| 4 | Packaged-client smoke exceeded its 20-second outer process timeout during a cold QML preload even though the in-application post-load watchdog remained healthy. | One outer timeout budget conflated process/cold-cache startup with the narrower loaded-application responsiveness assertion. | Raised only the outer process allowance to 120 seconds and retained the 15-second post-load watchdog in `Main.qml`. | Budget process startup and post-load liveness independently so a cold toolchain does not weaken the product watchdog. |
| 5 | A manual fixture status request polluted the exact discovery request ledger and made a coverage gate red. | The observer used the same externally asserted HTTP surface as the system under test. | Discarded the contaminated run and reran the gate without probing the fixture. | Observe exact-request fixtures through process/log state outside the protocol surface being asserted. |
| 6 | The first post-OpenWiki live QML registration scenario failed closed before personas loaded, while the immediate focused live rerun passed. | Host/PostgreSQL I/O consumed the existing five-second client request budget across unusually slow account/session/persona requests; this was a transient environment condition, not a product sequencing defect. | Preserved the fail-closed timeout, proved the untouched focused smoke, and reran the untouched canonical gate to a matching green receipt. | Reuse `PR-omarchy-gaming-system-budget-readiness-for-measured-cold-path-001`: diagnose measured cold-path/environment latency before weakening a security-relevant client timeout. |

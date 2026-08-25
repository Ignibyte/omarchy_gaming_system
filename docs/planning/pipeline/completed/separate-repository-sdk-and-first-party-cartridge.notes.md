---
title: Separate-repository OmarchyGS SDK and first-party cartridge — notes
pipeline_id: 10b7eba4-c415-4551-87ff-75084d0f015c
---

# Separate-repository OmarchyGS SDK and first-party cartridge — running notes

Chronological evidence and decisions. If a test or command did not run, these
notes must not claim it passed.

## Phase 1 — Plan

- Tickets 015 and 016 are Phase 5 complete. Production now has a canonical
  signed inert `.ogsc`, exact verification/conformance, a same-user local
  store, a Rust render-plan compiler, fixed trusted QML components, an isolated
  previewer, and measured Core/Rich-2D profiles.
- Bulletin `BUL-001-initial-push-pending` remains a warning. The ongoing work is
  local; no commit, push, pull request, SDK publication, or separate remote
  repository creation is authorized by this pipeline.
- Recalled the package and renderer AARs plus ADR-0002: parse authenticated
  bytes, bound work during construction, bind capabilities/actions exactly,
  render plain text through platform code, and preserve the current compiled
  server-authority model.
- Recalled the deferred Ticket 015 boundary: path-based store validation is
  sufficient only for same-user local use. Any privileged or cross-principal
  importer must anchor every descendant operation to already-open directory
  descriptors and fail closed on authoritative revocation uncertainty.
- Smallest useful proof: export one deterministic SDK snapshot, copy it and one
  first-party cartridge source fixture into a fresh temporary Git repository,
  produce the same signed release twice, verify/import it through public
  production APIs only, and prove lifecycle plus ancestor-swap containment.
- The user-facing graphics answer remains Ticket 016's measured contract. This
  ticket makes that vocabulary portable; it does not add higher graphics tiers.

## Phase 2 — Design

- Public SDK artifact:
  1. `sdk export` writes one exact v1 directory into an existing empty output
     directory. The compiled tool owns every byte: canonical lock, README, and
     JSON Schemas for the cartridge manifest, presentation vocabulary,
     restricted view schemas, release attestation, and catalog policy.
  2. `sdk verify` rejects symlinks, unknown/missing files, non-canonical JSON,
     byte/digest drift, unsupported lifecycle state, or a tool/version mismatch.
     The lock excludes itself from its sorted file inventory and its canonical
     SHA-256 identity is the SDK identity carried by releases.
  3. V1's compatibility policy is structured: exact SDK/presentation versions
     are current; deprecation permits building with a warning; retirement
     rejects new releases; active sessions remain governed by the signed
     cartridge catalog policy. No Rust type or workspace path is public SDK.
- First-party clean-room flow:
  1. A repository fixture contains only ordinary cartridge source, a bounded
     view fixture, a build script, and documentation. The focused harness copies
     it into a fresh Git repository with a deterministic commit, then clones it
     twice.
  2. Each clone receives only an exported SDK directory, copied production CLI
     binaries, and an explicit publisher key path outside the source tree. The
     platform source root, database, provider network, and platform credentials
     are not inputs.
  3. `release` verifies the SDK, packages and verifies the cartridge, emits the
     canonical conformance report, and signs a domain-separated release payload
     containing the Git revision, builder name/version/binary digest, SDK lock
     digest, publisher/key IDs, rules/cartridge versions, archive digest, signed
     content identity, and conformance digest. The two clones must produce
     byte-identical archive, report, and signed attestation.
  4. `verify-release` reopens only the three bounded release files, re-runs the
     production cartridge verifier, reconstructs the exact conformance bytes,
     verifies the publisher signature, and binds every provenance field. It
     never trusts the attestation in place of package verification.
- Catalog lifecycle:
  - A distinct catalog-authority Ed25519 key signs a canonical policy bound to
    authority, monotonically nonzero policy version, game, publisher, exact
    cartridge digest, status, and bounded reason. Publisher and catalog
    signatures are intentionally separate authorities.
  - The fixed decision table is:

    | Status | New launch | Active session |
    |---|---|---|
    | active | allow | continue |
    | deprecated | allow with warning | continue |
    | suspended | deny | suspend |
    | revoked | deny | terminate |
    | retired | deny | continue pinned |

  - Import and every later resolve operation require a currently supplied,
    valid signed catalog policy matching the exact release. Missing, malformed,
    mismatched, stale-by-caller, or unverifiable policy fails closed; no other
    installed digest may be silently selected.
- Secure store:
  - `SecureCartridgeStore::open_existing` opens one explicitly provisioned Unix
    root with `O_DIRECTORY|O_NOFOLLOW|O_CLOEXEC`, then creates/opens every fixed
    child relative to retained descriptors. The API never joins an untrusted
    descendant onto the root pathname.
  - Blob, activation, release, and cached-policy reads use `openat` with
    `O_NOFOLLOW` and streaming limits. Writes use random fixed-directory
    temporary names, `O_CREAT|O_EXCL|O_NOFOLLOW`, `fsync`, read-only mode,
    descriptor-relative rename, and parent-directory `fsync`. Names derive only
    from validated identifiers/digests.
  - An adversarial test opens the store, renames the path-visible root, replaces
    it with a symlink or attacker directory, imports and resolves, and proves
    every byte stayed beneath the originally opened descriptor. Permission and
    policy lookup errors remain denial, never absence.
- Database/API/QML effects: none to the main application. The external release
  can additionally pass through the existing production preview CLI, but no
  catalog route, migration, remote provider, main-client launcher, or gameplay
  authority changes.
- Planned file manifest:

  | Path | Purpose |
  |---|---|
  | `crates/game-cartridge/src/sdk.rs` plus embedded `sdk/v1/*` resources | Deterministic SDK export/verification, schema inventory, lock identity, and compatibility/retirement policy. |
  | `crates/game-cartridge/src/release.rs` | Canonical release construction, domain-separated publisher attestation, exact readback verification, and provenance report. |
  | `crates/game-cartridge/src/lifecycle.rs` | Distinct catalog keys/signature, status/decision matrix, exact release binding, and fail-closed policy verification. |
  | `crates/game-cartridge/src/secure_store.rs` | Existing-root descriptor-relative import and policy-bound resolution on the supported Linux/Unix platform. |
  | `crates/game-cartridge/src/bin/omarchygs-cartridge.rs`, contract/error/lib modules | Public CLI commands, stable reports/errors, exports, and shared bounded types. |
  | `crates/game-cartridge/tests/sdk_release.rs` | SDK, provenance, lifecycle, signature/tamper, descriptor containment, and no-substitution regressions. |
  | `examples/first-party-door-legends/` | Source-only repository fixture and public-SDK build instructions with no platform-private path. |
  | `scripts/test-game-cartridge-sdk.sh` | Fresh-Git clean-room reproducibility, production CLI/preview consumption, isolation, and adversarial store proof. |
  | `Cargo.toml`, `Cargo.lock`, `bin/gate.sh`, `CONSTITUTION.md` | Direct pinned filesystem dependency and canonical SDK/release gate. |
  | Architecture, OpenWiki, Ticket 017 spec/notes/AAR | Supported release flow, lifecycle operations, privilege boundary, limitations, and durable lessons. |
- Regression map:

  | Requirement | Evidence |
  |---|---|
  | REQ-001 | Two exact SDK exports compare byte-for-byte; lock/schema/tool/version skew and inventory attacks reject; fresh Git clones build/conform with unusable database/network/credential values. |
  | REQ-002 | Two clones at one commit produce identical release bytes; source/tool/SDK/publisher/artifact/report tampering and cross-key replay reject. |
  | REQ-003 | Production `verify-release`, secure import, resolve, and existing preview CLI consume only copied SDK/release/key/policy inputs; exact verifier reports and digest identities match. |
  | REQ-004 | Table tests cover all five states for new launches and active sessions, policy signature/binding/version failures, pinned continuation, and absence of digest substitution. |
  | REQ-005 | Descriptor-root rename/symlink/attacker-directory races, symlink children, wrong types, permission errors, malicious activation/policy bytes, and revocation transitions fail closed or remain anchored. |
- Risks and rollback:
  - Filesystem APIs are platform-specific. V1 explicitly supports the Omarchy
    Linux target; unsupported targets return a stable unsupported-boundary
    error instead of falling back to path traversal.
  - SDK JSON Schemas are developer guidance plus locked contract artifacts; the
    production Rust verifier remains the authority. Tests require schema/CLI
    agreement for every shipped fixture and reject a valid-schema artifact that
    the verifier rejects.
  - Attestation proves what the publisher signed, not that an external CI host
    was honest. Future public publication may add Sigstore/SLSA transparency;
    it does not weaken exact local verification now.
  - A caller remains responsible for supplying the latest authoritative signed
    policy. Ticket 017 proves fail-closed verification and transitions without
    adding the future network catalog service.
  - Rollback removes the additive SDK/release/lifecycle/secure-store APIs and
    focused gate while leaving canonical cartridge bytes and rendering intact.
- CodeGraph design exploration traced package collection/signing through
  `verify_archive_bytes`, immutable authenticated content, conformance reports,
  current path store mutations, and the CLI/test consumers. The blast radius is
  additive production cartridge code, its CLI/tests, and a new isolated gate;
  the renderer remains a downstream verifier/preview consumer and the server,
  PostgreSQL, WebSockets, QML shell, and game runtime have no caller edge into
  the new authority. Direct review covered shell/docs/fixtures and rustix's
  descriptor APIs, which CodeGraph does not model. The Phase 2 receipt matches
  pipeline `10b7eba4-c415-4551-87ff-75084d0f015c` and the designed worktree.

## Phase 3 — Implement

- Added deterministic `sdk-export`/`sdk-verify` production commands. The exact
  export contains seven locked static files plus a canonical `sdk-lock.json`;
  every file has a compiled expected byte identity, and the lock pins SDK and
  presentation v1 plus cartridge/preview tool version `0.1.0` and structured
  deprecation/retirement behavior.
- Added a domain-separated publisher release attestation and exact three-file
  release directory. Creation packages and re-verifies the cartridge before
  signing source revision, actual builder-binary digest, tool/SDK identities,
  publisher/key, game/rules/cartridge versions, archive/signed-content IDs, and
  conformance digest. Verification re-runs the production archive verifier and
  reconstructs exact conformance bytes before accepting provenance.
- Added distinct catalog-authority keys, signed policies, and the exact five-row
  lifecycle matrix. Policy verification binds authority, nonzero version,
  game, publisher, archive, status, and bounded reason; new-launch and
  active-session decisions remain platform code rather than publisher input.
- Added Linux `SecureCartridgeStore`. One no-follow root descriptor anchors
  fixed blob/activation/release/conformance/policy descriptors. All descendant
  reads and writes use rustix `*at` operations, bounded handles, exclusive
  temporaries, read-only publication, descriptor-relative rename, and fsync.
  Resolution re-verifies stored package, conformance, publisher attestation,
  exact activation, and supplied catalog policy. Cached policy versions reject
  downgrade or same-version equivocation.
- Added five production integration tests covering exact SDK reproduction and
  drift, byte-identical releases, provenance/conformance/cross-key tampering,
  every lifecycle state, stale-policy rejection, root rename plus symlink
  replacement, fixed-child symlinks, and continued resolution through the
  originally opened descriptor.
- Added `examples/first-party-door-legends` as a source-only external-repository
  fixture and `scripts/test-game-cartridge-sdk.sh` as canonical gate 13. The
  harness creates one deterministic Git commit, makes two no-hardlink clones,
  copies production binaries, exports the SDK, builds both releases under
  unusable database/proxy/credential values, compares all release bytes,
  verifies and securely imports the artifact, and feeds it through the existing
  production previewer. No output contains the platform source path.
- Focused evidence is green: five new integration tests plus the clean-room
  Git/CLI/import/preview harness. The representative run produced revision
  `b4d4075db167693ca5d80193b9062bf0776ffe8e`, SDK lock
  `7a732939918254ca1fb399f1fa4a4ef70d252ad683c13696dec8db8e2e88a045`,
  and an exact signed archive; archive identity intentionally changes with the
  freshly generated ephemeral publisher key.
- Approved security remediation implementation:
  - Linux secure-store open now captures the effective user and validates the
    root plus every retained fixed directory with descriptor metadata. Wrong
    owner and group/other writable modes return `unsafe_filesystem_path`; the
    existing no-follow, descriptor-relative, bounded, atomic, read-only, and
    fsync behaviors remain unchanged.
  - Every policy transition obtains a fresh descriptor for the retained policy
    directory, takes an exclusive `flock`, and performs the complete
    read/verify/compare/replace while holding it. `import_release` durably
    caches the highest authenticated policy before applying launch denial.
  - Core profile admission now allows at most 1,024 px / 1 MP / 4 MiB for one
    raster and 16 MiB of referenced raster instances per scene. Rich-2D allows
    2,048 px / 4 MP / 16 MiB and 64 MiB per scene. The broad signed-package
    envelope remains available to future reviewed profiles, but cannot reach
    current QML through Core/Rich-2D.
  - Trusted Image nodes request a bounded source size, decode asynchronously,
    and use Qt's cache. The focused harness generates a valid maximum Rich-2D
    PNG and the original 4,096 px trigger on every run; the former must satisfy
    frame/RSS bounds and the latter must return `renderer_budget_exceeded`
    without publishing a plan.
- Remediation focused evidence:
  - `cargo test -p omarchygs-game-cartridge --all-targets` passed one private
    ownership test, 20 conformance tests, and eight SDK/release/store tests.
    The new corpus covers permission rejection, restart-persistent denied
    policy, and 64 concurrent v2/v3 transition trials.
  - `cargo test -p omarchygs-game-cartridge-renderer --all-targets` passed two
    unit and nine integration tests; focused Clippy passed with warnings
    denied.
  - `scripts/test-game-cartridge-renderer.sh` passed every fixed state and the
    new raster boundaries. Its first post-fix maximum Rich-2D raster sample was
    15.992 ms average / 16.807 ms maximum / 257,848 KiB peak RSS; the 4,096 px
    trigger was rejected before plan publication.
  - `scripts/test-game-cartridge-sdk.sh` passed clean-room reproducibility,
    release verification, secure import, and preview consumption with the new
    store policy.
  - `bin/gate.sh --fast` passed all 14 fast gates. Its second raster-boundary
    sample was 15.997 ms average / 16.979 ms maximum / 249,872 KiB peak RSS;
    Core and normal Rich-2D remained within their existing frame/RSS ceilings.

## Phase 3.5 — Inspect

- The user approved remediation of all four sealed Codex Security findings on
  2026-08-25. The pipeline returned to Phase 2 Design before application edits.
- Approved filesystem and lifecycle design:
  1. Capture the effective user identity when opening the store. Validate the
     already-open root and every fixed child with `fstat`: the object must be a
     directory, owned by that effective user, and not writable by group or
     other. Existing owner-readable/executable roots such as `0755` remain
     compatible; generated children remain `0700`.
  2. Reopen the retained `policies` directory as `.` for every policy
     transition, producing a distinct open-file description, acquire an
     exclusive Linux `flock`, then re-read, compare, and replace beneath that
     lock. This keeps the lock descriptor-relative, serializes threads and
     processes, and retains atomic rename plus parent `fsync` publication.
  3. Cache the highest authenticated, release-bound policy before applying its
     new-launch decision. A denied import still installs no cartridge bytes or
     activation, but the monotonic denial survives process restart and blocks
     every older signed policy.
- Approved renderer design:
  1. Keep the broad signed package envelope for future reviewed profiles, but
     add selected-profile admission before any asset reaches QML. Core accepts
     at most 1,024 px per raster side, 1 MP / 4 MiB per raster, and 16 MiB of
     referenced decoded raster data per scene. Rich-2D accepts at most 2,048 px
     per side, 4 MP / 16 MiB per raster, and 64 MiB per scene.
  2. Charge decoded raster bytes for every admitted Image or Sprite instance,
     not only each unique file, in the same tentative `Usage` update as node,
     plan, and effect budgets. Required nodes reject; optional decorations use
     the existing deterministic omission behavior.
  3. Make trusted `Image` decoding asynchronous and request only a bounded
     host-chosen source size. Sprite sheets remain bounded by Rich-2D's per-file
     and aggregate limits; the QML process remains isolated from the main shell
     in the production preview path.
  4. Extend the focused renderer gate with a valid maximum Rich-2D raster that
     must remain responsive under the existing timeout/frame/RSS harness and a
     formerly legal 4,096 px trigger that must fail at profile admission before
     QML output is published.
- Remediation regression map:

  | Finding | Required evidence |
  |---|---|
  | Directory ownership/mode | Wrong expected owner, group-writable root, world-writable fixed child, symlinked child, ordinary owner-controlled root, and descriptor-root rename cases. |
  | Policy race | Repeated concurrent v2/v3 transitions through separately opened store handles always leave v3 authoritative; ordinary sequential progression and same-policy idempotency remain valid. |
  | Denied-policy persistence | Revoked v2 denies import, survives store reopen, blocks Active v1, writes no activation/content, and still permits a later valid v3 transition. |
  | Raster availability | Pure Rust per-file/per-scene Core/Rich-2D budget cases, normal image/sprite plans, valid 2,048 px Rich-2D QML smoke, rejected 4,096 px preview, frame ceiling, and RSS ceiling. |

| # | Lens | Finding | Severity | Disposition |
|---|---|---|---|---|
| 1 | Filesystem authority | The descriptor-relative store rejects symlink traversal but accepts a pre-existing root or fixed child that is owned by another user or writable by group/other. A disposable regression changed `policies/` to mode `0777`, deleted cached policy v2, and then replayed v1 successfully. | Low security / CWE-732, CWE-276 | Confirmed and approved for remediation. Validate effective-user ownership and reject group/other write permission on every retained directory descriptor. |
| 2 | Lifecycle concurrency | Policy comparison and atomic replacement are separate operations. Concurrent valid v2/v3 updates can both pass comparison and let v2 replace v3 last; the disposable repeated test reproduced rollback within 64 trials. | Low security / CWE-362, CWE-367 | Confirmed and approved for remediation. Serialize the descriptor-relative read/compare/replace transition across processes and re-read beneath the lock. |
| 3 | Renderer availability | A legal 4096x4096 RGBA PNG compressed to 65,299 bytes but decoded to 64 MiB. Production pack/preview accepted it; the trusted QML renderer exceeded 20 seconds and approximately 227 MiB RSS under the available software backend. | Low security / CWE-400, CWE-409 | Confirmed and approved for remediation. Introduce stricter per-profile decoded and aggregate scene budgets, asynchronous or isolated decoding, and worst-case delivery evidence. |
| 4 | Lifecycle rollback | `import_release` applies the new-launch denial before caching the authenticated policy. A revoked v2 was therefore denied but forgotten, after which active v1 imported successfully. | Low security / CWE-285, CWE-693 | Confirmed and approved for remediation. Persist the highest valid policy before applying its allow/deny decision and prove the result survives restart. |

- Codex Security diff scan `da83efef-2b0f-464c-a828-d3a6f3223e5f`
  reviewed all 41 frozen worktree items against snapshot
  `codex-security-snapshot/v1:sha256:3d2e5434c27fe577bdd02d7714fadf443bbf8a56405425d2c4fa5bcab0cab117`.
  It sealed four high-confidence, low-severity findings with complete coverage
  and no deferred surfaces. Dynamic validation ran only in the disposable scan
  copy; it did not mutate production source.
- The threat-advisory connector was unavailable, so the scan makes no claim of
  current third-party advisory coverage. The four local findings were each
  established from source-to-sink review and direct disposable reproduction.
- Pre-remediation CodeGraph exploration traced the affected store API to its
  CLI and integration-test consumers and confirmed an additive blast radius in
  the cartridge crate, trusted image/sprite nodes, and profile budget checks.
  The server, database, credentials, provider network, and game authority are
  not callers of these paths. A fresh post-fix CodeGraph receipt remains
  mandatory before Phase 3.5 can pass.
- The approval-ready remediation order is: make denied policies durable;
  serialize monotonic policy transitions; validate descriptor ownership/mode;
  then lower and enforce renderer-profile media budgets with asynchronous or
  isolated decode. Focused adversarial regressions precede the full gate.
- Phase 3.5 remains FAIL while the approved fixes are implemented and
  independently re-inspected. The durable status has returned to Phase 2
  Design PASS so application edits remain phase-correct.
- Implementation deviation: the first compile used the private
  `rustix::ugid::Uid` path. The implementation was immediately corrected to
  the public `rustix::process::{geteuid, Uid}` API, formatted, and all focused
  tests plus Clippy were rerun successfully. No contract or architecture
  decision changed.
- Fresh post-fix CodeGraph inspection traced `open_existing`, directory
  validation, `import_release`, `cache_policy`, raster descriptor lookup,
  per-reference charging, asset publication, CLI consumers, and integration
  tests on the final implementation. It confirmed that policy caching occurs
  beneath the exclusive retained-directory lock before lifecycle denial and
  that Image/Sprite charging occurs before asset or plan publication. The
  one-hop blast radius remains the production cartridge CLI/tests and trusted
  preview renderer; no server, PostgreSQL, credential, or gameplay-authority
  caller bypass was found. The matching CodeGraph receipt is current for
  pipeline `10b7eba4-c415-4551-87ff-75084d0f015c`.
- Post-remediation Codex Security diff scan
  `8be3cad5-e1c8-48b0-aab9-f086055cd4bc` sealed all 41 inventory items at
  snapshot
  `codex-security-snapshot/v1:sha256:6707b9a1048b002716b90996f16f58056fdb810070fc20c91ba5b266ce3ff899`
  with complete coverage and zero reportable findings. Its readable report is
  `/tmp/codex-security-scans-TFWcqD/omarchy_bbs/e4a059cddc44961604efc252c0651275c4b1107d_20260825T134347Z_lz7ew79i/report.md`.
- All four findings from scan `da83efef-2b0f-464c-a828-d3a6f3223e5f`
  are fixed. Its required visible remediation artifact is
  `/tmp/codex-security-scans-TFWcqD/omarchy_bbs/e4a059cddc44961604efc252c0651275c4b1107d_20260825T082056Z_5ge1nwlt/artifacts/fix_report.md`.
- The post-fix scan challenged three additional hypotheses:
  - a signed one-pixel PNG with a 512 MiB IDAT expansion completed the real
    software-QML smoke in 3.424 seconds at 239,092 KiB, matching ordinary
    Rich-2D behavior, so the decoder-amplification candidate was suppressed;
  - a 90,000-entry, 7,560,098-byte ZIP was rejected through the real CLI in
    0.255 seconds at 67,156 KiB, so the bounded local parser-overhead candidate
    was suppressed; and
  - a hostile process already running as the exact store-owner UID can delete
    that UID's cache, but the final attack-path policy rejected it because the
    effect is self-only and has no privilege delta or cross-user/server reach.
    A future privileged/shared launcher or public catalog must use a dedicated
    service identity plus an authority outside the desktop UID, or an online
    current-policy check, and requires a new security review.
- Inspect result: PASS. The lower-principal filesystem boundary, cooperating
  writer concurrency, denial persistence, and QML availability regressions are
  closed; no reportable finding or deferred review surface remains.

## Phase 4 — Validate

- `bin/gate.sh --diff` passed all 16 canonical gates and wrote worktree receipt
  `1b93cc2321f6fa94ef68a46948296135b4892167976bd0fbe3bffd1a29929cf6`.
- Production Rust validation passed formatting, Clippy with warnings denied,
  unit/integration/doc tests, docs, Compose validation, pipeline structure,
  secret scan, hook self-tests, shell syntax, and whitespace checks.
- The full PostgreSQL corpus passed 33 sequential integration tests. The live
  PostgreSQL → Rust API → QML smoke passed against the local server.
- Cartridge gates passed the production verifier, 20 hostile conformance
  tests, eight SDK/release/store tests, two renderer units, nine renderer
  integrations, the isolated provider proof, clean-room release flow, and all
  trusted QML states.
- The canonical raster samples remained within their ratified envelopes:
  Core 15.998 ms average / 16.335 ms maximum / 132,688 KiB peak RSS; Rich-2D
  16.000 / 18.668 / 244,664; the real 2,048-pixel boundary 16.006 / 16.623 /
  250,312; accessibility 16.001 / 16.726 / 237,864. The 4,096-pixel trigger was
  rejected before plan publication.
- Validation result: PASS. Any later gated Phase 5 documentation edit requires
  a fresh `--diff` receipt before completion.

## Phase 5 — Complete

- EARS audit:
  - REQ-001 PASS — two deterministic SDK exports compare byte-for-byte; the
    exact lock pins schema, tool, protocol, compatibility, deprecation, and
    retirement identities; two clean Git clones build with unusable platform
    database, proxy, and credential settings.
  - REQ-002 PASS — the two clones produce identical read-only archive,
    conformance, and attestation bytes, and verification binds source revision,
    builder binary, SDK, publisher/key, game versions, artifact identity, and
    report digest while rejecting every tamper and cross-key case.
  - REQ-003 PASS — the production verifier, release verifier, secure importer,
    and previewer consume only copied public binaries, exported SDK, explicit
    keys/policy, and the three-file release; no source-tree, database, provider,
    or platform credential integration enters the path.
  - REQ-004 PASS — the exact five-state lifecycle matrix covers new launches
    and active sessions, binds a distinct catalog authority to the exact
    release, persists authenticated denial, rejects stale/equivocating policy,
    and never substitutes another digest.
  - REQ-005 PASS — retained directories are no-follow descriptor-relative,
    expected-owner and non-group/world-writable; root replacement and fixed
    symlinks stay contained, concurrent versions remain monotonic, denied policy
    survives restart, and permission or verification uncertainty fails closed.
- OpenWiki run `064b68f2-1fc2-471e-a574-d39adc201974` reconciled
  `game-cartridges.md`, `quickstart.md`, `product-boundaries.md`, and
  `development-and-validation.md` with zero stale or unresolved claims. Its
  completion receipt records pipeline
  `10b7eba4-c415-4551-87ff-75084d0f015c` and state
  `0a70c181f456aa850ccf6b5d54444a1648d30c17c04ec93cf3121ef3e736851c`.
- AAR-017 records four fixed failures, four standing prevention rules, the v1
  public SDK/release/lifecycle/import decision, the same-UID authority limit,
  and the conditions that require a future security review.
- After the final generated-wiki edit, `bin/gate.sh --diff` passed all 16 gates
  and wrote the matching worktree receipt
  `0a70c181f456aa850ccf6b5d54444a1648d30c17c04ec93cf3121ef3e736851c`.
  This included 33 sequential PostgreSQL tests, the live PostgreSQL → Rust API
  → QML smoke, 20 cartridge conformance tests, eight SDK/release/store tests,
  two renderer units, nine renderer integrations, clean-room release/import,
  raster boundary evidence, and the isolated provider proof.
- Ticket 017 is closed and the spec/notes pair is archived. No commit, push,
  pull request, SDK publication, or external repository creation was performed.

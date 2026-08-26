---
title: Native Omarchy client package and clean-install smoke — notes
pipeline_id: b991a2ec-1d25-4651-ae8f-c58b4ef211be
---

# Native Omarchy client package and clean-install smoke — running notes

Chronological evidence and decisions. If a test or command did not run, these
notes must not claim it passed.

## Phase 1 — Plan

- Recall: no critical bulletin or active pipeline blocks work. Tickets 026 and
  027 were delivered to remote `main` at `4808a9e55679265fd1aeb73352f751c0ea3b9a6f`
  before this pipeline opened.
- Recall: the roadmap's first unchecked private-alpha outcome is the
  installer/package path; sysop operations and invite-only testing follow it.
- Recall: the current QML client already accepts a safely normalized
  `--server-url`, restricts remote origins to HTTPS, holds Bearer/MFA authority
  in process memory, and has fixture plus real migrated API smoke coverage.
- Recall: `PR-omarchy-bbs-verify-the-vertical-slice-001` requires proof through
  the real consumer, while `PR-omarchy-gaming-system-compile-production-qml-root-after-control-edits-001`
  and `PR-omarchy-gaming-system-own-headless-qt-test-environment-001` require
  production-root and deterministic offscreen QML evidence.
- Recall: `PR-omarchy-gaming-system-pin-executable-artifacts-before-install-001`,
  `PR-omarchy-gaming-system-use-nul-git-path-inventories-001`, and
  `PR-omarchy-gaming-system-gate-independent-executable-source-trees-001`
  require explicit payload/provenance controls and canonical gate coverage for
  delivered executable boundaries.
- Platform evidence: this workstation identifies as Omarchy 4.0.0/Arch and
  provides `makepkg` 7.1, `pacman`, `qml6`/Qt 6.11.2, `bsdtar`, and
  `desktop-file-validate`. The installed first-party
  `omarchy-dev-pkg-test` command builds local checkout packages through
  `makepkg` and installs with `pacman -U`; `qml6` and the required QtQuick/QML
  modules are owned by `qt6-declarative`.
- Decision: ship a non-privileged, client-only Arch package builder and test
  the extracted artifact. Installing into the workstation, server packaging,
  public repository/signing infrastructure, and new client authority are out.

## Phase 2 — Design

- Architecture and data flow:
  1. `scripts/check-client-package-source.sh` canonicalizes a selected source
     root, validates every packaging input as a non-symlink regular file,
     requires a newline-terminated set of safe sorted unique manifest records,
     and compares the manifest exactly with the complete non-test `client/qml/`
     file inventory. It also binds the PKGBUILD version to the workspace
     version and checks the shell launcher and desktop entry before any build
     work.
  2. `scripts/build-client-package.sh` invokes that source validator, computes
     a deterministic aggregate source digest over the manifest and packaging
     inputs, records the exact Git revision plus dirty state, and copies only
     the reviewed PKGBUILD into a private temporary build directory. It exports
     those bounded values and a stable `SOURCE_DATE_EPOCH` to `makepkg`.
  3. The PKGBUILD installs only the manifest-listed trusted QML files under
     `/usr/share/omarchy-gaming-system/qml`, the reviewed launcher under
     `/usr/bin/omarchygs`, the desktop entry under `/usr/share/applications`,
     and a fixed non-secret provenance record. The package depends on the
     system-owned `qt6-declarative` runtime and contains no server binary,
     PostgreSQL, Qt copy, test fixture, key, or credential.
  4. The launcher resolves its own installed/extracted `../share` tree, checks
     the packaged production root, and executes `qml6 Main.qml -- "$@"`. The
     explicit Qt option terminator prevents a server URL or future application
     argument from becoming a QML runner/import/plugin option.
  5. `scripts/test-client-package.sh` builds twice into separate temporary
     outputs, proves identical package bytes, inspects `.PKGINFO` and the exact
     archive payload/modes/types, extracts without privileged installation,
     and launches that extracted payload offscreen against the bounded normal
     fixture server. The fixture verifies that only the expected `/health`
     request crossed the boundary.
  6. `bin/gate.sh --diff/--full` runs the package conformance before the
     database/live paths and writes no receipt unless it passes. `--fast`
     remains portable to the Ubuntu CI runner, which does not provide Arch
     `makepkg`, `pacman`, or Qt QML.
- Ownership and authority: the installed files are platform-owned trusted QML,
  not Game Cartridge publisher content. The package holds no credentials;
  Bearer and MFA authority remain process-memory-only inside the unchanged
  QML client. Package construction is non-privileged and never invokes
  `pacman -U`; a human tester explicitly authorizes installation later.
- Database/migration consequences: none. The artifact includes no server or
  migration files and changes no database schema or state.
- API/compatibility consequences: none. `omarchygs --server-url=<origin>` maps
  to the existing QML application argument. The client retains localhost HTTP,
  remote HTTPS, exact health-document, redirect, response-size, and timeout
  rules. Existing source-checkout `qml6` and `scripts/dev.sh` paths remain
  supported.
- Exact file manifest:
  - `packaging/arch/PKGBUILD` — native client metadata and bounded `package()`
    copy logic; no fetch, install hook, or network action.
  - `packaging/arch/client-runtime-files.txt` — exact sorted non-test runtime
    inventory below `client/qml/`.
  - `packaging/arch/omarchygs` — relocatable command launcher with the Qt
    option terminator.
  - `packaging/arch/com.ignibyte.OmarchyGS.desktop` — Omarchy app-menu entry.
  - `scripts/check-client-package-source.sh` — reusable fail-closed source and
    manifest validator.
  - `scripts/build-client-package.sh` — non-installing reproducible artifact
    builder and SHA-256 sidecar writer.
  - `scripts/test-client-package.sh` — hostile-source, metadata, archive,
    reproducibility, desktop, and extracted-QML runtime evidence.
  - `bin/gate.sh`, `CONSTITUTION.md` — canonical DIFF/FULL package gate.
  - `README.md`, `docs/client-installation.md`, `docs/product-charter.md`,
    `docs/planning/ROADMAP.md`, and affected OpenWiki pages — user install
    workflow, limitations, and completed roadmap outcome.
  - Ticket/spec/notes/AAR/knowledge register — lifecycle evidence only.
- Regression plan:

| Requirement | Evidence |
|---|---|
| REQ-001 | Two private-output builds compare byte-for-byte, emit matching SHA-256 sidecars, create no package database state, and leave `git status --porcelain=v1 -z` unchanged. |
| REQ-002 | `.PKGINFO` asserts exact name/version/release/architecture/dependency and `pacman -Qip` accepts the artifact. |
| REQ-003 | Archive paths compare with a generated expected payload; `bsdtar` type/mode inspection rejects links, writable trusted code, tests, backend paths, or extra payload. |
| REQ-004 | The extracted `/usr/bin/omarchygs` launches its sibling packaged QML tree using host `qml6` under forced offscreen/software rendering and receives the valid fixture health document. |
| REQ-005 | Current source passes; isolated copies separately prove missing, extra, duplicate, traversal, unsorted, unterminated, and symlink manifest/source rejection. |
| REQ-006 | `desktop-file-validate` passes and exact fields bind `Exec=omarchygs`, `Terminal=false`, and the Game category. |
| REQ-007 | Documentation review covers all seven required operator/tester facts and does not claim public signing, auto-update, or persistent login. |
| REQ-008 | A fresh full diff gate prints the package stage PASS and `GATE GREEN [diff]`, then writes the exact worktree receipt. |

- Security/privacy risks and mitigations:
  - A recursive copy could ship fixtures or future unintended files; the safe,
    exact manifest is checked against the complete runtime inventory before
    both builds and archive acceptance.
  - Manifest paths could inject traversal, options, whitespace, or links; the
    checker permits a narrow ASCII relative-path grammar, rejects `.`/`..`,
    empty/duplicate/unsorted records, uses option terminators, canonicalizes
    roots, and requires non-symlink regular sources.
  - Application arguments could be interpreted by `qml6`; the launcher owns
    the source path and inserts `--` before every caller-supplied argument.
  - Package output can be mistaken for authenticated public distribution;
    docs and provenance call it unsigned private-alpha output. Public signing,
    repository trust, revocation, and publication remain an explicit later
    supply-chain slice.
  - Build provenance could leak local paths or credentials; the installed
    record contains only fixed keys, workspace version, Git revision/dirty
    state, and a source SHA-256. Temporary paths, environment, and user names
    are excluded from the payload contract.
  - QML runtime behavior could depend on the checkout; the smoke runs from an
    extracted package root, while only the fixture process remains test-owned.
- Operational/reconnect/rollback risks: package replacement uses normal
  `pacman -U`; removal uses normal `pacman -Rns`. The client retains no local
  durable state in this slice, so rollback is package downgrade/removal plus
  the existing server-side session lifecycle. Closing/relaunching requires a
  new sign-in because the raw Bearer is not persisted.
- Alternatives rejected:
  - A curl-to-shell installer would duplicate Arch ownership, bypass pacman
    inventory/removal, and create a larger privileged script surface.
  - Bundling Qt would enlarge the artifact and fork Omarchy's system update
    boundary.
  - AppImage/Flatpak were deferred because the target OS already provides a
    native package manager and neither format improves this first clean-install
    proof enough to justify another runtime/sandbox/update contract.
  - Packaging the Rust server together with the player client would conflate
    independent community-server and player-device deployment units.
  - Installing into the active workstation during the gate would mutate global
    state and make rollback part of every validation run; extracted-package
    execution proves the payload without that risk.
- CodeGraph evidence: design exploration for `Main.qml`,
  `OnboardingController.initialize`, `ApiClient`, `scripts/dev.sh`, and
  `bin/gate.sh` found only ambiguous indexed Rust `health` symbols. CodeGraph
  does not parse QML or shell here, so no Rust/API call path or dependent is
  changed by this slice. Direct inspection of the production QML root,
  onboarding/API controllers, fixture runner, gate, and Omarchy package tooling
  is authoritative for the package flow. The successful worktree-bound query
  supplies pipeline `b991a2ec-1d25-4651-ae8f-c58b4ef211be` design evidence.

## Phase 3 — Implement

- Built:
  - Added `omarchy-gaming-system-client` 0.1.0-1 as an `any` Arch package
    depending only on `qt6-declarative`. Its `package()` function copies the
    exact 37-file trusted production-QML manifest, the relocatable
    `/usr/bin/omarchygs` launcher, one Game desktop entry, and fixed build
    provenance. It has no sources, fetch hook, post-install hook, server
    payload, test fixture, credential, key, or packaged Qt library.
  - Added a source checker that rejects unterminated,
    unsafe/empty/duplicate/unsorted/stale records, missing or extra production
    runtime files, symlink/non-regular inputs, workspace/package version drift,
    invalid launcher Bash, and an invalid desktop entry before build work.
  - Added a non-installing builder that computes a NUL-record aggregate source
    digest, records exact revision/dirty state, serializes `makepkg` through a
    mode-0700 owner-checked stable per-user workspace, and atomically publishes
    one package plus SHA-256 sidecar to the selected non-symlink output
    directory.
  - Added package conformance that constructs seven hostile source copies,
    proves the builder stops before creating output for an invalid source,
    compares two package builds byte-for-byte, inspects exact Arch metadata and
    payload/modes/types/provenance, validates the desktop entry, extracts
    without `pacman -U`, and launches the packaged production root through the
    real launcher against the bounded health fixture under offscreen/software
    Qt.
  - Added portable source-contract gate 15 in every mode and Omarchy-only full
    package gate 16 in DIFF/FULL, shifting the existing integrated gates to
    17–20. Updated the Constitution, README, client installation guide, product
    charter, system overview, and private-alpha roadmap.
  - Focused evidence: `scripts/test-client-package.sh` passed with two
    byte-identical packages at SHA-256
    `ee294b9bcd73812a296ecb5ee84bc88f7ce419f2965d1a292cc317fee3128ca4`
    and 37 runtime files. `bin/gate.sh --fast` passed all 15 portable stages.
- Deviations:
  - The first conformance run incorrectly looked for `qml6` under the internal
    `qmake6` binary directory. Arch owns the actual runtime at `/usr/bin/qml6`;
    the test now resolves the real command and uses `C.UTF-8` for Qt.
  - Two valid first builds differed because Arch embeds `startdir` and
    `builddir` in `.BUILDINFO`. The builder now uses a serialized, private,
    owner/mode-checked stable per-user build workspace while keeping package
    destinations temporary. Repeated identical builds then matched exactly.
  - The desktop entry initially declared two main categories and produced a
    standards hint. It now uses the single `Game` main category while its name,
    generic name, comment, and keywords retain the network-community meaning.

## Phase 3.5 — Inspect ledger

| # | Lens | Finding | Severity | Disposition |
|---|---|---|---|---|
| 1 | Correctness / package manifest | The checker accepted an unterminated final manifest record, while the builder, PKGBUILD, and expected-payload loops would omit that record. | medium | Fixed by making newline termination an explicit source invariant and adding an unterminated-manifest rejection fixture; the full package test passed afterward. |
| 2 | Security / supply chain | The final-snapshot Codex Security diff scan reviewed every changed executable and package-control file and found no reportable vulnerability. | none | PASS — sealed report at `/tmp/codex-security-scans/omarchy_bbs/4808a9e-ticket028-final-OanFwovL/report.md`, snapshot `b7623ae87337f4ccea1dc6f8258492334fc29f3894b90442a54877c5f0dd0072`, complete coverage, zero findings. |
| 3 | Trust boundary | An adjacent SHA-256 sidecar does not authenticate an artifact distributor. | expected boundary | Accepted only for unsigned private-alpha local builds; README, installation guide, architecture, product charter, and threat model state that public signing/repository trust is absent. |
| 4 | Local build isolation | Byte reproducibility requires a stable build path, creating a retained per-UID workspace. | low | Accepted with non-symlink, UID-owner, mode-0700, and `flock` enforcement; package destinations remain private temporary directories and tests prove source status is unchanged. |
| 5 | Static package tooling | `shellcheck` and `namcap` are not installed on the Omarchy workstation. | informational | No claim was made that they ran. Bash/PKGBUILD syntax, `desktop-file-validate`, two real `makepkg` builds, `pacman -Qip`, exact extraction, and runtime smoke provide the required evidence. |
| 6 | Simplification / blast radius | Final CodeGraph inspection cannot parse the changed shell, PKGBUILD, desktop, or manifest sources and returned unrelated Rust symbols for ambiguous terms. | informational | No indexed Rust/API path changed. Direct full-file review is authoritative for the unsupported sources; focused package conformance and the canonical gate own behavior proof. |

- Post-implementation CodeGraph evidence: a fresh worktree-bound MCP
  exploration ran after the final manifest fix for the package scripts,
  controls, and gate. Its ambiguous provider/manifest matches did not establish
  a relevant caller. The direct eight-file inspection, exact archive
  assertions, extracted-QML smoke, and sealed final-snapshot security scan
  close the actual blast radius.
- Requirements/doc review: all eight EARS rows retain direct evidence. No REST,
  WebSocket, database, game, provider, cartridge, credential-persistence, or
  server-packaging contract changed.

## Phase 4 — Validate

- Tests run:
  - `scripts/check-client-package-source.sh` passed the final source contract,
    including explicit newline termination.
  - `scripts/test-client-package.sh` passed seven hostile source fixtures,
    two byte-identical native builds, exact metadata/payload/mode/provenance
    inspection, desktop validation, extraction, and the packaged production
    QML root's loopback health smoke. The final artifact SHA-256 was
    `ee294b9bcd73812a296ecb5ee84bc88f7ce419f2965d1a292cc317fee3128ca4`.
  - `scripts/check-pipeline.sh`, `git diff --check`, and Bash/PKGBUILD syntax
    checks passed after the inspection fixes and evidence updates.
- Gate run: `bin/gate.sh --diff` passed all 20 canonical stages and wrote the
  matching worktree receipt
  `ed87c43903e2cfd99f4b3adb55c85920823b49314d5afa82183993d70ee2b8d5`
  at 2026-08-26 11:56:36 -0500. The receipt was compared with a fresh
  `ogs_gate_state_hash` before advancing this phase.
- Skips or pre-existing failures: none. `shellcheck` and `namcap` were not
  installed and are recorded as unavailable inspection aids, not as passing
  validation commands.

## Phase 5 — Complete

- Acceptance-criteria audit:
  - REQ-001 PASS — the builder validated source, emitted exactly one Arch
    package and matching SHA-256 sidecar twice, produced byte-identical bytes,
    and left source-tree status unchanged without installing software.
  - REQ-002 PASS — `pacman -Qip` and extracted `.PKGINFO` proved exact package
    name `omarchy-gaming-system-client`, version `0.1.0-1`, `any`
    architecture, and sole application dependency `qt6-declarative`.
  - REQ-003 PASS — exact archive comparison proved the 37 allowlisted QML
    files, launcher, desktop entry, and non-secret provenance with regular-file
    types and fixed modes; tests, server/provider code, credentials, and build
    tools were absent.
  - REQ-004 PASS — the extracted relocatable launcher invoked packaged
    `Main.qml` through `qml6` with the Qt option terminator, reached only the
    loopback fixture `/health`, and exited under the offscreen smoke without a
    checkout, Cargo, Docker, or pacman installation at runtime.
  - REQ-005 PASS — positive source admission and seven isolated missing,
    extra, duplicate, traversal, unsorted, unterminated, and symlink fixtures
    proved fail-closed behavior before package output.
  - REQ-006 PASS — `desktop-file-validate` and exact packaged fields proved
    `Exec=omarchygs`, `Terminal=false`, and `Categories=Game;`.
  - REQ-007 PASS — the installation guide documents inspect/build, install,
    launch, update, and removal plus remote HTTPS, process-memory credentials,
    client/server separation, and the unsigned private-alpha boundary.
  - REQ-008 PASS — `bin/gate.sh --diff` ran the package source contract as
    gate 15 and full artifact conformance as gate 16 before all later
    integration stages and wrote the matching Phase 4 receipt.
- Docs: OpenWiki update run `591c441c-573e-4b97-9d20-bc3f1f99d627`
  added the native package lifecycle to the quickstart and development guide,
  updated the provider stages to gates 19/20, and returned `status: complete`.
  Its completion receipt matches pipeline
  `b991a2ec-1d25-4651-ae8f-c58b4ef211be` and gated state
  `0c50f1a0027d9d4a6993d029fcd805e17927ce4502f1bee9f4ac0f2a1b1cfaf0`.
- AAR: submitted at effectiveness 5 with four captured failures, four standing
  prevention rules, and the client-only native package architecture decision
  registered in the knowledge index.
- Archive: ticket moved to closed and spec/notes moved to completed. No active
  spec/notes pair remains. The final post-archive `bin/gate.sh --diff` passed
  all 20 stages and wrote the matching gated-state receipt
  `0c50f1a0027d9d4a6993d029fcd805e17927ce4502f1bee9f4ac0f2a1b1cfaf0`
  at 2026-08-26 12:06:50 -0500; the OpenWiki completion receipt remained
  matching at the same state.

## Defect and lesson ledger

| # | What happened | Root cause | Fix | Prevention |
|---|---|---|---|---|
| 1 | The first package smoke searched for `qml6` beside an internal qmake binary path. | The test inferred executable layout from a related Qt tool instead of resolving the delivered runtime command. | Resolve and execute the actual `qml6` command owned by `qt6-declarative`. | `PR-omarchy-gaming-system-resolve-runtime-executables-directly-001` |
| 2 | Two valid packages differed in `.BUILDINFO`. | Arch records `startdir` and `builddir`, and independently randomized build roots changed those fields. | Use one private, owner/mode-checked, serialized per-UID build root while keeping output roots temporary. | `PR-omarchy-gaming-system-stabilize-package-build-paths-for-reproducibility-001` |
| 3 | The desktop validator emitted a category hint. | The entry declared two main categories for one application. | Retain `Game` as the sole main category and express network/community context in the other descriptive fields. | Prefer one standards-defined main desktop category unless the application truly spans both. |
| 4 | An unterminated manifest could pass source validation but lose its last record in downstream consumers. | The checker intentionally handled a final non-newline record while the builder, PKGBUILD, and test expected POSIX newline-terminated text. | Require newline termination before record parsing and prove rejection with a hostile fixture. | `PR-omarchy-gaming-system-enforce-line-manifest-termination-001` |
| 5 | The first terminal security finalization rejected the canonical draft. | `findings.json` omitted the scan ID; the terminal finalizer has no workbench binding from which to synthesize it. | Preserve the scan, add the same ID used by manifest and coverage, and finalize it successfully on continuation. | `PR-omarchy-gaming-system-bind-terminal-scan-document-identities-001` |

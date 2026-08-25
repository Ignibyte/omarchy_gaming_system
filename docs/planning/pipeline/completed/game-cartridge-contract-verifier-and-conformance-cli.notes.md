---
title: Game Cartridge contract, verifier, and conformance CLI — notes
pipeline_id: 003d6707-f08b-4add-b612-705f5a0cc7bb
---

# Game Cartridge contract, verifier, and conformance CLI — running notes

Chronological evidence only; commands not run are not reported as passing.

## Phase 1 — Plan

- Ticket 014 completed with matching gate/OpenWiki receipts and no active
  spec/notes pair. Ticket 015 is the first accepted follow-up and the only new
  active pipeline.
- Bulletin `BUL-001-initial-push-pending` remains a warning: local work may
  continue, but the renamed GitHub repository still has no confirmed remote
  `main`; no commit or push is authorized in this pipeline.
- Recalled `AD-omarchy-gaming-system-game-cartridge-provider-boundary-001`:
  signed data-only cartridges are the accepted frontend direction, compiled
  OmarchyGS rules remain production authority, and provider work stays out of
  this ticket.
- Recalled Ticket 014 prevention rules: parse the bytes that were authenticated,
  enforce resource bounds during streaming, render untrusted text explicitly,
  bound all package traversal work, gate nested evidence workspaces, and bind
  signed identities to their registered context.
- OpenWiki recall: the Ticket 014 workspace is an isolated proof, not a public
  SDK. Ticket 015 owns deterministic packaging, strict verification,
  compatibility, conformance, and a local store; Ticket 016 owns rendering.
- Smallest production slice: a stored-only canonical ZIP avoids compression
  nondeterminism and decompression bombs in v1, while distinct archive and
  expanded byte ceilings remain explicit. A strict signed integrity index and
  exact canonical reconstruction make alternate ZIP encodings reject.
- Local store scope is deliberately filesystem-only. It provides immutable
  content-addressed blobs and atomic activation/revocation metadata without a
  server route, database migration, network client, or publisher registry.
- Official crate research: current `zip` 8.6 exposes stored compression,
  explicit fixed modification time and Unix permission options; current
  `ed25519-dalek` documents OS-random signing-key generation, byte
  serialization, signing, verification, and strict verification. The design
  pins reviewed exact major behavior and will compile/test the selected versions
  before application code is accepted.

## Phase 2 — Design

- CodeGraph exploration traced the proof's `sign_cartridge`,
  `verify_cartridge`, retained-byte `read_cartridge_files`, manifest and
  presentation validation, and callers in `cartridge-tool`, broker, and
  provider. The clean seam is a new production packaging crate; the provider
  grant/message code remains spike-only. No server route, game runtime,
  migration, or production QML caller depends on the proof package types.
- CodeGraph's test-link heuristic reported no coverage for proof functions even
  though the proof's seven in-module tests and shell flow were directly
  inspected and previously executed. As required by the recalled rule, graph
  coverage remains advisory. Design receipt for pipeline
  `003d6707-f08b-4add-b612-705f5a0cc7bb` matches gated state
  `c1e50be8ed5518601117dd54046cab4dd555711cdc15218df22792cfe6833fa0`.
- Selected `zip` 8.6.0 with default features disabled. `cargo info` records MIT,
  Rust 1.88, and optional compression/crypto/time features; stored-only v1 uses
  none of them. Direct source inspection confirmed raw name, encryption,
  compression, compressed/uncompressed size, directory/symlink/file, Unix mode,
  comment, and extra-data inspection plus deterministic writer options.
- Selected `ed25519-dalek` 2.2.0 with `rand_core`; `cargo info` records
  BSD-3-Clause and Rust 1.81. It is already exercised by the spike and keeps
  signing keys zeroized by default. Production verification will use strict
  verification and a new `omarchygs-cartridge-integrity-v1` domain separator,
  never the spike envelope domain.
- Locked canonical reconstruction as a second parser boundary: after archive,
  path, metadata, size, digest, signature, and typed-content verification, the
  verifier rewrites the retained entries with the canonical writer and requires
  byte-for-byte equality with the bounded input. This catches duplicate central
  records, alternate ordering, comments/extras, platform metadata, and other
  unsigned container encodings without extracting to disk.
- Locked package and compatibility validation as separate outcomes. A malformed
  or unauthenticated artifact is rejected; a valid artifact with an unsupported
  required capability produces a stable incompatible report and is rejected by
  install. Missing optional capabilities select their signed fallback.
- Locked the first media allowlist to PNG and PCM WAV. Their bounded headers
  provide enough information to verify dimensions, decoded pixels/bytes, audio
  format, and duration without pulling a general decoder into the installer.
  WebP/Ogg wait for Ticket 016's renderer/decoder threat review and enter only
  through a later contract capability.
- Locked local storage to archive bytes, not expanded files. Content-addressed
  immutable blobs reduce parser surface and preserve exact provenance; future
  rendering re-verifies/opens the blob through the same library. Activation and
  revocation are small atomic allowlisted JSON records, not mutable package
  directories.
- Direct review covered root Cargo membership, gate 11, the shell proof, docs,
  fixtures, and QML unsupported by CodeGraph. Production package work adds one
  root workspace member and focused gate script; server/game/QML sources remain
  outside the file manifest.

## Phase 3 — Implement

- Added the root-workspace `omarchygs-game-cartridge` production library and
  `omarchygs-cartridge` CLI. The crate has no HTTP, async runtime, SQL,
  dynamic-loader, QML, or platform-credential dependency; its public boundary
  is pack, verify/conform, compatibility, and local install/revoke/resolve.
- Implemented canonical stored-only ZIP writing and strict raw-entry inspection
  for ASCII allowlisted paths, sorted uniqueness, fixed epoch, read-only Unix
  mode, no compression/encryption/comments/extras/directory/link entries, and
  archive/entry/expanded ceilings. Verification retains the bounded bytes,
  verifies a domain-separated Ed25519 integrity index, validates typed content,
  then reconstructs and byte-compares the complete archive.
- Implemented strict manifest/presentation contracts, a bounded local JSON
  Schema 2020-12 subset with remote/unknown keywords prohibited, bounded
  localization, and PNG/PCM-WAV metadata validation. The v1 trusted node set is
  intentionally still `terminal`, `grid`, and `status`; the broader Rich 2D
  vocabulary remains Ticket 016 work.
- Implemented stable SDK/presentation version results, required-capability
  failure, all six typed optional fallbacks, machine-readable provenance, and
  distinct valid-but-incompatible behavior. A simpler-capability fallback must
  name a required baseline capability so the selected fallback cannot itself
  be unavailable.
- Implemented exact-archive SHA-256 blobs plus atomic activation and revocation
  records under an explicit local root. Stored archives and metadata are
  `0444`, never extracted or executed, reverified during resolution, and denied
  after revocation. Incompatible artifacts create no store state.
- Added a 19-test corpus covering deterministic identity, valid changes,
  signature tampering, oversized input, traversal, duplicate names, symlinks,
  executable mode, compression/noncanonical metadata, source links, hostile
  correctly signed schemas/media, PNG/WAV bounds, version/capability behavior,
  CLI isolation, filesystem permissions, content addressing, and fail-closed
  revocation.
- Added `scripts/test-game-cartridge.sh` for an end-to-end CLI run. It builds
  the same source under different mtimes/permissions, proves identical output,
  conforms with unusable network/database/credential environment values,
  installs a `0444` blob, and revokes it. The focused run passed 15 tests and
  produced a 2,469-byte canonical fixture archive.
- Promoted this production conformance script to gate 11 and retained the
  isolated provider/QML architecture proof as gate 12; DIFF/FULL database and
  application smoke gates are now 13 and 14. Constitution §0 documents the
  additive gate rather than weakening the existing proof.
- During implementation review, found that the first verifier draft rechecked
  schemas/localizations only during packing. Correctly signed malicious
  publishers are still an input threat, so verification now reparses those
  authenticated bytes and requires their canonical representation; a
  malicious-signer regression test proves both schema and media rejection.
- Actual focused evidence: `cargo clippy -p omarchygs-game-cartridge
  --all-targets -- -D warnings` passed; `cargo test -p
  omarchygs-game-cartridge --all-targets` passed 19 tests; and
  `scripts/test-game-cartridge.sh` passed its deterministic CLI/install flow.

## Phase 3.5 — Inspect ledger

| # | Lens | Finding | Severity | Disposition |
|---|---|---|---|---|
| 1 | Media/resource accounting | PNG validation trusted width × height × 4 without constraining bit depth or compressed ancillary chunks, so a correctly signed 16-bit image could underdeclare future decoder memory. | High before a renderer exists | Fixed: v1 accepts only CRC-checked 8-bit `IHDR`/`PLTE`/`IDAT`/`IEND` PNGs and accounts normalized RGBA bytes. Correctly signed 16-bit, corrupt-CRC, and compressed-ancillary regressions reject. |
| 2 | Capability compatibility | A presentation could instantiate a Grid/Status/Terminal node without declaring the matching required host capability. | Medium | Fixed: presentation validation binds every node family to its versioned required capability before compatibility evaluation; pack and malicious-signer verification regressions reject omission. |
| 3 | Input/resource handling | Path verification checked metadata and then used unbounded `fs::read`, allowing a selected FIFO or replaced file to stream beyond the archive limit before rejection. | Medium | Fixed: archive, source, key, host-profile, activation, revocation, and blob paths use bounded regular-file reads; the library checks handle metadata and Unix inode/device identity. Non-regular/symlink path regressions reject. |
| 4 | Store containment | Install uses pathname-based ancestor checks and atomic rename, leaving a theoretical ancestor replacement race if a privileged installer writes beneath an attacker-mutable root. | Deferred boundary | No supported privilege boundary makes this exploitable; 20,000 adversarial attempts produced no escaped write. Ticket 015 now explicitly supports only same-user roots, and Ticket 017 requires descriptor-relative containment or an OS sandbox before privileged/multi-user import. |
| 5 | Revocation mutation | Revocation metadata has the same ancestor pathname race under the unsupported privileged/attacker-mutable-root preconditions. | Deferred boundary | Same disposition as #4; no privilege or shared-root deployment is authorized, and Ticket 017 carries the production hardening requirement. |
| 6 | Revocation lookup | `Path::exists` converted lookup errors into “not revoked,” allowing resolution to fail open if the revocation path was inaccessible. | High correctness | Fixed: the store validates its directory shape, distinguishes `NotFound` from all other errors, bounded-parses canonical revocation records, and denies malformed/link/error paths. Resolution and reinstall regressions prove denial. |

- Formal Codex Security diff scan
  `c83513cf-7de0-4552-8543-354b7aee4b4b` completed against frozen snapshot
  `codex-security-snapshot/v1:sha256:28c5cd05cb3988bc2868237ec1e595e88cafc2a6fba7e370b19efd297580b0b4`.
  All 25 changed-file inventory items and eight risk surfaces were covered. Six
  candidates were validated and routed into the ledger above; final attack-path
  policy reported zero currently exploitable findings because no production
  renderer/decoder or privileged/shared store exists. TAC advisory status was
  unknown because its connector was unavailable.
- Dynamic security evidence reproduced acceptance of a signed 16-bit PNG,
  node/capability confusion, a 16 MiB FIFO read before rejection, and revocation
  lookup fail-open behavior before the fixes. The filesystem race did not escape
  in 20,000 attempts and remains explicitly outside the same-user store's trust
  model.
- Post-fix focused evidence: `cargo clippy -p omarchygs-game-cartridge
  --all-targets -- -D warnings`, `cargo test -p omarchygs-game-cartridge
  --all-targets` (19 tests), and `scripts/test-game-cartridge.sh` all passed.
- Fresh CodeGraph inspection traced path verification through authenticated
  payload parsing, manifest/presentation/media validation, compatibility, and
  store resolution after the fixes. Its blast-radius heuristic again omitted
  several indirect integration tests, so the 19-test corpus was reconciled by
  direct review. The Phase 3.5 receipt matches pipeline
  `003d6707-f08b-4add-b612-705f5a0cc7bb` and the post-fix gated state.

## Phase 4 — Validate

- `bin/gate.sh --diff` completed after the final implementation, inspection,
  architecture, and ticket edits and printed `GATE GREEN [diff]` with matching
  worktree receipt
  `889fb055f796190ebdb1a422e8fcb26a6ee12ff14969e97b556d0b728e9bec05`.
- The 14-gate run included rustfmt, workspace Clippy with warnings denied,
  production tests and rustdoc, Compose/shell/pipeline/secret/hook/whitespace
  checks, all 19 production cartridge tests and CLI lifecycle, the seven-test
  isolated provider/QML architecture proof, 33 PostgreSQL integration tests,
  and the real migration → Rust API → visible QML smoke path.

## Phase 5 — Complete

- EARS audit:
  - REQ-001 satisfied by byte-identical repeated packing, changed-content
    identity tests, signature tampering tests, and the deterministic CLI script.
  - REQ-002 satisfied by the hostile archive/schema/media/path corpus, strict
    8-bit PNG profile, capability binding, bounded handle reads, and the formal
    security inspection.
  - REQ-003 satisfied by the SDK/protocol compatibility matrix, missing required
    capabilities, and all six typed optional fallbacks.
  - REQ-004 satisfied by machine-readable conform output and the CLI isolation
    run with unusable network/database/credential environment values.
  - REQ-005 satisfied by content-addressed `0444` storage, atomic activation and
    revocation, re-verification, idempotent revoke, and malformed revocation
    denial tests.
- OpenWiki update run `5fc51913-7de7-4df0-95cf-83ff7c0d68d1` returned
  `status: complete`. It updated Game Cartridges, quickstart, development and
  validation, and product boundaries with the implemented v1 contract, focused
  gate, same-user store limitation, graphics stages, and remaining authority
  boundary.
- Durable knowledge records capture five inspection failures, five prevention
  rules, the canonical v1 contract decision, and the same-user store decision
  in both AAR-015 and the knowledge register.
- Ticket 016 now explicitly requires a reproducible minimum-hardware
  genre/effect benchmark matrix before Core/Rich-2D profile publication.
- No EARS requirement was deferred. Ticket 016 rendering, Ticket 017 SDK and
  privileged/multi-user import containment, and Tickets 018–019 provider work
  remain explicit out-of-scope follow-ups.
- Phase 5 archived the closed ticket, spec, and notes after the matching
  OpenWiki completion receipt was issued.

## Defect and lesson ledger

| # | What happened | Root cause | Fix | Prevention |
|---|---|---|---|---|

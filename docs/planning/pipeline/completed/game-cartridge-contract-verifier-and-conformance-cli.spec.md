---
title: Game Cartridge contract, verifier, and conformance CLI
pipeline_id: 003d6707-f08b-4add-b612-705f5a0cc7bb
status: Phase 5 — Complete PASS
ticket: TICKET-015
ticket_doc: docs/planning/tickets/closed/TICKET-015-game-cartridge-contract-verifier-and-conformance-cli.md
aar: docs/planning/knowledge/aar/AAR-015-game-cartridge-contract-verifier-and-conformance-cli.md
created: 2026-08-25
---

# Game Cartridge contract, verifier, and conformance CLI — spec

## Intent

Promote the data-only Ticket 014 proof into a production-quality local package
boundary: an independently reproducible `.ogsc` artifact, strict verifier,
stable compatibility report, database-free conformance CLI, and atomic
content-addressed local store. This ticket does not render a cartridge or
enable provider traffic.

## Scope

- In: all five EARS requirements in
  [`TICKET-015`](../../tickets/open/TICKET-015-game-cartridge-contract-verifier-and-conformance-cli.md#ears-requirements),
  a production Rust library and CLI, v1 manifests/presentation/schema/media
  contracts, deterministic signing and packaging, a hostile archive corpus,
  compatibility/fallback negotiation, machine-readable provenance, local
  install/activate/revoke lifecycle, documentation, and canonical-gate
  integration.
- Out: QML rendering or preview, provider/network access, publisher onboarding,
  marketplace policy, server routes or PostgreSQL migrations, arbitrary code,
  custom shaders, archive download, changes to Constitution §10, Git delivery,
  and publication of the SDK as a supported external release.

## Acceptance criteria

The authoritative acceptance criteria are REQ-001 through REQ-005 in
[`TICKET-015`](../../tickets/open/TICKET-015-game-cartridge-contract-verifier-and-conformance-cli.md#ears-requirements).

## Locked design decisions

### Artifact and signature profile

- `.ogsc` v1 is a canonical ZIP container with **stored entries only**. Pack
  output sorts canonical ASCII paths, emits no directory entries, encryption,
  comments, compression, or ZIP64 metadata, fixes the DOS timestamp to the ZIP
  epoch, and fixes file permissions to read-only. Avoiding compression in v1
  makes identical inputs byte-reproducible and rejects decompression bombs
  rather than attempting to estimate them.
- `integrity.signed.json` contains an Ed25519 envelope over a compact canonical
  integrity index with a domain-separated message, exact publisher key ID, and
  sorted entries for every other archive byte. Every entry binds path, media
  type, byte length, and SHA-256 digest. The complete archive also receives a
  SHA-256 content address.
- Verification reads the bounded archive once, streams each bounded entry into
  retained authenticated bytes, verifies the signed index, parses only those
  retained bytes, reconstructs the canonical container, and requires exact
  byte equality. Duplicate names, alternate metadata, ordering, compression,
  and normalized-path aliases therefore cannot be accepted as a second encoding
  of the same signed release.

### V1 package contract

- Required entries are `manifest.json`, `presentation.json`, at least one
  `schemas/<name>.schema.json`, and `integrity.signed.json`. Optional entries are
  `locales/<tag>.json` and declared `assets/<name>.(png|wav)` only.
  Paths are lowercase canonical ASCII, one directory deep, and cannot contain
  absolute, dot, parent, separator-alias, control, Unicode, or platform-prefix
  forms.
- The strict manifest binds game/publisher identity, exact rules and cartridge
  versions, SDK and presentation-protocol ranges, entry screen, required
  capabilities, optional capabilities with explicit fallbacks, and exact
  schema/localization/asset inventories. Unknown fields reject.
- Presentation v1 promotes the proof's data-only screen/action boundary. It
  supports bounded terminal, grid, and status nodes only; all IDs, bindings,
  labels, dimensions, screens, nodes, and action payload fields are bounded and
  unique. Ticket 016 may add versioned host nodes without weakening this parser.
- JSON schemas use a deliberately bounded local subset of draft 2020-12.
  Remote references, executable expressions, unknown keywords, excessive
  depth/nodes, permissive object shapes, and unbounded collections/strings
  reject. Localization is a bounded string map. V1 assets are strict 8-bit PNG
  with only CRC-checked `IHDR`/`PLTE`/`IDAT`/`IEND` chunks and PCM WAV only;
  bounded parsing must confirm media, normalized RGBA decoded memory, image
  dimensions/pixels, or audio format/duration/decoded bytes against the signed
  declaration.
  WebP/Ogg remain versioned Ticket 016 additions after its decoder review.
- V1 adopts the Cartridge Core planning package ceiling: 8 MiB archive,
  32 MiB declared expanded bytes, 256 entries, and 8 MiB per entry, with tighter
  JSON, scene, schema, localization, and asset metadata limits. Stored-only v1
  means any compressed entry rejects before decompression.

### Compatibility and conformance

- A host profile supplies one SDK version, one presentation-protocol version,
  and a canonical capability set. Compatibility returns stable machine-readable
  outcomes for version-too-old/new, missing required capabilities, and optional
  fallback selection. Install rejects an incompatible result; package integrity
  remains distinguishable from host incompatibility.
- Every optional visual/audio capability declares one typed fallback (`omit`,
  `static`, `reduced_motion`, `muted`, `platform_placeholder`, or a simpler
  capability). Missing optional capabilities select that fallback rather than
  making the package unverifiable.
- The `omarchygs-cartridge conform` command receives only an archive, public
  key, expected key identity, and explicit host profile. It writes one JSON
  provenance report, performs no installation, exposes no network/database
  dependency, and reads no ambient platform credential.

### Local store

- Successful installation writes the exact verified archive to
  `<root>/blobs/sha256/<archive-digest>.ogsc` with read-only, non-executable
  permissions through a same-directory temporary file and atomic rename.
- Activation writes an allowlisted JSON record at
  `<root>/active/<game-key>.json`; revocation writes a content-addressed record
  at `<root>/revoked/<archive-digest>.json`. Both use write/sync/permission/
  rename discipline. Resolution fails closed for absent, mismatched, or revoked
  blobs. This is local filesystem state only, not the future publisher catalog.
- This store is supported only as a same-user local root. Every file read is
  bounded through a checked regular-file handle and revocation lookup errors
  fail closed. A privileged or multi-user importer requires descriptor-relative
  containment and authoritative revocation under Ticket 017.

## Planned file manifest

| Path | Purpose |
|---|---|
| `Cargo.toml`, `Cargo.lock` | Add the production cartridge crate and reviewed package/signature dependencies to the root workspace. |
| `crates/game-cartridge/Cargo.toml` | Define the production library plus `omarchygs-cartridge` CLI. |
| `crates/game-cartridge/src/lib.rs` and focused modules | Own contract types, canonical paths/JSON/archive, signing, verification, compatibility, reports, and local store. |
| `crates/game-cartridge/src/bin/omarchygs-cartridge.rs` | Provide key generation, pack, conform, install, and revoke commands with JSON output and stable exits. |
| `crates/game-cartridge/tests/` and fixtures | Prove reproducibility, hostile archive/schema/media cases, capability matrix, CLI isolation, and filesystem lifecycle. |
| `scripts/test-game-cartridge.sh` | Run the focused production crate and CLI end-to-end checks. |
| `bin/gate.sh`, `CONSTITUTION.md` | Replace the Ticket 014 spike-only gate with production cartridge conformance while retaining the spike as architecture regression evidence if both remain necessary. |
| `docs/architecture/game-cartridges.md`, README/API/OpenWiki | Publish v1 decisions and distinguish implemented local tooling from future renderer/provider work. |

## Regression map

| Requirement | Evidence |
|---|---|
| REQ-001 | Two packs from copied sources with altered mtimes/permissions produce byte-identical archives and identities; any content mutation changes the signed identity and archive digest; signature/key mismatches reject. |
| REQ-002 | A crafted ZIP/JSON/media corpus covers traversal, absolute and Unicode aliases, duplicates, directories/links, comments/extra metadata, compression/ZIP64/encryption, unknown/executable/mismatched media, undeclared files, schema keywords/depth/bounds, node/capability confusion, strict PNG depth/chunks/CRC, non-regular path input, entry/file/archive/expanded limits, and tampering. |
| REQ-003 | Table-driven SDK/protocol old/current/new cases plus required/optional capability and every fallback kind produce stable report codes and install behavior. |
| REQ-004 | CLI integration runs in an environment with unusable database/network/credential values, emits exact JSON provenance, leaves no store state, and is deterministic except for explicitly absent timestamps. |
| REQ-005 | Filesystem tests install one digest idempotently, atomically switch exact activation metadata, reject conflicting content, prove `0444`/no execute bits, revoke idempotently, fail resolution after revocation, and deny malformed or inaccessible revocation paths. |

## Phase plan

| Phase | Deliverable | Exit gate |
|---|---|---|
| 1 Plan | Active ticket/spec/notes/AAR and recalled boundaries | scope and exclusions fixed |
| 2 Design | Exact archive/schema/signature/report/store contract and blast-radius evidence | CodeGraph receipt plus testable file manifest |
| 3 Implement | Production crate, CLI, corpus, focused script, docs, gate integration | focused test loop green |
| 3.5 Inspect | Independent correctness/security/package/filesystem/CLI review | finding ledger resolved and fresh CodeGraph receipt |
| 4 Validate | Complete matrix plus canonical diff gate | matching delivery receipt |
| 5 Complete | EARS audit, OpenWiki, AAR, ticket archive | matching OpenWiki and delivery receipts |
| Delivery | Authorized commit/push only | explicit user authorization |

---
title: Public marketplace trust enrollment, rotation, and client package channel
pipeline_id: d9d78401-aa06-4134-bba0-61c5683cd5c2
status: Phase 5 — Complete PASS
ticket: TICKET-036
ticket_doc: docs/planning/tickets/closed/TICKET-036-public-marketplace-trust-enrollment-rotation-and-client-package-channel.md
aar: docs/planning/knowledge/aar/AAR-036-public-marketplace-trust-enrollment-rotation-and-client-package-channel.md
created: 2026-08-27
---

# Public marketplace trust enrollment, rotation, and client package channel — spec

## Intent

Replace the public player's manual single-marketplace-key bootstrap with an
explicit independently authenticated enrollment and rotation path that remains
truthful for historical cartridge evidence. Use the same package-pinned root
to authenticate bounded native-client package artifacts without letting a
selected community server choose trust or invoke privileged installation.

## Scope

- In:
  - a versioned offline-root-signed trust/channel manifest and deterministic
    producer/verifier tooling;
  - build-time public root/channel binding, explicit player enrollment,
    guarded synchronization, atomic per-user trust state, and key rotation;
  - active/retired/revoked marketplace-key semantics through server sync,
    current/historical acquisition, mount resolution, and rendering;
  - root-authenticated Arch package metadata, bounded download staging,
    player-visible provenance, compatibility, recovery, and documentation.
- Out:
  - any marketplace or release private key in this repository/package, a live
    production marketplace deployment, or automatic offline-root rotation;
  - automatic `pacman`/`sudo`/shell execution, privilege escalation, daemon
    self-update, arbitrary repositories, or server-selected trust channels;
  - operator-custom cartridge trust, public Provider SDK/onboarding, server
    modules/hooks, federation, or changes to cartridge execution authority.

## Acceptance criteria (EARS)

| ID | EARS requirement | Verification |
|---|---|---|
| REQ-001 | When a marketplace release operator creates a trust-channel document, the tooling shall emit one bounded canonical root-signed manifest that binds channel identity, monotonic version, validity window, a bounded marketplace keyring, and exact native package artifacts without embedding private material. | Contract unit tests, deterministic signing fixture, exact-schema corpus, and secret scan. |
| REQ-002 | When the official client package is built for a public channel, it shall bind one exact bootstrap root and canonical channel origin into package provenance; when those inputs are absent, the existing manual-key client shall remain usable without claiming public enrollment. | Package source/build/reproducibility tests and extracted-package inspection. |
| REQ-003 | When a player explicitly enrolls marketplace trust, the companion shall fetch only the package-configured channel document and shall never derive a root, origin, or trust key from selected-server discovery, catalog, acquisition, QML, or environment response data. | Companion/QML integration and hostile selected-server substitution tests. |
| REQ-004 | When the companion contacts the trust channel, it shall use one bounded HTTPS request with public-destination enforcement, no proxy, redirect, ambient credential, or transparent decompression, and shall fail closed on DNS, TLS, origin, body, timeout, or content-type violations. | Guarded transport tests with private/mixed DNS, wrong root, redirect, proxy, timeout, oversized, and malformed fixtures. |
| REQ-005 | When a channel document is received, the companion shall verify the complete root signature, canonical bytes, channel identity, validity, monotonic version, keyring semantics, and artifact inventory before atomically publishing private local trust state; any failure shall preserve the prior state. | Trust-store unit/integration tests for tamper, rollback, expiry, collision, partial write, symlink, concurrency, and restart. |
| REQ-006 | When marketplace signing keys rotate, the trust contract shall distinguish the one key eligible for new snapshots from bounded retired historical keys and terminally revoked keys, with unambiguous snapshot-version ranges and no label-only equality. | Keyring conformance matrix for overlap, gaps, duplicate bytes/IDs, downgrade, retirement, revocation, and exact fingerprints. |
| REQ-007 | When a server synchronizes a current marketplace snapshot under an enrolled keyring, it shall accept only the exact active key and next monotonic snapshot contract while retaining authenticated older evidence under its allowed historical key; manual single-key configuration shall remain an explicit compatibility mode. | Server configuration, sync, PostgreSQL rotation, replay, restart, and compatibility tests. |
| REQ-008 | When a participant acquires a current or historical session cartridge after rotation, the server and client shall authorize the acquisition envelope's exact marketplace key and snapshot version under the same enrolled keyring, reject revoked or out-of-range evidence, and still apply current lifecycle policy independently. | Current/historical acquisition API and companion tests across active, retired, revoked, range, policy, and catalog transitions. |
| REQ-009 | When local mounts exist across a trusted key rotation, cache inventory, rendering, and exact removal shall accept each mount only under its retained exact key fingerprint and the current keyring decision, without rewriting provenance or sharing trust between server profiles. | Multi-key profile/cache/render tests for coexistence, revocation, restart, exact removal, and hostile fingerprint substitution. |
| REQ-010 | When trust synchronization and cartridge operations race, results shall linearize to one complete old or new trust version, terminal revocation shall survive restart, and no acquisition or render shall observe a partially updated keyring. | Companion/store concurrency and crash-recovery tests plus server writer/acquisition ordering cases. |
| REQ-011 | When a player checks the reviewed client package channel, the companion shall expose only bounded root-authenticated artifact metadata for the running platform/version and may download one exact artifact to a private non-executable staging file whose size and SHA-256 match before publication. | Package-channel selection, download, filesystem, tamper, ambiguity, capacity, cancellation, and restart tests. |
| REQ-012 | When a package artifact is ready, the QML client shall show exact channel/version/digest/provenance and an explicit reveal-or-copy-install-command action, but shall never invoke pacman, sudo, a shell, or another privileged installer. | Production-root QML keyboard/accessibility/hostile-envelope tests and process-spawn assertions. |
| REQ-013 | When enrollment, synchronization, rotation, revocation, offline use, or package download succeeds or fails, the client shall present explicit stable states, retain social/game access where safe, and never silently weaken cartridge trust or install software. | QML/runtime state matrix, offline/retry/restart fixtures, and live clean-client smoke. |
| REQ-014 | When no public channel is packaged or a user deliberately supplies the existing absolute manual key, current launcher, companion, marketplace sync, cartridge cache, gameplay, and package installation behavior shall remain compatible and fail closed on mixed trust modes. | Existing server/runtime/QML/package suites plus manual/channel precedence and legacy fixtures. |
| REQ-015 | Before delivery, channel tooling, package fixtures, SDK/native artifacts, authored docs, and the repository shall describe the same trust/rotation/package contract and pass focused checks plus the canonical worktree-bound diff gate. | Reproducibility checks, clean-package drill, documentation review, and `bin/gate.sh --diff`. |

## Locked decisions

| # | Decision | Why |
|---|---|---|
| 1 | A selected community server never supplies the marketplace bootstrap root, channel origin, trust bundle, or package artifact authority. | Marketplace review is an independent claim and cannot be authenticated by the authority distributing a cartridge. |
| 2 | The client package binds only public bootstrap material; every private signing key remains offline/outside the repository and installed artifact. | Public enrollment must not turn a player package or source checkout into the marketplace signing authority. |
| 3 | The trust bundle is a bounded keyring, not a mutable single-key file. | Honest rotation must preserve explicitly retired historical evidence while terminal revocation still fails closed. |
| 4 | Snapshot-version ranges and exact key bytes/fingerprints bind key eligibility. | Labels and unordered key lists cannot distinguish valid historical proof from overlap, rollback, or substitution. |
| 5 | Enrollment and trust synchronization are explicit user-visible transitions with atomic local persistence. | Network availability or server selection must not silently change the trust roots that authorize local cartridge content. |
| 6 | Package artifacts are root-authenticated immutable bytes, but installation remains a separate explicit OS/package-manager action. | The same-user QML/companion boundary has no authority to invoke sudo, pacman, a shell, or privileged mutation. |
| 7 | A package without public bootstrap material remains an honest manual-key build rather than trusting a development default. | No production root/private-key lifecycle has been provisioned in this repository. |
| 8 | Current lifecycle policy remains independent from historical marketplace provenance across key rotation. | Retired proof can authenticate an old reviewed release without granting current use after suspension or revocation. |

## Linked artifacts

- Ticket: [TICKET-036](../../tickets/closed/TICKET-036-public-marketplace-trust-enrollment-rotation-and-client-package-channel.md)
- Architecture: [Game Cartridges](../../../architecture/game-cartridges.md), [system overview](../../../architecture/system-overview.md), [ADR-0003](../../../architecture/adr-0003-owner-operated-server-and-extension-boundary.md)
- Intake: none

## Phase plan

| Phase | Deliverable | Exit gate |
|---|---|---|
| 1 Plan | Ticket, spec, notes, open AAR | autonomous approved-continuation scope review |
| 2 Design | Architecture, file manifest, regression plan | actionable design plus CodeGraph receipt |
| 3 Implement | Code matching the design | focused compilation/tests and self-review |
| 3.5 Inspect | Findings ledger and fixes | verified dispositions plus fresh CodeGraph receipt |
| 4 Validate | Tests run and delivery gate green | matching worktree gate receipt |
| 5 Complete | AC audit, docs, submitted AAR, archive | no silent drops plus OpenWiki receipt |
| Delivery | Fresh gate, staged review, authorized commit/push | matching receipt and remote readback |

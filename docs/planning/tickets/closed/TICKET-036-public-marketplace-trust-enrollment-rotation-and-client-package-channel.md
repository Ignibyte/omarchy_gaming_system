---
title: TICKET-036-public-marketplace-trust-enrollment-rotation-and-client-package-channel
status: closed
ticket_number: 036
type: feature
created: 2026-08-27
closed: 2026-08-27
intake:
pipeline_spec: docs/planning/pipeline/completed/public-marketplace-trust-enrollment-rotation-and-client-package-channel.spec.md
---

# TICKET-036-public-marketplace-trust-enrollment-rotation-and-client-package-channel

## Summary

Let a clean official client explicitly enroll an independently authenticated
marketplace trust bundle, accept bounded marketplace signing-key rotation
without losing valid historical evidence, and verify reviewed native package
artifacts through the same package-pinned channel.

## Why

Cartridge installation currently depends on a manually copied single public
key. That is secure but not a public-player workflow, and replacing the key
would make old mounts and historical session evidence unavailable. The client
needs a bootstrap root and channel that do not come from the selected community
server, plus honest rotation, rollback, revocation, and package provenance.

## EARS requirements

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

## Links

- Intake: none; continuation of the ordered owner-operated ecosystem roadmap.
- Pipeline spec: [completed spec](../../pipeline/completed/public-marketplace-trust-enrollment-rotation-and-client-package-channel.spec.md)
- Architecture: [Game Cartridges](../../../architecture/game-cartridges.md), [system overview](../../../architecture/system-overview.md), [ADR-0003](../../../architecture/adr-0003-owner-operated-server-and-extension-boundary.md)

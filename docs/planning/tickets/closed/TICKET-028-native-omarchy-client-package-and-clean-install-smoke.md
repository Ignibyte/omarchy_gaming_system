---
title: TICKET-028-native-omarchy-client-package-and-clean-install-smoke
status: closed
ticket_number: 028
type: feature
created: 2026-08-26
closed: 2026-08-26
intake:
pipeline_spec: docs/planning/pipeline/completed/native-omarchy-client-package-and-clean-install-smoke.spec.md
---

# TICKET-028-native-omarchy-client-package-and-clean-install-smoke

## Summary

Ship a client-only native Arch package for Omarchy with a command launcher,
desktop entry, exact production-QML payload, reproducible builder, and an
isolated extracted-package launch smoke against the real client boundary.

## Why

The private-alpha player flow is implemented, but it still requires a source
checkout and developer commands. A clean Omarchy machine needs one reviewable
package artifact that installs and launches the client without Cargo, Docker,
or repository knowledge before invite-only testing is credible.

## EARS requirements

| ID | EARS requirement | Verification |
|---|---|---|
| REQ-001 | When the package builder runs from a valid OmarchyGS checkout, the system shall emit one native Arch client package and SHA-256 receipt without installing software or modifying the source tree. | Build twice into isolated destinations, compare package bytes, inspect status and output paths. |
| REQ-002 | When package metadata is inspected, the package shall declare the OmarchyGS client identity, workspace version, `any` architecture, and only the Qt QML runtime as its application dependency. | Extract and assert `.PKGINFO`; run `pacman -Qip`. |
| REQ-003 | When the package payload is inspected, the package shall contain the exact allowlisted production QML tree, one launcher, one desktop entry, and non-secret build provenance, while excluding tests, server/provider code, credentials, and build tools. | Compare the archive payload to the committed manifest and reject prohibited paths/types. |
| REQ-004 | When the extracted package launcher receives an application server argument, the system shall invoke the packaged production root through `qml6`, keep application arguments behind the Qt option terminator, connect to the loopback fixture, and exit successfully without a checkout, Cargo, or Docker at runtime. | Offscreen/software QML launch from an isolated extracted root plus fixture request audit. |
| REQ-005 | When a packaging source manifest is malformed, stale, duplicated, unsafe, or names a symlink/non-regular file, the build shall fail before `makepkg` receives the source. | Positive source check plus isolated hostile-manifest cases. |
| REQ-006 | When Omarchy indexes the installed desktop entry, the client shall expose a valid non-terminal Game launcher for the network client that resolves the packaged `omarchygs` command. | `desktop-file-validate` plus exact entry assertions. |
| REQ-007 | When a tester follows the installation guide, the documentation shall give package inspection, install, launch, update, and removal commands and state the remote-HTTPS, process-memory credential, client/server separation, and unsigned private-alpha artifact boundaries. | Documentation review against product and architecture commitments. |
| REQ-008 | When the canonical DIFF/FULL gate runs on Omarchy, it shall build and smoke the native client package before writing the delivery receipt. | `bin/gate.sh --diff` stage and matching worktree receipt. |

## Scope

- In:
  - client-only Arch package metadata and exact payload manifest;
  - `/usr/bin/omarchygs` launcher and desktop application entry;
  - non-installing reproducible package builder and SHA-256 output;
  - source validation, archive inspection, and extracted-package QML smoke;
  - DIFF/FULL gate integration and installation/operator documentation.
- Out:
  - packaging or deploying the Rust/PostgreSQL community server;
  - installing a package into the developer workstation during validation;
  - public package repository, update service, release signing key, or AUR publication;
  - persistent client credentials, automatic sign-in, multi-server profiles, or cartridge acquisition;
  - changes to REST, WebSocket, database, game, provider, or cartridge contracts.

## Links

- Intake: direct continuation of the private-alpha roadmap
- Pipeline spec: [completed spec](../../pipeline/completed/native-omarchy-client-package-and-clean-install-smoke.spec.md)
- Architecture: [system overview](../../../architecture/system-overview.md)

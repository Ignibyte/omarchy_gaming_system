---
title: Native Omarchy client package and clean-install smoke
pipeline_id: b991a2ec-1d25-4651-ae8f-c58b4ef211be
status: Phase 5 — Complete PASS
ticket: TICKET-028
ticket_doc: docs/planning/tickets/closed/TICKET-028-native-omarchy-client-package-and-clean-install-smoke.md
aar: docs/planning/knowledge/aar/AAR-028-native-omarchy-client-package-and-clean-install-smoke.md
created: 2026-08-26
---

# Native Omarchy client package and clean-install smoke — spec

## Intent

Turn the completed keyboard-first QML player surface into one native Omarchy
client artifact that a tester can inspect, install, launch, update, and remove
without a repository checkout or developer runtime. Prove the package itself,
not merely the source tree, launches the production root and reaches an
admitted OmarchyGS server origin.

## Scope

- In:
  - client-only Arch `makepkg` definition, exact production-QML manifest,
    launcher, desktop entry, builder, conformance smoke, docs, and gate;
  - deterministic package bytes and non-secret commit/source-digest provenance;
  - hostile source-manifest rejection and exact archive-payload inspection;
  - extracted-package offscreen launch against the bounded fixture health API.
- Out:
  - host server packaging, production TLS/deployment, package publication or
    signing infrastructure, privileged installation during tests, persistent
    credentials, multi-server profiles, cartridge download, and application
    protocol changes.

## Acceptance criteria (EARS)

| ID | EARS requirement | Verification |
|---|---|---|
| REQ-001 | When the package builder runs from a valid OmarchyGS checkout, the system shall emit one native Arch client package and SHA-256 receipt without installing software or modifying the source tree. | Two isolated builds are byte-identical; worktree/status and artifact assertions pass. |
| REQ-002 | When package metadata is inspected, the package shall declare the OmarchyGS client identity, workspace version, `any` architecture, and only the Qt QML runtime as its application dependency. | Exact `.PKGINFO` and `pacman -Qip` checks. |
| REQ-003 | When the package payload is inspected, the package shall contain the exact allowlisted production QML tree, one launcher, one desktop entry, and non-secret build provenance, while excluding tests, server/provider code, credentials, and build tools. | Exact payload comparison, file-type/mode checks, and prohibited-path scan. |
| REQ-004 | When the extracted package launcher receives an application server argument, the system shall invoke the packaged production root through `qml6`, keep application arguments behind the Qt option terminator, connect to the loopback fixture, and exit successfully without a checkout, Cargo, or Docker at runtime. | Isolated extraction plus offscreen QML/fixture smoke and request audit. |
| REQ-005 | When a packaging source manifest is malformed, stale, duplicated, unsafe, or names a symlink/non-regular file, the build shall fail before `makepkg` receives the source. | Current-tree validation and isolated missing/extra/duplicate/traversal/unsorted/unterminated/symlink fixtures. |
| REQ-006 | When Omarchy indexes the installed desktop entry, the client shall expose a valid non-terminal Game launcher for the network client that resolves the packaged `omarchygs` command. | `desktop-file-validate` and exact field checks. |
| REQ-007 | When a tester follows the installation guide, the documentation shall give package inspection, install, launch, update, and removal commands and state the remote-HTTPS, process-memory credential, client/server separation, and unsigned private-alpha artifact boundaries. | Documentation review against charter, system overview, and operator guide. |
| REQ-008 | When the canonical DIFF/FULL gate runs on Omarchy, it shall build and smoke the native client package before writing the delivery receipt. | New DIFF/FULL gate stage and matching receipt. |

## Locked decisions

| # | Decision | Why |
|---|---|---|
| 1 | Use a native Arch `.pkg.tar.zst` artifact built with `makepkg`. | Omarchy 4.0 is Arch-based and exposes `pacman`, `makepkg`, and a first-party local package-development path. |
| 2 | Package only the QML client and depend on `qt6-declarative`; do not bundle the Rust server, PostgreSQL, Cargo, Docker, or Qt libraries. | Player and operator deployment units are separate, and system Qt receives security/compatibility updates through Arch. |
| 3 | Install immutable client data under `/usr/share/omarchy-gaming-system/qml`, expose `/usr/bin/omarchygs`, and register one desktop entry. | This follows the Arch filesystem boundary and gives both command and Omarchy application-launcher entrypoints. |
| 4 | Drive package contents from an exact committed production-QML manifest with fail-closed source validation. | A broad recursive copy could silently ship tests, fixtures, future executable-adjacent files, or omit a new trusted screen. |
| 5 | Build and launch-test an extracted package without invoking privileged installation. | Validation should prove the artifact while leaving the developer's system package database untouched. |
| 6 | Keep public repository publication and package signing out of this slice, and label locally built artifacts as unsigned private-alpha output. | No release key/repository policy exists yet; inventing one inside a client packaging ticket would create a larger supply-chain system. |

## Linked artifacts

- Ticket: [TICKET-028](../../tickets/closed/TICKET-028-native-omarchy-client-package-and-clean-install-smoke.md)
- Architecture: [system overview](../../../architecture/system-overview.md)
- Intake: direct continuation of the private-alpha roadmap

## Phase plan

| Phase | Deliverable | Exit gate |
|---|---|---|
| 1 Plan | Ticket, spec, notes, open AAR | scope and EARS complete |
| 2 Design | Package/data flow, exact manifest, regression plan | CodeGraph receipt and actionable design |
| 3 Implement | Package definition, launcher, scripts, docs, gate | focused package smoke |
| 3.5 Inspect | Correctness, supply-chain, shell, QML, UX findings and fixes | final CodeGraph receipt and dispositions |
| 4 Validate | Focused tests and delivery gate green | matching gate receipt |
| 5 Complete | AC audit, OpenWiki, submitted AAR, ticket/archive | no silent drops |
| Delivery | Fresh gate, staged review, authorized commit/push | remote readback |

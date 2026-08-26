---
aar: AAR-028-native-omarchy-client-package-and-clean-install-smoke
ticket: TICKET-028
pipeline: native-omarchy-client-package-and-clean-install-smoke
status: submitted
opened: 2026-08-26
submitted: 2026-08-26
effectiveness: 5
---

# AAR-028-native-omarchy-client-package-and-clean-install-smoke

## Recalled at plan

| ID or source | How surfaced | Useful? |
|---|---|---|
| Private-alpha roadmap and product charter | First unchecked playable-value outcome after the completed QML player flow | Yes — fixes the slice to a client package and defers operator/server work. |
| `PR-omarchy-bbs-verify-the-vertical-slice-001` | Knowledge register and current QML live smoke | Yes — the extracted package must launch and reach a server rather than merely build. |
| `PR-omarchy-gaming-system-pin-executable-artifacts-before-install-001` | Knowledge register and prior tool supply-chain work | Yes — package payload and provenance need explicit inspection before any tester installs it. |
| `PR-omarchy-gaming-system-compile-production-qml-root-after-control-edits-001` | Ticket 022 production-root failure | Yes — package evidence must instantiate packaged `Main.qml`. |
| `PR-omarchy-gaming-system-own-headless-qt-test-environment-001` | Ticket 022 inherited-Wayland hang | Yes — extracted-package smoke must force offscreen/software rendering. |
| Omarchy 4.0 local package tooling and Arch package ownership | Direct inspection of installed `omarchy-dev-pkg-test`, `pacman`, `makepkg`, and `qt6-declarative` | Yes — confirms the native package format and exact runtime dependency. |

## What happened

The client now builds as a native, non-installing Arch package with an exact
37-file trusted-QML manifest, application launcher, desktop entry, and
non-secret provenance. Two builds are byte-identical; the extracted artifact
passes metadata, payload, mode, provenance, desktop, and loopback runtime
checks. Inspection made record termination explicit, and the final-snapshot
security diff scan completed with full coverage and no reportable findings.
The full 20-stage diff gate passed, OpenWiki reconciled the package lifecycle
and renumbered provider gates, and the native player package is ready for
private-alpha installation from a reviewed checkout. Public signing and package
repository publication remain deliberately separate work.

## Failures captured

| ID | Failure | Where surfaced |
|---|---|---|
| `BF-omarchy-gaming-system-qml-runtime-location-assumption-001` | The package smoke inferred `qml6` from a qmake-internal directory instead of resolving the actual runtime executable. | First focused package conformance run |
| `BF-omarchy-gaming-system-arch-buildinfo-path-nondeterminism-001` | Random build roots changed Arch `.BUILDINFO` path fields and prevented byte-identical packages. | First two-build reproducibility comparison |
| `BF-omarchy-gaming-system-line-manifest-termination-mismatch-001` | Source validation accepted a final non-newline record that downstream line consumers would omit. | Phase 3.5 correctness inspection |
| `BF-omarchy-gaming-system-terminal-scan-document-id-omission-001` | The terminal security draft omitted `findings.scanId`, so finalization stopped before writing a report. | Phase 3.5 Codex Security finalization |

## Prevention rules captured

| ID | Rule | Why |
|---|---|---|
| `PR-omarchy-gaming-system-resolve-runtime-executables-directly-001` | Resolve the actual executable promised by a runtime dependency; do not infer its location from a sibling tool's internal layout. | Package and smoke paths must match what the OS package really delivers. |
| `PR-omarchy-gaming-system-stabilize-package-build-paths-for-reproducibility-001` | When package metadata records build paths, use a private owner-checked serialized stable root or remove/normalize the path before claiming byte reproducibility. | Independent random roots can change the artifact even when payload source is identical. |
| `PR-omarchy-gaming-system-enforce-line-manifest-termination-001` | Define and test final-record termination before multiple line-oriented consumers rely on a manifest. | A validator and consumer must not disagree about whether the final record exists. |
| `PR-omarchy-gaming-system-bind-terminal-scan-document-identities-001` | Before terminal security finalization, bind manifest, findings, and coverage to one explicit scan ID and verify equality. | Terminal finalizers cannot synthesize host-owned identity fields without a workbench binding. |

## Architecture decisions

| ID | Decision | ADR |
|---|---|---|
| `AD-omarchy-gaming-system-native-client-package-boundary-001` | The Omarchy player is a client-only native Arch package containing exact platform QML and using system Qt; server packaging and signed public distribution remain separate boundaries. | `docs/architecture/system-overview.md` |

Every new ID above must also be appended to
`docs/planning/knowledge/INDEX.md`.

## Effectiveness

5/5. All eight EARS requirements have direct source, hostile-fixture,
reproducibility, archive, extracted-runtime, documentation, security-review,
and canonical-gate evidence. Inspection found and fixed the final-record
consumer mismatch before delivery. OpenWiki completed without unresolved
claims, and the package boundary remains client-only, non-installing during
validation, and explicit about its unsigned private-alpha trust limit.

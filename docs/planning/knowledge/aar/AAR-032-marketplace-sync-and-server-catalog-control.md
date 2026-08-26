---
aar: AAR-032-marketplace-sync-and-server-catalog-control
ticket: TICKET-032
pipeline: marketplace-sync-and-server-catalog-control
status: submitted
opened: 2026-08-26
submitted: 2026-08-26
effectiveness: 5
---

# AAR-032 — Marketplace synchronization and server catalog control

## Recalled at plan

| ID or source | How surfaced | Useful? |
|---|---|---|
| `AD-omarchy-gaming-system-owner-operated-server-extension-boundary-001` | ADR-0003 and Ticket 027 recall | Yes — fixes the server-curated marketplace and independent admission boundary. |
| `AD-omarchy-gaming-system-portable-cartridge-sdk-release-v1-001` | Ticket 017 secure release/import recall | Yes — provides the production exact-release, lifecycle, and secure-store seams. |
| `PR-omarchy-gaming-system-separate-publisher-marketplace-server-attestations-001` | Knowledge register search | Yes — prevents one ambiguous verification flag from conflating three authorities. |
| `PR-omarchy-gaming-system-serialize-monotonic-policy-transitions-001` | Knowledge register search | Yes — requires cross-process serialization around lifecycle version changes. |
| `PR-omarchy-gaming-system-persist-authenticated-denial-before-enforcement-001` | Knowledge register search | Yes — prevents restart or concurrency from reopening denied content. |
| `PR-omarchy-gaming-system-validate-retained-directory-authority-001` | Knowledge register search | Yes — keeps privileged imports inside the descriptor-relative store boundary. |

## What happened

Ticket 032 turned the cartridge distribution roadmap into one bounded
server-side production slice. An owner-operated server can synchronize one
canonical HTTPS marketplace pinned by an exact Ed25519 authority and explicit
TLS root, verify a bounded monotonic signed snapshot and every exact publisher
release, stage immutable inert bytes through the descriptor-relative secure
store, and publish reviewed inventory atomically to PostgreSQL. Marketplace
review never activates a game: a separate database-local administrator command
selects one exact permitted release per game with expected-state idempotency,
monotonic admission revisions, and immutable audit receipts.

Authenticated players can now read an exact no-store metadata catalog for only
the server's effective selections. Public discovery advertises that implemented
capability, while acquisition URLs, local paths, key material, executable
content, package download/cache/mount, and client launch remain outside this
ticket. The operator recovery drill proves snapshot identity, reviewed
inventory, exact selection, lifecycle, revision, and audit state survive the
real backup/restore path.

The implementation passed strict contract and secure-store suites, a separately
spawned TLS marketplace lifecycle, 52 server/PostgreSQL tests, administrator and
real CLI tests, the 44-case QML regression fixture, native packaging, provider
security and clean-clone authority proofs, recovery, admission, CodeGraph
inspection, Codex Security review, and the complete 22-stage diff gate.

## Failures captured

| ID | Failure | Where surfaced |
|---|---|---|
| `BF-omarchy-gaming-system-discovery-capability-exact-fixture-drift-001` | Adding the truthful cartridge-catalog discovery capability left one full-document PostgreSQL fixture expecting the prior exact capability list. | First complete database validation run; 51 server tests passed and one exact discovery assertion failed. |
| `BF-omarchy-gaming-system-reserved-6bone-egress-classification-gap-001` | The shared public-egress classifier admitted the IANA-reserved former-6bone `3ffe::/16` prefix despite the production public-only destination contract. | Codex Security attack-path validation with a direct production-classifier harness. |
| `BF-omarchy-gaming-system-isolated-build-tmpfs-capacity-001` | The clean-clone provider authority pilot exhausted `/tmp` while recompiling its independent source tree, even though the main filesystem had ample space. | First retained canonical diff-gate rerun, stage 20. |

## Prevention rules captured

| ID | Rule | Why |
|---|---|---|
| `PR-omarchy-gaming-system-treat-discovery-capabilities-as-exact-contract-001` | When an implemented capability is added or removed, update every exact discovery-document fixture and capability consumer in the same change, then run the real migrated discovery test. | Capability truth is a versioned public compatibility contract, not incidental list metadata. |
| `PR-omarchy-gaming-system-test-reserved-prefix-interiors-at-shared-egress-boundary-001` | Deny complete special-purpose address prefixes in the shared production egress classifier and keep representative interior addresses in its direct regression corpus. | Testing only familiar private, loopback, metadata, or prefix-edge examples can miss a routable-looking reserved allocation used by every guarded caller. |
| `PR-omarchy-gaming-system-preflight-isolated-build-storage-001` | Before a gate compiles an independent clean-clone source tree, verify that the filesystem backing its temporary target has enough headroom and remove only scoped rebuildable caches when it does not. | Free space on the repository filesystem does not prove capacity on a separate tmpfs, and an environmental red gate must remain distinguishable from a product failure. |

## Architecture decisions

| ID | Decision | ADR |
|---|---|---|
| `AD-omarchy-gaming-system-marketplace-sync-and-server-catalog-boundary-001` | One operator-pinned marketplace may authenticate bounded reviewed release metadata and supply inert exact bytes, but PostgreSQL-owned server admission remains an independent explicit audited decision; the first player boundary is authenticated metadata only and grants no client acquisition or execution authority. | `docs/architecture/game-cartridges.md` and ADR-0003 |

Every new ID above must also be appended to
`docs/planning/knowledge/INDEX.md`.

## Effectiveness

5/5. All eleven EARS requirements have direct hostile contract, guarded TLS,
secure-store, PostgreSQL transaction/race/replay/lifecycle, exact CLI/API,
backup/restore, structural inspection, security review, and canonical gate
evidence. The one confirmed security finding was repaired in the shared
classifier and its focused regression passed before broad validation. The stale
discovery fixture and temporary-filesystem capacity failure were both exposed
by required evidence rather than waived. OpenWiki run
`049738d3-ec77-4703-8858-fa61508bde6c` completed under the Ticket 032 pipeline,
reconciled the implemented server-side cartridge flow, removed duplicate
claims, and corrected stale profile-roadmap wording. Client acquisition and
operator-custom cartridges remain explicit later outcomes.

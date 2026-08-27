---
aar: AAR-036-public-marketplace-trust-enrollment-rotation-and-client-package-channel
ticket: TICKET-036
pipeline: public-marketplace-trust-enrollment-rotation-and-client-package-channel
status: submitted
opened: 2026-08-27
submitted: 2026-08-27
effectiveness: effective
---

# AAR-036-public-marketplace-trust-enrollment-rotation-and-client-package-channel

## Recalled at plan

| ID or source | How surfaced | Useful? |
|---|---|---|
| `PR-omarchy-gaming-system-authenticate-independent-claims-outside-claiming-authority-001` | Ticket 033 trust-boundary recall and current launcher inspection | Yes; neither a selected server nor its acquisition envelope may supply the marketplace bootstrap root, keyring, or channel origin. |
| `PR-omarchy-gaming-system-separate-publisher-marketplace-server-attestations-001` | Ticket 032/033 knowledge search | Yes; the new client enrollment remains a fourth independent trust decision and does not collapse publisher integrity, marketplace review, or server admission. |
| `PR-omarchy-gaming-system-persist-authenticated-denial-before-enforcement-001` | Existing secure-store lifecycle and key-revocation analysis | Yes; a terminally revoked marketplace key must survive restart before old mounts or evidence are denied. |
| `PR-omarchy-gaming-system-serialize-monotonic-policy-transitions-001` | Keyring update/store concurrency recall | Yes; trust bundle versions and key states require one atomic monotonic transition rather than last-writer-wins files. |
| `PR-omarchy-gaming-system-pin-executable-artifacts-before-install-001` | Native package-channel planning | Yes; artifact size, digest, platform, version, and signed channel provenance must be exact before a package is exposed for manual installation. |
| `PR-omarchy-gaming-system-derive-digests-with-verifier-encoding-001` | Package reproducibility knowledge search | Yes; producer and verifier must share canonical byte and digest-record encodings. |
| `PR-omarchy-gaming-system-stabilize-package-build-paths-for-reproducibility-001` | Ticket 028 native-package AAR | Yes; channel/bootstrap inputs must join deterministic package provenance without reintroducing absolute build-path drift. |
| `PR-omarchy-gaming-system-prove-native-linking-in-package-environment-001` | Ticket 033 native package failure recall | Yes; new trust/channel runtime dependencies must be rebuilt inside the actual Arch package environment. |
| `PR-omarchy-gaming-system-separate-historical-provenance-from-current-use-policy-001` | Ticket 035 AAR | Yes; a retired key may authenticate bounded old review evidence while current lifecycle policy independently denies or allows use. |
| `PR-omarchy-gaming-system-align-producer-consumer-limits-and-uniqueness-001` | Ticket 035 AAR | Yes; key/artifact counts, ranges, identities, and uniqueness must agree across signer, server, companion, QML, and fixtures. |
| Tickets 032–035 completed specs, notes, and AARs | Nearest marketplace-to-player verticals | Yes; together they expose every current singleton-key consumer, exact retained evidence, mount fingerprint, package proof, and historical-use boundary. |
| Game Cartridge architecture, client installation, owner-operator guide, ADR-0003, and OpenWiki cartridge/runtime pages | Required architecture/OpenWiki recall | Yes; they require independent trust bootstrap, inert clients, explicit user transitions, no privileged installer, and honest vetted/custom provenance. |

## What happened

Ticket 036 replaced the public player's manual single-key dead end with an
explicit independently authenticated trust lifecycle while preserving manual
and no-key compatibility. A new non-SDK crate defines an offline-root-signed
channel containing package-pinned freshness floors, a bounded
active/retired/revoked marketplace keyring, exact current snapshot authority,
and immutable native-package metadata. The same contract now drives server
synchronization/distribution, a descriptor-bound client trust store, multi-key
mount/render decisions, acquisition v2's independent historical-evidence and
current-policy signatures, and a QML enrollment/package-staging surface that
never invokes an installer.

Repeated security inspection materially changed the result. It found an
unsigned current-policy version in acquisition v2, process-local revocation
state, a package-bootstrap pathname race, stale active-snapshot authorization,
retired-key render authority, missing durable server trust, equal-version
policy mutation, stale live server runtimes, first-enrollment replay, an
incorrect historical migration backfill, and a package-floor transition-history
gap. Each confirmed path was fixed and received a focused regression. The final
sealed scan `56922d30-0cad-4d75-a677-12e1219e3292` covered 35 authoritative
files at snapshot
`codex-security-snapshot/v1:sha256:dfeb1edf0c42017b814bb8e947c47a264a06fba99232b4c01a41eeec056bf91b`
and reported no findings.

Focused Rust, PostgreSQL, migration-upgrade, guarded TLS, QML, and native
package tests passed, followed by a green fast gate. OpenWiki run
`c6c9b71f-32e0-4eb1-a901-2c511ba2e626` completed after updating the cartridge,
runtime, validation, product-boundary, and quickstart pages; it retained
explicit pre-existing evidence-debt warnings on those pages. Durable API,
architecture, client-installation, owner-operator, README, and roadmap records
now distinguish the implemented trust/package protocol from future hosted
marketplace operations, root rotation, and privileged installation.

The first full diff gate then exposed one stale provider-pilot fixture: it
directly changed signed lifecycle policy without advancing the policy snapshot
provenance required by migration 0023. The pilot now uses the shared monotonic
lifecycle-publication helper, its complete suspend/replay/restart drill passes,
and the corrected `bin/gate.sh --diff` run passed all 23 labeled stages with
receipt `c788b1e6db9529538f399b10768c099e9c1c2f2f9c5de46b54fd3c1ed6aa0c3a`.

## Failures captured

| ID | Failure | Where surfaced |
|---|---|---|
| `BF-omarchy-gaming-system-unsigned-current-policy-snapshot-001` | Acquisition v2 originally carried an unsigned policy snapshot version beside authentic historical policy bytes, so a selected server could replay retired-key lifecycle state. | First sealed security diff scan |
| `BF-omarchy-gaming-system-process-local-trust-revocation-cache-001` | One live companion retained instance-local trust after another process durably rotated or revoked a key. | First sealed security diff scan and two-store regression |
| `BF-omarchy-gaming-system-package-bootstrap-path-toctou-001` | The native builder verified an external bootstrap path but later hashed and packaged the still-mutable caller path. | First sealed security diff scan and concurrent build fixture |
| `BF-omarchy-gaming-system-stale-live-server-trust-runtime-001` | A server runtime kept startup trust after another administrator process persisted a newer trust-only revocation. | Fresh architecture inspection and PostgreSQL rotation test |
| `BF-omarchy-gaming-system-fresh-enrollment-trust-replay-001` | A first-run or cache-cleared client knew the offline root and channel but no minimum acceptable bundle/snapshot, so an older still-valid root-signed bundle could revive a later-revoked key. | Exact-snapshot security discovery pass |
| `BF-omarchy-gaming-system-historical-migration-singleton-backfill-001` | Migration 0023 initially assigned every historical release the singleton's current snapshot version instead of the release's own last-seen version, making real retained history fail upgrade constraints. | Exact-snapshot security discovery pass and scratch-schema upgrade test |
| `BF-omarchy-gaming-system-trust-floor-hidden-transition-history-001` | Treating a persisted below-package-floor bundle as absent for both use and update allowed a newer individually valid bundle to overwrite authenticated terminal key history. | Final client trust discovery pass and floor-advance regression |

## Prevention rules captured

| ID | Rule | Why |
|---|---|---|
| `PR-omarchy-gaming-system-bind-fresh-enrollment-to-package-floors-001` | Package an authenticated minimum trust-bundle and current-snapshot version with every offline root/channel bootstrap. | A root alone authenticates old signatures but cannot tell a client with no local history which signed states are already obsolete. |
| `PR-omarchy-gaming-system-preserve-ineligible-trust-as-transition-evidence-001` | Separate whether persisted trust is eligible for current use from whether its authenticated history must constrain the next update. | Raising an eligibility floor must not erase terminal rotation/revocation facts and turn package upgrade into rollback permission. |
| `PR-omarchy-gaming-system-backfill-history-from-row-local-provenance-001` | Backfill new historical provenance columns from each row's retained historical identity, never from a mutable current singleton. | Current global state can be newer than the authentic evidence that created an older retained row. |
| `PR-omarchy-gaming-system-reconcile-persisted-trust-before-effects-001` | Reconcile the shared durable trust authority in the same snapshot or locked read used by every security-sensitive effect. | Startup or process-local trust becomes stale when another authorized process rotates or revokes authority. |
| `PR-omarchy-gaming-system-snapshot-mutable-build-inputs-before-verification-001` | Copy a caller-owned mutable build input once into private build-owned storage, then verify, hash, and package only that snapshot. | Separate reads of an external path create a verification-to-use substitution window even when every individual read is bounded. |
| `PR-omarchy-gaming-system-bind-current-policy-to-signed-current-snapshot-001` | Authenticate current-use lifecycle policy through an exact signed current snapshot and active key while validating historical provenance independently. | A genuine retired-key policy or unsigned version hint must not regain current authorization merely because its historical signature remains valid. |

## Architecture decisions

| ID | Decision | ADR |
|---|---|---|
| `AD-omarchy-gaming-system-offline-root-marketplace-trust-and-package-channel-001` | Public player trust is an explicit package-pinned offline-root channel with bounded monotonic marketplace-key history and native-package metadata; selected servers cannot choose it, historical and current policy keys remain separate, and the client may stage but never install privileged artifacts. | `../../architecture/game-cartridges.md`; `../../architecture/system-overview.md`; `../../architecture/adr-0003-owner-operated-server-and-extension-boundary.md` |

Every new ID above must also be appended to
`docs/planning/knowledge/INDEX.md`.

## Effectiveness

Effective. Recalled independent-attestation, authenticated-denial,
monotonic-transition, historical-policy, deterministic-package, and
producer/consumer-bound rules prevented the design from collapsing into a
server-supplied mutable key or self-updater. Independent security passes still
found seven durable classes of replay, stale-authority, provenance, and TOCTOU
failure that local happy-path tests did not reveal; focused regressions and the
final clean scan demonstrate that the inspection loop improved the shipped
boundary. The result permits public enrollment and honest marketplace-key
rotation without rewriting old cartridge provenance or granting QML, the
companion, a selected server, or a cartridge package-manager authority.

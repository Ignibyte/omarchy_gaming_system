---
aar: AAR-037-static-marketplace-publication-offline-root-handoff-and-mirror-operations
ticket: TICKET-037
pipeline: static-marketplace-publication-offline-root-handoff-and-mirror-operations
status: submitted
opened: 2026-08-27
submitted: 2026-08-27
effectiveness: effective
---

# AAR-037-static-marketplace-publication-offline-root-handoff-and-mirror-operations

## Recalled at plan

| ID or source | How surfaced | Useful? |
|---|---|---|
| `PR-omarchy-gaming-system-separate-publisher-marketplace-server-attestations-001` | Knowledge search and current marketplace architecture | Yes; publication cannot collapse publisher integrity, marketplace review, hosting, server admission, or client installation into one generic authority. |
| `PR-omarchy-gaming-system-pin-executable-artifacts-before-install-001` | Native package-channel recall | Yes; every staged native package needs exact signed size/digest/platform/version provenance before publication. |
| `PR-omarchy-gaming-system-snapshot-mutable-build-inputs-before-verification-001` | Ticket 036 package TOCTOU finding | Yes; publication must snapshot caller-owned inputs once and verify/use only owned bytes. |
| `PR-omarchy-gaming-system-bind-current-policy-to-signed-current-snapshot-001` | Ticket 036 acquisition finding | Yes; hosted output must bind current policy to the exact signed current snapshot and active key. |
| `PR-omarchy-gaming-system-bind-fresh-enrollment-to-package-floors-001` | Ticket 036 first-enrollment replay finding | Yes; each packaged/bootstrap generation must carry the minimum bundle and snapshot floors established by publication. |
| `PR-omarchy-gaming-system-preserve-ineligible-trust-as-transition-evidence-001` | Ticket 036 floor-advance finding | Yes; offline signing/finalization must validate continuity even when an older bundle is no longer eligible for use. |
| `PR-omarchy-gaming-system-persist-authenticated-denial-before-enforcement-001` | Existing lifecycle/revocation knowledge | Yes; a compromise drill must publish the higher authenticated revocation before relying on denial. |
| `PR-omarchy-gaming-system-serialize-monotonic-policy-transitions-001` | Trust and publication concurrency recall | Yes; snapshot, bundle, lifecycle, activation, and rollback state must move monotonically under one serialized boundary. |
| Tickets 032–036 completed specs/notes/AARs | Nearest marketplace-to-player pipelines | Yes; they define the exact hosted paths, signatures, rotation semantics, package metadata, consumer bounds, and missing producer operations. |
| Game Cartridge/system architecture plus OpenWiki quickstart, cartridge, runtime, and validation pages | Required durable context | Yes; they require inert signed content, independent trust, no privileged client installer, and evidence-backed operations. |

## What happened

Ticket 037 added `omarchygs-marketplace-publisher`, a non-SDK Rust library and
CLI that composes existing Game Cartridge release, catalog snapshot,
offline-root trust-channel, native package, and guarded transport contracts.
The workflow is explicitly split into online `prepare`, network-less
`offline-sign`, online `finalize`, locked monotonic `activate`, exact local
`verify`, and one-or-more-mirror `probe` commands.

Preparation verifies the supported SDK and each exact publisher release,
signs catalog lifecycle policy and a canonical snapshot, snapshots bounded
package inputs, and creates a public request that binds the complete prepared
inventory and previous trust transition. The offline command independently
revalidates the root, transition, keyring, package inventory, validity, and
snapshot ownership before returning one request-bound root signature. The
finalizer re-verifies the entire chain and creates a private immutable static
tree selected by one atomic relative `current` link under a cross-process
lock. Verification rejects extra, missing, linked, wrong-mode, oversized,
stale, rollback, or digest-divergent state. Guarded probes authenticate both
static namespaces and every artifact, enforce caller-held freshness/digest
floors, and require identical publication identity across mirrors.

Seven end-to-end integration tests plus two contract unit tests cover
deterministic duplicate builds, exact trees, permission/link tamper,
concurrent finalization, key and response substitution, a real
`bwrap --unshare-net` offline ceremony, two TLS mirrors, rotation/revocation,
advancing package floors, and rollback denial. The new drill is canonical gate
stage 15b. The full diff gate passed every Rust, PostgreSQL, QML, package,
provider, recovery, and private-alpha stage.

The first sealed security review found one path-based permission race in the
new file writers. Permissions are now applied through already-open file
descriptors; new directories request a restrictive creation mode and are
reopened with `O_DIRECTORY | O_NOFOLLOW` before exact descriptor-bound mode
application. The post-remediation exact-diff scan reviewed all nine executable
surfaces and reported zero findings with no deferred work.

OpenWiki refreshed the prior claim anchors, added the publication, custody,
mirror, authority, and validation facts, and completed with a matching Phase 5
receipt. Product and roadmap documents mark the deterministic local tooling and
drills complete while keeping real domains, object hosting/CDN behavior,
production root custody, staffing, monitoring, malware review, and incident
coordination explicitly external and incomplete.

## Failures captured

| ID | Failure | Where surfaced |
|---|---|---|
| `BF-omarchy-gaming-system-offline-response-path-chmod-race-001` | A new response or copied public file was safely created and written through one descriptor, but its final mode was changed by resolving the caller-visible pathname again; a local process controlling a shared parent could replace the entry with a symlink and redirect `chmod` to another custodian-owned file. | First sealed Codex Security diff scan `dde7506a-580e-45bd-a78e-40483fdc67bd` |

## Prevention rules captured

| ID | Rule | Why |
|---|---|---|
| `PR-omarchy-gaming-system-bind-permissions-to-opened-file-descriptors-001` | After securely creating or opening a file or directory, apply security-sensitive ownership or mode changes through that already-bound descriptor; do not re-resolve an attacker-visible pathname. | Create-new and no-follow protect only the open operation. A later path-based metadata mutation recreates a substitution race at the custody boundary. |

## Architecture decisions

| ID | Decision | ADR |
|---|---|---|
| `AD-omarchy-gaming-system-static-marketplace-publication-and-offline-root-handoff-001` | Marketplace distribution uses deterministic immutable static files and separated publisher, catalog, offline-root, hosting, server-admission, and client-installation authorities; routine publication cannot read the root key, mirrors create no authority, and real production custody/hosting remains an external rollout gate. | `../../architecture/adr-0003-owner-operated-server-and-extension-boundary.md`; `../../architecture/system-overview.md`; `../../operators/marketplace-publication.md` |

Every new ID above must also be appended to
`docs/planning/knowledge/INDEX.md`.

## Effectiveness

Effective. Recalled authority separation, immutable input snapshotting,
package floors, current-policy binding, transition-history retention,
authenticated denial, and serialized monotonic transitions prevented a mutable
marketplace service, online root custody, client mirror fallback, and automatic
evidence deletion from entering the design. The independent inspection still
found a descriptor/path identity gap that focused functional tests did not;
the fix generalized the descriptor-bound rule to files and directories, and a
fresh complete security scan plus the canonical gate verified the result. The
delivered slice makes the existing consumer contracts publishable and
operable without falsely claiming that local fixtures prove production hosting
or root custody.

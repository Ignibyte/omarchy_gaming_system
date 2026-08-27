---
aar: AAR-033-player-cartridge-acquisition-cache-and-mount-lifecycle
ticket: TICKET-033
pipeline: player-cartridge-acquisition-cache-and-mount-lifecycle
status: submitted
opened: 2026-08-26
submitted: 2026-08-26
effectiveness: effective
---

# AAR-033-player-cartridge-acquisition-cache-and-mount-lifecycle

## Recalled at plan

| ID or source | How surfaced | Useful? |
|---|---|---|
| `PR-omarchy-gaming-system-separate-publisher-marketplace-server-attestations-001` | Knowledge-register search plus Ticket 032 notes | Yes; it exposed the need for a fourth, client-controlled marketplace trust decision rather than a server-supplied verification key. |
| `PR-omarchy-gaming-system-validate-retained-directory-authority-001` | Knowledge-register search plus secure-store inspection | Yes; cache and mount operations were built around retained descriptor authority and no-follow validation. |
| `PR-omarchy-gaming-system-persist-authenticated-denial-before-enforcement-001` | Knowledge-register search plus marketplace lifecycle inspection | Yes; acquisition reuses the monotonic lifecycle store and never replaces denial evidence with a profile mount. |
| Ticket 032 completed spec/notes | Nearest completed pipeline | Yes; it identified the exact retained artifacts and the missing signed snapshot/key evidence needed for client verification. |
| `docs/architecture/game-cartridges.md` and ADR-0003 | Required architecture recall | Yes; they kept distribution, provider authority, trusted rendering, and future module work separate. |
| OpenWiki quickstart, product-boundaries, game-cartridges, and development pages | Required just-in-time repository context | Yes; stale metadata-only/package/gate descriptions became the Phase 5 reconciliation map. |

## What happened

Ticket 033 delivered the first complete player-side cartridge distribution
boundary. The server retains and serves one exact admitted immutable release;
the native client companion independently authenticates marketplace,
publisher, lifecycle, compatibility, digest, and selected-server claims before
writing private cached content and a server-profile mount. The QML shell can
browse, install, update, remove, and report mount state without receiving
filesystem authority or making a mounted cartridge executable.

Inspection found and fixed a retained cross-process lock, a real Arch native
link incompatibility, a server-controlled marketplace trust root, and a
catalog-only compatibility regression. The final 15-stage fast gate and
22-stage diff gate passed, including 33 shared cartridge tests, 7 client-runtime
tests, 53 PostgreSQL server tests, 46 QML fixture tests, two byte-identical
native packages, provider proofs, recovery, and private-alpha admission.

## Failures captured

| ID | Failure | Where surfaced |
|---|---|---|
| `BF-omarchy-gaming-system-companion-profile-lock-retention-001` | A completed mount operation retained its cross-process `flock`, so a second cache handle could block indefinitely. | Focused client-cache concurrency review and regression test |
| `BF-omarchy-gaming-system-arch-native-lto-link-incompatibility-001` | Makepkg's global GCC LTO flags produced `ring` native objects that Rust `lld` could not link. | First real x86_64 Arch package build |
| `BF-omarchy-gaming-system-server-supplied-marketplace-trust-anchor-001` | The verifier initially authenticated marketplace-vetted provenance using the public key supplied by the same selected server. | Codex Security scan `777bdabd-7634-488c-8585-e66b3674fad9` |
| `BF-omarchy-gaming-system-optional-acquisition-capability-hid-catalog-001` | Requiring the optional acquisition capability for cartridge authority hid the base catalog on metadata-only servers. | Independent fix review and QML compatibility inspection |
| `BF-omarchy-gaming-system-focused-tests-missed-workspace-clippy-001` | Focused compilation and tests passed while the canonical warning-denied workspace Clippy check still rejected a needless borrow. | First canonical fast gate |
| `BF-omarchy-gaming-system-completed-spec-status-enum-drift-001` | The first post-archive delivery gate rejected a completed spec whose status added prose after the exact accepted lifecycle value. | Canonical diff gate pipeline-structure stage |

## Prevention rules captured

| ID | Rule | Why |
|---|---|---|
| `PR-omarchy-gaming-system-release-retained-synchronization-locks-001` | Every retained synchronization lock needs explicit release evidence plus a second-handle progress test. | Mutual-exclusion assertions within one handle do not prove that later processes can make progress. |
| `PR-omarchy-gaming-system-prove-native-linking-in-package-environment-001` | Link native dependencies inside the real package toolchain and explicitly disposition incompatible global compiler flags. | Cargo-only tests do not reproduce makepkg's C/C++ flag injection or final linker combination. |
| `PR-omarchy-gaming-system-authenticate-independent-claims-outside-claiming-authority-001` | Authenticate an independent provenance claim with a trust root controlled outside the authority making the claim, then preserve that binding at rest. | A self-consistent signed envelope is not independent when its verifier key comes from the same untrusted envelope. |
| `PR-omarchy-gaming-system-negotiate-read-and-mutation-capabilities-separately-001` | Model base read surfaces and optional mutation/distribution capabilities separately and test every supported capability subset. | Optional write capability must not erase compatible read-only behavior. |
| `PR-omarchy-gaming-system-run-warning-denied-workspace-clippy-before-canonical-gate-001` | Run warning-denied workspace Clippy after final focused edits and before treating canonical validation as a formality. | Focused tests and formatting do not cover every workspace lint configuration. |
| `PR-omarchy-gaming-system-use-exact-pipeline-status-enum-001` | Treat pipeline frontmatter status as a closed enum and run the owning structure checker after lifecycle transitions. | Descriptive suffixes are documentation drift when automation consumes an exact state value. |

## Architecture decisions

| ID | Decision | ADR |
|---|---|---|
| `AD-omarchy-gaming-system-client-controlled-marketplace-trust-and-profile-mounts-001` | The client loads one complete marketplace public key independently, verifies exact server distribution through a same-user native companion, and records only private server-profile mounts into inert cached content; mounting is not gameplay launch. | `../../architecture/game-cartridges.md`; `../../architecture/adr-0003-owner-operated-server-and-extension-boundary.md` |

Every new ID above must also be appended to
`docs/planning/knowledge/INDEX.md`.

## Effectiveness

Effective. Recall materially shaped the implementation, every confirmed defect
received a focused regression, the independent security finding was fixed and
reviewed, CodeGraph inspection and the canonical gate matched the final gated
implementation, and the OpenWiki lifecycle completed. OpenWiki preserved the
five touched Claims sidecars because those pages still contain pre-existing
unresolved evidence debt; this warning was recorded rather than misreported as
full claim verification. The next slice can start from a truthful boundary:
mounts exist, but session identity, trusted render-plan preparation, and launch
remain deliberately unimplemented.

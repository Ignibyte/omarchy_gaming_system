---
aar: AAR-017-separate-repository-sdk-and-first-party-cartridge
ticket: TICKET-017
pipeline: separate-repository-sdk-and-first-party-cartridge
status: submitted
opened: 2026-08-25
submitted: 2026-08-25
effectiveness: 5
---

# AAR-017-separate-repository-sdk-and-first-party-cartridge

## Recalled at plan

| ID or source | How surfaced | Useful? |
|---|---|---|
| `AD-omarchy-gaming-system-game-cartridge-provider-boundary-001` | Knowledge register, ADR-0002, and OpenWiki | Yes — the SDK exports inert protocols and tooling without authorizing provider execution or remote authority. |
| `AD-omarchy-gaming-system-canonical-game-cartridge-v1-001` | Ticket 015 AAR and production verifier | Yes — a release attestation wraps the exact canonical archive; it does not replace package verification. |
| `AD-omarchy-gaming-system-trusted-cartridge-renderer-v1-001` | Ticket 016 AAR and OpenWiki | Yes — an external game targets fixed Core/Rich-2D data capabilities, never publisher QML. |
| `PR-omarchy-gaming-system-require-descriptor-relative-privileged-store-001` | Ticket 015 AAR and knowledge register | Yes — privileged/cross-principal import needs a new descriptor-anchored API and adversarial evidence. |
| `PR-omarchy-gaming-system-parse-the-bytes-that-were-authenticated-001` | Ticket 014 AAR | Yes — attestation, conformance, and imported archive verification must use the exact authenticated buffers. |
| `BUL-001-initial-push-pending` | Bulletin preflight | Yes — all release/repository evidence remains local and no Git delivery is authorized. |

## What happened

Ticket 017 turned the local cartridge contract into a portable release surface
without exposing the platform workspace as an SDK. The production CLI now
exports and self-verifies a deterministic, read-only, language-neutral v1 SDK
whose lock pins schemas, tools, presentation protocol, compatibility, and
retirement behavior. A source-only first-party Door Legends fixture is copied
into a fresh Git repository, cloned twice, and built using only copied public
CLI binaries, the exported SDK, and an explicit publisher key. Both clones
produce byte-identical cartridge, conformance, and signed release files.

The release attestation binds the source revision, builder identity and binary
digest, SDK identity, publisher/key, game/rules/cartridge versions, exact
archive and signed-content identities, and conformance digest. OmarchyGS
re-verifies all three release files instead of trusting the attestation as a
substitute for the cartridge verifier. A separate catalog-authority signature
binds the exact release to active, deprecated, suspended, revoked, or retired
policy with explicit new-launch and active-session decisions.

The Linux secure store keeps no-follow descriptors for its root and fixed
children and performs all descendant I/O relative to them. Security inspection
found four low-severity weaknesses: retained directories lacked owner/mode
validation, concurrent signed policy transitions could roll back, a denied
policy was not persisted, and the broad package PNG envelope could stall the
current software renderer. All four were fixed. The store now validates the
effective UID and group/other write bits, serializes policy transitions beneath
an exclusive descriptor-relative lock, and persists the highest authenticated
policy before enforcement. Core and Rich-2D now enforce stricter per-raster and
decoded-scene ceilings before node or asset publication; trusted QML requests a
bounded asynchronous decode.

The final CodeGraph inspection found no bypass from the server, database,
credentials, or gameplay authority into these local tooling paths. The
post-remediation Codex Security scan
`8be3cad5-e1c8-48b0-aab9-f086055cd4bc` covered all 41 frozen worktree items and
reported zero findings. OpenWiki run
`064b68f2-1fc2-471e-a574-d39adc201974` completed with no stale or unresolved
claims. The canonical 16-gate validation passed with all 33 PostgreSQL tests,
the live PostgreSQL → Rust API → QML smoke, and the clean-room SDK/release/import
path. No repository, SDK, or release publication occurred.

## Failures captured

| ID | Failure | Where surfaced |
|---|---|---|
| `BF-omarchy-gaming-system-store-directory-authority-gap-001` | Descriptor-relative containment still accepted a retained root or child owned by another user or writable by group/other, allowing a cooperating writer to erase cached policy. | Phase 3.5 Codex Security diff scan and disposable mode-0777 policy-cache reproduction. |
| `BF-omarchy-gaming-system-policy-cache-rollback-race-001` | Policy comparison and atomic replacement were not one serialized transition, so concurrent valid v2/v3 writers could leave v2 authoritative. | Phase 3.5 concurrency analysis and repeated 64-trial disposable reproduction. |
| `BF-omarchy-gaming-system-denied-policy-not-persisted-001` | Import applied new-launch denial before caching the authenticated policy, allowing an older active policy after restart. | Phase 3.5 lifecycle source-to-sink review and direct revoked-v2/active-v1 reproduction. |
| `BF-omarchy-gaming-system-render-raster-availability-gap-001` | The package accepted a compact 4,096×4,096 signed PNG whose 64 MiB decoded image exceeded the practical current software-rendering envelope. | Phase 3.5 Codex Security availability test through the production preview and QML path. |

## Prevention rules captured

| ID | Rule | Why |
|---|---|---|
| `PR-omarchy-gaming-system-validate-retained-directory-authority-001` | Validate type, expected owner, and group/other write permissions on every retained directory descriptor before treating it as a security boundary. | No-follow path containment does not prevent another authorized filesystem writer from changing security state. |
| `PR-omarchy-gaming-system-serialize-monotonic-policy-transitions-001` | Hold one cross-process lock across the complete authenticated read, compare, and replace of monotonic policy state, then re-read beneath that lock. | Individually atomic publication does not make a compound version transition monotonic. |
| `PR-omarchy-gaming-system-persist-authenticated-denial-before-enforcement-001` | Persist the highest authenticated policy before applying its allow or deny decision. | A denied transition is itself security state and must survive restart and reject older signed policy. |
| `PR-omarchy-gaming-system-charge-decoded-media-at-render-admission-001` | Charge per-instance decoded-media work against the selected render profile before publishing its node or asset, and exercise the maximum accepted decoder path. | A valid signed package can remain too expensive for the currently selected trusted runtime profile. |

## Architecture decisions

| ID | Decision | ADR |
|---|---|---|
| `AD-omarchy-gaming-system-portable-cartridge-sdk-release-v1-001` | Adopt a deterministic language-neutral SDK export, publisher-signed reproducible release, distinct platform-signed lifecycle policy, and Linux descriptor-relative secure importer as the v1 local portability boundary; keep server catalog ingestion, main-client launch, public distribution, and remote gameplay authority outside it. | `../../../architecture/game-cartridges.md`; `../../../../openwiki/game-cartridges.md` |

Every new ID above must also be appended to
`docs/planning/knowledge/INDEX.md`.

## Effectiveness

5/5. All five EARS requirements passed through public production interfaces,
two clean Git clones, exact release comparison, release/policy tamper matrices,
descriptor-root and fixed-child attacks, restart and concurrency regressions,
real 2,048-pixel rendering, pre-plan 4,096-pixel rejection, a clean post-fix
security scan, completed OpenWiki claims, and the full database/API/QML gate.
The result gives first-party and eventual external game repositories a genuine
ROM-like frontend release path while preserving compiled server authority and
making the remaining same-UID and future public-catalog limits explicit.

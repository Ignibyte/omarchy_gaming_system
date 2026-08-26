---
aar: AAR-027-owner-operated-servers-cartridge-distribution-and-extension-roadmap
ticket: TICKET-027
pipeline: owner-operated-servers-cartridge-distribution-and-extension-roadmap
status: submitted
opened: 2026-08-26
submitted: 2026-08-26
effectiveness: 5
---

# AAR-027-owner-operated-servers-cartridge-distribution-and-extension-roadmap

## Recalled at plan

| ID or source | How surfaced | Useful? |
|---|---|---|
| `AD-omarchy-gaming-system-game-cartridge-provider-boundary-001` | Cartridge/provider architecture recall | Yes — keeps frontend presentation, platform authority, and backend rules as separate release/trust domains. |
| `AD-omarchy-gaming-system-canonical-game-cartridge-v1-001` | Cartridge verifier and store recall | Yes — supplies the inert signed package and content-addressed distribution basis. |
| `AD-omarchy-gaming-system-portable-cartridge-sdk-release-v1-001` | Separate-repository SDK recall | Yes — shows publisher portability and catalog lifecycle already exist below the player marketplace. |
| `AD-omarchy-gaming-system-remote-provider-security-foundation-001` | Provider foundation recall | Yes — makes the OmarchyGS server the broker and keeps reusable player authority out of game backends. |
| `AD-omarchy-gaming-system-first-party-remote-authority-pilot-001` | Door Legends pilot recall | Yes — proves a server-agnostic provider protocol while retaining a deliberately narrow authorization. |

## What happened

Ticket 027 converted the user's server-ownership and game-distribution idea into
an accepted, separately gated architecture direction. The resulting model makes
an owner-operated standard OmarchyGS deployment the community trust domain;
keeps the marketplace, server admission, and client verification as distinct
authorities; defines the Game Cartridge as frontend-only inert data rendered by
trusted QML; and moves portable backend integration into a future public
Provider SDK. It also creates an explicitly labeled operator-custom trust path
and separates game providers from a future general module base and typed hook
system whose executable isolation still requires a dedicated spike.

Inspection corrected two material documentation problems: foundation pages had
not caught up with the activated Door Legends pilot, and early marketplace
wording conflated three different attestations. Validation preserved and
diagnosed one boundary-sensitive pre-existing MFA test failure before a fresh
18-stage gate passed. The first OpenWiki completion exposed nine stale claim
references; a recovery lifecycle reconciled all of them and finished cleanly.
No API, schema, runtime, package, QML behavior, marketplace service, custom-code
loader, or module system was implemented by this documentation slice.

## Failures captured

| ID | Failure | Where surfaced |
|---|---|---|
| `BF-omarchy-gaming-system-provider-activation-documentation-drift-001` | Current-state architecture summaries still described the provider foundation as dormant or the catalog as compiled-only after the optional Door Legends runtime was activated. | Phase 3.5 direct and CodeGraph-backed consistency review |
| `BF-omarchy-gaming-system-cartridge-distribution-trust-conflation-001` | Initial marketplace flow wording collapsed publisher-byte integrity, marketplace review, and server-local admission into a single verification concept. | Phase 3.5 supply-chain/trust review |

## Prevention rules captured

| ID | Rule | Why |
|---|---|---|
| `PR-omarchy-gaming-system-reconcile-foundation-docs-when-activated-001` | When a dormant foundation becomes an executable product path, reconcile every foundation architecture/current-state summary in the same delivery. | A correct implementation can still leave future work unsafe if durable documentation describes the old authority topology. |
| `PR-omarchy-gaming-system-separate-publisher-marketplace-server-attestations-001` | Model publisher integrity, marketplace review, and server admission as separate attestations with independent issuers, meanings, and absence states. | A generic verified flag can misrepresent provenance, support, and local authorization—especially for operator-custom content. |

## Architecture decisions

| ID | Decision | ADR |
|---|---|---|
| `AD-omarchy-gaming-system-owner-operated-server-extension-boundary-001` | Owner-operated servers are independent community trust domains; cartridges remain inert frontend data; provider backends and general server modules are separate extension families; marketplace and operator-custom paths preserve honest provenance and official-client containment. | `../../../architecture/adr-0003-owner-operated-server-and-extension-boundary.md` |

Every new ID above must also be appended to
`docs/planning/knowledge/INDEX.md`.

## Effectiveness

5/5. All six requirements are represented in product, architecture, operator,
roadmap, and generated knowledge surfaces; inspection found and corrected two
material ambiguities; OpenWiki finished without evidence debt; and the full
runtime gate remained green despite this being a documentation-only change.
Future marketplace, client acquisition, Provider SDK, local trust, and module
implementation remain clearly ordered and separately authorized.

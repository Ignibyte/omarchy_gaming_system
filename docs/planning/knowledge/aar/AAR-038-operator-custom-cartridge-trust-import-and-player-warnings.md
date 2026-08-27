---
aar: AAR-038-operator-custom-cartridge-trust-import-and-player-warnings
ticket: TICKET-038
pipeline: operator-custom-cartridge-trust-import-and-player-warnings
status: submitted
opened: 2026-08-27
submitted: 2026-08-27
effectiveness: effective
---

# AAR-038-operator-custom-cartridge-trust-import-and-player-warnings

## Recalled at plan

| ID or source | How surfaced | Useful? |
|---|---|---|
| `AD-omarchy-gaming-system-owner-operated-server-extension-boundary-001` | ADR-0003 and roadmap recall | Yes; authorizes a distinct operator-custom inert path and forbids marketplace/support conflation. |
| `PR-omarchy-gaming-system-separate-publisher-marketplace-server-attestations-001` | Knowledge-register search | Yes; fixes publisher, review, admission, and absence states as separate claims. |
| `PR-omarchy-gaming-system-authenticate-independent-claims-outside-claiming-authority-001` | Ticket 033 acquisition recall | Yes; the selected server may advertise a custom key candidate but cannot silently establish the player's trust in it. |
| `PR-omarchy-gaming-system-separate-historical-provenance-from-current-use-policy-001` | Ticket 035 historical acquisition recall | Yes; custom session history must retain authentic origin while current lifecycle remains separate authority. |
| `PR-omarchy-gaming-system-snapshot-mutable-build-inputs-before-verification-001` | Ticket 036/037 signing recall | Yes; local import must verify and persist one owned byte snapshot rather than reopen mutable inputs. |
| `PR-omarchy-gaming-system-bind-permissions-to-opened-file-descriptors-001` | Ticket 037 security finding | Yes; new client trust and admin-signing stores must apply permissions through bound descriptors. |
| Tickets 032–035 completed specs/notes/AARs | Nearest implementation pipelines | Yes; supply secure staging, admission, distribution, client trust/cache/mount, session pinning, and historical retrieval seams. |
| ADR-0003, Game Cartridges architecture, and affected OpenWiki pages | Required durable context | Yes; keep custom provenance visible and keep cartridges outside provider/module authority. |

## What happened

Ticket 038 composed the existing signed inert-cartridge, PostgreSQL admission,
authenticated distribution, client cache/mount, trusted renderer, and session
pinning seams into a visibly distinct operator-custom path. The admin process
alone owns the private key and verified import/lifecycle publication; normal
serving exposes public evidence. A player must explicitly pin the exact
canonical server origin, stable server UUID, and operator key before custom
content can install or render. Catalog, acquisition, mount, session, and QML
contracts preserve the custom source and fixed unreviewed-content warning, and
custom content gains no marketplace, executable, provider, or gameplay
authority.

The security diff inspection found one low-severity cross-domain race: the new
custom lifecycle writer did not acquire the global lifecycle lock already used
by durable cartridge-action admission. The writer now acquires that lock
exclusively before the per-game lock and holds both through commit; a
deterministic writer-first PostgreSQL regression proves that a later fresh
action waits and observes the committed denial. Independent server and client
reviews found no residual issue. Validation also separated cold package
startup from the post-load application watchdog and exposed that manual HTTP
observation can contaminate an exact-request fixture. The clean canonical gate
and OpenWiki lifecycle then completed.

## Failures captured

| ID | Failure | Where surfaced |
|---|---|---|
| `BF-omarchy-gaming-system-custom-policy-action-linearization-race-001` | An operator-custom lifecycle denial could commit concurrently with a fresh cartridge action because the writer and admission path did not share the global lifecycle lock domain. | Codex Security diff scan and deterministic PostgreSQL race regression. |
| `BF-omarchy-gaming-system-package-smoke-preload-watchdog-conflation-001` | A single short outer timeout treated cold QML process startup as if it were the loaded application's liveness deadline. | Complete client-package gate on a cold preload. |
| `BF-omarchy-gaming-system-contract-test-observer-request-pollution-001` | A manual status request used the fixture's asserted HTTP interface and changed the exact request history under test. | Coverage gate fixture request assertion. |

## Prevention rules captured

| ID | Rule | Why |
|---|---|---|
| `PR-omarchy-gaming-system-share-lifecycle-writer-use-admission-lock-domain-001` | Every lifecycle writer must acquire the same global lock domain and order used by durable use admission, and writer-first denial must be tested. | Per-record serialization alone cannot linearize a policy transition against a separately locked use-admission path. |
| `PR-omarchy-gaming-system-separate-process-startup-and-post-load-watchdogs-001` | Give process/cold-cache startup and loaded-application liveness independent deadlines. | Startup variance should not produce false failures or force weakening the narrower product watchdog. |
| `PR-omarchy-gaming-system-observe-exact-request-contracts-outside-tested-interface-001` | Observe exact-request fixtures through process or log state outside the protocol surface whose request sequence is asserted. | An observer request is still a request and can invalidate the contract under test. |

## Architecture decisions

| ID | Decision | ADR |
|---|---|---|
| `AD-omarchy-gaming-system-operator-custom-cartridge-trust-boundary-001` | Operator-custom cartridges use a distinct server-scoped attestation and explicit per-server player key pin while retaining inert presentation and existing gameplay authority. | `docs/architecture/game-cartridges.md`; ADR-0003. |

Every new ID above must also be appended to
`docs/planning/knowledge/INDEX.md`.

## Effectiveness

Effective. Recalled rules prevented custom content from borrowing marketplace
review authority, made the client trust decision independent from the server's
advertisement, and reused immutable evidence/current-policy separation. The
inspection still found a new interaction between the custom policy writer and
the existing action-admission lock; recording the shared lock-domain rule makes
that cross-cutting requirement explicit for future lifecycle writers. Existing
workspace-Clippy and exact-contract lessons correctly classified two
implementation failures, so no duplicate IDs were created for them.

---
aar: AAR-035-historical-session-cartridge-acquisition-and-multi-screen-navigation
ticket: TICKET-035
pipeline: historical-session-cartridge-acquisition-and-multi-screen-navigation
status: submitted
opened: 2026-08-26
submitted: 2026-08-27
effectiveness: effective
---

# AAR-035-historical-session-cartridge-acquisition-and-multi-screen-navigation

## Recalled at plan

| ID or source | How surfaced | Useful? |
|---|---|---|
| `PR-omarchy-gaming-system-bind-profile-mounts-to-origin-and-server-001` | Knowledge search plus Ticket 034 mount resolution | Yes; historical installation and rendering must remain scoped to canonical origin plus immutable server UUID. |
| `PR-omarchy-gaming-system-persist-action-admission-before-external-effects-001` | Ticket 034 action lifecycle review | Yes; secondary-screen gameplay actions must retain the same durable lifecycle linearization and retry semantics. |
| `PR-omarchy-gaming-system-render-only-from-accepted-plan-state-001` | Ticket 034 QML finding | Yes; navigation metadata and destination plans must be consumed only after exact envelope acceptance. |
| `PR-omarchy-gaming-system-authenticate-independent-claims-outside-claiming-authority-001` | Ticket 033 trust-boundary review | Yes; an owner-operated server cannot choose the client's marketplace trust key or manufacture historical marketplace evidence. |
| `PR-omarchy-gaming-system-bind-node-actions-to-exact-payloads-001` | Cartridge verifier and renderer inspection | Yes; host navigation needs a disjoint exact no-payload emitter contract, while gameplay remains schema-shaped. |
| `PR-omarchy-gaming-system-persist-authenticated-denial-before-enforcement-001` | Secure-store lifecycle inspection | Yes; older snapshot evidence must never override a newer authenticated suspension or revocation decision. |
| `PR-omarchy-gaming-system-recount-budgets-at-render-handoff-001` | Trusted QML boundary inspection | Yes; every secondary-screen plan keeps the same independent node and media budget recount. |
| Tickets 032–034 completed specs, notes, and AARs | Nearest completed marketplace, acquisition, and launch pipelines | Yes; together they identify the exact missing historical snapshot seam, immutable session pin, local mount trust, and entry-only limitation. |
| `docs/architecture/game-cartridges.md`, system overview, ADR-0003 | Required architecture recall | Yes; they require exact release history, inert signed presentation, host-owned navigation, and no direct cartridge/provider authority. |
| OpenWiki quickstart, game-cartridges, and runtime-foundation | Required generated context | Yes; they locate current sync, distribution, cache, session, companion, renderer, and QML boundaries for design and later reconciliation. |

## What happened

Ticket 035 completed the historical and multi-screen cartridge vertical. The
server now retains normalized immutable marketplace evidence for each reviewed
release, while current signed lifecycle policy remains the authority for use.
A participant can explicitly install the exact release pinned to an old
session even after catalog advancement, and one bounded profile can retain
multiple exact same-game mounts. Signed cartridges may declare cyclic
Button-only local navigation between reviewed screens; trusted QML owns the
bounded history and every gameplay action returns to the server with its exact
screen identity before durable admission and existing compiled/provider
dispatch.

Inspection found three contract defects: historical evidence lifecycle was
initially conflated with current authorization, malformed reserved navigation
could fall through to gameplay, and producer/consumer navigation constraints
disagreed. All were fixed with regressions. The first canonical gate exposed
only formatting and one Clippy layout issue; the fresh 22-stage gate passed all
Rust, PostgreSQL, QML, SDK, package, provider, recovery, and private-alpha
checks. Codex Security accounted for every changed runtime input and reported
no security finding. OpenWiki completed after reconciling the affected
cartridge, runtime, and quickstart pages, with only explicit pre-existing
evidence-debt warnings retained.

## Failures captured

| ID | Failure | Where surfaced |
|---|---|---|
| `BF-omarchy-gaming-system-historical-evidence-current-policy-conflation-001` | The companion compared a release's first retained marketplace lifecycle claim with the session's current lifecycle projection, rejecting valid continuing sessions when those independently authentic claims differed. | Focused historical-acquisition integration tests and inspection |
| `BF-omarchy-gaming-system-reserved-navigation-prefix-fallthrough-001` | The malformed reserved action `navigate.` was neither valid navigation nor rejected reserved input, so it could be interpreted as a gameplay action. | Protocol inspection and cartridge-verifier regression |
| `BF-omarchy-gaming-system-navigation-envelope-contract-drift-001` | The signed-plan producer and QML consumer disagreed on navigation cardinality and duplicate-emitter semantics, causing valid-plan rejection or late ambiguity. | Renderer/QML contract inspection and production-root fixtures |
| `BF-omarchy-gaming-system-clean-clone-cartridge-version-drift-001` | The clean-clone provider proof still expected Door Legends v1 after the immutable signed cartridge advanced to v2, so its exact protocol expectation failed after correct runtime behavior was built. | Clean-clone remote-provider authority pilot |

## Prevention rules captured

| ID | Rule | Why |
|---|---|---|
| `PR-omarchy-gaming-system-separate-historical-provenance-from-current-use-policy-001` | Authenticate historical release provenance independently from current selection and current-use lifecycle policy; never let retained older evidence grant or deny current use by itself. | A first authentic snapshot proves what was reviewed then, while a newer signed policy is the authority for whether the exact release may be used now. |
| `PR-omarchy-gaming-system-fail-closed-on-reserved-action-namespaces-001` | Reject every malformed member of a reserved action namespace before subtype parsing, and never let it fall through to ordinary gameplay semantics. | A parser result that combines “not reserved” with “reserved but invalid” creates ambiguous authority at downstream dispatch boundaries. |
| `PR-omarchy-gaming-system-align-producer-consumer-limits-and-uniqueness-001` | Derive cardinality and uniqueness rules from one contract and enforce them at every producer/consumer handoff. | Independently reasonable limits can still make authenticated output unusable or ambiguous when adjacent boundaries disagree. |
| `PR-omarchy-gaming-system-treat-clean-clone-fixtures-as-protocol-clients-001` | Update clean-clone fixtures, exact schemas, and immutable release identities in the same change as the protocol they exercise. | An integration fixture is a real protocol consumer, not disposable scaffolding, when it supplies delivery evidence. |

## Architecture decisions

| ID | Decision | ADR |
|---|---|---|
| `AD-omarchy-gaming-system-historical-acquisition-and-host-navigation-boundary-001` | Historical acquisition resolves only an immutable session pin from retained authentic evidence under current lifecycle policy; explicit client installation creates exact coexisting mounts; signed navigation remains a local bounded host operation; and gameplay authority stays screen-bound on the server. | `../../architecture/game-cartridges.md`; `../../architecture/system-overview.md` |

Every new ID above must also be appended to
`docs/planning/knowledge/INDEX.md`.

## Effectiveness

Effective. Recalled mount-origin, independent-trust, durable-admission,
accepted-plan, exact-action, lifecycle, and budget rules kept the extension
inside the established inert-cartridge boundary. Independent inspection found
three real cross-boundary correctness gaps before delivery, and each gained a
focused regression. The complete gate then proved the historical database,
participant API, companion, multi-mount cache, renderer, QML, package, SDK,
provider, and recovery path together. The resulting system can recover an old
reviewed frontend and navigate multiple screens without substituting a current
release, granting publisher execution/network authority, or changing the
game's sole server-side rules authority.

---
aar: AAR-034-session-pinned-cartridge-render-plan-and-gameplay-launch
ticket: TICKET-034
pipeline: session-pinned-cartridge-render-plan-and-gameplay-launch
status: submitted
opened: 2026-08-26
submitted: 2026-08-26
effectiveness: effective
---

# AAR-034-session-pinned-cartridge-render-plan-and-gameplay-launch

## Recalled at plan

| ID or source | How surfaced | Useful? |
|---|---|---|
| `PR-omarchy-gaming-system-bind-provider-messages-to-registered-identity-001` | Knowledge search plus provider-session inspection | Yes; session, release, cartridge, action, subject, revision, and provider authority must stay exact through dispatch. |
| `PR-omarchy-gaming-system-bind-node-actions-to-exact-payloads-001` | Knowledge search plus v1 presentation validation | Yes; the server action boundary must accept only the payload shape emitted by the verified node family. |
| `PR-omarchy-gaming-system-recount-budgets-at-render-handoff-001` | Knowledge search plus trusted QML inspection | Yes; the existing QML plan recount remains mandatory when plans move from the companion into the scene. |
| `PR-omarchy-gaming-system-persist-authenticated-denial-before-enforcement-001` | Ticket 033 cache/lifecycle inspection | Yes; active-session rendering must use the monotonic signed policy rather than only a stale mount label. |
| `PR-omarchy-gaming-system-authenticate-independent-claims-outside-claiming-authority-001` | Ticket 033 AAR | Yes; mounted rendering continues to require the independently provisioned complete marketplace key. |
| `PR-omarchy-gaming-system-bind-presentation-cardinality-before-indexing-001` | Game-controller knowledge recall | Yes; participant/session identity is validated before either cartridge or platform presentation may index it. |
| Ticket 033 completed spec/notes and AAR | Nearest completed pipeline | Yes; they define the exact mount facts, companion trust boundary, and intentionally deferred launch seam. |
| Ticket 024 completed notes | Nearest gameplay/QML pipeline | Yes; they define current game authority, mutation retry, platform-presenter fallback, and minimum-layout expectations. |
| `docs/architecture/game-cartridges.md`, system overview, ADR-0003 | Required architecture recall | Yes; they require exact session pinning, trusted rendering, server-brokered actions, and no cartridge execution/network authority. |
| OpenWiki quickstart, game-cartridges, runtime-foundation, and product-boundaries | Required just-in-time repository context | Yes; they map the current mount/session/provider seams and the documentation that Phase 5 must reconcile. |

## What happened

Ticket 034 completed the first truthful portable gameplay vertical. Eligible
new compiled and registered-provider sessions pin one exact current marketplace
release and admission revision without changing their sole rules authority. The
same-user companion resolves only the matching canonical server profile mount,
compiles the authenticated entry-screen view through the production renderer,
and exposes bounded digest assets through an ephemeral loopback capability.
Trusted QML independently accepts the plan, while every declared action returns
to OmarchyGS for participant authorization, exact signed-contract validation,
durable admission, and dispatch through the existing compiled/provider command
path. The clean-clone Door Legends cartridge/provider proof now reaches its
terminal result and recovers it after restart.

Codex Security's complete changed-file review found three low-severity trust
gaps: origin was absent from mounted-render resolution, and compiled/provider
actions could straddle a later lifecycle transition without a durable
authorization point. All three were fixed. The canonical gate then exposed a
standalone preview still reading raw input after the trusted surface began
retaining only accepted plans, plus formatting and Clippy placement drift. The
second complete 22-stage gate passed, including 55 PostgreSQL tests, live QML,
provider conformance, clean-clone authority, recovery, package, and
private-alpha proofs. OpenWiki completed and authored architecture/API/client/
operator/roadmap documentation was reconciled.

## Failures captured

| ID | Failure | Where surfaced |
|---|---|---|
| `BF-omarchy-gaming-system-render-mount-origin-substitution-001` | Render preparation bound a server UUID but omitted the currently selected canonical origin, so a UUID collision could resolve another origin's trusted profile mount. | Codex Security diff scan and hostile cross-origin client-runtime test |
| `BF-omarchy-gaming-system-compiled-cartridge-action-lifecycle-race-001` | Compiled cartridge action validation had no linearization point with marketplace suspension/revocation, so signed authorization could change before command execution. | Codex Security diff scan and PostgreSQL action-first/writer-first concurrency tests |
| `BF-omarchy-gaming-system-provider-cartridge-retry-lifecycle-race-001` | Provider actions shared the lifecycle race, and an exact uncertain retry could be reclassified by a later suspension before provider idempotency recovered the first operation. | Codex Security diff scan and post-suspension provider replay test |
| `BF-omarchy-gaming-system-trusted-preview-raw-plan-authority-drift-001` | The standalone preview still dereferenced raw `renderPlan` after direct `acceptPlan`, bypassing the trusted surface's new accepted-state separation and producing null access. | First canonical diff gate, trusted renderer/QML stage |

## Prevention rules captured

| ID | Rule | Why |
|---|---|---|
| `PR-omarchy-gaming-system-bind-profile-mounts-to-origin-and-server-001` | Bind a client profile mount to both the canonical selected origin and stable server UUID, and reject mixed-origin records inside one UUID profile. | A stable application UUID is a continuity fact, not transport authentication or a globally collision-proof origin selector. |
| `PR-omarchy-gaming-system-persist-action-admission-before-external-effects-001` | Linearize mutable lifecycle authorization into an immutable exact action admission before compiled execution or provider I/O, and resolve exact replay before current-policy denial. | A preflight check alone cannot define whether work authorized before a concurrent transition may complete or recover after uncertainty. |
| `PR-omarchy-gaming-system-render-only-from-accepted-plan-state-001` | After validating a render envelope, every presenter, metric, assertion, and component loader must consume only retained accepted plan state, never the raw input property. | Separating untrusted input from accepted state is ineffective if a secondary path continues to dereference the original input. |

## Architecture decisions

| ID | Decision | ADR |
|---|---|---|
| `AD-omarchy-gaming-system-session-pinned-cartridge-gameplay-boundary-001` | Presentation identity is an immutable optional exact session pin; rendering requires a separately trusted matching local mount; actions are unconfirmed until durably authorized by OmarchyGS and adapted to the session's sole existing rules authority. | `../../architecture/game-cartridges.md`; `../../architecture/system-overview.md` |

Every new ID above must also be appended to
`docs/planning/knowledge/INDEX.md`.

## Effectiveness

Effective. Recalled exact-identity, action-shape, independent-trust, and render
handoff rules prevented cartridge code/network authority from entering the
design. Independent inspection found three real low-severity gaps before
delivery, each gained adversarial regression evidence, and the canonical gate
caught the remaining trusted-preview integration drift. The final architecture
keeps marketplace review, server admission, client trust, presentation, and
gameplay authority distinct while proving one real portable game end to end.
The explicit next boundaries are historical-release acquisition, multi-screen
navigation, operator-custom trust, the public Provider SDK, external-provider
onboarding, and isolated server modules.

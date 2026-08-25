# ADR-0002: Game Cartridge and provider boundary

- Status: accepted; scoped first-party remote authority authorized by Ticket 019
- Date: 2026-08-24
- Knowledge ID: `AD-omarchy-gaming-system-game-cartridge-provider-boundary-001`

## Context

OmarchyGS currently compiles game rules into its Rust server and persists the
authoritative game snapshot and revision in PostgreSQL. That is an appropriate
private-alpha boundary, but the product direction calls for BBS-inspired games
to live in separate repositories, target an OmarchyGS SDK, and eventually
allow registered providers to own their server-side game rules and state.

Backend portability does not by itself solve frontend delivery. Loading a
publisher's QML, JavaScript, native library, or arbitrary remote page into the
trusted launcher would expose platform credentials, filesystem and network
capabilities, accessibility consistency, and the client process to the game.
Qt's security model explicitly treats QML and JavaScript as trusted code.

The current Constitution §10 also says the OmarchyGS server is authoritative
for game state, turns, time, randomness, rewards, and permissions. A remote
provider cannot be introduced honestly while both platform and provider claim
that authority.

Ticket 014 compared compiled, separately versioned, sandboxed-local, and remote
models and exercised the proposed boundary with a signed data package, trusted
QML renderer, platform broker, and separate provider process.

## Decision

Adopt the **OmarchyGS Game Cartridge** as the portable frontend and release
artifact. A cartridge is an immutable, publisher-signed, content-addressed
package containing only a versioned manifest, declarative screen templates,
schemas, localization, and bounded static assets. The OmarchyGS client renders
those records through a versioned vocabulary of trusted QML components.

Cartridges cannot contain or invoke QML, JavaScript, native code, shell
commands, arbitrary shaders, imports, dynamic remote assets, filesystem paths,
or network clients. Game gestures emit only declared, schema-validated action
IDs and arguments to the trusted OmarchyGS API. Authentication and MFA remain
reserved platform surfaces with unspoofable platform chrome.

Keep the compiled Rust runtime authoritative during the private alpha. Build
portability in stages:

1. publish the cartridge schema, verifier, capability negotiation, trusted
   Core/Rich-2D renderer, previewer, and conformance fixtures;
2. move a first-party game's source to a separate repository while consuming
   versioned artifacts through the same public contract;
3. retain OmarchyGS as the authenticated network broker if remote providers
   are enabled later; and
4. delegate durable gameplay state/revision to exactly one registered provider
   only after the production protocol, migration, and Constitution §10
   amendment pass their own pipeline.

In remote mode, OmarchyGS remains authoritative for accounts, sessions, MFA,
personas/avatar projections, social state, catalog and launch policy, provider
registration, the platform session envelope, achievements, notifications,
audit, and suspension. The provider becomes authoritative for its scoped game
rules, private game state, turns, game time/randomness, and provider revision.
OmarchyGS stores authenticated receipts and platform projections, not a second
writable gameplay snapshot.

Provider access is brokered. OmarchyGS resolves only an operator-registered
endpoint and issues a short-lived asymmetric grant bound to the provider
audience, game/rules/cartridge versions, platform session, one scope, pairwise
persona subject, expiry, and replay ID. Account identity, credentials, reusable
device tokens, and database access never cross that boundary. Commands retain
an idempotency key and expected provider revision across timeout retries;
results/events are authenticated and deduplicated before platform effects.

The initial presentation target is Cartridge Core plus Rich 2D. Terminal and
panel layouts, boards, cards, tile/sprite scenes, local animation, particles,
platform effects, and bounded audio are in scope. Advanced 2D/2.5D, Qt Quick
3D, WebEngine, and local Wasm are separate capability profiles with their own
dependency, hardware, licensing, and threat decisions. A cartridge never gains
general-purpose code because a richer profile is added.

## Constitution reconciliation

Ticket 019 amends Constitution §10 after implementing and exercising the
first-party authority gate. The amendment distinguishes **platform authority**
from **registered scoped gameplay authority**:

- OmarchyGS retains authentication, accounts/personas, social state, catalog
  and launch policy, the participant-private session envelope, public
  result/achievement definitions and projections, audit, suspension, and
  recovery;
- compiled sessions retain the original OmarchyGS rules/state/revision
  authority;
- a provider session pins one operator-registered exact immutable release as
  the sole durable owner of rules, private state, turns, game time/randomness,
  provider revision, and outcome;
- OmarchyGS stores no writable provider gameplay snapshot and offers no
  compiled failback;
- only brokered, signed, scoped exchanges may cross the boundary, and provider
  claims become platform effects only in the atomic policy/projection
  transaction; and
- REST/cursor recovery remains durable truth while WebSockets remain hints.

This authorization is deliberately narrow: Door Legends v1 is the sole
first-party pilot, and external or self-service providers remain unauthorized.
The amendment follows the implementation and separate-process proof; it does
not waive registration, TLS/egress, replay, quota, lifecycle, audit,
reconciliation, disaster-recovery, or delivery gates.

## Current-code gap map

| Current seam | Reusable property | Required future change |
|---|---|---|
| `GameDefinition` and `GameRegistry` | Database-free exact-version rules contract | Add a provider abstraction only after the remote protocol is approved; do not make the existing trait network-aware |
| `game_sessions` snapshot/revision | Exact identity, participants, optimistic revision, idempotency foundation | Separate platform envelope/receipts from provider-owned state without dual writes; migrate existing sessions explicitly |
| Authenticated persona routes | Owner and participant authorization | Derive scoped provider grants server-side; never forward device tokens or account IDs |
| Persona sync cursor and WebSocket hints | Durable recovery plus low-latency wake-up | Add minimal game/provider invalidations; keep provider callbacks out of the client socket path |
| QML health connector | Trusted executable shell and clear connection states | Add the trusted cartridge renderer/previewer; never load package QML or provider URLs |
| Empty compiled catalog | Stable public discovery boundary | Add approved cartridge/provider/version identity and revocation state before external listings |
| PostgreSQL transaction tests and delivery smoke | Real durability and vertical-slice evidence | Add provider failure, retry, replay, reconciliation, and minimum-hardware renderer conformance environments |

## Alternatives rejected or deferred

- **Raw QML/JavaScript or native plugins:** rejected because they are trusted
  executable code, not a cartridge sandbox.
- **Provider-hosted web UI as the baseline:** rejected because it adds mutable
  post-review content and a large browser/origin/bridge surface. A locked
  WebEngine profile may be evaluated later as an explicit compatibility tier.
- **Direct client-to-provider traffic:** deferred because it would distribute
  grants, endpoint policy, retries, audit, privacy enforcement, and revocation
  into every client.
- **Wasm as the entire SDK:** deferred. Wasm may isolate computation, but it
  still requires a capability ABI and trusted presentation contract.
- **Move all games remote immediately:** rejected because it adds distributed
  operations before the first playable and violates the current constitution.

## Consequences

- Games can carry a retro cartridge identity and release presentation assets
  independently while the player remains inside one OmarchyGS launcher.
- The safe graphics ceiling is a reviewed host vocabulary rather than the raw
  Qt ceiling. Rich 2D can be broad; new primitives require a renderer/SDK
  release and conformance budget.
- First-party games must use the same schemas and conformance suite intended
  for later publishers, avoiding a permanent private integration path.
- Remote play gains one broker hop and significant key, registry, quota,
  persistence, monitoring, and support work.
- The isolated Ticket 014 proof remains non-production evidence. Its ephemeral
  keys, loopback HTTP, in-memory replay state, and compact limits must not be
  copied into a deployed provider path.

## Follow-up sequence

1. `TICKET-015` — versioned cartridge contract, verifier, and conformance CLI.
2. `TICKET-016` — trusted Core/Rich-2D renderer and previewer.
3. `TICKET-017` — separate-repository SDK/release workflow and first-party
   cartridge consumption.
4. Resume challenges and the first playable against the compiled authority,
   using the accepted cartridge seams where appropriate.
5. `TICKET-018` — production provider identity, broker, and protocol security
   foundation after private-alpha needs justify it.
6. `TICKET-019` — first-party remote-provider migration pilot and the required
   Constitution §10 amendment.

The full contract, threat model, graphics profiles, provisional budgets,
protocol failure behavior, and proof evidence are maintained in
[Game Cartridges](game-cartridges.md).

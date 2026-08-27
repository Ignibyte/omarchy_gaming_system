# ADR-0003: Owner-operated server and extension boundary

- Status: accepted product direction; implementation remains separately gated
- Date: 2026-08-26
- Knowledge ID: `AD-omarchy-gaming-system-owner-operated-server-extension-boundary-001`

## Context

OmarchyGS already separates a signed inert Game Cartridge from trusted QML
rendering and from the single authority that owns game rules. It also has a
compiled first-party runtime, a secure cartridge import/provenance foundation,
and one operator-pinned remote-provider pilot. What remained unclear was who
owns a normal deployment, how a marketplace reaches players, and where custom
administrator code belongs.

The intended product is not only one central hosted service. An individual or
group should be able to run the standard OmarchyGS server, choose a library of
games, and invite friends into that server. Operators also control their own
machines and may choose to install content or code that the OmarchyGS
marketplace did not review. That freedom must not be misrepresented as project
vetting, and it must not let a remote server execute arbitrary code in the
official client.

The phrase "the game is the QML side" also needs a precise security meaning.
The portable game frontend is declarative cartridge data interpreted by
platform-owned QML; raw publisher QML is executable trusted application code
and remains prohibited.

## Decision

Treat an independently owner-operated OmarchyGS deployment as a first-class
community and trust domain. It runs the standard Rust/PostgreSQL architecture
and owns its accounts, personas, social graph, catalog/launch policy, platform
session envelopes, projections, audit, and recovery. Multiple processes behind
one server origin may form one logical deployment; independent origins do not
share identity or policy unless a future federation ADR explicitly adds it.

Adopt a server-curated marketplace model:

1. A vetted marketplace publishes an exact publisher-signed cartridge release
   with review/provenance and lifecycle metadata.
2. A server administrator imports and activates that exact release under the
   server's catalog policy. Marketplace publication does not force admission.
3. Players connecting to that server see only its admitted catalog and the
   provenance of each release.
4. The official client acquires the exact `.ogsc` bytes through a
   server-approved bounded distribution path, verifies the publisher integrity
   signature, optional marketplace review attestation, selected server's
   admission policy, and content digest, then stores them in a local
   content-addressed read-only cache.
5. A game session pins the exact cartridge, rules, and authority identities.
   The trusted client renders the cartridge locally, while actions travel only
   through the selected OmarchyGS server.

A cartridge is frontend presentation and integration data only. It contains a
manifest, declarative screens, schemas, localization, bounded assets, and
declared actions. It contains no raw QML/JavaScript, native code, server rules,
backend process, credentials, arbitrary destination, or direct network client.
The game backend is either a current platform-compiled definition or a
separately deployed registered provider.

Package the backend contract later as an **OmarchyGS Provider SDK**. It should
include the public provider model/protocol, starter service, version
negotiation, signing/grant helpers, conformance and fault fixtures, and
deployment/operations guidance. A provider owns only the rules/state surfaces
of its exact registered game release; OmarchyGS remains the authenticated
broker and platform authority. The provider contract—not a general plugin
hook—is how portable game backends integrate.

An operator may deploy a custom provider on the same infrastructure as their
OmarchyGS server, but it remains a separately identified service with separate
state and least-privilege credentials. Supporting a co-located provider needs
an explicit authenticated local-transport profile; it must not become a broad
loopback/private-network exemption or direct access to the platform database.

Permit a separately marked **operator-custom** path. A server administrator may
enable a local trust authority and import a locally signed inert cartridge
without marketplace approval. The administrator may eventually install
server-side extension code through the server module system. Catalog/API/UI
provenance must distinguish marketplace-vetted, first-party, and
operator-custom releases. Marketplace review, project support, and operator
trust must never be presented as equivalent.

Marketplace bypass does not bypass official-client safety. Every cartridge,
including an operator-custom one, still passes format, size, schema, media,
capability, and content-digest validation and renders only through trusted
components. A server cannot use a custom catalog entry to deliver publisher
QML, JavaScript, a native client plugin, arbitrary Web content, or a direct
provider connection.

Publisher integrity, marketplace review, and server admission are separate
claims. A vetted release carries all three. An operator-custom release carries
publisher/operator integrity plus server admission and explicitly has no
marketplace-review attestation. A signature proves provenance and unchanged
bytes; it does not by itself claim safety, support, or marketplace review.

Define general server modules as a third extension family, separate from
cartridges and game providers. A future module system must have:

- a versioned manifest and compatibility contract;
- explicit capability grants and typed lifecycle/domain hooks;
- configuration and state/migration namespaces;
- bounded time, resource, failure, retry, and ordering semantics;
- durable audit plus disable, upgrade, rollback, and recovery behavior;
- no raw credential, unrestricted database, or client-code bridge; and
- conformance fixtures for both marketplace-vetted and operator-custom use.

Hooks may observe allowlisted events and submit typed intents through core
domain authorization. They do not mutate protected database state behind the
platform's services. A dedicated architecture/security spike must select and
prove an external-process, Wasm, statically compiled, or other isolation model
before executable extensions are authorized. This ADR does not authorize an
unstable dynamic Rust ABI or third-party code in the main server process.

Independent operators own deployment configuration, availability, backups,
moderation, custom code, incident response, and applicable legal/privacy
obligations. The project does not remotely administer or continuously review
their servers. Player-visible provenance and reviewed terms/custom-content
warnings are required before public distribution, but disclaimers are not a
replacement for client isolation, transport security, least privilege,
revocation, audit, or resource limits.

## Trust classes

| Class | Installed by | Project/marketplace review | Executable location | Required player signal |
|---|---|---|---|---|
| Marketplace-vetted cartridge | Server operator from reviewed catalog | Exact release and provenance reviewed | None; inert local client data | Marketplace identity and exact release |
| Operator-custom cartridge | Server operator under local signing authority | None implied | None; inert local client data | Custom/unvetted server content and operator identity |
| Registered game provider | Operator-approved exact backend release | First-party pilot today; external review later | Separate provider service | Provider authority and availability/provenance |
| Server module | Server operator through future module runtime | Depends on distribution channel | Server side only under the selected isolation model | Module/custom-server status where it affects players |

## Current authorization boundary

This decision records product direction, not implementation authorization.
Private-alpha federation and user-supplied native plugins remain non-goals.
Door Legends v1 remains the sole remote-provider pilot. The current main client
can explicitly acquire, independently verify, cache, update, mount, and locally
remove marketplace-vetted cartridges selected by the active server. Independent
verification requires an exact marketplace public key provisioned on the
client outside that selected server; server-supplied key labels or key bytes
cannot establish marketplace provenance. It does not
yet bind a mount to a live game session or render it as the authoritative game
surface, and no general server plugin runtime exists. External providers,
marketplace services, operator-custom installation, and executable modules each
require their own ticketed security, operations, compatibility, and recovery
evidence.

## Alternatives rejected or deferred

- One mandatory central OmarchyGS server: rejected because it defeats the
  owner-operated community product.
- One global marketplace automatically controlling every server catalog:
  rejected because publication and local admission are different authorities.
- Federation as implicit multi-server behavior: deferred; identities and
  social/game state stay server-local.
- Cartridge-embedded backend executable: rejected because distribution,
  gameplay authority, operations, and frontend safety are separate concerns.
- Raw custom QML or native client plugins for administrator servers: rejected;
  choosing a server never grants local executable authority.
- General plugin hooks as the game-backend SDK: rejected because it would
  bypass the provider identity, replay, revision, quota, and recovery contract.
- An in-process dynamic Rust plugin ABI: not authorized; compatibility and
  compromise radius require the extension-runtime spike.
- Terms or warnings as the only custom-content control: rejected because legal
  allocation of responsibility does not contain malicious bytes or requests.

## Consequences

- The player experience resembles joining a friend's arcade: choose a server,
  see that operator's library, and cache its exact cartridges locally.
- Server operators retain meaningful ownership and can choose custom content,
  while players receive honest provenance and unchanged client-side safety.
- Marketplace services can distribute the same immutable bytes to many
  independent communities without gaining authority over their accounts or
  launch policy.
- Game developers can ship frontend and backend releases independently. The
  core platform remains game-agnostic at the provider contract even while
  current compiled first-party games remain supported.
- A real module system becomes a substantial post-alpha security and
  compatibility project rather than an unbounded callback registry.
- Cross-server identity, discovery, social graphs, challenges, and matches are
  still federation work and are not promised by this model.

## Follow-up sequence

1. Finish private-alpha packaging and operator reliability work.
2. Add stable server identity/capability discovery and isolated saved client
   profiles.
3. Bind mounted cartridge presentation to exact authoritative game sessions
   after server marketplace synchronization/import and player
   acquisition/cache/mounting.
4. Publish the Provider SDK and prove a second clean-room backend integration.
5. Add explicit operator-local cartridge trust and provenance disclosure.
6. Run the server extension isolation/hook architecture spike.
7. Implement the selected module base, administration, audit, and conformance
   system before allowing administrator-installed executable modules.
8. Complete reviewed self-hosting terms, privacy/custom-content disclosure,
   support boundaries, and external marketplace/provider policy.

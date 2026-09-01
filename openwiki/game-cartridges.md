---
type: "Reference"
title: "Game Cartridges and portable provider direction"
openwiki_generated: true
sources:
  - id: openwiki-source-0d99cc708822fd795c83ba12
    resource: repo://client/qml/cartridge/CartridgePreview.qml
  - id: openwiki-source-2bcdc046ce25b89194fc5af0
    resource: repo://client/qml/cartridge/nodes/TrustedButtonNode.qml
  - id: openwiki-source-8b590f320258f337a5d990d8
    resource: repo://client/qml/cartridge/nodes/TrustedParticleFieldNode.qml
  - id: openwiki-source-90c7a5a0010f8b345d61cb73
    resource: repo://client/qml/cartridge/nodes/TrustedTerminalNode.qml
  - id: openwiki-source-c566a55d52a9744f7b26b7c4
    resource: repo://client/qml/cartridge/TrustedCartridgeSurface.qml
  - id: openwiki-source-a046e08cc1ba7740db940ad2
    resource: repo://client/qml/game/SignalSiegeSurface.qml
  - id: openwiki-source-da678ac479c336e5e6fc1d04
    resource: repo://client/qml/GameController.qml
  - id: openwiki-source-bc8915a33f270bc28a270170
    resource: repo://crates/client-cartridge-runtime/src/service.rs
  - id: openwiki-source-f4e5b7474eca8daeac03aaab
    resource: repo://crates/game-cartridge-renderer/src/bin/omarchygs-cartridge-preview.rs
  - id: openwiki-source-fdf115002c4aabad0babec70
    resource: repo://crates/game-cartridge-renderer/src/lib.rs
  - id: openwiki-source-1b7f713ef3a21610bcb995cd
    resource: repo://crates/game-cartridge-spike/README.md
  - id: openwiki-source-45df52cda75cb0ccadd8ef3e
    resource: repo://crates/game-cartridge-spike/src/lib.rs
  - id: openwiki-source-8899ed5703baed5a96fa4f93
    resource: repo://crates/game-cartridge/src/archive.rs
  - id: openwiki-source-b4a2591d7d7f80d847ef95ed
    resource: repo://crates/game-cartridge/src/contract.rs
  - id: openwiki-source-e6274a9b801981dbeca2a0b5
    resource: repo://crates/game-cartridge/src/lifecycle.rs
  - id: openwiki-source-20452fec62fdae4a8bc45707
    resource: repo://crates/game-cartridge/src/marketplace.rs
  - id: openwiki-source-a1b45828c3f97dd0a06fb618
    resource: repo://crates/game-cartridge/src/release.rs
  - id: openwiki-source-111e4189516b7f457a68f043
    resource: repo://crates/game-cartridge/src/sdk.rs
  - id: openwiki-source-71f8ccb7a1e293121205a368
    resource: repo://crates/game-cartridge/src/secure_store.rs
  - id: openwiki-source-07e2881dc5e4740f35a238ee
    resource: repo://crates/game-cartridge/src/store.rs
  - id: openwiki-source-2c5e901f86bcbb656e1b9dfa
    resource: repo://crates/game-cartridge/src/validate.rs
  - id: openwiki-source-358b091c74e2027615ce8f4c
    resource: repo://crates/game-cartridge/tests/sdk_release.rs
  - id: openwiki-source-a28da20d4e4846b146ff3e2b
    resource: repo://crates/game-provider/src/broker.rs
  - id: openwiki-source-5e865738b8ee35e0eee853d7
    resource: repo://crates/game-provider/src/egress.rs
  - id: openwiki-source-183d71a1a996865fb003e694
    resource: repo://crates/game-provider/src/registry.rs
  - id: openwiki-source-2bc4557686cbe5b8dfa44f45
    resource: repo://crates/marketplace-publisher/src/lib.rs
  - id: openwiki-source-14be4e0321d2897243f11e10
    resource: repo://crates/marketplace-publisher/src/probe.rs
  - id: openwiki-source-18fcba4155ece2440818ba7e
    resource: repo://crates/marketplace-publisher/src/store.rs
  - id: openwiki-source-7495094e6001dc09ac9490e6
    resource: repo://crates/marketplace-trust/src/transport.rs
  - id: openwiki-source-01584c5ba7d35b160c5de691
    resource: repo://crates/provider-conformance/src/runner.rs
  - id: openwiki-source-e61b285fcaa489b63922f43f
    resource: repo://crates/server/src/app.rs
  - id: openwiki-source-7243a317e3224aa82795a5fc
    resource: repo://crates/server/src/cartridge_catalog.rs
  - id: openwiki-source-5942cee1725f1a3f7bf01ec7
    resource: repo://crates/server/src/cartridge_distribution.rs
  - id: openwiki-source-9a69a848b9d41472b0830bd4
    resource: repo://crates/server/src/marketplace_egress.rs
  - id: openwiki-source-f6dda000394ac1ba6bba8f65
    resource: repo://crates/server/src/marketplace_sync.rs
  - id: openwiki-source-ff1ed569f105aff512baba65
    resource: repo://crates/server/src/provider_game_api_tests.rs
  - id: openwiki-source-0e10f198b5749ecebf761185
    resource: repo://crates/server/src/provider_games.rs
  - id: openwiki-source-408aa68caebee417a5a319b8
    resource: repo://docs/architecture/adr-0002-game-cartridge-and-provider-boundary.md
  - id: openwiki-source-0fa8a0670e40aca3d14c3478
    resource: repo://docs/architecture/adr-0004-process-isolated-wasm-server-modules.md
  - id: openwiki-source-c22435ddb0c3a9abfe95d9af
    resource: repo://docs/architecture/game-cartridges.md
  - id: openwiki-source-e9c32af872bdfcc1f392d212
    resource: repo://docs/architecture/server-modules.md
  - id: openwiki-source-fa645fac0603cca986708fed
    resource: repo://docs/operators/marketplace-publication.md
  - id: openwiki-source-ff39fa8dfffbd1a097ab5e16
    resource: repo://docs/planning/pipeline/completed/separate-repository-sdk-and-first-party-cartridge.notes.md
  - id: openwiki-source-047cb62ee1741c598c0f11a5
    resource: repo://migrations/0014_provider_security_foundation.sql
  - id: openwiki-source-c1f2a0cfcd9a603e8e6b291c
    resource: repo://migrations/0015_first_party_remote_provider_authority.sql
  - id: openwiki-source-11256f84337d259ecf424a45
    resource: repo://migrations/0019_marketplace_catalog.sql
  - id: openwiki-source-d69dbacb0ae7fe382ee46161
    resource: repo://scripts/test-game-cartridge-renderer.sh
  - id: openwiki-source-8df9ad1a3495f8360740ff03
    resource: repo://scripts/test-game-cartridge-sdk.sh
  - id: openwiki-source-4e51428e90d3c7db3949b09b
    resource: repo://scripts/test-game-cartridge-spike.sh
  - id: openwiki-source-68106a790eb8acc94f8d3540
    resource: repo://scripts/test-game-cartridge.sh
generated: {by: "codex", at: "2026-09-01T22:46:24.106Z"}
---

# Game Cartridges and portable provider direction

## Status and boundary

ADR-0002 accepts the **OmarchyGS Game Cartridge** for staged adoption. Ticket
015 supplies the production local package, verifier/conformance, compatibility,
and inert store. Ticket 016 supplies the production render-plan compiler, fixed
trusted QML vocabulary, and isolated preview CLI. Ticket 017 adds the
deterministic public SDK export, signed release and catalog-policy verification,
separate-repository first-party proof, and secure local importer. Ticket 018
adds the production-grade provider registration, protocol, guarded-egress,
replay, quota, and audit foundation. Ticket 019 instantiates it as an optional
player-server runtime for
the operator-pinned Door Legends v1 first-party pilot and amends Constitution
§10 to assign each session exactly one rules/state/revision authority. Compiled
Signal Siege remains platform-authoritative; external providers remain
unauthorized. [Runtime foundation](runtime-foundation.md) maps both paths.
Ticket 032 implements ADR-0003's first owner-operated distribution slice: one
operator-pinned marketplace, immutable reviewed staging, atomic PostgreSQL
inventory, independent audited server admission, and an authenticated
metadata-only player catalog. Ticket 033 adds exact distribution from that
selection plus independently trusted client verification, a private
content-addressed cache, and per-server-profile mounts. Ticket 034 pins an exact
admitted release to each eligible new session, compiles its signed entry screen
from the matching mount in the native companion, renders it through trusted QML,
and sends declared actions back through the selected server's existing gameplay
authority. Ticket 035 retains immutable historical marketplace evidence,
allows a participant to install the exact old session pin after catalog change,
keeps multiple exact releases mounted side by side, and adds bounded signed
multi-screen host navigation with screen-bound action admission. Ticket 036
adds offline-root-authenticated public trust enrollment, monotonic marketplace
key rotation/revocation, separate historical-evidence/current-policy keys, and
root-authenticated native package staging without installer authority.
Ticket 037 adds deterministic static publication operations with online
catalog review, a public offline-root request/response handoff, immutable
version activation, exact local verification, guarded mirror probes, and a
catalog-compromise/rollback drill. The publisher remains operator tooling and
does not enter the Game Cartridge SDK.
Ticket 038 adds the explicit server-scoped operator-custom trust path,
including admin-only signing/import and lifecycle, source-aware server
admission/session history, player-confirmed client key pins, source-specific
mounts, and persistent unvetted warnings. Ticket 039 and ADR-0004 select and
prove the separate process-isolated no-WASI server-module boundary. Tickets 044
and 045 now implement the public Provider SDK plus its starter, conformance,
deterministic developer-kit release, and second clean-room game. Ticket 046
adds the reviewed exact-release TLS-loopback sidecar, deployment templates,
operator runbook, crash/restore drill, and provider-operation fencing. Real
external-provider onboarding remains outside the repository. Tickets 048
through 058 use those public seams for a separate local Usurper development
provider, including a persistent-game conformance profile, player-private
equipment, shops and haggling, bank and chest transfers, healing-potion
purchases, equipment-aware combat, configured quick-heal-then-attack turns,
and a signed seventeen-screen inert cartridge. Rules v4 also adds the three
source-linked level-one caster spells, mana spend and daily refill, resistance,
encounter reset, and same-turn monster response. Rules v5 adds the original
weapon-gated Assassin Backstab and HP-funded Paladin Soul Strike behind one
provider-routed inert class-special action. Rules v6 adds the passive Gnoll
bite and persistent encounter-owned monster poison across the existing attack,
configured quick-heal, spell, Backstab, and Soul Strike turns, including a
same-turn tick before monster response and provider replay/view coverage.
Rules v7 adds exact source-linked level-two records, bounded draw-free
level-one/level-two switching, preserved rejection-loop RNG work, level-aware
combat and retreat, and inert signed level controls. Its normal level-two loop
retains boundary record 10 as source data but accepts only records 11 through
19. Rules v8 adds exact source-linked level-three records, bounded draw-free
switching across levels one through three, and the same preserved rejection-loop
and level-aware combat path. Its normal level-three loop retains boundary record
20 as source data but accepts only records 21 through 29. Rules v9 extends that
same path through Level 4, retains boundary record 30 as source data, accepts
only records 31 through 39, and initializes combat at strength 14, defence 7,
and 42 HP. Rules v10 extends that path through Level 5, retains boundary record
40 as source data, accepts only records 41 through 49, and initializes combat
at strength 15, defence 7, and 45 HP. Rules v11 extends that path through Level
6, retains boundary record 50 as source data, accepts only records 51 through
59, and initializes combat at strength 16, defence 8, and 48 HP. These tickets add no
production registration, catalog admission, deployment,
publication, platform rule copy, protocol change, trusted QML node, or platform
migration.

Ticket 014 contributes an isolated executable architecture proof. Its broker,
provider, and QML surface are not a public SDK or deployed runtime. Ticket 018
replaces that proof's security assumptions with a production workspace crate
and durable schema. Ticket 019 adds the narrowly scoped player-server bridge,
authority migration, lifecycle, projection, and independent-database proof; it
does not generalize provider onboarding.

## Product and system model

A cartridge is the ROM-like frontend and release identity for one exact game
version. A player chooses it from the trusted OmarchyGS launcher and remains
inside the keyboard-first platform shell.

```text
game repository
  ├─ rules/provider artifact
  ├─ SDK conformance tests
  └─ signed immutable cartridge
       ├─ manifest and capability declarations
       ├─ declarative screen templates
       ├─ view/action/event schemas and localization
       └─ bounded static assets
                  │ approved exact release
                  ▼
       owner-operated server catalog
                  │ exact admitted release + retained evidence
                  ▼
       immutable session presentation pin
                  │ release digest + admission revision
                  ▼
       trusted native client companion
                  │ verified private cache + server-profile mount
                  ▼
       trusted Rust render-plan compiler
                  │ bounded inert plan + signed-screen navigation + digest assets
                  ▼
       trusted OmarchyGS QML components
                  │ unconfirmed declared action
                  ▼
       authenticated OmarchyGS broker
                  │ scoped, short-lived pairwise grant
                  ▼
       registered provider (Door Legends v1 pilot only)
```

The cartridge supplies signed presentation data. OmarchyGS supplies all
executable QML, focus/navigation, accessibility, themes, platform dialogs,
networking, and security policy. In the separately authorized Door Legends
pilot, the provider supplies game rules and private gameplay state; it never
supplies the trusted frontend.

The implemented preview, trusted surface, and fixed visual nodes now consume
the same repository-owned `OgsTheme` palette and typography contract as the
main shell. High contrast, visible focus, semantic roles, reduced motion, mute,
and literal plain-text rendering remain host preferences and behavior; signed
cartridge data can select declared content and actions but cannot inject colors,
markup, styles, or executable presentation code.

Signal Siege's first-playable QML surface is a separate trusted application
path for platform-compiled rules. It may reuse repository-owned inert status,
meter, and button components, but its view model is derived by platform code and
it does not manufacture a signed cartridge origin, content digest, or
`omarchygs.render-plan/v1` document. The signed renderer remains reserved for
packages that passed the verifier and content-addressed installation lifecycle.

## Owner-operated distribution status

The deployment unit is an independently owner-operated OmarchyGS community.
Its server may configure one canonical HTTPS marketplace origin, bounded DER
TLS root, existing secure-store root, and either one manual Ed25519 marketplace
key or an offline-root-verified trust bundle. The owner can synchronize
reviewed exact releases, inspect the resulting inventory, and independently
select one permitted digest per game. Authenticated players see only the
effective selected metadata. The same catalog may also select an explicitly
unvetted operator-custom release signed under the server's stable local
authority; the two sources are a mutually exclusive provenance union rather
than interchangeable digest aliases.

When distribution is configured, the server advertises a separate acquisition
capability and serves only the currently effective selected source and digest.
A marketplace response requires the retained signed snapshot and key; a custom
response requires the immutable server authority and operator attestation.
Both require current lifecycle, exact database admission, publisher evidence,
and immutable secure-store bytes to agree. The response is bounded and
self-verified, with no fallback to another release or provenance class.

That current-selection path is deliberately separate from session recovery.
Every newly created presentation pin now requires immutable normalized evidence
for the exact marketplace-vetted or operator-custom release that established
its provenance. A participant-authorized historical route resolves the exact
session pin through that source's retained evidence, not through today's
catalog selection. Current signed active-session lifecycle policy still
decides whether the old release may be used; retained provenance alone is
never authorization.

The packaged client starts a native loopback companion with a random
per-process credential. Its marketplace authority is mutually exclusive:
no-key mode keeps cartridge controls unavailable, manual mode accepts the
existing client-controlled public key, and channel mode accepts only the
package's offline root and fixed channel bootstrap. Channel enrollment is an
explicit player action and never accepts trust material or a channel location
from the selected server. The companion then rechecks publisher release,
marketplace snapshot, lifecycle policy, compatibility, digest, and
selected-server admission before staging content. Private descriptor-relative
storage keeps immutable cached content separate from exact server-profile
mount records.

Operator-custom trust is a second, explicit companion-owned decision. The
selected server may advertise its public key only as a candidate. Enrollment
binds the canonical origin, stable server UUID, complete key, and fingerprint
in a private descriptor-relative record; an existing binding cannot be
silently replaced or reused for another origin. Custom acquisition rechecks
discovery and catalog state around transfer, independently verifies the
operator and publisher evidence, and writes a source-specific mount. Removal
is refused until the profile's custom mounts have been removed. The Games UI
keeps the unvetted warning, operator identity, and key fingerprint visible and
keeps install/play disabled until the exact pin is current.

A profile can retain
up to 128 records keyed by server identity/origin, game, archive digest, and
admission revision, so installing an old session pin does not replace another
release of the same game. Each mount records its exact source-specific trust
identity and policy evidence; removal deletes only that exact
profile pointer and leaves authoritative game state unchanged. Session pinning
is deliberately separate from the profile mount: an eligible new session stores
one immutable current release and admission revision, while legacy or
ineligible sessions remain honestly unbound. Later catalog selection never
repins the session. The companion launches only when that server origin/UUID,
source trust, mount identity, digest, revision, signed policy, and
authoritative view agree; actions still travel only through the selected
OmarchyGS server.

## Implemented session launch and action path

The participant-visible session projection now includes either no presentation
or one exact `omarchygs.session-cartridge/v1` binding. It exposes the stable
publisher/game/rules/cartridge identity, archive and signed-identity digests,
pinned admission revision, source provenance, current lifecycle, and
active-session decision. Custom bindings also retain the public operator name,
key fingerprint, and mandatory unvetted warning, but no binding exposes
marketplace keys, filesystem paths, provider endpoints, grants, or credentials.
Suspended and revoked presentation authority fails closed;
deprecated and retired releases follow their signed active-session policy.

For a continuing bound session, `GameController` asks the authenticated
same-user companion to compile the authoritative view. If its exact mount is
absent, trusted QML exposes an explicit install control only when historical
acquisition, the helper credential, and the matching source-specific client
trust are all
available. The companion reads the participant-visible session before and
after acquisition, verifies the returned historical evidence and current
policy evidence against its exact client-authorized key and immutable
binding, and refuses changed admission or lifecycle state. It then
canonicalizes the selected origin, resolves only the exact source-specific
profile mount under client-controlled trust and cached publisher identity,
and runs
the production Rich-2D renderer on one requested signed screen. It returns the
inert plan plus the accepted screen ID, signed entry ID, exact local navigation
map, and a random per-plan loopback capability for digest-named PNG/WAV bytes.
Host checking, media allowlists, no-store responses, and plan/count/byte/age
bounds keep that asset authority local and ephemeral. QML independently checks
the exact plan, origin, nodes, preferences, navigation mapping, and aggregate
resource budgets before retaining `acceptedPlan` or instantiating
platform-owned components. Navigation keeps at most sixteen prior screens,
exposes Back and Entry, compiles each destination through the companion, and
makes no gameplay request.

An emitted gameplay Button or Grid action is still unconfirmed. QML sends only session
identity from the path, expected revision, pinned archive digest, declared
screen, action, shaped payload, and idempotency identity to OmarchyGS. Reserved
`navigate.<screen>` actions never enter this transport. The server
reauthorizes the participant, locks the lifecycle snapshot, re-verifies the
exact signed current-screen contract, translates the action itself, and records
one immutable admission before invoking the existing compiled or provider
command path. Exact replay uses the stored admitted command even after a later
suspension, while a fresh post-transition action is denied. No cartridge gets a
server credential, provider address, socket, filesystem path, or executable
frontend authority.

Door Legends is the first complete portable proof: its provider owns rules and
private state, its independently signed cartridge v2 declares cyclic Lobby and
Chronicle screens plus the same real `enter` action from either screen, and
OmarchyGS owns authentication, session envelope, historical presentation
acquisition, host navigation, action brokerage, projections, and recovery.
Acquisition remains explicit rather than automatic. Arbitrary publisher code,
direct client-provider networking, external-provider onboarding, and general
server plugins remain later designs.

Those are four distinct trust decisions: a publisher signature proves origin
and unchanged bytes, marketplace review records that marketplace's assessment,
server admission records the local operator's catalog decision, and the player
client independently chooses a marketplace key/root-channel or an exact
server-scoped operator-custom key pin. The selected server cannot replace that
client trust anchor, and marketplace publication cannot force server
admission. A server may admit an
`operator-custom` cartridge with no marketplace-review claim, but that changes
provenance rather than containment: the package remains signed, inert, bounded,
schema-checked, content-addressed, and rendered only through trusted QML. A
custom server cannot turn a catalog entry into publisher QML, JavaScript,
native client code, Web content, credentials, an arbitrary URL, or direct
client-provider networking.

Backend code is not part of the cartridge. Portable game rules use a separately
deployed registered provider. The public Provider SDK supplies exact-v1
negotiation and signing/grant helpers; the public starter owns the fixed
compatibility/launch/command/reconcile surface, provider-side PostgreSQL
identity, sessions, one-use grants, operation receipts, and callback outbox.
An embedded `ProviderGame` receives no transport, signing, database, callback,
account/persona, or platform-credential authority and implements only
deterministic launch, command, view, and optional-event logic. The separate
conformance package supplies the fixed fifteen-case TLS/fault corpus and signed
developer-kit export. It defaults to Relay Forge but may substitute one bounded
gameplay profile containing a launch payload, retry-safe timeout command,
finite continuation, and active or completed final status. That substitution
changes no transport, authentication, replay, fault, callback, reconciliation,
or receipt assertion. The reviewed sidecar profile maps only one exact
registered release to one exact nonzero loopback socket. Its canonical DNS URL,
SNI, Host, registered roots, signed authority, protocol, grants, quotas,
lifecycle, and replay contract remain unchanged. The deployment runbook and
templates require separate process, OS identity, PostgreSQL role/database,
secrets, writable state, backup, and lifecycle boundaries.
General server modules form a third extension family. Production admits exact
reviewed or operator-custom no-WASI Component Model releases into dedicated
contained host processes, with typed hooks and intents, core-owned
state/lifecycle, and core reauthorization of every protected effect. No
dynamic in-process Rust plugin ABI exists. A module cannot supply client QML or
become a game's second rules authority. See [Server modules](server-modules.md).

## Package and presentation trust

Production v1 is a canonical stored-only ZIP containing an exact manifest,
domain-separated Ed25519 integrity envelope, declarative presentation, schemas,
localization, and declared assets. It rejects non-canonical archive metadata,
compression, traversal, links, duplicates, undeclared files, unsupported media,
and bounded-resource violations, then reconstructs the archive and requires
byte-for-byte equality. Compatibility is a separate result: unsupported SDK,
protocol, or required capabilities make a valid artifact non-launchable, while
each optional capability selects its signed typed fallback.

The production v1 vocabulary is intentionally bounded. Core supplies
`terminal`, `grid`, `status`, `button`, `image`, and `meter`; Rich-2D adds
`sprite`, `particle_field`, and `audio_cue`. Every screen pins a declared local
JSON Schema, every node requires its exact host capability, Grid actions emit
exactly `column` and `row`, Button actions emit an empty object, and media stays
within strict 8-bit PNG and PCM WAV. The package CLI can generate publisher
keys, pack, conform, install, and revoke without HTTP, database,
platform-credential, QML, or dynamic-loader dependencies.

`presentation.navigation.v1` adds no executable vocabulary. It reserves only
`navigate.<screen_id>` with an empty payload, requires an existing signed target
and one unique Button emitter, and rejects Grids, malformed reserved actions,
missing targets, and gameplay interpretation. Signed cycles are valid because
the host owns the bounded navigation history and each destination is compiled
through the same verifier/renderer boundary.

The Ticket 015 store writes the exact verified archive to a content-addressed
read-only blob and atomically publishes allowlisted activation and revocation
records. Resolve re-verifies the blob and treats malformed or inaccessible
revocation state as denial. That original store remains a same-user developer
boundary.

Ticket 017 adds a Linux secure importer for a lower-trust cooperating game
process. It retains no-follow descriptors for the root and fixed children,
requires every directory to be owned by the effective user and not writable by
group or other, and performs blob, release, conformance, policy, and activation
I/O relative to those descriptors. Policy transitions take an exclusive lock,
reject rollback or conflicting same-version bytes, and persist an authenticated
newer policy before enforcing a denial. This closes pathname-swap and
cooperating-writer races, but the exact store UID remains the local authority;
a future privileged or shared launcher still needs a dedicated service identity
or equivalent external monotonic authority.

### Implemented public trust and package channel

`omarchygs.marketplace-trust-channel/v2` is a host-distribution contract, not
part of the Game Cartridge SDK. Its canonical signed payload binds one offline
root and stable channel to a marketplace origin and authority, a validity
window, strictly increasing bundle version, exact current marketplace snapshot
version, ordered marketplace-key history, and bounded native package artifacts.
The packaged bootstrap contains only the public root, fixed channel location,
platform identity, installed package version, and minimum acceptable bundle
and snapshot versions. Those floors let a first-run or cache-cleared client
reject an older still-valid bundle that predates a packaged revocation.

Exactly one final key is `active` and may authenticate the declared current
marketplace snapshot. Earlier keys are `retired` for only their closed
historical snapshot ranges or `revoked` for no use. A newer trust bundle must
retain complete prior key identity and ranges, preserve the root, channel,
origins, and authority, and make only monotonic active-to-retired/revoked or
retired-to-revoked transitions. The same constraints protect PostgreSQL server
trust and the client's private descriptor-bound trust store. A persisted bundle
below a newly packaged minimum is not usable, but its authenticated bytes still
constrain the next transition so terminal history cannot be erased.

Acquisition v2 carries two independently authenticated facts when rotation
requires them: the retained snapshot that established the immutable release
may be signed by an eligible retired key, while current lifecycle policy must
come from the separately signed current snapshot and active key. Version 1
remains compatible when evidence and policy use the same key. Neither form lets
the selected game server choose the client's trust root or channel.

Root-signed package records bind platform, architecture, version, filename,
size, SHA-256, source revision/digest, and build-provenance digest. The
companion selects only a newer exact artifact for its packaged platform,
streams it through guarded HTTPS into bounded mode-0600 same-user staging,
rechecks current trust before publication, and returns a fixed-path
`pacman -U` command as text. It never invokes a shell, package manager, sudo, or
other privileged installer. Staging authenticates provenance and bytes; it is
not a claim that a hosted marketplace service or malware-review operation
exists.

### Implemented static marketplace publication

`omarchygs-marketplace-publisher` composes the existing release, catalog,
trust-channel, and package contracts without adding a consumer protocol or SDK
surface. `prepare` reads an owner-private input workspace and explicit catalog
key, verifies the supported SDK and every exact release, signs lifecycle policy
and one catalog snapshot, snapshots package bytes, and emits only public
prepared state plus a canonical offline request. `offline-sign` accepts that
request and an explicit owner-only root key, independently validates the root,
validity window, key history, package inventory, snapshot ownership, and prior
transition, and emits a request-bound public signed response without network
work.

`finalize` re-verifies the entire handoff and authentic chain before creating a
private temporary tree. The selected static layout contains identical
`publication.json` manifests beneath `channel/` and `marketplace/`, exact
`trust.signed.json` and native packages in the former, and the signed snapshot
plus each exact release triple in the latter. Complete trees are renamed into
immutable bundle-and-manifest-digest versions; one cross-process lock and a
validated relative `current` link serialize monotonic activation. Local
verification rejects extra, missing, linked, mutable-mode, oversized, stale,
or digest-divergent state.

The hosted probe reuses the trust channel's guarded HTTPS transport, requires
operator-held minimum bundle and snapshot versions plus an optional exact
publication digest, streams bounded package bodies, authenticates every
root/catalog/publisher claim and artifact, and requires all supplied mirrors to
serve one identity. Mirrors add availability only. Real roots, HSM/media
custody, public origins, CDN/object-store rollout, staffing, monitoring, and
incident coordination remain external deployment work.

### Implemented marketplace and server catalog flow

The marketplace snapshot is bounded canonical JSON under its own signature
domain. Strict Ed25519 verification binds the manual authority or the public
channel's exact active key, a nonzero
monotonic snapshot version, bounded review facts, exact publisher/release
identities, signed lifecycle policy, and unique sorted relative release
directories. The snapshot and cartridge data cannot choose a host, scheme,
port, absolute path, query, fragment, or redirect target.

The server constructs a fresh guarded HTTPS client for the configured origin.
It rejects the complete DNS answer set if any address is private, local,
special-use, documentation, multicast, or reserved; pins accepted sockets while
retaining hostname verification; trusts only the configured DER root; and
disables ambient proxies, redirects, referer, decompression, connection reuse,
and unbounded response bodies. Tests alone can admit one exact generated
loopback socket for the separately spawned TLS fixture.

Synchronization first rejects stale or conflicting snapshot identity and, in
channel mode, any version other than the root-declared current snapshot. It then
downloads each release's existing `cartridge.ogsc`, `conformance.json`, and
`release.signed.json` beneath its relative directory. The production release
verifier reconstructs publisher and conformance identity before
`SecureCartridgeStore::stage_reviewed_release` retains immutable content without
writing the legacy active pointer. Immutable unreferenced bytes may remain
after a later failure, but the current reviewed inventory advances only after
all entries are staged and one serialized PostgreSQL transaction publishes the
complete snapshot. Omitted releases remain historical and ineffective.
PostgreSQL also persists the authenticated root fingerprint and complete trust
payload plus each release policy's exact marketplace key and snapshot version.
Security-sensitive live requests compare their runtime trust to that persisted
state, so a separate administrator process's newer rotation or revocation takes
effect without waiting for server restart.

Marketplace lifecycle, operator-custom lifecycle, and server admission are
separate facts. A local
`catalog-apply` command carries an idempotency UUID, exact expected selection,
exact desired source and digest, actor, and reason. It serializes one game,
resolves the desired current permitted source through the secure store, updates
at most one selected release, increments the admission revision, and appends
its immutable source transition in the same transaction. Lifecycle suspension,
removal, incompatibility, or local deactivation immediately hides the
selection without falling back to another digest or source. A newer
authenticated policy can make the same retained exact selection effective
again; choosing another release or rolling back remains explicit.

Admin-only custom import snapshots the publisher key and three fixed release
components once, verifies the production SDK/host contract, signs the
server-scoped attestation and initial policy, and publishes immutable authority,
release, and audit state. Policy updates are monotonic and take the global
exclusive lifecycle lock before the per-game lock. Fresh cartridge actions use
the same global domain in shared mode, so a queued suspension or revocation
commits before a later admission; an exact already-admitted action still
replays from its immutable receipt.

Public discovery advertises `games.cartridge-catalog.v1` and adds
`games.operator-custom-cartridges.v1` plus a bounded public authority candidate
only when custom distribution is valid. The authenticated
`GET /v1/cartridges` endpoint returns only current imported, compatible,
selected `active` or `deprecated` releases as an exact
`marketplace_vetted`/`operator_custom` union with source-appropriate public
provenance, exact digests, admission revision, and warning. It exposes no
acquisition URL, local path, public-key material, raw signed record, operator
reason, executable document, or alternate release fallback.

Cartridges cannot contain or invoke publisher QML, JavaScript, native code,
shell commands, arbitrary shaders, imports, dynamic remote assets, filesystem
paths, clipboard/process access, or network clients. The trusted renderer
interprets only versioned node and action records. Provider-returned data must
match the screen's pinned view schema, and text is rendered literally rather
than as automatic rich markup. Authentication and MFA remain reserved,
unspoofable platform surfaces.

### Implemented trusted renderer path

The renderer accepts only the verifier's externally immutable
`VerifiedCartridge`. For a ready exact requested screen—or the signed entry
screen when none is requested—it reads the authenticated pinned
schema, validates one bounded view, resolves only dotted bindings and declared
actions/assets, applies signed optional fallbacks plus trusted scale, contrast,
reduced-motion, and mute preferences, and emits
`omarchygs.render-plan/v1` plus authenticated current-screen, entry-screen, and
local navigation metadata. Non-ready loading, offline, stale, empty, protocol,
unsupported-capability, and revoked states use fixed platform messages and
contain zero cartridge nodes.

Resource admission is incremental. Core/Rich-2D count retained plan bytes,
nodes, grid cells, images, sprites, particles, audio cues, and animations before
keeping each node. Core also limits a referenced raster to 1,024 px per side,
1 MP, and 4 MiB decoded, with 16 MiB decoded across the scene; Rich-2D permits
2,048 px, 4 MP, and 16 MiB per raster, with 64 MiB across the scene. Raster
admission occurs before a node or asset is published. Authenticated asset
digests are cached once per package path; bytes publish once only after a
reference passes admission. The QML surface then independently validates exact
keys, digest tokens, per-node types, and aggregate profile totals before a fixed
switch instantiates repository-owned Components. Image decoding is asynchronous
and requests at most 2,048 px in either dimension. Cartridge strings always use
`Text.PlainText`.

The preview CLI runs that same verifier/compiler over bounded regular files and
requires an existing empty private output directory. It writes one read-only
plan and read-only digest-named assets and reports that no provider, database,
or platform credential was used. This is a same-user developer path, not the
main-client launcher or a privileged multi-user sandbox.

## Authority and provider flow

| Surface | Durable authority |
|---|---|
| Accounts, sessions, MFA, personas/avatar projections, social state | OmarchyGS |
| Catalog, launch policy, provider registration/revocation, audit | OmarchyGS |
| Platform session envelope, participants, pinned identities, accepted result receipts | OmarchyGS |
| Game rules, private gameplay state, turn/time/randomness, provider revision in the Door Legends remote mode | Exactly one registered provider |
| Rendering, input, accessibility, theme, local cosmetic animation | Trusted OmarchyGS client |
| Durable client recovery | OmarchyGS REST/cursor feed; WebSockets remain hints |

A valid publisher signature authenticates package identity and bytes; it does
not make the publisher trusted for memory, CPU, action shape, or UI authority.
OmarchyGS retains every executable QML component, trusted preference,
origin/failure surface, and server-authorized action dispatcher.

The implemented remote-provider pilot preserves the intended flow: the client
calls authenticated OmarchyGS APIs, and an OmarchyGS-only broker resolves the
operator-registered Door Legends destination. Before any grant or gameplay
effect, the broker signs an exact compatibility offer and accepts only the
provider-signed protocol-v1 selection containing launch, command, reconcile,
and event. The resulting short-lived grant binds that selection alongside the
provider audience, exact release/game/rules/cartridge identities, platform
session, one scope, a pairwise provider/game persona subject, expiry, and replay
ID. Account
identity, raw persona identity, reusable device-session credentials, and
database access never cross the boundary.

The compatibility result also binds the release configuration revision and the
message key that authenticated it. Grant issuance and final durable-attempt
creation reload and lock current provider policy, lifecycle, scope, revision,
and key material; an intervening operator change fails closed before outbound
operation I/O. Compatibility, grant preparation, and operation transport share
one aggregate deadline shorter than the PostgreSQL concurrency lease.

A player action carries one durable idempotency key and expected provider
revision. A timeout means unknown outcome, so retries retain both values while
using a fresh grant. A changed replay is a conflict; a stale revision requires
an explicit refresh rather than a silent rebase. Signed provider events use
stable IDs and are deduplicated before achievements, results, notifications,
or sync projections are applied. Provider outage may expose the last validated
view as stale/read-only, but OmarchyGS never invents a move or result.

Commands and explicit reconciliation additionally claim one bounded durable
session reservation. A transaction-scoped PostgreSQL advisory fence is held
across broker I/O, then the reservation UUID is revalidated before the
authenticated response can project. Competing work is rejected locally; an
expired abandoned reservation first moves a formerly ready session into
reconciliation, while the advisory fence prevents a still-live operation from
being reclaimed. Failure cleanup and response projection preserve a newer
operator suspension or retirement.

Ticket 018 persists immutable registered releases, lifecycle scopes,
append-only message-signing and TLS keys, grants, quota windows, concurrency
leases, operation attempts, authenticated callback receipts, and safe audit
events in PostgreSQL. Requests, responses, and callbacks use a fixed signed
message profile over the exact body and context. Provider and marketplace
production egress share the conservative public-unicast classifier. Each
accepts only its operator-pinned HTTPS DNS origin, rejects the complete answer
set when any address is not public, pins accepted sockets while retaining
hostname verification, trusts only its explicit roots, and disables proxies,
redirects, decompression, and unbounded responses. The compile-time conformance
modes admit one exact generated loopback socket; they cannot create a
production private-network allowlist.

Ticket 019 adds the player-facing authority bridge without creating a second
gameplay owner. Migration 0015 makes `platform_compiled` sessions require local
object state and no provider release, while `registered_provider` sessions
require an exact release pin, explicit availability, and null local rules
state. Door Legends launch first persists the platform envelope, participant,
start receipt, and sync invalidation, then performs network I/O. Commands and
explicit reconciliation reuse a stable idempotency key and expected provider
revision; no session transaction remains open across the provider call.

Only authenticated bounded provider views are returned to the cartridge. The
platform requires a non-empty object, applies the public SDK's key, value,
depth, cardinality, integer, control-character, and credential-shape rules, and
caps the serialized view at 64 KiB. This protocol-safety gate is game-neutral;
the authenticated signed screen schema remains responsible for the exact
presentation shape. A callback signature is authenticated before its session
fields are used, then current policy and the durable receipt gate the
transaction that records allowlisted results, achievement awards, audit, and
persona-sync effects. A
pre-negotiation persisted callback can be upgraded only after its exact
immutable receipt identity and body digest prove it is a duplicate; a fresh
legacy-shaped callback rejects. Suspension removes the pilot from new discovery
and denies launches, commands, and callbacks while preserving read-only views
and reconciliation.
Restoration requires authenticated reconciliation; retirement is terminal.
Unknown outcomes and outages never trigger a compiled failback.

## Graphics envelope

The safe ceiling is set by the reviewed host vocabulary and resource budgets,
not by every feature Qt or the GPU could execute.

| Profile | Intended range | Examples | Boundary |
|---|---|---|---|
| Cartridge Core | Terminal text, panels, menus, forms, lists, grids, boards, images, focus, state surfaces, simple transitions | Classic BBS games, interactive fiction, trivia, scoreboards | No arbitrary code, drawing, remote assets, shaders, video, or 3D |
| Rich 2D | Tile maps, sprites, cards, tactical boards, vector primitives, meters, local timelines, particles, platform effects, bounded audio/music | Roguelikes, asynchronous RPGs, strategy/management, puzzles, visual novels, polished retro games | Provider updates are action/state paced; host nodes animate locally |
| Advanced 2D/2.5D | Larger scrolling scenes, approved host primitives, bounded video and richer post-processing | Isometric tactics, animated maps, arcade-like presentation, cut scenes | Optional hardware profile and separate capability review |
| Future constrained 3D | Validated models and a host-owned scene schema through optional Qt Quick 3D | Turn-based 3D boards, simple dungeon scenes, model viewers | Separate dependency, licensing, GPU, asset, and threat gates |
| Isolated Web experience | Compatibility surface for games outside the DSL | Provider web applications | Larger Chromium/origin/permission surface; never the default cartridge path |

Core plus Rich 2D can go well beyond a text BBS: rich card and board games,
roguelikes, asynchronous RPGs, tactical maps, animated management games,
puzzles, visual novels, and elaborate retro successors are realistic targets.
The design deliberately excludes Halo-class first-person rendering,
high-frequency physics, competitive twitch networking, arbitrary publisher
rendering code, and a general Unity or Unreal runtime.

Local cosmetic animation can run at display rate without a provider round trip.
Meaningful state changes wait for the authoritative rules owner. Each host
advertises presentation capabilities and resource limits; required unsupported
capabilities fail clearly, while optional effects declare static,
reduced-motion, muted, software-rendered, or simpler-node fallbacks.

The delivery stages keep that ambition honest:

| Stage | Available | Deliberately absent |
|---|---|---|
| Ticket 015 contract | Signed inert Terminal/Grid/Status data, strict PNG/PCM WAV, compatibility and local install | No production renderer, sprites, particles, provider network, or gameplay authority |
| Ticket 016 renderer | Measured Core plus Rich-2D host components, local effects/audio, accessibility and previewer | No publisher QML/JS, custom shader code, WebEngine, video, or 3D |
| Later reviewed profiles | Advanced 2D/2.5D or constrained 3D host capabilities after separate reviews | No general engine or arbitrary third-party execution |

The implemented v1 profile ceilings are:

| Resource | Core | Rich-2D |
|---|---:|---:|
| View / render plan | 256 KiB / 1 MiB | 512 KiB / 2 MiB |
| Nodes / grid cells | 256 / 1,024 | 512 / 4,096 |
| Images / sprites / particles / audio | 32 / 0 / 0 / 0 | 64 / 128 / 2,048 / 16 |
| Simultaneous animations | 32 | 128 |
| Raster side / pixels / decoded bytes | 1,024 px / 1 MP / 4 MiB | 2,048 px / 4 MP / 16 MiB |
| Referenced decoded raster per scene | 16 MiB | 64 MiB |
| Surface RSS soft / hard | 256 / 384 MiB | 384 / 512 MiB |
| Software frame average | 16.67 ms target; 33.3 ms gate ceiling | Same |

## Renderer and provider evidence

`scripts/test-game-cartridge-renderer.sh` generates real signed base, Core, and
Rich-2D packages and prepares them through the production CLI under unusable
database, credential, and proxy settings. It runs Qt 6.11.2 at 920×600 with the
offscreen software backend and one-CPU affinity when available, warms 60
frames, samples 120, measures peak RSS, exercises keyboard focus/actions and
accessibility preferences, visits every fixed state, and proves a QML plan over
its claimed aggregate profile is rejected.

The final constrained green run rendered Core's stress scene at 15.998 ms
average / 16.335 ms maximum and 132,688 KiB peak RSS. Rich-2D measured 16.000 /
18.668 ms and 244,664 KiB. The largest accepted 2,048-pixel Rich-2D raster
measured 16.006 / 16.623 ms and 250,312 KiB, while a 2× high-contrast,
reduced-motion, muted run measured 16.001 / 16.726 ms and 237,864 KiB. The same
harness rejects a 4,096-pixel raster before a render plan is published. These
are exact local reference-host observations, not universal device performance
promises. Run the production profile evidence with:

```bash
scripts/test-game-cartridge-renderer.sh
```

The nested `crates/game-cartridge-spike` workspace proves a deliberately small
slice:

- Ed25519 signing and verification over a strict integrity index;
- bounded package paths, files, bytes, presentation nodes, views, and messages;
- three trusted node types—`terminal`, `grid`, and `status`—with keyboard,
  loading, offline, protocol-error, accessibility, and local-animation states;
- a loopback broker issuing 60-second exact-scope pairwise grants;
- a separate provider owning revision zero and one idempotent command; and
- signed result validation, duplicate-event rejection, privacy assertions, and
  retry of the same idempotency key.

The final diff-gate sample rendered 120 software-backend frames at 15.99 ms
average and 17.00 ms maximum, used 88,184 KiB peak QML RSS, and verified a
four-file, 2,436-byte expanded signed fixture. Proof enforcement is 32 files,
256 KiB per file, 1 MiB total, 8 screens, 128 nodes, a 16×16 grid, a 64 KiB
view, and a 128 KiB provider body.

Those Ticket 014 values validate the remote-provider proof harness, not the
production renderer profile above.

Run the proof directly with:

```bash
scripts/test-game-cartridge-spike.sh
```

The production renderer is gate 12, the Game Cartridge SDK/release/import proof
is gate 13, the Provider SDK deterministic release is gate 13a, the public
provider starter developer kit and clean-room Relay Forge build are gate 13b,
and the isolated provider proof is gate 14 in every `bin/gate.sh` mode. Gate 13a
packages only `omarchygs-provider-sdk`, rejects platform paths and dependencies,
and requires two clean consumer clones to produce identical signed exports:

```bash
scripts/test-provider-sdk.sh
```

Gate 13b packages the SDK, starter, and conformance crates twice, rejects
repository-path and private-platform dependencies, builds Relay Forge twice
from clean Git clones, and compares two signed developer-kit exports without
source-path, credential, or platform-identity leakage:

```bash
scripts/test-provider-developer-kit.sh
```

Tickets 048 through 058 provide a development-only second consumer shape outside
the platform repository: a persistent Usurper provider supplies its bounded
gameplay profile, runs the same fifteen cases twice across process restart, and
renders seventeen signed inert screens through the production preview boundary.
Rules v4 includes player-private pack/equipment, shops and haggling, bank and
chest transfers, healing-potion purchases, equipment-aware combat, and
configured quick-heal-then-attack turns, plus three class-specific level-one
spells with mana, resistance, temporary Fog absorption, encounter reset, and
same-turn monster response. Rules v5 extends that proof with source-faithful
Assassin Backstab and Paladin Soul Strike combat branches selected from current
provider state through one inert action. Rules v6 adds passive Gnoll poison;
rules v7 adds the exact bounded level-two dungeon band, rules v8 adds the exact
bounded level-three band, rules v9 adds the exact bounded level-four band, and
rules v10 adds the exact bounded level-five band. Rules v11 adds the exact
bounded level-six band.
All retain the original encounter rejection loop, level-aware combat, and inert
level controls across selectable levels one through six. Level 6 retains
boundary record 50 as source data, accepts records 51 through 59, and initializes
combat at strength 16, defence 8, and 48 HP. That proof is not
a production registration,
server admission, marketplace release, deployment, shared-realm state, or
additional platform gameplay authority.

In diff/full modes, gate 19 first exercises starter persistence and the real
broker against a distinct provider database, then runs the complete fifteen-case
TLS conformance corpus twice across provider restart before continuing through
the production provider-security boundary against migrated PostgreSQL:

```bash
scripts/test-provider-conformance.sh
```

Gate 19a then exercises the production sidecar profile against a clean-room
provider and independent PostgreSQL database. It proves exact TLS/socket/release
binding, hostile local-peer rejection, crash denial, restart/reconciliation,
callback recovery, separate backup/restore, hardened service/proxy templates,
and a locally signed bounded receipt containing no credentials or database URL:

```bash
scripts/test-provider-sidecar.sh
```

Gate 20 then packages the Provider SDK, builds Door Legends from a clean clone
without platform-only features, runs it as a separate TLS process
against its own PostgreSQL database, drives catalog/start/command/reconcile and
callback projection through the real server bridge, exercises lifecycle and
failure recovery, and restores the provider backup into a second database:

```bash
scripts/test-provider-authority-pilot.sh
```

Ticket 032 adds a real generated TLS marketplace fixture and migrated
PostgreSQL lifecycle path. It proves pinned-root enforcement, redirect and body
ceilings, signed snapshot replay/downgrade behavior, exact release staging,
concurrent expected-state admission, rollback, lifecycle denial with no
fallback, authenticated metadata filtering, operator CLI output, immutable
audit, and isolated backup/restore. The portable package tests also cover
canonical signature/schema rejection and the shared reserved-address egress
corpus. Run the complete database portion with:

```bash
scripts/test-database.sh
scripts/test-operator-recovery.sh
```

Ticket 035 extends that evidence with immutable snapshot replay/omission tests,
participant-private historical acquisition after catalog advancement, exact
multi-release cache and requested-screen compilation cases, malformed and
duplicate navigation rejection, zero-network QML Back/Entry/history behavior,
screen-bound gameplay replay, and the clean-clone Door Legends Lobby ↔
Chronicle pilot. The QML production-root fixture contains 47 passing cases.

See [Development and validation](development-and-validation.md) for the full
gate and failure routing.

## Staged SDK and rollout

1. Ticket 015 implements the versioned package/schema contract, verifier,
   conformance CLI, compatibility report, and same-user local store.
2. Ticket 016 implements the trusted Core/Rich-2D renderer and previewer and
   ratifies the first local software-rendered stress profile.
3. Ticket 017 implements and proves the deterministic SDK/release workflow,
   signed lifecycle policy, secure local import, and separate-repository
   first-party cartridge consumption while rules remain compiled and
   platform-authoritative.
4. Challenges and the first playable use those stable seams without waiting
   for remote hosting.
5. Ticket 018 implements production provider registration,
   grants/message security, guarded egress, quotas, replay state, audit, and
   revocation before connecting it to player routes.
6. Ticket 019 implements one first-party Door Legends remote-authority pilot
   and the required Constitution §10 amendment. External providers wait for a
   separate onboarding, operations, transparency, and support pipeline.
7. Tickets 031 and 032 implement stable server identity/profiles followed by
   one pinned marketplace, reviewed staging/inventory, independent audited
   server admission, and authenticated catalog metadata.
8. Ticket 033 implements exact server distribution, independent client trust,
   private cache, and server-profile mounts without weakening trusted rendering.
9. Ticket 034 binds exact admitted releases to eligible sessions, prepares the
   trusted mounted render plan, launches it inside the platform shell, and
   returns declared actions through durable server authorization.
10. Ticket 035 retains exact historical marketplace evidence, acquires an old
   session pin without consulting current selection, mounts exact releases side
   by side, and adds signed host-local multi-screen navigation with screen-bound
   action authorization.
11. Ticket 036 adds the offline-root-signed public trust channel, monotonic
   marketplace-key rotation and revocation, acquisition v2's separate evidence
   and current-policy keys, and bounded native package staging without
   privileged installation.
12. Ticket 037 adds deterministic static publication, a public offline-root
   handoff, immutable activation, exact local and hosted verification, mirror
   comparison, and catalog-compromise/rollback rehearsal.
13. Ticket 038 adds explicitly labeled operator-custom cartridge trust,
   admin-only import/lifecycle, exact source-aware admission/history, explicit
   client key pins, and persistent player warnings without adding executable
   authority.
14. Ticket 044 extracts the public-only Provider SDK preview, adds authenticated
   exact-v1 compatibility before effects, and proves a deterministic locally
   signed release in two independent clean consumer clones.
15. Ticket 045 adds the public starter/conformance/fault kit, deterministic
   three-package developer-kit release, and the second clean-room Relay Forge
   game, including real-broker, independent-database, restart, callback, replay,
   and unknown-outcome evidence.
16. Ticket 046 adds the reviewed exact-release sidecar/operations profile,
   hardened deployment templates, independent recovery drill, and durable
   cross-process provider-operation fencing. External onboarding still requires
   real provider, marketplace, hosting, custody, review, and support operations.
17. Tickets 048 through 058 exercise a persistent game's bounded conformance
   profile and game-neutral authenticated view projection with a separate local
   Usurper provider, then add a player-private equipment/potion economy,
   configured combat-quaff parity, three level-one caster spells and their mana
   lifecycle, the Assassin Backstab and Paladin Soul Strike combat branches,
   passive Gnoll poison, the exact level-two, level-three, level-four, level-five, and level-six dungeon
   bands and rejection loops, and seventeen inert signed screens without granting
   production
   registration, admission, shared-realm state, deployment, or publication.

First-party games use the same public schemas and conformance suite intended
for later publishers. They may have a higher catalog trust tier, but never a
private database or identity integration path.

The exported v1 SDK is language-neutral and read-only. Its lock pins the SDK,
presentation protocol, package and preview tool versions, file digests, and
compatibility/deprecation/retirement rules. A release directory contains only
`cartridge.ogsc`, `conformance.json`, and `release.signed.json`; the publisher
attestation binds source revision, builder identity and binary digest, exact SDK
identity, publisher/game/version identity, archive digest, and conformance
digest. Signed catalog policy supplies five explicit states—active, deprecated,
suspended, revoked, and retired—with separate new-launch and active-session
decisions and monotonic policy versions.

The separate Provider SDK preview is a no-default-feature Rust crate containing
only provider-facing errors, scopes, pairwise identity, compatibility, grants,
messages, signing/verification, schemas, fixtures, and release helpers. Its
export writes an exact compile-owned finite inventory into an existing empty
directory, pins every byte in a canonical lock, and signs authority, key,
source-revision, builder, and lock identity with a domain-separated Ed25519
envelope. Verification rejects unknown files or directories, symlinks,
non-native aliases, excessive depth/breadth/path bytes, byte drift, and
provenance or signature mismatch. The preview is locally signed, marked
non-publishable, and grants no license, registration, activation, or discovery
authority.

## Change map

| Intent | Read/change first | Required evidence |
|---|---|---|
| Package, signing, and capability contract | `crates/game-cartridge`; ADR-0002; `docs/architecture/game-cartridges.md`; Ticket 015 | `scripts/test-game-cartridge.sh`; deterministic fixtures, malformed package and resource-limit matrix, signature/capability/revocation checks |
| Trusted renderer and graphics profile | `crates/game-cartridge-renderer`; `client/qml/cartridge`; Ticket 016 | `scripts/test-game-cartridge-renderer.sh`; schema/action/resource rejection, keyboard/accessibility/fixed states, and constrained Core/Rich-2D measurements |
| Separate-repository SDK/release | `crates/game-cartridge/src/sdk.rs`, `release.rs`, `lifecycle.rs`, `secure_store.rs`; Ticket 017 | `scripts/test-game-cartridge-sdk.sh`; deterministic export, clean-clone reproducibility, signed provenance/policy, lifecycle matrix, descriptor-relative import, rollback/race/permission rejection |
| Public Provider SDK preview and negotiation | `crates/provider-sdk`; `crates/game-provider/src/broker.rs`, `registry.rs`; `docs/operators/provider-security.md`; Ticket 044 | `scripts/test-provider-sdk.sh`; `scripts/test-provider-conformance.sh`; exact package/inventory and two-clone release proof, compatibility downgrade/stripping denial, stale-material and aggregate-deadline races, strict network parsing, and exact historical duplicate recovery |
| Public provider starter, conformance, and clean-room game | `crates/provider-starter`; `crates/provider-conformance`; `examples/provider-relay-forge`; Tickets 045 and 048 | `scripts/test-provider-developer-kit.sh`; `scripts/test-provider-starter-conformance.sh`; deterministic three-package export, private-dependency denial, two clean-clone builds, provider-side PostgreSQL persistence/restart, real-broker integration, exact TLS binding, fixed fifteen-case corpus with bounded game profiles, callback recovery, and replay |
| Reviewed provider sidecar and operations | `crates/game-provider/src/egress.rs`; `crates/server/src/provider_games.rs`; migration `0029`; `deploy/provider-sidecar`; `docs/operators/provider-deployment.md`; Ticket 046 | `scripts/test-provider-sidecar.sh`; `scripts/test-provider-authority-pilot.sh`; exact release/socket/TLS identity, hostile peer and ambient proxy denial, durable reservation/advisory fencing, crash/restart/reconcile, lifecycle races, separate database restore, templates, and signed receipt |
| Provider security foundation | `crates/game-provider`; migration `0014_provider_security_foundation.sql`; `docs/operators/provider-security.md`; Ticket 018 | `scripts/test-provider-conformance.sh`; TLS and sender authentication, public-only pinned egress, grant/replay/key/quota/lease/audit, lifecycle, race, and failure tests |
| Remote authority migration | Constitution §10; ADR-0002; migration `0015`; `crates/server/src/provider_games.rs`; Ticket 019 | `scripts/test-provider-authority-pilot.sh`; one durable gameplay owner, exact replay/reconciliation, callback projection, lifecycle, independent database and restore evidence |
| Marketplace synchronization and server admission | `marketplace.rs`, `marketplace_egress.rs`, `marketplace_sync.rs`, `cartridge_catalog.rs`; migration `0019`; administrator CLI and authenticated catalog route; Ticket 032 | Canonical signature hostile corpus; real TLS root/redirect/size tests; PostgreSQL replay/race/rollback/lifecycle tests; exact API and CLI fixtures; recovery rehearsal; security and authority review |
| Player acquisition, cache, and mounts | `acquisition.rs`; `cartridge_distribution.rs`; `crates/client-cartridge-runtime`; `CartridgeController.qml`; migration `0020`; launcher/package scripts; Ticket 033 | Exact-selection/no-fallback PostgreSQL tests; hostile acquisition corpus; descriptor-relative permission/race tests; independent-key substitution denial; catalog-only QML compatibility; reproducible native package and cleanup smoke |
| Session pinning, trusted launch, and cartridge actions | `session_cartridges.rs`; migration `0021`; `crates/client-cartridge-runtime/src/render.rs`; `service.rs`; `GameController.qml`; `GameplayScreen.qml`; Ticket 034 | Immutable pin and admission tests; hostile origin/mount/lifecycle/action corpus; renderer/QML harness; clean-clone Door Legends authority pilot; canonical diff gate |
| Historical session acquisition and signed-screen navigation | retained evidence in `cartridge_catalog.rs`; `cartridge_distribution.rs::acquire_session_exact`; migration `0022`; client runtime remote/cache/render/service modules; navigation validator/renderer; `GameController.qml`; Door Legends cartridge v2; Ticket 035 | Snapshot replay/omission immutability; participant privacy and catalog-independent exact acquisition; key/binding/lifecycle substitution denial; multi-release mounts; malformed/duplicate navigation; zero-network QML navigation/history; screen-bound gameplay and exact replay; clean-clone historical Door Legends pilot |
| Public trust enrollment, key rotation, and native package staging | `crates/marketplace-trust`; client runtime trust/package modules; server marketplace/catalog/distribution/session modules; `MarketplaceController.qml`; migration `0023`; packaging scripts; Ticket 036 | Root/channel canonical and transition corpus; fresh-enrollment and floor-advance replay denial; live client/server revocation; acquisition-v2 dual-key verification; historical migration upgrade; QML trust/package states; deterministic manual/channel packages; root-signed channel gate |
| Static marketplace publication and offline-root operations | `crates/marketplace-publisher`; `docs/operators/marketplace-publication.md`; Ticket 037 | `scripts/test-marketplace-publication.sh`; canonical plan/handoff, release and package verification, network-unshared offline sign, exact immutable tree, concurrency, mirror, rotation, rollback, receipt, and security evidence |
| Operator-custom trust, import, lifecycle, and warnings | `operator_custom.rs` in the cartridge/server domains; `cartridge_catalog.rs`; `cartridge_distribution.rs`; `session_cartridges.rs`; client runtime cache/remote/service; QML cartridge/game controllers and Games screen; migration `0024`; Ticket 038 | Canonical attestation/acquisition hostile corpus; PostgreSQL import/policy/selection/action races and recovery; private exact client trust and source-specific mounts; warning/keyboard QML tests; security diff scan and writer-first linearization regression |
| Server-module architecture and production sequencing | ADR-0004; [Server modules](server-modules.md); `docs/architecture/server-modules.md`; `crates/server-module-spike`; Tickets 039–041 | `scripts/test-server-module-spike.sh`; exact WIT/trust/intents/state evidence; process containment, failure, recovery, and production-loader absence |

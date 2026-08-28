# OmarchyGS Game Cartridges — architecture and delivery plan

Status: the data-only cartridge/provider boundary and one scoped first-party
remote authority pilot were accepted by
[`ADR-0002`](adr-0002-game-cartridge-and-provider-boundary.md) after the
[`TICKET-014`](../planning/tickets/closed/TICKET-014-portable-games-sdk-and-remote-hosting-spike.md)
proof. Ticket 015 implements the local v1 package, verifier, conformance CLI,
and same-user store. Ticket 016 adds the production render-plan compiler, trusted QML
vocabulary, isolated preview command, and measured Core/Rich-2D reference
profiles. Ticket 017 adds the deterministic v1 SDK export, signed release
provenance, first-party clean-room repository proof, signed catalog lifecycle,
and descriptor-relative privileged import boundary. Public Internet publication,
third-party onboarding remains a gated follow-up stage. Ticket 018 adds the
production provider trust/protocol foundation. Ticket 019 connects Door
Legends v1 as the sole operator-enabled remote authority pilot with an
independent provider database, player routes, atomic projections, and tested
recovery; this design never authorizes loading third-party code.
Ticket 032 implements the first marketplace-vetted server half: guarded signed
snapshot synchronization, production release verification, descriptor-relative
staging, atomic reviewed inventory, independent audited local admission, and an
authenticated metadata-only cartridge catalog. Ticket 033 retains the exact
signed snapshot evidence, adds authenticated exact server distribution, and
ships a loopback Rust client companion for independent acquisition verification,
content-addressed cache staging, server-profile mounts, explicit update, and
local removal. Ticket 034 adds immutable exact session-presentation pins,
same-user companion compilation from the matching mount, trusted QML gameplay,
bounded ephemeral asset authority, and durable server-authorized cartridge
actions for both existing rules-authority paths. Door Legends is the first
complete portable playable. Ticket 035 adds immutable historical snapshot and
release evidence, participant-authorized acquisition of an old session pin,
exact multi-release profile mounts, signed host-local multi-screen navigation,
and screen-bound gameplay admission. Acquisition remains explicit rather than
automatic. Ticket 036 adds the separate offline-root-signed marketplace trust
channel, packaged first-enrollment freshness floors, active/retired/revoked key
rotation, persisted client/server revocation, acquisition v2's independent
historical and current-policy proofs, and bounded authenticated native-package
staging without installation authority.
[`ADR-0003`](adr-0003-owner-operated-server-and-extension-boundary.md) now
accepts the owner-operated server, server-curated marketplace, operator-custom
trust, future Provider SDK, and separately gated server module/hook direction.
[`ADR-0004`](adr-0004-process-isolated-wasm-server-modules.md) selects one
no-WASI Component Model release per OS-contained module-host process with typed
hooks/intents and core reauthorization; it leaves production loading disabled.

## Product model

An **OmarchyGS Game Cartridge** is the retro-styled, immutable presentation and
integration package for one exact game release. A player browses the trusted
OmarchyGS catalog, selects a cartridge, and plays inside the consistent
keyboard-first OmarchyGS shell.

That catalog belongs to the owner-operated server the player selected. A
marketplace may vet and distribute an exact release, but each administrator
chooses what to import, activate, suspend, or remove for their community.
Players see that server's library and cache its exact cartridge bytes locally;
marketplace publication does not create a global account, catalog, or launch
authority.

The cartridge supplies the game's identity, declared capabilities, schemas,
screen templates, and static assets. OmarchyGS supplies the executable QML
renderer, navigation, theme, accessibility, platform dialogs, network broker,
and security boundary. A separately deployed registered provider supplies the
server-side rules and gameplay state in remote-provider mode. The cartridge
never contains that backend.

The central rule is:

> Games provide signed presentation data, validated view models, declared
> actions, and assets. OmarchyGS provides the executable frontend.

The cartridge behaves like a ROM from the player's point of view: the catalog
shows cover art and compatibility, installation mounts one exact signed release,
and the same release can move between compatible OmarchyGS installations.
Unlike a historical ROM, it does not contain a machine-code game engine. Saved
games, identity, avatars, achievements, social state, and launch permission live
outside the cartridge, so replacing or uninstalling presentation bytes never
silently replaces or deletes authoritative player state.

Raw game-supplied QML, JavaScript, native libraries, shell commands, and direct
network clients are not cartridges. Qt explicitly assumes QML and JavaScript
are trusted application code and recommends a custom domain-specific language
when content is untrusted.

## System shape

```text
game repository
  ├─ optional provider server and rules
  ├─ SDK schemas and conformance tests
  └─ signed immutable Game Cartridge (frontend data only)
                    │ publish and vet
                    ▼
           OmarchyGS marketplace
                    │ operator imports exact release
                    ▼
       owner-operated OmarchyGS server catalog
                    │ immutable admitted release + retained provenance
                    ▼
          authoritative session presentation pin
                    │ exact digest + admission revision
                    ▼
          player local cartridge cache
                    │ exact multi-release mount + verified inert screen plan
                    ▼
          trusted OmarchyGS QML renderer
                    │ host-local navigation or declared gameplay action
                    ▼
          OmarchyGS authenticated broker
             ├─ compiled game runtime
             └─ short-lived provider-scoped grant
                              │
                              ▼
                     remote game provider
                              │ signed result/event
                              ▼
            OmarchyGS achievements, history, inbox, and sync
```

For the first-party transition, a game may remain a compiled Rust definition
while its source lives in a separate repository and its cartridge already uses
the public presentation contract. A later provider adapter moves gameplay
authority across the network without replacing the player-facing cartridge.

Signal Siege's private-alpha QML gameplay surface is platform-owned trusted
application code and intentionally does not manufacture a cartridge origin,
digest, or `omarchygs.render-plan/v1` document. It reuses inert nodes from the
trusted component vocabulary with a platform-derived view model. The signed
cartridge renderer remains reserved for packages that passed the verifier and
content-addressed installation lifecycle below.

## Execution-model decision

| Model | Gameplay authority | Isolation | Release and compatibility | Latency/offline | Operational cost | Decision |
|---|---|---|---|---|---|---|
| Current compiled Rust definition | OmarchyGS process and PostgreSQL | Strong capability boundary in the Rust trait, but no process boundary | Platform release pins exact compiled key/version | Lowest latency; platform can resume locally | Lowest initially; every game change releases the platform | Retain for the private alpha and deterministic first-game rules |
| Separate-repository compiled first-party artifact | OmarchyGS process after reviewed build/import | Same runtime boundary; source and release provenance improve | Independent source version, but platform still rebuilds to consume it | Same as current | Moderate CI/supply-chain work; no provider operations | First migration step for proving repository and SDK portability |
| Sandboxed local executable/Wasm rules | Local platform client or server sandbox | Depends on a narrow host ABI, quotas, and a maintained runtime | Portable artifact can release independently | Low latency and stronger offline behavior | New sandbox, determinism, resource, patch, and ABI burden | Defer; evaluate later for offline/local rules, never as a frontend escape hatch |
| Registered remote provider | Provider owns rules and durable gameplay revision; OmarchyGS owns the platform envelope | Separate process/network/failure domain with least-privilege grants | Provider and cartridge deploy independently; sessions pin exact identities | Network-dependent; cached views are read-only during outage | Highest: registry, egress, keys, quotas, audit, reconciliation, support | Enabled only for the operator-pinned Door Legends v1 first-party pilot; external providers remain gated |

The staged recommendation now retains compiled Signal Siege while the
operator-pinned Door Legends v1 release proves the brokered remote model.
External providers still require review/onboarding, transparency, operations,
and support gates rather than inheriting the first-party authorization.

## Authority and data ownership

| Surface | Authority |
|---|---|
| Account authentication, sessions, MFA, suspension | OmarchyGS only |
| Persona profile and avatar projection | OmarchyGS only |
| Connections, inbox, challenge policy, catalog, launch permission | OmarchyGS only |
| Server catalog admission, local signing trust, launch, suspension, revocation | Selected OmarchyGS deployment/operator |
| Marketplace review and publication provenance | Marketplace authority; never automatic server admission |
| Game rules, private gameplay state, turns, game clock, and game randomness in remote mode | Registered game provider |
| Platform session envelope, participants, pinned provider/rules/cartridge identities, status, and accepted result receipt | OmarchyGS |
| Game-scoped result and achievement claim | Provider proposes; OmarchyGS authenticates, validates policy/idempotency, and records |
| Rendering, input, accessibility, theme, and local cosmetic animation | Trusted OmarchyGS client |
| Durable recovery notification | OmarchyGS cursor feed; WebSockets remain hints |

The platform and provider must not both claim authority over the same gameplay
snapshot or revision. Migration 0015 makes that choice explicit: compiled
sessions require local object state and no provider release; registered-
provider sessions require a pinned release and null local state. The retained
authenticated view is presentation-only and cannot advance or restore rules.

## Cartridge identity and contents

The production v1 artifact is a canonical, stored-entry ZIP archive with the
`.ogsc` extension, a canonical Ed25519-signed integrity envelope, and this
bounded shape:

```text
manifest.json
presentation.json
schemas/
  <name>.schema.json
locales/
  <locale>.json
assets/
  <name>.png
  <name>.wav
integrity.signed.json
```

The manifest identifies, at minimum:

- canonical game key, publisher/provider registration, rules version, and
  cartridge version;
- compatible OmarchyGS SDK and presentation-protocol ranges;
- player and mode metadata plus catalog display information;
- one entry screen and all required/optional presentation capabilities;
- registered backend identity, never an arbitrary runtime URL;
- requested platform scopes and persona projection fields;
- hashes, media types, decoded-size metadata, and integrity identity for every
  packaged file;
- localization inventory and accessibility metadata; and
- signing key identity and release/retirement metadata.

A game session pins the exact game rules version, provider identity,
presentation protocol, and cartridge content digest. A publisher may release a
new cartridge without silently changing an active session.

## Catalog and installation lifecycle

1. A game repository's CI runs SDK conformance tests and produces an immutable
   cartridge plus independently deployable provider artifact.
2. A registered publisher signs a canonical integrity index covering the
   manifest and every package file.
3. The vetted marketplace verifies the exact publisher release, records review
   and provenance, and publishes lifecycle metadata without forcing any server
   to admit it.
4. A server administrator imports and activates an exact marketplace release
   through the server's catalog boundary. The server records its own catalog
   policy and advertises only admitted release identity, provenance,
   compatibility, and content digest to players.
5. A client acquires the exact bytes through a bounded server-approved
   distribution path. Both server import and client acquisition stream under
   compressed and expanded size limits and verify publisher integrity, any
   marketplace review attestation, the selected server's admission policy,
   digest, protocol range, and declared capabilities. The distribution
   destination comes from trusted platform/catalog configuration, never from
   the cartridge, and may not redirect outside that exact policy.
6. Extraction rejects absolute paths, parent traversal, links, duplicate or
   non-canonical names, unexpected file types, excessive file counts,
   compression bombs, and undeclared content.
7. Schemas and assets are parsed under strict byte, dimension, duration, node,
   and complexity limits. Only allowlisted decoders and media formats ship.
8. A verified cartridge is installed atomically into a content-addressed,
   read-only client cache. The same digest may be reused across server profiles,
   but admission/provenance policy remains scoped to each server. The package
   receives no executable permission.
9. Server catalog approval controls whether a cartridge is visible and
   launchable.
   Publisher, provider, release, or signing-key revocation can prevent new
   launches independently. Existing sessions follow an explicit suspend,
   migrate, or finish policy rather than silently changing versions.
10. An eligible new session atomically pins one exact current release and
    admission revision. That presentation identity is immutable, remains null
    for legacy or ineligible sessions, and never follows a later catalog
    selection.
11. If the exact session mount is absent later, a participant explicitly asks
    the companion to acquire the immutable pin. The server resolves retained
    signed snapshot/release evidence instead of current selection, while current
    active-session lifecycle policy still authorizes use. The companion verifies
    the session before and after acquisition before publishing the mount.

The Ticket 015 filesystem store is a same-user local-development boundary, not
a privileged or multi-user installer. It rejects direct links and unexpected
file types, bounds every read, and treats revocation lookup errors as denial.
Any later service that writes into a root an untrusted local user can rename or
replace must use descriptor-relative containment (or an equivalent OS sandbox)
and an authoritative catalog revocation check; pathname checks alone are not a
sufficient privilege boundary.

Ticket 017 implements that stronger Linux boundary as
`SecureCartridgeStore::open_existing`. An operator provisions one root; the
store opens it without following a symlink and retains descriptors for every
fixed child. Blob, release, conformance, policy, and activation reads/writes use
`openat`/`mkdirat`/descriptor-relative rename with no-follow, exclusive
temporary creation, bounded reads, read-only publication, and directory sync.

Ticket 032 composes that store with a separately authenticated marketplace
snapshot. Synchronization stages exact reviewed bytes without changing the
legacy active pointer, caches newer lifecycle policy monotonically, and commits
one PostgreSQL inventory only after the entire snapshot succeeds. The local
catalog selection is a distinct expected-state/idempotent transaction with an
immutable audit receipt. Only a present, imported, compatible, locally selected
`active` or `deprecated` release is effective; suspension, denial, snapshot
removal, or incompatibility fails closed with no version fallback.

Ticket 033 makes distribution an optional all-or-nothing server capability.
The configured manual marketplace key or root-authenticated channel bundle and
secure-store root must agree with the database's retained exact signed snapshot
evidence; otherwise the route is absent or startup/acquisition fails closed. An
authenticated request
can name only the selected game key and exact archive digest. The server
resolves that exact release under current lifecycle policy, emits a canonical
bounded acquisition envelope, and self-verifies it. The envelope has no
destination, URL, credential, executable, or alternate-release instruction.

Ticket 035 preserves the first authentic signed snapshot needed by every
session-pinned release in normalized immutable tables. Marketplace replay can
fill missing evidence but cannot rewrite a digest to different signed bytes or
key material; omitted releases remain historical and ineffective for current
selection. Every future presentation pin requires one retained release-evidence
link. The historical route participant-authorizes that pin and self-verifies it
under `active_session` policy without turning provenance into current catalog
authority.

Ticket 036 adds `omarchygs.marketplace-trust-channel/v2` outside the public
Game Cartridge SDK. One offline root signs a bounded validity-window payload
binding the stable channel and marketplace authority, strictly increasing
bundle version, exact current marketplace snapshot, complete ordered
active/retired/revoked key history, and immutable native-package records. Only
the active key may sign the declared current snapshot; retired keys authenticate
their exact closed historical ranges; revoked keys authorize nothing. Every
transition retains prior key identity and ranges and permits only monotonic
active-to-retired/revoked or retired-to-revoked movement.

The reviewed native package may embed only a public bootstrap: root, channel
location, platform identity, installed version, and minimum acceptable bundle
and snapshot versions. Those package floors prevent a first-run or cache-cleared
client from accepting known-old signed trust. The client persists the complete
root-verified bundle in a descriptor-bound private store. A below-floor bundle
is unavailable but remains transition evidence, so installing a package with a
higher floor cannot erase terminal key history. The server persists the same
root/key continuity in PostgreSQL, and live security-sensitive requests reject
a runtime made stale by a later administrator rotation or revocation.

The same root-signed payload may bind immutable native packages by platform,
architecture, version, filename, size, SHA-256, source revision/digest, and
build-provenance digest. The companion selects only a newer artifact for its
packaged platform, downloads it through the guarded channel into bounded
mode-0600 same-user staging, and rechecks current trust before atomic
publication. It returns a fixed-path `pacman -U` command as text; it never
executes a shell, package manager, sudo, or installer. This authenticates bytes
and provenance metadata without turning the same-user client into a privilege
boundary or claiming a hosted marketplace operation exists.

The player package owns the remote trust transition in a per-launch loopback
Rust companion, not in QML. It requires the selected server's immutable UUID,
canonical origin, device Bearer, exact digest, and admission revision; verifies
discovery and the initial catalog; performs the bounded same-origin request
without proxy, redirect, or decompression; requires every acquisition key to be
authorized by either a client-controlled manual key or an explicitly enrolled
packaged channel; verifies the retained marketplace snapshot, current signed
policy snapshot, publisher release, policy,
SDK/host compatibility, archive, conformance, and attestation; then re-reads
the catalog to close the admission race. Acquisition v2 permits an eligible
retired key for historical release evidence while requiring the exact active
key for current policy. The root, channel, and manual key never come from
discovery, catalog metadata, QML, or the acquisition response. The credential
is zeroized when possible and is never written to the cache.

The per-user cache is private and descriptor-anchored. Immutable content is
shared by digest, while mode-0400 canonical mount documents live under exact
server UUID profiles and bind provenance, the client-trusted marketplace-key
SHA-256 fingerprints for evidence and policy plus their snapshot versions, and
admission revision. A profile whose exact keys and versions are unknown or
revoked under current client trust fails closed on restart. Profile
documents retain up to 128 exact records keyed by game, archive digest, and
admission revision, so historical and current releases coexist. Replacement
of an identical tuple uses exclusive cross-process locking, exclusive temporary files,
sync, no-follow opens, and atomic rename. A failed install/update never replaces
another mount; removal deletes only the named exact local mount and deliberately
retains shared content. QML receives only bounded catalog/mount facts and
explicit operations; cartridge-supplied QML or networking remains impossible.
For live gameplay, the companion additionally requires the selected canonical
server origin and UUID, the session's exact digest and admission revision, the
profile's client-trusted evidence/policy fingerprints and snapshot versions,
the privately retained publisher key, and current signed active-session policy.
It compiles only the
requested exact signed screen—or entry when no screen is requested—against the
authoritative REST view. It returns accepted current-screen, entry-screen, and
navigation metadata around the unchanged inert render plan. Digest-named PNG/WAV
assets remain in a bounded in-memory cache behind a random per-plan loopback
capability, exact Host validation, allowlisted media types, and no-store
responses. QML receives no cache path, publisher code, marketplace selector,
provider endpoint, or gameplay credential.
The root and every fixed child must be owned by the opening process's effective
user and cannot be writable by group or other. Each policy transition acquires
an exclusive lock through a fresh descriptor-relative open of the retained
policy directory, then performs its read/compare/replace under that lock. The
highest authenticated policy is cached before its allow/deny decision, so a
denied update survives restart and concurrent imports cannot roll state back.
Renaming or replacing the path-visible root after it opens cannot redirect the
operation. The compatibility `install`/`revoke` path remains explicitly
same-user only.

### Operator-custom content

An owner-operated server may enable a server-local signing and catalog
authority and import a custom cartridge without marketplace review.
The source is recorded as `operator-custom`, along with the server/operator
identity and exact digest; it cannot reuse a marketplace-vetted provenance
label. Ticket 038 implements this as an explicit database-local administrator
import, source-aware catalog selection, monotonic signed lifecycle, immutable
audit, current and historical acquisition, and source-pinned session
presentation. The normal server reads only the public operator key; the
mode-0600 private key is required only by the local admin process.

Marketplace bypass never means verifier bypass. An operator-custom cartridge
has the same canonical inert format, signatures, bounds, media profiles,
schema checks, trusted render-plan compilation, and lack of executable/network
authority as a marketplace release. A server may distribute custom cartridge
bytes only after the player explicitly pins the exact canonical origin, stable
server UUID, and advertised operator key in the local companion. The client
stores that decision privately, does not follow key replacement, shows a
permanent unreviewed warning and full fingerprint, and keeps marketplace and
custom mount provenance separate. The server may not turn those bytes into raw QML,
JavaScript, native client code, WebEngine content, or a direct provider URL.

Custom executable server code is not a cartridge. Game rules use the registered
provider boundary. General behavior uses the separately versioned server module
base described in [`server-modules.md`](server-modules.md): one exact no-WASI
component per OS-contained host, capability-scoped typed hooks/intents,
core-owned configuration/state namespaces, audit, and lifecycle controls. The
architecture proof does not authorize production loading, and dynamic
in-process Rust modules remain rejected.

## Trusted presentation contract

The cartridge uses a versioned declarative screen language interpreted by
trusted OmarchyGS components. It is a data model, not a general programming
language.

Initial node families should cover:

- terminal/ANSI-inspired text with an allowlisted style model;
- rows, columns, panels, overlays, tabs, scroll areas, lists, and grids;
- menus, buttons, forms, dialogs, focus order, shortcuts, and help text;
- board cells, card stacks, meters, charts, maps, tile layers, and sprite
  layers;
- raster images, sprite sheets, platform vector primitives, and platform-owned
  effects;
- local cosmetic transitions, tweens, timelines, particles, and sound cues;
- loading, offline, stale, empty, error, and reconnect states; and
- semantic labels, roles, reading order, reduced-motion alternatives, scalable
  text, and high-contrast metadata.

Bindings may select and format fields from a schema-validated view model. They
may not evaluate expressions, import modules, construct URLs, access global
objects, or execute scripts. An input gesture emits only a declared action ID
and schema-validated arguments. The trusted client sends that action to
OmarchyGS; the cartridge cannot open its own socket or HTTP request.

The v1 action shapes are intentionally exact: a Grid action declares sorted
`column` and `row` fields and emits exactly those integer coordinates; a Button
action declares and emits an empty object. A package whose action declaration
does not match its node's platform-owned emitter is non-conformant. Future node
families must add a new reviewed action contract rather than treating declared
field names as an open-ended permission.

`presentation.navigation.v1` reserves `navigate.<screen_id>` for inert
host-local movement. The suffix must identify an existing signed screen, the
action payload is empty, and exactly one Button emits each navigation action.
Grids, unknown targets, duplicate emitters, malformed reserved-prefix values,
and use as gameplay all fail verification. Cycles are valid because the host
caps local history and compiles each requested destination through the same
authenticated renderer; the value is never a URL or provider command.

Screen templates live in the reviewed cartridge, while the provider returns
only view-model data conforming to the pinned schema. A compromised provider
therefore cannot replace a reviewed screen with a credential prompt or inject
new executable UI.

### Implemented v1 renderer contract

Each screen pins one manifest-declared view schema. The production verifier
retains the exact authenticated payload bytes behind read-only accessors, so
the renderer never reopens a publisher-controlled path and callers cannot
mutate a `VerifiedCartridge` after verification. The renderer then:

1. validates a bounded JSON view against the signed restricted schema;
2. resolves only dotted object bindings and signed action/asset references;
3. applies host capabilities, the cartridge's typed fallbacks, trusted scale,
   high-contrast, reduced-motion, and audio-mute preferences;
4. incrementally enforces the selected Core or Rich-2D node, effect,
   per-raster, referenced decoded-raster, and retained-plan-byte budgets before
   keeping each node;
5. authenticates each referenced asset path once, publishes an accepted asset
   once, and converts it into a SHA-256 filename with only `.png` or `.wav`
   extensions; and
6. emits `omarchygs.render-plan/v1`, which contains inert allowlisted tags and
   never QML, JavaScript, markup, arbitrary URLs, or cartridge paths. The
   companion's outer v2 response separately binds the accepted screen, entry
   screen, and exact navigation map without changing render-plan v1.

The implemented Core nodes are `terminal`, `grid`, `status`, `button`, `image`,
and `meter`. Rich-2D adds `sprite`, `particle_field`, and `audio_cue`. QML maps
those tags through an explicit switch to repository-owned components. All text
uses `Text.PlainText`; the preview, trusted surface, and visual nodes consume
the same host-owned `OgsTheme` palette and typography contract as the main
shell. Grid and Button share their keyboard, pointer, and accessibility press
paths; sprites and particles honor reduced motion; audio honors mute; and the
origin strip remains platform-owned. Cartridge data cannot inject colors,
styles, or markup. The QML boundary
independently recounts aggregate grid cells, images, sprites, particles, audio
cues, and animations against the plan's claimed profile before instantiating
any component. Trusted Image nodes request a host-bounded source size and load
asynchronously; the renderer refuses oversized raster references before it
publishes plan or asset bytes.

`loading`, `offline`, `stale`, `empty`, `protocol_error`,
`unsupported_capability`, and `revoked` are fixed platform states. They render
the signed origin plus a platform message and instantiate zero cartridge
nodes. The vocabulary has no editable or password component, so a cartridge
cannot construct an OmarchyGS authentication or MFA prompt.

The developer preview command verifies the real `.ogsc`, compiles the same
production plan, and writes a canonical plan plus digest-named assets into an
explicit empty `0700` directory. Its files are `0444`; it reads no platform
credentials and has no database, provider, or network dependency. This is a
same-user developer boundary. A future privileged or multi-user launcher still
requires descriptor-relative directory containment and stronger process
isolation.

### Frontend option decision

| Frontend family | Keyboard/accessibility and platform fit | Isolation and bridge risk | Version/update shape | Decision |
|---|---|---|---|---|
| Signed declarative cartridge rendered by trusted QML | Native focus, shortcuts, themes, semantic roles, reduced-motion behavior, and consistent platform chrome | Small allowlisted data vocabulary; no game network, filesystem, script, import, process, or credential bridge | Cartridge pins schemas/capabilities; trusted renderer updates with OmarchyGS | Baseline |
| Raw publisher QML/JavaScript | Native-looking UI is possible but accessibility quality is publisher-controlled | Rejected: Qt treats QML/JS as trusted code and does not provide an untrusted-code sandbox | Publisher code would execute on every update | Prohibited |
| Provider-hosted WebEngine page | Familiar web tooling; keyboard/accessibility and visual integration vary | Separate renderer helps, but Chromium, origins, navigation, downloads, permissions, and bridge APIs create a much larger attack surface | Independently deployed pages can change after review unless content is pinned | Future isolated compatibility profile only |
| Wasm computation with trusted cartridge rendering | Can preserve native presentation if Wasm supplies only bounded deterministic rules | Requires a versioned capability ABI, fuel/memory limits, and deterministic host calls; Wasm alone does not render safely | Independently versioned module plus renderer contract | Future local-rules experiment |
| Native publisher plugin | Maximum graphics/runtime freedom | Shares the process or needs a full OS sandbox; compromise approaches arbitrary code execution | ABI and operating-system coupling | Prohibited for third-party cartridges |

## Launch and command flow

1. OmarchyGS authenticates the device session, derives the owned acting
   persona, enforces launch/challenge policy, creates the platform session
   envelope, and atomically pins the current exact presentation release and
   admission revision when one is eligible.
2. For registered-provider authority, OmarchyGS contacts only the endpoint
   stored in the provider registry and issues a short-lived grant bound to the
   audience, pairwise persona subject, session, release, scope, and replay
   identity. The cartridge cannot choose the destination, and reusable device
   credentials never cross the boundary.
3. The participant reads the authoritative session projection. The QML client
   sends its public server origin/UUID, exact presentation binding, current
   object view, and trusted preferences to the same-user companion.
4. If the exact mount is absent, the participant explicitly chooses install.
   The companion requests the session-pinned release, independently verifies
   retained marketplace evidence and the client trust key, re-reads the session,
   and publishes only the unchanged exact digest/revision mount.
5. The companion resolves only that exact mount under current signed
   active-session policy and compiles the entry or requested signed screen. QML
   independently validates the inert plan, screen/navigation metadata, and
   host-created asset capability before instantiating repository-owned components.
6. A navigation gesture selects only a companion-approved signed target, keeps
   bounded Back/Entry history inside trusted QML, and performs no gameplay or
   provider request.
7. A gameplay gesture emits only the signed declarative action and exact shaped
   payload. The client sends that intent, accepted screen, pinned archive
   digest, expected session revision, and idempotency key to OmarchyGS—not to a
   provider.
8. OmarchyGS reauthorizes the participant, linearizes against lifecycle change,
   verifies the exact signed release and current-screen action, rejects the
   reserved navigation namespace, translates the command itself, and stores one immutable admission before dispatching to the
   session's sole compiled or registered-provider authority.
9. Existing command idempotency and revision semantics remain authoritative.
   Exact replay recovers the admitted operation after lifecycle change, while a
   fresh post-suspension/revocation action is denied. The client refetches REST
   truth rather than inventing state.
10. Provider-initiated result, turn, or achievement events still return through
   the authenticated replay-protected callback. OmarchyGS records their bounded
   platform effect once and wakes affected personas through the existing cursor
   sync boundary.

### Failure, retry, and reconciliation contract

- A missing exact session mount is a recoverable presentation state, not a
  gameplay-state failure. Historical acquisition is explicit; any authorization,
  evidence, trust-key, pin, lifecycle, or post-fetch recheck failure leaves
  authoritative state and every other exact mount unchanged.
- Cartridge action admission is a short database transaction that ends before
  compiled execution or provider network I/O. It records actor, exact session
  revision, release and policy identity, exact screen, signed action/payload, translated
  command, and authority under the marketplace lifecycle lock. This is the
  durable authorization point for an exact uncertain retry.
- A platform command receives one durable idempotency key and expected
  provider revision before the outbound request. A timeout is **unknown**, not
  failure; OmarchyGS retries the same command and idempotency key with a fresh
  short-lived grant until it retrieves the provider's original receipt or a
  bounded retry policy expires.
- A reused idempotency key with different session, screen, action, arguments, or
  expected revision is a conflict. A stale revision causes the client to fetch
  the latest validated view and explicitly reissue user intent; it is never
  silently rebased.
- Provider results and events carry stable event IDs and monotonic provider
  revisions. OmarchyGS records an event receipt before applying platform-side
  results, achievements, or notifications; duplicate deliveries return the
  stored disposition.
- Provider unavailability leaves the platform session pending or unavailable.
  A last validated view may render as stale/read-only, but OmarchyGS does not
  invent an accepted move, advance a provider revision, or award a result.
- Reconciliation queries the authenticated provider by pinned platform session
  and compares signed provider identity, terminal status, revision, and event
  receipts. Operator-visible mismatches suspend commands; they do not choose a
  winner by timestamp.
- WebSockets may reduce wake-up latency only. Durable provider events enter the
  OmarchyGS cursor feed, and clients recover through REST/cursor state after a
  disconnect.

The initial remote model should use OmarchyGS as the network broker. That adds
one hop but keeps device credentials, provider grants, endpoint policy, rate
limits, audit, retries, and privacy enforcement out of the cartridge and QML
runtime. Direct client-to-provider access can be reconsidered only as a
separately threat-modeled capability.

## Provider protocol security

- Use TLS end to end and asymmetric provider authentication. The spike will
  evaluate sender-constrained OAuth-style access tokens and/or HTTP Message
  Signatures rather than inventing an unauthenticated callback scheme.
- Restrict grants by audience, exact game/provider/version, persona subject,
  session, scope, expiry, and unique identifier. Providers cannot refresh or
  exchange a grant for broader platform access.
- Use pairwise persona subjects per provider or game. Supply only the public
  display/avatar projection required by the declared capability.
- Sign callbacks over the method, authority, target, content digest, timestamp,
  nonce, provider, session, event ID, and revision. Store replay/idempotency
  receipts before applying a result or achievement claim.
- Resolve provider endpoints through an operator registry and guarded egress
  policy. Reject loopback, link-local, private, metadata-service, DNS-rebinding,
  redirect, and unregistered destinations according to deployment policy.
- Bound request bodies, response bodies, redirects, duration, concurrency,
  commands, callbacks, events, and per-provider resource use. Use timeouts,
  circuit breakers, backoff, and a kill switch.
- A provider outage yields an explicit unavailable/stale state. A cached last
  validated view may be shown read-only; commands are not reported as accepted
  until the authoritative provider confirms their idempotency key.
- Achievements are game-scoped manifest definitions. A provider submits a
  signed claim; OmarchyGS validates publisher/game/version, participant,
  definition, bounds, and replay state before writing the platform ledger.
  A provider never directly writes global achievement state.

## Frontend and package threat model

| Threat | Required control |
|---|---|
| Arbitrary QML/JavaScript or native-code execution | Custom declarative DSL rendered only by trusted built-in components; no cartridge imports, loaders, plugins, eval, shell, or FFI |
| Package substitution or publisher compromise | Asymmetric publisher signature, canonical integrity index, content digest pinning, transparent version identity, key rotation, revocation, and audit |
| Archive traversal, links, duplicates, or decompression bomb | Streaming byte/file/ratio limits; canonical paths; reject absolute/parent/link entries and duplicate normalized names; atomic read-only install |
| Image/audio/parser resource exhaustion | Allowlisted current decoders, compressed and decoded limits, dimension/duration caps, asynchronous parsing, and preferably a disposable low-privilege validation worker |
| Credential phishing inside a game | Unspoofable platform chrome; authentication/MFA never rendered by cartridge nodes; reserved platform dialogs and visual origin indicator |
| Data exfiltration | No cartridge network, filesystem, clipboard, camera, microphone, process, environment, or arbitrary URL capability; provider sees only scoped platform data |
| UI or GPU denial of service | Incremental plan-byte and scene-node admission; unique authenticated-asset hashing/publication; independent QML aggregate recounting; texture, decoded-pixel, particle, animation, audio, frame-time, and memory budgets; suspend the game surface without killing the shell |
| Malicious shaders | No cartridge-supplied shaders in the baseline; expose only platform-owned named effects. Any future custom shader tier requires separate review and containment. |
| Provider request forgery or token replay | Registered egress destination, asymmetric authentication, audience/scope/expiry checks, nonce/jti replay cache, idempotency keys, expected revisions, and TLS |
| Provider compromise or abuse | Per-provider isolation and quotas, minimal pairwise identity, signed audit trail, scoped achievements, suspension/revocation, and no database or reusable session access |

Renderer crash containment should eventually place cartridge parsing and the
game surface behind a constrained process boundary. Even before that exists,
the custom DSL materially reduces risk because it never enters the QML compiler
or JavaScript engine.

## Graphics capability envelope

The graphics ceiling is primarily a product and security choice, not a basic Qt
Quick limitation. Qt Quick uses a retained scene graph rendered through modern
graphics APIs and supports animation, sprites, particles, effects, shapes, and
custom shaders. Rendering remains local, so remote-provider latency need not
drive every animation frame.

The practical answer to “how far can it go?” is **well beyond a text BBS and up
through polished, animated 2D games**, provided the game fits the reviewed
presentation vocabulary. The system can make a terminal door game feel
authentically retro, but it can also render card tables, tactical maps,
tile-based worlds, sprite characters, particle accents, charts, dialog-heavy
RPG screens, and local 60 FPS cosmetic animation. The boundary is closer to a
safe, portable 2D console UI than to an unrestricted PC game engine.

The proposed profiles are:

| Profile | Intended capability | Good fits | Deliberate limits |
|---|---|---|---|
| Cartridge Core | Styled terminal text, panels, menus, forms, lists, grids, board cells, images, focus/navigation, loading/offline/error states, and simple transitions | Classic BBS games, interactive fiction, trivia, menus, scoreboards | No code, arbitrary markup, remote assets, custom drawing, video, shaders, or 3D |
| Rich 2D | Tile maps, sprite sheets, cards, tactical boards, vector primitives, meters/charts, local animation timelines, particles, platform effects, and bounded sound/music | Roguelikes, card/board games, asynchronous RPGs, strategy, management, puzzles, visual novels, polished retro games | Provider updates are state/action paced rather than per-frame; only host-defined nodes/effects |
| Advanced 2D/2.5D | Larger scrolling scenes, custom host-implemented render primitives, bounded video, richer post-processing, and approved capability extensions | Isometric tactics, animated maps, elaborate arcade-like presentation, cut scenes | Opt-in hardware/profile checks; no arbitrary JavaScript, QML, shader, or network access |
| Future 3D | Trusted renderer support for a constrained scene schema and validated 3D assets through an optional Qt Quick 3D module | Turn-based 3D boards, simple dungeon scenes, model viewers, lightweight tactical games | Separate dependency/license/security review, GPU baselines, asset and scene budgets; not part of the initial cartridge contract |
| Isolated Web experience | Provider-hosted HTML in a locked, separately profiled WebEngine surface | A compatibility escape hatch for games that cannot fit the DSL | Larger Chromium attack/patch surface, weaker native consistency, stricter origin/permission controls; never the default cartridge path |

This is enough for visually rich retro and modern 2D games. It can plausibly
support experiences comparable in presentation complexity to polished card
games, tactical maps, turn-based RPGs, roguelikes, and animated BBS successors.
It is not intended for a Halo-class first-person game, high-frequency physics,
competitive twitch networking, a general Unity/Unreal runtime, or arbitrary
publisher rendering code.

### Capability by delivery stage

| Stage | What a cartridge can do | What remains unavailable |
|---|---|---|
| Ticket 015 local contract | Carry signed `terminal`, `grid`, and `status` screens, schemas, localization, strict 8-bit PNG assets, and PCM WAV assets; prove host compatibility and install as inert bytes | No rendering in the Ticket 015 slice; no provider network |
| Ticket 016 trusted renderer | Compile schema-valid views into Core `terminal/grid/status/button/image/meter` or Rich-2D `sprite/particle_field/audio_cue` plans; render through platform-owned QML with measured bounds, keyboard/accessibility states, fallbacks, and an isolated production previewer | No publisher QML/JS, expression language, Canvas, shader code, WebEngine, video, 3D, provider network, or confirmed game mutation |
| Ticket 033 player mount | Acquire and independently verify one exact admitted release, retain immutable content privately, and mount it under an exact server origin/UUID profile | A mount alone creates no session, presentation authority, or game mutation |
| Ticket 034 trusted gameplay | Pin one exact admitted release to an eligible session, compile its mounted signed entry screen, render through trusted QML, and route declared actions through durable OmarchyGS authorization | No historical auto-download, multi-screen navigation, publisher executable code, direct provider networking, or arbitrary URL |
| Ticket 035 historical navigation | Explicitly acquire an old session pin through retained evidence, keep exact releases mounted side by side, and navigate signed screens locally with screen-bound real actions | No silent download, current-catalog substitution, publisher execution, or navigation network request |
| Ticket 036 public trust/package channel | Enroll an offline-root-signed marketplace keyring, apply rotation/revocation to exact snapshot ranges, verify dual-key acquisition evidence, and stage an exact newer native package | No selected-server trust bootstrap, hosted-marketplace claim, root rotation, or automatic/privileged installation |
| Later reviewed profiles | Add Advanced 2D/2.5D host primitives and possibly constrained 3D assets/scenes when separate hardware, decoder, dependency, and threat reviews pass | No promise of a general engine or arbitrary third-party execution |

This staging makes graphics additive. A new renderer primitive becomes a
versioned capability; older hosts reject it when required or use the cartridge's
declared fallback when optional. A game never receives extra execution,
filesystem, credential, or network authority merely because its graphics tier
is richer.

The hard practical limits are:

1. **Vocabulary:** a cartridge can use only renderer capabilities supported by
   the player's OmarchyGS version. New visual primitives require a reviewed host
   update and SDK capability version.
2. **Authority latency:** local cosmetic animation can run at display rate, but
   meaningful game-state transitions wait for the authoritative provider.
3. **Resource budgets:** package size, decoded pixels, texture dimensions,
   scene nodes, active sprites, particles, simultaneous animations, audio,
   state payloads, and frame time are bounded.
4. **Portability:** a required capability launches only on clients that support
   it. Optional effects must have a declared fallback, including reduced motion
   and software-rendering behavior.
5. **Safety:** custom shaders, scripts, native extensions, dynamic remote assets,
   and unreviewed media formats stay outside the baseline even when the hardware
   could execute them.

The first renderer budgets are calibrated on the exact local software-rendered
reference guest. They are a reproducible compatibility floor for that host,
not evidence for an untested low-end physical GPU. The host publishes a
presentation profile containing its limits; a cartridge declares requirements;
launch fails clearly when a required limit or capability cannot be satisfied.

Qt's own performance guidance treats 60 frames per second as a common target,
leaving roughly 16 milliseconds for a frame. The renderer should use scene
graph primitives and local animations, keep provider/network work asynchronous,
avoid large frequently updated Canvas textures, and profile particles/effects
on the supported hardware.

### Reference measurements and ratified v1 renderer ceilings

The Ticket 014 loopback proof exercised one screen with three trusted node
types, a 3×3 board, one local scan-line animation, and no audio or decoded
image. On the development host with Qt 6.11.2's software backend, its captured
sample was:

| Measure | Observed proof value |
|---|---:|
| Signed expanded fixture | 4 files / 2,436 bytes |
| Rendered frame sample | 120 frames / 15.99 ms average / 17.00 ms maximum |
| QML process peak resident memory | 88,184 KiB |
| Provider command/view | one action / 64 KiB maximum validated view / 128 KiB HTTP-body ceiling |
| Proof package enforcement | 32 files / 256 KiB each / 1 MiB total |
| Proof scene enforcement | 8 screens / 128 nodes / 16×16 maximum grid |

Ticket 016 runs the production compiler and QML components at 920×600 with
Qt 6.11.2, `QT_QUICK_BACKEND=software`, and the offscreen platform plugin. The
reference environment is a KVM guest with six exposed Intel i9-12900K vCPUs,
11 GiB RAM, Virtio GPU, and Linux 7.1.8. The focused gate includes a 60-frame
warm-up and measures the next 120 frames; it enforces a 33.3 ms average ceiling
and each profile's hard RSS ceiling while retaining the sample maximum as a
diagnostic.

The implemented **v1 ceilings** are:

| Resource | Cartridge Core v1 | Rich-2D v1 | Required fallback/failure |
|---|---:|---:|---|
| Archive / expanded / entries / entry | 8 MiB / 32 MiB / 256 / 8 MiB | Same v1 cartridge envelope | Verifier rejects before renderer |
| Signed package decoded envelope / raster | 128 MiB total / 4,096 px side / 32 MP | Same strict 8-bit PNG and PCM-WAV envelope | Verifier rejects undeclared, malformed, or over-envelope media |
| Rendered raster / referenced scene raster | 1,024 px side, 1 MP, 4 MiB each / 16 MiB scene | 2,048 px side, 4 MP, 16 MiB each / 64 MiB scene | Required node rejects; optional decoration drops before plan/asset publication |
| View JSON / render plan | 256 KiB / 1 MiB | 512 KiB / 2 MiB | Reject before QML publication |
| Active nodes / grid cells | 256 / 1,024 | 512 / 4,096 | Required content rejects; optional decoration drops deterministically |
| Images / sprites / particles / audio cues | 32 / 0 / 0 / 0 | 64 / 128 / 2,048 / 16 | Apply signed fallback before instantiation; then enforce profile |
| Simultaneous local animations | 32 | 128 | Reduced motion disables nonessential animation |
| Surface RSS | 256 MiB soft / 384 MiB hard | 384 MiB soft / 512 MiB hard | Focused gate fails above hard ceiling |
| Software frame sample | 16.67 ms target / 33.3 ms average ceiling | Same | Preserve input; lower optional effects if repeatable evidence misses ceiling |

The stress fixtures intentionally exercise Core at 256 nodes, including a
32×32 grid and 32 images, and Rich-2D at 213 nodes, including 64 images, 127
simultaneous sprites, 2,048 particles, 16 audio cues, and 128 animations. A
post-remediation single-CPU-affinity run measured Core at 15.996 ms average /
16.699 ms maximum and 133,336 KiB peak RSS, and Rich-2D at 16.000 ms average /
17.820 ms maximum and 244,764 KiB peak RSS. The accepted 2,048 px / 16 MiB
Rich-2D raster remained responsive at 15.992 ms average / 16.807 ms maximum and
257,848 KiB peak RSS; the former 4,096 px trigger was rejected before plan
publication. A 2× scale, high-contrast, reduced-motion, muted-audio run measured
16.003 ms average / 17.273 ms maximum and 237,596 KiB peak RSS. Every fixed
failure state instantiated zero game nodes and remained within the same frame
and hard-memory gates.

These figures are local evidence, not a promise that every Omarchy device or
every future scene will hit the same numbers. Required capabilities fail
clearly on an insufficient client; optional capabilities must declare an omit,
static, reduced-motion, muted, placeholder, or simpler-node fallback supported
for that node family.

## Separate-repository SDK model

The production v1 OmarchyGS Cartridge SDK is protocol-first and
language-neutral. `omarchygs-cartridge sdk-export` deterministically emits an
exact lock, compatibility/retirement policy, README, and JSON Schemas for the
manifest, presentation, restricted view schema, release attestation, catalog
policy, and lock itself. The production Rust verifier remains authoritative;
the schemas are pinned developer artifacts, not an alternate permissive parser.

The SDK surface includes:

- canonical JSON or binary schemas for manifests, views, actions, launch
  grants, commands, results, achievements, and callbacks;
- a conformance runner that a game repository executes without the OmarchyGS
  database;
- generated or hand-written provider adapters for supported languages;
- a trusted cartridge previewer using the same renderer and limits as the
  production client;
- fixtures for replay, revision conflict, expiry, invalid signatures, unknown
  capabilities, malformed views, resource limits, outage, and revocation;
- deterministic packaging, integrity indexing, signing, and local developer
  registration; and
- compatibility rules separating SDK/protocol version, rules version,
  cartridge presentation version, and provider deployment version.

A release directory contains exactly `cartridge.ogsc`, `conformance.json`, and
`release.signed.json`. Its domain-separated publisher attestation binds the
source Git revision, builder name/version/binary digest, SDK lock digest,
publisher/key IDs, game/rules/cartridge versions, canonical archive and signed
content identities, and the exact conformance-report digest. Platform
consumption re-runs the production archive verifier and reconstructs the report
before trusting those provenance fields.

Publisher trust and catalog authority are separate. The platform catalog signs
an exact game/publisher/archive policy with a monotonically nonzero version and
one of five states:

| State | New launch | Active session |
|---|---|---|
| active | allow | continue |
| deprecated | allow with warning | continue |
| suspended | deny | suspend |
| revoked | deny | terminate |
| retired | deny | continue the pinned release |

Every secure import and resolution requires an explicitly supplied valid policy
matching the installed digest. A cached higher version prevents downgrade; an
uncertain, mismatched, or invalid policy fails closed and never substitutes a
different installed cartridge.

First-party games use the same public contracts and conformance suite as later
providers. They may receive a higher catalog trust tier, but not private
database access or a different identity model.

The current exported v1 SDK is cartridge/release focused, while the public
surface of `omarchy-game-provider` and the Door Legends clean-clone pilot prove
the backend protocol seam. A later **OmarchyGS Provider SDK** will turn that
seam into a supported developer product: versioned protocol/model packages,
starter backend service, signing and grant helpers, conformance/fault fixtures,
deployment templates, and operational guidance. It will not bundle backend
code into the cartridge or grant a provider direct client access.

The SDK may support an operator running that provider beside their OmarchyGS
deployment as a separate service. A co-located profile still needs exact
provider identity, separate state/credentials, authentication, bounds, and an
explicit local transport design; it cannot reuse the conformance-only loopback
escape hatch or gain platform database access.

## Implemented remote-provider security foundation

Ticket 018 implements `omarchy-game-provider` as a production workspace crate.
It was deliberately dormant until Ticket 019 added an optional, all-or-none
server bridge for the sole operator-pinned Door Legends pilot. PostgreSQL
migration 0014 provides operator-controlled providers and immutable exact
releases; append-only message/TLS key history; lifecycle scopes; short-lived
grant records; database quota windows and expiring concurrency leases; durable
operation attempts and authenticated message receipts; and immutable safe
security audit events.

The protocol now uses Ed25519 grants over retained canonical claim bytes. Each
grant lasts at most 60 seconds and binds the OmarchyGS issuer, provider
audience, exact release/game/rules/cartridge identities, platform session, one
scope, replay UUID, and an HMAC-derived provider/game pairwise persona subject.
The raw persona/account identity and reusable device credentials never cross
the provider boundary.

Requests, responses, and callback-shaped events use a fixed OmarchyGS v1
profile of RFC 9421 HTTP Message Signatures and RFC 9530 `Content-Digest`.
Method, authority, path or status, originating request context, content type,
provider/release/message identities, creation/expiry, nonce, algorithm, key,
and protocol tag are signed. The strict parser rejects extra, duplicate,
reordered, stale, future, or mismatched fields and authenticates exact body
bytes before JSON parsing.

Production egress accepts only an operator-registered HTTPS DNS origin. One
bounded resolution must contain only public unicast destinations; the client
pins those sockets while retaining the registered hostname for SNI. It trusts
only registered DER roots and disables proxies, redirects, referers,
decompression, and connection reuse while enforcing registered connect/total
deadlines and streaming response ceilings. The conformance feature cannot
create a general private-network allowlist: it admits only one exact generated
loopback socket and is absent from the platform server.

Every semantic operation is retained before I/O under an idempotency UUID and
expected revision. Exact completed replay resolves from PostgreSQL; changed
intent conflicts. A timeout remains `unknown`, so a retry uses a fresh grant
and signed message to retrieve the provider's stable receipt. Authenticated
callbacks are deduplicated by message/event identity and digest. Ticket 018
records no result, achievement, notification, or gameplay projection; those
authority and policy decisions remain Ticket 019 work.

## Staged adoption

1. **Cartridge vocabulary and preview:** define the core/rich-2D DSL, package
   verification, capabilities, fixtures, and a trusted previewer while current
   gameplay remains compiled into OmarchyGS.
2. **Separate first-party game repository — implemented for the cartridge:**
   the Door Legends repository fixture is materialized as a fresh Git repository
   and cloned twice. With only copied CLI binaries, an exported SDK, and an
   explicit publisher key, both clones produce byte-identical releases that the
   production verifier, descriptor-relative importer, and trusted previewer
   consume. Compiled server rules remain in OmarchyGS pending later game work.
3. **First-party remote provider — Door Legends pilot implemented:** Ticket
   018 supplies registry, scoped grants/messages, guarded egress, durable
   replay controls, and TLS conformance. Ticket 019 adds the single-owner
   migration, optional all-or-none broker runtime, provider-backed catalog/
   start/command/reconcile APIs, atomic result/achievement callbacks, explicit
   availability/lifecycle states, and a separately built Door Legends TLS
   process with its own PostgreSQL database and callback outbox.
4. **Owner-operated catalog and player launch — implemented:** Tickets 032–035
   let an
   administrator synchronize one pinned vetted marketplace, stage exact
   releases, independently activate/deactivate/upgrade/rollback them with
   audit, expose server-scoped authenticated metadata, distribute the exact
   selected release, and let the independently trusting client cache/mount it.
   Eligible sessions then pin that exact release. A participant can explicitly
   acquire an old immutable pin after selection changes, keep exact releases
   mounted side by side, navigate cyclic signed screens locally, and route only
   screen-bound gameplay actions back through OmarchyGS.
5. **Public backend SDK:** package the provider protocol as a supported starter
   server, versioned SDK, conformance suite, and operations contract.
6. **Operator-custom trust — implemented:** local cartridge signing/import,
   source-aware lifecycle/admission, explicit per-server client trust,
   persistent provenance warnings, current/historical acquisition, and
   unchanged inert-cartridge verification/rendering.
7. **Server modules and hooks — fixed first-party base implemented:** the
   extension isolation proof now feeds one production observation-only
   capability/lifecycle/audit base independently of the provider contract;
   custom installation and additional hooks remain gated.
8. **Reviewed external providers:** add publisher onboarding, catalog review,
   quotas, monitoring, suspension, support policy, and game-scoped achievement
   trust.
9. **Optional advanced presentation tiers:** add capabilities such as isolated
   web content, approved custom rendering, or constrained 3D only through
   separate threat models and compatibility profiles.

The challenge and first-game work should consume only seams confirmed by the
spike. It should not implement speculative remote tables or expose provider
endpoints before the proof and ADR are accepted.

## Production decisions that remain after the first-party pilot

- pairwise persona subject derivation and avatar delivery/cache policy;
- how later media formats or profile budgets are added without weakening v1;
- whether the production renderer is a new constrained process or a hardened
  surface inside the OmarchyGS client;
- generalized provider event pull/callback policy and disaster recovery beyond
  the one Door Legends runbook;
- Qt Quick 3D/WebEngine packaging, licensing, update, and containment policy;
  and
- public SDK hosting, transparency/CI attestation, signing-key operations, and
  third-party support policy beyond the exact local first-party release proof;
- server identity, marketplace availability, package mirroring, cache/update,
  and server-scoped catalog-policy distribution;
- operator-local trust enrollment, provenance UX, and reviewed self-hosting
  terms/custom-content disclosures; and
- server module isolation, hook ordering, capabilities, state/migration
  ownership, upgrade/rollback, fault containment, and support policy.

The original Ticket 014 proof deliberately used ephemeral Ed25519 keys,
loopback HTTP, and in-memory replay receipts. Ticket 018 replaces those gaps
with the production TLS, registry, rotation/revocation, durable controls,
guarded egress, quota, and fixed RFC 9421/9530-shaped profile described above.
The separate fixture still uses ephemeral keys because it is test evidence;
production keys are generated and retained outside this repository.

## Evidence consulted

- Local QML runtime is Qt 6.11.2 with Qt Base, Declarative, Multimedia, and
  WebEngine installed; Qt Quick 3D is not currently installed.
- [Qt Quick](https://doc.qt.io/qt-6/qtquick-index.html) documents the visual
  canvas, animation, particles, effects, input, and scene primitives.
- [Qt Quick Scene Graph](https://doc.qt.io/qt-6/qtquick-visualcanvas-scenegraph.html)
  documents retained, graphics-API-backed rendering and batching.
- [Qt Quick graphical effects](https://doc.qt.io/qt-6/qtquick-effects-topic.html)
  documents transforms, shaders, particles, and sprites.
- [Qt Quick performance guidance](https://doc.qt.io/qt-6/qtquick-performance.html)
  documents frame budgets, profiling, asynchronous work, and particle/effect
  constraints.
- [Qt Quick Canvas](https://doc.qt.io/qt-6/qml-qtquick-canvas.html) warns that
  large, frequently updated Canvas images incur texture-upload cost.
- [Qt Quick 3D](https://doc.qt.io/qt-6/qtquick3d-index.html) confirms that Qt
  can mix 2D and 3D content, while remaining a separate optional module.
- [Qt QML network transparency](https://doc.qt.io/qt-6/qtqml-documents-networktransparency.html),
  [Qt's shared security model](https://doc.qt.io/qt-6/shared-security-model.html),
  and [handling untrusted data](https://doc.qt.io/qt-6/untrusteddata.html)
  explicitly treat QML/JavaScript as trusted code and recommend a custom DSL or
  other sandbox for untrusted content.
- [Qt WebEngine security considerations](https://doc.qt.io/qt-6/qtwebengine-security.html)
  document its separate security controls and ongoing Chromium patch surface.
- [RFC 9700](https://www.rfc-editor.org/rfc/rfc9700.html) recommends asymmetric
  client authentication, audience/privilege restriction, sender-constrained
  tokens, replay defenses, and TLS.
- [RFC 9068](https://www.rfc-editor.org/rfc/rfc9068.html) defines relevant
  audience, expiry, token ID, client, and scope validation for JWT access-token
  profiles.
- [RFC 9421](https://www.rfc-editor.org/rfc/rfc9421.html) defines HTTP Message
  Signatures and discusses component coverage, timestamps, expiry, nonce, TLS,
  and replay prevention.

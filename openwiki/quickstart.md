---
type: "Reference"
title: "Omarchy Gaming System engineering quickstart"
openwiki_generated: true
sources:
  - id: openwiki-source-0bb8016edf4f4744d3a09cf4
    resource: repo://bin/gate.sh
  - id: openwiki-source-2bcdc046ce25b89194fc5af0
    resource: repo://client/qml/cartridge/nodes/TrustedButtonNode.qml
  - id: openwiki-source-c566a55d52a9744f7b26b7c4
    resource: repo://client/qml/cartridge/TrustedCartridgeSurface.qml
  - id: openwiki-source-0196de8872a3fef5b0b350d3
    resource: repo://client/qml/CartridgeController.qml
  - id: openwiki-source-998b0f5a7b56d7475101b7a2
    resource: repo://client/qml/components/OgsTheme.qml
  - id: openwiki-source-da678ac479c336e5e6fc1d04
    resource: repo://client/qml/GameController.qml
  - id: openwiki-source-d392f8f0962c50f0d66e0629
    resource: repo://client/qml/Main.qml
  - id: openwiki-source-f73ad44f40942d16dc369861
    resource: repo://client/qml/OnboardingController.qml
  - id: openwiki-source-fb3bac0b93c3046f977a1023
    resource: repo://client/qml/screens/SocialScreen.qml
  - id: openwiki-source-4f5334e859a4d83e2a196fcf
    resource: repo://client/qml/SocialController.qml
  - id: openwiki-source-29b3070f4c2872e1150fb5ea
    resource: repo://client/qml/tests/cartridge/tst_trusted_cartridge_controls.qml
  - id: openwiki-source-fc035ef77d2451c6e8138211
    resource: repo://client/qml/tests/fixture/tst_accessibility.qml
  - id: openwiki-source-3156e0b1532bb1d02a0118e1
    resource: repo://client/qml/tests/live/tst_live_onboarding.qml
  - id: openwiki-source-bc8915a33f270bc28a270170
    resource: repo://crates/client-cartridge-runtime/src/service.rs
  - id: openwiki-source-20452fec62fdae4a8bc45707
    resource: repo://crates/game-cartridge/src/marketplace.rs
  - id: openwiki-source-df8490db5b51be8096630e7e
    resource: repo://crates/game-signal-siege/src/lib.rs
  - id: openwiki-source-eac208eae3530bb62f49d2bc
    resource: repo://crates/marketplace-publisher/src/bin/omarchygs-marketplace-publisher.rs
  - id: openwiki-source-2bc4557686cbe5b8dfa44f45
    resource: repo://crates/marketplace-publisher/src/lib.rs
  - id: openwiki-source-18fcba4155ece2440818ba7e
    resource: repo://crates/marketplace-publisher/src/store.rs
  - id: openwiki-source-e61b285fcaa489b63922f43f
    resource: repo://crates/server/src/app.rs
  - id: openwiki-source-ba203ea2e600f294ab58ef02
    resource: repo://crates/server/src/bin/omarchygs-admin.rs
  - id: openwiki-source-7243a317e3224aa82795a5fc
    resource: repo://crates/server/src/cartridge_catalog.rs
  - id: openwiki-source-a3892e0554790e3efc606fe1
    resource: repo://crates/server/src/challenges.rs
  - id: openwiki-source-4b133589ca70bd174cf19eb9
    resource: repo://crates/server/src/connections.rs
  - id: openwiki-source-26aac996689c040c6aab6825
    resource: repo://crates/server/src/games.rs
  - id: openwiki-source-b2c7af59f511c4ed8a004fb0
    resource: repo://crates/server/src/inbox_api_tests.rs
  - id: openwiki-source-a13fe4db1eee073d0a7e2c4d
    resource: repo://crates/server/src/main.rs
  - id: openwiki-source-f6dda000394ac1ba6bba8f65
    resource: repo://crates/server/src/marketplace_sync.rs
  - id: openwiki-source-83e16151ac88c29a31cb79d2
    resource: repo://crates/server/src/mfa.rs
  - id: openwiki-source-94ddb58f2dc1a71ed1959533
    resource: repo://crates/server/src/operator_admin.rs
  - id: openwiki-source-54f6da1456b2b76d94d11b0e
    resource: repo://crates/server/src/personas.rs
  - id: openwiki-source-0e10f198b5749ecebf761185
    resource: repo://crates/server/src/provider_games.rs
  - id: openwiki-source-e4423ee4de83f38bd240bf8b
    resource: repo://crates/server/src/reports.rs
  - id: openwiki-source-42fe6bf463fcb01dc5566e16
    resource: repo://crates/server/src/server_discovery.rs
  - id: openwiki-source-d943a78fae758ed47e30a12a
    resource: repo://crates/server/src/sessions.rs
  - id: openwiki-source-76060b846b9222af2c790243
    resource: repo://crates/server/src/signal_siege_api_tests.rs
  - id: openwiki-source-e7a72df5b89c1ac350ffe062
    resource: repo://crates/server/src/sync.rs
  - id: openwiki-source-408aa68caebee417a5a319b8
    resource: repo://docs/architecture/adr-0002-game-cartridge-and-provider-boundary.md
  - id: openwiki-source-bfc109ee5d2c2f6c0f5c5f77
    resource: repo://docs/architecture/adr-0003-owner-operated-server-and-extension-boundary.md
  - id: openwiki-source-c22435ddb0c3a9abfe95d9af
    resource: repo://docs/architecture/game-cartridges.md
  - id: openwiki-source-e9c32af872bdfcc1f392d212
    resource: repo://docs/architecture/server-modules.md
  - id: openwiki-source-872141f77f71851168245852
    resource: repo://docs/architecture/system-overview.md
  - id: openwiki-source-831ed1de42e0dff0edb87b3b
    resource: repo://docs/client-installation.md
  - id: openwiki-source-fa645fac0603cca986708fed
    resource: repo://docs/operators/marketplace-publication.md
  - id: openwiki-source-c3d1d450d3a3561b368e5307
    resource: repo://docs/planning/ROADMAP.md
  - id: openwiki-source-85dba8f87dd5947de337aca5
    resource: repo://docs/product-charter.md
  - id: openwiki-source-cb6494f7cbf0d5d23ffe082a
    resource: repo://migrations/0012_game_challenges.sql
  - id: openwiki-source-11256f84337d259ecf424a45
    resource: repo://migrations/0019_marketplace_catalog.sql
  - id: openwiki-source-449de92825ee702b9aa05d2a
    resource: repo://packaging/arch/client-runtime-files.txt
  - id: openwiki-source-d85e6ea816d7c91e9828f7b2
    resource: repo://packaging/arch/omarchygs
  - id: openwiki-source-c909643e4ac6a14f500d178e
    resource: repo://packaging/arch/PKGBUILD
  - id: openwiki-source-f30a02c87f1e4ddc4bad65fa
    resource: repo://scripts/check-qml-style.py
generated: {by: "codex", at: "2026-09-03T15:38:08.451Z"}
---

# Omarchy Gaming System engineering quickstart

Omarchy Gaming System is an API-first social gaming system with a keyboard-first QML
connector as its flagship client. The implemented runtime now starts
PostgreSQL, applies migrations, exposes database-backed `/health`, accepts
invitation-required account registration at `POST /v1/accounts`, provides revocable Bearer device
sessions, offers opt-in TOTP two-factor authentication with single-use recovery
codes, supports account-owned personas with public exact-handle lookup, and
supports persona-scoped connection requests, accepted connections, and private
directional blocks. An authenticated owned persona can also file a bounded,
retry-safe report about another public persona for local operator review.
Accepted persona pairs also own durable private
conversations with typed messages and per-participant unread state. Every
persona also has a retained, monotonic synchronization cursor: REST recovers
durable changes and an authenticated WebSocket supplies owner-scoped wakeup
hints. Connected personas can now create retry-safe, exact-version two-person
game challenges, receive their lifecycle as typed private inbox history, retain
terminal challenge history, and accept one into an atomic version-pinned game
session. A database-free compiled game registry validates exact rules
versions, public metadata, deterministic bounded initialization, and bounded
deterministic commands. PostgreSQL stores version-pinned sessions with ordered
persona participants, while participant-private REST routes read the durable
snapshot and apply revision-checked, session-idempotent commands. Production
publishes immutable Signal Siege v1, a deterministic one-human tactical game
against a server-owned bot, plus Signal Siege v2, an exact two-human
alternating-turn definition used by accepted challenges. Both use bounded
deterministic rules, durable completion, exact command replay, and retained
history. Production also includes canonical signed Game Cartridges, an
isolated trusted Core/Rich-2D renderer/preview CLI, a deterministic Game Cartridge SDK
export, signed release and catalog-policy verification, and a secure local
cartridge importer. One owner-configured marketplace can now synchronize a
canonical signed snapshot over guarded pinned HTTPS, stage exact reviewed
releases, publish one atomic PostgreSQL inventory, and expose a separately
admitted metadata-only catalog to authenticated players. Server trust can be
either the legacy manual marketplace key or an offline-root-authenticated
channel bundle with bounded active, retired, and revoked keys. An independently
configured distribution runtime can return the exact selected immutable
release and its source-specific evidence through a bounded authenticated
acquisition route. A separately configured operator-custom authority lets the
local administrator verify, sign, import, lifecycle-manage, and admit an inert
release without manufacturing marketplace review. The normal server receives
only that authority's public key and advertises it as a candidate; the player
must explicitly pin the exact origin, stable server UUID, and key fingerprint
in the native companion before custom install or play is eligible. When the
optional provider runtime is configured, the server also exposes the
operator-pinned Door Legends provider release and routes its player operations
through authenticated exact-v1 capability negotiation to a separate provider
process and database. The public provider developer kit packages that
provider-facing contract, a fixed four-route Axum/PostgreSQL starter, a
fifteen-case TLS conformance runner, and deterministic locally signed release
artifacts. The runner retains Relay Forge as its default sequence but accepts
one bounded optional gameplay profile for a persistent or terminal game's
launch, timeout/replay command, finite continuation, and expected final status;
the authentication, replay, fault, callback, reconciliation, and receipt corpus
does not change. Its `ProviderGame` seam receives only deterministic game state
and payloads; it has no platform registration, broker, egress, admission,
administration, or player-route authority. Relay Forge consumes those public
packages as an independent clean-room proof. Tickets 048 through 075 additionally
exercise them with a separate local Usurper development provider. Its rules-v3
slice adds player-private equipment, shops and haggling, bank and chest
transfers, healing-potion purchases, equipment-aware combat, configured
quick-heal-then-attack turns, and seventeen inert signed screens. Its rules-v4
slice adds three class-specific level-one spells, mana spend and daily refill,
resistance, temporary Fog absorption, encounter reset, and same-turn monster
response. Its rules-v5 slice adds the exact player mental and addiction defaults
plus the original weapon-gated Assassin Backstab and HP-funded Paladin Soul
Strike through one state-routed inert class-special action. Rules v6 adds the
passive Gnoll bite, persistent encounter-owned monster poison, same-turn poison
ticks before the monster response, and the source's no-immediate-reward outcome
when poison is lethal. Rules v7 adds exact source-linked level-two monster data,
draw-free level-one/level-two switching, the original rejection-loop draw order,
level-aware combat and retreat, and inert signed controls; normal level-two
selection preserves record 10 as data while accepting only records 11 through
19. Rules v8 adds exact source-linked level-three records, draw-free switching
across levels one through three, and the same rejection-loop and level-aware
combat path; it retains record 20 as data while normally selecting only records
21 through 29. Rules v9 extends that path through Level 4, retains record 30 as
source data while normally selecting records 31 through 39, and initializes the
selected monster at strength 14, defence 7, and 42 HP. Rules v10 extends that
path through Level 5, retains record 40 as source data while normally selecting
records 41 through 49, and initializes the selected monster at strength 15,
defence 7, and 45 HP. Rules v11 extends that path through Level 6, retains
record 50 as source data while normally selecting records 51 through 59, and
initializes the selected monster at strength 16, defence 8, and 48 HP. Rules v12
extends the same path through Level 7, retains record 60 as source data while
normally selecting records 61 through 69, and initializes the selected monster
at strength 17, defence 8, and 51 HP. Rules v13 extends the same path through
Level 8, retains record 70 as source data while normally selecting records 71
through 79, and initializes the selected monster at strength 18, defence 9, and
54 HP. Rules v14 extends it through Level 9, retains record 80 as source data
while normally selecting records 81 through 89, and initializes the selected
monster at strength 19, defence 9, and 57 HP. Rules v15 extends the same path
through Level 10, retains record 90 as source data while normally selecting
records 91 through 99, and initializes the selected monster at strength 20,
defence 10, and 60 HP. Rules v16 extends it through Level 11, retains record
100 as normally unreachable source data, and selects records 101 through 109
at strength 20, defence 10, and 60 HP. Rules v17 extends it through Level 12,
retains record 110 as normally unreachable source data, and selects records 111
through 119 at strength 20, defence 10, and 60 HP. Rules v18 extends it through
Level 13, retains record 120 as normally unreachable source data, and selects
records 121 through 129 at strength 20, defence 10, and 60 HP. Rules v19
extends the same path through Level 14, retains record 130 as normally
unreachable source data, and selects records 131 through 139 at strength 20,
defence 10, and 60 HP. Rules v20 extends it through Level 15, retains record 140
as normally unreachable source data, and selects records 141 through 149 at
strength 20, defence 10, and 60 HP. Rules v21 extends it through Level 16,
retains record 150 as normally unreachable source data, and selects records 151
through 159 at strength 20, defence 10, and 60 HP. Rules v22 extends it through
Level 17, retains record 160 as normally unreachable source data, and selects
records 161 through 169 at strength 21, defence 10, and 63 HP. Rules v23 extends
it through Level 18, retains record 170 as normally unreachable source data,
and selects records 171 through 179 at strength 22, defence 11, and 66 HP.
Rules v24 extends it through Level 19, retains record 180 as normally
unreachable source data, and selects records 181 through 189 at strength 23,
defence 11, and 69 HP. Rules v25 extends it through Level 20, retains record
190 as normally unreachable source data, and selects records 191 through 199
at strength 24, defence 12, and 72 HP. Rules v26 extends it through Level 21,
retains record 200 as normally unreachable source data, and selects records
201 through 209 at strength 25, defence 12, and 75 HP. Its signed cartridge
appends exactly one zero-payload Level 21 choice through bounded `option_u`, and
provider-backed QML advances one expected revision per accepted activation
while duplicate-label rejection and keyboard auto-repeat suppression remain
enforced. The signed visible fixture preview
renders fixed state, disables ordinary input, and emits only unconfirmed action
requests. A separate development-only local-play launcher now drives one
in-memory real Usurper provider session through exact revision/screen/action
requests and the same trusted render-plan surface. It passes QML only a private
startup-file path, keeps the bearer out of process arguments, and confirms
seven successive entry, creation, street, dungeon, and combat actions while
requiring one loaded node and one exact action match at every step. Each visible
Usurper choice is one phase-valid provider command;
provider mutations follow the authenticated candidate view's next screen and
are confirmed only after that signed screen renders successfully. Trusted
buttons and grids ignore keyboard auto-repeat so one held activation cannot
cross a plan replacement into the newly focused control.
Ticket 065 adds a visible-window Qt Quick regression suite that sends actual
mouse and Return input, proves a control loaded while disabled becomes active,
checks one emitted action, and verifies that plan replacement removes the old
delegates. Ticket 067 introduces recursive counting of the real delegate tree,
Ticket 068 ratchets the replacement to sixteen controls followed by seventeen,
Ticket 069 ratchets it again from seventeen to eighteen, Ticket 070 ratchets it
from eighteen to nineteen, and Ticket 071 ratchets it from nineteen to twenty
while requiring exactly one current delegate per node. Ticket 072 ratchets the
replacement from twenty to twenty-one, requires every loaded node to own one
explicit non-overlapping row, and sends a real Return event to each current
control. Ticket 073 ratchets that replacement from twenty-one to twenty-two and
hit-tests every current control through the surface mouse path and Return-key
path, requiring exactly one emitted action from each input. Ticket 074 ratchets
the replacement from twenty-two to twenty-three and retains the same one-action
pointer/Return proof for every current control. Ticket 075 ratchets it from
twenty-three to twenty-four, requires an observable empty delegate turn before
guarded rematerialization, waits for layout before exposing accessibility, and
uses one native Button activation path while retaining one pointer and Return
action per current control. The renderer gate
discovers and runs the matching Qt 6 test runner.
Tickets 066 through 075 reuse that input boundary for the Level 12, Level 13,
Level 14, Level 15, Level 16, Level 17, Level 18, Level 19, Level 20, and Level 21 actions
without changing the provider protocol or trusted renderer vocabulary.
The complete live provider conformance corpus remains the durable state,
security, callback, and restart proof. The existing provider
protocol, trusted QML vocabulary, and platform-owned state remain unchanged,
and neither development game is
thereby production-registered or admitted. The
main QML connector now
handles direct or saved server selection through exact public
discovery, masked invitation entry and account registration, password or MFA
sign-in, and owned-persona creation or selection before entering an
authenticated home. From there it can manage persona connections and private
blocks, submit a report by exact persona handle, browse private conversations,
page history, send messages, clear unread state, browse the compiled game
catalog and session history, create or resolve challenges, and play Signal
Siege through authoritative REST commands. The packaged client can also browse
signed cartridge metadata and, when both server acquisition and the selected
source's client-controlled trust are available, acquire, verify,
privately cache, update, remove, and mount the exact admitted release for the
selected server profile. Eligible newly created sessions now pin one exact
admitted cartridge. A participant can explicitly acquire that immutable pin
after catalog selection changes; the companion independently verifies the
session before and after acquisition and retains exact release-and-admission
mounts side by side. It compiles the requested signed screen from the matching
mount, trusted QML performs bounded host-local navigation, and screen-bound
gameplay actions return through the selected server's compiled or
registered-provider authority. Door Legends cartridge v2 is the first cyclic
multi-screen portable proof. The connector still does not poll or
subscribe to live WebSocket hints.

Production also has an opt-in server-module boundary. The exact compiled-in
Sentinel catalog contains release `1.0.0` and compatible release `1.1.0`; it and
up to eight explicitly admitted operator-custom identities share one no-WASI
process host, exact WIT, privacy-minimized persona-report hook, typed
`priority_review` proposal, core reauthorization, bounded durable outbox/state,
and immutable request/response receipt evidence. The local owner can explicitly
upgrade the reviewed module to `1.1.0` with a complete candidate state and roll
back once to the retained immediate predecessor. Reviewed lifecycle, custom
import, custom lifecycle/removal, and restore review are database-local owner
operations over private canonical files. Module inactivity, saturation, absent
runtime keys, or an unavailable exact selected package never rejects the
report; it records bounded aggregate gap evidence instead. Public
administration, admission hooks, egress, gameplay authority, and client code
remain unavailable.

An active or degraded custom module adds only a stable-server-bound aggregate
to public discovery: bounded count and behavior class plus fixed unreviewed-code
and operator-support warnings. The official client exact-validates, persists,
identity-binds, and continuously renders that warning before and after sign-in;
it receives no component bytes, private inventory, path, state, or signing
authority.

The main shell, all ten routes, and the trusted cartridge visual boundary now
share one host-owned theme and explicit plain-text policy. Semantic headings,
non-color status labels, accessible control names, deterministic initial focus,
reversible Tab traversal, Escape authority, contrast, and 640×420 containment
are exercised by the focused QML fixture before a delivery can pass.
One persistent shell-owned EXIT button requests a normal window close from
every route. Keyboard and pointer tests prove that it remains visible,
accessible, and bounded and that closing does not log out, revoke the durable
device session, or clear the selected persona through controller logic.

The same flagship client is now available as the native
`omarchy-gaming-system-client` Arch package for private-alpha testing. The
`x86_64` package contains the exact 40-file production QML inventory, a native
loopback cartridge companion, a relocatable `omarchygs` launcher, one Game
desktop entry, and non-secret build provenance; it contains no Rust game
server. A normal local build remains unsigned. A reviewed build may instead
embed a public bootstrap that pins an offline root, exact channel, platform,
and minimum trust/snapshot versions. The Games screen can explicitly enroll or
synchronize that channel and verify and stage an exact newer package artifact,
but the client never invokes `pacman`, `sudo`, a shell, or an automatic
installer. Deterministic static publication, offline-root handoff, immutable
activation, and guarded mirror verification now exist as local operator
tooling. Real public origins, root custody, repository signing, CDN rollout,
monitoring, and automatic privileged installation remain future operations.
The package includes bounded public-only profiles for
deliberately selecting among independent compatible servers; it does not
persist credentials or federate their identity, moderation, catalog, or
history.

The private-alpha operator path is deliberately separate from the player API.
`omarchygs-admin` uses a reviewed local `DATABASE_URL` to list a bounded report
queue, resolve or dismiss an open report, reversibly suspend or reactivate an
account, issue, inventory, or revoke registration invitations, inspect reviewed
cartridges, apply exact expected-state catalog selections, and execute
admin-only operator-custom cartridge/server-module commands plus the fixed
packaged reviewed module upgrade/rollback command. Those custom commands load
checked owner-private signing material;
the player server never loads provenance private keys. Its separate
`marketplace-sync` action also requires a canonical HTTPS origin, bounded DER
TLS root, pre-provisioned secure store, and either one manual marketplace key
or a matching offline-root-signed channel bundle. Mixed or partial trust
configuration fails closed. The same executable provides bounded module
inventory and expected-revision lifecycle/restore commands; those commands
accept only private stable local files and never become HTTP administration.
Every mutation carries an operation UUID, actor, and reason and commits an
immutable audit event with the state change. An invitation's 48-character raw
bearer code
is delivered only on its first successful issue receipt; PostgreSQL retains
only its digest, and later inventory never exposes code or credential material.
Suspension revokes all current device sessions; reactivation never resurrects
them. The isolated recovery drill proves that report, audit, suspension, and
representative platform history survive a custom PostgreSQL dump and restore.
Gate stage 22 separately proves the complete invite-only admission lifecycle
and software readiness for a private-alpha event.

The product is game-first: connections, private inboxes, challenges, and
persistent game history define the intended experience. A public message board
may complement that system later, but it is not the current identity or
private-alpha focus.

The accepted long-term deployment unit is an independently owner-operated
OmarchyGS community. An individual or group runs the standard server, curates
its game library, and invites players into server-local accounts, personas,
policy, and history. Supporting multiple compatible servers therefore means
isolated community profiles, not implicit federation or shared identity. Each
server publishes one database-owned UUID and bounded public compatibility
document. The UUID survives ordinary restart and database backup/restore and is
a continuity check, not a replacement for HTTPS server authentication.

The first playable now spans account, authentication, persona, connection,
private inbox, durable synchronization, two-person challenge-to-session
orchestration, alternating asynchronous play, and terminal-result recovery in
the flagship QML connector. The server also retains the independent solo path.
Compiled Signal Siege outcome-derived achievements and rewards remain future
work.

ADR-0002 now accepts the **Game Cartridge** as the staged portable-game
direction: a publisher-signed, data-only presentation package rendered by
trusted OmarchyGS QML components. Ticket 015 now implements deterministic v1
packing, strict verification/conformance, compatibility reporting, and a
same-user content-addressed local store. Ticket 016 implements the bounded
render-plan compiler, fixed trusted QML vocabulary, private preview output, and
measured Core/Rich-2D profile. Ticket 017 implements a deterministic public SDK,
signed reproducible release and five-state catalog policy, a Linux
descriptor-relative secure importer, and a clean-clone first-party repository
proof. Ticket 018 adds the production provider security crate and durable
schema for operator-pinned registration, signed grants and messages, guarded
egress, replay, quotas, leases, lifecycle, and audit. Ticket 019 connects that
foundation to one narrowly authorized first-party pilot: compiled Signal Siege
sessions retain OmarchyGS rules authority, while a Door Legends session pins
one exact provider release as its only durable rules/state/revision authority.
Ticket 044 extracts the provider-facing half into a public-only crate, requires
an authenticated exact-v1/four-capability preflight before effects, and proves
deterministic locally signed exports in two clean consumer clones. Ticket 045
adds the public starter and conformance crates, deterministic three-package
developer-kit export, and independently built Relay Forge second-game proof.
Ticket 046 adds the reviewed exact-release TLS-loopback sidecar, separate
deployment and recovery templates/runbook, hostile-peer and independent-restore
drill, and durable cross-process command/reconcile fencing. Tickets 048 through
075 prove a persistent game's bounded profile and game-neutral authenticated
views with a separate local Usurper provider, then extend it to a player-private
equipment/potion economy, configured combat-quaff parity, and signed
seventeen-screen cartridge plus the source-linked level-one spell and mana
loop, the source-linked Assassin Backstab and Paladin Soul Strike combat
branches, passive Gnoll poison, and the bounded source-faithful level-two,
level-three, level-four, level-five, level-six, level-seven, level-eight,
level-nine, level-ten, level-eleven, level-twelve, level-thirteen,
level-fourteen, level-fifteen, level-sixteen, level-seventeen, level-eighteen,
level-nineteen, level-twenty, and level-twenty-one dungeon bands.
They add no platform rule copy, route,
migration, registration, admission, deployment, or publication. Real external
provider onboarding remains separate work. Ticket
033 adds independently
trusted main-client acquisition,
private caching, and server-profile mounting. Ticket 034 adds immutable exact
session presentation pins, mounted render-plan launch, trusted QML gameplay,
and participant-authorized cartridge actions for both existing authority paths.
Ticket 035 retains immutable historical marketplace evidence, adds explicit
participant acquisition for an old session pin, permits exact multi-release
profile mounts, and adds signed host-only multi-screen navigation with
screen-bound gameplay admission. Ticket 036 adds the packaged public-channel
bootstrap, explicit monotonic enrollment and key rotation/revocation, separate
historical-evidence and current-policy authorization, and a root-authenticated
native-package staging channel without granting installer authority.
Ticket 037 adds a separate non-SDK publisher that verifies reviewed releases
and packages online, hands one public canonical request to an offline root
signer, finalizes and atomically selects an exact immutable static tree, and
verifies one or more hosted mirrors without merging publisher, catalog, root,
hosting, server-admission, or client-installation authority.
Ticket 038 adds the distinct operator-custom path: immutable publisher and
server attestations, source-aware PostgreSQL admission and session history,
explicit per-server client key pins, source-specific mounts, and persistent
unvetted warnings without changing gameplay authority.

ADR-0003 adds the owner-operated distribution and extension direction. Ticket
032 implements its first server-side slice: one pinned marketplace can supply
signed exact release policy, while each server administrator independently
admits one exact release per game and authenticated players see only effective
catalog metadata. Ticket 033 adds an optional exact-release distribution route
and a native client companion that verifies a client-controlled marketplace
key, publisher release, lifecycle policy, and selected-server admission before
writing private content and profile mounts. Ticket 034 then requires that mount
to match the selected server origin/UUID, session digest, admission revision,
and active-session policy before compiling the signed entry screen. Ticket 035
separates historical provenance from current selection, lets the player install
the exact old pin explicitly, and keeps navigation inside the trusted client.
The explicit operator-custom path may bypass marketplace review but cannot
bypass the inert package, publisher signature, current lifecycle, selected
server admission, or trusted-QML boundary. Ticket 039 and ADR-0004 now select
and prove a dedicated no-WASI process boundary for a separate capability-scoped
server module/hook system. Ticket 040 implements the reviewed Sentinel
observation base, Ticket 041 adds database-local operator-custom custody, exact
lifecycle, shared runtime dispatch, restore review, aggregate disclosure, and
operator responsibility, and Ticket 042 packages the compatible reviewed
successor with exact owner-controlled upgrade and immediate rollback. None
expands the hook or capability vocabulary. The public Provider SDK, starter,
conformance runner, deterministic developer-kit release, and second clean-room
game are implemented, as is the reviewed sidecar/operations profile for one
exact registered release. External provider onboarding, public module
administration, marketplace module admission, and broader hook classes remain
roadmap or external work. The bounded custom path is not a general plugin loader.

## Task routing

| Engineering intent | Read first | Primary source entrypoints | Narrow validation |
|---|---|---|---|
| Change server startup, configuration, migrations, health, or public discovery | [Runtime foundation](runtime-foundation.md) | `crates/server/src/main.rs`, `config.rs`, `app.rs`, `server_discovery.rs`; `migrations/` | focused discovery/configuration tests; `scripts/test-database.sh`; live smoke |
| Change account registration, invitations, device sessions, MFA, personas, or connections | [Runtime foundation](runtime-foundation.md) | `accounts.rs`, `registration_invites.rs`, `credentials.rs`, `sessions.rs`, `mfa.rs`, `personas.rs`, `connections.rs`; migrations `0001`–`0005` and `0017`; `docs/api.md` | Domain tests plus multi-account PostgreSQL evidence; `scripts/test-private-alpha.sh` for admission changes |
| Change player reporting, account suspension, report disposition, invitation administration, operator audit, or platform restore | [Runtime foundation](runtime-foundation.md) and [Development and validation](development-and-validation.md) | `reports.rs`, `operator_admin.rs`, `bin/omarchygs-admin.rs`; migrations `0016`–`0017`; `docs/operators/operator-safety-and-recovery.md`; `docs/operators/private-alpha.md` | Report API and operator-domain PostgreSQL tests; real CLI test; recovery and private-alpha drills |
| Change QML endpoint/profile selection, appearance/accessibility, account access, MFA sign-in, persona onboarding, social/inbox, game catalog, challenges, or gameplay | [Runtime foundation](runtime-foundation.md) and [Development and validation](development-and-validation.md) | `client/qml/Main.qml`, `ApiClient.qml`, `ServerProfiles.qml`, `OnboardingController.qml`, `SocialController.qml`, `GameController.qml`, `client/qml/components/`, `client/qml/screens/`, `client/qml/game/` | `scripts/check-qml-style.py`; `scripts/test-qml-onboarding.sh`; live QML smoke in `scripts/dev.sh --smoke-test` |
| Change inbox, challenges, synchronization, or game behavior | [Runtime foundation](runtime-foundation.md) and [Product boundaries](product-boundaries.md) | `inboxes.rs`, `challenges.rs`, `sync.rs`, `games.rs`, `crates/game-runtime`, `crates/game-signal-siege`; migrations `0007`–`0013`; challenge, game, Signal Siege, inbox, and sync API tests | Participant privacy, relationship policy, exact-version state, lifecycle, expiry, transition and revision races, retry effects, cursor/reconnect, and PostgreSQL evidence |
| Change cartridge packaging, trusted rendering, SDK portability, provider integration or sidecar operations, marketplace or operator-custom trust, synchronization, server admission, player acquisition, session pinning, historical recovery, signed-screen navigation, development local play, package staging, or trusted launch | [Game Cartridges](game-cartridges.md) and [Product boundaries](product-boundaries.md) | `crates/game-cartridge`; `crates/provider-sdk`; `crates/provider-starter`; `crates/provider-conformance`; `examples/provider-relay-forge`; `crates/game-cartridge-renderer`; `crates/client-cartridge-runtime`; `crates/marketplace-trust`; `crates/game-provider`; `crates/server/src/provider_games.rs`; `operator_custom.rs`; `session_cartridges.rs`; `marketplace_sync.rs`; `cartridge_catalog.rs`; `cartridge_distribution.rs`; `deploy/provider-sidecar`; `client/qml/MarketplaceController.qml`; `CartridgeController.qml`; `GameController.qml`; `client/qml/cartridge`; `client/qml/tests/CartridgeLocalPlay.qml`; `client/qml/tests/cartridge/tst_trusted_cartridge_controls.qml`; migrations `0014`–`0015`, `0019`–`0024`, and `0029`; ADR-0002; Tickets 015–019, 032–038, and 044–075 | Cartridge/renderer/SDK/provider focused scripts, including `scripts/test-provider-sdk.sh`, `scripts/test-provider-developer-kit.sh`, `scripts/test-provider-starter-conformance.sh`, and `scripts/test-provider-sidecar.sh`; root-signed trust-channel test; marketplace/custom PostgreSQL lifecycle/admission/migration tests; hostile companion/QML contract tests; private local startup-file checks and seven-action provider-backed trusted-QML confirmation; real pointer and Return input across a non-overlapping, accessibility-ready 23-to-24 control replacement; clean-clone Relay Forge and Door Legends proofs; sidecar containment/restore/operation-fence proof; native package smoke; threat/authority review and constitutional authority check |
| Change static marketplace preparation, offline-root signing, immutable activation, local verification, or mirror probes | [Game Cartridges](game-cartridges.md) and [Development and validation](development-and-validation.md) | `crates/marketplace-publisher`; `docs/operators/marketplace-publication.md`; Ticket 037 | `scripts/test-marketplace-publication.sh`; exact-tree, network-less ceremony, mirror, rotation, rollback, security, and canonical diff-gate evidence |
| Change owner-operated server, Provider SDK, or executable custom-content direction | [Product boundaries](product-boundaries.md) and [Game Cartridges](game-cartridges.md) | ADR-0003; `crates/provider-sdk`; `crates/provider-starter`; `crates/provider-conformance`; `docs/architecture/game-cartridges.md`; `docs/operators/owner-operated-servers.md`; `docs/planning/ROADMAP.md` | `scripts/test-provider-sdk.sh`; `scripts/test-provider-developer-kit.sh`; `scripts/test-provider-starter-conformance.sh`; current-versus-future audit; provenance/authority review; official-client containment |
| Change the server-module WIT, fixed loader, host, custom import/custody, report observation, dispatch, typed intent, receipts/gaps, state, lifecycle, disclosure, restore, or containment | [Server modules](server-modules.md) and [Product boundaries](product-boundaries.md) | ADR-0004; `docs/architecture/server-modules.md`; `docs/operators/server-modules.md`; `crates/server-module-runtime`; `crates/server/src/server_modules.rs`; `server_module_custom.rs`; `server_discovery.rs`; trusted QML profile/shell surfaces; migrations `0025`–`0027`; `crates/server-module-spike`; Tickets 039–041 | `scripts/test-server-modules.sh`; focused PostgreSQL/operator/discovery/QML tests; restore drill; `scripts/test-server-module-spike.sh`; threat/authority review; canonical local diff gate |
| Build, inspect, install, upgrade, remove, or diagnose the native player package | [Development and validation](development-and-validation.md) and `docs/client-installation.md` | `packaging/arch/`; `scripts/check-client-package-source.sh`; `scripts/build-client-package.sh`; `scripts/test-client-package.sh` | Source-contract check; extracted-package conformance; `bin/gate.sh --diff` before delivery |
| Run or diagnose the local stack and quality gate | [Development and validation](development-and-validation.md) | `scripts/dev.sh`; `bin/gate.sh`; `client/qml/Main.qml` | `bin/gate.sh --fast` or `--diff` |
| Start or resume a non-trivial change | [Codex workflow](codex-workflow.md) | `AGENTS.md`; `$omarchy-workflow`; active pipeline | Phase receipts and canonical gate |

## Current boundary

The database-backed health, invite-only account-registration,
revocable-device-session, opt-in TOTP MFA, and persona slices are executable
today. Registration atomically consumes a valid invitation and creates no
session or persona implicitly. A first submission returns `201`; only an exact
canonical-username and password replay of the same used invitation recovers the
same receipt with `200`. Other unavailable or mismatched invitations share one
generic denial. Clients then exchange credentials for an opaque token and use
that account authority to manage its devices, optional MFA, and one or more
personas. Once MFA is enabled, correct primary credentials
return a short-lived challenge rather than a session; a TOTP or unused recovery
code must complete that challenge before a new device token is issued.
Persona responses expose only public profile fields. Exact canonical handle
lookup is public, while the owning account remains private.

The keyboard-first QML connector now exercises that complete entry path. Its
registration mode masks the invitation bearer secret, includes it only in the
registration request, and clears it after completion, mode changes, and server
changes. It
accepts a bare server origin, allows HTTP only for exact loopback hosts, and
requires HTTPS remotely. Before account access it requires the exact public
OmarchyGS discovery fields, protocol 1, the implemented onboarding capability
subset, and any UUID already remembered for that canonical origin. Players may
connect once or save up to sixteen exact public-only profiles; incompatible or
replacement identities stay on the connection screen. Successful password or
MFA authentication then loads owned personas, permits creation when needed, and
requires explicit selection before the authenticated home. Bearer tokens, MFA
challenges, usernames, and persona authority remain only in process memory and
are cleared before endpoint changes, as well as on logout, challenge expiry,
terminal authentication failures, invalid sessions, or malformed authenticated
success responses.

The same shell exposes explicit Social, Inbox, Games, Challenges, and Gameplay
routes only after a valid owned persona is selected. One bearer-owning
transport stays behind the onboarding authority controller; the social and game
controllers receive a gated request function and derive actor paths from that
selected persona. Social
entry manually refreshes incoming/outgoing requests, accepted connections, and
the actor's private block inventory. Its report form resolves an exact public
handle, accepts one fixed category and 1–1,000 control-safe detail characters,
retains the same operation UUID only for an exact uncertain retry, and clears
player text only after validating the minimal creation receipt. Reports create
no subject notification or synchronization event. Inbox entry manually
refreshes at most 100 conversations, loads ascending bounded message pages,
prepends older pages by
the conversation-local cursor, sends trimmed control-safe text, and advances
unread state through the latest loaded message. Exact public profiles and
allowlisted user/system message shapes render as plain text. Malformed,
oversized, stale, or invalid-session responses fail closed; the last case also
clears bearer, personas, selection, and dependent controller state. The game
controller serially loads bounded catalog/session or catalog/connection/
challenge inventories, retains an uncertain mutation's exact idempotency
identity for explicit retry, and refetches session truth after commands or a
revision conflict. The connector does not poll or open the persona-sync
WebSocket yet, so screen entry, completed actions, and visible refresh commands
recover durable REST truth.

Connection commands authenticate the device session and require the acting
persona to belong to that account. Requests are directional until the
addressee accepts; accepted connections are mutual. Blocks remain private and
directional, atomically remove any existing relationship, and prevent new
requests in either direction until the blocker explicitly unblocks. Responses
embed only the public persona profile.

An actual connection-acceptance transition creates or reuses one private
conversation and appends one server-authored `connection_accepted` message.
Only either participant can inventory or read that conversation. User sends
require the pair to remain accepted and unblocked, while existing history stays
readable after removal or blocking. Message order and unread positions are
conversation-local and monotonic. A separate persona cursor recovers retained
cross-resource changes through REST; hint-only WebSockets tell clients when to
query it.

The current game boundary is executable. `GET /v1/games` always returns stable
compiled Signal Siege v1 and v2 metadata and, when the optional all-or-none provider
configuration is present, adds only an active operator-pinned Door Legends v1
manifest. Every catalog record declares `platform_compiled` or
`registered_provider` authority and an optional exact provider release. An
authenticated account may launch either admitted one-human definition for an
owned persona with a durable UUID receipt. The persona-root transaction checks
exact replay before current admission, admits at most 25 active solo starts,
and creates only the human seat; the deterministic Signal Siege bot has no
account or persona row.

Connected, unblocked personas may use the challenge flow for Signal Siege v2's
exact two-human definition. Challenge creation is bounded and idempotent, and
acceptance atomically fixes challenger/challenged at seats 0/1. V2 starts at
seat zero, alternates strike, guard, or charge actions, and completes on core
destruction or its twenty-four-turn bound.
All participants read durable snapshots without consulting today's registry
and submit bounded commands with a session-wide idempotency UUID and expected
revision. A transition now returns both state and authoritative active or
completed lifecycle. Completion stores its timestamp and explicit outcome;
the exact final command replays after completion, while new commands conflict.
List/detail history and payload-minimal sync invalidations remain reconnect
safe. Door Legends commands retain their idempotency key and expected provider
revision through the broker, while explicit reconciliation recovers unknown
outcomes. Provider session reads expose only the authority-tagged platform
envelope, last authenticated bounded view, availability, and optional
allowlisted result—not provider-private rules state. A projected provider view
must be a non-empty object, pass the public SDK's safe-payload rules, and remain
within 64 KiB; the exact signed screen schema still owns the game-specific
presentation shape. The QML flow currently
presents exact compiled Signal Siege v1/v2 state through its platform-owned
surface and presents an eligible bound Door Legends session through the trusted
cartridge surface. Missing or mismatched mounts, trust keys, helper authority,
or lifecycle decisions fail closed without making provider state executable.

The production cartridge crates can independently pack and verify a canonical
signed `.ogsc`, report host compatibility, install/revoke it in a bounded
same-user local store, validate a schema-conforming view, and compile a bounded
Core or Rich-2D plan for platform-owned QML components. They also export and
self-verify a deterministic public SDK, create and verify signed reproducible
release attestations, enforce signed five-state lifecycle policy, and stage a
release through a Linux descriptor-relative secure store.

The distinct `omarchygs-provider-sdk` preview owns provider-facing errors,
scopes, pairwise identity, exact-v1 compatibility, grants, signed messages,
schemas, fixtures, and deterministic export/verification. It accepts only
protocol 1 with launch, command, reconcile, and event; the broker authenticates
that selection before grant issuance and binds it through every operation,
response, and callback. Its signed finite-inventory export is independently
buildable, but publishing or consuming it does not register or activate a
provider.

The server-admin
path verifies one monotonic signed marketplace snapshot under either a manual
key or the root-channel's exact active key, retrieves each three-file release
below a fixed guarded origin, and publishes current reviewed inventory only
after the complete snapshot succeeds. PostgreSQL retains the exact signed
snapshot, marketplace key, root-channel history, and per-release policy-key
provenance. Local selection remains a separate
idempotent source-aware audited database command. The admin-only custom path
instead snapshots one verified publisher release, signs a server-scoped
attestation and lifecycle policy, and stores immutable custom authority,
release, and audit evidence without creating marketplace review. Public
discovery always advertises
`games.cartridge-catalog.v1` and, only when distribution is configured,
advertises current exact acquisition, session presentation, and historical
session-acquisition capabilities. A valid public custom configuration also
advertises `games.operator-custom-cartridges.v1` and its bounded public
authority candidate.
Authenticated `GET /v1/cartridges` remains metadata-only; the separate
acquisition route serves one bounded, self-verified exact selected release
with no digest fallback. The preview CLI
writes only read-only plan/assets into a caller-created private directory and
reports no provider, database, or credential use. The main QML connector's
compiled Signal Siege surface is platform-owned trusted UI and does not claim a
signed cartridge origin, digest, or render plan. The native companion now
verifies acquisitions against either client-controlled marketplace trust or
an explicit canonical server-origin/server-UUID/operator-key pin, caches inert
content privately, and maintains exact server-profile mounts. For
an exact immutable session binding, it also compiles the authenticated view for
one exact signed screen into a bounded plan, exposes only capability-scoped
digest assets, and gives QML no cache path or publisher code. A missing mount is
an explicit state: an authorized participant can ask the companion to acquire
the old session pin without making current catalog selection authoritative.
QML independently validates the plan and its exact navigation map, keeps Back
and Entry history locally, and sends only non-navigation gameplay actions with
the accepted screen ID to the selected OmarchyGS server. The
`omarchy-game-provider` crate implements
operator-pinned releases, signed pairwise grants and messages, public-only
pinned HTTPS egress, and durable replay/quota/lease/audit controls. The optional
production bridge instantiates it only for the Door Legends pilot. Before a
grant or provider effect it authenticates the SDK's exact-v1/four-capability
selection, binds the release revision and selected message key through final
locked admission, and keeps compatibility plus operation I/O under one
aggregate deadline. Migration
0015 prevents dual
authority: compiled sessions require local object state and no provider release,
whereas provider sessions require a release pin and null local rules state.
Authenticated callbacks become results, achievements, views, audit, and sync
effects only through one policy-checked projection transaction. A retained
pre-negotiation callback can recover only as an exact already-recorded
duplicate; new legacy-shaped network messages remain invalid.

Migration 0021 stores at most one immutable exact presentation release and
admission revision for an eligible session. Cartridge actions are separately
recorded as immutable admissions while the current participant, revision,
release digest, signed policy, lifecycle, declared action, shaped payload, and
host-translated command still agree. This preserves exact retry after an
uncertain provider or compiled operation, even if lifecycle later changes,
while fresh actions after suspension or revocation are denied.

Migration 0022 retains normalized signed marketplace snapshot/release evidence
as immutable historical provenance and requires it for future presentation
pins. New action admissions also retain the exact signed screen. Reserved
`navigate.<screen>` Buttons are accepted only as host-local presentation
transitions and are rejected at the gameplay boundary.

Migration 0023 binds every retained release policy to its exact marketplace
key and historical snapshot version, persists root-channel continuity, permits
only authenticated monotonic key transitions, and prevents a later singleton
snapshot from rewriting older provenance. Acquisition v2 can therefore verify
an old release snapshot under its eligible historical key while separately
requiring the current policy-bearing snapshot under the active key.

Migration 0024 adds immutable operator-custom authority and release evidence,
monotonic custom lifecycle and append-only audit, a mutually exclusive
marketplace-vetted/operator-custom catalog selection, and source-pinned current
and historical session presentations. A custom lifecycle writer joins the
same global lifecycle lock domain as fresh cartridge-action admission, so a
queued denial commits before a later new action while an exact already-admitted
replay remains recoverable.

Migrations 0025 and 0026 add the exact first-party module registry and
admissions, lifecycle/audit, bounded observation outbox, immutable delivery and
intent receipts, core-owned report labels, namespaced state and rollback
snapshots, aggregate observation-gap evidence, and retained request/response
preimages for new delivery receipts. Legacy receipts remain explicitly
distinguishable from complete evidence.

Migration 0027 generalizes that registry while preserving reviewed evidence. It
adds bounded immutable PostgreSQL custody for custom component and public trust
material, server-bound operator provenance, exact custom-operation receipts,
one-step predecessor/snapshot rollback, terminal retained state disposition,
and explicit runtime-unconfigured, replaced-admission, and removed-module gap
reasons.

Migration 0028 adds immutable whole-command evidence for packaged reviewed
release transitions. Its database checks admit only the fixed Sentinel
`1.0.0 → 1.1.0` upgrade and `1.1.0 → 1.0.0` rollback edges with monotonic
lifecycle/state revisions and the corresponding state schema.

Migration 0029 adds one bounded command/reconcile reservation to each eligible
registered-provider session. Runtime combines it with a transaction-scoped
PostgreSQL advisory fence held across provider transport, then revalidates the
reservation before authenticated projection so multiple platform processes,
expiry recovery, and operator lifecycle changes cannot create overlapping
provider effects or stale cleanup.

Current runtime identifiers use the gaming-system namespace; see [Runtime
foundation](runtime-foundation.md) for the narrow local compatibility window
retained for old bind configuration and session tokens.

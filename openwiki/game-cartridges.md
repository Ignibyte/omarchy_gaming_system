---
type: "Reference"
title: "Game Cartridges and portable provider direction"
openwiki_generated: true
sources:
  - id: openwiki-source-c566a55d52a9744f7b26b7c4
    resource: repo://client/qml/cartridge/TrustedCartridgeSurface.qml
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
  - id: openwiki-source-408aa68caebee417a5a319b8
    resource: repo://docs/architecture/adr-0002-game-cartridge-and-provider-boundary.md
  - id: openwiki-source-c22435ddb0c3a9abfe95d9af
    resource: repo://docs/architecture/game-cartridges.md
  - id: openwiki-source-4ab9cb9784f211e865cabf2a
    resource: repo://docs/planning/pipeline/active/separate-repository-sdk-and-first-party-cartridge.notes.md
  - id: openwiki-source-c3d1d450d3a3561b368e5307
    resource: repo://docs/planning/ROADMAP.md
  - id: openwiki-source-d69dbacb0ae7fe382ee46161
    resource: repo://scripts/test-game-cartridge-renderer.sh
  - id: openwiki-source-8df9ad1a3495f8360740ff03
    resource: repo://scripts/test-game-cartridge-sdk.sh
  - id: openwiki-source-4e51428e90d3c7db3949b09b
    resource: repo://scripts/test-game-cartridge-spike.sh
  - id: openwiki-source-68106a790eb8acc94f8d3540
    resource: repo://scripts/test-game-cartridge.sh
generated: {by: "codex", at: "2026-08-25T14:08:06.805Z"}
---

# Game Cartridges and portable provider direction

## Status and boundary

ADR-0002 accepts the **OmarchyGS Game Cartridge** for staged adoption. Ticket
015 supplies the production local package, verifier/conformance, compatibility,
and inert store. Ticket 016 supplies the production render-plan compiler, fixed
trusted QML vocabulary, and isolated preview CLI. Ticket 017 adds the
deterministic public SDK export, signed release and catalog-policy verification,
separate-repository first-party proof, and secure local importer. The main QML
connector does not launch cartridges yet, and production rendering does not
authorize remote gameplay authority. The compiled Rust runtime and OmarchyGS
PostgreSQL snapshot/revision remain authoritative under Constitution §10;
[Runtime foundation](runtime-foundation.md) maps that path.

Ticket 014 contributes an isolated executable architecture proof. Its broker,
provider, and QML surface are not a public SDK or deployed runtime. Remote
authority still needs a later protocol, migration, and explicit Constitution
amendment before it can leave operator-controlled proof work.

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
       trusted Rust render-plan compiler
                  │ bounded inert plan + digest assets
                  ▼
       trusted OmarchyGS QML components
                  │ unconfirmed declared action
                  ▼
       authenticated OmarchyGS broker (future)
                  │ scoped, short-lived pairwise grant
                  ▼
       registered provider in a future mode
```

The cartridge supplies signed presentation data. OmarchyGS supplies all
executable QML, focus/navigation, accessibility, themes, platform dialogs,
networking, and security policy. A future provider supplies game rules and
private gameplay state; it never supplies the trusted frontend.

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

Cartridges cannot contain or invoke publisher QML, JavaScript, native code,
shell commands, arbitrary shaders, imports, dynamic remote assets, filesystem
paths, clipboard/process access, or network clients. The trusted renderer
interprets only versioned node and action records. Provider-returned data must
match the screen's pinned view schema, and text is rendered literally rather
than as automatic rich markup. Authentication and MFA remain reserved,
unspoofable platform surfaces.

### Implemented trusted renderer path

The renderer accepts only the verifier's externally immutable
`VerifiedCartridge`. For a ready screen it reads the authenticated pinned
schema, validates one bounded view, resolves only dotted bindings and declared
actions/assets, applies signed optional fallbacks plus trusted scale, contrast,
reduced-motion, and mute preferences, and emits
`omarchygs.render-plan/v1`. Non-ready loading, offline, stale, empty, protocol,
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
| Game rules, private gameplay state, turn/time/randomness, provider revision in a future remote mode | Exactly one registered provider |
| Rendering, input, accessibility, theme, local cosmetic animation | Trusted OmarchyGS client |
| Durable client recovery | OmarchyGS REST/cursor feed; WebSockets remain hints |

A valid publisher signature authenticates package identity and bytes; it does
not make the publisher trusted for memory, CPU, action shape, or UI authority.
OmarchyGS retains every executable QML component, trusted preference,
origin/failure surface, and future action dispatcher.

In the proposed remote flow, the client always calls authenticated OmarchyGS
APIs. OmarchyGS resolves an operator-registered destination and sends a
short-lived grant bound to provider audience, game and release versions,
platform session, one scope, a pairwise provider/game persona subject, expiry,
and replay ID. Account identity, credentials, reusable device-session tokens,
and database access never cross the boundary.

A player action carries one durable idempotency key and expected provider
revision. A timeout means unknown outcome, so retries retain both values while
using a fresh grant. A changed replay is a conflict; a stale revision requires
an explicit refresh rather than a silent rebase. Signed provider events use
stable IDs and are deduplicated before achievements, results, notifications,
or sync projections are applied. Provider outage may expose the last validated
view as stale/read-only, but OmarchyGS never invents a move or result.

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

## Renderer evidence and isolated provider proof

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

The production renderer is gate 12, the SDK/release/import proof is gate 13,
and this isolated provider proof is gate 14 in every `bin/gate.sh` mode. See
[Development and validation](development-and-validation.md) for the full gate
and failure routing.

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
5. Ticket 018, after private-alpha need justifies it, builds production provider
   registration, grants/message security, guarded egress, quotas, replay state,
   audit, and revocation.
6. Ticket 019 pilots one first-party remote authority migration and proposes
   the required Constitution §10 amendment. External providers wait until
   operations, recovery, suspension, and support policy are proven.

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

## Change map

| Intent | Read/change first | Required evidence |
|---|---|---|
| Package, signing, and capability contract | `crates/game-cartridge`; ADR-0002; `docs/architecture/game-cartridges.md`; Ticket 015 | `scripts/test-game-cartridge.sh`; deterministic fixtures, malformed package and resource-limit matrix, signature/capability/revocation checks |
| Trusted renderer and graphics profile | `crates/game-cartridge-renderer`; `client/qml/cartridge`; Ticket 016 | `scripts/test-game-cartridge-renderer.sh`; schema/action/resource rejection, keyboard/accessibility/fixed states, and constrained Core/Rich-2D measurements |
| Separate-repository SDK/release | `crates/game-cartridge/src/sdk.rs`, `release.rs`, `lifecycle.rs`, `secure_store.rs`; Ticket 017 | `scripts/test-game-cartridge-sdk.sh`; deterministic export, clean-clone reproducibility, signed provenance/policy, lifecycle matrix, descriptor-relative import, rollback/race/permission rejection |
| Provider security foundation | authority/threat model; Ticket 018 | TLS and sender authentication, registered egress, grant/replay/key/quotas/audit/failure tests |
| Remote authority migration | Constitution §10; ADR-0002; Ticket 019 | One durable gameplay owner, explicit migration/rollback/reconciliation, accepted constitutional change |

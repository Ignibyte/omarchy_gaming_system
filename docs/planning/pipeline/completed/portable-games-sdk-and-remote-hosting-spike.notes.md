---
title: Portable games SDK and remote hosting architecture spike — notes
pipeline_id: cc5d1f80-b2cc-4bb7-929e-657b1e26f761
---

# Portable games SDK and remote hosting architecture spike — running notes

Chronological evidence and decisions. If a test or command did not run, these
notes must not claim it passed.

## Phase 1 — Plan

- User direction: games should be portable, live in separate repositories, and
  eventually target an OmarchyGS SDK. OmarchyGS should own shared platform
  identity, personas/avatars, achievements, social services, and provider
  access; a future remote provider should own its server-side game.
- User question: a remote backend alone is insufficient unless OmarchyGS also
  defines how the independently owned game frontend is delivered, embedded, or
  rendered. Frontend integration is therefore a first-class spike requirement.
- Bulletin recall: `BUL-001-initial-push-pending` remains an active warning.
  The local `main` branch contains the new milestone commit, but the renamed
  GitHub repository still has no tracked `main`; this does not block local
  planning and no push was authorized in this request.
- Workflow preflight: no prior active spec/notes pair existed; Ticket 013 was
  closed; Ticket 014 was the next number; and the pinned CodeGraph 1.5.0 and
  OpenWiki 0.3.3 tools passed `scripts/check-pipeline-tools.sh`.
- Architecture recall: Constitution §10 currently makes the OmarchyGS server
  authoritative for game state, turns, time, randomness, rewards, and
  permissions. Delegating gameplay authority to a provider is a real future
  architecture change, not an implementation detail, and requires an accepted
  ADR plus a later constitution amendment.
- Runtime recall: `crates/game-runtime` exposes a database-free
  `GameDefinition` trait and immutable exact-version `GameRegistry`. Compiled
  definitions receive bounded JSON state, actor seat, and command; they receive
  no database, network, clock, account/session identity, or ambient randomness.
- Persistence recall: Tickets 012 and 013 pin each durable session to an exact
  compiled key/version, persist the platform-owned snapshot and revision, and
  commit idempotency receipts plus minimal participant invalidations inside one
  PostgreSQL transaction. A remote model must decide which of these remain
  platform records and which become provider-owned without dual authority.
- Product recall: the private-alpha charter intentionally excludes
  user-supplied native plugins and browser clients and labels a sandboxed SDK as
  a later decision. The spike may recommend future changes but must not smuggle
  them into the current alpha boundary.
- Smallest honest spike: compare four execution models, threat-model the
  provider boundary, evaluate three frontend families, specify an SDK/package
  lifecycle, exercise one isolated cross-process launch/command/result flow,
  and record an ADR plus sequenced implementation tickets. Do not enable a
  production provider endpoint or add speculative persistence.
- Locked safety boundary: providers receive only explicit, short-lived,
  audience-bound persona capabilities. They never receive a reusable OmarchyGS
  Bearer token, account identity, credential, or database connection. Results,
  achievement claims, and callbacks must be authenticated, replay-safe,
  bounded, and attributable to a registered provider/version.
- User confirmation: the retro **Game Cartridge** framing is now the primary
  frontend direction. OmarchyGS owns the launcher and executable renderer; a
  game cartridge supplies signed declarative presentation data and assets.

## Phase 2 — Design

- Architecture: documented the complete proposed contract in
  `docs/architecture/game-cartridges.md`. A separate game repository produces
  a signed immutable cartridge and a provider artifact. OmarchyGS verifies and
  installs the cartridge, renders its data-only screen templates through
  trusted QML components, brokers declared actions to the registered provider,
  and records authenticated results/achievements plus durable sync hints.
- Trust boundary: raw cartridge QML, JavaScript, native code, scripts, dynamic
  URLs, imports, filesystem, clipboard, process, and direct network access are
  rejected. Screen layouts are pinned in the signed cartridge; the provider
  returns only schema-validated view data, preventing an unreviewed remote UI
  swap or platform credential prompt.
- Authority split: OmarchyGS owns accounts, persona/avatar projection, social
  state, catalog/launch policy, provider trust, session envelope, achievements,
  notifications, and audit. A future remote provider owns game rules, state,
  turns, time, randomness, and results. Platform and provider must not maintain
  competing authoritative gameplay snapshots/revisions. The ADR must reconcile
  this with the current local-authority constitution before production work.
- Network flow: client actions always enter the authenticated OmarchyGS API.
  The server resolves the operator-registered endpoint, sends a short-lived
  audience/game/version/session/scope-bound pairwise persona grant, and applies
  provider responses/callbacks only after signature, expiry, replay,
  idempotency, revision, body, and policy checks. Device tokens, account IDs,
  credentials, and database access never cross the boundary. WebSockets remain
  hints over the existing persona cursor recovery model.
- Cartridge lifecycle: canonical integrity index and asymmetric publisher
  signature; bounded streaming download; rejection of traversal, links,
  duplicates, compression bombs, unexpected content, and unsupported
  capabilities; bounded schema/asset parsing; atomic content-addressed
  read-only installation; exact session pinning; and independent publisher,
  provider, key, or release revocation.
- Graphics direction: Cartridge Core covers terminal text, panels, menus,
  forms, lists, grids, boards, images, keyboard focus, and state surfaces. Rich
  2D adds tile maps, sprites, cards, vector primitives, local timelines,
  particles, platform effects, and sound/music. Advanced 2D/2.5D and bounded
  video are opt-in host capabilities. Constrained Qt Quick 3D and isolated
  WebEngine are future profiles with separate dependency, patch, licensing,
  hardware, and threat reviews—not baseline escape hatches.
- Graphics ceiling: local Qt rendering can comfortably target visually rich
  card/board games, roguelikes, asynchronous RPGs, strategy/management games,
  visual novels, puzzles, animated maps, and elaborate retro successors. The
  intended contract excludes Halo-class first-person rendering, high-frequency
  physics and twitch networking, a general game engine, arbitrary shaders, and
  per-frame provider round trips. Local cosmetic animation remains independent
  of authoritative provider updates.
- Capability and budget contract: the host publishes supported presentation
  profiles and limits; the cartridge declares required and optional
  capabilities plus fallbacks. Limits cover compressed/expanded bytes, file
  count, decoded pixels, texture dimensions, nodes, sprites, particles,
  simultaneous animations, audio, state/command payloads, memory, and frame
  time. Exact numbers remain a measured proof output on minimum supported
  Omarchy hardware rather than guessed protocol constants.
- Local platform evidence: `qml6 --version` reports 6.11.2. Qt Base,
  Declarative, Multimedia, and WebEngine 6.11.2 are installed; Qt Quick 3D is
  absent. The current trusted `Main.qml` is a single health client that uses
  direct `XMLHttpRequest`; it loads no third-party content and is not yet a
  cartridge host.
- Qt research: official Qt 6.11 documentation confirms that Qt Quick has a
  retained graphics-API-backed scene graph plus animation, sprites, particles,
  effects, audio/video extensions, and optional mixed 2D/3D. Qt's security
  documentation explicitly says QML/JavaScript are trusted application code,
  arbitrary untrusted QML is unsupported, and a custom DSL is the appropriate
  alternative. Qt also warns that large frequently updated Canvas images incur
  texture uploads and recommends profiling to preserve the common ~16 ms frame
  budget at 60 FPS.
- Protocol research: RFC 9700 supports asymmetric client authentication,
  sender-constrained and audience/privilege-restricted short-lived tokens,
  replay defenses, and TLS. RFC 9068 provides relevant `aud`, `exp`, `jti`,
  client, and scope validation. RFC 9421 provides an option for signing the
  covered HTTP message components with timestamps, expiry, and nonces while
  retaining TLS for confidentiality. The proof must select and document a
  narrow application profile instead of composing these loosely.
- CodeGraph design evidence: exploration for pipeline
  `cc5d1f80-b2cc-4bb7-929e-657b1e26f761` traced `GameRegistry` in `AppState`,
  the catalog/session/command handlers, `games::create_session`,
  `games::apply_command`, and `sync::append_event`. Clean future seams are an
  explicit execution/provider abstraction beside the compiled registry, a
  separate presentation catalog identity, and reuse of owner/participant REST
  authorization plus minimal sync invalidation. CodeGraph's heuristic again
  reported no test link for the command handler despite the directly inspected
  PostgreSQL API module; graph coverage remains advisory. QML and docs were
  inspected directly.

### File manifest for the spike proof

| Path | Purpose |
|---|---|
| `docs/architecture/game-cartridges.md` | Preserve the proposed product, package, renderer, provider, security, graphics, SDK, and rollout plan; refine it with proof findings. |
| `docs/architecture/adr-0002-game-cartridge-and-provider-boundary.md` | Record the final recommendation, authority transition, alternatives, constitution impact, and adoption decision. |
| `crates/game-cartridge-spike/Cargo.toml`, `Cargo.lock` | Isolate a non-production Rust proof and its exact dependencies from the product workspace while keeping every proof artifact inside the delivery hash. |
| `crates/game-cartridge-spike/src/lib.rs` | Define the candidate manifest/view/action/grant/result contracts, bounds, signing helpers, and validation used by both processes. |
| `crates/game-cartridge-spike/src/bin/provider.rs` | Run an external fixture provider that owns one game session and revision-aware idempotent command. |
| `crates/game-cartridge-spike/src/bin/broker.rs` | Model the trusted OmarchyGS broker, scoped persona grant, registered endpoint, callback validation, replay defense, and sanitized presentation response. |
| `crates/game-cartridge-spike/fixtures/cartridge/` | Hold one deterministic signed-style manifest, declarative terminal/board screen, schemas, and tiny bounded assets without executable content. |
| `crates/game-cartridge-spike/qml/CartridgeProof.qml` | Render the sanitized fixture view through a trusted proof-only QML component vocabulary and demonstrate keyboard launch/action plus failure states. |
| `crates/game-cartridge-spike/tests/` | Exercise launch/command/result across separate broker/provider processes plus invalid audience, expiry, signature, replay, revision, endpoint, schema, capability, and resource-limit cases. |
| `scripts/test-game-cartridge-spike.sh` | Run proof formatting/tests and the cross-process/QML smoke path with temporary keys/state and deterministic cleanup. |
| Ticket/spec/notes/AAR/knowledge/OpenWiki | Preserve requirement evidence, lessons, final decision, follow-up tickets, and generated documentation. |

The proof will not change product routes, migrations, the current
`GameRegistry`, or the production QML entrypoint. It is intentionally isolated
and removable until the ADR is accepted.

### Regression and evidence plan

| Requirement | Evidence |
|---|---|
| REQ-001 | Decision matrix compares compiled local, separate-repo compiled, sandboxed local, and remote provider models and ties the staged recommendation to current seams and operating costs. |
| REQ-002 | Threat-model sequence and proof validate registered provider identity, scoped/audience-bound expiry, pairwise persona subject, key rotation/revocation shape, allowlisted endpoint, and absence of device/account/database material. |
| REQ-003 | Authority matrix plus separate-process proof demonstrate one provider-owned revision, idempotent replay, authenticated result, failure/retry behavior, and platform sync envelope without duplicate gameplay authority. |
| REQ-004 | Cartridge schema/proof renderer rejects executable/import/network constructs and demonstrates trusted keyboard/accessibility/loading/offline/error presentation; option matrix records WebEngine/Wasm/native tradeoffs. |
| REQ-005 | Independent spike workspace, fixture repository layout, compatibility axes, packaging/signing flow, and conformance cases form the SDK lifecycle proposal. |
| REQ-006 | Script starts provider and broker as distinct processes, launches a scoped persona, applies one revision-aware idempotent action, validates one signed result/event, and executes negative credential/database boundary checks. |
| REQ-007 | ADR records recommendation and constitution conflict; current-code gap map plus newly numbered follow-up tickets sequence any production SDK, renderer, broker, migration, and first-game work. |
| REQ-008 | Capability matrix, proof fixture, measured frame/resource evidence, limit rejection tests, unsupported-required capability failure, and optional/reduced-motion fallback exercise the graphics envelope. |

### Alternatives and boundaries

- Raw QML/JavaScript was rejected as a cartridge format: Qt treats it as
  trusted native-application content, it can load local/remote resources, and
  it does not create privacy domains.
- Provider-hosted WebEngine content remains a compatibility tier, not the
  default. Although Qt WebEngine isolates renderer processes, it adds a large
  continuously patched Chromium surface and weakens native consistency.
- WebAssembly alone does not solve presentation: it can sandbox computation but
  still needs a capability ABI and trusted renderer. Evaluate it later for
  local rules or advanced drawing, not as permission to bypass cartridge data
  contracts.
- Provider-returned dynamic UI trees were rejected for the baseline. A signed
  template plus validated view model prevents the backend from replacing
  reviewed UX after publication.
- Direct client-to-provider calls were deferred. A platform broker costs one
  hop but centralizes credentials, egress policy, authorization, rate limits,
  audit, retries, and revocation.
- Arbitrary custom shaders and fonts were rejected from the initial profile;
  both expand parser/GPU attack surface and portability variance. Platform-owned
  named effects and fonts preserve the visual vocabulary safely.
- Exact numeric graphics limits were not invented during desk design. The
  cross-process/QML proof must measure and record defensible baseline values.

## Phase 3 — Implement

- Built an isolated nested Rust workspace under
  `crates/game-cartridge-spike/`. It signs and verifies a strict data-only
  cartridge with Ed25519, canonical safe paths, exact file digests, bounded
  package/view/body/schema records, a capability/action allowlist, and explicit
  rejection of links, QML, JavaScript, native libraries, URL files, malformed
  identifiers, over-scoped/expired/wrong-audience grants, and unregistered
  proof endpoints.
- Built a loopback-only provider process that owns game session state and
  revision, verifies ephemeral platform grants, consumes grant replay IDs,
  checks the pairwise subject, resolves idempotency receipts before current
  revision, and signs launch/command results. It receives no account ID,
  device token, database handle, or platform credential.
- Built a loopback-only broker process that verifies the signed cartridge on
  startup, derives a provider/game pairwise persona subject, issues separate
  one-scope 60-second launch and command grants, uses a no-proxy/no-redirect
  bounded HTTP client, validates provider signatures and exact pinned
  identities, retries the same command/idempotency key with a fresh grant, and
  rejects the duplicate signed event before returning a sanitized view.
- Built a trusted proof-only QML renderer with platform-owned terminal, grid,
  and status components. It interprets the signed presentation vocabulary,
  supports keyboard retry/exit, exposes loading/offline/protocol-error states,
  keeps animation local, checks broker privacy/replay assertions, and never
  evaluates cartridge code or follows cartridge/provider URLs.
- Built `scripts/test-game-cartridge-spike.sh` to format/test/build the isolated
  workspace, generate ephemeral mode-0600 keys, sign a copied fixture, start
  broker/provider as separate child processes, run a Rust contract probe, run
  the QML surface offscreen, reject QML runtime contract errors, capture frame
  and resident-memory evidence, and clean up exact PIDs/temporary data.
- The latest Phase 3 proof run passed seven Rust tests and the cross-process
  QML flow. It measured 120 frames at 15.99 ms average/17.00 ms maximum, 88,312
  KiB peak QML RSS, and a four-file 2,436-byte expanded signed fixture. These
  validate the harness, not the final minimum-hardware Rich-2D profile.
- Recorded ADR-0002 with the staged decision and Constitution §10 conflict,
  expanded `game-cartridges.md` with execution/frontend matrices, authority,
  failure/retry/reconciliation behavior, graphics profiles, proof evidence,
  provisional conformance ceilings, threat model, SDK lifecycle, and rollout,
  and opened sequenced Tickets 015–019.
- Deviations: moved the originally planned `spikes/game-cartridge/` proof to
  `crates/game-cartridge-spike/` so every source, fixture, and QML artifact is
  included in the repository's gated worktree hash. The crate remains a nested
  Cargo workspace and is not linked into the production server. The proof uses
  an unpacked directory rather than claiming a final archive encoding, and its
  ephemeral signed envelope/loopback HTTP/in-memory replay state deliberately
  stop short of a production cryptographic or persistence profile.
- Gate ratchet: the isolated nested Cargo workspace was initially outside the
  canonical repository quality loop even though its source was worktree-gated.
  Added executable `scripts/test-game-cartridge-spike.sh` as gate 11 in every
  `bin/gate.sh` mode and reconciled Constitution gates 11--13 so future delivery
  receipts cannot omit the architecture proof.

## Phase 3.5 — Inspect ledger

| # | Lens | Finding | Severity | Disposition |
|---|---|---|---|---|
| 1 | Authenticated input / correctness | Cartridge verification hashed files and later reopened them for parsing, leaving a same-user time-of-check/time-of-use seam between the authenticated bytes and interpreted bytes. | Medium for a future installer; low in the isolated proof | Remediated: verification now reads each file once, hashes those bytes, retains them in a verified package, and parses only the retained authenticated bytes. |
| 2 | Resource exhaustion / provider boundary | The broker bounded the response only after `reqwest` had buffered it. A future hostile provider could consume memory before rejection. | Medium for production remote providers; low for the loopback proof | Remediated: the broker streams response chunks, rejects as soon as the response cap would be exceeded, and only then parses the bounded buffer. |
| 3 | Untrusted presentation / QML | Provider-derived `Text` values relied on Qt's automatic text-format detection, allowing markup-like text to be interpreted rather than displayed literally. | Medium for a production renderer; low in the fixed fixture | Remediated: every provider-derived `Text` surface explicitly uses `Text.PlainText`; the cartridge remains declarative data rendered by trusted QML. |
| 4 | Filesystem / package bounds | The verifier bounded accepted files but directory traversal itself was not bounded, so empty or deeply nested directories could increase work without entering the file-count budget. | Medium for a future installer; low in the isolated fixture | Remediated: only the root plus the allowlisted `assets`, `locales`, and `schemas` directories are accepted; entry count and depth are bounded and nested/unexpected directories reject. |
| 5 | Identity / authorization | The proof checked signatures and envelope identities but did not make the operator-registered publisher/provider identifiers an explicit startup binding. | Low now; medium before production | Remediated: broker configuration requires the registered publisher and provider IDs and rejects a signed cartridge or provider response that does not exactly match them. |
| 6 | Replay / attribution | Result receipts did not carry the pairwise subject, and a grant at exactly its expiry boundary was accepted. | Low | Remediated: signed receipts bind the pairwise subject; expiry is exclusive and exact-boundary grants reject; broker validation checks both. |
| 7 | Evidence integrity | The canonical gate covered the production workspace but not the nested proof workspace, allowing the primary spike evidence to drift after validation. | Medium | Remediated: gate 11 runs the full cartridge proof in fast and diff modes; the Constitution enumerates the ratcheted 13-gate contract. |
| 8 | Security scan | Formal Codex Security working-tree diff scan `887ffea4-3265-409e-952e-8241fa49647f` completed with complete coverage and no reportable deployed vulnerability. It surfaced findings 1--4 as future-boundary hardening candidates rather than current vulnerabilities because the proof is isolated, loopback-only, and not wired to production. | Informational | Accepted the severity calibration and remediated all four candidates before phase exit. Scan usage: 5,260,903 total tokens (5,242,647 input, 5,156,608 cached input, 18,256 output, 6,487 reasoning). |
| 9 | Correctness / portability / operations | Final CodeGraph pass traced the changed proof, script, and gate integration; direct review covered unsupported QML, fixtures, shell lifecycle, docs, and tests. No unresolved correctness, privacy, game-state, complexity, or portability defect remained. | Informational | Fresh inspect receipt matches gated state `160e9c3c099ea3d306c3300f3b6df90c83b1a96d9f83ad4ba4c06126616a2c4f`; `git diff --check` and `bin/gate.sh --fast` passed. |

## Phase 4 — Validate

- Tests run: `bin/gate.sh --fast` passed after the final inspection fix. It
  exercised rustfmt, Clippy with warnings denied, 5 runtime tests, 30
  database-free server tests, rustdoc with warnings denied, Compose and shell
  validation, pipeline structure, changed-file secret scan, Codex hook
  self-tests, whitespace checks, and the complete Game Cartridge proof with 7
  tests plus its broker/provider/probe/QML flow.
- Gate run: `bin/gate.sh --diff` printed `GATE GREEN [diff]` across all 13
  gates. In addition to the fast evidence, all 33 PostgreSQL tests passed and
  the PostgreSQL + Rust API + visible QML smoke passed. The final cartridge
  sample rendered 120 frames at 15.99 ms average and 17.00 ms maximum, used
  88,184 KiB peak QML RSS, and verified a four-file, 2,436-byte expanded signed
  fixture. The delivery, inspection, and current gated-state hashes all match
  `160e9c3c099ea3d306c3300f3b6df90c83b1a96d9f83ad4ba4c06126616a2c4f`.
- Skips or pre-existing failures: none. The ordinary workspace test pass
  correctly skipped 33 PostgreSQL-required cases; the diff gate then ran and
  passed those exact 33 cases against PostgreSQL. Mesa emitted two non-fatal
  `failed to create dri2 screen` warnings during the QML smoke, which completed
  successfully through the supported fallback path.

## Phase 5 — Complete

- Acceptance-criteria audit:
  - REQ-001 PASS: `game-cartridges.md` compares compiled, separate-repository
    compiled, sandboxed-local, and remote-provider execution across authority,
    isolation, release, latency/offline, compatibility, and operations, and
    records the staged target in ADR-0002.
  - REQ-002 PASS: the threat model and proof bind registered publisher/provider
    identity, audience, exact game/release/session, one scope, pairwise subject,
    expiry, replay ID, and endpoint policy while privacy assertions prove no raw
    persona, account, reusable device token, credential, or database access is
    returned.
  - REQ-003 PASS: the authority matrix, retry/reconciliation contract, and
    separate provider proof assign one provider revision, idempotent receipt,
    signed events, outage behavior, and REST/cursor recovery without creating a
    second production authority or treating WebSockets as durable truth.
  - REQ-004 PASS: the signed data-only cartridge and trusted QML proof prohibit
    game code/networking, render keyboard/accessibility/loading/offline/error
    states, and the frontend option matrix records QML, WebEngine, Wasm, and
    native-plugin containment tradeoffs.
  - REQ-005 PASS: the protocol-first SDK model specifies manifest/schema,
    compatibility axes, capability negotiation, adapters, conformance fixtures,
    provenance, local preview, registration, rollout, suspension, revocation,
    retirement, and the same public contract for first-party games.
  - REQ-006 PASS: `scripts/test-game-cartridge-spike.sh` starts separate broker
    and provider processes, launches one pairwise persona, advances revision
    zero to one, retries the same idempotency key, validates a signed result and
    duplicate-event rejection, renders trusted QML, and executes the negative
    privacy checks.
  - REQ-007 PASS: ADR-0002 explicitly preserves Constitution §10 today, maps
    current seams and the future amendment/migration, and Tickets 015--019
    sequence the contract, renderer, separate repository, production security,
    and remote-authority pilot without exposing a production provider route.
  - REQ-008 PASS: Core, Rich 2D, Advanced 2D/2.5D, future 3D, and isolated Web
    profiles document compatibility and fallback behavior. The proof records
    package/scene limits plus 120-frame, memory, and package measurements, and
    Ticket 016 owns minimum-hardware stress calibration before publication.
- Docs: ADR-0002 and the comprehensive hand-maintained Game Cartridge
  architecture capture the decision, trust/authority model, protocol,
  graphics ceiling, provisional budgets, SDK lifecycle, and rollout. Final
  OpenWiki update `a719e1c6-54bd-470c-b332-265a40a04416` completed without
  warnings and reconciled quickstart, product boundaries, validation, and the
  new generated Game Cartridges page. Its completion receipt matches gated
  state `c1e50be8ed5518601117dd54046cab4dd555711cdc15218df22792cfe6833fa0`.
- AAR: submitted 2026-08-25 at effectiveness 5/5 with six failures, six
  prevention rules, and
  `AD-omarchy-gaming-system-game-cartridge-provider-boundary-001`; every new ID
  is present in the knowledge register.
- Archive: Ticket 014 closed, the roadmap spike checked, the open queue row
  removed, and this sole active spec/notes pair moved to `completed/`. Tickets
  015--019 remain open in the accepted sequence.
- Final delivery proof: `bin/gate.sh --diff` passed all 13 gates after the last
  OpenWiki and archive work. Its cartridge sample recorded 120 frames at 16.01
  ms average/18.00 ms maximum, 88,004 KiB peak QML RSS, and the same four-file,
  2,436-byte signed fixture. The delivery-gate and OpenWiki completion receipts
  both match the current gated state
  `c1e50be8ed5518601117dd54046cab4dd555711cdc15218df22792cfe6833fa0`.

## Defect and lesson ledger

| # | What happened | Root cause | Fix | Prevention |
|---|---|---|---|---|
| 1 | Initial offscreen QML runs completed without exposing console metrics to the shell harness. | Qt's ordinary console route was not reliably visible in this headless invocation. | Enabled `QT_FORCE_STDERR_LOGGING=1`, retained bounded log capture, and made missing metrics a hard failure. | `BF-omarchy-gaming-system-qml-proof-log-routing-001`: smoke harnesses must explicitly route and assert the signal they use as evidence. |
| 2 | The first isolated Cargo run created a large nested `target/` inside the new proof workspace. | The nested workspace used Cargo's default target before the shared target path was pinned. | Moved the generated directory to Trash, added the nested target to `.gitignore`, and exported `CARGO_TARGET_DIR=target/game-cartridge-spike` in the proof script. | Keep generated workspace outputs in an already ignored shared target and verify untracked status after the first build. |
| 3 | The required proof passed independently but the canonical gate did not initially execute it. | Gated-file hashing and executable validation were treated as the same coverage question. | Added the proof as gate 11 in every mode and reconciled the Constitution's 13-gate list. | `PR-omarchy-gaming-system-gate-every-nested-workspace-proof-001`. |
| 4 | The first OpenWiki finish completed with a warning that one new page's claim evidence changed after staging; a later clean finish after archival could not refresh the pipeline receipt. | The final measured architecture value and roadmap closeout were reconciled after the initial Grounded Claims operation, and the active pair had already been archived before the last receipt attempt. | Restaged claims after the final source edit, temporarily restored the completed active pair at the Phase 4 → Phase 5 transition, finalized with zero issues, verified the matching receipt, then re-archived. | Resolve factual claims after the last source edit, finalize while the pipeline is active at the completion transition, and treat lifecycle warnings as actionable even when status is complete. |

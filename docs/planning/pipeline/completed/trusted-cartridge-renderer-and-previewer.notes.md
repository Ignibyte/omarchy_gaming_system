---
title: Trusted Cartridge renderer and previewer — notes
pipeline_id: 6494fb94-54e8-4fcf-b50d-9220c46f4564
---

# Trusted Cartridge renderer and previewer — running notes

Chronological evidence and decisions. If a test or command did not run, these
notes must not claim it passed.

## Phase 1 — Plan

- Ticket 015 is Phase 5 complete and left no active pipeline. Its production
  crate verifies inert cartridge bytes but deliberately has no renderer,
  catalog ingestion, provider network, or game-authority change.
- Bulletin `BUL-001-initial-push-pending` remains a warning. Work may continue
  locally; no commit or push is authorized.
- Recalled ADR-0002 and the Ticket 014/015 notes: OmarchyGS owns every executable
  QML component; publisher strings stay plain text; authenticated bytes are not
  reopened; package/decoder/scene work is bounded; WebSockets/provider work is
  unrelated to local cosmetic animation.
- Recalled Ticket 015 inspection rules: validate the exact decoder profile,
  bind each presentation node to a capability, read bounded input from checked
  handles, and keep privileged/multi-user storage out until descriptor-relative
  containment exists.
- The user-facing graphics answer is now explicit: the intended sweet spot is
  rich retro through polished 2D, not Halo, a general engine, publisher code,
  high-frequency physics, or twitch networking. Ticket 016 owns the measured
  evidence for that claim.
- Smallest useful slice: a Rust render-plan compiler plus trusted QML previewer
  renders Core and a bounded Rich-2D foundation from the same verified package;
  it does not add a launcher or remote server path.

## Phase 2 — Design

- Architecture and data flow:
  1. The production cartridge crate verifies the bounded `.ogsc`, signature,
     canonical content, manifest, presentation, schemas, media, and host
     compatibility exactly as Ticket 015 established.
  2. `VerifiedCartridge` retains authenticated payload bytes behind read-only
     accessors. Each screen names one manifest-declared view schema. The new
     renderer crate validates one bounded provider/developer view instance
     against that restricted schema before resolving any binding.
  3. The Rust compiler maps only typed presentation variants to a versioned
     `RenderPlan`, applies profile limits and optional-capability fallbacks,
     converts publisher asset paths to digest-based tokens, and emits fixed
     platform state/origin chrome. It never emits QML source, expressions,
     markup, arbitrary URLs, or filesystem paths.
  4. The preview command writes the canonical plan plus authenticated assets
     into a caller-selected private temporary directory. Asset filenames are
     derived from SHA-256 plus an allowlisted extension, never from cartridge
     input.
  5. `client/qml/cartridge/CartridgePreview.qml` loads the trusted plan from the
     explicit preview directory, rechecks its outer format/kinds/counts, and
     maps tags only to repository-owned `sourceComponent` values. Components
     build asset URLs from the separately trusted directory plus digest token.
  6. Interactive Grid cells and Buttons emit declared action IDs and bounded
     arguments to the preview log. The previewer labels them requested only;
     it has no server/provider path and cannot report a game command accepted.
- Vocabulary and fallbacks:
  - Core capabilities: `presentation.terminal.v1`, `grid.v1`, `status.v1`,
    `button.v1`, `image.v1`, and `meter.v1`.
  - Rich-2D capabilities: `presentation.sprite.v1`,
    `presentation.particles.v1`, and `audio.effects.v1`.
  - Terminal/Status require bounded string bindings; Grid requires an exact
    rows × columns string array; Button requires a string label and declared
    action; Meter requires a finite numeric value inside its signed range.
    Image/Sprite/AudioCue reference an exact declared authenticated asset.
  - Missing optional Image uses omit or platform placeholder. Sprite accepts
    omit, static/reduced-motion, platform placeholder, or a simpler Image
    capability. ParticleField accepts omit/static/reduced-motion. AudioCue
    accepts omit or muted. Invalid node/fallback combinations reject rather
    than improvising.
- Initial profile budgets, subject to executed benchmark evidence:

  | Resource | Core | Rich 2D | Failure/degrade rule |
  |---|---:|---:|---|
  | View-model JSON / render-plan JSON | 256 KiB / 1 MiB | 512 KiB / 2 MiB | Reject before QML publication |
  | Active screen nodes / grid cells | 256 / 1,024 | 512 / 4,096 | Required content rejects; optional decoration drops deterministically |
  | Images / sprites / particles / audio cues | 32 / 0 / 0 / 0 | 64 / 128 / 2,048 / 16 | Apply signed fallback before QML instantiation |
  | Simultaneous local animations | 32 | 128 | Reduced-motion disables all nonessential animation |
  | Authenticated decoded assets | Inherit 128 MiB v1 hard total | Inherit 128 MiB v1 hard total | Reject before renderer; Ticket 015 decoder rules stay binding |
  | Surface RSS | 256 MiB soft / 384 MiB hard | 384 MiB soft / 512 MiB hard | Benchmark fails above hard; production shell may suspend later |
  | Software frame time | 16.67 ms target / 33.3 ms degraded | 16.67 ms target / 33.3 ms degraded | Preserve focus/input; lower optional counts if repeatable evidence misses degraded bound |

- Reference benchmark environment: the current KVM guest exposes six vCPUs
  from a 12th Gen Intel i9-12900K, 11 GiB RAM, Virtio GPU, Linux 7.1.8, and Qt
  6.11.2. The focused benchmark will force `QT_QUICK_BACKEND=software`, offscreen
  920×600 rendering, one-CPU affinity when available, and explicit RSS/frame
  capture. This is the first constrained reference profile, not evidence for an
  untested low-end physical GPU.
- State and trust contract:
  - `loading`, `offline`, `stale`, `empty`, `protocol_error`,
    `unsupported_capability`, and `revoked` map to fixed platform labels,
    colors, and actions; only `ready` instantiates cartridge nodes.
  - Publisher/game/version/digest origin remains visible in all states.
    Authentication/MFA text inputs do not exist in the cartridge vocabulary.
  - All rendered strings use `Text.PlainText`; high contrast, scale, reduced
    motion, and mute are trusted preferences outside the cartridge.
- Official Qt 6.11 review confirmed that `AnimatedSprite` takes a platform URL
  plus bounded frame geometry/rate, `QtQuick.Particles` is a separate built-in
  module with explicit system running state, `MediaPlayer` routes through an
  `AudioOutput`, `FrameAnimation` exposes per-frame timing, `Keys` owns keyboard
  handling, and interactive items must expose focus plus `Accessible` role,
  name, focused state, and press action. The repository has QtQuick.Particles
  and QtMultimedia installed; no raw shader or Canvas path is selected.
- Database/API effects: none. The previewer reads explicit local inputs and
  writes only its explicit output directory. The server, migrations, durable
  game authority, REST/WebSocket contracts, and existing health-only main QML
  entrypoint remain unchanged.
- Planned file manifest:

  | Path | Purpose |
  |---|---|
  | `crates/game-cartridge/src/contract.rs`, `archive.rs`, `validate.rs`, new view accessor module, tests/fixtures | Bind screens to schemas, add versioned Core/Rich nodes and capability/fallback checks, retain authenticated files, and expose safe read-only content to the renderer. |
  | `crates/game-cartridge-renderer/Cargo.toml`, `src/lib.rs`, focused modules | Own view-schema instance validation, binding resolution, profiles/budgets, fallback lowering, asset tokens, fixed states, render plans, and reports. |
  | `crates/game-cartridge-renderer/src/bin/omarchygs-cartridge-preview.rs` | Verify explicit inputs and prepare a private plan/asset directory with machine-readable output and stable exits. |
  | `crates/game-cartridge-renderer/tests/` and fixtures | Prove schema/binding/action/fallback/budget/origin/state/resource behavior and exact plan output. |
  | `client/qml/cartridge/CartridgePreview.qml`, `TrustedCartridgeSurface.qml`, `nodes/*.qml` | Instantiate only platform-owned components, trusted state/origin chrome, plain text, deterministic focus/actions, and preference fallbacks. |
  | `scripts/test-game-cartridge-renderer.sh` | Run Rust conformance, generate real signed Core/Rich fixtures, exercise every trusted QML state, and capture constrained frame/RSS/profile evidence. |
  | `Cargo.toml`, `Cargo.lock`, `bin/gate.sh`, `CONSTITUTION.md` | Add the production renderer crate and a canonical renderer/preview gate without weakening existing gates. |
  | Ticket/AAR, `docs/architecture/game-cartridges.md`, benchmark report, README, OpenWiki | Publish exact implemented vocabulary, honest reference environment/results, limitations, and next extension seam. |
- Regression map:

  | Requirement | Evidence |
  |---|---|
  | REQ-001 | Correctly signed fixtures for every node compile to exact allowlisted plan tags; malformed schema/view/binding/asset/action/capability inputs reject; QML asserts no dynamic source/import/expression path. |
  | REQ-002 | QML smoke matrix proves Tab/arrow/Enter/Space/pointer-equivalent actions, accessible IDs/roles/names/focus, scale/high-contrast, reduced animation, and muted audio. |
  | REQ-003 | Rust boundary fixtures exceed every node/view/asset/effect count; Core/Rich stress plans run with software rendering and one-CPU affinity while capturing frames/RSS and deterministic optional degradation. |
  | REQ-004 | Every fixed state renders trusted origin/status chrome with zero cartridge nodes and no editable/password component; revoked/unsupported cannot request actions. |
  | REQ-005 | End-to-end preview uses the production verifier/compiler, explicit files only, unusable network/database/credential environment values, private output modes, and machine-readable report. |
  | REQ-006 | Versioned benchmark report records OS/Qt/CPU/GPU/backend/affinity/resolution, exact fixture counts, measured frame/RSS values, ratified budgets, realistic game examples, and limitations. |
- Risks and rollback:
  - QML plan or asset substitution is contained to an explicit same-user
    preview directory and revalidated at both Rust and QML boundaries; no
    privilege claim is made.
  - Real Qt image/audio decoder and particle paths can crash or exhaust the QML
    process. Strict Ticket 015 formats, pre-QML budgets, decoder-state smoke,
    and a separate preview process reduce impact; full client-process isolation
    remains a later hardening choice.
  - Frame/RSS tests can be noisy. The gate records metrics and enforces generous
    hard ceilings; profile publication uses repeated measurements and lowers
    optional counts rather than hiding red evidence.
  - Audio devices may be absent in CI/offscreen mode. Muted instantiation is
    mandatory; unmuted decode/playback is tested when the backend is available
    and reported honestly otherwise.
  - Rollback removes the new renderer crate/QML/script and additive node
    variants while leaving Ticket 015 canonical packages and server authority
    intact. No migration rollback exists because no database changes occur.
- CodeGraph design exploration traced `verify_archive_bytes` into authenticated
  manifest/presentation/media validation and the resulting
  `VerifiedCartridge`, then mapped `PresentationNode`, `HostProfile`,
  compatibility, store resolution, current production callers, conformance
  tests, and the isolated proof renderer. It confirms the additive blast radius
  is the production cartridge crate/tests plus a new independent renderer and
  QML preview path; no server/game-runtime caller constructs these types.
  CodeGraph does not parse QML reliably, so the health connector, proof QML,
  shell launch/metrics flow, and installed Qt modules were reviewed directly.
  The Phase 2 receipt matches pipeline
  `6494fb94-54e8-4fcf-b50d-9220c46f4564` and the designed worktree.

## Phase 3 — Implement

- Extended the production cartridge contract so every screen pins one declared
  schema and typed Button/Image/Meter/Sprite/ParticleField/AudioCue nodes join
  Terminal/Grid/Status behind exact capabilities. Cross-validation rejects
  missing schemas, assets, actions, incompatible fallbacks, unsafe meter
  integers, and sprite sheets that cannot contain their declared frames.
- `VerifiedCartridge` now retains exact authenticated payload bytes and exposes
  them through read-only accessors. Its manifest, presentation, compatibility,
  archive identity, and payload are no longer externally mutable, closing a
  trust gap found during implementation review.
- Added the production `omarchygs-game-cartridge-renderer` workspace crate. It
  validates the restricted signed schema against a bounded view, resolves only
  dotted object bindings, applies signed optional fallbacks and trusted user
  preferences, charges Core/Rich-2D budgets, and emits only typed inert
  `omarchygs.render-plan/v1` tags. Authenticated media is published under
  SHA-256 `.png`/`.wav` tokens; unused over-budget optional assets are not
  emitted.
- Added `omarchygs-cartridge-preview prepare`. It reuses the production archive
  verifier and compiler, reads bounded checked regular files, requires an
  existing empty private output directory, writes create-only/read-only plan
  and asset files, and reports that no provider, database, or platform
  credential was used.
- Added trusted QML components for all nine node types plus a platform
  placeholder. `TrustedCartridgeSurface` revalidates the plan envelope, exact
  keys, types, limits, tokens, state, and signed origin before mapping tags
  through a fixed `sourceComponent` switch. Strings use `Text.PlainText`;
  Grid/Button share keyboard, pointer, and accessibility action methods; and
  motion, contrast, scale, and audio preferences stay platform-owned.
- Non-ready states render fixed state/origin chrome with an empty node model.
  Actions log `requested` and `confirmed=false`; the previewer has no path that
  can claim an authoritative game mutation.
- Added nine renderer Rust/CLI integration tests, one focused unit test, and
  expanded the cartridge
  suite from 19 to 20 tests. The signed fixture exercises every node, all
  fallback paths, schema/binding failures, immutable digest assets, private
  output, CLI isolation, all fixed states, preference behavior, repeated asset
  references, and early rejection of repeated large bindings.
- Added `scripts/test-game-cartridge-renderer.sh` and canonical gate 12. The
  harness generates real signed base/Core/Rich cartridges, forces Qt 6.11.2
  offscreen software rendering with one-CPU affinity when available, warms 60
  frames, samples 120, records peak RSS, enforces 33.3 ms average and profile
  hard-memory ceilings, exercises focus/actions, and runs every fixed state.
  The spike, database, and live application checks move to gates 13–15.
- Published focused profile evidence on KVM / six exposed i9-12900K vCPUs / 11 GiB
  RAM / Virtio GPU / Linux 7.1.8 / Qt 6.11.2, constrained to CPU 0:

  | Fixture | Load | Average / max frame | Peak RSS |
  |---|---|---:|---:|
  | Core stress | 256 nodes, 1,024 cells, 32 images | 16.001 / 17.799 ms | 121,152 KiB |
  | Rich-2D stress | 213 nodes, 64 images, 127 sprites, 2,048 particles, 16 audio, 128 animations | 16.004 / 16.813 ms | 232,328 KiB |
  | Accessibility/reduced motion | 9 nodes, 2× scale, high contrast, mute | 15.998 / 16.890 ms | 225,988 KiB |
  | Seven fixed states | zero game nodes | 15.992–16.005 ms average | 94,104–94,408 KiB |
- Focused Rust tests, package/CLI preparation, QML plan validation, interaction
  smoke, every fixed state, and the constrained benchmark are green. Full
  workspace/canonical validation remains Phase 4 work.

## Phase 3.5 — Inspect ledger

| # | Lens | Finding | Severity | Disposition |
|---|---|---|---|---|
| 1 | Resource accounting | Every asset reference was hashed and cloned before aggregate admission, so many optional nodes could repeat full-buffer work even after the profile limit was reached. | Low security / CWE-400 | Fixed: authenticated asset tokens are cached once per validated path; bytes are cloned and published once only after a referencing node passes admission. A unit test proves one digest per path and a 200-reference integration case publishes one buffer. |
| 2 | Resource accounting | The 1/2 MiB render-plan cap was first checked after all bound strings had been cloned into the plan and a full serialization allocated. | Low security / CWE-400 | Fixed: exact serialized-node bytes are counted with a non-allocating writer and charged before a node is retained; checked arithmetic and the final exact envelope check remain. A repeated 65,536-character binding regression fails with `BudgetExceeded`. |
| 3 | Cross-layer action integrity | Grid and Button validation required only an existing action ID, while platform components always emit `{column,row}` and `{}` respectively. | Future authorization boundary | Fixed before dispatch exists: Grid declarations must be exactly sorted `column,row`; Button declarations must be empty. Pack/verification regressions reject both mismatches. |
| 4 | QML defense in depth | The QML validator checked each node and total node count but did not independently sum the claimed profile's grid/image/sprite/particle/audio/animation budgets. | Same-user defense in depth | Fixed: QML mirrors aggregate Core/Rich-2D counters before instantiation. The focused harness substitutes a per-node-valid Core plan with one particle and proves rejection. |
| 5 | Error integrity | Optional-node admission treated every `charge` error as a budget fallback, which could hide a future serialization/accounting failure. | Low correctness | Fixed: only `BudgetExceeded` may drop optional decoration; every other error is returned unchanged. |

- Formal Codex Security diff scan
  `3556535a-4937-46f3-a080-71445770d8bd` completed against frozen snapshot
  `codex-security-snapshot/v1:sha256:6ddc7a9cbaf38b524215cf932c7356fd68a21792453eedd32bf709c308d64063`.
  All 30 workbench review items plus direct QML/test/documentation review were
  covered. It reported the two low-severity resource-amplification findings
  above; action shape and QML aggregate recounting were suppressed from the
  security report because no current dispatcher or lower-privilege plan writer
  exists, but the workflow still required both fixes. TAC advisory status was
  unknown because its connector was unavailable.
- Post-fix focused evidence: `cargo test -p omarchygs-game-cartridge
  --all-targets` passed 20 tests; `cargo test -p
  omarchygs-game-cartridge-renderer --all-targets` passed one unit and nine
  integration tests; focused Clippy passed with warnings denied; and
  `scripts/test-game-cartridge-renderer.sh` passed production package/preview
  preparation, all constrained QML profiles/states, input/accessibility smoke,
  and the malicious aggregate-plan rejection.
- Fresh CodeGraph inspection traced canonical archive verification into the
  immutable authenticated-file store, schema and action-contract validation,
  render lowering, incremental admission, unique asset publication, and the
  final private preview output. The only production compiler caller is the
  isolated preview CLI; no server, database, credential, provider-network, or
  confirmed action-dispatch authority entered the blast radius. CodeGraph's
  direct-test heuristic missed private helpers, so its hints were reconciled
  with the one-unit/nine-integration renderer corpus and the QML harness. The
  Phase 3.5 receipt matches pipeline
  `6494fb94-54e8-4fcf-b50d-9220c46f4564` and the post-fix gated state.

## Phase 4 — Validate

- `bin/gate.sh --diff` completed after the final implementation, inspection,
  architecture, security, and test edits and printed `GATE GREEN [diff]` with
  matching worktree receipt
  `9fc3d3ed26a092c8bbaebaad293f79aa5cb4a97f275fa389c5a62bbf4c260e2c`.
- The 15-gate run included rustfmt, workspace Clippy with warnings denied,
  production tests and rustdoc, Compose/shell/pipeline/secret/hook/whitespace
  checks, all 20 cartridge tests, one renderer unit plus nine integration
  tests, production package/preview and constrained QML renderer evidence, the
  seven-test isolated provider architecture proof, all 33 PostgreSQL
  integration tests, and the real migration → Rust API → visible QML smoke.

## Phase 5 — Complete

- EARS audit:
  - REQ-001 PASS — real signed fixtures compile all nine allowlisted node kinds;
    schema, binding, asset, action, capability, fallback, and substituted-plan
    negatives reject before a trusted component is instantiated.
  - REQ-002 PASS — the QML harness exercises deterministic keyboard, pointer,
    and accessibility activation plus scale, contrast, reduced-motion, and
    muted-audio preferences through the same platform-owned component paths.
  - REQ-003 PASS — exact Core/Rich node, grid, image, sprite, particle, audio,
    animation, plan-byte, frame, and RSS limits are enforced during lowering,
    rechecked at the QML handoff, and exercised by constrained stress fixtures.
  - REQ-004 PASS — all seven non-ready states render fixed origin/state chrome,
    zero cartridge nodes, no credential controls, and no confirmed command.
  - REQ-005 PASS — the production preview CLI verifies the package and prepares
    a private, read-only plan/assets directory while unusable provider,
    database, proxy, and credential environment values prove isolation.
  - REQ-006 PASS — `docs/architecture/game-cartridges.md` and OpenWiki publish
    the exact reference host, fixture loads, measured frame/RSS results,
    ratified budgets, fallback behavior, supported genres/effects, and honest
    advanced-2D/3D/general-engine limitations.
- OpenWiki update run `e6eacce5-2d0c-453f-817d-15a0b46a877d` completed after
  reconciling `game-cartridges.md`, `development-and-validation.md`,
  `quickstart.md`, and `product-boundaries.md`. The completion receipt records
  pipeline `6494fb94-54e8-4fcf-b50d-9220c46f4564` and state
  `53542c89a8fe641de0b1e00210d2a96d6dacd429839fd925e180288cbc52607d`.
- The AAR records the two fixed resource-amplification failures, four standing
  prevention rules, and the trusted renderer profile decision. Ticket 016 is
  closed and the active spec/notes pair is archived.
- After OpenWiki, AAR, ticket, roadmap, and archive reconciliation,
  `bin/gate.sh --diff` passed all 15 gates again: formatting, Clippy, unit and
  documentation tests, Compose/shell/pipeline/secret/hook/whitespace checks,
  both cartridge suites and the constrained QML benchmark, architecture proof,
  all 33 PostgreSQL integration tests, and the PostgreSQL → Rust API → visible
  QML smoke path.
- No commit, push, pull request, or other publication was performed.

## Defect and lesson ledger

| # | What happened | Root cause | Fix | Prevention |
|---|---|---|---|---|
| 1 | Initial QML load rejected `Accessible.value`. | The installed Qt Accessible attached type does not expose that property. | Folded the meter value into its accessible name and ran the real installed runtime. | Validate accessibility APIs against the installed Qt runtime, not memory alone. |
| 2 | Local plan XMLHttpRequest was disabled. | Qt disables local-file XHR unless the trusted process opts in. | The isolated preview harness sets `QML_XHR_ALLOW_FILE_READ=1`; the production surface still receives only an explicit private plan. | Exercise the exact launch environment for local developer tooling. |
| 3 | Loader-created nodes reported uninitialized required properties and transient bad asset URLs. | `Loader.sourceComponent` constructs the item before `onLoaded` can assign required values. | Nodes use inert safe defaults; the Loader assigns trusted root/preferences first and node data last. | Account for QML construction order at every untrusted-data boundary. |
| 4 | A smoke-only `index` reference was unavailable in the required-property delegate context. | Required delegate properties disable implicit role injection for undeclared roles. | Removed the unnecessary construction-time focus branch and exercise focus after instantiation. | Declare every delegate role used or avoid implicit context. |
| 5 | `VerifiedCartridge` authenticated structures remained externally mutable. | Ticket 015 exposed owned public fields before downstream trusted consumers existed. | Sealed fields to the crate and added read-only accessors used by the renderer. | Authenticated wrappers must be immutable outside the verifier boundary. |
| 6 | Signed repeated asset references amplified hashing and allocation before admission. | Authentication was performed in a fresh per-node map before the aggregate profile charge. | Cache the digest token by authenticated path and publish bytes only after successful admission. | Make expensive work proportional to unique authenticated inputs, not attacker-controlled references. |
| 7 | Plan bytes were enforced only after full construction. | Node-class counters and serialized-size accounting were separate, late controls. | Charge exact serialized-node bytes incrementally with checked arithmetic and keep the final exact check. | Enforce a resource promise while constructing the object, not after materializing it. |
| 8 | Action declarations could disagree with the platform emitter. | The contract linked nodes to action identifiers but not node-specific payload shapes. | Require exact Grid and Button payload fields during package verification. | Validate cross-layer command shape before any authorization/dispatch consumer exists. |
| 9 | QML trusted the compiler's aggregate counters. | The second parser mirrored types and per-node bounds but not aggregate profile semantics. | Recount the complete profile budget before any Loader instantiation. | Every meaningful security check at a serialized trust handoff needs a cheap independent verifier. |

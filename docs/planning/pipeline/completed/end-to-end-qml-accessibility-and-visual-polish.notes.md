---
title: End-to-end QML accessibility and visual polish — notes
pipeline_id: 3c97b5a7-d0e8-4d3e-a28a-e028358fe255
---

# End-to-end QML accessibility and visual polish — completion notes

Chronological evidence and decisions. If a test or command did not run, these
notes must not claim it passed.

## Phase 1 — Plan

- Recall: no active bulletin or pipeline blocked new work; `main` was clean and
  matched `origin/main` at the start of the slice. Ticket 025 was the next
  number, and end-to-end accessibility/visual polish was the next ordered
  unchecked private-alpha roadmap outcome.
- Recall: `scripts/check-pipeline-tools.sh` reported CodeGraph 1.5.0 and
  OpenWiki 0.3.3 ready with Codex-only provenance. `docker compose ps` showed
  the PostgreSQL service healthy. These are preflight facts, not test results.
- Recall: Tickets 022–024 already established the single bearer-owning API
  boundary, per-screen `focusInitial`, Enter/Escape behavior, high-contrast
  focus rings, plain-text untrusted presentation, and settled minimum-layout
  assertions. Ticket 024's prior delivery evidence reported 33 QML cases and a
  live two-account match, but no Ticket 025 validation has run yet.
- Recall: `PR-omarchy-gaming-system-compile-production-qml-root-after-control-edits-001`
  requires compiling the real `Main.qml` after shared-control changes;
  `PR-omarchy-gaming-system-own-headless-qt-test-environment-001` requires the
  harness to own its offscreen/software environment; and
  `PR-omarchy-gaming-system-assert-minimum-layout-after-settle-001` requires
  geometry checks only after Qt layout settles.
- Recall: the current shell routes ten states and the production QML tree still
  repeats raw colors, font sizes, headings, status text, cards, and section
  treatments across separately delivered screens. Existing accessibility
  properties concentrate on buttons and text inputs; the full screen/status
  semantics and cross-screen focus/layout matrix are not yet one contract.
- Decision: take one QML-only slice across the full current player flow; keep
  installer, operations, live-sync scheduling, and server behavior in later
  tickets.
- Decision: Phase 1 is settled by the ordered roadmap and the user's request to
  continue; proceed without a phase-by-phase approval pause.

## Phase 2 — Design

- Architecture and data flow:
  - `Main.qml` remains the only shell/router. It owns the window chrome and
    loads one of the ten existing trusted screen components from the
    `OnboardingController.state` allowlist. No screen or theme value may select
    a URL, import, component path, or executable cartridge content.
  - `OnboardingController`, `SocialController`, and `GameController` remain the
    only state/command producers. Their bearer, selected-persona, exact-schema,
    retry, and participant boundaries do not change. Validated controller text
    and state flow into shared platform-owned heading, banner, section, card,
    and control primitives, then into explicit `Text.PlainText` leaves.
  - A repository-owned `OgsTheme` object defines the semantic palette,
    typography, spacing, geometry, and tone mapping. Components instantiate
    that immutable QML type; screens consume semantic components rather than
    embedding color literals. The selected palette is deliberately dark/retro
    but its proposed normal-text ratios range from 6.28:1 to 18.19:1 on the
    darkest/raised surfaces, and its focus/control boundary ratios remain at
    least 4.04:1.
  - `OgsScreenHeader` owns one accessible heading plus stable status/error
    banners. `OgsStatusBanner` prefixes state with visible text such as
    `STATUS`, `WORKING`, `WARNING`, or `ERROR`, supplies an accessibility role
    and name, and never relies on its accent stripe alone. `OgsSectionLabel`
    and `OgsCard` provide consistent landmarks and bounded surfaces.
  - Existing shared buttons and fields retain native Qt control semantics,
    strong tab focus, and Enter behavior while adopting the centralized focus,
    disabled, hover, pressed, selected, label, and description contract.
    Screens retain explicit `focusInitial`/Escape functions. Mode changes and
    asynchronous completions restore focus only when no usable descendant
    still owns it, avoiding both dead ends and unsolicited focus theft.
  - The deterministic QML fixture remains the runtime evidence boundary. A new
    production-root accessibility matrix walks every routed state, checks
    headings/banners/accessibility properties, forward/reverse focus, Escape
    authority preservation, and settled geometry at 640×420 and 920×600. A
    source-policy check prevents raw palette literals from drifting back into
    the shell/screens/shared game surface. The existing hostile and live flows
    remain the semantic regression proof.
- CodeGraph evidence:
  - `mcp__codegraph__codegraph_explore` ran after Phase 1 against the current
    pipeline and requested the shell, controllers, shared controls, focus
    paths, and fixture entrypoints. The index returned the fixture HTTP
    producer and its game/social request surface, but confirmed its parser does
    not model the QML graph. Direct inspection therefore covered `Main.qml`,
    all ten screens, four existing shared controls, Signal Siege presentation,
    controller state transitions, all 33 existing fixture cases, and the
    offscreen runner. CodeGraph remains the worktree-bound design receipt; it
    does not replace the direct unsupported-file review.
- Database and migration consequences: none. No migration, SQL, Rust type, API
  route, payload, authorization, or server lifecycle changes are permitted by
  this manifest.
- API compatibility: unchanged. Screens continue to consume only the existing
  controller properties and functions. Object names used by the fixture corpus
  remain stable unless a test is updated in the same change.
- Exact file manifest:

| File | Purpose |
|---|---|
| `client/qml/components/OgsTheme.qml` (new) | Central semantic colors, contrast-bearing tones, fonts, spacing, radii, control sizes, and state helpers. |
| `client/qml/components/OgsStatusBanner.qml` (new) | Non-color state prefix, accessible status/error role/name, bounded wrapping, and semantic tone. |
| `client/qml/components/OgsScreenHeader.qml` (new) | Consistent accessible screen heading plus status/error banner stack. |
| `client/qml/components/OgsSectionLabel.qml` (new) | Reusable accessible section heading/label. |
| `client/qml/components/OgsCard.qml` (new) | Shared bounded surface and border treatment for profile, message, game, challenge, and social rows. |
| `client/qml/components/OgsButton.qml`, `OgsTextField.qml`, `OgsTextArea.qml`, `SocialRow.qml` | Adopt theme, selected/disabled/focus semantics, descriptions, and consistent minimum geometry without changing input bounds or emitted actions. |
| `client/qml/Main.qml` | Adopt semantic shell chrome, expose current screen/navigation status without color-only meaning, and preserve allowlisted loader/focus transfer. |
| `client/qml/screens/ConnectionScreen.qml`, `AccessScreen.qml`, `MfaScreen.qml`, `PersonaScreen.qml` | Adopt shared headings/status/sections/cards, preserve secret clearing and access/MFA/persona authority, and close focus/layout gaps. |
| `client/qml/screens/HomeScreen.qml`, `SocialScreen.qml`, `InboxScreen.qml` | Adopt shared landmarks/cards/state feedback and prove responsive keyboard navigation for dynamic social and message collections. |
| `client/qml/screens/GamesScreen.qml`, `ChallengesScreen.qml`, `GameplayScreen.qml` | Adopt shared landmarks/cards/state feedback while retaining exact compiled/provider presentation and mutation controls. |
| `client/qml/game/SignalSiegeSurface.qml` | Use platform semantic theme/sections/cards while preserving only the fixed derived view and strike/guard/charge actions. |
| `client/qml/tests/fixture/tst_accessibility.qml` (new) | Full-shell routed-screen, theme contrast, accessibility, focus, Escape, and settled geometry matrix. |
| `client/qml/tests/fixture/tst_onboarding.qml`, `tst_social.qml`, `tst_games.qml` | Retain semantic flow coverage and add shared-state/focus/layout assertions where the new primitives make them observable. |
| `scripts/check-qml-style.py` (new) | Deterministically reject raw six-digit palette literals and non-plain dynamic text in the in-scope production QML surface. |
| `scripts/test-qml-onboarding.sh` | Run the style policy before the existing offscreen Qt Quick Test corpus. |
| `docs/product-charter.md`, `docs/architecture/system-overview.md`, `docs/planning/ROADMAP.md` | Reconcile the finished accessibility contract and private-alpha status during Phase 5. |
| Active ticket/spec/notes and `AAR-025`; generated OpenWiki/claims during Phase 5 | Durable pipeline evidence and knowledge capture. |

- Regression plan:

| Requirement | Evidence |
|---|---|
| REQ-001 | `tst_accessibility.qml` routes all ten states and requires unique heading/status/navigation semantics and non-color status prefixes. |
| REQ-002 | Forward/reverse key traversal, visible focus geometry, selected-mode state, and post-response focus restoration at both sizes. |
| REQ-003 | Escape matrix asserts the documented parent/mode and verifies bearer, selected persona, and selected game/social authority remain or clear only where already intended. |
| REQ-004 | Settled bounding-box and scroll-surface checks for every routed screen plus populated persona/social/message/game/challenge/gameplay representatives at 640×420 and 920×600. |
| REQ-005 | Shared-component contract tests for enabled/disabled/focused/pressed/selected/status tones plus the raw-style source policy. |
| REQ-006 | Deterministic sRGB relative-luminance calculations over every declared text, focus, and control-boundary foreground/background pair. |
| REQ-007 | Existing hostile social/game envelopes and explicit `Text.PlainText` source policy; direct provenance inspection of Signal Siege versus cartridge surface. |
| REQ-008 | Existing offline/timeout/malformed/oversized/empty/invalid-session/revision/retry fixtures augmented with banner text, role/name, and disabled-action assertions. |
| REQ-009 | New production-root keyboard tour across onboarding, social/inbox, games/challenges, and gameplay; existing live migrated scenarios remain in the canonical gate. |

- Risk and rollback analysis:
  - Security/privacy: accessibility names must describe password/factor fields
    without echoing values; state banners may show only controller-mapped safe
    messages. Theme/status components accept no markup, URLs, QML, action IDs,
    or arbitrary imports. Cartridge/provider data stays downstream of exact
    validation and cannot alter shell chrome or theme.
  - Accessibility: changing item nesting can silently alter tab order, active
    focus, ListView delegate visibility, and ScrollView geometry. The matrix
    tests both directions after Qt settles and uses the real production root.
  - Layout: centralized minimum heights can increase content height. Every
    screen must either fit or expose keyboard-reachable scrolling; raising the
    application minimum is forbidden.
  - Concurrency/reconnect: no new request, timer, polling, WebSocket, or shared
    mutable controller state is introduced. Loading/retry banners reflect
    existing state only; they cannot dispatch a duplicate action.
  - Performance: the primitives add a small bounded number of rectangles/text
    nodes per screen and no animation. Dynamic collections remain under their
    existing server/client bounds; the challenge history Repeater is not made
    automatic or heavier in this slice.
  - Compatibility: raw object names and controller calls remain stable. The
    focus ring may consume interior border pixels, so minimum control geometry
    is defined centrally and verified at both sizes.
  - Rollback: no durable data changes exist. Reverting the QML/components/tests
    and source-policy script restores the prior presentation without migration
    or server coordination.
- Alternatives rejected:
  - Golden screenshot diffs are too renderer/font/platform-sensitive to be the
    delivery oracle; deterministic semantic, contrast, focus, and geometry
    evidence is stronger. Screenshots may aid human review but are not the gate.
  - Raising the minimum window would hide layout defects and break the existing
    Omarchy contract.
  - A data-driven screen generator would obscure explicit trusted QML and
    create unnecessary dispatch complexity. Shared primitives are sufficient.
  - Publisher-supplied themes or raw QML would cross the cartridge execution
    boundary and are prohibited.
  - OS screen-reader certification and decorative motion are separate target-
    matrix decisions, not claims this private-alpha slice can honestly prove.

## Phase 3 — Implement

- Built:
  - Added immutable `OgsTheme` tokens for normal and high-contrast palettes,
    typography, spacing, borders, focus geometry, minimum controls, and state
    tone/prefix mapping. Added shared screen-header, status-banner,
    section-label, and card primitives; updated the existing button, text-field,
    text-area, and social-row controls to consume them.
  - Migrated `Main.qml` and all ten routed screens to one plain-text heading,
    status/error, navigation, semantic state, and high-visibility focus
    contract without changing controller functions, request routing, or
    authority. The shell status rail now reports `SETUP`, `PLAYER READY`, or
    `ERROR` in text as well as color.
  - Preserved every existing `focusInitial`/Escape contract and added explicit
    restoration for access-mode changes, delayed persona-list delegate
    materialization, and completed authoritative gameplay loads. Focus-only
    callbacks call no action or network function.
  - Migrated Signal Siege, `TrustedCartridgeSurface`, `CartridgePreview`, and
    every visual trusted cartridge node to the same semantic theme while
    retaining high-contrast preferences, scaling, exact plan validation,
    fixed component selection, and action dispatch.
  - Added `scripts/check-qml-style.py` and made it the first QML fixture step.
    The check covers 33 visual production QML files, permits raw palette values
    only in `OgsTheme`, rejects Rich/Auto/Styled/Markdown text modes, and uses a
    string/comment-aware balanced-block scan to require explicit
    `Text.PlainText` on every `Text` object.
  - Added `tst_accessibility.qml`: three production-root cases cover contrast,
    every routed screen, accessible headings/state/navigation, non-color state
    prefixes, enabled initial focus, forward/reverse traversal, Escape
    authority preservation, compact/default widths, and heading/status
    non-overlap. The full QML corpus increased from 33 to 38 passing cases.
- Deviations:
  - Inspection expanded the exact manifest from the compiled Signal Siege
    surface to the pre-existing trusted signed-cartridge preview/surface/node
    library. Leaving those platform-owned controls on a second raw palette
    would have made the centralized-theme claim false and left shared button,
    meter, status, placeholder, and render-state surfaces outside enforcement.
    This is presentation-only and changes no cartridge/provider semantics.
  - Existing onboarding/social/game tests did not need modification; their 33
    semantic cases remained green and the new matrix owns the added contract.
  - Product/architecture/roadmap reconciliation remains Phase 5 work as
    designed; no generated OpenWiki page was hand-edited during implementation.

## Phase 3.5 — Inspect ledger

| # | Lens | Finding | Severity | Disposition |
|---|---|---|---|---|
| 1 | Accessibility/correctness | `GameplayScreen.focusInitial()` ran before an authoritative session produced a visible enabled presenter action, leaving the loader as the active item. | medium | Resolved: `onBusyChanged` schedules focus restoration after the validated presentation settles; the production-root matrix proves Strike focus and round-trip traversal. |
| 2 | Accessibility/correctness | Persona selection delegates materialize after the loader's initial focus callback, so a populated list could retain focus on the loader instead of its first persona. | medium | Resolved: the list owns index zero and schedules a bounded post-materialization focus handoff; the compact public-flow matrix proves delegate focus and traversal. |
| 3 | Reuse/visual consistency | The first implementation centralized shell/screens/Signal Siege but left the trusted signed-cartridge surface and node library on a duplicate raw palette. | medium | Resolved: migrated the preview, surface, and every visual trusted node; source policy now covers all 33 visual production QML files. |
| 4 | Security/provenance | Rejecting explicit rich-text tokens did not by itself prevent a future `Text` object from retaining Qt's implicit `AutoText` default. | medium | Resolved: the policy parses balanced `Text` blocks and requires explicit `Text.PlainText`; direct inspection confirmed controller-derived and cartridge-derived leaves remain bounded plain text. |
| 5 | Correctness/layout | Shared 44-pixel controls and stacked state banners could have introduced compact-width clipping or heading/status overlap across separately built screens. | medium | Resolved: every routed screen proves horizontal bounds at 640×420, representative existing tests cover settled dynamic layouts, and the new matrix asserts heading/status non-overlap plus default 920×600 behavior. |
| 6 | Security/privacy | New accessibility names and state surfaces could have exposed password, MFA, bearer, recovery, private-message, or provider-controlled executable content. | high hypothesis | Closed with no finding: secret fields retain fixed descriptive labels, controller errors are code-mapped, fixed component/action switches are unchanged, all dynamic leaves are explicit plain text, and unsupported provider presentation stays inert. |
| 7 | CodeGraph/blast radius | Both post-implementation CodeGraph runs indexed the Python source-policy helper but do not parse QML; unrelated Rust symbols matched ambiguous names. | info | Accepted limitation: direct file-by-file inspection covered every changed QML file and existing/new tests; the fresh CodeGraph query remains the worktree receipt and reported no indexed Rust/API blast radius from this QML-only slice. |
| 8 | Security scan | Final Codex Security diff scan `3253bf0a-d607-48cf-bd0c-f0e9f2fe88ba` found zero reportable findings across the final snapshot. Native inventory recognized two script files and not QML. | info | PASS with explicit coverage note: all changed QML was directly inspected; canonical coverage is complete, TAC advisory was unavailable, and the independent architecture worker did not return so the parent performed the sequential source-backed model. Report: `/tmp/codex-security-scans-gVuCBs/omarchy_bbs/afff58ed05245e40b2980225ef4324ba95326549_20260826T033553Z_js2pr_3c/report.md`. |

- Fresh post-implementation CodeGraph receipt: the final explore call returned
  current `scripts/check-qml-style.py` source and its one entrypoint/caller,
  while confirming QML remains unsupported. Direct inspection then covered the
  final shell, screens, shared components, trusted cartridge surface/nodes,
  Signal Siege surface, and QML fixture corpus.
- Focused evidence before Phase 4:
  - `./scripts/test-qml-onboarding.sh`: PASS, 38 passed, 0 failed; visual policy
    passed across 33 production QML files.
  - `./scripts/test-game-cartridge-renderer.sh`: PASS, 2 unit + 9 rendering
    cases plus ready/non-ready Core/Rich-2D QML preview and frame/RSS evidence.
  - `bin/gate.sh --fast`: PASS before the final trusted-node theme expansion;
    the final canonical diff gate remains Phase 4 evidence.

## Phase 4 — Validate

- Tests run:
  - `./scripts/test-qml-onboarding.sh` — PASS: visual policy passed for 33
    production QML files; Qt Quick Test passed 38, failed 0, skipped 0.
  - `./scripts/test-game-cartridge-renderer.sh` — PASS: 2 renderer unit and 9
    rendering integration cases plus ready/non-ready Core/Rich-2D QML preview,
    accessibility, action, frame, RSS, and raster evidence.
  - `bin/gate.sh --diff` — PASS: all 18 stages, including 45 migrated
    PostgreSQL cases, live API/QML registration/social/MFA/two-authority game
    scenarios, provider conformance, and the Door Legends authority pilot.
- Gate run: `GATE GREEN [diff]`; the Phase 4 worktree receipt at
  `.git/omarchy-gaming-system-gate-receipt` contained
  `f2b7dd86e276f36fd802343f6e9ab64174f9395ed06c9bf20c91925ace648c19`.
  Phase 5 documentation and archival intentionally make it stale, so the
  completed worktree requires one final diff gate.
- Skips or pre-existing failures: none. Qt emitted non-fatal host EGL `dri2`
  diagnostics only after successful offscreen scenarios. The TAC connector was
  unavailable, so its advisory status could not be verified; required local
  security review and canonical validation completed independently.

## Phase 5 — Complete

- Acceptance-criteria audit:

| Requirement | Evidence | Result |
|---|---|---|
| REQ-001 | `tst_accessibility.qml` routes all ten production screens and requires a visible plain-text heading, accessible status, non-color prefix, and navigation hint; `Main.qml` adds the semantic shell rail/legend. | PASS |
| REQ-002 | Production-root cases prove enabled deterministic initial focus and Tab/Shift-Tab round trips; access mode, delayed persona delegates, and completed gameplay loads restore focus after their targets exist. | PASS |
| REQ-003 | MFA Escape clears its challenge; authenticated Social, Inbox, Challenges, and Gameplay Escape paths return to their documented parent while preserving the bearer and exact selected persona. | PASS |
| REQ-004 | The fixture asserts heading/status/control horizontal bounds and heading/status non-overlap at 640×420 across every route, then repeats representative state at 920×600; existing collection fixtures retain settled dynamic-layout coverage. | PASS |
| REQ-005 | Shared button/field/area/row/header/banner/card/section components own semantic enabled, disabled, focus, state, role, name, and non-color cues; the style policy rejects palette drift across all 33 visual files. | PASS |
| REQ-006 | The deterministic theme test calculates sRGB contrast and requires 4.5:1 normal text plus 3:1 focus/control/status indicators over declared surfaces. | PASS |
| REQ-007 | The style checker requires explicit `Text.PlainText` on every visual `Text` object and forbids rich/automatic modes; hostile social/game fixtures and trusted cartridge renderer tests preserve schema/provenance bounds. | PASS |
| REQ-008 | Shared banners expose stable accessible loading/error/status text while existing malformed, oversized, timeout, invalid-session, conflict, retry, empty, completed, and provider-inert fixtures prove safe action state. | PASS |
| REQ-009 | The production-root keyboard tour traverses access, MFA, persona, home, social, inbox, games, challenges, and gameplay with no mouse; the canonical live smoke retains the real migrated two-account match path. | PASS |

- Docs: updated the product charter, system overview, Game Cartridge host-theme
  boundary, and roadmap. OpenWiki run
  `b0d45c4e-8d8f-4ed4-881f-58b7bf709541` reconciled quickstart, runtime,
  cartridge, and validation pages plus grounded claims and returned
  `status: complete`.
- AAR: submitted `AAR-025` with three failures, three prevention rules, and one
  architecture decision; every new ID is present in the knowledge register.
- Archive: Ticket 025 is closed and its only active spec/notes pair is moved to
  `docs/planning/pipeline/completed/`. No active pipeline remains. Delivery is
  intentionally uncommitted and unpushed pending explicit user authorization.
- Final receipt: the post-OpenWiki, post-archive `bin/gate.sh --diff` run passed
  all 18 stages with `GATE GREEN [diff]`. Both the delivery receipt and current
  gated state equal
  `404e289bebfa17ac7e11694e1e83e5aa3fd7e4a48e580e038d18189503697b7f`;
  the Ticket 025 completion receipt matches the same gated worktree.

## Defect and lesson ledger

| # | What happened | Root cause | Fix | Prevention |
|---|---|---|---|---|
| 1 | Gameplay focus stayed on the Loader until an action existed. | Initial component load completed before authoritative session presentation. | Restore focus on busy completion after the presenter settles. | `PR-omarchy-gaming-system-restore-focus-after-qml-materialization-001` |
| 2 | A populated Persona screen could lack delegate focus. | The ListView delegate materialized after Loader focus handoff. | Own index zero and schedule a bounded delegate focus handoff. | `PR-omarchy-gaming-system-restore-focus-after-qml-materialization-001` |
| 3 | Trusted cartridge visuals retained a second raw palette. | Initial style inventory stopped at application routes and Signal Siege. | Migrate preview/surface/nodes and expand the enforced inventory. | `PR-omarchy-gaming-system-scope-style-policy-to-the-trusted-visual-boundary-001` |
| 4 | Token rejection still permitted implicit `AutoText`. | The first policy searched only explicitly named unsafe formats. | Parse balanced QML `Text` blocks and require `Text.PlainText`. | `PR-omarchy-gaming-system-require-plain-text-on-every-qml-text-object-001` |
| 5 | Shared control height risked compact-screen overlap. | Individually valid components had not been proven as a settled routed composition. | Assert all route landmarks and controls through `Main.qml` at both supported sizes. | `PR-omarchy-gaming-system-assert-minimum-layout-after-settle-001` |

---
aar: AAR-024-signal-siege-versus-and-keyboard-first-game-flow
ticket: TICKET-024
pipeline: signal-siege-versus-and-keyboard-first-game-flow
status: submitted
opened: 2026-08-25
submitted: 2026-08-25
effectiveness: 5
---

# AAR-024-signal-siege-versus-and-keyboard-first-game-flow

## Recalled at plan

| ID or source | How surfaced | Useful? |
|---|---|---|
| `AD-omarchy-gaming-system-durable-game-challenge-orchestration-001` | Ticket 020 AAR, API contract, and runtime architecture | Yes — QML challenge actions must preserve participant direction, exact versions, durable history, and REST recovery. |
| `AD-omarchy-gaming-system-signal-siege-solo-game-lifecycle-001` | Ticket 021 AAR and current rules/runtime code | Yes — v1 is immutable; a two-human definition must be a new exact version and reuse the same deterministic session transaction. |
| `PR-omarchy-gaming-system-resolve-durable-replay-before-current-admission-001` | Knowledge register and challenge/solo transactions | Yes — committed create/start retries must survive later connection, registry, cap, or lifecycle drift. |
| `PR-omarchy-gaming-system-check-replay-before-current-revision-001` | Knowledge register and command transaction | Yes — client retry and terminal command replay must remain stable after revisions advance. |
| `PR-omarchy-gaming-system-validate-game-state-cross-field-invariants-001` | Ticket 021 inspection | Yes — Signal Siege v2 must validate turn, active-seat, combatant, last-action, and outcome relationships, not only JSON shape. |
| `AD-omarchy-gaming-system-qml-onboarding-authority-boundary-001` and `AD-omarchy-gaming-system-qml-social-inbox-authority-boundary-001` | Tickets 022–023 AARs and QML controllers | Yes — game surfaces receive neither bearer nor arbitrary actor authority and continue explicit REST recovery. |
| `PR-omarchy-gaming-system-mirror-authoritative-client-response-bounds-001` | Knowledge register and current strict QML validators | Yes — catalog, challenge, session, state, participant, result, and command schemas must fail closed against the server contract. |
| Product charter, roadmap, and Ticket 023 gap | Product preflight | Yes — a real two-human definition plus challenge/game QML is the next complete player outcome; fixture-only challenge UI is insufficient. |

## What happened

The pipeline closed the private-alpha first-playable gap as one vertical slice.
Signal Siege v1 stayed immutable and production gained exact two-human v2 with
bounded alternating turns, cross-field state validation, and explicit terminal
outcomes. Existing challenge acceptance and revision/idempotency session
transactions carried that definition without a migration or new route.

The QML shell gained Games, Challenges, and Gameplay paths plus one bearer-free
game controller behind the selected-persona request gateway. It validates exact
catalog, relationship, challenge, participant, session, provider, command, and
v1/v2 state envelopes; retains one exact uncertain mutation for explicit retry;
refetches after revision conflict; and clears complete authority on a valid
invalid-session response. Compiled Signal Siege renders through a platform-
owned presenter assembled from inert repository components without minting a
signed cartridge origin or render-plan document.

Inspection found and fixed participant uniqueness/cardinality gaps before
indexed presentation and a six-button home layout that overflowed the 640×420
contract. The final fixture corpus reached 33 cases. Ten rules tests, 45
PostgreSQL cases, and the live two-controller scenario proved a real challenge,
acceptance, alternating match, terminal outcome, and fresh-controller recovery.
The canonical diff gate passed every platform, cartridge, provider, and remote-
authority proof. The initial security scan found no reportable issue but could
not inventory new untracked files, so delivery requires the final scan against
the staged snapshot.

## Failures captured

| ID | Failure | Where surfaced |
|---|---|---|
| `BF-omarchy-gaming-system-qml-game-session-cardinality-gap-001` | The initial QML session validator bounded participant count but did not reject duplicate personas or bind exact Signal Siege v1/v2 cardinality before seat-indexed presentation. | Direct Phase 3.5 response/provenance inspection and hostile envelope construction. |
| `BF-omarchy-gaming-system-qml-home-action-overflow-001` | Six fixed home actions in one row could extend beyond the supported 640-pixel window. | Minimum-size production-root QML inspection; the first geometry assertion also demonstrated that layout evidence must wait for Qt to settle. |
| `BF-omarchy-gaming-system-security-scan-untracked-inventory-gap-001` | The first immutable security workbench inventory omitted newly created untracked QML paths, leaving formally partial coverage despite direct manual review. | Codex Security scan `d99f4742-297d-46ae-9eb6-0cc39f63b76f`; final staged scan was made a delivery requirement. |

## Prevention rules captured

| ID | Rule | Why |
|---|---|---|
| `PR-omarchy-gaming-system-bind-presentation-cardinality-before-indexing-001` | Before trusted client presentation indexes participants or state arrays, bind uniqueness, actor membership, exact game version, and exact version-specific cardinality. | Generic transport bounds do not prove the cross-field relationships assumed by a game-specific presenter. |
| `PR-omarchy-gaming-system-assert-minimum-layout-after-settle-001` | Exercise the production root at every supported minimum size and assert actual child geometry only after the asynchronous layout has settled. | Declared minimum dimensions and component implicit widths do not prove visible containment; immediate coordinate assertions can test an intermediate layout. |
| `PR-omarchy-gaming-system-stage-new-paths-before-final-security-scan-001` | Run the delivery security scan against a staged snapshot so newly created paths are included in the immutable inventory, then make no repository-content changes before delivery. | A working-tree scan can be honest but incomplete when its inventory excludes untracked files. |

## Architecture decisions

| ID | Decision | ADR |
|---|---|---|
| `AD-omarchy-gaming-system-signal-siege-versus-version-boundary-001` | Preserve one-human Signal Siege v1 byte-for-behavior and add exact two-human alternating play as immutable v2 under the same canonical game key. | `docs/architecture/system-overview.md` |
| `AD-omarchy-gaming-system-platform-compiled-presenter-provenance-001` | A platform-compiled game may reuse inert repository UI components through a platform-derived view model, but only a verified installed cartridge may claim signed origin, digest, or `omarchygs.render-plan/v1` provenance. | `docs/architecture/game-cartridges.md` |

Every new ID above must also be appended to
`docs/planning/knowledge/INDEX.md`.

## Effectiveness

Effective. All nine EARS requirements have direct rule, database, QML fixture,
and/or live vertical-slice evidence. Inspection changed the implementation in
three material ways before delivery: it prevented hostile participant indexing,
restored minimum-width containment, and required staged security coverage for
new files. OpenWiki completed, the knowledge register contains every new ID,
and the closure leaves no active spec/notes pair. The final delivery gate and
staged security scan remain receipts for publishing the already accepted slice,
not deferred acceptance work.

---
aar: AAR-016-trusted-cartridge-renderer-and-previewer
ticket: TICKET-016
pipeline: trusted-cartridge-renderer-and-previewer
status: submitted
opened: 2026-08-25
submitted: 2026-08-25
effectiveness: 5
---

# AAR-016-trusted-cartridge-renderer-and-previewer

## Recalled at plan

| ID or source | How surfaced | Useful? |
|---|---|---|
| `AD-omarchy-gaming-system-game-cartridge-provider-boundary-001` | Knowledge register, ADR-0002, and OpenWiki | Yes — the renderer is trusted platform code over data-only cartridges; remote authority stays disabled. |
| `AD-omarchy-gaming-system-canonical-game-cartridge-v1-001` | Ticket 015 AAR and production crate | Yes — renderer input starts only after exact archive/signature/content verification. |
| `PR-omarchy-gaming-system-make-untrusted-text-format-explicit-001` | Ticket 014 knowledge | Yes — every cartridge or provider-derived string must use plain-text rendering. |
| `PR-omarchy-gaming-system-validate-decoder-profile-not-headers-001` | Ticket 015 knowledge | Yes — production QML decoders and the accepted strict PNG/WAV profile require end-to-end stress evidence. |
| `PR-omarchy-gaming-system-bind-presentation-nodes-to-capabilities-001` | Ticket 015 knowledge | Yes — every new Rich-2D node must enter through an exact capability and deterministic fallback. |
| `BUL-001-initial-push-pending` | Bulletin preflight | Yes — delivery remains local and unauthorized. |

## What happened

Ticket 016 delivered the first production-owned renderer for a verified Game
Cartridge. The cartridge contract now pins a restricted view schema per screen
and supports nine typed nodes across Core and Rich-2D profiles. A new Rust
compiler validates the view, bindings, action shapes, assets, fallbacks, and
profile budgets, then lowers them into an inert render plan. The isolated
preview CLI publishes only digest-named authenticated assets and a bounded plan
inside a caller-provided private directory.

The QML side is entirely platform-owned. Fixed components render plain-text
data, preserve trusted origin and failure-state chrome, expose shared keyboard,
pointer, and accessibility actions, and independently recount aggregate
profile limits before instantiation. No publisher QML, JavaScript, URL,
filesystem path, provider network, platform credential, database, or confirmed
game-command authority entered the runtime.

The constrained Qt 6.11 software-rendering benchmark sustained the full Core
fixture at 16.001 ms average / 17.799 ms maximum and 121,152 KiB peak RSS. The
Rich-2D fixture sustained 16.004 / 16.813 ms and 232,328 KiB. This establishes
the current product envelope as authentic BBS through polished animated 2D:
cards and boards, roguelikes, tile worlds, asynchronous RPG/strategy/management
games, tactical maps, puzzles, visual novels, sprites, particles, and bounded
audio. Advanced 2D/2.5D and constrained 3D remain possible future reviewed
profiles; Halo-class FPS gameplay, high-frequency physics/twitch networking,
and a general Unity/Unreal-style engine are outside the cartridge runtime.

Formal Codex Security inspection found two low-severity resource-amplification
issues. Both were fixed by making asset authentication proportional to unique
authenticated inputs and enforcing exact plan bytes during construction.
Cross-layer review also bound each node to its exact emitted action payload and
added the independent QML aggregate recount. The final focused corpus contains
20 cartridge tests, one renderer unit test, and nine renderer integration tests.
The canonical 15-gate validation passed before the OpenWiki lifecycle completed
as run `e6eacce5-2d0c-453f-817d-15a0b46a877d`, then passed again after the AAR,
ticket, roadmap, wiki, and pipeline archive were reconciled.

## Failures captured

| ID | Failure | Where surfaced |
|---|---|---|
| `BF-omarchy-gaming-system-repeated-asset-authentication-amplification-001` | Renderer asset authentication repeated full-buffer hashing and cloning for every optional reference before aggregate admission. | Phase 3.5 Codex Security diff scan and direct compiler review. |
| `BF-omarchy-gaming-system-late-render-plan-byte-budget-001` | The declared render-plan byte ceiling was enforced only after all repeated binding clones and a full serialization existed. | Phase 3.5 Codex Security diff scan and resource-bound analysis. |

## Prevention rules captured

| ID | Rule | Why |
|---|---|---|
| `PR-omarchy-gaming-system-make-expensive-authentication-unique-001` | Cache expensive authentication work by immutable authenticated identity and publish retained bytes only after the referencing object passes admission. | An attacker controls reference cardinality even when each referenced byte buffer is signed and bounded. |
| `PR-omarchy-gaming-system-enforce-render-budgets-during-construction-001` | Charge retained render-plan bytes with checked arithmetic before keeping each node, then preserve a final exact envelope check. | A late size check does not enforce transient memory or construction-work promises. |
| `PR-omarchy-gaming-system-bind-node-actions-to-exact-payloads-001` | Bind each declarative interactive node to one exact platform-emitted payload shape before a dispatcher exists. | Identifier-only checks permit contract confusion at the future authorization boundary. |
| `PR-omarchy-gaming-system-recount-budgets-at-render-handoff-001` | Independently recount cheap aggregate profile budgets when a serialized plan crosses into the trusted UI runtime. | Per-node checks alone do not preserve a producer's aggregate resource guarantee after handoff or substitution. |

## Architecture decisions

| ID | Decision | ADR |
|---|---|---|
| `AD-omarchy-gaming-system-trusted-cartridge-renderer-v1-001` | Adopt a Rust-validated inert render plan and fixed platform-owned QML vocabulary with measured Core/Rich-2D budgets; keep publisher code, direct networking, 3D, and general-engine capabilities outside v1. | `../../../architecture/game-cartridges.md`; `../../../../openwiki/game-cartridges.md` |

Every new ID above must also be appended to
`docs/planning/knowledge/INDEX.md`.

## Effectiveness

5/5. All six EARS requirements passed with production Rust and QML paths,
hostile package/view/action/resource regressions, a constrained frame/RSS
benchmark, formal security and CodeGraph inspection, a green 15-gate vertical
slice, and completed OpenWiki reconciliation. The result makes the cartridge
metaphor executable while preserving the platform/provider authority boundary
and an honest graphics ceiling.

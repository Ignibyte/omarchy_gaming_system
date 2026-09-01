---
aar: AAR-049-usurper-solo-equipment-economy
ticket: TICKET-049
pipeline: usurper-solo-equipment-economy
status: submitted
opened: 2026-08-31
submitted: 2026-08-31
effectiveness: effective
---

# AAR-049-usurper-solo-equipment-economy

## Recalled at plan

| ID or source | How surfaced | Useful? |
|---|---|---|
| `AD-omarchy-gaming-system-usurper-v020e-deterministic-provider-port-001` | Ticket 047/048 knowledge and port map | Yes — keeps rules/provider separate and shared realm gated. |
| `PR-omarchy-gaming-system-prove-provider-state-topology-fit-001` | Knowledge register | Yes — bounds this slice to player-private session state. |
| `PR-omarchy-gaming-system-prove-legacy-branch-scope-before-porting-behavior-001` | Ticket 048 AAR | Yes — requires canonical source scope for each translated economy rule. |
| Ticket 048 completed evidence | nearest completed notes and OpenWiki | Yes — provides the tested reducer/provider/cartridge baseline. |

## What happened

The adjacent Usurper workspace advanced from one deterministic BBS day to a
rules-v2 player-private equipment economy. The model, source-linked data, and
pure reducer now cover a fifteen-slot pack and chest, weapon/body-armor
equipment, buy/sell/haggle flows, solo banking, daily shop counters, and
equipment-aware combat. The signed inert cartridge grew from eleven to sixteen
screens, while the platform-owned trusted QML renderer remained the only
executable presentation surface.

The slice stayed inside the existing public Provider seam and independent
provider PostgreSQL database. Its fixed gameplay profile passed the unchanged
fifteen-case TLS, authentication, replay, fault, callback, reconciliation, and
receipt corpus twice across process restart. Direct dependency, content,
privacy, CodeGraph, and migration/route reviews found no new platform gameplay
authority or shared-realm claim.

One live conformance attempt failed before network effects because two nested
command objects were valid JSON but not canonical JSON. Reordering their fields
and rerunning the whole corpus fixed the defect. The source review also exposed
mutually exclusive classic and non-classic equipment implementations; declaring
the non-classic mode before selecting records prevented a plausible but
historically nonexistent hybrid port.

## Failures captured

| ID | Failure | Where surfaced |
|---|---|---|
| `BF-omarchy-gaming-system-provider-profile-noncanonical-nested-field-order-001` | A generated rules-v2 provider profile emitted two nested `buy_item` objects in non-canonical field order, so the public conformance CLI rejected the configuration before the live run. | First Ticket 049 provider conformance invocation. |

## Prevention rules captured

| ID | Rule | Why |
|---|---|---|
| `PR-omarchy-gaming-system-verify-generated-provider-profiles-before-live-use-001` | Round-trip every generated provider gameplay profile through canonical JSON serialization and require byte equality before invoking live conformance. | Strict canonical transport fixtures must fail locally and precisely rather than spending a provider startup to discover producer ordering drift. |
| `PR-omarchy-gaming-system-declare-legacy-mode-before-composing-subsystems-001` | Declare the exact legacy build/runtime mode first, then admit subsystem producers and consumers only from that compatible branch unless a documented cross-mode mapping exists. | Adjacent files and overlapping names do not prove that mutually exclusive classic and non-classic implementations ever formed one executable game. |

## Architecture decisions

| ID | Decision | ADR |
|---|---|---|
| `AD-omarchy-gaming-system-usurper-v020e-deterministic-provider-port-001` | Keep this player-private equipment economy in the separate deterministic provider/cartridge and continue deferring shared-realm storage until its topology seam is reviewed. | Existing Ticket 047 decision; consistent with ADR-0002 and ADR-0003. |

Every new ID above must also be appended to
`docs/planning/knowledge/INDEX.md`.

## Effectiveness

Effective. Recalled topology and branch-scope rules kept the slice on the
session-bounded provider path and the coherent non-classic source branch. The
new profile-canonicalization rule comes directly from a fail-closed live check,
and the corrected profile passed the complete corpus twice across restart. The
external workspace checks, visible signed preview, fresh CodeGraph inspection,
OpenWiki lifecycle, and full platform gate jointly cover the intended economy
without widening platform or shared-realm authority.

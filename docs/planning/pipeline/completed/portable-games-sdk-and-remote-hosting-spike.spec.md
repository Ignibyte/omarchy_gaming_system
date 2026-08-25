---
title: Portable games SDK and remote hosting architecture spike
pipeline_id: cc5d1f80-b2cc-4bb7-929e-657b1e26f761
status: Phase 5 — Complete PASS
ticket: TICKET-014
ticket_doc: docs/planning/tickets/closed/TICKET-014-portable-games-sdk-and-remote-hosting-spike.md
aar: docs/planning/knowledge/aar/AAR-014-portable-games-sdk-and-remote-hosting-spike.md
created: 2026-08-24
---

# Portable games SDK and remote hosting architecture spike — spec

## Intent

Produce an evidence-backed, technically exercised architecture for portable
OmarchyGS games. The target vision keeps platform identity and social services
inside OmarchyGS, permits games to live and version independently, and supports
a later remote game-provider mode without granting providers platform
credentials, account identity, database access, or arbitrary code execution in
the trusted client or server.

This pipeline ships a decision and proof, not production remote hosting.

## Scope

- In: the eight EARS requirements in
  [`TICKET-014`](../../tickets/closed/TICKET-014-portable-games-sdk-and-remote-hosting-spike.md#ears-requirements),
  including execution-model research, an authority and threat model, launch and
  callback contracts, the Game Cartridge and graphics capability profiles,
  separate-repository SDK lifecycle, an isolated cross-process proof, an ADR,
  and follow-up tickets.
- Out: production external access, a released SDK, arbitrary native plugins,
  marketplace/commercial policy, a durable migration, implementation of a real
  game, and any constitution change during the spike that would pre-approve the
  researched outcome.

## Acceptance criteria (EARS)

The authoritative acceptance criteria are REQ-001 through REQ-008 in
[`TICKET-014`](../../tickets/closed/TICKET-014-portable-games-sdk-and-remote-hosting-spike.md#ears-requirements).

## Locked decisions

| # | Decision | Why |
|---|---|---|
| 1 | Treat OmarchyGS as the long-term platform control plane for authentication, personas/avatars, social state, discovery/launch, achievements, notifications, and provider trust policy. | These are the cross-game capabilities that give the system one durable identity and community. |
| 2 | Treat server-side gameplay owned by a registered game provider as the long-term target to evaluate, while preserving the current compiled runtime until a later approved implementation changes it. | This captures the requested direction without claiming the present constitution already permits delegated authority. |
| 3 | Require first-party games, including BBS-inspired ports, to be independently versioned and capable of living in separate repositories; evaluate how they use the same contract or conformance suite as later providers. | A first-party-only private interface would recreate coupling and leave portability untested. |
| 4 | Never give a game provider direct OmarchyGS database access, account credentials, reusable device-session tokens, or private account ownership. | Provider compromise must remain bounded to explicit game and persona capabilities. |
| 5 | Treat backend execution and frontend delivery as one integration problem; the spike is incomplete if it solves only remote commands or only UI embedding. | Players need a safe, coherent way to launch and interact with independently hosted games. |
| 6 | Require an isolated cross-process proof plus negative checks before recommending a protocol, but do not merge a production remote adapter in this ticket. | The proof must expose impractical assumptions without turning exploratory code into an accidental public boundary. |
| 7 | Complete this spike before resuming challenge and first-game implementation, then use its accepted follow-up sequence to avoid speculative plumbing. | The next product slices create the launch/session seams that would otherwise be expensive to reverse. |
| 8 | Name the primary frontend artifact an **OmarchyGS Game Cartridge**: an immutable signed package of declarative screens, schemas, metadata, localization, and bounded assets rendered only by trusted platform components. Raw cartridge QML, JavaScript, native code, arbitrary URLs, and direct network access are prohibited. | This preserves the ROM-like product identity and gives untrusted games a data boundary instead of treating QML as a sandbox. |
| 9 | Target a polished rich-2D profile first, with terminal, board/card, tile/sprite, animation, particle/effect, and audio capabilities supplied by the host. Treat advanced rendering, WebEngine, and constrained 3D as separately negotiated future profiles. | Rich local presentation is compatible with asynchronous BBS-style games; unbounded custom rendering would erase portability and containment. |

## Linked artifacts

- Ticket: [TICKET-014](../../tickets/closed/TICKET-014-portable-games-sdk-and-remote-hosting-spike.md)
- Architecture: [system overview](../../../architecture/system-overview.md)
- Cartridge proposal: [Game Cartridges](../../../architecture/game-cartridges.md)
- Product: [product charter](../../../product-charter.md)
- Intake:

## Phase plan

| Phase | Deliverable | Exit gate |
|---|---|---|
| 1 Plan | Spike ticket, bounded questions, spec, notes, and open AAR | scope captured |
| 2 Design | Current-seam map, research sources, option matrices, threat model, proof design, and exact file manifest | CodeGraph design evidence and actionable proof plan |
| 3 Implement | Isolated cross-process proof, protocol artifacts, ADR draft, and proposed follow-up tickets | proof runs and artifacts match the design |
| 3.5 Inspect | Correctness, security, trust, portability, operations, and frontend containment finding ledger | findings resolved or explicitly reflected in recommendation |
| 4 Validate | Conformance/negative proof runs and repository delivery gate | matching gate receipt |
| 5 Complete | EARS audit, accepted recommendation, docs/OpenWiki, submitted AAR, and archive | no silent drops |
| Delivery | Fresh gate, staged review, authorized commit | matching receipt |

---
title: Operator-custom cartridge trust, import, and player warnings
pipeline_id: b4f37837-c7c8-4a29-9747-fb128045c289
status: Phase 5 — Complete PASS
ticket: TICKET-038
ticket_doc: docs/planning/tickets/closed/TICKET-038-operator-custom-cartridge-trust-import-and-player-warnings.md
aar: docs/planning/knowledge/aar/AAR-038-operator-custom-cartridge-trust-import-and-player-warnings.md
created: 2026-08-27
---

# Operator-custom cartridge trust, import, and player warnings — spec

## Intent

Ship the explicit operator-custom cartridge path reserved by ADR-0003: an owner
may import publisher-signed inert content under a server-local authority, but
players must knowingly pin that authority and the system must never describe
the content as marketplace reviewed or broaden its presentation/gameplay
authority.

## Scope

- In:
  - server-local custom signing, import, lifecycle, provenance, admission,
    distribution, and audit;
  - explicit per-server client trust, custom acquisition verification, private
    cache/mount provenance, QML warnings, and historical presentation;
  - additive forward-only schema and complete hostile/recovery evidence.
- Out:
  - executable cartridge content, implicit client trust, server modules/hooks,
    provider registration, marketplace review, and external infrastructure.

## Acceptance criteria (EARS)

The binding acceptance criteria are REQ-001 through REQ-015 in
[`TICKET-038`](../../tickets/closed/TICKET-038-operator-custom-cartridge-trust-import-and-player-warnings.md).

## Locked decisions

| # | Decision | Why |
|---|---|---|
| 1 | Custom content remains the existing publisher-signed inert cartridge format and trusted renderer boundary. | Marketplace bypass changes provenance, not containment or executable authority. |
| 2 | A new server-scoped operator attestation/acquisition format will be used instead of fabricating marketplace snapshot or review evidence. | Reusing marketplace-shaped review fields would make absence of review ambiguous. |
| 3 | The operator catalog key signs custom provenance/lifecycle only; the normal server receives public material and never reads the private key. | Routine serving and compromise must not gain signing authority. |
| 4 | Player trust is an explicit local-companion enrollment bound to canonical server origin, stable server UUID, and exact operator key. | A selected server may advertise a candidate key but cannot silently make itself trusted for custom content. |
| 5 | Existing marketplace JSON and verification remain byte-shape compatible; mixed catalogs use an exact provenance union only for custom rows. | Existing vetted releases and installed clients must not be relabeled or weakened. |
| 6 | Custom import does not register gameplay code or select a backend; sessions keep using compiled or registered-provider authority. | A cartridge is presentation/input metadata, never the server-side rules engine. |
| 7 | Automatic or transparent custom-key replacement is out of scope; mismatch fails closed and requires an explicit future recovery/reenrollment procedure. | Silent key rollover would defeat the player's server-specific trust decision. |

## Phase 2 — Design

- Add one canonical server-scoped operator attestation and acquisition format
  to the cartridge contract. It reuses publisher release and lifecycle
  verification but contains no marketplace snapshot, reviewer, or root claim.
- Store custom authority/release/lifecycle/audit state in additive PostgreSQL
  tables, extend the existing catalog selection with a mutually exclusive
  custom release reference, and pin provenance into new session presentations.
- Keep private signing material admin-only. Normal serving loads only the
  configured public key, compares it with the immutable database authority,
  and advertises the candidate public identity in discovery when enabled.
- Add a private descriptor-anchored client trust store keyed by canonical
  server origin and stable UUID. Enrollment/removal are authenticated local
  companion operations initiated by explicit QML actions; advertised keys are
  never automatically trusted and an existing binding cannot be overwritten.
- Generalize verified acquisition, cache mount, and render trust into exact
  marketplace-vetted versus operator-custom variants. Legacy marketplace
  acquisition remains unchanged; mount v1 records are validated and upgraded
  in memory to an exact v2 provenance union.
- Extend catalog/session/QML contracts only where custom provenance is present.
  Marketplace release JSON stays unchanged, while custom rows and sessions
  carry a permanent plain-text warning and exact operator fingerprint.
- The complete implementation/test/threat model and file manifest are recorded
  in the matching running notes. Phase 2 is PASS subject to its matching
  CodeGraph design receipt.

## Linked artifacts

- Ticket: [TICKET-038](../../tickets/closed/TICKET-038-operator-custom-cartridge-trust-import-and-player-warnings.md)
- Architecture: [ADR-0003](../../../architecture/adr-0003-owner-operated-server-and-extension-boundary.md), [Game Cartridges](../../../architecture/game-cartridges.md)
- Prior pipelines: [Ticket 032](../completed/marketplace-sync-and-server-catalog-control.spec.md), [Ticket 033](../completed/player-cartridge-acquisition-cache-and-mount-lifecycle.spec.md), [Ticket 035](../completed/historical-session-cartridge-acquisition-and-multi-screen-navigation.spec.md), [Ticket 037](../completed/static-marketplace-publication-offline-root-handoff-and-mirror-operations.spec.md)

## Phase plan

| Phase | Deliverable | Exit gate |
|---|---|---|
| 1 Plan | Ticket, spec, notes, open AAR | autonomous approved-continuation scope review |
| 2 Design | Authority/acquisition schema, file manifest, regression and threat model | actionable design plus CodeGraph receipt |
| 3 Implement | Contracts, migration, server/client/QML paths, tests, docs | focused compile/tests and self-review |
| 3.5 Inspect | Correctness/security/authority findings ledger | verified dispositions plus fresh CodeGraph receipt |
| 4 Validate | Focused suites and complete delivery gate | matching worktree gate receipt |
| 5 Complete | AC audit, OpenWiki, AAR, archive | no silent drops |
| Delivery | Fresh gate, staged review, commit/push | matching receipt and remote readback |

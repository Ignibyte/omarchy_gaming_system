---
title: <TITLE>
pipeline_id: <uuid4>
status: Phase 1 — Plan: in progress
ticket: TICKET-<number>
ticket_doc: docs/planning/tickets/open/TICKET-<number>-<slug>.md
aar: docs/planning/knowledge/aar/AAR-<number>-<slug>.md
created: YYYY-MM-DD
---

# <TITLE> — spec

## Intent

<What this pipeline ships and why now.>

## Scope

- In:
- Out:

## Acceptance criteria (EARS)

| ID | EARS requirement | Verification |
|---|---|---|
| REQ-001 | When `<trigger>`, the system shall `<observable behavior>`. | `<test, command, or review>` |

## Locked decisions

| # | Decision | Why |
|---|---|---|
| 1 | | |

## Linked artifacts

- Ticket:
- Architecture:
- Intake:

## Phase plan

| Phase | Deliverable | Exit gate |
|---|---|---|
| 1 Plan | Ticket, spec, notes, open AAR | human confirmation |
| 2 Design | Architecture, file manifest, regression plan | human confirmation |
| 3 Implement | Code matching the design | compile and self-review |
| 3.5 Inspect | Findings ledger and fixes | lead disposition |
| 4 Validate | Tests run and delivery gate green | gate receipt |
| 5 Complete | AC audit, docs, submitted AAR, archive | no silent drops |
| Delivery | Fresh gate, staged review, authorized commit | matching receipt |

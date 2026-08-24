---
phase: 1
title: Pipeline Planner
---

You are Phase 1: **Plan**. Produce the ticket, EARS spec, running notes, and
open AAR. Do not write application code or settle implementation details.

Read [CONSTITUTION.md](../../../CONSTITUTION.md) §3, §18, and §19.

## Entry checks

- Confirm `docs/planning/pipeline/active/` has no other spec.
- Search `docs/planning/knowledge/INDEX.md` and nearest completed notes.
- Read relevant `docs/architecture/*.md`.
- Create one Claude task per step below before starting; resolve all at close.

## Steps

1. Parse `$ARGUMENTS` and keep it to one shippable slice.
2. Take the next number from `docs/planning/tickets/INDEX.md` and increment it.
3. Create the ticket from `docs/planning/_templates/ticket.md`, add it to the
   open queue, and preserve any intake link.
4. Create a real UUID and instantiate the spec/notes templates under
   `docs/planning/pipeline/active/`.
5. Write scope in/out, locked decisions, and an EARS acceptance table with one
   observable requirement and verification method per row.
6. Open the AAR from its template and seed its recall/surfacing log.
7. Present the scope and criteria for review. Wait unless the user explicitly
   requested autonomous progress through commit.

After confirmation, set the spec status to
`Phase 1 — Plan PASS; ready for Phase 2 — Design`, resolve all tasks, and say:
**Phase 1 PASS. Run `/pipeline:design`.**

$ARGUMENTS

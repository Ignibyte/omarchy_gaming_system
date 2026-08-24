---
phase: 2
title: Pipeline Designer
---

You are Phase 2: **Design**. Turn the confirmed active spec into an exact
implementation design and regression plan. Do not write application code.

Read [CONSTITUTION.md](../../../CONSTITUTION.md). Phase 1 must be PASS.

## Before starting

- Re-read the active spec and its ticket.
- Read relevant architecture docs and knowledge entries.
- Inspect existing producers, consumers, migrations, and tests.
- Create one Claude task per design step; resolve all at close.

## Required design in the notes

1. Architecture and data flow, including server/client ownership.
2. Exact file manifest with one purpose per file.
3. Database and migration consequences.
4. API contract changes and compatibility behavior.
5. Regression table mapping every EARS requirement to test evidence.
6. Security, privacy, concurrency, reconnect, and rollback risks as relevant.
7. Decisions made and alternatives rejected.

Present for review. After confirmation, set status to
`Phase 2 — Design PASS; ready for Phase 3 — Implement`, resolve all tasks, and
say: **Phase 2 PASS. Run `/pipeline:implement`.**

$ARGUMENTS

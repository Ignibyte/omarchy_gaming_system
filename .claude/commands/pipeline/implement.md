---
phase: 3
title: Pipeline Implementer
---

You are Phase 3: **Implement**. Build the approved file manifest and nothing
outside the confirmed scope.

Read [CONSTITUTION.md](../../../CONSTITUTION.md) §10 and §14. Phase 2 must be
PASS. Before writing code, re-read relevant architecture docs, search the local
knowledge register, and inspect callers. Create tasks per file or logical unit.

## Rules

- Keep Axum handlers thin and domain/game logic deterministic and testable.
- Keep account identity separate from persona identity.
- Use forward-only SQL migrations; never rewrite a migration that may have run.
- Preserve API compatibility or document an intentional versioned break.
- Implement client loading/offline/error states and keyboard behavior.
- Run `cargo check --workspace` and formatting as you work. Cargo commands are
  sequential. Phase 4 owns the complete test/gate execution.
- Record deviations and their reasons in the running notes.

Set status to
`Phase 3 — Implement PASS; ready for Phase 3.5 — Inspect`, resolve all tasks,
and say: **Phase 3 PASS. Run `/pipeline:inspect`.**

$ARGUMENTS

---
phase: 3.5
title: Pipeline Inspector
---

You are Phase 3.5: **Inspect**. Run a skeptical review of the implementation
before validation. Phase 3 must be PASS.

Read [CONSTITUTION.md](../../../CONSTITUTION.md) §18. Create tasks for critics,
triage, fixes, verification, and the ledger.

Review the complete diff through independent lenses, using Claude subagents in
parallel when available:

- correctness and EARS coverage;
- auth/authz, inputs, secrets, identity privacy, and abuse cases;
- SQL migrations, transactions, concurrency, idempotency, and saved-game state;
- unnecessary complexity and missed reuse;
- QML loading/error/empty states, keyboard navigation, and visual regressions.

Critics are read-only. The lead verifies each finding, rejects false positives
with a reason, and fixes confirmed defects. Write every finding and disposition
to `## Phase 3.5 — Inspect ledger` in the active notes. Record real failures in
the AAR and register durable rules in the knowledge index.

Set status to
`Phase 3.5 — Inspect PASS; ready for Phase 4 — Validate`, resolve all tasks,
and say: **Phase 3.5 PASS. Run `/pipeline:validate`.**

$ARGUMENTS

---
phase: 4
title: Pipeline Validator
---

You are Phase 4: **Validate**. Write any remaining tests from the Phase 2 plan,
run them, and obtain a real delivery-gate result. Phase 3.5 must be PASS.

Read [CONSTITUTION.md](../../../CONSTITUTION.md) §0, §7, and §15. Create one
task per planned test plus a gate task.

1. Implement every test in the regression table, including inspect findings.
2. Run focused tests, then `cargo test --workspace`; report actual output.
3. Run `bin/gate.sh --diff`. Fix every red at its source and re-run until it
   prints `GATE GREEN [diff]`.
4. Record commands, outcomes, smoke evidence, and any justified pre-existing
   failure in the notes.

Set status to
`Phase 4 — Validate PASS; ready for Phase 5 — Complete`, resolve all tasks,
and say: **Phase 4 PASS. Run `/pipeline:complete`.**

$ARGUMENTS

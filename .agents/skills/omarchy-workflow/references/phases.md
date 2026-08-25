# Omarchy Gaming System workflow phases

## Recall and preflight

Before opening or resuming implementation work:

- Read `docs/planning/bulletins/INDEX.md`; a critical bulletin blocks work.
- Search `docs/planning/knowledge/INDEX.md`, the nearest completed pipeline
  notes, and relevant architecture documents.
- Inspect the affected code, callers, migrations, and tests.
- Run `scripts/check-pipeline-tools.sh`; bootstrap with
  `scripts/setup-pipeline-tools.sh` when local generated state is absent.
- Check the toolchain needed by the slice and run `docker compose ps` when the
  database or live client path matters.

Record useful recalled material in the running notes and AAR. If an active spec
already exists, resume it rather than creating new artifacts.

## Phase 1 — Plan

Do not write application code or settle unnecessary implementation details.

1. Keep the request to one shippable slice.
2. Take and increment the next number in `docs/planning/tickets/INDEX.md`.
3. Create the open ticket from `docs/planning/_templates/ticket.md` and add it
   to the open queue.
4. Create a real UUID and instantiate the spec/notes templates in
   `docs/planning/pipeline/active/`.
5. Write scope in/out, locked decisions, and observable EARS requirements with
   an explicit verification method for each.
6. Open an AAR and seed its recall log.

When scope is settled, set the spec status to
`Phase 1 — Plan PASS; ready for Phase 2 — Design`.

## Phase 2 — Design

Do not write application code. Re-read the active spec, ticket, relevant
architecture, knowledge entries, and actual producers and consumers.

Add the following to the running notes:

1. Architecture and data flow, including server/client ownership.
2. Exact file manifest with one purpose per file.
3. Database and migration consequences.
4. API contract and compatibility behavior.
5. A regression table mapping every EARS requirement to evidence.
6. Relevant security, privacy, concurrency, reconnect, and rollback risks.
7. Decisions made and material alternatives rejected.

Use `codegraph_explore` for the relevant symbols and runtime flows after the
plan is stable. When the MCP server is unavailable only because Codex has not
restarted after setup, run `scripts/codegraph.sh explore ...`. Record useful
topology, blast-radius, and coverage evidence in the notes. Directly inspect
unsupported file types; CodeGraph evidence complements rather than replaces
that review.

When the design is actionable, set the status to
`Phase 2 — Design PASS; ready for Phase 3 — Implement`. Codex's Stop hook
rejects this claim unless the current pipeline and gated worktree have a design
CodeGraph receipt.

## Phase 3 — Implement

Build the approved file manifest and remain inside the confirmed scope.

- Keep Axum handlers thin and domain/game logic deterministic and testable.
- Keep account identity separate from persona identity.
- Use forward-only SQL migrations; never rewrite a migration that may have run.
- Preserve API compatibility or document an intentional versioned break.
- Preserve QML loading, offline, empty, error, and keyboard behavior.
- Run formatting and focused compilation/checks during implementation.
- Record deviations and reasons in the notes.

When implementation and focused checks are ready for independent review, set
the status to
`Phase 3 — Implement PASS; ready for Phase 3.5 — Inspect`.

## Phase 3.5 — Inspect

Review the complete diff skeptically through the relevant independent lenses:

- correctness and EARS coverage;
- authentication, authorization, input, secrets, privacy, and abuse cases;
- migrations, transactions, concurrency, idempotency, and game state;
- unnecessary complexity and missed reuse;
- QML loading/error/empty states, keyboard navigation, and visual regressions.

Use specialized review skills when the task triggers them. Verify every
finding, reject false positives with a reason, fix confirmed defects, and write
the finding/disposition ledger in the active notes. Record durable failures or
rules in the AAR.

After the last gated implementation or inspection fix, run a fresh
`codegraph_explore` over the changed symbols and one-hop dependents. If the MCP
server is pending restart, use the repository wrapper fallback. Reconcile its
blast radius and test hints with the finding ledger.

Set the status to
`Phase 3.5 — Inspect PASS; ready for Phase 4 — Validate` only after confirmed
findings are resolved. The Stop hook rejects this claim unless the
post-implementation gated worktree has a fresh inspect CodeGraph receipt.

## Phase 4 — Validate

1. Implement every remaining test from the regression table.
2. Run focused tests and relevant workspace tests; record actual commands and
   outcomes.
3. Run `bin/gate.sh --diff` after the last gated edit. Fix red at its source
   and rerun until it prints `GATE GREEN [diff]`.
4. Record smoke evidence and any honest pre-existing failure or skip.

Only a matching worktree receipt proves delivery validation. Set the status to
`Phase 4 — Validate PASS; ready for Phase 5 — Complete` after it exists, and
report the milestone as `Phase 4 PASS`.

## Phase 5 — Complete

1. Audit every EARS requirement. Mark it satisfied with concrete evidence or
   open a follow-up ticket; never silently drop one.
2. Invoke `$openwiki` in `init` mode when no generated wiki exists, otherwise
   `update` mode. Reconcile affected architecture, flows, tests, and operations
   through Grounded Claims, then call `openwiki_finish` until it returns
   complete. This must issue a matching completion receipt.
3. Update affected hand-maintained architecture, product, API, and operator
   documentation that is outside the generated wiki.
4. If OpenWiki or completion edits changed any gated path after Phase 4, rerun
   `bin/gate.sh --diff` and retain the matching delivery receipt.
5. Submit and date the AAR. Put every new `BF-*`, `PR-*`, and `AD-*` in both
   the AAR and `docs/planning/knowledge/INDEX.md`.
6. Move the ticket from `open/` to `closed/`, mark it done, and remove its open
   queue row.
7. Set the status to `Phase 5 — Complete PASS`, then move the spec/notes pair
   from `active/` to `completed/`.

Report the milestone as `Phase 5 PASS` only after the active pair is archived.
The Stop hook rejects the claim unless the archived pipeline matches the fresh
OpenWiki completion receipt.

## Delivery — only when authorized

1. Run `bin/gate.sh --diff` after the final gated edit.
2. Confirm completed pipeline work has no active pair and its ticket is closed.
3. Stage the intended files and inspect the staged diff for credentials,
   generated state, and unrelated changes.
4. Commit with a clear subject/body and ticket identifier.
5. Push or open a pull request only when separately authorized.

Report the gate result, commit SHA and branch, plus push or pull-request status.

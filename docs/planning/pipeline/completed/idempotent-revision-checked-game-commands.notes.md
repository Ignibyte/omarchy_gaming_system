---
title: Idempotent revision-checked game commands — notes
pipeline_id: 0857c2e2-6272-46f1-88d0-972c3d6d8f97
---

# Idempotent revision-checked game commands — running notes

Chronological evidence and decisions. If a test or command did not run, these
notes must not claim it passed.

## Phase 1 — Plan

- User directive: continue through the ordered five-ticket set. Ticket 012 is
  archived and its final gate, OpenWiki, and gated-state receipts match, so
  Ticket 013 is the fifth slice and the only active pipeline.
- Knowledge recall: durable game snapshots must pin an exact immutable rules
  version; game surfaces expose personas rather than account ownership;
  repository-derived command paths need explicit parsing boundaries; graph
  coverage remains advisory; and transaction/concurrency behavior needs real
  PostgreSQL evidence.
- Constitution and architecture recall: game mutations must be idempotent and
  revision-aware; compiled game code is deterministic and database-free;
  transactions update snapshot, revision, and notifications atomically; REST is
  durable truth and WebSockets remain wakeup hints.
- OpenWiki recall: Ticket 012 exposes only public compiled metadata and private
  stored session reads. Trusted creation already owns exact-version initial
  state, ordered seats, canonical participant locks, and minimal sync events.
  Ticket 013 must resolve that stored exact version before mutation and must not
  add a public creation route.
- Smallest honest slice: one participant command POST with bounded object JSON,
  a session-wide UUID idempotency receipt, optimistic expected revision,
  deterministic exact-version transition, atomic snapshot/revision/receipt/sync
  persistence, and no user-visible command history. Challenges, turns, time,
  randomness, results, production rules, and QML game UI remain later work.

## Phase 2 — Design

- Required recall completed against the knowledge index, OpenWiki runtime and
  product pages, Ticket 012's nearest completed notes, Constitution sections
  10/14/15/18, the system overview, and the existing game API contract.
- CodeGraph design exploration traced the immutable registry through trusted
  creation, private session reads, router assembly, and `sync::append_event`.
  Receipt `.git/omarchy-gaming-system-pipeline-tools/design.receipt` records
  pipeline `0857c2e2-6272-46f1-88d0-972c3d6d8f97`.
- Locked the route as a body-limited participant-persona POST. Responses expose
  only the session ID, committed revision, and state and inherit `no-store`.
- Locked session-wide replay ordering: session row lock, receipt lookup, replay
  comparison using JSONB semantics, then expected-revision enforcement. This
  permits a genuine retry after the first command has advanced the revision.
- Locked the runtime boundary to exact stored key/version plus bounded object
  state, actor seat, and bounded object command. Compiled game definitions get
  no ambient infrastructure or nondeterministic source.
- Locked successful commands as the only receipt-producing outcome. State,
  revision, receipt, timestamp, and one minimal invalidation per participant
  commit or roll back together.
- Rejected WebSocket mutation, actor-scoped keys, raw-JSON comparison, visible
  command history, and automatic rules-version substitution.

## Phase 3 — Implement

- Extended `GameDefinition` with a deterministic command transition and added
  exact-version registry enforcement for bounded object state, actor seat,
  bounded object command, stable rejection, and bounded object output.
- Added forward-only migration 0011 with a session-wide UUID primary key,
  participant composite foreign key, one-receipt-per-applied-revision unique
  constraint, one-step revision check, and object-shaped JSONB command/state.
- Added the body-limited command POST plus the serialized domain transaction:
  owner auth, participant session row lock, receipt-before-revision lookup,
  exact-version transition, snapshot/revision/timestamp update, receipt insert,
  minimal participant sync events, and one commit.
- Added stable API errors and a minimal no-store success response. Production
  still builds an empty registry; test routers inject deterministic rules.
- Added runtime bounds/exact-version/rejection tests and PostgreSQL API tests for
  committed state, semantic JSONB replay (`1` versus `1.0` and reordered
  objects), all collision axes, stale/future revisions, rejection/invalid
  output rollback, unavailable rules, privacy, and two concurrent commands.
- `cargo test -p omarchy-game-runtime`: PASS, 5 tests.
- `cargo check -p omarchy-gaming-system-server --tests`: PASS after adding the
  required fixture transition.
- `./scripts/test-database.sh`: PASS, all 33 PostgreSQL tests including the
  three new command cases.
- `cargo clippy --workspace --all-targets -- -D warnings`: PASS after grouping
  the command fields into `GameCommandInput`.
- `cargo test --workspace`: PASS, 34 local tests across the runtime and server
  (33 PostgreSQL cases intentionally ignored in this command).
- Updated the README, API reference, and system overview with the durable
  command and WebSocket boundary.

## Phase 3.5 — Inspect ledger

| # | Lens | Finding | Severity | Disposition |
|---|---|---|---|---|
| 1 | Transaction time | `now()` is fixed at transaction start, so a command that waits on the session lock could write an older timestamp than a transaction that committed first. | correctness | Fixed with `GREATEST(updated_at, clock_timestamp())`; the database test proves a committed command advances the timestamp. |
| 2 | Replay identity | The actor and expected-revision collision cases also changed command JSON, so they did not independently prove each session-wide idempotency axis. | test gap | Fixed: actor-only, revision-only, and command-only mismatches now differ on exactly one receipt identity field. |
| 3 | HTTP boundary | The 32 KiB command body cap was configured but had no isolated pre-database regression. | test gap | Fixed with a lazy-pool router test proving `413 Payload Too Large` and `Cache-Control: no-store`. |
| 4 | Security diff scan | A frozen 69-file review found no Ticket 013 command vulnerability, but dynamically proved that newline-bearing Git paths bypassed gated-state hashing, commit classification, and high-signal secret scanning. | low | Fixed after explicit user approval with NUL-delimited Git inventories, sorting, consumers, and hash records. Isolated reproduction now changes the hash and produces exit `2` from both enforcement hooks; newline, space, dash, and ordinary controls pass self-tests. |
| 5 | Security diff scan | CodeGraph used an exact npm version but no repository-reviewed wrapper/platform digest before native setup and MCP execution. | low | Fixed after explicit user approval: setup verifies pinned SHA-512 tarballs, installs only the reviewed platform pair with lifecycle scripts disabled, checks a hardcoded relative package-tree digest and executable link, and writes provenance that readiness revalidates. A one-line installed-package tamper was rejected before exact restoration. |
| 6 | Final structural inspection | Fresh CodeGraph traced the sole HTTP caller through participant authorization, receipt-before-revision replay, exact-version runtime transition, atomic state/receipt/sync commit, and current test blast radius. Its test-link heuristic did not associate the integration tests, so their runtime evidence was inspected directly. | none | Pass; no missing authorization, privacy, replay, rollback, concurrency, timestamp, or response-shape branch remained. Fresh worktree-bound inspect receipt recorded. |

- Codex Security scan `3aec4c1d-8c0e-44c6-b7da-b8e1d3e5339a`
  completed against frozen digest
  `b8befbfd509e3e6076459c4356ba0f20fa59116dae3812e3ed82fc2043cd1c55`.
  TAC access could not be verified because that connector was not connected;
  this did not limit repository evidence or the two local remediations.
- Focused remediation checks: Bash syntax PASS; hook self-tests PASS; original
  newline reproduction changed the receipt hash and returned `2` from commit
  and secret hooks; CodeGraph setup/readiness PASS; tamper-negative readiness
  check returned `1`; restored readiness PASS.
- `bin/gate.sh --fast`: PASS after the final remediation, including rustfmt,
  clippy, 35 local Rust tests, rustdoc, Compose, shell syntax, pipeline
  structure, changed-file secret scan, hook self-tests, and whitespace.

## Phase 4 — Validate

- `bin/gate.sh --diff`: PASS.
- The gate passed rustfmt, workspace/all-target Clippy with warnings denied,
  35 local Rust tests, rustdoc with warnings denied, Compose validation, Bash
  syntax, pipeline structure, changed-file secret scanning, Codex hook
  self-tests, whitespace checks, all 33 real PostgreSQL integration tests, and
  the PostgreSQL → Rust API → visible QML smoke path.
- The PostgreSQL command cases passed committed transition/timestamp, semantic
  replay, isolated collision axes, stale/future revisions, rejection and
  unavailable-rule rollback, authorization privacy, and one-winner
  concurrency.
- Delivery receipt
  `.git/omarchy-gaming-system-gate-receipt` contains
  `761bbecf56c43da27f37ba811fc3350a17c66ba395e6f04cbf64f0796fba56a4`,
  exactly matching the current gated-state hash after validation.

## Phase 5 — Complete

- Acceptance-criteria audit:
  - REQ-001 PASS: five runtime tests prove exact-version execution,
    deterministic transition, input/output bounds, and stable rejection.
  - REQ-002 PASS: multi-account PostgreSQL/router coverage preserves owner and
    participant privacy for absent, malformed, and unauthorized sessions.
  - REQ-003 PASS: committed commands update state, revision, and a monotonic
    timestamp exactly once; stale and future revisions leave no durable change.
  - REQ-004 PASS: semantic JSONB replay returns the original response, while
    actor-only, revision-only, and command-only UUID collisions are rejected.
  - REQ-005 PASS: the concurrent PostgreSQL case produces one winner at the
    shared expected revision and no lost update.
  - REQ-006 PASS: a commit appends one minimal invalidation per participant;
    rejection, conflict, replay, unavailable rules, and rollback append none.
  - REQ-007 PASS: the canonical diff gate exercised 35 local Rust tests, all
    33 PostgreSQL tests, the empty production catalog, and the visible QML
    smoke path.
- Docs: README, API, architecture, and product contracts were reconciled.
  OpenWiki update `c456c89a-bb36-4327-bcc2-d170501cc92c` completed and wrote a
  worktree-bound lifecycle receipt for runtime, product, validation, and Codex
  workflow pages.
- AAR: submitted at effectiveness 5/5 with four failure IDs, five prevention
  rules, and the idempotent revision command-boundary architecture decision;
  every ID is registered in the knowledge index.
- Archive: Ticket 013 closed, its roadmap item checked, ticket numbering
  advanced to 014, and the sole active spec/notes pair moved to completed.
- Final delivery proof: `bin/gate.sh --diff` passed again after OpenWiki and
  archive completion. Gate receipt and OpenWiki completion receipt both match
  current gated-state hash
  `6fadfd495a1a3a4f72c396fee1c14535c089271024f930bfe27356b25e686951`.

## Defect and lesson ledger

| # | What happened | Root cause | Fix | Prevention |
|---|---|---|---|---|
| 1 | A focused test command initially named a nonexistent `omarchy-server` package. | The package ID was recalled instead of read from the manifest. | Reran with `omarchy-gaming-system-server`; the test passed. | Read the owning `Cargo.toml` before using a package-qualified focused command. |
| 2 | The first integrity-pinned setup rejected the installed CodeGraph tree. | The reviewed tree digest had been derived from newline-formatted checksum records while the committed verifier intentionally uses NUL records. | Recomputed both reviewed platform digests with the exact verifier encoding; setup then passed and wrote provenance. | Derive and verify pinned aggregate digests with the same byte-record algorithm, and preserve fail-closed setup behavior. |

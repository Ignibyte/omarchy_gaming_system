---
title: First-party remote-provider migration pilot — notes
pipeline_id: f1e50ed7-4fdc-4df7-9aa9-5a208b7405a5
---

# First-party remote-provider migration pilot — running notes

Chronological evidence and decisions. If a test or command did not run, these
notes must not claim it passed.

## Phase 1 — Plan

- Recall: the branch is clean at delivered Ticket 018 commit `9f72a79`; no
  active bulletin or pipeline blocks work. Ticket 019 is the sole open ticket,
  and `scripts/check-pipeline-tools.sh` passed CodeGraph 1.5.0, OpenWiki 0.3.3,
  frozen pnpm, patch, build, and Codex-only provenance checks.
- Recall: Constitution §10 still assigns all game rules/state/revision
  authority to the platform. ADR-0002 accepts only the data-only cartridge and
  staged broker boundary until this ticket records the explicit amendment.
- Recall: Ticket 018 provides immutable provider releases, independent message
  and TLS key rotation, pairwise one-scope grants, public-only pinned HTTPS,
  durable operation/callback receipts, quotas, leases, lifecycle, audit, a
  separate TLS fixture, and gate 17. It deliberately has no player caller or
  projection transaction.
- Recall: `AD-omarchy-gaming-system-game-cartridge-provider-boundary-001` and
  `AD-omarchy-gaming-system-remote-provider-security-foundation-001` reserve
  accounts, personas, catalog, envelope, result acceptance, achievements,
  notifications, and recovery to OmarchyGS while allowing exactly one
  registered provider to own remote gameplay.
- Recall: `PR-omarchy-gaming-system-bind-provider-messages-to-registered-identity-001`
  requires the provider/release/game/rules/cartridge/session/subject/scope/
  expiry context everywhere. `PR-omarchy-gaming-system-serialize-dedupe-on-existing-root-001`
  requires callback first delivery to serialize on an extant root, and both
  replay-before-current-admission rules apply to player retries.
- Recall: the current `game_sessions.state` is non-null and authoritative, and
  `games::apply_command` always invokes the compiled `GameRegistry` under a
  session-row lock. Provider activation therefore requires a new explicit
  authority discriminator and constraints; overloading the current JSON state
  would silently create a writable shadow.
- Recall: Door Legends already builds from two clean Git clones with the public
  cartridge SDK and signed release evidence. Extending that same first-party
  repository into a separate provider is the shortest honest path to the
  user's cartridge-plus-remote-server model without rewriting Signal Siege.
- Decision: the complete seven-requirement Door Legends authority pilot is one
  shippable slice. Existing compiled games remain compatible; external
  onboarding, arbitrary UI, direct client networking, and Git delivery remain
  excluded.

## Phase 2 — Design

- CodeGraph evidence: the worktree-bound design receipt records pipeline
  `f1e50ed7-4fdc-4df7-9aa9-5a208b7405a5` at state hash
  `102ea24af0175b5369ca2e8edc372ee3f94854ce1283eaac8e481cbd968345dc`.
  The trace covers the public game handlers through `GameRegistry` and
  `game_sessions`, plus `ProviderBroker`, durable operation/callback receipts,
  provider lifecycle, audit, and sync. Its blast radius includes server app,
  game/challenge APIs, runtime registry, provider registry/broker/protocol, and
  their integration suites.
- Architecture: add an explicit `platform_compiled` or
  `registered_provider` authority to each session. Compiled sessions retain
  their current non-null rules state and registry path. Provider sessions pin
  one immutable release, keep local rules state null, and dispatch launch,
  command, and reconcile only through `ProviderBroker`. A bounded,
  authenticated provider view cache may be returned to the cartridge but is
  never command input or recovery authority.
- Architecture: Door Legends runs as a separately built Rust/Axum TLS process
  with an independent PostgreSQL database. It owns its rules state, revision,
  time, outcome, operation receipts, and callback outbox. The platform owns
  authentication, personas, catalog activation, the session envelope,
  provider grants, broker receipts, public result/achievement definitions and
  projections, participant sync, lifecycle policy, and audit. Neither side
  receives the other database's credentials.
- Architecture: split `omarchy-game-provider` into a public protocol/model
  surface usable with default features disabled and a default `platform`
  feature containing registry, broker, egress, and operator controls. The
  production server uses the real broker only. Exact-loopback conformance
  remains test-only and cannot enter its production dependency graph.
- Architecture: extend the provider registry with one operator-enabled pilot
  release, a public catalog manifest and player bounds, lifecycle status, and
  platform-pinned achievement definitions. Provider configuration is
  optional but all-or-none: when entirely absent the remote catalog/routes
  are disabled; when partial or malformed server startup fails.
- Architecture: start first persists an idempotent provisioning envelope and
  participant before crossing the network. A retry resolves the durable start
  receipt and broker operation before current catalog admission. Successful
  authenticated responses advance only the envelope revision/status and view
  cache. Unknown outcomes enter `reconciling`; explicit reconciliation uses
  the same stable operation identity. No transaction or session row lock is
  held across provider I/O.
- Architecture: command dispatch first locks and reads the authority. The
  provider path never reaches `GameRegistry`; it uses a stable idempotency key
  and expected provider revision. Accepted responses conditionally advance the
  provider revision and view. Conflicts return the authenticated provider
  revision/view, while timeout and outage produce an explicit recoverable
  availability state. No timestamps select a winner and no compiled failback
  exists.
- Architecture: refactor callback processing into authentication and a
  transaction-aware receipt claim. The server derives the expected pairwise
  subject from the pinned participant, validates the configured callback
  authority and fixed release path, locks an extant release/session root, and
  atomically deduplicates, validates exact event/release/game/session/subject/
  revision/definition policy, writes allowlisted result and achievement
  projections, appends audit plus sync, and commits. Authenticated but invalid
  policy events are durably ignored and audited without projection.
- Architecture: recovery is REST/cursor based. Suspension disables new work
  and exposes existing sessions read-only. Restore requires provider database
  recovery followed by authenticated reconciliation; permanent retirement is
  terminal. Backup/restore never copies provider gameplay state into the
  platform. WebSockets remain wake-up hints only.
- Data/API manifest: forward-only migration 0015 adds the authority
  discriminator, nullable compiled state constraints, pinned provider release,
  provider availability/revision, view cache, result projection, achievement
  definitions and persona awards, and the singleton pilot activation policy.
  Additive public session fields expose only authority, release identity,
  availability, validated view/result, participant, and existing timestamps.
  Provider endpoints, pairwise subjects, grants, keys, signed bodies, account
  IDs, credentials, and database details have no response serializer.
- File manifest: root/server/provider Cargo manifests and lockfile;
  `crates/game-provider` model, protocol, registry, broker, operator CLI and
  tests; server config/main/app/games/sync plus a new provider-game domain and
  API integration suite; migration 0015; the Door Legends example provider,
  its own migrations and lockfile; an authority-pilot clean-clone script and
  gate stage 18; Constitution §10, ADR-0002, cartridge/API/product/operator
  docs; and this ticket/spec/notes/AAR/OpenWiki lifecycle.
- Regression plan: preserve exact compiled Signal Siege behavior with provider
  config absent. Prove clean-clone protocol-only compilation, separate TLS
  process and databases, start replay and crash recovery, concurrent expected-
  revision commands, timeout-after-commit reconciliation, callback tamper/
  duplicate/race/rollback behavior, exact achievement policy, multi-account
  privacy, outage/restart/backup/restore/suspend/retire states, and production
  exclusion of conformance transport. Run focused unit/API/database/provider
  suites, the new separate-process drill, then `bin/gate.sh --diff`.
- Risk controls: schema constraints prevent dual durable authority; receipt
  roots serialize first delivery; response revisions prevent late responses
  overwriting newer state; operator activation pins identity and definitions;
  provider state never appears in platform backups; public serializers are
  allowlists; and full broker binding/TLS/egress/quota controls remain in force.
- Rejected alternatives: converting live Signal Siege sessions, persisting a
  provider rules snapshot, using the generic fixture as the production pilot,
  direct client-provider sockets, standalone callback receipt commits,
  timestamp conflict resolution, provider-defined global achievements,
  dynamic unconfigured catalog discovery, process-local idempotency, or a
  compiled fallback.
- Regression matrix: REQ-001 maps to Constitution/ADR structure checks;
  REQ-002 to migration and independent-database assertions; REQ-003 to replay,
  race, fault, restart, and reconciliation tests; REQ-004 to atomic callback
  policy/privacy tests; REQ-005 to the lifecycle/DR drill; REQ-006 to exact
  response and multi-account negative tests; and REQ-007 to the clean-clone
  authority-pilot gate.

## Phase 3 — Implement

- Built: migration 0015 adds the exclusive `platform_compiled`/
  `registered_provider` session authority shape, exact release pin,
  availability state, singleton pilot registry, bounded authenticated view,
  immutable result projection, operator-pinned achievement definitions and
  awards, and terminal lifecycle guards. Existing compiled sessions are
  backfilled as platform-owned and keep non-null object state.
- Built: `omarchy-game-provider` now separates the public protocol/model surface
  from its default platform feature, adds singleton pilot activation and
  lifecycle commands, and incorporates pilot status into every broker
  admission. Callback authentication is separate from caller-owned receipt and
  projection transactions.
- Built: the server accepts optional all-or-none provider secrets, constructs
  the production broker only when fully configured, merges the active pilot
  into the catalog, dispatches provider start/command/reconcile without holding
  a database transaction across I/O, exposes participant-private authority/
  availability/view/result fields, and atomically projects authenticated
  callbacks, achievements, audit, and sync.
- Built: Door Legends now includes a separately compiled Rust/Axum TLS provider
  with its own PostgreSQL migrations, authoritative session revisions,
  operation receipts, fault modes, and signed callback outbox. It depends only
  on the packaged public provider protocol when built from a clean clone.
- Built: `scripts/test-provider-authority-pilot.sh` packages that protocol,
  creates and clones a fresh Door Legends repository, rejects platform feature
  or path leakage, runs the real server/provider flow against independent
  databases, and verifies a provider backup restored into a second database.
  Gate 18 makes the proof mandatory in diff/full mode.
- Built: Constitution §10, ADR-0002, API/product/cartridge/operator documents,
  the first-party runbook, smoke expectations, and gate-state coverage now
  describe the accepted first-party authority pilot and its external-provider
  limits.
- Deviations: no existing Signal Siege session was migrated, no cartridge file
  is ingested by the server, and no main-QML gameplay or third-party onboarding
  was added. Those exclusions were intentional locked decisions rather than
  incomplete acceptance criteria.
- Deviation: security inspection required callback quota charging to move
  after signature verification, provider HTTP clients to use only the pinned
  roots and no redirects, lifecycle to be rechecked at callback projection, and
  release/pilot/session locks to use one order. These harden the designed flow
  without changing its public contract.

## Phase 3.5 — Inspect ledger

| # | Lens | Finding | Severity | Disposition |
|---|---|---|---|---|
| 1 | Correctness/replay | An exact authenticated callback replay could be evaluated against the already-advanced current projection policy and proposed as `ignored`, poisoning its original accepted disposition. | Medium | Fixed: the immutable authenticated body/session identity now determines duplicate disposition; first-delivery accepted/ignored policy remains durable and exact replay is effect-free. |
| 2 | Security/availability | Callback quota was charged from unauthenticated traffic before signature verification, allowing a caller with a release UUID to exhaust the shared provider callback window. | Medium | Fixed: load bounded verification material, authenticate exact bytes first, then recheck policy/key/body bounds and charge authenticated quota; a regression proves invalid signatures consume no quota. |
| 3 | Security/lifecycle | General provider admission omitted the first-party pilot lifecycle, and callback projection did not recheck it inside the effect transaction. | Medium | Fixed: pilot status is locked into all launch/command/reconcile/event admission; the projection transaction rechecks the pilot before effects; suspended and retired matrices are exercised. |
| 4 | Security/TLS | The per-provider client merged registered roots with ambient platform roots. | Low | Fixed: the client trusts only the exact registered roots for that operation. |
| 5 | Security/redirects | Provider requests could follow redirects and escape the reviewed origin/path contract. | Low | Fixed: redirects are disabled explicitly and conformance remains fail-closed. |
| 6 | Concurrency | Response and callback paths could acquire provider release and session locks in inverse order, permitting a cross-request deadlock. | Low | Fixed: provider effect paths use release → pilot → session ordering and tests exercise concurrent revisions/callbacks. |
| 7 | Delivery integrity | The gate-state hash omitted the independently built `examples/first-party-door-legends/provider` sources. | Medium | Fixed: the complete first-party Door Legends tree is now gated, so its edits invalidate receipts. |
| 8 | Lifecycle correctness | A player command could remain admissible after pilot suspension/retirement even though current public use is solo/self-only. | Low | Fixed with the same pilot lifecycle overlay; suspension allows only explicit reconciliation and retirement denies every scope. |
| 9 | CodeGraph | Final inspection traced `ProviderRegistry`, `ProviderBroker`, server routing/projection, session loading, migrations, tests, and gate blast radius after remediation. | — | PASS; receipt state hash `97fb3a2218d589e4d2487290d608c03ab639fc74bc489b6912d0655bdb0c8037`. |
| 10 | Codex Security | The sealed independent diff scan covered the baseline-to-worktree source set and reported the five findings above; all were remediated before validation. | — | PASS with remediation; scan `e7269ef7-abf8-4782-bafd-81c0f100ac41`. Advisory-connector intelligence was unavailable and disclosed before the scan. |

## Phase 4 — Validate

- Focused tests run: `cargo check --workspace --all-targets`;
  `cargo clippy --workspace --all-targets --all-features -- -D warnings`;
  `scripts/test-provider-conformance.sh` (13 provider unit, 3 egress, 4
  protocol, 1 operator CLI, 1 separate-process TLS conformance, and 5
  PostgreSQL registry tests); `scripts/test-provider-authority-pilot.sh` (one
  clean-clone separate-process authority test plus provider backup/restore);
  and `scripts/dev.sh --smoke-test`. All final focused runs passed.
- First canonical gate run: `bin/gate.sh --diff` completed all stages but was
  red on two validation defects: warning-denied Clippy found one needless
  borrow, and the QML/API smoke still expected the pre-authority catalog field
  set. Both were corrected and their focused checks passed.
- Pre-completion canonical gate rerun: all 18 stages passed, including 44/44
  PostgreSQL server tests, live API/QML smoke, provider security conformance,
  and the clean-clone Door Legends authority/restore proof.
- Final canonical gate rerun after OpenWiki and archival: all 18 stages passed
  again and wrote receipt
  `18623d038d5d2293dc1da0838c258e27f59079cd45f166a03ed1861f18ee4a11`,
  matching the exact gated delivery state.
- Skips or pre-existing failures: ordinary workspace tests continue to mark
  database/separate-process cases ignored by design; the canonical non-fast
  gate ran them explicitly. Cargo package emits the pre-existing missing
  package-metadata warning for the provider crate; no license was invented.

## Phase 5 — Complete

- Acceptance-criteria audit: REQ-001 is satisfied by Constitution §10 and
  ADR-0002; REQ-002 by migration 0015 plus the null platform-state assertion;
  REQ-003 by start/command replay, expected-revision race, timeout-after-commit,
  restart, and reconcile evidence; REQ-004 by callback tamper, duplicate,
  policy-ignore, result/achievement, audit, and sync assertions; REQ-005 by
  outage, suspension, reactivation/reconciliation, retirement, and independent
  provider restore; REQ-006 by exact public response and foreign-participant
  privacy assertions; and REQ-007 by gate 18's packaged-protocol clean clone,
  separate TLS process/database, and restore drill. All seven pass.
- Docs: OpenWiki run `37920d9c-caa6-4cec-bef5-dbc465d6603a` completed after
  reconciling quickstart, cartridge, runtime, product-boundary, and validation
  pages. It retained four pre-existing unresolved evidence-debt warnings for
  existing claim sidecars; completion itself returned `status: complete`.
  After advancing the durable spec phase, follow-up run
  `16a04b23-bed0-4cfa-b378-c247839cbf8d` completed without warnings and wrote
  Ticket 019's matching completion receipt.
  Source/API/architecture/product/operator docs and the Door Legends runbook
  are updated as authoritative hand-maintained documentation.
- AAR: `AAR-019-first-party-remote-provider-migration-pilot.md` records the
  implementation, inspection failures, prevention rules, and scoped authority
  decision; every new ID is appended to the knowledge register.
- Archive: ticket, spec, and notes moved to their closed/completed locations.
  The phase marker had not been advanced
  from Inspect before the OpenWiki call; this repeated the known
  `PR-omarchy-gaming-system-advance-durable-phase-before-phase-tools-001`
  sequencing mistake, was corrected before archival, and does not alter the
  completed OpenWiki receipt.

## Defect and lesson ledger

| # | What happened | Root cause | Fix | Prevention |
|---|---|---|---|---|
| 1 | Exact callback replay could conflict with its original platform disposition. | Mutable current projection policy was allowed to influence an immutable receipt replay. | Durable authenticated identity now wins duplicate handling before any new disposition has an effect. | Preserve the first authenticated callback disposition on exact replay. |
| 2 | Invalid callbacks could consume shared quota. | Cost was charged at release lookup rather than after authentication. | Signature/body/context verification precedes quota admission and policy is rechecked afterward. | Charge shared authenticated quotas only after authentication. |
| 3 | Pilot suspension was not a universal provider gate. | The new lifecycle lived beside the general release policy instead of inside its admission material. | Lock and evaluate pilot status in every admission and again at projection. | Overlay narrow activation lifecycle at every work/effect boundary. |
| 4 | Callback and response lock orders differed. | Transaction ownership was designed per path instead of as one provider-root ordering. | Standardized release → pilot → session before receipts/effects. | Document and test one cross-domain lock order. |
| 5 | Clean-clone provider code could change without invalidating a receipt. | Gate path classification covered crates but not the independently compiled example tree. | Added the complete Door Legends example tree to gated state. | Gate every source tree that contributes executable delivery evidence. |
| 6 | The first full gate found Clippy and smoke drift. | A small borrow cleanup and additive catalog fields escaped focused functional tests. | Fixed the borrow and exact smoke JSON, then reran focused and full gates. | Treat warning-denied lint and public smoke allowlists as delivery contracts. |
| 7 | Phase state lagged the actual successful validation before OpenWiki. | The durable spec marker was not advanced immediately after the gate returned green. | Corrected the phase record before archival and retained the honest chronology here. | Apply the existing advance-phase-before-tool rule mechanically. |

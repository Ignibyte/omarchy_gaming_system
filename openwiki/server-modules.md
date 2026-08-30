---
type: "Reference"
title: "Server modules and typed hook boundary"
openwiki_generated: true
sources:
  - id: openwiki-source-d392f8f0962c50f0d66e0629
    resource: repo://client/qml/Main.qml
  - id: openwiki-source-f73ad44f40942d16dc369861
    resource: repo://client/qml/OnboardingController.qml
  - id: openwiki-source-7ea06d71b0299905dc0706ce
    resource: repo://client/qml/ServerProfiles.qml
  - id: openwiki-source-18d930eefcf679a5ad34b413
    resource: repo://crates/server-module-runtime/src/lib.rs
  - id: openwiki-source-ba203ea2e600f294ab58ef02
    resource: repo://crates/server/src/bin/omarchygs-admin.rs
  - id: openwiki-source-a13fe4db1eee073d0a7e2c4d
    resource: repo://crates/server/src/main.rs
  - id: openwiki-source-42fe6bf463fcb01dc5566e16
    resource: repo://crates/server/src/server_discovery.rs
  - id: openwiki-source-d0ad10f0eb7c1e026ba825e6
    resource: repo://crates/server/src/server_module_custom.rs
  - id: openwiki-source-37dbfbe1f44680fae1be88b6
    resource: repo://crates/server/src/server_module_tests.rs
  - id: openwiki-source-2d8ea93c101c36a0e0974581
    resource: repo://crates/server/src/server_modules.rs
  - id: openwiki-source-e9c32af872bdfcc1f392d212
    resource: repo://docs/architecture/server-modules.md
  - id: openwiki-source-6d761b854f0836930f612db4
    resource: repo://docs/operators/server-modules.md
  - id: openwiki-source-dc62400b0039f0daf5073bd4
    resource: repo://migrations/0026_server_module_observation_evidence.sql
  - id: openwiki-source-f4fac8f05085c377def6e545
    resource: repo://migrations/0027_operator_custom_server_modules.sql
  - id: openwiki-source-312e5b9c241219a3ef1b5c9e
    resource: repo://migrations/0028_packaged_reviewed_server_module_releases.sql
  - id: openwiki-source-8128bd5b86e858053bc20c68
    resource: repo://scripts/test-server-module-spike.sh
  - id: openwiki-source-5f564ae64057cbe621fc587a
    resource: repo://scripts/test-server-modules.sh
generated: {by: "codex", at: "2026-08-30T02:11:48.120Z"}
verified:
  - by: openwiki/0.3.3
    at: 2026-08-30T02:11:48.120Z
---

# Server modules and typed hook boundary

## Current status

ADR-0004 is implemented for a disabled-by-default two-release reviewed
first-party catalog and for up to eight explicitly admitted operator-custom
module identities. The packaged `ignibyte.sentinel` releases and
database-custodied custom releases use the same exact WIT, typed hook and
intent, core reauthorization, durable state and receipts, packaged host, and
OS-containment path. Ticket 039 remains the independent architecture proof,
Ticket 040 owns the production observation base, Ticket 041 owns bounded custom
custody and lifecycle, and Ticket 042 owns explicit reviewed release upgrade
and immediate rollback.

There is still no module discovery marketplace, remote or public
administration, operator-selected execution path, admission hook, network
egress, client executable delivery, or gameplay authority. Custom import is a
database-local operation over owner-private canonical files; placing a Wasm
artifact on disk cannot load it.

The extension families stay deliberately separate:

| Family | Supplied artifact | Authority |
|---|---|---|
| Game Cartridge | Signed inert schemas and assets | Presentation only through trusted OmarchyGS QML |
| Compiled game | Reviewed Rust linked into the platform | Exact-version platform game rules |
| Registered provider | Independent authenticated service | Sole remote rules and private-state authority for its pinned sessions |
| Server module | Process-isolated no-WASI component | Observe admitted hooks and propose typed effects |

A module cannot supply client QML, access a provider grant, implement a second
gameplay authority, or receive a database handle.

## Trust and runtime boundary

Release, provenance, server admission, and measured runtime containment are
independent claims. A publisher signs the exact component and contract. A
marketplace or server operator separately records provenance. Core admits exact
hooks, capabilities, budgets, configuration/state revisions, lifecycle, and
server identity. Operator-custom provenance must name that same admitted
server. The host then measures the actual containment used to execute it.

```text
core transaction + durable outbox event
                 ↓ bounded partitioned dispatch
exact release + admission + typed event
                 ↓ bounded local RPC
dedicated OS-contained module host
                 ↓ exact no-WASI WIT component
              typed intent
                 ↓
core reauthorization + domain transaction + immutable receipt
```

The component sees only bounded typed hook data, pairwise/public identifiers,
immutable configuration, and a state snapshot. It returns a no-op or typed
intent. Core rechecks the exact release, admission, capability, lifecycle,
target, policy, idempotency identity, and expected revision before any protected
effect commits.

The first production observation is emitted by report creation. The
authoritative report transaction deterministically considers the reviewed
fixture and every active subscribed custom instance, with at most eight custom
identities. For each eligible instance below its 1,024-row outstanding ceiling,
it appends one privacy-minimized `persona_reported` event. Exact report replay
returns before emission, so it cannot enqueue duplicates. Each event contains
the report UUID, bounded category, module-scoped pairwise subject, and exact
configuration/state snapshots; it excludes report detail, reporter/account
identity, credentials, arbitrary paths or URLs, and provider authority.

Observation is optional rather than admission. If an instance is inactive, its
queue is saturated, or runtime keys are absent, the report still commits and
the same transaction increments a saturating aggregate gap count with
`module_inactive`, `queue_saturated`, or `runtime_unconfigured` plus its
timestamp. After enqueue, timeout, trap, malformed output, crash, or retry
cannot roll back the completed report. A later admission hook would have to run
against an immutable snapshot outside a database transaction, then re-lock and
revalidate authoritative state before commit.

## State, lifecycle, and operations

Configuration and state remain in core-owned bounded namespaces with
compare-and-set revisions. Both reviewed and custom upgrade paths use a
complete isolated candidate namespace, contained readiness, quota/schema
validation, and one retained immediate-predecessor snapshot. Legacy reviewed
state maintenance now locks and compares instance, namespace, and selected
release schemas, labels snapshots with the live schema, and rejects a snapshot
from another schema. Backups include manifests, admissions, audit, outbox,
namespaced state, aggregate gap evidence, and immutable delivery receipts. New
receipts retain the bounded attempt-normalized canonical request, exact
response, their digests, and target report even after delivered outbox pruning.
Upgrade-era rows that predate migration 0026 remain explicitly identifiable as
incomplete rather than receiving fabricated evidence.

The database-local `omarchygs-admin` process implements bounded inventory,
reviewed-fixture lifecycle and restore, the separate
`reviewed-module-apply` upgrade/rollback command, plus custom import and exact
enable/disable/suspend/recover/upgrade/rollback/remove commands. Reviewed and
custom transition commands carry lifecycle, configuration, and state
revisions. Upgrade proves a complete bounded candidate namespace and retains
only the immediate predecessor snapshot; rollback consumes that predecessor
once. Custom removal is terminal but preserves artifact, provenance, state,
receipt, and audit evidence.

The packaged reviewed catalog fixes Sentinel `1.0.0` to `state/v1` and
Sentinel `1.1.0` to `state/v2`. Startup registers and byte-compares both exact
release/provenance envelopes but selects `1.0.0` only for a new instance; it
never upgrades an existing selection. `reviewed-module-apply` permits only the
fixed `1.0.0 → 1.1.0` edge with a complete candidate state or the one-use
rollback to its retained predecessor. Readiness runs outside SQL. Finalization
takes the shared registry lock, rechecks release, lifecycle, configuration,
state, both schema roots, activation, restore, and predecessor inputs, then
atomically publishes the fresh admission, namespace, release selection,
stale-work evidence, audits, and immutable operation receipt. Migration 0028
also constrains the retained receipt to those exact two edges and rejects row
mutation or truncation.

Command and referenced artifact paths are absolute owner-private mode-0600
regular files. The custom reader uses Linux `openat2` to reject symlinked
ancestors and magic links, then enforces final no-follow open, single-link
ownership/mode, byte ceilings, and stable descriptor metadata before and after
the read. Every custom mutation has a whole-command UUID and digest, actor, reason,
and immutable operation receipt; a UUID replays only the same canonical body.

PostgreSQL cannot infer that a raw archive has been restored. The operator must
therefore run the audited `module-restore` command before any restored server
startup. It leaves retired instances terminal, disables every other module,
clears stale leases and admission selection, blocks activation, and requires
explicit review plus recovery before fresh readiness. Custom recovery repeats
contained readiness and publishes a fresh exact admission. A configured server
still starts while persisted module policy denies activation.

The two module runtime secrets are all-or-none and enable the generic
dispatcher; selecting the reviewed catalog is independent. With both secrets
absent—or when the configured worker cannot reproduce the exact selected
packaged release—core starts with an unconfigured emitter and records bounded
`runtime_unconfigured` gaps without claiming that module behavior ran. It does
not substitute `1.0.0` for a selected `1.1.0` release.

## Player disclosure and support

When at least one custom module is active or degraded, public discovery adds
`server.operator-custom-modules.v1` and one aggregate bound to the stable server
UUID. It contains only a count from 1 through 8, the bounded public behavior
class `moderation_labels`, the fixed unreviewed-code warning, and the operator
support boundary. It contains no module/release identity, component, path,
configuration, state, operator identity, private inventory, or signing
authority.

The onboarding controller and saved-profile store exact-validate and
server-identity-bind that aggregate. The trusted shell renders it continuously
as plain-text accessible warning chrome before and after sign-in, including at
the 640x420 minimum window. It provides no acknowledgement bypass or
administration action.

Operator-custom provenance changes claims and support responsibility, not
runtime power. The server owner is responsible for the custom code's security,
privacy, availability, moderation, terms, telemetry, patching, incident
response, backup/recovery, and support. OmarchyGS neither reviews nor supports
the bypassed component merely because the shared host contains it.

## Proof

`scripts/test-server-module-spike.sh` retains the independent exact-WIT hostile
proof. `scripts/test-server-modules.sh` is the production entrypoint: it runs
runtime tests and rustdoc, executes the packaged host under real systemd-user,
Bubblewrap, prlimit, Wasmtime memory/fuel, and outer-time containment, and
asserts the fixed sibling loader, private bounded custom custody, absent public
module routes, and absent network/database client dependencies.

The migrated PostgreSQL corpus proves atomic private emission, ordering,
receipt replay and retention, fail-open gap accounting, bounded failure and
circuit behavior, readiness configuration/state races, lifecycle/state CAS,
restore, and legacy receipt semantics. Three reviewed-release cases prove exact
upgrade/dispatch/rollback, replay and atomic failure, missing or changed package
behavior, restore, and schema-aware maintenance. Five custom-module cases prove
import, immutable evidence, the eight-identity ceiling, lifecycle races,
upgrade/rollback/removal, restore review, shared dispatch, and current-admission
reauthorization. The real CLI suite proves private command custody, exact
reviewed upgrade/rollback receipts, custom import replay, and contained enable.
Gate 21 proves pre-start restore reconciliation with module configuration still
present; gate 24 runs the production conformance script for both packaged
reviewed releases. Additional hook classes, marketplace admission, and remote
administration remain separately reviewed work.

## Change map

| Intent | Read first | Narrow evidence |
|---|---|---|
| Change release, WIT, signatures, admission, framing, host, or limits | ADR-0004, `docs/architecture/server-modules.md`, and `crates/server-module-runtime` | `scripts/test-server-modules.sh` plus deterministic and contained-host conformance |
| Change report observation, dispatch, receipts, gaps, intents, state, lifecycle, or restore | `crates/server/src/server_modules.rs`, `reports.rs`, migrations `0025`–`0027`, and `docs/operators/server-modules.md` | Focused ignored server-module and operator-CLI tests, gate 21 recovery drill, then gate 24 |
| Change the packaged reviewed catalog, upgrade/rollback edge, schema migration, or exact startup selection | `crates/server-module-runtime/src/lib.rs`, `crates/server/src/server_modules.rs`, migration `0028`, and the operator runbook | Reviewed-release runtime/PostgreSQL/CLI cases plus `scripts/test-server-modules.sh` and gate 24 |
| Change custom import, immutable custody, provenance, upgrade/rollback/removal, or local command handling | `crates/server/src/server_module_custom.rs`, `bin/omarchygs-admin.rs`, migration `0027`, and the operator runbook | Custom-module unit/PostgreSQL/CLI hostile corpus plus shared runtime and gate 24 |
| Change active custom disclosure or warning persistence | `server_discovery.rs`, `OnboardingController.qml`, `ServerProfiles.qml`, and `Main.qml` | Discovery API privacy cases plus QML transport/profile/accessibility fixtures and live smoke |
| Change the independent architecture proof | `crates/server-module-spike` | `scripts/test-server-module-spike.sh` and gate 23 |
| Add a marketplace module path or another hook/intent | Constitution extension boundaries | New ticketed threat/authority review, CodeGraph inspection, database/process evidence, and canonical local diff gate |

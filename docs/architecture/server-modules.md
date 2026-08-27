# OmarchyGS server modules

Status: architecture and isolated proof accepted by ADR-0004. Production module
loading, installation, persistence, routes, discovery, and administration are
not implemented or authorized.

## Purpose and extension families

Server modules are future operator-admitted executable extensions for general
community behavior such as moderation annotations or bounded integrations.
They do not provide a game's frontend or own a game's rules.

| Family | Artifact/runtime | Authority |
|---|---|---|
| Game Cartridge | Signed inert schemas/assets rendered by platform QML | Presentation only; no executable/network authority |
| Compiled game | Reviewed Rust linked into OmarchyGS | Platform-authoritative rules for exact versions |
| Registered provider | Independent authenticated service | Sole rules/private-state authority for one pinned provider session |
| Server module | Process-isolated no-WASI Wasm component | Observes admitted hooks and proposes typed effects that core reauthorizes |

A convenient module hook must never invoke a transport handler, receive a
database handle, rewrite a provider receipt, implement `GameDefinition`, or
become a direct client-code/download channel.

## Trust and admission documents

The contract keeps these independently verifiable:

1. `omarchygs.server-module-release/v1`: publisher-signed canonical release
   manifest binding module/publisher/release/version, exact component digest,
   WIT identity, requested hooks/capabilities, budgets, configuration/state
   schemas, and entrypoint.
2. `omarchygs.server-module-provenance/v1`: separately signed marketplace
   review or server-operator custom trust bound to the release-manifest digest;
   operator-custom trust is also bound to the exact admitted server identity.
3. `omarchygs.server-module-admission/v1`: core-owned exact server grant binding
   release, component, WIT, provenance, granted subsets, budgets, state/config
   revisions, and active lifecycle revision.
4. Runtime containment: measured host/service state. It is not asserted by any
   signature.

Every signed payload uses strict canonical JSON, unknown-field rejection,
domain-separated Ed25519 signatures, canonical unpadded base64url, lowercase
SHA-256, and bounded input before decoding. Requested power is never granted
implicitly; admission hooks and capabilities must be sorted unique subsets.

## Runtime boundary

```text
authoritative domain transaction
  └─ mutation + immutable module outbox event (same PostgreSQL commit)
                         ↓
bounded partitioned dispatcher
  └─ exact release/admission + typed event snapshot
                         ↓ canonical bounded local RPC
dedicated OS-contained omarchygs-module-host
  └─ pinned Wasmtime, no WASI/imports, fresh limited Store/instance
                         ↓ exact WIT record
                    component handle
                         ↑ typed intent/no-op
host validates declared grant and response bounds
                         ↑ canonical bounded local RPC
core validates current lifecycle/capability/target/revision/policy
  └─ authoritative effect + immutable receipt (one transaction)
```

The spike uses a four-byte big-endian frame length followed by canonical JSON,
with a 64 KiB ceiling rejected before payload allocation. Production may use a
different local transport only if it preserves the same exact authenticated
context, framing bounds, deadlines, and stable errors.

Component files are opened once and read through a `MAX+1` bounded reader
before compilation or signature work. A path metadata pre-check followed by an
unbounded read is not an artifact ceiling because the backing file can change.

The proof WIT is
`ignibyte:omarchygs-server-module@1.0.0/module-proof`. OmarchyGS admits one
supported major plus the exact checked-in WIT SHA-256. Compatible additions
use new named hooks, intents, or capabilities; incompatible field/semantic
changes require a new major. Current Component Model tooling remains an
explicitly pinned dependency rather than a permissive negotiation surface.

## Hook data and privacy

A future `ModuleHookEventV1` binds:

- server, module, exact release, and admission identities;
- core-owned event UUID, delivery attempt, hook kind/version, causal revision,
  and deadline budget;
- an opaque pairwise subject or explicitly public domain identifier;
- bounded immutable configuration and module-state snapshots plus revisions;
  and
- one allowlisted typed payload.

It must not include account ownership, a username mapping, password/MFA
material, bearer/device token, database row/handle/credential, raw private
message unless a separately designed content hook explicitly grants it,
arbitrary URL/path, provider grant, or client executable content.

The proof's `persona-reported` observation contains a report UUID, bounded
category, and pairwise persona subject; it does not expose reporter account
ownership or free-form report detail.

## Typed intents and core authorization

Components have no direct mutation hostcall. Configuration and state are input
snapshots. Output is no-op or a bounded named intent such as
`moderation-add-label` with an expected target revision. An integration intent
would select a preconfigured core-owned destination slot, never supply a URL.

For each ordinal in a response, core derives the receipt identity from exact
release/event/ordinal and rechecks:

- response event/release/admission context;
- active lifecycle and current exact admission;
- named capability and hook relationship;
- target type and pairwise/public subject binding;
- current expected revision and idempotency identity;
- operator/platform policy and argument bounds; and
- domain authorization at the eventual transaction.

Only a core service commits protected state. Identical delivery returns the
original receipt; the same event identity with a different body conflicts.

## Ordering, failures, and backpressure

Delivery is at least once. The dispatcher partitions by exact module release,
hook, and subject and preserves event order within a partition. Independent
partitions may execute concurrently. Memory queues and frame/payload sizes are
bounded, and empty partition state is removed after delivery; saturation
remains durable backpressure, not unbounded process memory.

Observation hooks are post-commit and fail-open with respect to the completed
platform mutation. Timeout, trap, malformed response, unavailable host, or
process exit records a stable failure and follows bounded retry/backoff and
dead-letter policy. Repeated faults trip a core-owned circuit breaker to
`degraded`; fresh deliveries pause while durable work remains.

Any future admission hook is a different class. It evaluates an immutable
pre-commit snapshot outside database transactions, has an operator-selected
required/optional failure policy recorded in admission, and must re-lock and
revalidate current domain state before commit. Component output cannot choose
fail-open. Production should ship observation hooks before this class.

## Configuration and state

Module configuration and state are core-owned namespaces keyed by stable
server/module/schema identity. Names, values, entry counts, and total bytes are
bounded. Writes are typed compare-and-set intents using an exact revision;
modules never receive OmarchyGS SQL or credentials.

Upgrade creates an isolated candidate namespace, copies the current snapshot,
applies explicit forward migrations, validates quota/schema, and proves host
readiness before admission changes atomically. The prior exact release and
pre-upgrade namespace snapshot are retained for rollback. A failed migration
does not mutate the live namespace. Uninstall retains audit/tombstone and
requires an explicit state-disposition decision.

Backups include manifests, provenance, admissions, audit, outbox/receipts, and
namespaced state. They exclude process/JIT images and untrusted native caches.
Restore starts every module disabled until exact artifacts, WIT/host
compatibility, admission, state, and pending receipts have been reverified.

## Lifecycle and operations

The state machine is:

```text
staged → disabled → enabling → active → degraded/suspended
             ↑          │          │              │
             └──────────┴──────────┴──────────────┘
disabled → retired (terminal)
```

Install, enable, disable, upgrade, rollback, suspend, and remove are future
database-local administrator operations with operation UUID, expected state
and revision, actor, bounded reason, exact release/provenance/capability review,
and an immutable same-transaction audit record. Disable/suspend stops new work
before process termination. Recovery re-verifies bytes and reconciles receipts
before returning active.

Marketplace-vetted and operator-custom modules execute under the same WIT,
conformance, capability, and sandbox rules. Custom provenance requires explicit
operator acknowledgement and permanent player-facing custom-server disclosure
where behavior affects players. OmarchyGS does not certify or support an
owner's bypassed executable code merely because the host contains it.

## Host policy

The production service profile must provide at least:

- one exact release and service identity per process;
- read-only component, system libraries, and host binary;
- private/absent home, devices, temporary storage, environment, and network;
- no database socket or credential and no inherited server secret;
- no guest import, including WASI clock/random/filesystem/socket/process;
- `NoNewPrivileges`, no capabilities, bounded cgroup memory/CPU/tasks, bounded
  file descriptors, and an outer kill deadline;
- fresh limited Store/instance per event with memory/fuel ceilings; and
- structured health, trap, timeout, retry, circuit-breaker, queue, and resource
  telemetry that never logs private payloads or credentials.

The proof applies Bubblewrap `--unshare-all`, capability removal, cleared guest
environment, read-only exact binds, no home or `/etc/passwd`, a private network
namespace, systemd user-scope memory/CPU/task ceilings, inherited `prlimit`
file-descriptor ceilings, Wasmtime memory/fuel limits, and a parent-owned 500 ms
execution deadline. These are measured proof ceilings, not final defaults.

## Conformance and implementation sequence

`scripts/test-server-module-spike.sh` deterministically componentizes inert
fixture modules from the exact WIT, builds them twice, and exercises:

- strict signing/canonicalization, separate provenance classes, exact WIT and
  digest binding, unknown/duplicate/downgrade rejection, bounded framing, and
  sensitive-field absence;
- valid typed event to core-authorized intent, no-op, exact/changed replay,
  queue ordering/backpressure, state CAS/quota/backup/restore, atomic
  migration/rollback, and lifecycle idempotency/retirement;
- undeclared capability, component/signature/context tamper, wrong interface,
  forbidden import, excessive memory, trap, infinite loop/fuel exhaustion,
  process exit, outer timeout, and clean restart; and
- sandbox readiness, host RSS ceiling, deterministic artifacts, no production
  loader/configuration, and local-only quality automation enforcement.

The next production ticket may implement only the versioned module registry,
observation outbox/dispatcher, one safe typed observation hook/intent, state and
lifecycle base, and conformance tooling. Administrator custom installation and
additional hooks remain separately reviewed follow-up work.

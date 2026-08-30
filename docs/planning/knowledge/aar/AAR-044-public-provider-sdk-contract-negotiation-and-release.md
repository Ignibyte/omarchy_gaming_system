---
aar: AAR-044-public-provider-sdk-contract-negotiation-and-release
ticket: TICKET-044
pipeline: public-provider-sdk-contract-negotiation-and-release
status: submitted
opened: 2026-08-30
submitted: 2026-08-30
effectiveness: effective
---

# AAR-044-public-provider-sdk-contract-negotiation-and-release

## Recalled at plan

| ID or source | How surfaced | Useful? |
|---|---|---|
| Tickets 017–019 notes and architecture | Provider SDK, clean-clone, and remote-provider knowledge search | Yes — supplied the existing contract, release-proof pattern, and authority boundary. |
| Provider binding, replay, callback, quota, lifecycle, lock-order, and clean-source prevention rules | Knowledge register search | Yes — constrained the negotiation and migration design before code. |
| Public Provider SDK intake and ADR-0003 | Roadmap promotion review | Yes — isolated the first shippable slice and preserved onboarding/sidecar exclusions. |

## What happened

Ticket 044 extracted the provider-facing protocol from the platform crate into
the independently packageable `omarchygs-provider-sdk` crate, while preserving
one internal implementation through source-compatible re-exports. Protocol v1
now authenticates one exact compatibility profile before any provider effect
and binds that selection through grants, requests, responses, and callbacks.
The broker performs final locked trust admission, uses one aggregate deadline,
and leaves Door Legends as the sole admitted provider.

The deterministic preview release exports a finite public-only inventory with
schemas, fixtures, documentation, a canonical lock, and signed provenance. A
focused local stage packages only that crate and proves byte-identical exports
from two clean Git clones without an OmarchyGS path dependency. The existing
Door Legends pilot consumes the package from a clean clone and retains its
separate persistence, callback, restart, and recovery behavior.

Inspection found four lower-severity boundary defects and two medium replay
compatibility defects. All six were fixed before validation. In particular,
the schema upgrade preserves historical durable intent preimages and permits a
legacy callback only after current-key authentication and exact immutable
local duplicate resolution; fresh legacy network traffic still fails closed.
OpenWiki reconciled the durable docs, and every local gate stage passed. No
hosted workflow, public registration route, registry publication, or external
provider authority was introduced.

## Failures captured

| ID | Failure | Where surfaced |
|---|---|---|
| `BF-omarchy-gaming-system-provider-compatibility-stale-trust-snapshot-001` | A compatibility response could authenticate against a registry snapshot that changed before grant issuance or operation transport. | Security inspection of the preflight-to-attempt flow. |
| `BF-omarchy-gaming-system-provider-two-post-lease-undercoverage-001` | Compatibility and operation POSTs each received a full timeout while one concurrency lease covered only one timeout window. | Security inspection of broker deadlines and lease duration. |
| `BF-omarchy-gaming-system-provider-sdk-unbounded-inventory-walk-001` | SDK verification traversed directory breadth before enforcing the signed exact inventory. | Security inspection of release verification. |
| `BF-omarchy-gaming-system-provider-sdk-path-separator-alias-001` | A literal Unix backslash filename could alias a slash-separated signed inventory identity. | Hostile release-verification review. |
| `BF-omarchy-gaming-system-provider-schema-upgrade-durable-replay-drift-001` | Adding negotiated compatibility to historical durable intent bytes changed receipt digests and would deny exact retry recovery. | Independent post-patch replay review. |
| `BF-omarchy-gaming-system-provider-legacy-callback-lost-ack-denial-001` | Strict mandatory compatibility rejected an exact historical callback replay before duplicate resolution after a lost acknowledgement. | Independent callback idempotency review. |

## Prevention rules captured

| ID | Rule | Why |
|---|---|---|
| `PR-omarchy-gaming-system-finalize-provider-effects-from-current-locked-trust-001` | After out-of-transaction compatibility work, re-admit the provider and release under their authority locks and use that exact material for grant, transport, and verification. | An authenticated preflight cannot authorize a later effect after mutable trust changes. |
| `PR-omarchy-gaming-system-budget-provider-preflight-and-operation-together-001` | Compatibility, grant preparation, and operation transport must share one aggregate deadline covered by the acquired concurrency lease. | Sequential individually bounded calls can otherwise exceed the resource reservation. |
| `PR-omarchy-gaming-system-bound-native-signed-artifact-inventory-001` | Enforce finite native path, entry, depth, and aggregate-name bounds while traversing a signed artifact; never normalize platform separators into its identity. | Exact signed inventories must reject breadth exhaustion and path aliases before comparison. |
| `PR-omarchy-gaming-system-preserve-durable-wire-preimages-across-upgrades-001` | Treat already-persisted canonical request and receipt bytes as immutable across protocol-schema upgrades, and upgrade only explicitly authenticated local representations. | Re-encoding historical intent breaks semantic replay and recovery identity. |
| `PR-omarchy-gaming-system-admit-legacy-provider-messages-as-local-duplicates-only-001` | A legacy provider message may bypass a new field only after current-key authentication and exact immutable local receipt matching, and may resolve only as a duplicate. | This preserves lost-ack recovery without making the legacy schema a fresh network input. |

## Architecture decisions

| ID | Decision | ADR |
|---|---|---|
| `AD-omarchy-gaming-system-public-provider-sdk-without-admission-authority-001` | A reproducible public-only Provider SDK preview owns provider-facing types and verification but grants no registration, activation, discovery, database, client-network, or publication authority; platform admission remains separate. | `docs/architecture/adr-0003-owner-operated-server-and-extension-boundary.md`; `docs/architecture/game-cartridges.md` |

Every new ID above must also be appended to
`docs/planning/knowledge/INDEX.md`.

## Effectiveness

Effective (5/5). Earlier provider binding, replay, clean-source, and authority
rules prevented the extraction from widening the platform boundary. The
specialized security pass and independent replay reviews caught trust
freshness, resource budgeting, artifact traversal, path identity, durable
preimage, and lost-ack edge cases before delivery. Focused tests then exercised
each repair through the public package and real broker/provider path, while the
complete local gate and matching OpenWiki receipt closed the implementation,
documentation, and delivery-evidence loops without adding hosted automation.

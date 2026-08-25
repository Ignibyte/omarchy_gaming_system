---
aar: AAR-018-production-remote-provider-security-foundation
ticket: TICKET-018
pipeline: production-remote-provider-security-foundation
status: submitted
opened: 2026-08-25
submitted: 2026-08-25
effectiveness: 5
---

# AAR-018-production-remote-provider-security-foundation

## Recalled at plan

| ID or source | How surfaced | Useful? |
|---|---|---|
| `AD-omarchy-gaming-system-game-cartridge-provider-boundary-001` | Knowledge register, ADR-0002, Game Cartridges architecture, and Ticket 014 notes | Yes — fixes the broker, authority, identity, and presentation boundaries while preserving compiled authority today. |
| `PR-omarchy-gaming-system-bind-provider-messages-to-registered-identity-001` | Knowledge register and Ticket 014 inspection | Yes — every grant and message must bind the complete registered context, not only a valid signature. |
| `PR-omarchy-gaming-system-enforce-bounds-during-streaming-001` | Knowledge register and Ticket 014 broker finding | Yes — hostile provider bodies must be stopped before full buffering. |
| `PR-omarchy-gaming-system-resolve-durable-replay-before-current-admission-001` | Knowledge register and Ticket 020 AAR | Yes — authenticated retries need durable receipt semantics that survive mutable policy while never bypassing identity or revocation safety. |
| Ticket 014 proof limitations | Completed pipeline notes and ADR consequences | Yes — production cannot reuse ephemeral keys, plain loopback HTTP, or in-memory replay state as security evidence. |
| Constitution §§10, 14, 15, and 18 | Workflow preflight | Yes — provider plumbing must stay dormant, fail closed, retain exact versions, use real evidence, and receive independent security/data review. |

## What happened

Ticket 018 built the production security and control-plane foundation for a
future registered remote game provider without changing current gameplay
authority. The new dormant `omarchy-game-provider` crate owns immutable
operator-pinned provider releases, lifecycle and key rotation, pairwise
60-second grants, fixed signed HTTP messages, public-only pinned HTTPS egress,
durable replay and callback receipts, quotas, concurrency leases, and safe
audit records. Migration 0014 persists those controls, and an operator-only CLI
provides bounded registration and lifecycle changes.

The conformance environment launches a separate TLS provider process and uses
migrated PostgreSQL to prove exact replay, changed-intent and revision
conflicts, commit-then-timeout recovery, callback deduplication, outage,
reconciliation, lifecycle/key denial, quota and lease races, redirect/body/
signature failures, and registered trust roots. Gate 17 now runs that corpus in
every diff/full delivery loop. The player server does not depend on or
instantiate the crate, so Constitution §10 authority remains unchanged.

Independent inspection found two medium defects before activation. Callback
deduplication attempted to lock a row that might not exist, so simultaneous
first delivery could expose a uniqueness conflict instead of a duplicate
receipt. The fix serializes on the guaranteed release root and proves one
accepted plus one duplicate result concurrently. The IPv6 egress classifier
also omitted local-use NAT64 and reserved allocation space; it now starts from
the allocated global-unicast block and rejects current special-purpose ranges.
A fresh complete Codex Security scan covered all 13 changed source surfaces
with zero findings. CodeGraph inspection, OpenWiki reconciliation, the focused
provider suite, and the final 17-stage canonical gate all passed.

## Failures captured

| ID | Failure | Where surfaced |
|---|---|---|
| `BF-omarchy-gaming-system-provider-callback-absent-row-race-001` | Concurrent first delivery of one authenticated callback could race on an absent receipt and return a uniqueness conflict instead of durable duplicate success. | Phase 3.5 concurrency inspection and pre-fix Codex Security scan |
| `BF-omarchy-gaming-system-provider-ipv6-special-use-egress-gap-001` | A denylist-based IPv6 classifier admitted local-use NAT64 and reserved space that was not safe for production provider egress. | Phase 3.5 SSRF inspection and pre-fix Codex Security scan |

## Prevention rules captured

| ID | Rule | Why |
|---|---|---|
| `PR-omarchy-gaming-system-serialize-dedupe-on-existing-root-001` | When a deduplication receipt may not exist, lock a guaranteed durable domain root before the first-read/insert decision and prove simultaneous first delivery. | Row locks cannot serialize absence; a stable parent lock makes first acceptance and exact duplicate disposition deterministic across processes. |
| `PR-omarchy-gaming-system-classify-provider-egress-by-global-allocation-001` | Classify provider destinations from the positively allocated global address space, then exclude current special-purpose ranges and test translation/reserved prefixes. | An ad hoc private/special denylist inevitably misses newly assigned or obscure non-public ranges at an SSRF boundary. |

## Architecture decisions

| ID | Decision | ADR |
|---|---|---|
| `AD-omarchy-gaming-system-remote-provider-security-foundation-001` | Remote-provider plumbing is a dormant OmarchyGS-only broker boundary: immutable operator-pinned releases, pairwise one-scope grants, fixed signed messages, public-only pinned HTTPS with registered roots, and durable PostgreSQL replay/quota/lease/audit state; player authority remains compiled until a separately authorized migration. | `docs/architecture/game-cartridges.md` |

Every new ID above must also be appended to
`docs/planning/knowledge/INDEX.md`.

## Effectiveness

5/5. All five Ticket 018 requirements have focused hostile-input, real
PostgreSQL race, operator CLI, and separate-process TLS evidence, and the final
canonical gate repeated the complete platform regression suite. The dormant
crate boundary avoided a partial authority migration while leaving concrete
production plumbing for Ticket 019. Independent security inspection found and
closed two issues before any player-facing reachability existed; the post-fix
scan had zero findings. OpenWiki completed without warnings, and its completion
receipt, final gate receipt, and current gated state all match
`acbbb4f207642e022848a1fe4fb9ba943fc8a5ece4a7033b0b09bed2332f8f55`.

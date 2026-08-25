---
aar: AAR-015-game-cartridge-contract-verifier-and-conformance-cli
ticket: TICKET-015
pipeline: game-cartridge-contract-verifier-and-conformance-cli
status: submitted
opened: 2026-08-25
submitted: 2026-08-25
effectiveness: 5
---

# AAR-015-game-cartridge-contract-verifier-and-conformance-cli

## Recalled at plan

| ID or source | How surfaced | Useful? |
|---|---|---|
| `AD-omarchy-gaming-system-game-cartridge-provider-boundary-001` | Knowledge register, ADR-0002, and OpenWiki | Yes — local data-only packaging comes before trusted rendering and any provider authority. |
| `PR-omarchy-gaming-system-parse-the-bytes-that-were-authenticated-001` | Ticket 014 AAR | Yes — the verifier must parse retained verified entry bytes, never reopen archive paths. |
| `PR-omarchy-gaming-system-enforce-bounds-during-streaming-001` | Ticket 014 AAR | Yes — archive and entry readers stop at their limits before allocating or decoding complete hostile input. |
| `PR-omarchy-gaming-system-bound-package-traversal-work-001` | Ticket 014 AAR | Yes — entry count, path depth, and directory shape are independent budgets. |
| `PR-omarchy-gaming-system-gate-every-nested-workspace-proof-001` | Ticket 014 AAR | Yes — the production cartridge crate and CLI must become canonical gate evidence. |
| `BUL-001-initial-push-pending` | Bulletin preflight | Yes — delivery remains local and unauthorized for Git publication. |

## What happened

Ticket 015 promoted the Ticket 014 data-only concept into a production Rust
library and `omarchygs-cartridge` CLI. V1 deterministically emits a canonical
stored-only ZIP, signs an exact integrity index with domain-separated Ed25519,
strictly verifies archive and authenticated payload bytes, reconstructs the
canonical container, and separates artifact validity from host compatibility.
It supports bounded Terminal/Grid/Status presentation data, a restricted local
JSON Schema profile, localization, strict 8-bit PNG, and PCM WAV.

The local lifecycle stores exact archives as read-only content-addressed blobs
and uses atomic activation/revocation records. It is deliberately a same-user
boundary: no package content is extracted or executed, and no server route,
PostgreSQL migration, network client, QML renderer, or platform credential was
added. Ticket 017 now carries descriptor-relative containment as a prerequisite
for any privileged or multi-user importer.

Inspection and Codex Security scan
`c83513cf-7de0-4552-8543-354b7aee4b4b` surfaced six candidates. Four concrete
pre-renderer defects were repaired: decoded PNG accounting admitted 16-bit
data, presentation nodes were not bound to required capabilities, path reads
could outgrow a pre-read metadata check, and revocation lookup errors failed
open. Two pathname ancestor races remain outside the explicitly supported
same-user store and are carried forward to Ticket 017. The final scan reported
no currently exploitable vulnerability because no production renderer/decoder
or privileged/shared store is deployed.

The focused corpus grew to nineteen tests. The final 14-gate run covered that
corpus, deterministic and isolated CLI behavior, the retained Ticket 014
provider/QML proof, 33 PostgreSQL tests, and the live PostgreSQL → Rust API →
QML smoke path. OpenWiki update run
`5fc51913-7de7-4df0-95cf-83ff7c0d68d1` completed and now distinguishes the
implemented inert contract from Ticket 016 rendering and later provider work.

## Failures captured

| ID | Failure | Where surfaced |
|---|---|---|
| `BF-omarchy-gaming-system-png-decoded-profile-underbound-001` | PNG validation accounted normalized RGBA bytes but did not constrain bit depth or compressed ancillary chunks, so a validly signed 16-bit image could understate future decoder work. | Codex Security media/resource validation. |
| `BF-omarchy-gaming-system-presentation-capability-confusion-001` | Grid, Status, or Terminal nodes could be present without the corresponding required host capability. | Codex Security compatibility validation. |
| `BF-omarchy-gaming-system-path-read-after-metadata-bound-gap-001` | Path APIs checked metadata and then used an unbounded reopen/read, allowing a FIFO or replaced file to cross the intended byte limit before rejection. | Dynamic FIFO validation during Phase 3.5. |
| `BF-omarchy-gaming-system-revocation-lookup-fail-open-001` | `Path::exists` collapsed revocation lookup errors into “not revoked.” | Dynamic external-harness validation during Phase 3.5. |
| `BF-omarchy-gaming-system-pathname-store-containment-boundary-001` | Atomic same-directory rename did not eliminate ancestor pathname replacement if a future privileged installer wrote below an attacker-mutable root. | Static store attack-path review; 20,000 attempts did not reproduce an escape. |

## Prevention rules captured

| ID | Rule | Why |
|---|---|---|
| `PR-omarchy-gaming-system-validate-decoder-profile-not-headers-001` | Before accepting an asset, validate the exact decoder profile and every source of decoded work, not only dimensions and magic bytes. | Header dimensions alone do not account bit depth, compressed metadata, or unsupported decoder paths. |
| `PR-omarchy-gaming-system-bind-presentation-nodes-to-capabilities-001` | Bind every presentation node and effect to the exact required host capability before compatibility evaluation. | A manifest capability list is ineffective when content can silently use undeclared vocabulary. |
| `PR-omarchy-gaming-system-read-bounded-input-from-checked-handle-001` | Read untrusted filesystem input through the same checked regular-file handle with an enforced streaming byte ceiling. | Path metadata followed by an unbounded reopen leaves type, race, and memory seams. |
| `PR-omarchy-gaming-system-distinguish-not-found-from-denial-001` | Treat only an explicit `NotFound` as absence; propagate or deny every other lookup failure at an authorization or revocation boundary. | Convenience existence checks can convert permission and I/O failures into authorization success. |
| `PR-omarchy-gaming-system-require-descriptor-relative-privileged-store-001` | Before a cartridge store crosses a user or privilege boundary, use descriptor-relative containment or an equivalent OS sandbox plus authoritative revocation. | Pathname validation and atomic rename do not secure attacker-mutable ancestors. |

## Architecture decisions

| ID | Decision | ADR |
|---|---|---|
| `AD-omarchy-gaming-system-canonical-game-cartridge-v1-001` | Adopt a canonical stored-only `.ogsc` v1 with an Ed25519-signed exact integrity index, strict data-only payload vocabulary, explicit capability compatibility, and no publisher execution. | `../../../architecture/game-cartridges.md`; `../../../../openwiki/game-cartridges.md` |
| `AD-omarchy-gaming-system-same-user-cartridge-store-001` | Limit the v1 filesystem lifecycle to a bounded same-user local store; defer privileged/multi-user containment to Ticket 017. | `../../../architecture/game-cartridges.md`; `../../../../openwiki/game-cartridges.md` |

Every new ID above must also be appended to
`docs/planning/knowledge/INDEX.md`.

## Effectiveness

5/5. The pipeline delivered all five EARS requirements with nineteen focused
tests, a formal security scan, fresh CodeGraph design/inspection evidence, a
green 14-gate vertical slice, and completed OpenWiki reconciliation. It also
turned the retro cartridge metaphor and graphics ceiling into a concrete
delivery sequence without claiming a renderer or remote provider exists.

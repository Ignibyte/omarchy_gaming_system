---
aar: AAR-014-portable-games-sdk-and-remote-hosting-spike
ticket: TICKET-014
pipeline: portable-games-sdk-and-remote-hosting-spike
status: submitted
opened: 2026-08-24
submitted: 2026-08-25
effectiveness: 5
---

# AAR-014-portable-games-sdk-and-remote-hosting-spike

## Recalled at plan

| ID or source | How surfaced | Useful? |
|---|---|---|
| Constitution §10 and §14 | Binding architecture review | Yes — current gameplay authority is local, deterministic, and revision-aware; remote authority requires an explicit later change. |
| `AD-omarchy-gaming-system-version-pinned-game-session-foundation-001` | Knowledge-register and Ticket 012 recall | Yes — portable providers still need immutable game/version identity and must not relabel durable sessions. |
| `AD-omarchy-gaming-system-idempotent-revision-command-boundary-001` | Knowledge-register and Ticket 013 recall | Yes — retries, optimistic concurrency, and durable notification recovery remain required even if authority moves across a network. |
| `AD-omarchy-gaming-system-public-persona-boundary-001` | Knowledge-register and system-overview recall | Yes — game integrations expose personas, never private account ownership. |
| `AD-omarchy-gaming-system-persona-sync-boundary-001` | Knowledge-register and OpenWiki recall | Yes — WebSockets remain advisory and durable recovery must survive disconnects and provider outages. |
| `PR-omarchy-gaming-system-pin-durable-game-rules-version-001` | Knowledge-register search | Yes — a separately deployed provider cannot silently reinterpret an existing session through a newer rules version. |
| Product charter and OpenWiki product/runtime pages | Direct recall | Yes — the SDK, sandbox, remote server, and browser/frontend questions are deliberately unresolved later decisions rather than current alpha promises. |
| `BUL-001-initial-push-pending` | Bulletin preflight | Yes — local planning may continue, but remote CI and the renamed repository's `main` branch remain unconfirmed until a separately authorized push. |
| Qt 6.11 official Quick, performance, security, untrusted-data, 3D, and WebEngine documentation | Phase 2 primary-source review | Yes — rich local 2D/optional 3D are technically available, while untrusted QML is explicitly unsupported and a custom DSL is recommended. |
| RFC 9700, RFC 9068, and RFC 9421 | Phase 2 primary-source review | Yes — the provider proof must profile asymmetric authentication, audience/scope/expiry restriction, sender/replay defenses, TLS, and signed message coverage rather than invent a loose bearer scheme. |
| Ticket 014 CodeGraph design exploration | Current-flow and blast-radius review | Yes — the compiled registry is injected through `AppState`; game commands currently combine pool and registry; sync already provides the minimal participant wakeup seam. |

## What happened

Ticket 014 turned the portable-game idea into an accepted staged architecture
and an executable proof without weakening the private-alpha authority model.
The resulting **Game Cartridge** is a signed, immutable, data-only package of
declarative screens, schemas, localization, and bounded assets rendered by
trusted OmarchyGS QML. Rich local 2D presentation is the baseline; arbitrary
game QML, JavaScript, native code, direct networking, and general-purpose game
engines remain outside the cartridge trust boundary.

The isolated proof signed and verified a fixture cartridge, launched separate
broker and provider processes, derived a pairwise persona grant, advanced one
provider-owned revision, replayed the command idempotently, rejected a duplicate
event, and rendered the sanitized view through trusted QML. The Phase 4 diff
gate sample covered 120 frames at 15.99 ms average and 17.00 ms maximum, 88,184 KiB
peak QML RSS, and a four-file 2,436-byte package. Those numbers validate the
harness, not a shipping Rich-2D budget.

Inspection caught four future-boundary weaknesses even though the proof is
loopback-only and not deployed: parsing reopened files after hashing, provider
responses were capped after buffering, QML text relied on automatic markup
detection, and directory traversal work was not bounded by the accepted-file
budget. All four were repaired. Identity binding, receipt attribution, expiry
semantics, and canonical-gate coverage were hardened too. Codex Security scan
`887ffea4-3265-409e-952e-8241fa49647f` completed with complete coverage and no
reportable deployed vulnerability after severity calibration.

ADR-0002 retains compiled OmarchyGS authority now and sequences the public
cartridge contract, trusted renderer, separate-repository SDK proof, production
provider security, and only then a first-party remote-authority migration with
an explicit Constitution §10 amendment. OpenWiki repair run
`a719e1c6-54bd-470c-b332-265a40a04416` completed without warnings, added the
durable cartridge navigation page, and wrote a matching completion receipt.

## Failures captured

| ID | Failure | Where surfaced |
|---|---|---|
| `BF-omarchy-gaming-system-authenticated-cartridge-reopen-001` | Cartridge files were hashed and later reopened for parsing, so interpreted bytes were not necessarily the authenticated bytes. | Phase 3.5 authenticated-input review. |
| `BF-omarchy-gaming-system-provider-response-post-buffer-bound-001` | The broker checked the provider response limit only after the HTTP library buffered the complete body. | Codex Security provider-boundary review. |
| `BF-omarchy-gaming-system-qml-auto-text-untrusted-markup-001` | Provider-derived QML `Text` values used automatic text-format detection and could be interpreted as markup. | Codex Security trusted-renderer review. |
| `BF-omarchy-gaming-system-cartridge-directory-budget-gap-001` | Accepted files were bounded, but empty or deeply nested directory traversal was not covered by the file budget. | Codex Security package-boundary review. |
| `BF-omarchy-gaming-system-nested-proof-gate-omission-001` | The nested cartridge proof workspace was included in the gated hash but initially absent from the canonical test sequence. | Phase 3.5 evidence-integrity review. |
| `BF-omarchy-gaming-system-qml-proof-log-routing-001` | The first offscreen QML proof did not expose console metrics to the harness without explicit stderr logging. | Phase 3 proof execution. |

## Prevention rules captured

| ID | Rule | Why |
|---|---|---|
| `PR-omarchy-gaming-system-parse-the-bytes-that-were-authenticated-001` | Parse security-sensitive package records from the exact byte buffers whose lengths and digests were verified. | Hashing a path and reopening it leaves a time-of-check/time-of-use seam. |
| `PR-omarchy-gaming-system-enforce-bounds-during-streaming-001` | Enforce an untrusted response or archive byte limit while streaming, before buffering or decoding the complete input. | A post-buffer size check cannot prevent memory exhaustion. |
| `PR-omarchy-gaming-system-make-untrusted-text-format-explicit-001` | Render untrusted text through an explicit plain-text mode unless a separately sanitized markup contract is intended and tested. | Automatic format detection turns data into presentation syntax. |
| `PR-omarchy-gaming-system-bound-package-traversal-work-001` | Bound package entries, directory depth, and accepted directory names in addition to accepted-file count and bytes. | Empty and nested directory trees can consume work without entering a file budget. |
| `PR-omarchy-gaming-system-gate-every-nested-workspace-proof-001` | When a ticket's evidence lives in a nested workspace, make its focused proof a canonical gate before relying on a worktree receipt. | Hash coverage proves the files are unchanged, not that their independent toolchain and runtime flow passed. |
| `PR-omarchy-gaming-system-bind-provider-messages-to-registered-identity-001` | Bind cartridge, grant, request, receipt, and event validation to operator-registered publisher/provider IDs plus exact game, release, session, subject, scope, and expiry. | A valid signature is insufficient if the signed principal or context is not the one selected by policy. |

## Architecture decisions

| ID | Decision | ADR |
|---|---|---|
| `AD-omarchy-gaming-system-game-cartridge-provider-boundary-001` | Adopt signed data-only Game Cartridges rendered by trusted OmarchyGS components; retain compiled platform authority now and permit a brokered single-authority provider mode only after the sequenced security, migration, and Constitution work. | `../../../architecture/adr-0002-game-cartridge-and-provider-boundary.md`; `../../../../openwiki/game-cartridges.md` |

Every new ID above must also be appended to
`docs/planning/knowledge/INDEX.md`.

## Effectiveness

5/5. The pipeline answered the frontend-delivery and graphics-ceiling questions,
proved the chosen trust and retry boundary across processes, repaired every
confirmed inspection issue, preserved the current production authority model,
created an implementation-ready ticket sequence, reconciled durable docs, and
completed all eight requirements without enabling speculative remote access.

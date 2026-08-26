---
title: TICKET-024-signal-siege-versus-and-keyboard-first-game-flow
status: closed
ticket_number: 024
type: feature
created: 2026-08-25
closed: 2026-08-25
intake:
pipeline_spec: docs/planning/pipeline/completed/signal-siege-versus-and-keyboard-first-game-flow.spec.md
---

# TICKET-024-signal-siege-versus-and-keyboard-first-game-flow

## Summary

Complete the first-playable challenge-to-match path with an immutable
two-human Signal Siege version and keyboard-first QML catalog, challenge,
session, and trusted gameplay presentation screens.

## Why

The server already owns durable challenges and game sessions, and the QML
client now owns account, persona, connections, and inbox flows. Production,
however, advertises only one-human game definitions, so no real challenge can
be accepted into a playable match. This slice closes that product gap without
rewriting Signal Siege v1 or loading game-supplied executable frontend code.

## EARS requirements

| ID | EARS requirement | Verification |
|---|---|---|
| REQ-001 | When production starts, the system shall retain immutable one-human Signal Siege v1 and additionally advertise Signal Siege v2 as an exact two-human `platform_compiled` definition, while solo start shall continue to admit v1 and challenge creation shall admit v2. | Runtime manifest/rules unit tests, catalog tests, solo/challenge PostgreSQL tests |
| REQ-002 | When either participant submits a valid Signal Siege v2 action on that participant's active turn, the system shall deterministically apply one bounded alternating turn, reject the wrong seat or unaffordable/malformed action, advance exactly one revision, and reach an explicit bounded winner/draw outcome by core destruction or the turn limit. | Exhaustive v2 rule matrix and existing revision/idempotency PostgreSQL tests plus challenged-match integration |
| REQ-003 | When an authenticated selected persona opens the games client, the QML system shall load and strictly validate the public catalog plus participant-private session inventory, distinguish loading/empty/offline/protocol/error states, and expose only exact supported game/version actions. | Deterministic QML catalog/session and hostile-schema cases |
| REQ-004 | When the player starts Signal Siege v1 or opens an existing supported session, the QML system shall use the selected owned persona, a fresh idempotency key for new intent, durable REST truth, and participant-authorized detail without receiving or persisting the bearer. | Deterministic QML solo start/replay/history cases and real migrated smoke |
| REQ-005 | When the selected persona manages game challenges, the QML system shall load bounded challenge history and accepted connections, create only an exact two-human supported challenge, apply directionally valid accept/decline/cancel actions, and open an accepted session through its returned UUID. | Deterministic QML challenge lifecycle/policy cases and real two-account challenge acceptance |
| REQ-006 | When a supported compiled session is displayed, the QML system shall project its strictly validated state through a clearly platform-owned presenter using repository-owned inert nodes, emit only allowlisted action payloads, enable commands only for the actor's active seat, refetch after revision conflict or another participant's turn, and retain completed history without polling or claiming an authenticated cartridge origin. | Presenter/controller unit interaction tests, wrong-turn/conflict/completion fixtures, provenance assertion, and real two-account playthrough |
| REQ-007 | If transport, schema, size, authorization, availability, or session authority becomes invalid, the QML system shall reject partial/untrusted state, preserve the last safe snapshot where appropriate, show a bounded recovery action, and clear all player authority on a valid `invalid_session` response. | Hostile deterministic fixtures and existing authority cleanup assertions |
| REQ-008 | While navigating catalog, challenges, sessions, and gameplay at 640×420 or larger, the QML system shall provide keyboard-only focus order, Enter activation, Escape recovery, accessible names, plain-text peer/game state, and visible loading/empty/error/completed states. | QML keyboard/accessibility tests at minimum and normal sizes |
| REQ-009 | When two real accounts with connected personas use the production QML controllers against migrated PostgreSQL, they shall create and accept a Signal Siege v2 challenge, alternate authoritative commands to completion, restart/refetch the session, and observe the same terminal result with no developer-only game mutation path. | Live two-authority QML/PostgreSQL/Rust acceptance scenario plus canonical diff gate |

## Scope

- In: immutable Signal Siege v2 two-human rules; production registration;
  existing challenge/session APIs; QML game controller; catalog, challenge,
  session, and gameplay screens; trusted platform presenter for Signal Siege
  v1/v2 using repository-owned inert nodes; manual REST refresh/recovery;
  deterministic hostile fixtures;
  real two-account QML challenged-match evidence; docs, OpenWiki, security
  inspection, and AAR.
- Out: changes to Signal Siege v1 semantics; new migrations or API versions;
  executable publisher QML/JavaScript; cartridge download/install discovery;
  remote-provider gameplay presentation; WebSocket/polling client lifetime;
  matchmaking, spectators, rematches, achievements/rewards; broad visual
  polish, installer, reporting/moderation, and Git delivery.

## Links

- Depends on: `TICKET-013`, `TICKET-016`, `TICKET-020`, `TICKET-021`, `TICKET-022`, `TICKET-023`
- Pipeline spec: [completed spec](../../pipeline/completed/signal-siege-versus-and-keyboard-first-game-flow.spec.md)
- Architecture: [system overview](../../../architecture/system-overview.md)

## Outcome

Signal Siege v1 remains the immutable one-human solo definition. Production now
also advertises exact two-human v2, whose deterministic alternating turns reach
an explicit bounded outcome. The keyboard-first QML client can browse compiled
games and session history, manage challenges, play either supported version,
recover authoritative state, and complete a real two-account challenged match.

The client keeps the bearer behind the onboarding authority gateway, validates
every game/challenge/session envelope before presentation, and renders compiled
Signal Siege through an explicitly platform-owned surface without claiming
signed cartridge provenance. Thirty-three QML fixture cases, forty-five migrated
PostgreSQL tests, the live two-authority scenario, the canonical gate, OpenWiki,
CodeGraph inspection, and security review provide the completion evidence.

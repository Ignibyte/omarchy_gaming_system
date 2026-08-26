# Omarchy Gaming System roadmap

The roadmap is ordered by playable value. Tickets become active one at a time
through the local pipeline.

The roadmap is game-first. A public message board may be considered after the
private alpha, but it does not displace connections, inbox challenges, or the
server-authoritative game runtime below.

## Foundation — complete

- Rust/Axum service with database-backed `/health`
- PostgreSQL Compose service and embedded identity migration
- QML connector health screen
- One-command development and smoke workflows
- Codex work pipeline and delivery gate

## Identity and personas — complete

- [x] Account registration and Argon2id password storage
- [x] Revocable device sessions
- [x] Opt-in TOTP two-factor authentication and recovery codes
- [x] Persona creation, editing, handle lookup, and privacy boundaries

## Connections and inbox

- [x] Requests, acceptance, removal, and blocking
- [x] Conversations, messages, unread state, and typed system messages
- [x] Durable cursor sync and WebSocket notifications

## First game runtime

- [x] Game registry and versioned sessions
- [x] Idempotent, revision-checked commands
- [x] Architecture spike for portable games, a versioned OmarchyGS SDK, and remote game providers
- [x] Versioned signed Game Cartridge contract, verifier, and conformance CLI
- [x] Trusted keyboard/accessibility-first Core and Rich-2D cartridge renderer
- [x] Separate-repository OmarchyGS SDK and first-party cartridge release proof
- [x] Challenges, turn notifications, history, and expiration
- [x] One original asynchronous game with a bot opponent

## Private alpha

- [x] Keyboard-first account, password/MFA session, and persona onboarding
- [x] Keyboard-first connections and private inbox screens
- [x] Keyboard-first challenge and gameplay screens
- [x] End-to-end accessibility and visual polish across the complete player flow
- Installer/package path for Omarchy
- Reporting, suspension, audit records, backups, and restore drill
- Invite-only external testing

## Owner-operated server ecosystem

- [ ] Stable server identity and capability discovery plus saved, isolated
  client profiles for multiple independent OmarchyGS communities
- [ ] Server-admin marketplace synchronization, review metadata, exact release
  import, catalog activation, lifecycle policy, and rollback controls
- [ ] Player-facing acquisition, verification, content-addressed local cache,
  mounting, update, and removal of the selected server's signed cartridges
- [ ] Operator-local signing/import for inert custom cartridges that bypass the
  marketplace while remaining visibly distinct from vetted releases
- [ ] Server extension architecture spike comparing external-process RPC,
  Wasm, statically compiled modules, and other isolation/upgrade models
- [ ] Versioned server module base and capability-scoped typed hooks with
  configuration/state namespaces, compatibility negotiation, audit,
  disable/upgrade/rollback behavior, and conformance fixtures
- [ ] Administrator-controlled custom server-module installation with explicit
  operator trust, player-facing provenance, and no client executable bridge
- [ ] Reviewed self-hosting terms, privacy/telemetry disclosures, custom-content
  warnings, security contact expectations, and operator responsibility guide

Marketplace publication and a server's local admission are separate decisions.
The official client applies the same inert cartridge and trusted-renderer
boundary to vetted and operator-custom content. Federation, shared global
identity, and cross-server social/gameplay are later projects, not side effects
of supporting more servers.

## Post-alpha provider path

- [x] Production provider registry, scoped grant/message security, guarded egress,
  quotas, replay state, audit, and revocation
- [x] First-party remote-provider authority migration pilot plus the required
  Constitution §10 amendment
- [ ] Public OmarchyGS Provider SDK, starter backend server, version
  negotiation, conformance fixtures, reviewed co-located sidecar profile, and
  deployment/operations guide
- [ ] Reviewed external providers only after operations, recovery, suspension,
  and support policy are proven

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
- [ ] One original asynchronous game with a bot opponent

## Private alpha

- Keyboard and accessibility polish
- Installer/package path for Omarchy
- Reporting, suspension, audit records, backups, and restore drill
- Invite-only external testing

## Post-alpha provider path

- Production provider registry, scoped grant/message security, guarded egress,
  quotas, replay state, audit, and revocation
- First-party remote-provider authority migration pilot plus the required
  Constitution §10 amendment
- Reviewed external providers only after operations, recovery, suspension, and
  support policy are proven

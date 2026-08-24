# Omarchy BBS roadmap

The roadmap is ordered by playable value. Tickets become active one at a time
through the local pipeline.

## Foundation — complete

- Rust/Axum service with database-backed `/health`
- PostgreSQL Compose service and embedded identity migration
- QML connector health screen
- One-command development and smoke workflows
- Claude work pipeline and delivery gate

## Identity and personas — next

- Account registration and Argon2id password storage
- Revocable device sessions
- Persona creation, editing, handle lookup, and privacy boundaries

## Connections and inbox

- Requests, acceptance, removal, and blocking
- Conversations, messages, unread state, and typed system messages
- Durable cursor sync and WebSocket notifications

## First game runtime

- Game registry and versioned sessions
- Idempotent, revision-checked commands
- Challenges, turn notifications, history, and expiration
- One original asynchronous game with a bot opponent

## Private alpha

- Keyboard and accessibility polish
- Installer/package path for Omarchy
- Reporting, suspension, audit records, backups, and restore drill
- Invite-only external testing

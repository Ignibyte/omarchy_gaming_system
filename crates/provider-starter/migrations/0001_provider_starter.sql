CREATE TABLE provider_starter_identity (
    singleton BOOLEAN PRIMARY KEY DEFAULT TRUE CHECK (singleton),
    provider_id TEXT NOT NULL,
    release_id UUID NOT NULL,
    game_key TEXT NOT NULL,
    rules_version BIGINT NOT NULL,
    cartridge_digest TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT clock_timestamp(),
    CONSTRAINT provider_starter_identity_provider CHECK (
        octet_length(provider_id) BETWEEN 3 AND 64
        AND provider_id ~ '^[a-z0-9_-]+$'
    ),
    CONSTRAINT provider_starter_identity_game CHECK (
        octet_length(game_key) BETWEEN 3 AND 32
        AND game_key ~ '^[a-z0-9_-]+$'
    ),
    CONSTRAINT provider_starter_identity_rules CHECK (rules_version BETWEEN 1 AND 4294967295),
    CONSTRAINT provider_starter_identity_digest CHECK (
        octet_length(cartridge_digest) = 64
        AND cartridge_digest ~ '^[0-9a-f]+$'
    )
);

CREATE TABLE provider_starter_sessions (
    platform_session_id UUID PRIMARY KEY,
    provider_id TEXT NOT NULL,
    release_id UUID NOT NULL,
    game_key TEXT NOT NULL,
    rules_version BIGINT NOT NULL,
    cartridge_digest TEXT NOT NULL,
    pairwise_subject TEXT NOT NULL,
    revision BIGINT NOT NULL DEFAULT 0,
    status TEXT NOT NULL,
    game_state JSONB NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT clock_timestamp(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT clock_timestamp(),
    completed_at TIMESTAMPTZ,
    CONSTRAINT provider_starter_sessions_subject CHECK (
        octet_length(pairwise_subject) = 43
        AND pairwise_subject ~ '^[A-Za-z0-9_-]+$'
    ),
    CONSTRAINT provider_starter_sessions_revision CHECK (revision >= 0),
    CONSTRAINT provider_starter_sessions_status CHECK (status IN ('active', 'completed')),
    CONSTRAINT provider_starter_sessions_state CHECK (
        jsonb_typeof(game_state) = 'object'
        AND octet_length(game_state::text) <= 32768
    ),
    CONSTRAINT provider_starter_sessions_completion CHECK (
        (status = 'active' AND completed_at IS NULL)
        OR (status = 'completed' AND completed_at IS NOT NULL)
    )
);

CREATE TABLE provider_starter_consumed_grants (
    token_id UUID PRIMARY KEY,
    provider_id TEXT NOT NULL,
    release_id UUID NOT NULL,
    platform_session_id UUID NOT NULL,
    idempotency_key UUID NOT NULL,
    request_sha256 TEXT NOT NULL,
    consumed_at TIMESTAMPTZ NOT NULL DEFAULT clock_timestamp(),
    CONSTRAINT provider_starter_grants_digest CHECK (
        octet_length(request_sha256) = 64
        AND request_sha256 ~ '^[0-9a-f]+$'
    )
);

CREATE TABLE provider_starter_operation_receipts (
    platform_session_id UUID NOT NULL,
    idempotency_key UUID NOT NULL,
    provider_id TEXT NOT NULL,
    release_id UUID NOT NULL,
    operation TEXT NOT NULL,
    expected_revision BIGINT NOT NULL,
    intent_sha256 TEXT NOT NULL,
    response_body BYTEA NOT NULL,
    provider_revision BIGINT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT clock_timestamp(),
    PRIMARY KEY (platform_session_id, idempotency_key),
    CONSTRAINT provider_starter_receipts_operation CHECK (operation IN ('launch', 'command', 'reconcile')),
    CONSTRAINT provider_starter_receipts_revision CHECK (expected_revision >= 0 AND provider_revision >= 0),
    CONSTRAINT provider_starter_receipts_digest CHECK (
        octet_length(intent_sha256) = 64
        AND intent_sha256 ~ '^[0-9a-f]+$'
    ),
    CONSTRAINT provider_starter_receipts_body CHECK (octet_length(response_body) BETWEEN 1 AND 65536)
);

CREATE TABLE provider_starter_event_outbox (
    event_id UUID PRIMARY KEY,
    provider_id TEXT NOT NULL,
    release_id UUID NOT NULL,
    platform_session_id UUID NOT NULL REFERENCES provider_starter_sessions(platform_session_id) ON DELETE RESTRICT,
    message_id UUID NOT NULL UNIQUE,
    provider_revision BIGINT NOT NULL,
    body BYTEA NOT NULL,
    status TEXT NOT NULL DEFAULT 'pending',
    attempt_count INTEGER NOT NULL DEFAULT 0,
    next_attempt_at TIMESTAMPTZ NOT NULL DEFAULT clock_timestamp(),
    delivered_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT clock_timestamp(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT clock_timestamp(),
    CONSTRAINT provider_starter_outbox_revision CHECK (provider_revision > 0),
    CONSTRAINT provider_starter_outbox_body CHECK (octet_length(body) BETWEEN 1 AND 65536),
    CONSTRAINT provider_starter_outbox_status CHECK (status IN ('pending', 'delivered', 'failed')),
    CONSTRAINT provider_starter_outbox_attempts CHECK (attempt_count BETWEEN 0 AND 8),
    CONSTRAINT provider_starter_outbox_delivery CHECK (
        (status = 'delivered' AND delivered_at IS NOT NULL)
        OR (status <> 'delivered' AND delivered_at IS NULL)
    )
);

CREATE INDEX provider_starter_outbox_pending_idx
    ON provider_starter_event_outbox(next_attempt_at, created_at, event_id)
    WHERE status = 'pending';

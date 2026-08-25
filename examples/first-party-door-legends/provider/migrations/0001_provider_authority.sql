CREATE TABLE door_legends_sessions (
    platform_session_id UUID PRIMARY KEY,
    pairwise_subject TEXT NOT NULL,
    revision BIGINT NOT NULL DEFAULT 0,
    status TEXT NOT NULL DEFAULT 'active',
    room TEXT NOT NULL DEFAULT 'brass_door',
    created_at TIMESTAMPTZ NOT NULL DEFAULT clock_timestamp(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT clock_timestamp(),
    completed_at TIMESTAMPTZ,
    CONSTRAINT door_legends_sessions_pairwise_subject CHECK (
        octet_length(pairwise_subject) = 43
        AND pairwise_subject ~ '^[A-Za-z0-9_-]+$'
    ),
    CONSTRAINT door_legends_sessions_nonnegative_revision CHECK (revision >= 0),
    CONSTRAINT door_legends_sessions_known_status CHECK (status IN ('active', 'completed')),
    CONSTRAINT door_legends_sessions_known_room CHECK (room IN ('brass_door', 'sunlit_gate')),
    CONSTRAINT door_legends_sessions_completion_shape CHECK (
        (status = 'active' AND completed_at IS NULL)
        OR (status = 'completed' AND completed_at IS NOT NULL)
    ),
    CONSTRAINT door_legends_sessions_timestamp_order CHECK (
        updated_at >= created_at
        AND (completed_at IS NULL OR completed_at >= created_at)
    )
);

CREATE TABLE door_legends_consumed_grants (
    token_id UUID PRIMARY KEY,
    platform_session_id UUID NOT NULL,
    idempotency_key UUID NOT NULL,
    request_sha256 TEXT NOT NULL,
    consumed_at TIMESTAMPTZ NOT NULL DEFAULT clock_timestamp(),
    CONSTRAINT door_legends_consumed_grants_digest CHECK (
        octet_length(request_sha256) = 64
        AND request_sha256 ~ '^[0-9a-f]+$'
    )
);

CREATE TABLE door_legends_operation_receipts (
    platform_session_id UUID NOT NULL,
    idempotency_key UUID NOT NULL,
    operation TEXT NOT NULL,
    expected_revision BIGINT NOT NULL,
    intent_sha256 TEXT NOT NULL,
    response_body BYTEA NOT NULL,
    provider_revision BIGINT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT clock_timestamp(),
    PRIMARY KEY (platform_session_id, idempotency_key),
    CONSTRAINT door_legends_operation_receipts_known_operation
        CHECK (operation IN ('launch', 'command', 'reconcile')),
    CONSTRAINT door_legends_operation_receipts_nonnegative_revision
        CHECK (expected_revision >= 0 AND provider_revision >= 0),
    CONSTRAINT door_legends_operation_receipts_digest CHECK (
        octet_length(intent_sha256) = 64
        AND intent_sha256 ~ '^[0-9a-f]+$'
    ),
    CONSTRAINT door_legends_operation_receipts_response_size
        CHECK (octet_length(response_body) BETWEEN 1 AND 65536)
);

CREATE TABLE door_legends_event_outbox (
    event_id UUID PRIMARY KEY,
    platform_session_id UUID NOT NULL REFERENCES door_legends_sessions(platform_session_id) ON DELETE RESTRICT,
    message_id UUID NOT NULL UNIQUE,
    provider_revision BIGINT NOT NULL,
    body BYTEA NOT NULL,
    status TEXT NOT NULL DEFAULT 'pending',
    attempt_count INTEGER NOT NULL DEFAULT 0,
    delivered_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT clock_timestamp(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT clock_timestamp(),
    CONSTRAINT door_legends_event_outbox_positive_revision CHECK (provider_revision > 0),
    CONSTRAINT door_legends_event_outbox_body_size CHECK (octet_length(body) BETWEEN 1 AND 65536),
    CONSTRAINT door_legends_event_outbox_known_status CHECK (status IN ('pending', 'delivered')),
    CONSTRAINT door_legends_event_outbox_delivery_shape CHECK (
        (status = 'pending' AND delivered_at IS NULL)
        OR (status = 'delivered' AND delivered_at IS NOT NULL)
    ),
    CONSTRAINT door_legends_event_outbox_attempts CHECK (attempt_count >= 0)
);

CREATE INDEX door_legends_event_outbox_pending_idx
    ON door_legends_event_outbox(status, created_at, event_id)
    WHERE status = 'pending';

CREATE TABLE game_challenges (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    idempotency_key UUID NOT NULL,
    challenger_persona_id UUID NOT NULL REFERENCES personas(id) ON DELETE RESTRICT,
    challenged_persona_id UUID NOT NULL REFERENCES personas(id) ON DELETE RESTRICT,
    game_key TEXT NOT NULL,
    game_version BIGINT NOT NULL,
    status TEXT NOT NULL DEFAULT 'pending',
    game_session_id UUID UNIQUE REFERENCES game_sessions(id) ON DELETE RESTRICT,
    expires_at TIMESTAMPTZ NOT NULL,
    resolved_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    CONSTRAINT game_challenges_idempotency_unique
        UNIQUE (challenger_persona_id, idempotency_key),
    CONSTRAINT game_challenges_distinct_participants
        CHECK (challenger_persona_id <> challenged_persona_id),
    CONSTRAINT game_challenges_canonical_key
        CHECK (
            octet_length(game_key) BETWEEN 3 AND 32
            AND game_key = lower(game_key)
            AND game_key ~ '^[a-z0-9][a-z0-9_-]*$'
        ),
    CONSTRAINT game_challenges_positive_version CHECK (game_version > 0),
    CONSTRAINT game_challenges_known_status
        CHECK (status IN ('pending', 'accepted', 'declined', 'cancelled', 'expired')),
    CONSTRAINT game_challenges_state_shape CHECK (
        (status = 'pending' AND game_session_id IS NULL AND resolved_at IS NULL)
        OR (status = 'accepted' AND game_session_id IS NOT NULL AND resolved_at IS NOT NULL)
        OR (status IN ('declined', 'cancelled', 'expired')
            AND game_session_id IS NULL
            AND resolved_at IS NOT NULL)
    ),
    CONSTRAINT game_challenges_timestamp_order CHECK (
        expires_at > created_at
        AND updated_at >= created_at
        AND (resolved_at IS NULL OR resolved_at >= created_at)
    )
);

CREATE UNIQUE INDEX game_challenges_pending_exact_game_idx
    ON game_challenges (
        challenger_persona_id,
        challenged_persona_id,
        game_key,
        game_version
    )
    WHERE status = 'pending';

CREATE INDEX game_challenges_challenger_history_idx
    ON game_challenges (challenger_persona_id, created_at DESC, id DESC);

CREATE INDEX game_challenges_challenged_history_idx
    ON game_challenges (challenged_persona_id, created_at DESC, id DESC);

CREATE INDEX game_challenges_pending_expiry_idx
    ON game_challenges (expires_at)
    WHERE status = 'pending';

ALTER TABLE inbox_messages
    ADD COLUMN system_game_challenge_id UUID REFERENCES game_challenges(id) ON DELETE RESTRICT,
    ADD COLUMN system_game_session_id UUID REFERENCES game_sessions(id) ON DELETE RESTRICT;

ALTER TABLE inbox_messages
    DROP CONSTRAINT inbox_messages_content_valid;

ALTER TABLE inbox_messages
    ADD CONSTRAINT inbox_messages_content_valid CHECK (
        (
            message_type = 'user'
            AND sender_persona_id IS NOT NULL
            AND user_body IS NOT NULL
            AND char_length(user_body) BETWEEN 1 AND 4000
            AND system_type IS NULL
            AND system_actor_persona_id IS NULL
            AND system_game_challenge_id IS NULL
            AND system_game_session_id IS NULL
        )
        OR
        (
            message_type = 'system'
            AND sender_persona_id IS NULL
            AND user_body IS NULL
            AND system_type = 'connection_accepted'
            AND system_actor_persona_id IS NOT NULL
            AND system_game_challenge_id IS NULL
            AND system_game_session_id IS NULL
        )
        OR
        (
            message_type = 'system'
            AND sender_persona_id IS NULL
            AND user_body IS NULL
            AND system_type IN (
                'game_challenge_created',
                'game_challenge_declined',
                'game_challenge_cancelled'
            )
            AND system_actor_persona_id IS NOT NULL
            AND system_game_challenge_id IS NOT NULL
            AND system_game_session_id IS NULL
        )
        OR
        (
            message_type = 'system'
            AND sender_persona_id IS NULL
            AND user_body IS NULL
            AND system_type = 'game_challenge_accepted'
            AND system_actor_persona_id IS NOT NULL
            AND system_game_challenge_id IS NOT NULL
            AND system_game_session_id IS NOT NULL
        )
    );

ALTER TABLE persona_sync_events
    ADD COLUMN game_challenge_id UUID;

ALTER TABLE persona_sync_events
    DROP CONSTRAINT persona_sync_events_known_type,
    DROP CONSTRAINT persona_sync_events_payload_shape;

ALTER TABLE persona_sync_events
    ADD CONSTRAINT persona_sync_events_known_type
        CHECK (event_type IN (
            'connection_requests_changed',
            'connections_changed',
            'blocks_changed',
            'conversation_changed',
            'game_session_changed',
            'game_challenge_changed'
        )),
    ADD CONSTRAINT persona_sync_events_payload_shape
        CHECK (
            (event_type = 'conversation_changed'
                AND conversation_id IS NOT NULL
                AND game_session_id IS NULL
                AND game_challenge_id IS NULL)
            OR (event_type = 'game_session_changed'
                AND conversation_id IS NULL
                AND game_session_id IS NOT NULL
                AND game_challenge_id IS NULL)
            OR (event_type = 'game_challenge_changed'
                AND conversation_id IS NULL
                AND game_session_id IS NULL
                AND game_challenge_id IS NOT NULL)
            OR (event_type NOT IN (
                    'conversation_changed',
                    'game_session_changed',
                    'game_challenge_changed'
                )
                AND conversation_id IS NULL
                AND game_session_id IS NULL
                AND game_challenge_id IS NULL)
        );

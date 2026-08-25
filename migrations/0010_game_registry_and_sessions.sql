CREATE TABLE game_sessions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    game_key TEXT NOT NULL,
    game_version BIGINT NOT NULL,
    revision BIGINT NOT NULL DEFAULT 0,
    status TEXT NOT NULL DEFAULT 'active',
    state JSONB NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    CONSTRAINT game_sessions_canonical_key
        CHECK (
            octet_length(game_key) BETWEEN 3 AND 32
            AND game_key = lower(game_key)
            AND game_key ~ '^[a-z0-9][a-z0-9_-]*$'
        ),
    CONSTRAINT game_sessions_positive_version CHECK (game_version > 0),
    CONSTRAINT game_sessions_nonnegative_revision CHECK (revision >= 0),
    CONSTRAINT game_sessions_known_status CHECK (status = 'active'),
    CONSTRAINT game_sessions_object_state CHECK (jsonb_typeof(state) = 'object'),
    CONSTRAINT game_sessions_timestamp_order CHECK (updated_at >= created_at)
);

CREATE TABLE game_session_participants (
    game_session_id UUID NOT NULL REFERENCES game_sessions(id) ON DELETE CASCADE,
    persona_id UUID NOT NULL REFERENCES personas(id) ON DELETE RESTRICT,
    seat SMALLINT NOT NULL,
    PRIMARY KEY (game_session_id, persona_id),
    CONSTRAINT game_session_participants_unique_seat UNIQUE (game_session_id, seat),
    CONSTRAINT game_session_participants_bounded_seat CHECK (seat BETWEEN 0 AND 7)
);

CREATE INDEX game_session_participants_persona_session_idx
    ON game_session_participants (persona_id, game_session_id);

ALTER TABLE persona_sync_events
    ADD COLUMN game_session_id UUID;

ALTER TABLE persona_sync_events
    DROP CONSTRAINT persona_sync_events_known_type,
    DROP CONSTRAINT persona_sync_events_conversation_shape;

ALTER TABLE persona_sync_events
    ADD CONSTRAINT persona_sync_events_known_type
        CHECK (event_type IN (
            'connection_requests_changed',
            'connections_changed',
            'blocks_changed',
            'conversation_changed',
            'game_session_changed'
        )),
    ADD CONSTRAINT persona_sync_events_payload_shape
        CHECK (
            (event_type = 'conversation_changed'
                AND conversation_id IS NOT NULL
                AND game_session_id IS NULL)
            OR (event_type = 'game_session_changed'
                AND conversation_id IS NULL
                AND game_session_id IS NOT NULL)
            OR (event_type NOT IN ('conversation_changed', 'game_session_changed')
                AND conversation_id IS NULL
                AND game_session_id IS NULL)
        );

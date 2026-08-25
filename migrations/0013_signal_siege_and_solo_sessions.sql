ALTER TABLE game_sessions
    DROP CONSTRAINT game_sessions_known_status,
    ADD COLUMN completed_at TIMESTAMPTZ,
    ADD CONSTRAINT game_sessions_known_status
        CHECK (status IN ('active', 'completed')),
    ADD CONSTRAINT game_sessions_completion_shape
        CHECK (
            (status = 'active' AND completed_at IS NULL)
            OR (status = 'completed'
                AND completed_at IS NOT NULL
                AND completed_at >= created_at
                AND completed_at <= updated_at)
        );

ALTER TABLE game_session_commands
    ADD COLUMN session_status TEXT NOT NULL DEFAULT 'active',
    ADD CONSTRAINT game_session_commands_known_status
        CHECK (session_status IN ('active', 'completed'));

ALTER TABLE game_session_commands
    ALTER COLUMN session_status DROP DEFAULT;

CREATE TABLE game_session_starts (
    persona_id UUID NOT NULL,
    idempotency_key UUID NOT NULL,
    game_session_id UUID NOT NULL UNIQUE,
    game_key TEXT NOT NULL,
    game_version BIGINT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    PRIMARY KEY (persona_id, idempotency_key),
    CONSTRAINT game_session_starts_participant_fk
        FOREIGN KEY (game_session_id, persona_id)
        REFERENCES game_session_participants (game_session_id, persona_id)
        ON DELETE CASCADE,
    CONSTRAINT game_session_starts_canonical_key
        CHECK (
            octet_length(game_key) BETWEEN 3 AND 32
            AND game_key = lower(game_key)
            AND game_key ~ '^[a-z0-9][a-z0-9_-]*$'
        ),
    CONSTRAINT game_session_starts_positive_version CHECK (game_version > 0)
);

CREATE INDEX game_session_starts_persona_session_idx
    ON game_session_starts (persona_id, game_session_id);

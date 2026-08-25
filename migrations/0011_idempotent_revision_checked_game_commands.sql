CREATE TABLE game_session_commands (
    game_session_id UUID NOT NULL,
    idempotency_key UUID NOT NULL,
    actor_persona_id UUID NOT NULL,
    expected_revision BIGINT NOT NULL,
    applied_revision BIGINT NOT NULL,
    command JSONB NOT NULL,
    state JSONB NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    PRIMARY KEY (game_session_id, idempotency_key),
    CONSTRAINT game_session_commands_participant_fk
        FOREIGN KEY (game_session_id, actor_persona_id)
        REFERENCES game_session_participants (game_session_id, persona_id)
        ON DELETE CASCADE,
    CONSTRAINT game_session_commands_unique_revision
        UNIQUE (game_session_id, applied_revision),
    CONSTRAINT game_session_commands_nonnegative_expected_revision
        CHECK (expected_revision >= 0),
    CONSTRAINT game_session_commands_next_revision
        CHECK (applied_revision = expected_revision + 1),
    CONSTRAINT game_session_commands_object_command
        CHECK (jsonb_typeof(command) = 'object'),
    CONSTRAINT game_session_commands_object_state
        CHECK (jsonb_typeof(state) = 'object')
);

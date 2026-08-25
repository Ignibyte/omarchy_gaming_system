CREATE TABLE persona_sync_state (
    persona_id UUID PRIMARY KEY REFERENCES personas(id) ON DELETE CASCADE,
    last_event_sequence BIGINT NOT NULL DEFAULT 0,
    CONSTRAINT persona_sync_state_nonnegative_sequence
        CHECK (last_event_sequence >= 0)
);

CREATE TABLE persona_sync_events (
    persona_id UUID NOT NULL REFERENCES personas(id) ON DELETE CASCADE,
    event_sequence BIGINT NOT NULL,
    event_type TEXT NOT NULL,
    conversation_id UUID,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    PRIMARY KEY (persona_id, event_sequence),
    CONSTRAINT persona_sync_events_positive_sequence
        CHECK (event_sequence > 0),
    CONSTRAINT persona_sync_events_known_type
        CHECK (event_type IN (
            'connection_requests_changed',
            'connections_changed',
            'blocks_changed',
            'conversation_changed'
        )),
    CONSTRAINT persona_sync_events_conversation_shape
        CHECK (
            (event_type = 'conversation_changed' AND conversation_id IS NOT NULL)
            OR (event_type <> 'conversation_changed' AND conversation_id IS NULL)
        )
);

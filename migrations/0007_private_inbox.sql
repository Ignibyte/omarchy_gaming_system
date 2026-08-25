CREATE TABLE inbox_conversations (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    persona_low_id UUID NOT NULL REFERENCES personas(id) ON DELETE RESTRICT,
    persona_high_id UUID NOT NULL REFERENCES personas(id) ON DELETE RESTRICT,
    low_last_read_sequence BIGINT NOT NULL DEFAULT 0,
    high_last_read_sequence BIGINT NOT NULL DEFAULT 0,
    last_message_sequence BIGINT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    CONSTRAINT inbox_conversations_pair_unique
        UNIQUE (persona_low_id, persona_high_id),
    CONSTRAINT inbox_conversations_canonical_pair
        CHECK (persona_low_id < persona_high_id),
    CONSTRAINT inbox_conversations_low_read_nonnegative
        CHECK (low_last_read_sequence >= 0),
    CONSTRAINT inbox_conversations_high_read_nonnegative
        CHECK (high_last_read_sequence >= 0),
    CONSTRAINT inbox_conversations_latest_nonnegative
        CHECK (last_message_sequence IS NULL OR last_message_sequence > 0)
);

CREATE INDEX inbox_conversations_high_persona_idx
    ON inbox_conversations (persona_high_id, updated_at DESC);

CREATE TABLE inbox_messages (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    message_sequence BIGINT GENERATED ALWAYS AS IDENTITY UNIQUE,
    conversation_id UUID NOT NULL REFERENCES inbox_conversations(id) ON DELETE CASCADE,
    sender_persona_id UUID REFERENCES personas(id) ON DELETE RESTRICT,
    message_type TEXT NOT NULL,
    user_body TEXT,
    system_type TEXT,
    system_actor_persona_id UUID REFERENCES personas(id) ON DELETE RESTRICT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    CONSTRAINT inbox_messages_sequence_positive CHECK (message_sequence > 0),
    CONSTRAINT inbox_messages_content_valid CHECK (
        (
            message_type = 'user'
            AND sender_persona_id IS NOT NULL
            AND user_body IS NOT NULL
            AND char_length(user_body) BETWEEN 1 AND 4000
            AND system_type IS NULL
            AND system_actor_persona_id IS NULL
        )
        OR
        (
            message_type = 'system'
            AND sender_persona_id IS NULL
            AND user_body IS NULL
            AND system_type = 'connection_accepted'
            AND system_actor_persona_id IS NOT NULL
        )
    )
);

CREATE INDEX inbox_messages_conversation_sequence_idx
    ON inbox_messages (conversation_id, message_sequence DESC);

ALTER TABLE inbox_conversations
    ADD CONSTRAINT inbox_conversations_last_message_fk
    FOREIGN KEY (last_message_sequence)
    REFERENCES inbox_messages(message_sequence)
    DEFERRABLE INITIALLY DEFERRED;

INSERT INTO inbox_conversations (
    persona_low_id,
    persona_high_id,
    created_at,
    updated_at
)
SELECT
    persona_low_id,
    persona_high_id,
    accepted_at,
    accepted_at
FROM persona_connections
WHERE status = 'accepted';

WITH inserted_messages AS (
    INSERT INTO inbox_messages (
        conversation_id,
        message_type,
        system_type,
        system_actor_persona_id,
        created_at
    )
    SELECT
        conversation.id,
        'system',
        'connection_accepted',
        connection.addressee_id,
        connection.accepted_at
    FROM inbox_conversations AS conversation
    JOIN persona_connections AS connection
      ON connection.persona_low_id = conversation.persona_low_id
     AND connection.persona_high_id = conversation.persona_high_id
    WHERE connection.status = 'accepted'
    RETURNING conversation_id, message_sequence, system_actor_persona_id
)
UPDATE inbox_conversations AS conversation
SET
    last_message_sequence = inserted.message_sequence,
    low_last_read_sequence = CASE
        WHEN inserted.system_actor_persona_id = conversation.persona_low_id
            THEN inserted.message_sequence
        ELSE conversation.low_last_read_sequence
    END,
    high_last_read_sequence = CASE
        WHEN inserted.system_actor_persona_id = conversation.persona_high_id
            THEN inserted.message_sequence
        ELSE conversation.high_last_read_sequence
    END
FROM inserted_messages AS inserted
WHERE conversation.id = inserted.conversation_id;

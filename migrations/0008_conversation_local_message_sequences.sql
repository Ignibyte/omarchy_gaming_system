CREATE TEMPORARY TABLE inbox_message_sequence_migration ON COMMIT DROP AS
SELECT
    id AS message_id,
    conversation_id,
    message_sequence AS old_sequence,
    row_number() OVER (
        PARTITION BY conversation_id
        ORDER BY message_sequence
    )::BIGINT AS new_sequence
FROM inbox_messages;

ALTER TABLE inbox_conversations
    DROP CONSTRAINT inbox_conversations_last_message_fk;

ALTER TABLE inbox_messages
    DROP CONSTRAINT inbox_messages_message_sequence_key;

ALTER TABLE inbox_messages
    ALTER COLUMN message_sequence DROP IDENTITY;

UPDATE inbox_conversations AS conversation
SET
    low_last_read_sequence = COALESCE((
        SELECT max(sequence_map.new_sequence)
        FROM inbox_message_sequence_migration AS sequence_map
        WHERE sequence_map.conversation_id = conversation.id
          AND sequence_map.old_sequence <= conversation.low_last_read_sequence
    ), 0),
    high_last_read_sequence = COALESCE((
        SELECT max(sequence_map.new_sequence)
        FROM inbox_message_sequence_migration AS sequence_map
        WHERE sequence_map.conversation_id = conversation.id
          AND sequence_map.old_sequence <= conversation.high_last_read_sequence
    ), 0),
    last_message_sequence = (
        SELECT max(sequence_map.new_sequence)
        FROM inbox_message_sequence_migration AS sequence_map
        WHERE sequence_map.conversation_id = conversation.id
    );

UPDATE inbox_messages AS message
SET message_sequence = sequence_map.new_sequence
FROM inbox_message_sequence_migration AS sequence_map
WHERE message.id = sequence_map.message_id;

ALTER TABLE inbox_messages
    ADD CONSTRAINT inbox_messages_conversation_sequence_unique
    UNIQUE (conversation_id, message_sequence);

ALTER TABLE inbox_conversations
    ADD CONSTRAINT inbox_conversations_last_message_fk
    FOREIGN KEY (id, last_message_sequence)
    REFERENCES inbox_messages(conversation_id, message_sequence)
    DEFERRABLE INITIALLY DEFERRED;

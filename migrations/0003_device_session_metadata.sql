ALTER TABLE account_sessions
    ADD COLUMN device_name TEXT NOT NULL DEFAULT 'Unknown device',
    ADD CONSTRAINT account_sessions_device_name_length
        CHECK (char_length(device_name) BETWEEN 1 AND 64),
    ADD CONSTRAINT account_sessions_token_hash_length
        CHECK (octet_length(token_hash) = 32);

ALTER TABLE account_sessions
    ALTER COLUMN device_name DROP DEFAULT;

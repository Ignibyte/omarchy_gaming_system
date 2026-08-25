CREATE TABLE account_totp_authenticators (
    account_id UUID PRIMARY KEY REFERENCES accounts(id) ON DELETE CASCADE,
    encrypted_secret BYTEA NOT NULL,
    secret_nonce BYTEA NOT NULL,
    enabled_at TIMESTAMPTZ,
    last_used_step BIGINT,
    failed_attempts SMALLINT NOT NULL DEFAULT 0,
    locked_until TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    CONSTRAINT account_totp_secret_ciphertext_length
        CHECK (octet_length(encrypted_secret) = 36),
    CONSTRAINT account_totp_secret_nonce_length
        CHECK (octet_length(secret_nonce) = 12),
    CONSTRAINT account_totp_failed_attempts_range
        CHECK (failed_attempts BETWEEN 0 AND 5)
);

CREATE TABLE account_mfa_recovery_codes (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    account_id UUID NOT NULL REFERENCES account_totp_authenticators(account_id)
        ON DELETE CASCADE,
    code_hash BYTEA NOT NULL,
    used_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    CONSTRAINT account_mfa_recovery_code_hash_length
        CHECK (octet_length(code_hash) = 32),
    UNIQUE (account_id, code_hash)
);

CREATE INDEX account_mfa_recovery_codes_account_id_idx
    ON account_mfa_recovery_codes (account_id);

CREATE TABLE account_mfa_login_challenges (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    account_id UUID NOT NULL REFERENCES accounts(id) ON DELETE CASCADE,
    token_hash BYTEA NOT NULL UNIQUE,
    device_name TEXT NOT NULL,
    expires_at TIMESTAMPTZ NOT NULL,
    failed_attempts SMALLINT NOT NULL DEFAULT 0,
    consumed_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    CONSTRAINT account_mfa_challenge_token_hash_length
        CHECK (octet_length(token_hash) = 32),
    CONSTRAINT account_mfa_challenge_device_name_length
        CHECK (char_length(device_name) BETWEEN 1 AND 64),
    CONSTRAINT account_mfa_challenge_failed_attempts_range
        CHECK (failed_attempts BETWEEN 0 AND 5),
    CONSTRAINT account_mfa_challenge_expiry_after_creation
        CHECK (expires_at > created_at)
);

CREATE INDEX account_mfa_login_challenges_account_id_idx
    ON account_mfa_login_challenges (account_id);

CREATE INDEX account_mfa_login_challenges_expiry_idx
    ON account_mfa_login_challenges (expires_at)
    WHERE consumed_at IS NULL;

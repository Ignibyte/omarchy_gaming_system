CREATE TABLE provider_registrations (
    provider_id TEXT PRIMARY KEY,
    display_name TEXT NOT NULL,
    status TEXT NOT NULL DEFAULT 'active',
    revoked_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT clock_timestamp(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT clock_timestamp(),
    CONSTRAINT provider_registrations_canonical_id CHECK (
        octet_length(provider_id) BETWEEN 3 AND 64
        AND provider_id = lower(provider_id)
        AND provider_id ~ '^[a-z0-9][a-z0-9_-]*$'
    ),
    CONSTRAINT provider_registrations_display_name CHECK (
        char_length(display_name) BETWEEN 1 AND 96
        AND display_name !~ '[[:cntrl:]]'
    ),
    CONSTRAINT provider_registrations_known_status
        CHECK (status IN ('active', 'suspended', 'revoked')),
    CONSTRAINT provider_registrations_revocation_shape CHECK (
        (status = 'revoked' AND revoked_at IS NOT NULL)
        OR (status <> 'revoked' AND revoked_at IS NULL)
    ),
    CONSTRAINT provider_registrations_timestamp_order
        CHECK (updated_at >= created_at AND (revoked_at IS NULL OR revoked_at >= created_at))
);

CREATE TABLE provider_releases (
    release_id UUID PRIMARY KEY,
    provider_id TEXT NOT NULL REFERENCES provider_registrations(provider_id) ON DELETE RESTRICT,
    game_key TEXT NOT NULL,
    rules_version BIGINT NOT NULL,
    cartridge_digest TEXT NOT NULL,
    endpoint_host TEXT NOT NULL,
    endpoint_port INTEGER NOT NULL DEFAULT 443,
    endpoint_base_path TEXT NOT NULL,
    status TEXT NOT NULL DEFAULT 'active',
    active_session_policy TEXT NOT NULL,
    config_revision BIGINT NOT NULL DEFAULT 1,
    grant_limit_per_minute INTEGER NOT NULL,
    request_limit_per_minute INTEGER NOT NULL,
    callback_limit_per_minute INTEGER NOT NULL,
    max_concurrent_requests INTEGER NOT NULL,
    request_body_limit_bytes INTEGER NOT NULL,
    response_body_limit_bytes INTEGER NOT NULL,
    connect_timeout_ms INTEGER NOT NULL,
    total_timeout_ms INTEGER NOT NULL,
    revoked_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT clock_timestamp(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT clock_timestamp(),
    CONSTRAINT provider_releases_exact_identity UNIQUE (
        provider_id,
        game_key,
        rules_version,
        cartridge_digest
    ),
    CONSTRAINT provider_releases_canonical_game_key CHECK (
        octet_length(game_key) BETWEEN 3 AND 32
        AND game_key = lower(game_key)
        AND game_key ~ '^[a-z0-9][a-z0-9_-]*$'
    ),
    CONSTRAINT provider_releases_positive_rules_version CHECK (rules_version > 0),
    CONSTRAINT provider_releases_sha256_cartridge_digest CHECK (
        octet_length(cartridge_digest) = 64
        AND cartridge_digest = lower(cartridge_digest)
        AND cartridge_digest ~ '^[0-9a-f]+$'
    ),
    CONSTRAINT provider_releases_canonical_endpoint_host CHECK (
        octet_length(endpoint_host) BETWEEN 3 AND 253
        AND endpoint_host = lower(endpoint_host)
        AND endpoint_host ~ '^[a-z0-9][a-z0-9.-]*[a-z0-9]$'
        AND endpoint_host LIKE '%.%'
        AND endpoint_host NOT LIKE '%.local'
        AND endpoint_host NOT LIKE '%.localhost'
    ),
    CONSTRAINT provider_releases_endpoint_port CHECK (endpoint_port BETWEEN 1 AND 65535),
    CONSTRAINT provider_releases_canonical_base_path CHECK (
        octet_length(endpoint_base_path) BETWEEN 1 AND 256
        AND left(endpoint_base_path, 1) = '/'
        AND right(endpoint_base_path, 1) = '/'
        AND endpoint_base_path !~ '[?#\\]'
        AND endpoint_base_path !~ '(^|/)\.{1,2}(/|$)'
        AND endpoint_base_path !~ '[[:cntrl:]]'
    ),
    CONSTRAINT provider_releases_known_status
        CHECK (status IN ('active', 'suspended', 'revoked')),
    CONSTRAINT provider_releases_known_active_session_policy
        CHECK (active_session_policy IN ('terminate', 'read_only', 'continue')),
    CONSTRAINT provider_releases_positive_config_revision CHECK (config_revision > 0),
    CONSTRAINT provider_releases_rate_limits CHECK (
        grant_limit_per_minute BETWEEN 1 AND 10000
        AND request_limit_per_minute BETWEEN 1 AND 10000
        AND callback_limit_per_minute BETWEEN 1 AND 10000
    ),
    CONSTRAINT provider_releases_concurrency_limit
        CHECK (max_concurrent_requests BETWEEN 1 AND 64),
    CONSTRAINT provider_releases_body_limits CHECK (
        request_body_limit_bytes BETWEEN 1024 AND 65536
        AND response_body_limit_bytes BETWEEN 1024 AND 524288
    ),
    CONSTRAINT provider_releases_timeouts CHECK (
        connect_timeout_ms BETWEEN 100 AND 5000
        AND total_timeout_ms BETWEEN 250 AND 15000
        AND total_timeout_ms >= connect_timeout_ms
    ),
    CONSTRAINT provider_releases_revocation_shape CHECK (
        (status = 'revoked' AND revoked_at IS NOT NULL)
        OR (status <> 'revoked' AND revoked_at IS NULL)
    ),
    CONSTRAINT provider_releases_timestamp_order CHECK (
        updated_at >= created_at
        AND (revoked_at IS NULL OR revoked_at >= created_at)
    )
);

CREATE INDEX provider_releases_provider_status_idx
    ON provider_releases(provider_id, status, release_id);

CREATE TABLE provider_release_scopes (
    release_id UUID NOT NULL REFERENCES provider_releases(release_id) ON DELETE RESTRICT,
    scope TEXT NOT NULL,
    status TEXT NOT NULL DEFAULT 'active',
    revoked_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT clock_timestamp(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT clock_timestamp(),
    PRIMARY KEY (release_id, scope),
    CONSTRAINT provider_release_scopes_known_scope
        CHECK (scope IN ('game.launch', 'game.command', 'game.reconcile', 'game.event')),
    CONSTRAINT provider_release_scopes_known_status
        CHECK (status IN ('active', 'suspended', 'revoked')),
    CONSTRAINT provider_release_scopes_revocation_shape CHECK (
        (status = 'revoked' AND revoked_at IS NOT NULL)
        OR (status <> 'revoked' AND revoked_at IS NULL)
    ),
    CONSTRAINT provider_release_scopes_timestamp_order CHECK (
        updated_at >= created_at
        AND (revoked_at IS NULL OR revoked_at >= created_at)
    )
);

CREATE TABLE provider_release_keys (
    release_id UUID NOT NULL REFERENCES provider_releases(release_id) ON DELETE RESTRICT,
    key_kind TEXT NOT NULL,
    key_id TEXT NOT NULL,
    public_material BYTEA NOT NULL,
    material_sha256 TEXT NOT NULL,
    status TEXT NOT NULL DEFAULT 'active',
    valid_from TIMESTAMPTZ NOT NULL,
    valid_until TIMESTAMPTZ,
    revoked_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT clock_timestamp(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT clock_timestamp(),
    PRIMARY KEY (release_id, key_kind, key_id),
    CONSTRAINT provider_release_keys_known_kind
        CHECK (key_kind IN ('message_ed25519', 'tls_root_der')),
    CONSTRAINT provider_release_keys_canonical_id CHECK (
        octet_length(key_id) BETWEEN 3 AND 64
        AND key_id = lower(key_id)
        AND key_id ~ '^[a-z0-9][a-z0-9._-]*$'
    ),
    CONSTRAINT provider_release_keys_material_shape CHECK (
        (key_kind = 'message_ed25519' AND octet_length(public_material) = 32)
        OR (key_kind = 'tls_root_der'
            AND octet_length(public_material) BETWEEN 64 AND 32768)
    ),
    CONSTRAINT provider_release_keys_sha256_digest CHECK (
        octet_length(material_sha256) = 64
        AND material_sha256 = lower(material_sha256)
        AND material_sha256 ~ '^[0-9a-f]+$'
    ),
    CONSTRAINT provider_release_keys_known_status
        CHECK (status IN ('active', 'suspended', 'revoked')),
    CONSTRAINT provider_release_keys_validity
        CHECK (valid_until IS NULL OR valid_until > valid_from),
    CONSTRAINT provider_release_keys_revocation_shape CHECK (
        (status = 'revoked' AND revoked_at IS NOT NULL)
        OR (status <> 'revoked' AND revoked_at IS NULL)
    ),
    CONSTRAINT provider_release_keys_timestamp_order CHECK (
        updated_at >= created_at
        AND (revoked_at IS NULL OR revoked_at >= created_at)
    )
);

CREATE INDEX provider_release_keys_active_idx
    ON provider_release_keys(release_id, key_kind, status, valid_from, valid_until);

CREATE TABLE provider_grants (
    token_id UUID PRIMARY KEY,
    release_id UUID NOT NULL REFERENCES provider_releases(release_id) ON DELETE RESTRICT,
    platform_session_id UUID NOT NULL,
    pairwise_subject TEXT NOT NULL,
    scope TEXT NOT NULL,
    claims_sha256 TEXT NOT NULL,
    signed_grant BYTEA NOT NULL,
    issued_at TIMESTAMPTZ NOT NULL,
    expires_at TIMESTAMPTZ NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT clock_timestamp(),
    CONSTRAINT provider_grants_pairwise_subject CHECK (
        octet_length(pairwise_subject) = 43
        AND pairwise_subject ~ '^[A-Za-z0-9_-]+$'
    ),
    CONSTRAINT provider_grants_known_scope
        CHECK (scope IN ('game.launch', 'game.command', 'game.reconcile')),
    CONSTRAINT provider_grants_sha256_digest CHECK (
        octet_length(claims_sha256) = 64
        AND claims_sha256 = lower(claims_sha256)
        AND claims_sha256 ~ '^[0-9a-f]+$'
    ),
    CONSTRAINT provider_grants_signed_size
        CHECK (octet_length(signed_grant) BETWEEN 1 AND 8192),
    CONSTRAINT provider_grants_lifetime CHECK (
        expires_at > issued_at
        AND expires_at <= issued_at + INTERVAL '60 seconds'
    )
);

CREATE INDEX provider_grants_release_expiry_idx
    ON provider_grants(release_id, expires_at);

CREATE TABLE provider_quota_windows (
    release_id UUID NOT NULL REFERENCES provider_releases(release_id) ON DELETE RESTRICT,
    quota_kind TEXT NOT NULL,
    window_started_at TIMESTAMPTZ NOT NULL,
    used BIGINT NOT NULL,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT clock_timestamp(),
    PRIMARY KEY (release_id, quota_kind, window_started_at),
    CONSTRAINT provider_quota_windows_known_kind
        CHECK (quota_kind IN ('grant', 'request', 'callback')),
    CONSTRAINT provider_quota_windows_minute_boundary
        CHECK (window_started_at = date_trunc('minute', window_started_at)),
    CONSTRAINT provider_quota_windows_positive_used CHECK (used > 0)
);

CREATE TABLE provider_concurrency_leases (
    lease_id UUID PRIMARY KEY,
    release_id UUID NOT NULL REFERENCES provider_releases(release_id) ON DELETE RESTRICT,
    expires_at TIMESTAMPTZ NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT clock_timestamp(),
    CONSTRAINT provider_concurrency_leases_lifetime CHECK (
        expires_at > created_at
        AND expires_at <= created_at + INTERVAL '30 seconds'
    )
);

CREATE INDEX provider_concurrency_leases_release_expiry_idx
    ON provider_concurrency_leases(release_id, expires_at);

CREATE TABLE provider_operations (
    release_id UUID NOT NULL REFERENCES provider_releases(release_id) ON DELETE RESTRICT,
    platform_session_id UUID NOT NULL,
    idempotency_key UUID NOT NULL,
    scope TEXT NOT NULL,
    expected_revision BIGINT NOT NULL,
    intent_sha256 TEXT NOT NULL,
    intent_body BYTEA NOT NULL,
    status TEXT NOT NULL DEFAULT 'prepared',
    attempt_count INTEGER NOT NULL DEFAULT 0,
    provider_revision BIGINT,
    response_sha256 TEXT,
    response_body BYTEA,
    last_error_code TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT clock_timestamp(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT clock_timestamp(),
    PRIMARY KEY (release_id, platform_session_id, idempotency_key),
    CONSTRAINT provider_operations_known_scope
        CHECK (scope IN ('game.launch', 'game.command', 'game.reconcile')),
    CONSTRAINT provider_operations_nonnegative_expected_revision
        CHECK (expected_revision >= 0),
    CONSTRAINT provider_operations_intent_digest CHECK (
        octet_length(intent_sha256) = 64
        AND intent_sha256 = lower(intent_sha256)
        AND intent_sha256 ~ '^[0-9a-f]+$'
    ),
    CONSTRAINT provider_operations_intent_size
        CHECK (octet_length(intent_body) BETWEEN 1 AND 65536),
    CONSTRAINT provider_operations_known_status
        CHECK (status IN ('prepared', 'in_flight', 'completed', 'unknown', 'failed')),
    CONSTRAINT provider_operations_attempt_count CHECK (attempt_count >= 0),
    CONSTRAINT provider_operations_provider_revision
        CHECK (provider_revision IS NULL OR provider_revision >= 0),
    CONSTRAINT provider_operations_response_digest CHECK (
        response_sha256 IS NULL
        OR (
            octet_length(response_sha256) = 64
            AND response_sha256 = lower(response_sha256)
            AND response_sha256 ~ '^[0-9a-f]+$'
        )
    ),
    CONSTRAINT provider_operations_response_size
        CHECK (response_body IS NULL OR octet_length(response_body) <= 524288),
    CONSTRAINT provider_operations_error_code CHECK (
        last_error_code IS NULL
        OR (
            octet_length(last_error_code) BETWEEN 3 AND 64
            AND last_error_code ~ '^[a-z0-9_]+$'
        )
    ),
    CONSTRAINT provider_operations_completion_shape CHECK (
        (status = 'completed'
            AND provider_revision IS NOT NULL
            AND response_sha256 IS NOT NULL
            AND response_body IS NOT NULL
            AND last_error_code IS NULL)
        OR (status <> 'completed')
    ),
    CONSTRAINT provider_operations_timestamp_order CHECK (updated_at >= created_at)
);

CREATE INDEX provider_operations_session_idx
    ON provider_operations(release_id, platform_session_id, updated_at DESC);

CREATE TABLE provider_operation_attempts (
    release_id UUID NOT NULL,
    platform_session_id UUID NOT NULL,
    idempotency_key UUID NOT NULL,
    attempt_number INTEGER NOT NULL,
    message_id UUID NOT NULL UNIQUE,
    grant_token_id UUID NOT NULL REFERENCES provider_grants(token_id) ON DELETE RESTRICT,
    request_sha256 TEXT NOT NULL,
    request_body BYTEA NOT NULL,
    status TEXT NOT NULL DEFAULT 'in_flight',
    error_code TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT clock_timestamp(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT clock_timestamp(),
    PRIMARY KEY (release_id, platform_session_id, idempotency_key, attempt_number),
    CONSTRAINT provider_operation_attempts_operation_fk
        FOREIGN KEY (release_id, platform_session_id, idempotency_key)
        REFERENCES provider_operations (release_id, platform_session_id, idempotency_key)
        ON DELETE RESTRICT,
    CONSTRAINT provider_operation_attempts_positive_number CHECK (attempt_number > 0),
    CONSTRAINT provider_operation_attempts_digest CHECK (
        octet_length(request_sha256) = 64
        AND request_sha256 = lower(request_sha256)
        AND request_sha256 ~ '^[0-9a-f]+$'
    ),
    CONSTRAINT provider_operation_attempts_request_size
        CHECK (octet_length(request_body) BETWEEN 1 AND 65536),
    CONSTRAINT provider_operation_attempts_known_status
        CHECK (status IN ('in_flight', 'completed', 'unknown', 'failed')),
    CONSTRAINT provider_operation_attempts_error_code CHECK (
        error_code IS NULL
        OR (
            octet_length(error_code) BETWEEN 3 AND 64
            AND error_code ~ '^[a-z0-9_]+$'
        )
    ),
    CONSTRAINT provider_operation_attempts_timestamp_order CHECK (updated_at >= created_at)
);

CREATE TABLE provider_message_receipts (
    release_id UUID NOT NULL REFERENCES provider_releases(release_id) ON DELETE RESTRICT,
    direction TEXT NOT NULL,
    message_id UUID NOT NULL,
    platform_session_id UUID NOT NULL,
    event_id UUID,
    authenticated_sha256 TEXT NOT NULL,
    disposition TEXT NOT NULL,
    provider_revision BIGINT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT clock_timestamp(),
    PRIMARY KEY (release_id, direction, message_id),
    CONSTRAINT provider_message_receipts_known_direction
        CHECK (direction IN ('response', 'callback')),
    CONSTRAINT provider_message_receipts_digest CHECK (
        octet_length(authenticated_sha256) = 64
        AND authenticated_sha256 = lower(authenticated_sha256)
        AND authenticated_sha256 ~ '^[0-9a-f]+$'
    ),
    CONSTRAINT provider_message_receipts_known_disposition
        CHECK (disposition IN ('accepted', 'duplicate', 'ignored')),
    CONSTRAINT provider_message_receipts_provider_revision
        CHECK (provider_revision IS NULL OR provider_revision >= 0)
);

CREATE UNIQUE INDEX provider_message_receipts_event_idx
    ON provider_message_receipts(release_id, direction, event_id)
    WHERE event_id IS NOT NULL;

CREATE TABLE provider_security_audit_events (
    sequence BIGINT GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    event_id UUID NOT NULL UNIQUE DEFAULT gen_random_uuid(),
    provider_id TEXT NOT NULL REFERENCES provider_registrations(provider_id) ON DELETE RESTRICT,
    release_id UUID REFERENCES provider_releases(release_id) ON DELETE RESTRICT,
    actor_type TEXT NOT NULL,
    actor_id TEXT NOT NULL,
    event_type TEXT NOT NULL,
    outcome TEXT NOT NULL,
    reason_code TEXT NOT NULL,
    correlation_id UUID,
    safe_details JSONB NOT NULL DEFAULT '{}'::jsonb,
    created_at TIMESTAMPTZ NOT NULL DEFAULT clock_timestamp(),
    CONSTRAINT provider_security_audit_actor_type
        CHECK (actor_type IN ('operator', 'broker', 'provider')),
    CONSTRAINT provider_security_audit_actor_id CHECK (
        octet_length(actor_id) BETWEEN 1 AND 96
        AND actor_id !~ '[[:cntrl:]]'
    ),
    CONSTRAINT provider_security_audit_event_type CHECK (
        octet_length(event_type) BETWEEN 3 AND 64
        AND event_type ~ '^[a-z0-9_]+$'
    ),
    CONSTRAINT provider_security_audit_outcome
        CHECK (outcome IN ('allowed', 'denied', 'failed', 'recorded')),
    CONSTRAINT provider_security_audit_reason_code CHECK (
        octet_length(reason_code) BETWEEN 2 AND 64
        AND reason_code ~ '^[a-z0-9_]+$'
    ),
    CONSTRAINT provider_security_audit_details_object
        CHECK (jsonb_typeof(safe_details) = 'object'),
    CONSTRAINT provider_security_audit_details_size
        CHECK (octet_length(safe_details::text) <= 4096)
);

CREATE INDEX provider_security_audit_lookup_idx
    ON provider_security_audit_events(provider_id, release_id, sequence DESC);

CREATE FUNCTION provider_release_identity_immutable() RETURNS trigger
LANGUAGE plpgsql AS $$
BEGIN
    IF NEW.release_id IS DISTINCT FROM OLD.release_id
        OR NEW.provider_id IS DISTINCT FROM OLD.provider_id
        OR NEW.game_key IS DISTINCT FROM OLD.game_key
        OR NEW.rules_version IS DISTINCT FROM OLD.rules_version
        OR NEW.cartridge_digest IS DISTINCT FROM OLD.cartridge_digest
        OR NEW.endpoint_host IS DISTINCT FROM OLD.endpoint_host
        OR NEW.endpoint_port IS DISTINCT FROM OLD.endpoint_port
        OR NEW.endpoint_base_path IS DISTINCT FROM OLD.endpoint_base_path
        OR NEW.created_at IS DISTINCT FROM OLD.created_at
    THEN
        RAISE EXCEPTION 'provider release identity is immutable';
    END IF;
    IF OLD.status = 'revoked' AND NEW.status <> 'revoked' THEN
        RAISE EXCEPTION 'revoked provider release is terminal';
    END IF;
    RETURN NEW;
END;
$$;

CREATE TRIGGER provider_release_identity_immutable_trigger
BEFORE UPDATE ON provider_releases
FOR EACH ROW EXECUTE FUNCTION provider_release_identity_immutable();

CREATE FUNCTION provider_key_identity_immutable() RETURNS trigger
LANGUAGE plpgsql AS $$
BEGIN
    IF NEW.release_id IS DISTINCT FROM OLD.release_id
        OR NEW.key_kind IS DISTINCT FROM OLD.key_kind
        OR NEW.key_id IS DISTINCT FROM OLD.key_id
        OR NEW.public_material IS DISTINCT FROM OLD.public_material
        OR NEW.material_sha256 IS DISTINCT FROM OLD.material_sha256
        OR NEW.valid_from IS DISTINCT FROM OLD.valid_from
        OR NEW.valid_until IS DISTINCT FROM OLD.valid_until
        OR NEW.created_at IS DISTINCT FROM OLD.created_at
    THEN
        RAISE EXCEPTION 'provider key identity is immutable';
    END IF;
    IF OLD.status = 'revoked' AND NEW.status <> 'revoked' THEN
        RAISE EXCEPTION 'revoked provider key is terminal';
    END IF;
    RETURN NEW;
END;
$$;

CREATE TRIGGER provider_key_identity_immutable_trigger
BEFORE UPDATE ON provider_release_keys
FOR EACH ROW EXECUTE FUNCTION provider_key_identity_immutable();

CREATE FUNCTION provider_scope_identity_immutable() RETURNS trigger
LANGUAGE plpgsql AS $$
BEGIN
    IF NEW.release_id IS DISTINCT FROM OLD.release_id
        OR NEW.scope IS DISTINCT FROM OLD.scope
        OR NEW.created_at IS DISTINCT FROM OLD.created_at
    THEN
        RAISE EXCEPTION 'provider scope identity is immutable';
    END IF;
    IF OLD.status = 'revoked' AND NEW.status <> 'revoked' THEN
        RAISE EXCEPTION 'revoked provider scope is terminal';
    END IF;
    RETURN NEW;
END;
$$;

CREATE TRIGGER provider_scope_identity_immutable_trigger
BEFORE UPDATE ON provider_release_scopes
FOR EACH ROW EXECUTE FUNCTION provider_scope_identity_immutable();

CREATE FUNCTION provider_registration_lifecycle_guard() RETURNS trigger
LANGUAGE plpgsql AS $$
BEGIN
    IF NEW.provider_id IS DISTINCT FROM OLD.provider_id
        OR NEW.created_at IS DISTINCT FROM OLD.created_at
    THEN
        RAISE EXCEPTION 'provider identity is immutable';
    END IF;
    IF OLD.status = 'revoked' AND NEW.status <> 'revoked' THEN
        RAISE EXCEPTION 'revoked provider is terminal';
    END IF;
    RETURN NEW;
END;
$$;

CREATE TRIGGER provider_registration_lifecycle_guard_trigger
BEFORE UPDATE ON provider_registrations
FOR EACH ROW EXECUTE FUNCTION provider_registration_lifecycle_guard();

CREATE FUNCTION provider_forbid_delete() RETURNS trigger
LANGUAGE plpgsql AS $$
BEGIN
    RAISE EXCEPTION 'provider control and audit evidence cannot be deleted';
END;
$$;

CREATE TRIGGER provider_registrations_forbid_delete
BEFORE DELETE ON provider_registrations
FOR EACH ROW EXECUTE FUNCTION provider_forbid_delete();

CREATE TRIGGER provider_releases_forbid_delete
BEFORE DELETE ON provider_releases
FOR EACH ROW EXECUTE FUNCTION provider_forbid_delete();

CREATE TRIGGER provider_release_scopes_forbid_delete
BEFORE DELETE ON provider_release_scopes
FOR EACH ROW EXECUTE FUNCTION provider_forbid_delete();

CREATE TRIGGER provider_release_keys_forbid_delete
BEFORE DELETE ON provider_release_keys
FOR EACH ROW EXECUTE FUNCTION provider_forbid_delete();

CREATE FUNCTION provider_audit_immutable() RETURNS trigger
LANGUAGE plpgsql AS $$
BEGIN
    RAISE EXCEPTION 'provider security audit is append-only';
END;
$$;

CREATE TRIGGER provider_security_audit_immutable_trigger
BEFORE UPDATE OR DELETE ON provider_security_audit_events
FOR EACH ROW EXECUTE FUNCTION provider_audit_immutable();

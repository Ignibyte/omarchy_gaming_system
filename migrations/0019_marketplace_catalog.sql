CREATE TABLE marketplace_sync_state (
    singleton BOOLEAN PRIMARY KEY DEFAULT TRUE CHECK (singleton),
    marketplace_origin TEXT NOT NULL
        CHECK (char_length(marketplace_origin) BETWEEN 9 AND 512),
    authority_id TEXT NOT NULL
        CHECK (authority_id ~ '^[a-z][a-z0-9._-]{0,95}$'),
    key_id TEXT NOT NULL
        CHECK (key_id ~ '^[a-z][a-z0-9._-]{0,95}$'),
    marketplace_name TEXT NOT NULL
        CHECK (char_length(marketplace_name) BETWEEN 1 AND 128
            AND marketplace_name = btrim(marketplace_name)
            AND marketplace_name !~ '[[:cntrl:]]'),
    snapshot_version BIGINT NOT NULL CHECK (snapshot_version > 0),
    snapshot_sha256 TEXT NOT NULL
        CHECK (snapshot_sha256 ~ '^[0-9a-f]{64}$'),
    synchronized_at TIMESTAMPTZ NOT NULL DEFAULT clock_timestamp()
);

CREATE TABLE marketplace_releases (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    game_key TEXT NOT NULL
        CHECK (game_key ~ '^[a-z][a-z0-9._-]{0,95}$'),
    publisher_id TEXT NOT NULL
        CHECK (publisher_id ~ '^[a-z][a-z0-9._-]{0,95}$'),
    publisher_key JSONB NOT NULL CHECK (jsonb_typeof(publisher_key) = 'object'),
    rules_version BIGINT NOT NULL CHECK (rules_version > 0),
    cartridge_version BIGINT NOT NULL CHECK (cartridge_version > 0),
    archive_sha256 TEXT NOT NULL UNIQUE
        CHECK (archive_sha256 ~ '^[0-9a-f]{64}$'),
    signed_identity_sha256 TEXT NOT NULL
        CHECK (signed_identity_sha256 ~ '^[0-9a-f]{64}$'),
    display_name TEXT NOT NULL
        CHECK (char_length(display_name) BETWEEN 1 AND 128
            AND display_name = btrim(display_name)
            AND display_name !~ '[[:cntrl:]]'),
    release_path TEXT NOT NULL
        CHECK (char_length(release_path) BETWEEN 2 AND 256
            AND release_path = btrim(release_path)
            AND left(release_path, 1) <> '/'
            AND right(release_path, 1) = '/'),
    reviewed_by TEXT NOT NULL
        CHECK (reviewed_by ~ '^[a-z][a-z0-9._-]{0,95}$'),
    review_summary TEXT NOT NULL
        CHECK (char_length(review_summary) BETWEEN 1 AND 512
            AND review_summary = btrim(review_summary)
            AND review_summary !~ '[[:cntrl:]]'),
    signed_policy JSONB NOT NULL CHECK (jsonb_typeof(signed_policy) = 'object'),
    policy_version BIGINT NOT NULL CHECK (policy_version > 0),
    policy_status TEXT NOT NULL
        CHECK (policy_status IN ('active', 'deprecated', 'suspended', 'revoked', 'retired')),
    policy_reason TEXT NOT NULL
        CHECK (char_length(policy_reason) BETWEEN 1 AND 512
            AND policy_reason !~ '[[:cntrl:]]'),
    compatible BOOLEAN NOT NULL,
    imported BOOLEAN NOT NULL,
    first_seen_snapshot_version BIGINT NOT NULL
        CHECK (first_seen_snapshot_version > 0),
    last_seen_snapshot_version BIGINT NOT NULL
        CHECK (last_seen_snapshot_version >= first_seen_snapshot_version),
    created_at TIMESTAMPTZ NOT NULL DEFAULT clock_timestamp(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT clock_timestamp(),
    CONSTRAINT marketplace_release_exact_identity UNIQUE (
        game_key, publisher_id, rules_version, cartridge_version, archive_sha256
    ),
    CONSTRAINT marketplace_release_import_compatible CHECK (NOT imported OR compatible),
    CONSTRAINT marketplace_release_monotonic_time CHECK (updated_at >= created_at)
);

CREATE INDEX marketplace_releases_inventory_idx
    ON marketplace_releases(game_key, rules_version DESC, cartridge_version DESC, archive_sha256);

CREATE INDEX marketplace_releases_current_idx
    ON marketplace_releases(last_seen_snapshot_version, policy_status, game_key)
    WHERE imported AND compatible;

CREATE TABLE server_cartridge_catalogs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    game_key TEXT NOT NULL UNIQUE
        CHECK (game_key ~ '^[a-z][a-z0-9._-]{0,95}$'),
    active_release_id UUID REFERENCES marketplace_releases(id) ON DELETE RESTRICT,
    admission_revision BIGINT NOT NULL DEFAULT 0 CHECK (admission_revision >= 0),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT clock_timestamp()
);

CREATE TABLE cartridge_catalog_audit_events (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    operation_id UUID NOT NULL UNIQUE,
    catalog_id UUID NOT NULL
        REFERENCES server_cartridge_catalogs(id) ON DELETE RESTRICT,
    action TEXT NOT NULL
        CHECK (action IN (
            'activate_cartridge',
            'deactivate_cartridge',
            'upgrade_cartridge',
            'rollback_cartridge'
        )),
    actor TEXT NOT NULL
        CHECK (char_length(actor) BETWEEN 1 AND 64 AND actor = btrim(actor)),
    reason TEXT NOT NULL
        CHECK (char_length(reason) BETWEEN 1 AND 500 AND reason = btrim(reason)),
    previous_archive_sha256 TEXT
        CHECK (previous_archive_sha256 IS NULL OR previous_archive_sha256 ~ '^[0-9a-f]{64}$'),
    resulting_archive_sha256 TEXT
        CHECK (resulting_archive_sha256 IS NULL OR resulting_archive_sha256 ~ '^[0-9a-f]{64}$'),
    admission_revision BIGINT NOT NULL CHECK (admission_revision > 0),
    created_at TIMESTAMPTZ NOT NULL DEFAULT clock_timestamp(),
    CONSTRAINT cartridge_catalog_audit_changes_state CHECK (
        previous_archive_sha256 IS DISTINCT FROM resulting_archive_sha256
    ),
    CONSTRAINT cartridge_catalog_audit_action_shape CHECK (
        (action = 'activate_cartridge'
            AND previous_archive_sha256 IS NULL
            AND resulting_archive_sha256 IS NOT NULL)
        OR (action = 'deactivate_cartridge'
            AND previous_archive_sha256 IS NOT NULL
            AND resulting_archive_sha256 IS NULL)
        OR (action = 'rollback_cartridge'
            AND previous_archive_sha256 IS NOT NULL
            AND resulting_archive_sha256 IS NOT NULL)
        OR (action = 'upgrade_cartridge'
            AND previous_archive_sha256 IS NOT NULL
            AND resulting_archive_sha256 IS NOT NULL)
    )
);

CREATE INDEX cartridge_catalog_audit_catalog_idx
    ON cartridge_catalog_audit_events(catalog_id, admission_revision DESC);

CREATE FUNCTION marketplace_release_guard() RETURNS trigger
LANGUAGE plpgsql AS $$
BEGIN
    IF TG_OP = 'DELETE' THEN
        RAISE EXCEPTION 'marketplace releases cannot be deleted';
    END IF;
    IF NEW.game_key <> OLD.game_key
        OR NEW.publisher_id <> OLD.publisher_id
        OR NEW.publisher_key <> OLD.publisher_key
        OR NEW.rules_version <> OLD.rules_version
        OR NEW.cartridge_version <> OLD.cartridge_version
        OR NEW.archive_sha256 <> OLD.archive_sha256
        OR NEW.signed_identity_sha256 <> OLD.signed_identity_sha256
        OR NEW.display_name <> OLD.display_name
        OR NEW.compatible <> OLD.compatible
        OR (OLD.imported AND NOT NEW.imported)
        OR NEW.first_seen_snapshot_version <> OLD.first_seen_snapshot_version
        OR NEW.last_seen_snapshot_version < OLD.last_seen_snapshot_version
        OR NEW.policy_version < OLD.policy_version
        OR (NEW.policy_version = OLD.policy_version AND NEW.signed_policy <> OLD.signed_policy)
    THEN
        RAISE EXCEPTION 'marketplace release identity or monotonic state cannot regress';
    END IF;
    RETURN NEW;
END;
$$;

CREATE TRIGGER marketplace_release_guard_trigger
BEFORE UPDATE OR DELETE ON marketplace_releases
FOR EACH ROW EXECUTE FUNCTION marketplace_release_guard();

CREATE FUNCTION marketplace_release_no_truncate() RETURNS trigger
LANGUAGE plpgsql AS $$
BEGIN
    RAISE EXCEPTION 'marketplace releases cannot be truncated';
END;
$$;

CREATE TRIGGER marketplace_release_no_truncate_trigger
BEFORE TRUNCATE ON marketplace_releases
FOR EACH STATEMENT EXECUTE FUNCTION marketplace_release_no_truncate();

CREATE FUNCTION server_cartridge_catalog_validate() RETURNS trigger
LANGUAGE plpgsql AS $$
DECLARE
    selected_game_key TEXT;
BEGIN
    IF NEW.active_release_id IS NOT NULL THEN
        SELECT game_key INTO selected_game_key
        FROM marketplace_releases
        WHERE id = NEW.active_release_id;
        IF selected_game_key IS DISTINCT FROM NEW.game_key THEN
            RAISE EXCEPTION 'catalog selection must match its game';
        END IF;
    END IF;
    IF TG_OP = 'UPDATE'
        AND (NEW.game_key <> OLD.game_key
            OR NEW.admission_revision < OLD.admission_revision)
    THEN
        RAISE EXCEPTION 'catalog identity or revision cannot regress';
    END IF;
    RETURN NEW;
END;
$$;

CREATE TRIGGER server_cartridge_catalog_validate_trigger
BEFORE INSERT OR UPDATE ON server_cartridge_catalogs
FOR EACH ROW EXECUTE FUNCTION server_cartridge_catalog_validate();

CREATE FUNCTION cartridge_catalog_audit_immutable() RETURNS trigger
LANGUAGE plpgsql AS $$
BEGIN
    RAISE EXCEPTION 'cartridge catalog audit is append-only';
END;
$$;

CREATE TRIGGER cartridge_catalog_audit_immutable_trigger
BEFORE UPDATE OR DELETE ON cartridge_catalog_audit_events
FOR EACH ROW EXECUTE FUNCTION cartridge_catalog_audit_immutable();

CREATE TRIGGER cartridge_catalog_audit_no_truncate_trigger
BEFORE TRUNCATE ON cartridge_catalog_audit_events
FOR EACH STATEMENT EXECUTE FUNCTION cartridge_catalog_audit_immutable();

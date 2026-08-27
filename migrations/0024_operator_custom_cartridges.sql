CREATE TABLE operator_custom_authority (
    singleton BOOLEAN PRIMARY KEY DEFAULT TRUE CHECK (singleton),
    server_id UUID NOT NULL REFERENCES server_identity(id) ON DELETE RESTRICT,
    operator_name TEXT NOT NULL
        CHECK (char_length(operator_name) BETWEEN 1 AND 128
            AND operator_name = btrim(operator_name)
            AND operator_name !~ '[[:cntrl:]]'),
    authority_id TEXT NOT NULL
        CHECK (authority_id ~ '^[a-z][a-z0-9._-]{0,95}$'),
    key_id TEXT NOT NULL
        CHECK (key_id ~ '^[a-z][a-z0-9._-]{0,95}$'),
    public_key JSONB NOT NULL CHECK (jsonb_typeof(public_key) = 'object'),
    key_sha256 TEXT NOT NULL CHECK (key_sha256 ~ '^[0-9a-f]{64}$'),
    created_at TIMESTAMPTZ NOT NULL DEFAULT clock_timestamp(),
    CONSTRAINT operator_custom_authority_key_identity CHECK (
        public_key ->> 'authority_id' = authority_id
        AND public_key ->> 'key_id' = key_id
    )
);

CREATE FUNCTION operator_custom_authority_immutable() RETURNS trigger
LANGUAGE plpgsql AS $$
BEGIN
    RAISE EXCEPTION 'operator custom authority is immutable';
END;
$$;

CREATE TRIGGER operator_custom_authority_immutable_trigger
BEFORE UPDATE OR DELETE ON operator_custom_authority
FOR EACH ROW EXECUTE FUNCTION operator_custom_authority_immutable();

CREATE TRIGGER operator_custom_authority_no_truncate_trigger
BEFORE TRUNCATE ON operator_custom_authority
FOR EACH STATEMENT EXECUTE FUNCTION operator_custom_authority_immutable();

CREATE TABLE operator_custom_releases (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    import_operation_id UUID NOT NULL UNIQUE,
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
    operator_key JSONB NOT NULL CHECK (jsonb_typeof(operator_key) = 'object'),
    operator_key_sha256 TEXT NOT NULL CHECK (operator_key_sha256 ~ '^[0-9a-f]{64}$'),
    operator_name TEXT NOT NULL
        CHECK (char_length(operator_name) BETWEEN 1 AND 128
            AND operator_name = btrim(operator_name)
            AND operator_name !~ '[[:cntrl:]]'),
    signed_operator_attestation JSONB NOT NULL
        CHECK (jsonb_typeof(signed_operator_attestation) = 'object'),
    attestation_version BIGINT NOT NULL CHECK (attestation_version = 1),
    warning TEXT NOT NULL
        CHECK (char_length(warning) BETWEEN 1 AND 512
            AND warning = btrim(warning)
            AND warning !~ '[[:cntrl:]]'),
    signed_policy JSONB NOT NULL CHECK (jsonb_typeof(signed_policy) = 'object'),
    policy_version BIGINT NOT NULL CHECK (policy_version > 0),
    policy_status TEXT NOT NULL
        CHECK (policy_status IN ('active', 'deprecated', 'suspended', 'revoked', 'retired')),
    policy_reason TEXT NOT NULL
        CHECK (char_length(policy_reason) BETWEEN 1 AND 512
            AND policy_reason = btrim(policy_reason)
            AND policy_reason !~ '[[:cntrl:]]'),
    compatible BOOLEAN NOT NULL,
    imported BOOLEAN NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT clock_timestamp(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT clock_timestamp(),
    CONSTRAINT operator_custom_release_exact_identity UNIQUE (
        game_key, publisher_id, rules_version, cartridge_version, archive_sha256
    ),
    CONSTRAINT operator_custom_release_import_compatible CHECK (NOT imported OR compatible),
    CONSTRAINT operator_custom_release_monotonic_time CHECK (updated_at >= created_at)
);

CREATE INDEX operator_custom_releases_inventory_idx
    ON operator_custom_releases(
        game_key, rules_version DESC, cartridge_version DESC, archive_sha256
    );

CREATE FUNCTION operator_custom_release_guard() RETURNS trigger
LANGUAGE plpgsql AS $$
BEGIN
    IF TG_OP = 'DELETE' THEN
        RAISE EXCEPTION 'operator custom releases cannot be deleted';
    END IF;
    IF NEW.import_operation_id <> OLD.import_operation_id
        OR NEW.game_key <> OLD.game_key
        OR NEW.publisher_id <> OLD.publisher_id
        OR NEW.publisher_key <> OLD.publisher_key
        OR NEW.rules_version <> OLD.rules_version
        OR NEW.cartridge_version <> OLD.cartridge_version
        OR NEW.archive_sha256 <> OLD.archive_sha256
        OR NEW.signed_identity_sha256 <> OLD.signed_identity_sha256
        OR NEW.display_name <> OLD.display_name
        OR NEW.operator_key <> OLD.operator_key
        OR NEW.operator_key_sha256 <> OLD.operator_key_sha256
        OR NEW.operator_name <> OLD.operator_name
        OR NEW.signed_operator_attestation <> OLD.signed_operator_attestation
        OR NEW.attestation_version <> OLD.attestation_version
        OR NEW.warning <> OLD.warning
        OR NEW.compatible <> OLD.compatible
        OR (OLD.imported AND NOT NEW.imported)
        OR NEW.policy_version < OLD.policy_version
        OR (NEW.policy_version = OLD.policy_version
            AND (
                NEW.signed_policy <> OLD.signed_policy
                OR NEW.policy_status <> OLD.policy_status
                OR NEW.policy_reason <> OLD.policy_reason
            ))
    THEN
        RAISE EXCEPTION 'operator custom release identity or policy cannot regress';
    END IF;
    RETURN NEW;
END;
$$;

CREATE TRIGGER operator_custom_release_guard_trigger
BEFORE UPDATE OR DELETE ON operator_custom_releases
FOR EACH ROW EXECUTE FUNCTION operator_custom_release_guard();

CREATE TRIGGER operator_custom_release_no_truncate_trigger
BEFORE TRUNCATE ON operator_custom_releases
FOR EACH STATEMENT EXECUTE FUNCTION marketplace_release_no_truncate();

CREATE TABLE operator_custom_audit_events (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    operation_id UUID NOT NULL UNIQUE,
    release_id UUID NOT NULL REFERENCES operator_custom_releases(id) ON DELETE RESTRICT,
    action TEXT NOT NULL CHECK (action IN ('import_custom_cartridge', 'set_custom_policy')),
    actor TEXT NOT NULL
        CHECK (char_length(actor) BETWEEN 1 AND 64 AND actor = btrim(actor)
            AND actor !~ '[[:cntrl:]]'),
    reason TEXT NOT NULL
        CHECK (char_length(reason) BETWEEN 1 AND 500 AND reason = btrim(reason)
            AND reason !~ '[[:cntrl:]]'),
    previous_policy_version BIGINT CHECK (previous_policy_version IS NULL OR previous_policy_version > 0),
    previous_policy_status TEXT
        CHECK (previous_policy_status IS NULL OR previous_policy_status IN (
            'active', 'deprecated', 'suspended', 'revoked', 'retired'
        )),
    resulting_policy_version BIGINT NOT NULL CHECK (resulting_policy_version > 0),
    resulting_policy_status TEXT NOT NULL
        CHECK (resulting_policy_status IN ('active', 'deprecated', 'suspended', 'revoked', 'retired')),
    created_at TIMESTAMPTZ NOT NULL DEFAULT clock_timestamp(),
    CONSTRAINT operator_custom_audit_previous_pair CHECK (
        (previous_policy_version IS NULL) = (previous_policy_status IS NULL)
    ),
    CONSTRAINT operator_custom_audit_action_shape CHECK (
        (action = 'import_custom_cartridge' AND previous_policy_version IS NULL)
        OR (action = 'set_custom_policy'
            AND previous_policy_version IS NOT NULL
            AND resulting_policy_version > previous_policy_version)
    )
);

CREATE INDEX operator_custom_audit_release_idx
    ON operator_custom_audit_events(release_id, created_at DESC);

CREATE FUNCTION operator_custom_audit_immutable() RETURNS trigger
LANGUAGE plpgsql AS $$
BEGIN
    RAISE EXCEPTION 'operator custom audit is append-only';
END;
$$;

CREATE TRIGGER operator_custom_audit_immutable_trigger
BEFORE UPDATE OR DELETE ON operator_custom_audit_events
FOR EACH ROW EXECUTE FUNCTION operator_custom_audit_immutable();

CREATE TRIGGER operator_custom_audit_no_truncate_trigger
BEFORE TRUNCATE ON operator_custom_audit_events
FOR EACH STATEMENT EXECUTE FUNCTION operator_custom_audit_immutable();

ALTER TABLE server_cartridge_catalogs
    ADD COLUMN active_custom_release_id UUID
        REFERENCES operator_custom_releases(id) ON DELETE RESTRICT,
    ADD CONSTRAINT server_cartridge_catalog_one_source CHECK (
        num_nonnulls(active_release_id, active_custom_release_id) <= 1
    );

ALTER TABLE cartridge_catalog_audit_events
    ADD COLUMN previous_provenance_class TEXT,
    ADD COLUMN resulting_provenance_class TEXT;

UPDATE cartridge_catalog_audit_events
SET previous_provenance_class = 'marketplace_vetted'
WHERE previous_archive_sha256 IS NOT NULL;

UPDATE cartridge_catalog_audit_events
SET resulting_provenance_class = 'marketplace_vetted'
WHERE resulting_archive_sha256 IS NOT NULL;

ALTER TABLE cartridge_catalog_audit_events
    ADD CONSTRAINT cartridge_catalog_audit_previous_source CHECK (
        (previous_archive_sha256 IS NULL) = (previous_provenance_class IS NULL)
        AND (previous_provenance_class IS NULL OR previous_provenance_class IN (
            'marketplace_vetted', 'operator_custom'
        ))
    ),
    ADD CONSTRAINT cartridge_catalog_audit_resulting_source CHECK (
        (resulting_archive_sha256 IS NULL) = (resulting_provenance_class IS NULL)
        AND (resulting_provenance_class IS NULL OR resulting_provenance_class IN (
            'marketplace_vetted', 'operator_custom'
        ))
    );

CREATE OR REPLACE FUNCTION server_cartridge_catalog_validate() RETURNS trigger
LANGUAGE plpgsql AS $$
DECLARE
    selected_game_key TEXT;
BEGIN
    IF NEW.active_release_id IS NOT NULL AND NEW.active_custom_release_id IS NOT NULL THEN
        RAISE EXCEPTION 'catalog selection must use one provenance source';
    ELSIF NEW.active_release_id IS NOT NULL THEN
        SELECT game_key INTO selected_game_key
        FROM marketplace_releases
        WHERE id = NEW.active_release_id;
    ELSIF NEW.active_custom_release_id IS NOT NULL THEN
        SELECT game_key INTO selected_game_key
        FROM operator_custom_releases
        WHERE id = NEW.active_custom_release_id;
    ELSE
        selected_game_key := NEW.game_key;
    END IF;
    IF selected_game_key IS DISTINCT FROM NEW.game_key THEN
        RAISE EXCEPTION 'catalog selection must match its game';
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

ALTER TABLE game_session_cartridge_presentations
    ALTER COLUMN marketplace_release_id DROP NOT NULL,
    ADD COLUMN operator_custom_release_id UUID
        REFERENCES operator_custom_releases(id) ON DELETE RESTRICT,
    ADD COLUMN provenance_class TEXT;

UPDATE game_session_cartridge_presentations
SET provenance_class = 'marketplace_vetted';

ALTER TABLE game_session_cartridge_presentations
    ALTER COLUMN provenance_class SET NOT NULL,
    ADD CONSTRAINT game_session_cartridge_one_source CHECK (
        num_nonnulls(marketplace_release_id, operator_custom_release_id) = 1
    ),
    ADD CONSTRAINT game_session_cartridge_source_shape CHECK (
        (provenance_class = 'marketplace_vetted'
            AND marketplace_release_id IS NOT NULL
            AND operator_custom_release_id IS NULL)
        OR (provenance_class = 'operator_custom'
            AND marketplace_release_id IS NULL
            AND operator_custom_release_id IS NOT NULL)
    );

CREATE INDEX game_session_cartridge_presentations_custom_release_idx
    ON game_session_cartridge_presentations(operator_custom_release_id, game_session_id)
    WHERE operator_custom_release_id IS NOT NULL;

CREATE OR REPLACE FUNCTION game_session_cartridge_presentation_validate() RETURNS trigger
LANGUAGE plpgsql AS $$
DECLARE
    session_game_key TEXT;
    session_game_version BIGINT;
    release_game_key TEXT;
    release_rules_version BIGINT;
    release_imported BOOLEAN;
    release_compatible BOOLEAN;
    release_policy_status TEXT;
    selected_release_id UUID;
    selected_admission_revision BIGINT;
    release_snapshot_version BIGINT;
    current_snapshot_version BIGINT;
BEGIN
    IF NEW.provenance_class = 'marketplace_vetted' THEN
        SELECT session.game_key, session.game_version,
               release.game_key, release.rules_version,
               release.imported, release.compatible, release.policy_status,
               catalog.active_release_id, catalog.admission_revision,
               release.last_seen_snapshot_version, sync.snapshot_version
        INTO session_game_key, session_game_version,
             release_game_key, release_rules_version,
             release_imported, release_compatible, release_policy_status,
             selected_release_id, selected_admission_revision,
             release_snapshot_version, current_snapshot_version
        FROM game_sessions AS session
        JOIN marketplace_releases AS release ON release.id = NEW.marketplace_release_id
        JOIN server_cartridge_catalogs AS catalog ON catalog.game_key = release.game_key
        JOIN marketplace_sync_state AS sync ON sync.singleton
        WHERE session.id = NEW.game_session_id
        FOR SHARE OF session, release, catalog, sync;

        IF NOT FOUND
            OR release_snapshot_version IS DISTINCT FROM current_snapshot_version
            OR selected_release_id IS DISTINCT FROM NEW.marketplace_release_id
        THEN
            RAISE EXCEPTION 'game session marketplace presentation must match current admission';
        END IF;
    ELSIF NEW.provenance_class = 'operator_custom' THEN
        SELECT session.game_key, session.game_version,
               release.game_key, release.rules_version,
               release.imported, release.compatible, release.policy_status,
               catalog.active_custom_release_id, catalog.admission_revision
        INTO session_game_key, session_game_version,
             release_game_key, release_rules_version,
             release_imported, release_compatible, release_policy_status,
             selected_release_id, selected_admission_revision
        FROM game_sessions AS session
        JOIN operator_custom_releases AS release ON release.id = NEW.operator_custom_release_id
        JOIN server_cartridge_catalogs AS catalog ON catalog.game_key = release.game_key
        WHERE session.id = NEW.game_session_id
        FOR SHARE OF session, release, catalog;

        IF NOT FOUND OR selected_release_id IS DISTINCT FROM NEW.operator_custom_release_id THEN
            RAISE EXCEPTION 'game session custom presentation must match current admission';
        END IF;
    ELSE
        RAISE EXCEPTION 'game session cartridge provenance is invalid';
    END IF;

    IF session_game_key IS DISTINCT FROM release_game_key
        OR session_game_version IS DISTINCT FROM release_rules_version
        OR NOT release_imported
        OR NOT release_compatible
        OR release_policy_status NOT IN ('active', 'deprecated')
        OR selected_admission_revision IS DISTINCT FROM NEW.admission_revision
    THEN
        RAISE EXCEPTION 'game session cartridge presentation must match the current exact admission';
    END IF;
    RETURN NEW;
END;
$$;

CREATE OR REPLACE FUNCTION game_session_cartridge_presentation_evidence_validate() RETURNS trigger
LANGUAGE plpgsql AS $$
BEGIN
    IF NEW.provenance_class = 'marketplace_vetted'
        AND NOT EXISTS (
            SELECT 1
            FROM marketplace_release_acquisition_evidence
            WHERE marketplace_release_id = NEW.marketplace_release_id
        )
    THEN
        RAISE EXCEPTION 'marketplace session presentation requires retained acquisition evidence';
    ELSIF NEW.provenance_class = 'operator_custom'
        AND NOT EXISTS (
            SELECT 1
            FROM operator_custom_releases
            WHERE id = NEW.operator_custom_release_id
              AND imported
              AND compatible
        )
    THEN
        RAISE EXCEPTION 'custom session presentation requires retained acquisition evidence';
    END IF;
    RETURN NEW;
END;
$$;

ALTER TABLE game_session_cartridge_action_admissions
    ALTER COLUMN marketplace_release_id DROP NOT NULL,
    ADD COLUMN operator_custom_release_id UUID
        REFERENCES operator_custom_releases(id) ON DELETE RESTRICT,
    ADD COLUMN provenance_class TEXT;

UPDATE game_session_cartridge_action_admissions
SET provenance_class = 'marketplace_vetted';

ALTER TABLE game_session_cartridge_action_admissions
    ALTER COLUMN provenance_class SET NOT NULL,
    ADD CONSTRAINT game_session_cartridge_action_one_source CHECK (
        num_nonnulls(marketplace_release_id, operator_custom_release_id) = 1
    ),
    ADD CONSTRAINT game_session_cartridge_action_source_shape CHECK (
        (provenance_class = 'marketplace_vetted'
            AND marketplace_release_id IS NOT NULL
            AND operator_custom_release_id IS NULL)
        OR (provenance_class = 'operator_custom'
            AND marketplace_release_id IS NULL
            AND operator_custom_release_id IS NOT NULL)
    );

CREATE OR REPLACE FUNCTION game_session_cartridge_action_admission_validate() RETURNS trigger
LANGUAGE plpgsql AS $$
DECLARE
    current_marketplace_release_id UUID;
    current_custom_release_id UUID;
    current_provenance_class TEXT;
    current_admission_revision BIGINT;
    current_authority TEXT;
    current_revision BIGINT;
    current_archive_sha256 TEXT;
    current_signed_identity_sha256 TEXT;
    current_policy_version BIGINT;
    current_policy_status TEXT;
BEGIN
    IF NEW.provenance_class = 'marketplace_vetted' THEN
        SELECT presentation.marketplace_release_id,
               presentation.operator_custom_release_id,
               presentation.provenance_class,
               presentation.admission_revision,
               session.authority, session.revision,
               release.archive_sha256, release.signed_identity_sha256,
               release.policy_version, release.policy_status
        INTO current_marketplace_release_id, current_custom_release_id,
             current_provenance_class, current_admission_revision,
             current_authority, current_revision,
             current_archive_sha256, current_signed_identity_sha256,
             current_policy_version, current_policy_status
        FROM game_session_cartridge_presentations AS presentation
        JOIN game_sessions AS session ON session.id = presentation.game_session_id
        JOIN marketplace_releases AS release ON release.id = presentation.marketplace_release_id
        WHERE presentation.game_session_id = NEW.game_session_id
          AND EXISTS (
              SELECT 1 FROM game_session_participants AS participant
              WHERE participant.game_session_id = NEW.game_session_id
                AND participant.persona_id = NEW.actor_persona_id
          );
    ELSIF NEW.provenance_class = 'operator_custom' THEN
        SELECT presentation.marketplace_release_id,
               presentation.operator_custom_release_id,
               presentation.provenance_class,
               presentation.admission_revision,
               session.authority, session.revision,
               release.archive_sha256, release.signed_identity_sha256,
               release.policy_version, release.policy_status
        INTO current_marketplace_release_id, current_custom_release_id,
             current_provenance_class, current_admission_revision,
             current_authority, current_revision,
             current_archive_sha256, current_signed_identity_sha256,
             current_policy_version, current_policy_status
        FROM game_session_cartridge_presentations AS presentation
        JOIN game_sessions AS session ON session.id = presentation.game_session_id
        JOIN operator_custom_releases AS release ON release.id = presentation.operator_custom_release_id
        WHERE presentation.game_session_id = NEW.game_session_id
          AND EXISTS (
              SELECT 1 FROM game_session_participants AS participant
              WHERE participant.game_session_id = NEW.game_session_id
                AND participant.persona_id = NEW.actor_persona_id
          );
    ELSE
        RAISE EXCEPTION 'cartridge action provenance is invalid';
    END IF;

    IF NOT FOUND
        OR current_marketplace_release_id IS DISTINCT FROM NEW.marketplace_release_id
        OR current_custom_release_id IS DISTINCT FROM NEW.operator_custom_release_id
        OR current_provenance_class IS DISTINCT FROM NEW.provenance_class
        OR current_admission_revision IS DISTINCT FROM NEW.admission_revision
        OR current_authority IS DISTINCT FROM NEW.authority
        OR current_revision IS DISTINCT FROM NEW.expected_revision
        OR current_archive_sha256 IS DISTINCT FROM NEW.archive_sha256
        OR current_signed_identity_sha256 IS DISTINCT FROM NEW.signed_identity_sha256
        OR current_policy_version IS DISTINCT FROM NEW.policy_version
        OR current_policy_status IS DISTINCT FROM NEW.lifecycle_status
    THEN
        RAISE EXCEPTION 'cartridge action admission must match the current exact session policy';
    END IF;
    RETURN NEW;
END;
$$;

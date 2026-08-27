CREATE TABLE game_session_cartridge_presentations (
    game_session_id UUID PRIMARY KEY
        REFERENCES game_sessions(id) ON DELETE RESTRICT,
    marketplace_release_id UUID NOT NULL
        REFERENCES marketplace_releases(id) ON DELETE RESTRICT,
    admission_revision BIGINT NOT NULL CHECK (admission_revision > 0),
    created_at TIMESTAMPTZ NOT NULL DEFAULT clock_timestamp()
);

CREATE INDEX game_session_cartridge_presentations_release_idx
    ON game_session_cartridge_presentations(marketplace_release_id, game_session_id);

CREATE FUNCTION game_session_cartridge_presentation_validate() RETURNS trigger
LANGUAGE plpgsql AS $$
DECLARE
    session_game_key TEXT;
    session_game_version BIGINT;
    release_game_key TEXT;
    release_rules_version BIGINT;
    release_imported BOOLEAN;
    release_compatible BOOLEAN;
    release_snapshot_version BIGINT;
    release_policy_status TEXT;
    selected_release_id UUID;
    selected_admission_revision BIGINT;
    current_snapshot_version BIGINT;
BEGIN
    SELECT session.game_key,
           session.game_version,
           release.game_key,
           release.rules_version,
           release.imported,
           release.compatible,
           release.last_seen_snapshot_version,
           release.policy_status,
           catalog.active_release_id,
           catalog.admission_revision,
           sync.snapshot_version
    INTO session_game_key,
         session_game_version,
         release_game_key,
         release_rules_version,
         release_imported,
         release_compatible,
         release_snapshot_version,
         release_policy_status,
         selected_release_id,
         selected_admission_revision,
         current_snapshot_version
    FROM game_sessions AS session
    JOIN marketplace_releases AS release
      ON release.id = NEW.marketplace_release_id
    JOIN server_cartridge_catalogs AS catalog
      ON catalog.game_key = release.game_key
    JOIN marketplace_sync_state AS sync
      ON sync.singleton
    WHERE session.id = NEW.game_session_id
    FOR SHARE OF session, release, catalog, sync;

    IF NOT FOUND
        OR session_game_key IS DISTINCT FROM release_game_key
        OR session_game_version IS DISTINCT FROM release_rules_version
        OR NOT release_imported
        OR NOT release_compatible
        OR release_snapshot_version IS DISTINCT FROM current_snapshot_version
        OR release_policy_status NOT IN ('active', 'deprecated')
        OR selected_release_id IS DISTINCT FROM NEW.marketplace_release_id
        OR selected_admission_revision IS DISTINCT FROM NEW.admission_revision
    THEN
        RAISE EXCEPTION 'game session cartridge presentation must match the current exact admission';
    END IF;
    RETURN NEW;
END;
$$;

CREATE TRIGGER game_session_cartridge_presentation_validate_trigger
BEFORE INSERT ON game_session_cartridge_presentations
FOR EACH ROW EXECUTE FUNCTION game_session_cartridge_presentation_validate();

CREATE FUNCTION game_session_cartridge_presentation_immutable() RETURNS trigger
LANGUAGE plpgsql AS $$
BEGIN
    RAISE EXCEPTION 'game session cartridge presentations are immutable';
END;
$$;

CREATE TRIGGER game_session_cartridge_presentation_immutable_trigger
BEFORE UPDATE OR DELETE ON game_session_cartridge_presentations
FOR EACH ROW EXECUTE FUNCTION game_session_cartridge_presentation_immutable();

CREATE TRIGGER game_session_cartridge_presentation_no_truncate_trigger
BEFORE TRUNCATE ON game_session_cartridge_presentations
FOR EACH STATEMENT EXECUTE FUNCTION game_session_cartridge_presentation_immutable();

CREATE TABLE game_session_cartridge_action_admissions (
    game_session_id UUID NOT NULL
        REFERENCES game_session_cartridge_presentations(game_session_id) ON DELETE RESTRICT,
    idempotency_key UUID NOT NULL,
    actor_persona_id UUID NOT NULL REFERENCES personas(id) ON DELETE RESTRICT,
    marketplace_release_id UUID NOT NULL
        REFERENCES marketplace_releases(id) ON DELETE RESTRICT,
    admission_revision BIGINT NOT NULL CHECK (admission_revision > 0),
    authority TEXT NOT NULL
        CHECK (authority IN ('platform_compiled', 'registered_provider')),
    expected_revision BIGINT NOT NULL CHECK (expected_revision >= 0),
    archive_sha256 TEXT NOT NULL CHECK (archive_sha256 ~ '^[0-9a-f]{64}$'),
    signed_identity_sha256 TEXT NOT NULL
        CHECK (signed_identity_sha256 ~ '^[0-9a-f]{64}$'),
    policy_version BIGINT NOT NULL CHECK (policy_version > 0),
    lifecycle_status TEXT NOT NULL
        CHECK (lifecycle_status IN ('active', 'deprecated', 'retired')),
    action TEXT NOT NULL CHECK (action ~ '^[a-z][a-z0-9._-]{0,95}$'),
    payload JSONB NOT NULL CHECK (jsonb_typeof(payload) = 'object'),
    translated_command JSONB NOT NULL
        CHECK (jsonb_typeof(translated_command) = 'object'),
    created_at TIMESTAMPTZ NOT NULL DEFAULT clock_timestamp(),
    PRIMARY KEY (game_session_id, idempotency_key)
);

CREATE INDEX game_session_cartridge_action_admissions_actor_idx
    ON game_session_cartridge_action_admissions(actor_persona_id, created_at DESC);

CREATE FUNCTION game_session_cartridge_action_admission_validate() RETURNS trigger
LANGUAGE plpgsql AS $$
DECLARE
    current_release_id UUID;
    current_admission_revision BIGINT;
    current_authority TEXT;
    current_revision BIGINT;
    current_archive_sha256 TEXT;
    current_signed_identity_sha256 TEXT;
    current_policy_version BIGINT;
    current_policy_status TEXT;
BEGIN
    SELECT presentation.marketplace_release_id,
           presentation.admission_revision,
           session.authority,
           session.revision,
           release.archive_sha256,
           release.signed_identity_sha256,
           release.policy_version,
           release.policy_status
    INTO current_release_id,
         current_admission_revision,
         current_authority,
         current_revision,
         current_archive_sha256,
         current_signed_identity_sha256,
         current_policy_version,
         current_policy_status
    FROM game_session_cartridge_presentations AS presentation
    JOIN game_sessions AS session ON session.id = presentation.game_session_id
    JOIN marketplace_releases AS release ON release.id = presentation.marketplace_release_id
    WHERE presentation.game_session_id = NEW.game_session_id
      AND EXISTS (
          SELECT 1
          FROM game_session_participants AS participant
          WHERE participant.game_session_id = NEW.game_session_id
            AND participant.persona_id = NEW.actor_persona_id
      );

    IF NOT FOUND
        OR current_release_id IS DISTINCT FROM NEW.marketplace_release_id
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

CREATE TRIGGER game_session_cartridge_action_admission_validate_trigger
BEFORE INSERT ON game_session_cartridge_action_admissions
FOR EACH ROW EXECUTE FUNCTION game_session_cartridge_action_admission_validate();

CREATE FUNCTION game_session_cartridge_action_admission_immutable() RETURNS trigger
LANGUAGE plpgsql AS $$
BEGIN
    RAISE EXCEPTION 'game session cartridge action admissions are immutable';
END;
$$;

CREATE TRIGGER game_session_cartridge_action_admission_immutable_trigger
BEFORE UPDATE OR DELETE ON game_session_cartridge_action_admissions
FOR EACH ROW EXECUTE FUNCTION game_session_cartridge_action_admission_immutable();

CREATE TRIGGER game_session_cartridge_action_admission_no_truncate_trigger
BEFORE TRUNCATE ON game_session_cartridge_action_admissions
FOR EACH STATEMENT EXECUTE FUNCTION game_session_cartridge_action_admission_immutable();

CREATE FUNCTION bound_game_session_identity_immutable() RETURNS trigger
LANGUAGE plpgsql AS $$
BEGIN
    IF (NEW.game_key,
        NEW.game_version,
        NEW.authority,
        NEW.provider_release_id)
       IS DISTINCT FROM
       (OLD.game_key,
        OLD.game_version,
        OLD.authority,
        OLD.provider_release_id)
       AND EXISTS (
           SELECT 1
           FROM game_session_cartridge_presentations
           WHERE game_session_id = OLD.id
       )
    THEN
        RAISE EXCEPTION 'a cartridge-bound game session has immutable authority identity';
    END IF;
    RETURN NEW;
END;
$$;

CREATE TRIGGER bound_game_session_identity_immutable_trigger
BEFORE UPDATE OF game_key, game_version, authority, provider_release_id ON game_sessions
FOR EACH ROW EXECUTE FUNCTION bound_game_session_identity_immutable();

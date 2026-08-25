ALTER TABLE game_sessions
    DROP CONSTRAINT game_sessions_object_state,
    ADD COLUMN authority TEXT NOT NULL DEFAULT 'platform_compiled',
    ADD COLUMN provider_release_id UUID
        REFERENCES provider_releases(release_id) ON DELETE RESTRICT,
    ADD COLUMN provider_availability TEXT,
    ALTER COLUMN state DROP NOT NULL,
    ADD CONSTRAINT game_sessions_known_authority
        CHECK (authority IN ('platform_compiled', 'registered_provider')),
    ADD CONSTRAINT game_sessions_known_provider_availability
        CHECK (
            provider_availability IS NULL
            OR provider_availability IN (
                'provisioning',
                'ready',
                'reconciling',
                'unavailable',
                'suspended',
                'retired'
            )
        ),
    ADD CONSTRAINT game_sessions_id_provider_release_unique
        UNIQUE (id, provider_release_id),
    ADD CONSTRAINT game_sessions_single_authority CHECK (
        (authority = 'platform_compiled'
            AND provider_release_id IS NULL
            AND provider_availability IS NULL
            AND state IS NOT NULL
            AND jsonb_typeof(state) = 'object')
        OR
        (authority = 'registered_provider'
            AND provider_release_id IS NOT NULL
            AND provider_availability IS NOT NULL
            AND state IS NULL)
    );

ALTER TABLE game_sessions
    ALTER COLUMN authority DROP DEFAULT;

CREATE INDEX game_sessions_provider_release_idx
    ON game_sessions(provider_release_id, status, provider_availability, id)
    WHERE provider_release_id IS NOT NULL;

CREATE TABLE provider_game_pilots (
    release_id UUID PRIMARY KEY REFERENCES provider_releases(release_id) ON DELETE RESTRICT,
    display_name TEXT NOT NULL,
    min_human_players SMALLINT NOT NULL,
    max_human_players SMALLINT NOT NULL,
    status TEXT NOT NULL DEFAULT 'active',
    activated_at TIMESTAMPTZ NOT NULL DEFAULT clock_timestamp(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT clock_timestamp(),
    retired_at TIMESTAMPTZ,
    CONSTRAINT provider_game_pilots_display_name CHECK (
        char_length(display_name) BETWEEN 1 AND 64
        AND display_name !~ '[[:cntrl:]]'
    ),
    CONSTRAINT provider_game_pilots_player_bounds CHECK (
        min_human_players BETWEEN 1 AND 8
        AND max_human_players BETWEEN min_human_players AND 8
    ),
    CONSTRAINT provider_game_pilots_known_status
        CHECK (status IN ('active', 'suspended', 'retired')),
    CONSTRAINT provider_game_pilots_retirement_shape CHECK (
        (status = 'retired' AND retired_at IS NOT NULL)
        OR (status <> 'retired' AND retired_at IS NULL)
    ),
    CONSTRAINT provider_game_pilots_timestamp_order CHECK (
        updated_at >= activated_at
        AND (retired_at IS NULL OR retired_at >= activated_at)
    )
);

CREATE UNIQUE INDEX provider_game_pilots_single_active_idx
    ON provider_game_pilots ((status))
    WHERE status = 'active';

CREATE TABLE provider_game_session_views (
    game_session_id UUID PRIMARY KEY REFERENCES game_sessions(id) ON DELETE CASCADE,
    release_id UUID NOT NULL REFERENCES provider_releases(release_id) ON DELETE RESTRICT,
    provider_revision BIGINT NOT NULL,
    authenticated_sha256 TEXT NOT NULL,
    view JSONB NOT NULL,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT clock_timestamp(),
    CONSTRAINT provider_game_session_views_authority_fk
        FOREIGN KEY (game_session_id, release_id)
        REFERENCES game_sessions(id, provider_release_id)
        ON DELETE CASCADE,
    CONSTRAINT provider_game_session_views_nonnegative_revision
        CHECK (provider_revision >= 0),
    CONSTRAINT provider_game_session_views_digest CHECK (
        octet_length(authenticated_sha256) = 64
        AND authenticated_sha256 = lower(authenticated_sha256)
        AND authenticated_sha256 ~ '^[0-9a-f]+$'
    ),
    CONSTRAINT provider_game_session_views_object CHECK (
        jsonb_typeof(view) = 'object'
        AND octet_length(view::text) <= 65536
    )
);

CREATE TABLE provider_game_results (
    game_session_id UUID PRIMARY KEY REFERENCES game_sessions(id) ON DELETE RESTRICT,
    release_id UUID NOT NULL REFERENCES provider_releases(release_id) ON DELETE RESTRICT,
    provider_revision BIGINT NOT NULL,
    outcome TEXT NOT NULL,
    public_summary JSONB NOT NULL DEFAULT '{}'::jsonb,
    projected_at TIMESTAMPTZ NOT NULL DEFAULT clock_timestamp(),
    CONSTRAINT provider_game_results_authority_fk
        FOREIGN KEY (game_session_id, release_id)
        REFERENCES game_sessions(id, provider_release_id)
        ON DELETE RESTRICT,
    CONSTRAINT provider_game_results_positive_revision CHECK (provider_revision > 0),
    CONSTRAINT provider_game_results_canonical_outcome CHECK (
        octet_length(outcome) BETWEEN 2 AND 32
        AND outcome = lower(outcome)
        AND outcome ~ '^[a-z0-9][a-z0-9_-]*$'
    ),
    CONSTRAINT provider_game_results_public_summary CHECK (
        jsonb_typeof(public_summary) = 'object'
        AND octet_length(public_summary::text) <= 8192
    )
);

CREATE TABLE provider_achievement_definitions (
    release_id UUID NOT NULL REFERENCES provider_releases(release_id) ON DELETE RESTRICT,
    achievement_key TEXT NOT NULL,
    display_name TEXT NOT NULL,
    description TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT clock_timestamp(),
    PRIMARY KEY (release_id, achievement_key),
    CONSTRAINT provider_achievement_definitions_canonical_key CHECK (
        octet_length(achievement_key) BETWEEN 2 AND 48
        AND achievement_key = lower(achievement_key)
        AND achievement_key ~ '^[a-z0-9][a-z0-9_-]*$'
    ),
    CONSTRAINT provider_achievement_definitions_display_name CHECK (
        char_length(display_name) BETWEEN 1 AND 96
        AND display_name !~ '[[:cntrl:]]'
    ),
    CONSTRAINT provider_achievement_definitions_description CHECK (
        char_length(description) BETWEEN 1 AND 256
        AND description !~ '[[:cntrl:]]'
    )
);

CREATE TABLE persona_provider_achievements (
    persona_id UUID NOT NULL REFERENCES personas(id) ON DELETE RESTRICT,
    release_id UUID NOT NULL,
    achievement_key TEXT NOT NULL,
    game_session_id UUID NOT NULL REFERENCES game_sessions(id) ON DELETE RESTRICT,
    provider_revision BIGINT NOT NULL,
    awarded_at TIMESTAMPTZ NOT NULL DEFAULT clock_timestamp(),
    PRIMARY KEY (persona_id, release_id, achievement_key),
    CONSTRAINT persona_provider_achievements_definition_fk
        FOREIGN KEY (release_id, achievement_key)
        REFERENCES provider_achievement_definitions(release_id, achievement_key)
        ON DELETE RESTRICT,
    CONSTRAINT persona_provider_achievements_participant_fk
        FOREIGN KEY (game_session_id, persona_id)
        REFERENCES game_session_participants(game_session_id, persona_id)
        ON DELETE RESTRICT,
    CONSTRAINT persona_provider_achievements_positive_revision
        CHECK (provider_revision > 0)
);

CREATE INDEX persona_provider_achievements_persona_awarded_idx
    ON persona_provider_achievements(persona_id, awarded_at DESC, release_id, achievement_key);

CREATE FUNCTION provider_game_pilot_guard() RETURNS trigger
LANGUAGE plpgsql AS $$
BEGIN
    IF NEW.release_id IS DISTINCT FROM OLD.release_id
        OR NEW.display_name IS DISTINCT FROM OLD.display_name
        OR NEW.min_human_players IS DISTINCT FROM OLD.min_human_players
        OR NEW.max_human_players IS DISTINCT FROM OLD.max_human_players
        OR NEW.activated_at IS DISTINCT FROM OLD.activated_at
    THEN
        RAISE EXCEPTION 'provider pilot identity is immutable';
    END IF;
    IF OLD.status = 'retired' AND NEW.status <> 'retired' THEN
        RAISE EXCEPTION 'retired provider pilot is terminal';
    END IF;
    RETURN NEW;
END;
$$;

CREATE TRIGGER provider_game_pilot_guard_trigger
BEFORE UPDATE ON provider_game_pilots
FOR EACH ROW EXECUTE FUNCTION provider_game_pilot_guard();

CREATE TRIGGER provider_game_pilots_forbid_delete
BEFORE DELETE ON provider_game_pilots
FOR EACH ROW EXECUTE FUNCTION provider_forbid_delete();

CREATE TRIGGER provider_achievement_definitions_forbid_delete
BEFORE DELETE ON provider_achievement_definitions
FOR EACH ROW EXECUTE FUNCTION provider_forbid_delete();

CREATE FUNCTION provider_projection_immutable() RETURNS trigger
LANGUAGE plpgsql AS $$
BEGIN
    RAISE EXCEPTION 'provider result and achievement projections are append-only';
END;
$$;

CREATE TRIGGER provider_game_results_immutable_trigger
BEFORE UPDATE OR DELETE ON provider_game_results
FOR EACH ROW EXECUTE FUNCTION provider_projection_immutable();

CREATE TRIGGER persona_provider_achievements_immutable_trigger
BEFORE UPDATE OR DELETE ON persona_provider_achievements
FOR EACH ROW EXECUTE FUNCTION provider_projection_immutable();

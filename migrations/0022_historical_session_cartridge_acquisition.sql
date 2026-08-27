CREATE TABLE marketplace_snapshot_acquisition_evidence (
    snapshot_sha256 TEXT PRIMARY KEY
        CHECK (snapshot_sha256 ~ '^[0-9a-f]{64}$'),
    snapshot_version BIGINT NOT NULL CHECK (snapshot_version > 0),
    marketplace_key JSONB NOT NULL CHECK (jsonb_typeof(marketplace_key) = 'object'),
    signed_snapshot BYTEA NOT NULL
        CHECK (octet_length(signed_snapshot) BETWEEN 1 AND 1048576),
    created_at TIMESTAMPTZ NOT NULL DEFAULT clock_timestamp()
);

CREATE TABLE marketplace_release_acquisition_evidence (
    marketplace_release_id UUID PRIMARY KEY
        REFERENCES marketplace_releases(id) ON DELETE RESTRICT,
    snapshot_sha256 TEXT NOT NULL
        REFERENCES marketplace_snapshot_acquisition_evidence(snapshot_sha256)
        ON DELETE RESTRICT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT clock_timestamp()
);

CREATE INDEX marketplace_release_acquisition_snapshot_idx
    ON marketplace_release_acquisition_evidence(snapshot_sha256, marketplace_release_id);

CREATE FUNCTION marketplace_acquisition_evidence_immutable() RETURNS trigger
LANGUAGE plpgsql AS $$
BEGIN
    RAISE EXCEPTION 'marketplace acquisition evidence is immutable';
END;
$$;

CREATE TRIGGER marketplace_snapshot_acquisition_evidence_immutable_trigger
BEFORE UPDATE OR DELETE ON marketplace_snapshot_acquisition_evidence
FOR EACH ROW EXECUTE FUNCTION marketplace_acquisition_evidence_immutable();

CREATE TRIGGER marketplace_snapshot_acquisition_evidence_no_truncate_trigger
BEFORE TRUNCATE ON marketplace_snapshot_acquisition_evidence
FOR EACH STATEMENT EXECUTE FUNCTION marketplace_acquisition_evidence_immutable();

CREATE TRIGGER marketplace_release_acquisition_evidence_immutable_trigger
BEFORE UPDATE OR DELETE ON marketplace_release_acquisition_evidence
FOR EACH ROW EXECUTE FUNCTION marketplace_acquisition_evidence_immutable();

CREATE TRIGGER marketplace_release_acquisition_evidence_no_truncate_trigger
BEFORE TRUNCATE ON marketplace_release_acquisition_evidence
FOR EACH STATEMENT EXECUTE FUNCTION marketplace_acquisition_evidence_immutable();

INSERT INTO marketplace_snapshot_acquisition_evidence (
    snapshot_sha256,
    snapshot_version,
    marketplace_key,
    signed_snapshot
)
SELECT snapshot_sha256,
       snapshot_version,
       marketplace_key,
       signed_snapshot
FROM marketplace_sync_state
WHERE singleton
  AND marketplace_key IS NOT NULL
  AND signed_snapshot IS NOT NULL;

INSERT INTO marketplace_release_acquisition_evidence (
    marketplace_release_id,
    snapshot_sha256
)
SELECT release.id,
       sync.snapshot_sha256
FROM marketplace_releases AS release
JOIN marketplace_sync_state AS sync
  ON sync.singleton
 AND release.last_seen_snapshot_version = sync.snapshot_version
JOIN marketplace_snapshot_acquisition_evidence AS evidence
  ON evidence.snapshot_sha256 = sync.snapshot_sha256;

CREATE FUNCTION game_session_cartridge_presentation_evidence_validate() RETURNS trigger
LANGUAGE plpgsql AS $$
BEGIN
    IF NOT EXISTS (
        SELECT 1
        FROM marketplace_release_acquisition_evidence
        WHERE marketplace_release_id = NEW.marketplace_release_id
    ) THEN
        RAISE EXCEPTION 'game session cartridge presentation requires retained acquisition evidence';
    END IF;
    RETURN NEW;
END;
$$;

CREATE TRIGGER game_session_cartridge_presentation_evidence_validate_trigger
BEFORE INSERT ON game_session_cartridge_presentations
FOR EACH ROW EXECUTE FUNCTION game_session_cartridge_presentation_evidence_validate();

ALTER TABLE game_session_cartridge_action_admissions
    ADD COLUMN screen_id TEXT
        CHECK (screen_id IS NULL OR screen_id ~ '^[a-z][a-z0-9._-]{0,95}$'),
    ADD COLUMN screen_explicit BOOLEAN,
    ADD CONSTRAINT game_session_cartridge_action_screen_shape CHECK (
        (screen_id IS NULL) = (screen_explicit IS NULL)
    );

CREATE FUNCTION game_session_cartridge_action_screen_validate() RETURNS trigger
LANGUAGE plpgsql AS $$
BEGIN
    IF NEW.screen_id IS NULL OR NEW.screen_explicit IS NULL THEN
        RAISE EXCEPTION 'new cartridge action admissions require an exact screen';
    END IF;
    RETURN NEW;
END;
$$;

CREATE TRIGGER game_session_cartridge_action_screen_validate_trigger
BEFORE INSERT ON game_session_cartridge_action_admissions
FOR EACH ROW EXECUTE FUNCTION game_session_cartridge_action_screen_validate();

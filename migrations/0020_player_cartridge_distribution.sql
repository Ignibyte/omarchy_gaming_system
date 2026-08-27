ALTER TABLE marketplace_sync_state
    ADD COLUMN signed_snapshot BYTEA
        CHECK (signed_snapshot IS NULL OR (
            octet_length(signed_snapshot) BETWEEN 1 AND 1048576
        )),
    ADD COLUMN marketplace_key JSONB
        CHECK (marketplace_key IS NULL OR jsonb_typeof(marketplace_key) = 'object'),
    ADD CONSTRAINT marketplace_sync_distribution_evidence_pair CHECK (
        (signed_snapshot IS NULL) = (marketplace_key IS NULL)
    );

CREATE FUNCTION marketplace_sync_state_guard() RETURNS trigger
LANGUAGE plpgsql AS $$
BEGIN
    IF TG_OP = 'DELETE' THEN
        RAISE EXCEPTION 'marketplace sync state cannot be deleted';
    END IF;
    IF NEW.marketplace_origin <> OLD.marketplace_origin
        OR NEW.authority_id <> OLD.authority_id
        OR NEW.key_id <> OLD.key_id
        OR NEW.snapshot_version < OLD.snapshot_version
        OR (NEW.snapshot_version = OLD.snapshot_version
            AND NEW.snapshot_sha256 <> OLD.snapshot_sha256)
        OR (OLD.signed_snapshot IS NOT NULL AND NEW.signed_snapshot IS NULL)
        OR (NEW.snapshot_version = OLD.snapshot_version
            AND OLD.signed_snapshot IS NOT NULL
            AND (
                NEW.signed_snapshot <> OLD.signed_snapshot
                OR NEW.marketplace_key <> OLD.marketplace_key
            ))
    THEN
        RAISE EXCEPTION 'marketplace sync identity or evidence cannot regress';
    END IF;
    RETURN NEW;
END;
$$;

CREATE TRIGGER marketplace_sync_state_guard_trigger
BEFORE UPDATE OR DELETE ON marketplace_sync_state
FOR EACH ROW EXECUTE FUNCTION marketplace_sync_state_guard();

CREATE TRIGGER marketplace_sync_state_no_truncate_trigger
BEFORE TRUNCATE ON marketplace_sync_state
FOR EACH STATEMENT EXECUTE FUNCTION marketplace_release_no_truncate();

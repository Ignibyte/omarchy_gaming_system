ALTER TABLE marketplace_releases
    ADD COLUMN policy_marketplace_key JSONB
        CHECK (policy_marketplace_key IS NULL
            OR jsonb_typeof(policy_marketplace_key) = 'object'),
    ADD COLUMN policy_snapshot_version BIGINT
        CHECK (policy_snapshot_version IS NULL OR policy_snapshot_version > 0),
    ADD CONSTRAINT marketplace_release_policy_evidence_pair CHECK (
        (policy_marketplace_key IS NULL) = (policy_snapshot_version IS NULL)
    ),
    ADD CONSTRAINT marketplace_release_policy_snapshot_seen CHECK (
        policy_snapshot_version IS NULL
        OR policy_snapshot_version <= last_seen_snapshot_version
    );

UPDATE marketplace_releases AS release
SET policy_marketplace_key = sync.marketplace_key,
    policy_snapshot_version = release.last_seen_snapshot_version
FROM marketplace_sync_state AS sync
WHERE sync.singleton
  AND sync.marketplace_key IS NOT NULL;

ALTER TABLE marketplace_sync_state
    ADD COLUMN trust_root_sha256 TEXT
        CHECK (trust_root_sha256 IS NULL
            OR trust_root_sha256 ~ '^[0-9a-f]{64}$'),
    ADD COLUMN trust_payload JSONB
        CHECK (trust_payload IS NULL OR (
            jsonb_typeof(trust_payload) = 'object'
            AND jsonb_typeof(trust_payload -> 'bundle_version') = 'number'
            AND (trust_payload ->> 'bundle_version')::BIGINT > 0
        )),
    ADD CONSTRAINT marketplace_sync_trust_evidence_pair CHECK (
        (trust_root_sha256 IS NULL) = (trust_payload IS NULL)
    ),
    ADD CONSTRAINT marketplace_sync_key_identity CHECK (
        marketplace_key IS NULL OR (
            marketplace_key ->> 'authority_id' = authority_id
            AND marketplace_key ->> 'key_id' = key_id
        )
    );

CREATE OR REPLACE FUNCTION marketplace_sync_state_guard() RETURNS trigger
LANGUAGE plpgsql AS $$
BEGIN
    IF TG_OP = 'DELETE' THEN
        RAISE EXCEPTION 'marketplace sync state cannot be deleted';
    END IF;
    IF NEW.marketplace_origin <> OLD.marketplace_origin
        OR NEW.authority_id <> OLD.authority_id
        OR NEW.snapshot_version < OLD.snapshot_version
        OR (NEW.snapshot_version = OLD.snapshot_version
            AND (
                NEW.key_id <> OLD.key_id
                OR NEW.snapshot_sha256 <> OLD.snapshot_sha256
            ))
        OR (OLD.signed_snapshot IS NOT NULL AND NEW.signed_snapshot IS NULL)
        OR (NEW.snapshot_version = OLD.snapshot_version
            AND OLD.signed_snapshot IS NOT NULL
            AND (
                NEW.signed_snapshot <> OLD.signed_snapshot
                OR NEW.marketplace_key <> OLD.marketplace_key
            ))
        OR (OLD.trust_payload IS NOT NULL AND NEW.trust_payload IS NULL)
        OR (OLD.trust_payload IS NOT NULL
            AND NEW.trust_root_sha256 IS DISTINCT FROM OLD.trust_root_sha256)
        OR (OLD.trust_payload IS NOT NULL
            AND NEW.trust_payload IS NOT NULL
            AND (NEW.trust_payload ->> 'bundle_version')::BIGINT
                < (OLD.trust_payload ->> 'bundle_version')::BIGINT)
        OR (OLD.trust_payload IS NOT NULL
            AND NEW.trust_payload IS NOT NULL
            AND (NEW.trust_payload ->> 'bundle_version')::BIGINT
                = (OLD.trust_payload ->> 'bundle_version')::BIGINT
            AND NEW.trust_payload <> OLD.trust_payload)
    THEN
        RAISE EXCEPTION 'marketplace sync identity or evidence cannot regress';
    END IF;
    RETURN NEW;
END;
$$;

CREATE OR REPLACE FUNCTION marketplace_release_guard() RETURNS trigger
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
        OR (NEW.policy_version = OLD.policy_version
            AND (
                NEW.signed_policy <> OLD.signed_policy
                OR NEW.policy_marketplace_key
                    IS DISTINCT FROM OLD.policy_marketplace_key
            ))
        OR (OLD.policy_snapshot_version IS NOT NULL
            AND NEW.policy_snapshot_version IS NULL)
        OR (OLD.policy_snapshot_version IS NOT NULL
            AND NEW.policy_snapshot_version < OLD.policy_snapshot_version)
        OR (NEW.policy_snapshot_version IS NOT DISTINCT FROM OLD.policy_snapshot_version
            AND (
                NEW.policy_marketplace_key IS DISTINCT FROM OLD.policy_marketplace_key
                OR (
                    NEW.signed_policy <> OLD.signed_policy
                    AND NOT (
                        OLD.policy_snapshot_version IS NULL
                        AND NEW.policy_snapshot_version IS NULL
                        AND NEW.policy_version > OLD.policy_version
                    )
                )
            ))
    THEN
        RAISE EXCEPTION 'marketplace release identity or monotonic state cannot regress';
    END IF;
    RETURN NEW;
END;
$$;

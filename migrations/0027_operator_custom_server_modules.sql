-- Generalize the fixed reviewed observation-module tables for a bounded,
-- database-custodied operator-custom module lifecycle. Existing Ticket 040
-- evidence remains valid and immutable.

ALTER TABLE server_module_releases
    DROP CONSTRAINT server_module_releases_exact_fixture,
    ALTER COLUMN review_id DROP NOT NULL,
    ADD COLUMN component_bytes BYTEA,
    ADD COLUMN artifact_custody TEXT NOT NULL DEFAULT 'packaged_reviewed_fixture',
    ADD COLUMN publisher_key_id TEXT,
    ADD COLUMN publisher_public_key TEXT,
    ADD COLUMN publisher_key_sha256 TEXT,
    ADD COLUMN provenance_key_id TEXT,
    ADD COLUMN provenance_public_key TEXT,
    ADD COLUMN provenance_key_sha256 TEXT,
    ADD COLUMN provenance_server_id UUID REFERENCES server_identity(id) ON DELETE RESTRICT,
    ADD CONSTRAINT server_module_releases_general_contract CHECK (
        module_id ~ '^[a-z][a-z0-9._-]{0,95}$'
        AND publisher_id ~ '^[a-z][a-z0-9._-]{0,95}$'
        AND char_length(version) BETWEEN 1 AND 64
        AND version !~ '[[:cntrl:]]'
        AND release_format = 'omarchygs.server-module-release/v1'
        AND provenance_class IN (
            'first_party_reviewed_fixture', 'marketplace_vetted', 'operator_custom'
        )
        AND wit_package = 'ignibyte:omarchygs-server-module@1.0.0'
        AND wit_world = 'module-production'
        AND wit_major = 1
        AND requested_capabilities <@ ARRAY['moderation_add_label']::TEXT[]
        AND cardinality(requested_capabilities) <= 1
        AND subscribed_hooks = ARRAY['persona_reported']::TEXT[]
        AND config_schema ~ '^[a-z][a-z0-9._/-]{0,127}$'
        AND state_schema ~ '^[a-z][a-z0-9._/-]{0,127}$'
    ),
    ADD CONSTRAINT server_module_releases_custody_contract CHECK (
        (
            artifact_custody = 'packaged_reviewed_fixture'
            AND provenance_class = 'first_party_reviewed_fixture'
            AND review_id IS NOT NULL
            AND component_bytes IS NULL
            AND publisher_key_id IS NULL
            AND publisher_public_key IS NULL
            AND publisher_key_sha256 IS NULL
            AND provenance_key_id IS NULL
            AND provenance_public_key IS NULL
            AND provenance_key_sha256 IS NULL
            AND provenance_server_id IS NULL
        )
        OR
        (
            artifact_custody = 'database_immutable'
            AND provenance_class IN ('marketplace_vetted', 'operator_custom')
            AND component_bytes IS NOT NULL
            AND octet_length(component_bytes) BETWEEN 8 AND 2097152
            AND publisher_key_id ~ '^[a-z][a-z0-9._-]{0,95}$'
            AND publisher_public_key ~ '^[A-Za-z0-9_-]{43}$'
            AND publisher_key_sha256 ~ '^[0-9a-f]{64}$'
            AND provenance_key_id ~ '^[a-z][a-z0-9._-]{0,95}$'
            AND provenance_public_key ~ '^[A-Za-z0-9_-]{43}$'
            AND provenance_key_sha256 ~ '^[0-9a-f]{64}$'
            AND (
                (
                    provenance_class = 'marketplace_vetted'
                    AND review_id IS NOT NULL
                    AND provenance_server_id IS NULL
                )
                OR
                (
                    provenance_class = 'operator_custom'
                    AND review_id IS NULL
                    AND provenance_server_id IS NOT NULL
                )
            )
        )
    );

ALTER TABLE server_module_admissions
    DROP CONSTRAINT server_module_admissions_exact_grant,
    ADD CONSTRAINT server_module_admissions_general_grant CHECK (
        admission_format = 'omarchygs.server-module-admission/v1'
        AND lifecycle = 'active'
        AND granted_capabilities <@ ARRAY['moderation_add_label']::TEXT[]
        AND cardinality(granted_capabilities) <= 1
        AND subscribed_hooks = ARRAY['persona_reported']::TEXT[]
        AND state_schema ~ '^[a-z][a-z0-9._/-]{0,127}$'
    );

ALTER TABLE server_module_instances
    DROP CONSTRAINT server_module_instances_exact_identity,
    ADD COLUMN previous_release_id UUID
        REFERENCES server_module_releases(release_id) ON DELETE RESTRICT,
    ADD COLUMN rollback_snapshot_id UUID
        REFERENCES server_module_state_snapshots(snapshot_id) ON DELETE RESTRICT,
    ADD COLUMN state_disposition TEXT NOT NULL DEFAULT 'live',
    ADD CONSTRAINT server_module_instances_general_identity CHECK (
        module_id ~ '^[a-z][a-z0-9._-]{0,95}$'
        AND state_schema ~ '^[a-z][a-z0-9._/-]{0,127}$'
        AND state_disposition IN ('live', 'retain_for_audit')
    ),
    ADD CONSTRAINT server_module_instances_rollback_shape CHECK (
        (previous_release_id IS NULL AND rollback_snapshot_id IS NULL)
        OR (previous_release_id IS NOT NULL AND rollback_snapshot_id IS NOT NULL)
    );

ALTER TABLE server_module_state_namespaces
    DROP CONSTRAINT server_module_state_namespaces_exact_schema,
    ADD CONSTRAINT server_module_state_namespaces_general_schema CHECK (
        state_schema ~ '^[a-z][a-z0-9._/-]{0,127}$'
    );

ALTER TABLE server_module_lifecycle_audit
    DROP CONSTRAINT server_module_lifecycle_audit_action,
    ADD CONSTRAINT server_module_lifecycle_audit_action CHECK (
        action IN (
            'register', 'import', 'enable', 'disable', 'suspend', 'degrade',
            'recover', 'restore', 'retire', 'upgrade', 'rollback', 'remove'
        )
    );

ALTER TABLE server_module_instances
    DROP CONSTRAINT server_module_instances_observation_gap_bounds,
    ADD CONSTRAINT server_module_instances_observation_gap_bounds CHECK (
        observation_gap_count >= 0
        AND (
            (observation_gap_count = 0
                AND last_observation_gap_reason IS NULL
                AND last_observation_gap_at IS NULL)
            OR
            (observation_gap_count > 0
                AND last_observation_gap_reason IN (
                    'module_inactive', 'queue_saturated', 'runtime_unconfigured',
                    'admission_replaced', 'module_removed'
                )
                AND last_observation_gap_at IS NOT NULL)
        )
    );

CREATE TABLE server_module_custom_operations (
    operation_id UUID PRIMARY KEY,
    action TEXT NOT NULL,
    command_sha256 TEXT NOT NULL,
    instance_id UUID NOT NULL
        REFERENCES server_module_instances(instance_id) ON DELETE RESTRICT,
    release_id UUID NOT NULL
        REFERENCES server_module_releases(release_id) ON DELETE RESTRICT,
    publisher_key_sha256 TEXT NOT NULL,
    provenance_key_sha256 TEXT NOT NULL,
    requested_capabilities TEXT[] NOT NULL,
    granted_capabilities TEXT[] NOT NULL,
    acknowledgement TEXT NOT NULL,
    expected_lifecycle_revision BIGINT NOT NULL,
    resulting_lifecycle TEXT NOT NULL,
    resulting_lifecycle_revision BIGINT NOT NULL,
    resulting_state_revision BIGINT NOT NULL,
    actor TEXT NOT NULL,
    reason TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT clock_timestamp(),
    CONSTRAINT server_module_custom_operations_action CHECK (
        action IN (
            'import', 'enable', 'disable', 'suspend', 'recover',
            'upgrade', 'rollback', 'remove'
        )
    ),
    CONSTRAINT server_module_custom_operations_digest_shape CHECK (
        command_sha256 ~ '^[0-9a-f]{64}$'
        AND publisher_key_sha256 ~ '^[0-9a-f]{64}$'
        AND provenance_key_sha256 ~ '^[0-9a-f]{64}$'
    ),
    CONSTRAINT server_module_custom_operations_grant_shape CHECK (
        requested_capabilities <@ ARRAY['moderation_add_label']::TEXT[]
        AND granted_capabilities <@ requested_capabilities
        AND cardinality(requested_capabilities) <= 1
        AND cardinality(granted_capabilities) <= 1
    ),
    CONSTRAINT server_module_custom_operations_acknowledgement CHECK (
        acknowledgement =
            'I understand this module is unreviewed and unsupported by OmarchyGS.'
    ),
    CONSTRAINT server_module_custom_operations_result CHECK (
        expected_lifecycle_revision >= 0
        AND resulting_lifecycle IN (
            'disabled', 'enabling', 'active', 'degraded', 'suspended', 'retired'
        )
        AND resulting_lifecycle_revision > 0
        AND resulting_state_revision >= 0
    ),
    CONSTRAINT server_module_custom_operations_text CHECK (
        char_length(actor) BETWEEN 1 AND 64
        AND actor !~ '[[:cntrl:]]'
        AND char_length(reason) BETWEEN 1 AND 500
        AND reason !~ '[[:cntrl:]]'
    )
);

CREATE INDEX server_module_custom_operations_instance_idx
    ON server_module_custom_operations(instance_id, created_at, operation_id);

CREATE FUNCTION server_module_reject_custom_operation_mutation() RETURNS trigger
LANGUAGE plpgsql AS $$
BEGIN
    RAISE EXCEPTION 'server module custom-operation evidence is immutable';
END;
$$;

CREATE TRIGGER server_module_custom_operations_immutable_rows
BEFORE UPDATE OR DELETE ON server_module_custom_operations
FOR EACH ROW EXECUTE FUNCTION server_module_reject_custom_operation_mutation();

CREATE TRIGGER server_module_custom_operations_immutable_truncate
BEFORE TRUNCATE ON server_module_custom_operations
FOR EACH STATEMENT EXECUTE FUNCTION server_module_reject_custom_operation_mutation();

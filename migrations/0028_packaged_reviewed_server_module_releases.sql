-- Retain exact idempotency and transition evidence for administrator-selected
-- packaged first-party reviewed release upgrades and one-step rollbacks.

CREATE TABLE server_module_reviewed_operations (
    operation_id UUID PRIMARY KEY,
    action TEXT NOT NULL,
    command_sha256 TEXT NOT NULL,
    instance_id UUID NOT NULL
        REFERENCES server_module_instances(instance_id) ON DELETE RESTRICT,
    previous_release_id UUID NOT NULL
        REFERENCES server_module_releases(release_id) ON DELETE RESTRICT,
    release_id UUID NOT NULL
        REFERENCES server_module_releases(release_id) ON DELETE RESTRICT,
    predecessor_snapshot_id UUID NOT NULL
        REFERENCES server_module_state_snapshots(snapshot_id) ON DELETE RESTRICT,
    expected_lifecycle_revision BIGINT NOT NULL,
    expected_config_revision BIGINT NOT NULL,
    expected_state_revision BIGINT NOT NULL,
    resulting_lifecycle TEXT NOT NULL,
    resulting_lifecycle_revision BIGINT NOT NULL,
    resulting_state_schema TEXT NOT NULL,
    resulting_state_revision BIGINT NOT NULL,
    actor TEXT NOT NULL,
    reason TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT clock_timestamp(),
    UNIQUE (instance_id, resulting_lifecycle_revision),
    CONSTRAINT server_module_reviewed_operations_identity CHECK (
        instance_id = '12000000-0000-4000-8000-000000000001'::UUID
        AND previous_release_id <> release_id
    ),
    CONSTRAINT server_module_reviewed_operations_action CHECK (
        action IN ('upgrade', 'rollback')
    ),
    CONSTRAINT server_module_reviewed_operations_exact_edge CHECK (
        (
            action = 'upgrade'
            AND previous_release_id = '10000000-0000-4000-8000-000000000001'::UUID
            AND release_id = '10000000-0000-4000-8000-000000000002'::UUID
            AND resulting_state_schema = 'ignibyte.sentinel.state/v2'
        ) OR (
            action = 'rollback'
            AND previous_release_id = '10000000-0000-4000-8000-000000000002'::UUID
            AND release_id = '10000000-0000-4000-8000-000000000001'::UUID
            AND resulting_state_schema = 'ignibyte.sentinel.state/v1'
        )
    ),
    CONSTRAINT server_module_reviewed_operations_digest_shape CHECK (
        command_sha256 ~ '^[0-9a-f]{64}$'
    ),
    CONSTRAINT server_module_reviewed_operations_revisions CHECK (
        expected_lifecycle_revision > 0
        AND expected_config_revision > 0
        AND expected_state_revision >= 0
        AND resulting_lifecycle = 'active'
        AND resulting_lifecycle_revision = expected_lifecycle_revision + 1
        AND resulting_state_revision = expected_state_revision + 1
    ),
    CONSTRAINT server_module_reviewed_operations_schema CHECK (
        resulting_state_schema ~ '^[a-z][a-z0-9._/-]{0,127}$'
    ),
    CONSTRAINT server_module_reviewed_operations_text CHECK (
        char_length(actor) BETWEEN 1 AND 64
        AND actor !~ '[[:cntrl:]]'
        AND char_length(reason) BETWEEN 1 AND 500
        AND reason !~ '[[:cntrl:]]'
    )
);

CREATE INDEX server_module_reviewed_operations_instance_idx
    ON server_module_reviewed_operations(instance_id, created_at, operation_id);

CREATE TRIGGER server_module_reviewed_operations_immutable_rows
BEFORE UPDATE OR DELETE ON server_module_reviewed_operations
FOR EACH ROW EXECUTE FUNCTION server_module_reject_immutable_mutation();

CREATE TRIGGER server_module_reviewed_operations_immutable_truncate
BEFORE TRUNCATE ON server_module_reviewed_operations
FOR EACH STATEMENT EXECUTE FUNCTION server_module_reject_immutable_mutation();

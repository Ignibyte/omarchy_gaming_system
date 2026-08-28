CREATE TABLE server_module_releases (
    release_id UUID PRIMARY KEY,
    module_id TEXT NOT NULL,
    publisher_id TEXT NOT NULL,
    version TEXT NOT NULL,
    release_format TEXT NOT NULL,
    signed_release BYTEA NOT NULL,
    release_sha256 TEXT NOT NULL,
    signed_provenance BYTEA NOT NULL,
    provenance_sha256 TEXT NOT NULL,
    provenance_class TEXT NOT NULL,
    review_id UUID NOT NULL,
    component_sha256 TEXT NOT NULL,
    wit_package TEXT NOT NULL,
    wit_world TEXT NOT NULL,
    wit_major INTEGER NOT NULL,
    wit_sha256 TEXT NOT NULL,
    requested_capabilities TEXT[] NOT NULL,
    subscribed_hooks TEXT[] NOT NULL,
    frame_bytes INTEGER NOT NULL,
    memory_bytes INTEGER NOT NULL,
    fuel BIGINT NOT NULL,
    execution_ms INTEGER NOT NULL,
    config_schema TEXT NOT NULL,
    state_schema TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT clock_timestamp(),
    CONSTRAINT server_module_releases_exact_fixture CHECK (
        module_id = 'ignibyte.sentinel'
        AND publisher_id = 'ignibyte'
        AND version = '1.0.0'
        AND release_format = 'omarchygs.server-module-release/v1'
        AND provenance_class = 'first_party_reviewed_fixture'
        AND wit_package = 'ignibyte:omarchygs-server-module@1.0.0'
        AND wit_world = 'module-production'
        AND wit_major = 1
        AND requested_capabilities = ARRAY['moderation_add_label']::TEXT[]
        AND subscribed_hooks = ARRAY['persona_reported']::TEXT[]
        AND config_schema = 'ignibyte.sentinel.config/v1'
        AND state_schema = 'ignibyte.sentinel.state/v1'
    ),
    CONSTRAINT server_module_releases_signed_bounds CHECK (
        octet_length(signed_release) BETWEEN 1 AND 1048576
        AND octet_length(signed_provenance) BETWEEN 1 AND 1048576
    ),
    CONSTRAINT server_module_releases_digest_shape CHECK (
        release_sha256 ~ '^[0-9a-f]{64}$'
        AND provenance_sha256 ~ '^[0-9a-f]{64}$'
        AND component_sha256 ~ '^[0-9a-f]{64}$'
        AND wit_sha256 ~ '^[0-9a-f]{64}$'
    ),
    CONSTRAINT server_module_releases_budget_bounds CHECK (
        frame_bytes BETWEEN 1 AND 65536
        AND memory_bytes BETWEEN 1 AND 4194304
        AND fuel BETWEEN 1 AND 100000
        AND execution_ms BETWEEN 1 AND 500
    )
);

CREATE TABLE server_module_admissions (
    admission_id UUID NOT NULL,
    lifecycle_revision BIGINT NOT NULL,
    release_id UUID NOT NULL REFERENCES server_module_releases(release_id) ON DELETE RESTRICT,
    server_id UUID NOT NULL REFERENCES server_identity(id) ON DELETE RESTRICT,
    admission_format TEXT NOT NULL,
    signed_admission BYTEA NOT NULL,
    admission_sha256 TEXT NOT NULL,
    lifecycle TEXT NOT NULL,
    granted_capabilities TEXT[] NOT NULL,
    subscribed_hooks TEXT[] NOT NULL,
    config_revision BIGINT NOT NULL,
    state_schema TEXT NOT NULL,
    state_revision BIGINT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT clock_timestamp(),
    PRIMARY KEY (admission_id, lifecycle_revision),
    CONSTRAINT server_module_admissions_exact_grant CHECK (
        admission_format = 'omarchygs.server-module-admission/v1'
        AND lifecycle = 'active'
        AND granted_capabilities = ARRAY['moderation_add_label']::TEXT[]
        AND subscribed_hooks = ARRAY['persona_reported']::TEXT[]
        AND state_schema = 'ignibyte.sentinel.state/v1'
    ),
    CONSTRAINT server_module_admissions_positive_revisions CHECK (
        lifecycle_revision > 0
        AND config_revision > 0
        AND state_revision >= 0
    ),
    CONSTRAINT server_module_admissions_signed_bounds CHECK (
        octet_length(signed_admission) BETWEEN 1 AND 1048576
        AND admission_sha256 ~ '^[0-9a-f]{64}$'
    )
);

CREATE TABLE server_module_instances (
    instance_id UUID PRIMARY KEY,
    module_id TEXT NOT NULL UNIQUE,
    release_id UUID NOT NULL REFERENCES server_module_releases(release_id) ON DELETE RESTRICT,
    current_admission_id UUID,
    current_admission_revision BIGINT,
    lifecycle TEXT NOT NULL DEFAULT 'disabled',
    lifecycle_revision BIGINT NOT NULL DEFAULT 1,
    config JSONB NOT NULL DEFAULT '{"policy":"strict"}'::JSONB,
    config_revision BIGINT NOT NULL DEFAULT 1,
    state_schema TEXT NOT NULL DEFAULT 'ignibyte.sentinel.state/v1',
    state_revision BIGINT NOT NULL DEFAULT 0,
    consecutive_failures INTEGER NOT NULL DEFAULT 0,
    activation_allowed BOOLEAN NOT NULL DEFAULT TRUE,
    restored_pending_review BOOLEAN NOT NULL DEFAULT FALSE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT clock_timestamp(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT clock_timestamp(),
    CONSTRAINT server_module_instances_admission_fk
        FOREIGN KEY (current_admission_id, current_admission_revision)
        REFERENCES server_module_admissions(admission_id, lifecycle_revision)
        ON DELETE RESTRICT,
    CONSTRAINT server_module_instances_exact_identity CHECK (
        module_id = 'ignibyte.sentinel'
        AND state_schema = 'ignibyte.sentinel.state/v1'
    ),
    CONSTRAINT server_module_instances_known_lifecycle CHECK (
        lifecycle IN ('disabled', 'enabling', 'active', 'degraded', 'suspended', 'retired')
    ),
    CONSTRAINT server_module_instances_revision_bounds CHECK (
        lifecycle_revision > 0
        AND config_revision > 0
        AND state_revision >= 0
        AND consecutive_failures BETWEEN 0 AND 1000000
    ),
    CONSTRAINT server_module_instances_admission_shape CHECK (
        (lifecycle = 'active'
            AND current_admission_id IS NOT NULL
            AND current_admission_revision IS NOT NULL)
        OR lifecycle <> 'active'
    ),
    CONSTRAINT server_module_instances_config_shape CHECK (
        jsonb_typeof(config) = 'object'
        AND octet_length(config::TEXT) <= 4096
    ),
    CONSTRAINT server_module_instances_timestamp_order CHECK (updated_at >= created_at)
);

CREATE TABLE server_module_lifecycle_audit (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    operation_id UUID NOT NULL,
    instance_id UUID NOT NULL REFERENCES server_module_instances(instance_id) ON DELETE RESTRICT,
    action TEXT NOT NULL,
    expected_revision BIGINT NOT NULL,
    previous_state TEXT NOT NULL,
    resulting_state TEXT NOT NULL,
    resulting_revision BIGINT NOT NULL,
    actor TEXT NOT NULL,
    reason TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT clock_timestamp(),
    UNIQUE (instance_id, operation_id),
    CONSTRAINT server_module_lifecycle_audit_action CHECK (
        action IN ('register', 'enable', 'disable', 'suspend', 'degrade', 'recover', 'restore', 'retire')
    ),
    CONSTRAINT server_module_lifecycle_audit_states CHECK (
        previous_state IN ('absent', 'disabled', 'enabling', 'active', 'degraded', 'suspended', 'retired')
        AND resulting_state IN ('disabled', 'enabling', 'active', 'degraded', 'suspended', 'retired')
    ),
    CONSTRAINT server_module_lifecycle_audit_revision CHECK (
        expected_revision >= 0 AND resulting_revision > expected_revision
    ),
    CONSTRAINT server_module_lifecycle_audit_text CHECK (
        char_length(actor) BETWEEN 1 AND 64
        AND actor !~ '[[:cntrl:]]'
        AND char_length(reason) BETWEEN 1 AND 500
        AND reason !~ '[[:cntrl:]]'
    )
);

CREATE TABLE server_module_outbox (
    sequence BIGINT GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    event_id UUID NOT NULL UNIQUE,
    instance_id UUID NOT NULL REFERENCES server_module_instances(instance_id) ON DELETE RESTRICT,
    release_id UUID NOT NULL REFERENCES server_module_releases(release_id) ON DELETE RESTRICT,
    admission_id UUID NOT NULL,
    admission_revision BIGINT NOT NULL,
    hook TEXT NOT NULL,
    partition_subject TEXT NOT NULL,
    subject_persona_id UUID NOT NULL REFERENCES personas(id) ON DELETE RESTRICT,
    target_report_id UUID NOT NULL REFERENCES persona_reports(id) ON DELETE RESTRICT,
    causal_revision BIGINT NOT NULL,
    payload JSONB NOT NULL,
    payload_sha256 TEXT NOT NULL,
    config_snapshot JSONB NOT NULL,
    config_revision BIGINT NOT NULL,
    state_snapshot JSONB NOT NULL,
    state_revision BIGINT NOT NULL,
    status TEXT NOT NULL DEFAULT 'pending',
    attempt_count INTEGER NOT NULL DEFAULT 0,
    next_attempt_at TIMESTAMPTZ NOT NULL DEFAULT clock_timestamp(),
    lease_id UUID,
    lease_expires_at TIMESTAMPTZ,
    last_error_code TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT clock_timestamp(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT clock_timestamp(),
    delivered_at TIMESTAMPTZ,
    dead_lettered_at TIMESTAMPTZ,
    CONSTRAINT server_module_outbox_admission_fk
        FOREIGN KEY (admission_id, admission_revision)
        REFERENCES server_module_admissions(admission_id, lifecycle_revision)
        ON DELETE RESTRICT,
    CONSTRAINT server_module_outbox_exact_hook CHECK (hook = 'persona_reported'),
    CONSTRAINT server_module_outbox_subject_bound CHECK (
        octet_length(partition_subject) BETWEEN 16 AND 96
        AND partition_subject ~ '^[A-Za-z0-9_-]+$'
    ),
    CONSTRAINT server_module_outbox_revisions CHECK (
        admission_revision > 0
        AND causal_revision >= 0
        AND config_revision > 0
        AND state_revision >= 0
    ),
    CONSTRAINT server_module_outbox_payload_bounds CHECK (
        jsonb_typeof(payload) = 'object'
        AND jsonb_typeof(config_snapshot) = 'object'
        AND jsonb_typeof(state_snapshot) = 'object'
        AND octet_length(payload::TEXT) <= 4096
        AND octet_length(config_snapshot::TEXT) <= 4096
        AND octet_length(state_snapshot::TEXT) <= 4096
        AND payload_sha256 ~ '^[0-9a-f]{64}$'
    ),
    CONSTRAINT server_module_outbox_known_status CHECK (
        status IN ('pending', 'in_flight', 'retry', 'delivered', 'dead_letter')
    ),
    CONSTRAINT server_module_outbox_attempts CHECK (attempt_count BETWEEN 0 AND 8),
    CONSTRAINT server_module_outbox_error_code CHECK (
        last_error_code IS NULL
        OR (octet_length(last_error_code) BETWEEN 3 AND 64 AND last_error_code ~ '^[a-z0-9_]+$')
    ),
    CONSTRAINT server_module_outbox_lease_shape CHECK (
        (status = 'in_flight' AND lease_id IS NOT NULL AND lease_expires_at IS NOT NULL)
        OR (status <> 'in_flight' AND lease_id IS NULL AND lease_expires_at IS NULL)
    ),
    CONSTRAINT server_module_outbox_terminal_shape CHECK (
        (status = 'delivered' AND delivered_at IS NOT NULL AND dead_lettered_at IS NULL)
        OR (status = 'dead_letter' AND delivered_at IS NULL AND dead_lettered_at IS NOT NULL)
        OR (status NOT IN ('delivered', 'dead_letter')
            AND delivered_at IS NULL AND dead_lettered_at IS NULL)
    ),
    CONSTRAINT server_module_outbox_timestamp_order CHECK (updated_at >= created_at)
);

CREATE INDEX server_module_outbox_claim_idx
    ON server_module_outbox(status, next_attempt_at, sequence)
    WHERE status IN ('pending', 'retry');

CREATE INDEX server_module_outbox_partition_idx
    ON server_module_outbox(release_id, hook, partition_subject, sequence);

CREATE TABLE server_module_delivery_receipts (
    event_id UUID PRIMARY KEY,
    release_id UUID NOT NULL REFERENCES server_module_releases(release_id) ON DELETE RESTRICT,
    request_sha256 TEXT NOT NULL,
    response_sha256 TEXT NOT NULL,
    response_body BYTEA NOT NULL,
    outcome_code TEXT NOT NULL,
    attempt_count INTEGER NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT clock_timestamp(),
    CONSTRAINT server_module_delivery_receipts_digest_shape CHECK (
        request_sha256 ~ '^[0-9a-f]{64}$' AND response_sha256 ~ '^[0-9a-f]{64}$'
    ),
    CONSTRAINT server_module_delivery_receipts_bounds CHECK (
        octet_length(response_body) BETWEEN 1 AND 65536
        AND octet_length(outcome_code) BETWEEN 2 AND 64
        AND outcome_code ~ '^[a-z0-9_]+$'
        AND attempt_count BETWEEN 1 AND 8
    )
);

CREATE TABLE server_module_intent_receipts (
    release_id UUID NOT NULL REFERENCES server_module_releases(release_id) ON DELETE RESTRICT,
    event_id UUID NOT NULL,
    ordinal INTEGER NOT NULL,
    request_sha256 TEXT NOT NULL,
    intent_sha256 TEXT NOT NULL,
    outcome_code TEXT NOT NULL,
    target_report_id UUID NOT NULL REFERENCES persona_reports(id) ON DELETE RESTRICT,
    expected_revision BIGINT NOT NULL,
    resulting_revision BIGINT NOT NULL,
    label BIGINT,
    committed BOOLEAN NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT clock_timestamp(),
    PRIMARY KEY (release_id, event_id, ordinal),
    CONSTRAINT server_module_intent_receipts_ordinal CHECK (ordinal = 0),
    CONSTRAINT server_module_intent_receipts_digest_shape CHECK (
        request_sha256 ~ '^[0-9a-f]{64}$' AND intent_sha256 ~ '^[0-9a-f]{64}$'
    ),
    CONSTRAINT server_module_intent_receipts_bounds CHECK (
        octet_length(outcome_code) BETWEEN 2 AND 64
        AND outcome_code ~ '^[a-z0-9_]+$'
        AND expected_revision >= 0
        AND resulting_revision >= expected_revision
        AND (label IS NULL OR label BETWEEN 0 AND 100)
    )
);

CREATE TABLE server_module_report_labels (
    instance_id UUID NOT NULL REFERENCES server_module_instances(instance_id) ON DELETE RESTRICT,
    report_id UUID NOT NULL REFERENCES persona_reports(id) ON DELETE RESTRICT,
    label TEXT NOT NULL,
    revision BIGINT NOT NULL,
    source_event_id UUID NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT clock_timestamp(),
    PRIMARY KEY (instance_id, report_id, label),
    UNIQUE (source_event_id),
    CONSTRAINT server_module_report_labels_exact_label CHECK (label = 'priority_review'),
    CONSTRAINT server_module_report_labels_revision CHECK (revision = 1)
);

CREATE TABLE server_module_state_namespaces (
    instance_id UUID PRIMARY KEY REFERENCES server_module_instances(instance_id) ON DELETE RESTRICT,
    state_schema TEXT NOT NULL,
    revision BIGINT NOT NULL,
    entries JSONB NOT NULL,
    byte_size INTEGER NOT NULL,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT clock_timestamp(),
    CONSTRAINT server_module_state_namespaces_exact_schema CHECK (
        state_schema = 'ignibyte.sentinel.state/v1'
    ),
    CONSTRAINT server_module_state_namespaces_bounds CHECK (
        revision >= 0
        AND jsonb_typeof(entries) = 'object'
        AND byte_size BETWEEN 2 AND 4096
        AND byte_size = octet_length(entries::TEXT)
    )
);

CREATE TABLE server_module_state_snapshots (
    snapshot_id UUID PRIMARY KEY,
    instance_id UUID NOT NULL REFERENCES server_module_instances(instance_id) ON DELETE RESTRICT,
    source_schema TEXT NOT NULL,
    source_revision BIGINT NOT NULL,
    entries JSONB NOT NULL,
    byte_size INTEGER NOT NULL,
    reason TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT clock_timestamp(),
    CONSTRAINT server_module_state_snapshots_bounds CHECK (
        source_revision >= 0
        AND jsonb_typeof(entries) = 'object'
        AND byte_size BETWEEN 2 AND 4096
        AND byte_size = octet_length(entries::TEXT)
        AND char_length(reason) BETWEEN 1 AND 500
        AND reason !~ '[[:cntrl:]]'
    )
);

CREATE TABLE server_module_data_audit (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    operation_id UUID NOT NULL,
    instance_id UUID NOT NULL REFERENCES server_module_instances(instance_id) ON DELETE RESTRICT,
    action TEXT NOT NULL,
    command_sha256 TEXT NOT NULL,
    expected_revision BIGINT NOT NULL,
    resulting_revision BIGINT NOT NULL,
    snapshot_id UUID REFERENCES server_module_state_snapshots(snapshot_id) ON DELETE RESTRICT,
    actor TEXT NOT NULL,
    reason TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT clock_timestamp(),
    UNIQUE (instance_id, operation_id),
    CONSTRAINT server_module_data_audit_action CHECK (
        action IN ('configure', 'state_update', 'state_migrate', 'state_rollback')
    ),
    CONSTRAINT server_module_data_audit_revision CHECK (
        expected_revision >= 0 AND resulting_revision = expected_revision + 1
    ),
    CONSTRAINT server_module_data_audit_digest CHECK (command_sha256 ~ '^[0-9a-f]{64}$'),
    CONSTRAINT server_module_data_audit_text CHECK (
        char_length(actor) BETWEEN 1 AND 64
        AND actor !~ '[[:cntrl:]]'
        AND char_length(reason) BETWEEN 1 AND 500
        AND reason !~ '[[:cntrl:]]'
    )
);

CREATE FUNCTION server_module_reject_immutable_mutation() RETURNS trigger
LANGUAGE plpgsql AS $$
BEGIN
    RAISE EXCEPTION 'server module evidence is immutable';
END;
$$;

CREATE TRIGGER server_module_releases_immutable_rows
BEFORE UPDATE OR DELETE ON server_module_releases
FOR EACH ROW EXECUTE FUNCTION server_module_reject_immutable_mutation();
CREATE TRIGGER server_module_releases_immutable_truncate
BEFORE TRUNCATE ON server_module_releases
FOR EACH STATEMENT EXECUTE FUNCTION server_module_reject_immutable_mutation();

CREATE TRIGGER server_module_admissions_immutable_rows
BEFORE UPDATE OR DELETE ON server_module_admissions
FOR EACH ROW EXECUTE FUNCTION server_module_reject_immutable_mutation();
CREATE TRIGGER server_module_admissions_immutable_truncate
BEFORE TRUNCATE ON server_module_admissions
FOR EACH STATEMENT EXECUTE FUNCTION server_module_reject_immutable_mutation();

CREATE TRIGGER server_module_lifecycle_audit_immutable_rows
BEFORE UPDATE OR DELETE ON server_module_lifecycle_audit
FOR EACH ROW EXECUTE FUNCTION server_module_reject_immutable_mutation();
CREATE TRIGGER server_module_lifecycle_audit_immutable_truncate
BEFORE TRUNCATE ON server_module_lifecycle_audit
FOR EACH STATEMENT EXECUTE FUNCTION server_module_reject_immutable_mutation();

CREATE TRIGGER server_module_delivery_receipts_immutable_rows
BEFORE UPDATE OR DELETE ON server_module_delivery_receipts
FOR EACH ROW EXECUTE FUNCTION server_module_reject_immutable_mutation();
CREATE TRIGGER server_module_delivery_receipts_immutable_truncate
BEFORE TRUNCATE ON server_module_delivery_receipts
FOR EACH STATEMENT EXECUTE FUNCTION server_module_reject_immutable_mutation();

CREATE TRIGGER server_module_intent_receipts_immutable_rows
BEFORE UPDATE OR DELETE ON server_module_intent_receipts
FOR EACH ROW EXECUTE FUNCTION server_module_reject_immutable_mutation();
CREATE TRIGGER server_module_intent_receipts_immutable_truncate
BEFORE TRUNCATE ON server_module_intent_receipts
FOR EACH STATEMENT EXECUTE FUNCTION server_module_reject_immutable_mutation();

CREATE TRIGGER server_module_report_labels_immutable_rows
BEFORE UPDATE OR DELETE ON server_module_report_labels
FOR EACH ROW EXECUTE FUNCTION server_module_reject_immutable_mutation();
CREATE TRIGGER server_module_report_labels_immutable_truncate
BEFORE TRUNCATE ON server_module_report_labels
FOR EACH STATEMENT EXECUTE FUNCTION server_module_reject_immutable_mutation();

CREATE TRIGGER server_module_state_snapshots_immutable_rows
BEFORE UPDATE OR DELETE ON server_module_state_snapshots
FOR EACH ROW EXECUTE FUNCTION server_module_reject_immutable_mutation();
CREATE TRIGGER server_module_state_snapshots_immutable_truncate
BEFORE TRUNCATE ON server_module_state_snapshots
FOR EACH STATEMENT EXECUTE FUNCTION server_module_reject_immutable_mutation();

CREATE TRIGGER server_module_data_audit_immutable_rows
BEFORE UPDATE OR DELETE ON server_module_data_audit
FOR EACH ROW EXECUTE FUNCTION server_module_reject_immutable_mutation();
CREATE TRIGGER server_module_data_audit_immutable_truncate
BEFORE TRUNCATE ON server_module_data_audit
FOR EACH STATEMENT EXECUTE FUNCTION server_module_reject_immutable_mutation();

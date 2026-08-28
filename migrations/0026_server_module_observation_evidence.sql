-- Preserve core availability when an optional observation cannot be queued,
-- and keep delivery-request evidence after delivered outbox rows are pruned.

ALTER TABLE server_module_instances
    ADD COLUMN observation_gap_count BIGINT NOT NULL DEFAULT 0,
    ADD COLUMN last_observation_gap_reason TEXT,
    ADD COLUMN last_observation_gap_at TIMESTAMPTZ,
    ADD CONSTRAINT server_module_instances_observation_gap_bounds CHECK (
        observation_gap_count >= 0
        AND (
            (observation_gap_count = 0
                AND last_observation_gap_reason IS NULL
                AND last_observation_gap_at IS NULL)
            OR
            (observation_gap_count > 0
                AND last_observation_gap_reason IN ('module_inactive', 'queue_saturated')
                AND last_observation_gap_at IS NOT NULL)
        )
    );

ALTER TABLE server_module_delivery_receipts
    ADD COLUMN request_body BYTEA,
    ADD COLUMN target_report_id UUID
        REFERENCES persona_reports(id) ON DELETE RESTRICT;

ALTER TABLE server_module_delivery_receipts
    ADD CONSTRAINT server_module_delivery_receipts_request_evidence CHECK (
        request_body IS NOT NULL
        AND octet_length(request_body) BETWEEN 1 AND 65536
        AND target_report_id IS NOT NULL
    ) NOT VALID;

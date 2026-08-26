CREATE TABLE persona_reports (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    reporter_persona_id UUID NOT NULL
        REFERENCES personas(id) ON DELETE RESTRICT,
    subject_persona_id UUID NOT NULL
        REFERENCES personas(id) ON DELETE RESTRICT,
    idempotency_key UUID NOT NULL,
    category TEXT NOT NULL
        CHECK (category IN ('harassment', 'spam', 'cheating', 'other')),
    detail TEXT NOT NULL
        CHECK (char_length(detail) BETWEEN 1 AND 1000),
    status TEXT NOT NULL DEFAULT 'open'
        CHECK (status IN ('open', 'resolved', 'dismissed')),
    created_at TIMESTAMPTZ NOT NULL DEFAULT clock_timestamp(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT clock_timestamp(),
    closed_at TIMESTAMPTZ,
    CONSTRAINT persona_reports_not_self
        CHECK (reporter_persona_id <> subject_persona_id),
    CONSTRAINT persona_reports_idempotency
        UNIQUE (reporter_persona_id, idempotency_key),
    CONSTRAINT persona_reports_terminal_time CHECK (
        (status = 'open' AND closed_at IS NULL)
        OR (status IN ('resolved', 'dismissed') AND closed_at IS NOT NULL)
    ),
    CONSTRAINT persona_reports_monotonic_time
        CHECK (updated_at >= created_at AND (closed_at IS NULL OR closed_at >= created_at))
);

CREATE INDEX persona_reports_open_reporter_idx
    ON persona_reports(reporter_persona_id, created_at DESC, id)
    WHERE status = 'open';

CREATE INDEX persona_reports_operator_queue_idx
    ON persona_reports(status, created_at DESC, id);

CREATE INDEX persona_reports_subject_idx
    ON persona_reports(subject_persona_id, created_at DESC, id);

CREATE TABLE operator_audit_events (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    operation_id UUID NOT NULL,
    target_kind TEXT NOT NULL CHECK (target_kind IN ('account', 'report')),
    target_account_id UUID REFERENCES accounts(id) ON DELETE RESTRICT,
    target_report_id UUID REFERENCES persona_reports(id) ON DELETE RESTRICT,
    action TEXT NOT NULL CHECK (action IN ('set_account_status', 'set_report_status')),
    actor TEXT NOT NULL CHECK (char_length(actor) BETWEEN 1 AND 64),
    reason TEXT NOT NULL CHECK (char_length(reason) BETWEEN 1 AND 500),
    previous_state TEXT NOT NULL CHECK (char_length(previous_state) BETWEEN 1 AND 32),
    resulting_state TEXT NOT NULL CHECK (char_length(resulting_state) BETWEEN 1 AND 32),
    created_at TIMESTAMPTZ NOT NULL DEFAULT clock_timestamp(),
    CONSTRAINT operator_audit_exact_target CHECK (
        (target_kind = 'account'
            AND target_account_id IS NOT NULL
            AND target_report_id IS NULL
            AND action = 'set_account_status')
        OR (target_kind = 'report'
            AND target_account_id IS NULL
            AND target_report_id IS NOT NULL
            AND action = 'set_report_status')
    )
);

CREATE UNIQUE INDEX operator_audit_account_operation_idx
    ON operator_audit_events(target_account_id, operation_id)
    WHERE target_kind = 'account';

CREATE UNIQUE INDEX operator_audit_report_operation_idx
    ON operator_audit_events(target_report_id, operation_id)
    WHERE target_kind = 'report';

CREATE INDEX operator_audit_recent_idx
    ON operator_audit_events(created_at DESC, id);

CREATE FUNCTION persona_reports_prevent_delete() RETURNS trigger
LANGUAGE plpgsql AS $$
BEGIN
    RAISE EXCEPTION 'persona reports cannot be deleted';
END;
$$;

CREATE TRIGGER persona_reports_prevent_delete_trigger
BEFORE DELETE ON persona_reports
FOR EACH ROW EXECUTE FUNCTION persona_reports_prevent_delete();

CREATE FUNCTION operator_audit_immutable() RETURNS trigger
LANGUAGE plpgsql AS $$
BEGIN
    RAISE EXCEPTION 'operator audit is append-only';
END;
$$;

CREATE TRIGGER operator_audit_immutable_trigger
BEFORE UPDATE OR DELETE ON operator_audit_events
FOR EACH ROW EXECUTE FUNCTION operator_audit_immutable();

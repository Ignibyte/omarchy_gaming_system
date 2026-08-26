CREATE TABLE registration_invites (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    code_hash BYTEA NOT NULL UNIQUE
        CHECK (octet_length(code_hash) = 32),
    label TEXT NOT NULL
        CHECK (char_length(label) BETWEEN 1 AND 64 AND label = btrim(label)),
    valid_for_hours SMALLINT NOT NULL
        CHECK (valid_for_hours BETWEEN 1 AND 720),
    issued_operation_id UUID NOT NULL UNIQUE,
    issued_by TEXT NOT NULL
        CHECK (char_length(issued_by) BETWEEN 1 AND 64 AND issued_by = btrim(issued_by)),
    issued_reason TEXT NOT NULL
        CHECK (char_length(issued_reason) BETWEEN 1 AND 500 AND issued_reason = btrim(issued_reason)),
    created_at TIMESTAMPTZ NOT NULL,
    expires_at TIMESTAMPTZ NOT NULL,
    used_at TIMESTAMPTZ,
    used_by_account_id UUID UNIQUE REFERENCES accounts(id) ON DELETE RESTRICT,
    revoked_at TIMESTAMPTZ,
    revoked_by TEXT,
    revoked_reason TEXT,
    revoked_operation_id UUID UNIQUE,
    CONSTRAINT registration_invites_exact_expiry CHECK (
        expires_at = created_at + make_interval(hours => valid_for_hours)
    ),
    CONSTRAINT registration_invites_exact_use CHECK (
        (used_at IS NULL AND used_by_account_id IS NULL)
        OR (used_at IS NOT NULL AND used_by_account_id IS NOT NULL
            AND used_at >= created_at)
    ),
    CONSTRAINT registration_invites_exact_revocation CHECK (
        (revoked_at IS NULL AND revoked_by IS NULL
            AND revoked_reason IS NULL AND revoked_operation_id IS NULL)
        OR (revoked_at IS NOT NULL
            AND char_length(revoked_by) BETWEEN 1 AND 64
            AND revoked_by = btrim(revoked_by)
            AND char_length(revoked_reason) BETWEEN 1 AND 500
            AND revoked_reason = btrim(revoked_reason)
            AND revoked_operation_id IS NOT NULL
            AND revoked_at >= created_at)
    ),
    CONSTRAINT registration_invites_one_terminal_state CHECK (
        NOT (used_at IS NOT NULL AND revoked_at IS NOT NULL)
    )
);

CREATE INDEX registration_invites_inventory_idx
    ON registration_invites(created_at DESC, id DESC);

CREATE INDEX registration_invites_live_idx
    ON registration_invites(expires_at, id)
    WHERE used_at IS NULL AND revoked_at IS NULL;

ALTER TABLE operator_audit_events
    DROP CONSTRAINT operator_audit_exact_target,
    DROP CONSTRAINT operator_audit_events_target_kind_check,
    DROP CONSTRAINT operator_audit_events_action_check,
    ADD COLUMN target_registration_invite_id UUID
        REFERENCES registration_invites(id) ON DELETE RESTRICT,
    ADD CONSTRAINT operator_audit_events_target_kind_check CHECK (
        target_kind IN ('account', 'report', 'registration_invite')
    ),
    ADD CONSTRAINT operator_audit_events_action_check CHECK (
        action IN (
            'set_account_status',
            'set_report_status',
            'issue_registration_invite',
            'revoke_registration_invite'
        )
    ),
    ADD CONSTRAINT operator_audit_exact_target CHECK (
        (target_kind = 'account'
            AND target_account_id IS NOT NULL
            AND target_report_id IS NULL
            AND target_registration_invite_id IS NULL
            AND action = 'set_account_status')
        OR (target_kind = 'report'
            AND target_account_id IS NULL
            AND target_report_id IS NOT NULL
            AND target_registration_invite_id IS NULL
            AND action = 'set_report_status')
        OR (target_kind = 'registration_invite'
            AND target_account_id IS NULL
            AND target_report_id IS NULL
            AND target_registration_invite_id IS NOT NULL
            AND action IN (
                'issue_registration_invite',
                'revoke_registration_invite'
            ))
    );

CREATE UNIQUE INDEX operator_audit_registration_invite_operation_idx
    ON operator_audit_events(target_registration_invite_id, operation_id)
    WHERE target_kind = 'registration_invite';

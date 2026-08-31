-- Serialize provider operations across platform processes and retain a bounded
-- recovery marker if a platform process exits while a provider request is live.

ALTER TABLE game_sessions
    ADD COLUMN provider_operation_reservation_id UUID,
    ADD COLUMN provider_operation_reservation_kind TEXT,
    ADD COLUMN provider_operation_reservation_expires_at TIMESTAMPTZ,
    ADD CONSTRAINT game_sessions_provider_operation_reservation_shape CHECK (
        (
            provider_operation_reservation_id IS NULL
            AND provider_operation_reservation_kind IS NULL
            AND provider_operation_reservation_expires_at IS NULL
        ) OR (
            authority = 'registered_provider'
            AND provider_operation_reservation_id IS NOT NULL
            AND provider_operation_reservation_kind IN ('command', 'reconcile')
            AND provider_operation_reservation_expires_at IS NOT NULL
        )
    );

CREATE INDEX game_sessions_provider_operation_reservation_idx
    ON game_sessions(provider_operation_reservation_expires_at, id)
    WHERE provider_operation_reservation_id IS NOT NULL;

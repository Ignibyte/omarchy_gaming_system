CREATE TABLE persona_connections (
    persona_low_id UUID NOT NULL REFERENCES personas(id) ON DELETE CASCADE,
    persona_high_id UUID NOT NULL REFERENCES personas(id) ON DELETE CASCADE,
    requester_id UUID NOT NULL,
    addressee_id UUID NOT NULL,
    status TEXT NOT NULL DEFAULT 'pending',
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    accepted_at TIMESTAMPTZ,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    PRIMARY KEY (persona_low_id, persona_high_id),
    CONSTRAINT persona_connections_distinct_pair
        CHECK (persona_low_id < persona_high_id),
    CONSTRAINT persona_connections_direction_matches_pair
        CHECK (
            requester_id <> addressee_id
            AND (
                (requester_id = persona_low_id AND addressee_id = persona_high_id)
                OR
                (requester_id = persona_high_id AND addressee_id = persona_low_id)
            )
        ),
    CONSTRAINT persona_connections_status_valid
        CHECK (status IN ('pending', 'accepted')),
    CONSTRAINT persona_connections_acceptance_matches_status
        CHECK (
            (status = 'pending' AND accepted_at IS NULL)
            OR
            (status = 'accepted' AND accepted_at IS NOT NULL)
        )
);

CREATE INDEX persona_connections_high_id_idx
    ON persona_connections (persona_high_id);

CREATE INDEX persona_connections_pending_requester_idx
    ON persona_connections (requester_id, created_at)
    WHERE status = 'pending';

CREATE INDEX persona_connections_pending_addressee_idx
    ON persona_connections (addressee_id, created_at)
    WHERE status = 'pending';

CREATE TABLE persona_blocks (
    blocker_id UUID NOT NULL REFERENCES personas(id) ON DELETE CASCADE,
    blocked_id UUID NOT NULL REFERENCES personas(id) ON DELETE CASCADE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    PRIMARY KEY (blocker_id, blocked_id),
    CONSTRAINT persona_blocks_distinct_personas CHECK (blocker_id <> blocked_id)
);

CREATE INDEX persona_blocks_blocked_id_idx
    ON persona_blocks (blocked_id, blocker_id);

CREATE TABLE server_identity (
    singleton BOOLEAN PRIMARY KEY DEFAULT TRUE CHECK (singleton),
    id UUID NOT NULL UNIQUE DEFAULT gen_random_uuid(),
    created_at TIMESTAMPTZ NOT NULL DEFAULT clock_timestamp()
);

INSERT INTO server_identity (singleton) VALUES (TRUE);

CREATE FUNCTION server_identity_immutable() RETURNS trigger
LANGUAGE plpgsql AS $$
BEGIN
    RAISE EXCEPTION 'server identity is immutable';
END;
$$;

CREATE TRIGGER server_identity_immutable_rows_trigger
BEFORE UPDATE OR DELETE ON server_identity
FOR EACH ROW EXECUTE FUNCTION server_identity_immutable();

CREATE TRIGGER server_identity_immutable_truncate_trigger
BEFORE TRUNCATE ON server_identity
FOR EACH STATEMENT EXECUTE FUNCTION server_identity_immutable();

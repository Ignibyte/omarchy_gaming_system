ALTER TABLE personas
    ADD CONSTRAINT personas_handle_canonical
    CHECK (handle ~ '^[a-z0-9][a-z0-9_-]{2,23}$');

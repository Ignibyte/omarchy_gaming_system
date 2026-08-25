ALTER TABLE accounts
    ADD CONSTRAINT accounts_username_canonical
    CHECK (username ~ '^[a-z0-9][a-z0-9_-]{2,31}$');

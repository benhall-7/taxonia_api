-- Add migration script here

-- No specific login method required in this table
CREATE TABLE users(
    id bigserial PRIMARY KEY,
    display_name text NOT NULL,
    primary_email text,
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL DEFAULT now()
);

CREATE INDEX idx_users_primary_email ON users(primary_email);

CREATE OR REPLACE FUNCTION set_updated_at()
    RETURNS TRIGGER
    AS $$
BEGIN
    NEW.updated_at = now();
    RETURN NEW;
END;
$$
LANGUAGE plpgsql;

CREATE TRIGGER trg_users_set_updated_at
    BEFORE UPDATE ON users
    FOR EACH ROW
    EXECUTE FUNCTION set_updated_at();


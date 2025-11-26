-- Start with 'inat' auth provider first, and leave room for 'local' later.
DO $$
BEGIN
    IF NOT EXISTS(
        SELECT
            1
        FROM
            pg_type
        WHERE
            typname = 'auth_provider') THEN
    CREATE TYPE auth_provider AS ENUM(
        'inat'
);
END IF;
END;
$$;

CREATE TABLE auth_identities(
    id bigserial PRIMARY KEY,
    user_id bigint NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    provider auth_provider NOT NULL,
    provider_user_id text NOT NULL, -- user id in auth provider system
    access_token text, -- optional, can encrypt later
    refresh_token text,
    token_expires_at timestamptz,
    created_at timestamptz NOT NULL DEFAULT now(),
    last_used_at timestamptz
);

-- each provider & id pair is unique (one identity per external account)
CREATE UNIQUE INDEX uq_auth_identity_provider_user ON auth_identities(provider, provider_user_id);

-- improve lookup speed by user_id
CREATE INDEX idx_auth_identities_user_id ON auth_identities(user_id);


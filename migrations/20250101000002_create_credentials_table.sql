-- Create credentials table for WebAuthn passkeys
CREATE TABLE credentials (
    id BYTEA PRIMARY KEY,
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    public_key BYTEA NOT NULL,
    counter INTEGER NOT NULL DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Index for user credential lookups
CREATE INDEX idx_credentials_user_id ON credentials(user_id);

-- Ensure credential IDs are unique per user
CREATE UNIQUE INDEX idx_credentials_user_id_id ON credentials(user_id, id);

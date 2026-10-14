CREATE TABLE mcp_access_tokens (
    id BIGSERIAL PRIMARY KEY,
    guild_id TEXT NOT NULL,
    discord_user_id TEXT NOT NULL,
    name TEXT NOT NULL CHECK (char_length(name) BETWEEN 1 AND 80),
    token_hash TEXT NOT NULL UNIQUE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    expires_at TIMESTAMPTZ NOT NULL,
    last_used_at TIMESTAMPTZ,
    revoked_at TIMESTAMPTZ
);

CREATE INDEX mcp_access_tokens_active_hash_idx
    ON mcp_access_tokens (token_hash)
    WHERE revoked_at IS NULL;

CREATE INDEX mcp_access_tokens_guild_actor_idx
    ON mcp_access_tokens (guild_id, discord_user_id, created_at DESC);

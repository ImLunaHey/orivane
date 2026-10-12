CREATE TABLE connected_guilds (
    guild_id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    member_count BIGINT NOT NULL DEFAULT 0 CHECK (member_count >= 0),
    shard_id BIGINT NOT NULL DEFAULT 0 CHECK (shard_id >= 0),
    owner_id TEXT,
    installer_id TEXT,
    connected BOOLEAN NOT NULL DEFAULT true,
    installed_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    removed_at TIMESTAMPTZ,
    CONSTRAINT connected_guild_id_is_snowflake CHECK (guild_id ~ '^[1-9][0-9]{0,19}$')
);

CREATE INDEX connected_guilds_size_idx ON connected_guilds (member_count DESC) WHERE connected;
CREATE INDEX connected_guilds_installer_idx ON connected_guilds (installer_id) WHERE connected;


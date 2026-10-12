CREATE TABLE guild_configs (
    guild_id TEXT PRIMARY KEY,
    config JSONB NOT NULL,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    CONSTRAINT guild_id_is_snowflake CHECK (guild_id ~ '^[1-9][0-9]{0,19}$')
);

CREATE INDEX guild_configs_updated_at_idx ON guild_configs (updated_at DESC);


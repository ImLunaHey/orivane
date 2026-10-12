CREATE TABLE temporary_voice_channels (
    id BIGSERIAL PRIMARY KEY,
    guild_id TEXT NOT NULL,
    channel_id TEXT NOT NULL UNIQUE,
    owner_id TEXT NOT NULL,
    name TEXT NOT NULL,
    user_limit SMALLINT NOT NULL DEFAULT 0,
    locked BOOLEAN NOT NULL DEFAULT FALSE,
    status TEXT NOT NULL DEFAULT 'active' CHECK (status IN ('active', 'deleted')),
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    deleted_at TIMESTAMPTZ,
    delete_after TIMESTAMPTZ
);

CREATE INDEX temporary_voice_channels_guild_active_idx
ON temporary_voice_channels (guild_id, updated_at DESC)
WHERE status = 'active';

CREATE UNIQUE INDEX temporary_voice_channels_owner_active_idx
ON temporary_voice_channels (guild_id, owner_id)
WHERE status = 'active';

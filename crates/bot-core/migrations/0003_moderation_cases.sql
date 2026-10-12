CREATE TABLE moderation_cases (
    id BIGSERIAL PRIMARY KEY,
    guild_id TEXT NOT NULL,
    member_id TEXT NOT NULL,
    moderator_id TEXT NOT NULL,
    action TEXT NOT NULL CHECK (action IN ('note', 'warn', 'timeout', 'kick', 'ban')),
    reason TEXT NOT NULL,
    status TEXT NOT NULL DEFAULT 'open' CHECK (status IN ('open', 'resolved', 'reversed')),
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    CONSTRAINT case_guild_id_is_snowflake CHECK (guild_id ~ '^[1-9][0-9]{0,19}$'),
    CONSTRAINT case_member_id_is_snowflake CHECK (member_id ~ '^[1-9][0-9]{0,19}$')
);

CREATE INDEX moderation_cases_guild_created_idx ON moderation_cases (guild_id, created_at DESC);
CREATE INDEX moderation_cases_member_created_idx ON moderation_cases (member_id, created_at DESC);


CREATE TABLE member_xp (
    guild_id TEXT NOT NULL,
    user_id TEXT NOT NULL,
    xp BIGINT NOT NULL DEFAULT 0 CHECK (xp >= 0),
    message_count BIGINT NOT NULL DEFAULT 0 CHECK (message_count >= 0),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    PRIMARY KEY (guild_id, user_id)
);

CREATE INDEX member_xp_guild_leaderboard_idx
    ON member_xp (guild_id, xp DESC);

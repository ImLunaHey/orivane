CREATE TABLE starboard_posts (
    guild_id TEXT NOT NULL,
    source_message_id TEXT NOT NULL,
    starboard_channel_id TEXT NOT NULL,
    posted_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    PRIMARY KEY (guild_id, source_message_id)
);

ALTER TABLE starboard_posts
    ADD COLUMN source_channel_id TEXT,
    ADD COLUMN starboard_message_id TEXT,
    ADD COLUMN reaction_count BIGINT NOT NULL DEFAULT 0;

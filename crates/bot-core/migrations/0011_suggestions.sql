CREATE TABLE suggestions (
    id BIGSERIAL PRIMARY KEY,
    guild_id TEXT NOT NULL,
    channel_id TEXT NOT NULL,
    message_id TEXT,
    author_id TEXT,
    content TEXT NOT NULL CHECK (char_length(content) BETWEEN 1 AND 1800),
    status TEXT NOT NULL DEFAULT 'pending' CHECK (status IN ('pending', 'approved', 'rejected', 'implemented', 'duplicate')),
    staff_note TEXT CHECK (staff_note IS NULL OR char_length(staff_note) <= 500),
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE suggestion_votes (
    suggestion_id BIGINT NOT NULL REFERENCES suggestions(id) ON DELETE CASCADE,
    user_id TEXT NOT NULL,
    direction SMALLINT NOT NULL CHECK (direction IN (-1, 1)),
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    PRIMARY KEY (suggestion_id, user_id)
);

CREATE INDEX suggestions_guild_created_idx ON suggestions (guild_id, created_at DESC, id DESC);
CREATE INDEX suggestion_votes_suggestion_idx ON suggestion_votes (suggestion_id);

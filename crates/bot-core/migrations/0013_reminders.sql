CREATE TABLE reminders (
    id BIGSERIAL PRIMARY KEY,
    guild_id TEXT NOT NULL,
    channel_id TEXT,
    user_id TEXT NOT NULL,
    message TEXT NOT NULL CHECK (char_length(message) BETWEEN 1 AND 1000),
    delivery_mode TEXT NOT NULL CHECK (delivery_mode IN ('channel', 'dm')),
    status TEXT NOT NULL DEFAULT 'scheduled' CHECK (status IN ('scheduled', 'delivering', 'delivered', 'cancelled', 'failed')),
    remind_at TIMESTAMPTZ NOT NULL,
    next_attempt_at TIMESTAMPTZ NOT NULL,
    delivered_at TIMESTAMPTZ,
    attempts SMALLINT NOT NULL DEFAULT 0 CHECK (attempts BETWEEN 0 AND 5),
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    CHECK ((delivery_mode = 'dm') OR channel_id IS NOT NULL)
);

CREATE INDEX reminders_due_idx ON reminders (guild_id, next_attempt_at, id)
    WHERE status = 'scheduled';
CREATE INDEX reminders_user_idx ON reminders (guild_id, user_id, created_at DESC, id DESC);
CREATE INDEX reminders_guild_idx ON reminders (guild_id, created_at DESC, id DESC);

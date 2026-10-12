CREATE TABLE tickets (
    id BIGSERIAL PRIMARY KEY,
    guild_id TEXT NOT NULL,
    opener_id TEXT NOT NULL,
    channel_id TEXT NOT NULL UNIQUE,
    status TEXT NOT NULL DEFAULT 'open' CHECK (status IN ('open', 'closed')),
    opened_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    closed_at TIMESTAMPTZ,
    closed_by TEXT
);

CREATE UNIQUE INDEX tickets_one_open_per_member_idx
    ON tickets (guild_id, opener_id)
    WHERE status = 'open';

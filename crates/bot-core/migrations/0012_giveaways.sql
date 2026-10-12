CREATE TABLE giveaways (
    id BIGSERIAL PRIMARY KEY,
    guild_id TEXT NOT NULL,
    channel_id TEXT NOT NULL,
    message_id TEXT,
    creator_id TEXT NOT NULL,
    title TEXT NOT NULL CHECK (char_length(title) BETWEEN 1 AND 100),
    description TEXT NOT NULL DEFAULT '' CHECK (char_length(description) <= 1800),
    status TEXT NOT NULL DEFAULT 'pending_delivery' CHECK (status IN ('pending_delivery', 'active', 'ended', 'cancelled', 'delivery_failed')),
    winner_count SMALLINT NOT NULL CHECK (winner_count BETWEEN 1 AND 25),
    minimum_account_age_days INTEGER NOT NULL DEFAULT 0 CHECK (minimum_account_age_days BETWEEN 0 AND 3650),
    required_role_ids JSONB NOT NULL DEFAULT '[]'::jsonb,
    bonus_role_id TEXT,
    bonus_entries SMALLINT NOT NULL DEFAULT 1 CHECK (bonus_entries BETWEEN 0 AND 25),
    winner_ids JSONB NOT NULL DEFAULT '[]'::jsonb,
    ends_at TIMESTAMPTZ NOT NULL,
    ended_at TIMESTAMPTZ,
    announcement_sent_at TIMESTAMPTZ,
    reroll_count INTEGER NOT NULL DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE giveaway_entries (
    giveaway_id BIGINT NOT NULL REFERENCES giveaways(id) ON DELETE CASCADE,
    user_id TEXT NOT NULL,
    weight SMALLINT NOT NULL DEFAULT 1 CHECK (weight BETWEEN 1 AND 26),
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    PRIMARY KEY (giveaway_id, user_id)
);

CREATE INDEX giveaways_guild_created_idx ON giveaways (guild_id, created_at DESC, id DESC);
CREATE INDEX giveaways_due_idx ON giveaways (guild_id, ends_at) WHERE status = 'active';
CREATE INDEX giveaway_entries_giveaway_idx ON giveaway_entries (giveaway_id, created_at);

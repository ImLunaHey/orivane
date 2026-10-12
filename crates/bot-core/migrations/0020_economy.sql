CREATE TABLE economy_accounts (
    guild_id TEXT NOT NULL,
    user_id TEXT NOT NULL,
    balance BIGINT NOT NULL DEFAULT 0 CHECK (balance >= 0),
    lifetime_earned BIGINT NOT NULL DEFAULT 0 CHECK (lifetime_earned >= 0),
    lifetime_spent BIGINT NOT NULL DEFAULT 0 CHECK (lifetime_spent >= 0),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    PRIMARY KEY (guild_id, user_id)
);

CREATE TABLE economy_transactions (
    id BIGSERIAL PRIMARY KEY,
    guild_id TEXT NOT NULL,
    user_id TEXT NOT NULL,
    counterparty_id TEXT,
    amount BIGINT NOT NULL,
    kind TEXT NOT NULL,
    reference TEXT NOT NULL UNIQUE,
    metadata JSONB NOT NULL DEFAULT '{}'::jsonb,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE INDEX economy_transactions_guild_created_idx ON economy_transactions (guild_id, created_at DESC, id DESC);
CREATE INDEX economy_transactions_user_created_idx ON economy_transactions (guild_id, user_id, created_at DESC);

CREATE TABLE economy_claims (
    guild_id TEXT NOT NULL,
    user_id TEXT NOT NULL,
    kind TEXT NOT NULL,
    next_claim_at TIMESTAMPTZ NOT NULL,
    PRIMARY KEY (guild_id, user_id, kind)
);

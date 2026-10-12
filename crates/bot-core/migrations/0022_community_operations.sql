CREATE TABLE polls (
    id BIGSERIAL PRIMARY KEY,
    guild_id TEXT NOT NULL,
    channel_id TEXT NOT NULL,
    creator_id TEXT NOT NULL,
    question TEXT NOT NULL,
    options JSONB NOT NULL,
    anonymous BOOLEAN NOT NULL DEFAULT FALSE,
    status TEXT NOT NULL DEFAULT 'active',
    ends_at TIMESTAMPTZ NOT NULL,
    closed_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    CHECK (status IN ('active', 'closed', 'expired'))
);
CREATE INDEX polls_guild_created_idx ON polls (guild_id, created_at DESC);
CREATE INDEX polls_due_idx ON polls (ends_at) WHERE status = 'active';

CREATE TABLE poll_votes (
    poll_id BIGINT NOT NULL REFERENCES polls(id) ON DELETE CASCADE,
    user_id TEXT NOT NULL,
    option_index INTEGER NOT NULL,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    PRIMARY KEY (poll_id, user_id),
    CHECK (option_index >= 0)
);

CREATE TABLE invite_attributions (
    id BIGSERIAL PRIMARY KEY,
    guild_id TEXT NOT NULL,
    joined_user_id TEXT NOT NULL,
    invite_code TEXT NOT NULL,
    inviter_id TEXT,
    attributed_at TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE INDEX invite_attributions_guild_idx ON invite_attributions (guild_id, attributed_at DESC);
CREATE INDEX invite_attributions_inviter_idx ON invite_attributions (guild_id, inviter_id, attributed_at DESC);

CREATE TABLE afk_statuses (
    guild_id TEXT NOT NULL,
    user_id TEXT NOT NULL,
    reason TEXT NOT NULL,
    set_at TIMESTAMPTZ NOT NULL,
    expires_at TIMESTAMPTZ NOT NULL,
    PRIMARY KEY (guild_id, user_id)
);
CREATE INDEX afk_statuses_expiry_idx ON afk_statuses (expires_at);

CREATE TABLE verification_attempts (
    id BIGSERIAL PRIMARY KEY,
    guild_id TEXT NOT NULL,
    user_id TEXT NOT NULL,
    outcome TEXT NOT NULL,
    reason TEXT,
    account_age_days INTEGER NOT NULL DEFAULT 0,
    has_avatar BOOLEAN NOT NULL DEFAULT FALSE,
    occurred_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    CHECK (outcome IN ('pending', 'verified', 'rejected', 'failed'))
);
CREATE INDEX verification_attempts_guild_idx ON verification_attempts (guild_id, occurred_at DESC);

CREATE TABLE counter_runs (
    id BIGSERIAL PRIMARY KEY,
    guild_id TEXT NOT NULL,
    counter_id TEXT NOT NULL,
    channel_id TEXT NOT NULL,
    value BIGINT NOT NULL,
    rendered_name TEXT NOT NULL,
    outcome TEXT NOT NULL,
    error TEXT,
    occurred_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    CHECK (outcome IN ('success', 'failure'))
);
CREATE INDEX counter_runs_guild_idx ON counter_runs (guild_id, occurred_at DESC);

CREATE TABLE voice_link_events (
    id BIGSERIAL PRIMARY KEY,
    guild_id TEXT NOT NULL,
    user_id TEXT NOT NULL,
    voice_channel_id TEXT NOT NULL,
    text_channel_id TEXT NOT NULL,
    action TEXT NOT NULL,
    occurred_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    CHECK (action IN ('joined', 'left'))
);
CREATE INDEX voice_link_events_guild_idx ON voice_link_events (guild_id, occurred_at DESC);

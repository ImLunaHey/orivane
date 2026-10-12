CREATE TABLE autorole_jobs (
    id BIGSERIAL PRIMARY KEY,
    guild_id TEXT NOT NULL,
    user_id TEXT NOT NULL,
    role_id TEXT NOT NULL,
    action TEXT NOT NULL,
    status TEXT NOT NULL DEFAULT 'scheduled',
    due_at TIMESTAMPTZ NOT NULL,
    attempts SMALLINT NOT NULL DEFAULT 0,
    last_error TEXT,
    completed_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    CHECK (action IN ('add', 'remove')),
    CHECK (status IN ('scheduled', 'delivering', 'completed', 'failed')),
    UNIQUE (guild_id, user_id, role_id, action, due_at)
);

CREATE INDEX autorole_jobs_due_idx ON autorole_jobs (guild_id, due_at)
WHERE status IN ('scheduled', 'delivering');

CREATE TABLE moderation_jobs (
    id BIGSERIAL PRIMARY KEY,
    guild_id TEXT NOT NULL,
    member_id TEXT NOT NULL,
    action TEXT NOT NULL,
    reason TEXT NOT NULL,
    status TEXT NOT NULL DEFAULT 'scheduled',
    due_at TIMESTAMPTZ NOT NULL,
    attempts SMALLINT NOT NULL DEFAULT 0,
    last_error TEXT,
    completed_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    CHECK (action IN ('unban')),
    CHECK (status IN ('scheduled', 'delivering', 'completed', 'failed'))
);

CREATE INDEX moderation_jobs_due_idx ON moderation_jobs (guild_id, due_at)
WHERE status IN ('scheduled', 'delivering');

ALTER TABLE moderation_cases
ADD COLUMN evidence_urls JSONB NOT NULL DEFAULT '[]'::jsonb;

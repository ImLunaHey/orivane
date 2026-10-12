CREATE TABLE message_deletion_jobs (
    guild_id TEXT NOT NULL,
    channel_id TEXT NOT NULL,
    message_id TEXT NOT NULL,
    rule_id TEXT NOT NULL,
    author_id TEXT NOT NULL,
    source TEXT NOT NULL CHECK (source IN ('rule', 'purge')),
    status TEXT NOT NULL DEFAULT 'scheduled' CHECK (status IN ('scheduled','delivering','deleted','failed','cancelled')),
    due_at TIMESTAMPTZ NOT NULL,
    attempts SMALLINT NOT NULL DEFAULT 0 CHECK (attempts BETWEEN 0 AND 5),
    deleted_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    PRIMARY KEY (guild_id, channel_id, message_id)
);
CREATE INDEX message_deletion_jobs_due_idx ON message_deletion_jobs (guild_id, due_at, message_id) WHERE status = 'scheduled';
CREATE INDEX message_deletion_jobs_guild_idx ON message_deletion_jobs (guild_id, created_at DESC);

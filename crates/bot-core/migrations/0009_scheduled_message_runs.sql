CREATE TABLE scheduled_message_runs (
    guild_id TEXT NOT NULL,
    schedule_id TEXT NOT NULL,
    occurrence_unix BIGINT NOT NULL,
    claimed_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    PRIMARY KEY (guild_id, schedule_id, occurrence_unix)
);

CREATE INDEX scheduled_message_runs_claimed_at_idx ON scheduled_message_runs (claimed_at);

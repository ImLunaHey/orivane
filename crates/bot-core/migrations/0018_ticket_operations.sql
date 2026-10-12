ALTER TABLE ticket_panels
    ADD COLUMN auto_close_hours INTEGER NOT NULL DEFAULT 0 CHECK (auto_close_hours BETWEEN 0 AND 8760),
    ADD COLUMN transcript_retention_days INTEGER NOT NULL DEFAULT 90 CHECK (transcript_retention_days BETWEEN 1 AND 3650),
    ADD COLUMN default_priority TEXT NOT NULL DEFAULT 'normal' CHECK (default_priority IN ('low','normal','high','urgent'));

ALTER TABLE tickets
    ADD COLUMN priority TEXT NOT NULL DEFAULT 'normal' CHECK (priority IN ('low','normal','high','urgent')),
    ADD COLUMN tags JSONB NOT NULL DEFAULT '[]'::jsonb,
    ADD COLUMN first_response_at TIMESTAMPTZ,
    ADD COLUMN last_activity_at TIMESTAMPTZ NOT NULL DEFAULT now();

CREATE TABLE ticket_notes (
    id BIGSERIAL PRIMARY KEY,
    ticket_id BIGINT NOT NULL REFERENCES tickets(id) ON DELETE CASCADE,
    guild_id TEXT NOT NULL,
    author_id TEXT NOT NULL,
    body TEXT NOT NULL CHECK (char_length(body) BETWEEN 1 AND 2000),
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE INDEX ticket_notes_ticket_idx ON ticket_notes (ticket_id, created_at DESC);
CREATE INDEX tickets_activity_idx ON tickets (last_activity_at) WHERE status = 'open';

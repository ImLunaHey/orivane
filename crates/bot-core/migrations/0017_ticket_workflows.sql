DROP INDEX IF EXISTS tickets_one_open_per_member_idx;
ALTER TABLE tickets DROP CONSTRAINT IF EXISTS tickets_status_check;
ALTER TABLE tickets
    ADD COLUMN IF NOT EXISTS panel_id BIGINT,
    ADD COLUMN IF NOT EXISTS subject TEXT NOT NULL DEFAULT 'Support request',
    ADD COLUMN IF NOT EXISTS opening_note TEXT NOT NULL DEFAULT '',
    ADD COLUMN IF NOT EXISTS claimed_by TEXT,
    ADD COLUMN IF NOT EXISTS assigned_to TEXT,
    ADD COLUMN IF NOT EXISTS closed_reason TEXT,
    ADD COLUMN IF NOT EXISTS transcript TEXT,
    ADD COLUMN IF NOT EXISTS reopened_at TIMESTAMPTZ,
    ADD COLUMN IF NOT EXISTS updated_at TIMESTAMPTZ NOT NULL DEFAULT now();
ALTER TABLE tickets ADD CONSTRAINT tickets_status_check CHECK (status IN ('open', 'closed'));

CREATE TABLE ticket_panels (
    id BIGSERIAL PRIMARY KEY,
    guild_id TEXT NOT NULL,
    name TEXT NOT NULL,
    title TEXT NOT NULL,
    description TEXT NOT NULL DEFAULT '',
    panel_channel_id TEXT,
    panel_message_id TEXT,
    category_id TEXT NOT NULL,
    support_role_ids JSONB NOT NULL DEFAULT '[]'::jsonb,
    button_label TEXT NOT NULL DEFAULT 'Open ticket',
    welcome_message TEXT NOT NULL,
    channel_name TEXT NOT NULL DEFAULT 'ticket-{user_id}',
    intake_enabled BOOLEAN NOT NULL DEFAULT TRUE,
    intake_label TEXT NOT NULL DEFAULT 'How can we help?',
    max_open_per_user SMALLINT NOT NULL DEFAULT 1 CHECK (max_open_per_user BETWEEN 1 AND 5),
    enabled BOOLEAN NOT NULL DEFAULT TRUE,
    created_by TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    deleted_at TIMESTAMPTZ
);
CREATE UNIQUE INDEX ticket_panels_guild_name_idx ON ticket_panels (guild_id, lower(name)) WHERE deleted_at IS NULL;
CREATE INDEX ticket_panels_guild_idx ON ticket_panels (guild_id, updated_at DESC) WHERE deleted_at IS NULL;

ALTER TABLE tickets ADD CONSTRAINT tickets_panel_id_fkey FOREIGN KEY (panel_id) REFERENCES ticket_panels(id) ON DELETE SET NULL;
CREATE INDEX tickets_guild_status_idx ON tickets (guild_id, status, updated_at DESC);
CREATE INDEX tickets_panel_idx ON tickets (panel_id, status) WHERE panel_id IS NOT NULL;

CREATE TABLE ticket_events (
    id BIGSERIAL PRIMARY KEY,
    ticket_id BIGINT NOT NULL REFERENCES tickets(id) ON DELETE CASCADE,
    guild_id TEXT NOT NULL,
    actor_id TEXT NOT NULL,
    action TEXT NOT NULL,
    metadata JSONB NOT NULL DEFAULT '{}'::jsonb,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE INDEX ticket_events_ticket_idx ON ticket_events (ticket_id, created_at ASC);

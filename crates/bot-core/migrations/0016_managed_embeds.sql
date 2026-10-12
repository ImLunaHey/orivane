CREATE TABLE embed_templates (
    id BIGSERIAL PRIMARY KEY,
    guild_id TEXT NOT NULL,
    name TEXT NOT NULL CHECK (char_length(name) BETWEEN 1 AND 100),
    payload JSONB NOT NULL,
    created_by TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    deleted_at TIMESTAMPTZ
);

CREATE TABLE embed_deliveries (
    id BIGSERIAL PRIMARY KEY,
    guild_id TEXT NOT NULL,
    template_id BIGINT NOT NULL REFERENCES embed_templates(id),
    channel_id TEXT NOT NULL,
    message_id TEXT,
    actor_id TEXT NOT NULL,
    action TEXT NOT NULL CHECK (action IN ('send', 'edit')),
    status TEXT NOT NULL CHECK (status IN ('delivered', 'failed')),
    error TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE UNIQUE INDEX embed_templates_guild_name_active_idx ON embed_templates (guild_id, name) WHERE deleted_at IS NULL;
CREATE INDEX embed_templates_guild_updated_idx ON embed_templates (guild_id, updated_at DESC, id DESC) WHERE deleted_at IS NULL;
CREATE INDEX embed_deliveries_guild_created_idx ON embed_deliveries (guild_id, created_at DESC, id DESC);
CREATE INDEX embed_deliveries_template_created_idx ON embed_deliveries (template_id, created_at DESC, id DESC);

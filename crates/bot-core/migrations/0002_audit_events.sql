CREATE TABLE audit_events (
    id BIGSERIAL PRIMARY KEY,
    occurred_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    scope TEXT NOT NULL CHECK (scope IN ('global', 'guild')),
    guild_id TEXT,
    actor_id TEXT NOT NULL,
    actor_kind TEXT NOT NULL CHECK (actor_kind IN ('user', 'bot', 'system', 'api')),
    action TEXT NOT NULL,
    target_type TEXT NOT NULL,
    target_id TEXT,
    outcome TEXT NOT NULL CHECK (outcome IN ('success', 'failure', 'denied')),
    source TEXT NOT NULL,
    request_id TEXT,
    reason TEXT,
    metadata JSONB NOT NULL DEFAULT '{}'::jsonb,
    before_state JSONB,
    after_state JSONB,
    CONSTRAINT audit_scope_guild CHECK (
        (scope = 'guild' AND guild_id IS NOT NULL) OR scope = 'global'
    )
);

CREATE INDEX audit_events_occurred_at_idx ON audit_events (occurred_at DESC);
CREATE INDEX audit_events_guild_occurred_at_idx ON audit_events (guild_id, occurred_at DESC);
CREATE INDEX audit_events_actor_occurred_at_idx ON audit_events (actor_id, occurred_at DESC);
CREATE INDEX audit_events_action_occurred_at_idx ON audit_events (action, occurred_at DESC);

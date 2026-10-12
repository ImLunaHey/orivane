CREATE INDEX IF NOT EXISTS audit_events_actor_id_desc_idx
    ON audit_events (actor_id, id DESC);

CREATE INDEX IF NOT EXISTS audit_events_target_id_desc_idx
    ON audit_events (target_id, id DESC)
    WHERE target_id IS NOT NULL;

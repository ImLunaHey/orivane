CREATE INDEX reminders_recovery_idx ON reminders (guild_id, updated_at, id)
    WHERE status = 'delivering';

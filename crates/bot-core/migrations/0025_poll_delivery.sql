ALTER TABLE polls ADD COLUMN message_id TEXT;
CREATE INDEX polls_due_delivery_idx ON polls (ends_at)
WHERE status = 'active' AND message_id IS NOT NULL;

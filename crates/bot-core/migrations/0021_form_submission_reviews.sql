ALTER TABLE form_submissions ADD COLUMN status text NOT NULL DEFAULT 'submitted';
ALTER TABLE form_submissions ADD COLUMN reviewer_id text;
ALTER TABLE form_submissions ADD COLUMN review_note text;
ALTER TABLE form_submissions ADD COLUMN reviewed_at timestamptz;
CREATE INDEX form_submissions_review_queue ON form_submissions (guild_id, form_id, status, created_at DESC);

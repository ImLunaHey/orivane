CREATE TABLE form_submissions (
    id BIGSERIAL PRIMARY KEY,
    guild_id TEXT NOT NULL,
    form_id TEXT NOT NULL,
    submitter_id TEXT,
    answers JSONB NOT NULL CHECK (jsonb_typeof(answers) = 'array'),
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX form_submissions_guild_form_created_idx
    ON form_submissions (guild_id, form_id, created_at DESC, id DESC);

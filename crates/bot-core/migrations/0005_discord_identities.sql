CREATE TABLE discord_identities (
    user_id TEXT PRIMARY KEY,
    username TEXT NOT NULL,
    display_name TEXT,
    avatar_url TEXT,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

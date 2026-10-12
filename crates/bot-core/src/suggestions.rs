use chrono::{DateTime, Utc};
use serde::Serialize;
use sqlx::FromRow;

#[derive(Clone, Debug, Serialize, FromRow)]
pub struct Suggestion {
    pub id: i64,
    pub guild_id: String,
    pub channel_id: String,
    pub message_id: Option<String>,
    pub author_id: Option<String>,
    pub username: Option<String>,
    pub display_name: Option<String>,
    pub content: String,
    pub status: String,
    pub staff_note: Option<String>,
    pub upvotes: i64,
    pub downvotes: i64,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Clone, Debug)]
pub struct NewSuggestion {
    pub guild_id: String,
    pub channel_id: String,
    pub author_id: Option<String>,
    pub content: String,
}

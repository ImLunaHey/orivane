use chrono::{DateTime, Utc};
use serde::Serialize;
use sqlx::FromRow;

#[derive(Clone, Debug, FromRow, Serialize)]
pub struct MessageDeletionJob {
    pub guild_id: String,
    pub channel_id: String,
    pub message_id: String,
    pub rule_id: String,
    pub author_id: String,
    pub source: String,
    pub status: String,
    pub due_at: DateTime<Utc>,
    pub attempts: i16,
    pub deleted_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

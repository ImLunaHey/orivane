use chrono::{DateTime, Utc};
use serde::Serialize;
use sqlx::FromRow;

#[derive(Clone, Debug, FromRow, Serialize)]
pub struct TemporaryVoiceChannel {
    pub id: i64,
    pub guild_id: String,
    pub channel_id: String,
    pub owner_id: String,
    pub name: String,
    pub user_limit: i16,
    pub locked: bool,
    pub status: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
    pub delete_after: Option<DateTime<Utc>>,
}

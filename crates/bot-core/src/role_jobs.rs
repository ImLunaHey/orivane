use chrono::{DateTime, Utc};
use serde::Serialize;
use sqlx::FromRow;

#[derive(Clone, Debug, FromRow, Serialize)]
pub struct AutoroleJob {
    pub id: i64,
    pub guild_id: String,
    pub user_id: String,
    pub role_id: String,
    pub action: String,
    pub status: String,
    pub due_at: DateTime<Utc>,
    pub attempts: i16,
    pub last_error: Option<String>,
    pub completed_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Clone, Debug, FromRow, Serialize)]
pub struct ModerationJob {
    pub id: i64,
    pub guild_id: String,
    pub member_id: String,
    pub action: String,
    pub reason: String,
    pub status: String,
    pub due_at: DateTime<Utc>,
    pub attempts: i16,
    pub last_error: Option<String>,
    pub completed_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

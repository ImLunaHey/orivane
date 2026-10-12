use chrono::{DateTime, Utc};
use serde::Serialize;
use sqlx::FromRow;

#[derive(Clone, Debug)]
pub struct NewReminder {
    pub guild_id: String,
    pub channel_id: Option<String>,
    pub user_id: String,
    pub message: String,
    pub delivery_mode: String,
    pub remind_at: DateTime<Utc>,
}

#[derive(Clone, Debug, FromRow, Serialize)]
pub struct Reminder {
    pub id: i64,
    pub guild_id: String,
    pub channel_id: Option<String>,
    pub user_id: String,
    pub username: Option<String>,
    pub display_name: Option<String>,
    pub message: String,
    pub delivery_mode: String,
    pub status: String,
    pub remind_at: DateTime<Utc>,
    pub next_attempt_at: DateTime<Utc>,
    pub delivered_at: Option<DateTime<Utc>>,
    pub attempts: i16,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

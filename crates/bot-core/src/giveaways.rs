use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Clone, Debug)]
pub struct NewGiveaway {
    pub guild_id: String,
    pub channel_id: String,
    pub creator_id: String,
    pub title: String,
    pub description: String,
    pub winner_count: i16,
    pub minimum_account_age_days: i32,
    pub required_role_ids: serde_json::Value,
    pub bonus_role_id: Option<String>,
    pub bonus_entries: i16,
    pub ends_at: DateTime<Utc>,
}

#[derive(Clone, Debug, FromRow, Serialize)]
pub struct Giveaway {
    pub id: i64,
    pub guild_id: String,
    pub channel_id: String,
    pub message_id: Option<String>,
    pub creator_id: String,
    pub creator_username: Option<String>,
    pub creator_display_name: Option<String>,
    pub title: String,
    pub description: String,
    pub status: String,
    pub winner_count: i16,
    pub minimum_account_age_days: i32,
    pub required_role_ids: serde_json::Value,
    pub bonus_role_id: Option<String>,
    pub bonus_entries: i16,
    pub winner_ids: serde_json::Value,
    pub entrant_count: i64,
    pub total_entries: i64,
    pub ends_at: DateTime<Utc>,
    pub ended_at: Option<DateTime<Utc>>,
    pub announcement_sent_at: Option<DateTime<Utc>>,
    pub reroll_count: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Clone, Debug, Deserialize, FromRow, Serialize)]
pub struct GiveawayEntry {
    pub user_id: String,
    pub username: Option<String>,
    pub display_name: Option<String>,
    pub weight: i16,
    pub created_at: DateTime<Utc>,
}

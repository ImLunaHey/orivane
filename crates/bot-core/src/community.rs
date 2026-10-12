use chrono::{DateTime, Utc};
use serde::Serialize;
use sqlx::FromRow;

#[derive(Clone, Debug, Serialize, FromRow)]
pub struct Poll {
    pub id: i64,
    pub guild_id: String,
    pub channel_id: String,
    pub creator_id: String,
    pub creator_username: Option<String>,
    pub creator_display_name: Option<String>,
    pub question: String,
    pub options: serde_json::Value,
    pub counts: serde_json::Value,
    pub total_votes: i64,
    pub anonymous: bool,
    pub status: String,
    pub ends_at: DateTime<Utc>,
    pub closed_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

#[derive(Clone, Debug, Serialize, FromRow)]
pub struct InviteAttribution {
    pub id: i64,
    pub guild_id: String,
    pub joined_user_id: String,
    pub joined_username: Option<String>,
    pub joined_display_name: Option<String>,
    pub invite_code: String,
    pub inviter_id: Option<String>,
    pub inviter_username: Option<String>,
    pub inviter_display_name: Option<String>,
    pub attributed_at: DateTime<Utc>,
}

#[derive(Clone, Debug, Serialize, FromRow)]
pub struct InviteLeaderboardEntry {
    pub inviter_id: String,
    pub username: Option<String>,
    pub display_name: Option<String>,
    pub attributed_joins: i64,
    pub latest_attribution: DateTime<Utc>,
}

#[derive(Clone, Debug, Serialize, FromRow)]
pub struct AfkStatus {
    pub guild_id: String,
    pub user_id: String,
    pub username: Option<String>,
    pub display_name: Option<String>,
    pub reason: String,
    pub set_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
}

#[derive(Clone, Debug, Serialize, FromRow)]
pub struct VerificationAttempt {
    pub id: i64,
    pub guild_id: String,
    pub user_id: String,
    pub username: Option<String>,
    pub display_name: Option<String>,
    pub outcome: String,
    pub reason: Option<String>,
    pub account_age_days: i32,
    pub has_avatar: bool,
    pub occurred_at: DateTime<Utc>,
}

#[derive(Clone, Debug, Serialize, FromRow)]
pub struct CounterRun {
    pub id: i64,
    pub guild_id: String,
    pub counter_id: String,
    pub channel_id: String,
    pub value: i64,
    pub rendered_name: String,
    pub outcome: String,
    pub error: Option<String>,
    pub occurred_at: DateTime<Utc>,
}

#[derive(Clone, Debug, Serialize, FromRow)]
pub struct VoiceLinkEvent {
    pub id: i64,
    pub guild_id: String,
    pub user_id: String,
    pub username: Option<String>,
    pub display_name: Option<String>,
    pub voice_channel_id: String,
    pub text_channel_id: String,
    pub action: String,
    pub occurred_at: DateTime<Utc>,
}

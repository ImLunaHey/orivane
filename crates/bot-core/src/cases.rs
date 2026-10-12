use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Clone, Debug, Serialize, Deserialize, FromRow)]
pub struct ModerationCase {
    pub id: i64,
    pub guild_id: String,
    pub member_id: String,
    pub moderator_id: String,
    pub action: String,
    pub reason: String,
    pub status: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct NewModerationCase {
    pub member_id: String,
    pub action: String,
    pub reason: String,
}

impl NewModerationCase {
    pub fn validate(&self) -> Result<(), &'static str> {
        if !self.member_id.parse::<u64>().is_ok_and(|id| id > 0) {
            return Err("member_id must be a Discord snowflake");
        }
        if !matches!(
            self.action.as_str(),
            "note" | "warn" | "timeout" | "kick" | "ban" | "unban"
        ) {
            return Err("action must be note, warn, timeout, kick, ban, or unban");
        }
        if self.reason.trim().is_empty() || self.reason.chars().count() > 1_000 {
            return Err("reason must contain between 1 and 1,000 characters");
        }
        Ok(())
    }
}

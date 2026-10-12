use chrono::{DateTime, Utc};
use serde::Serialize;
use sqlx::FromRow;

#[derive(Clone, Debug, FromRow, Serialize)]
pub struct EconomyAccount {
    pub guild_id: String,
    pub user_id: String,
    pub balance: i64,
    pub lifetime_earned: i64,
    pub lifetime_spent: i64,
    pub updated_at: DateTime<Utc>,
}
#[derive(Clone, Debug, FromRow, Serialize)]
pub struct EconomyTransaction {
    pub id: i64,
    pub guild_id: String,
    pub user_id: String,
    pub counterparty_id: Option<String>,
    pub amount: i64,
    pub kind: String,
    pub reference: String,
    pub metadata: serde_json::Value,
    pub created_at: DateTime<Utc>,
}

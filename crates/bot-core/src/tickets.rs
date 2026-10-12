use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sqlx::FromRow;

#[derive(Clone, Debug, Serialize, Deserialize, FromRow)]
pub struct TicketPanel {
    pub id: i64,
    pub guild_id: String,
    pub name: String,
    pub title: String,
    pub description: String,
    pub panel_channel_id: Option<String>,
    pub panel_message_id: Option<String>,
    pub category_id: String,
    pub support_role_ids: Value,
    pub button_label: String,
    pub welcome_message: String,
    pub channel_name: String,
    pub intake_enabled: bool,
    pub intake_label: String,
    pub max_open_per_user: i16,
    pub enabled: bool,
    pub created_by: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct NewTicketPanel {
    pub guild_id: String,
    pub name: String,
    pub title: String,
    pub description: String,
    pub panel_channel_id: Option<String>,
    pub category_id: String,
    pub support_role_ids: Value,
    pub button_label: String,
    pub welcome_message: String,
    pub channel_name: String,
    pub intake_enabled: bool,
    pub intake_label: String,
    pub max_open_per_user: i16,
    pub enabled: bool,
    pub created_by: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, FromRow)]
pub struct Ticket {
    pub id: i64,
    pub guild_id: String,
    pub panel_id: Option<i64>,
    pub opener_id: String,
    pub opener_username: Option<String>,
    pub opener_display_name: Option<String>,
    pub channel_id: String,
    pub subject: String,
    pub opening_note: String,
    pub status: String,
    pub claimed_by: Option<String>,
    pub assigned_to: Option<String>,
    pub closed_by: Option<String>,
    pub closed_reason: Option<String>,
    pub transcript: Option<String>,
    pub opened_at: DateTime<Utc>,
    pub closed_at: Option<DateTime<Utc>>,
    pub reopened_at: Option<DateTime<Utc>>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Clone, Debug)]
pub struct NewTicket {
    pub guild_id: String,
    pub panel_id: Option<i64>,
    pub opener_id: String,
    pub channel_id: String,
    pub subject: String,
    pub opening_note: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, FromRow)]
pub struct TicketEvent {
    pub id: i64,
    pub ticket_id: i64,
    pub guild_id: String,
    pub actor_id: String,
    pub action: String,
    pub metadata: Value,
    pub created_at: DateTime<Utc>,
}

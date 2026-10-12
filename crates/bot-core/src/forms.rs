use chrono::{DateTime, Utc};
use serde::Serialize;
use sqlx::FromRow;

#[derive(Clone, Debug, Serialize, FromRow)]
pub struct FormSubmission {
    pub id: i64,
    pub guild_id: String,
    pub form_id: String,
    pub submitter_id: Option<String>,
    pub username: Option<String>,
    pub display_name: Option<String>,
    pub answers: serde_json::Value,
    pub created_at: DateTime<Utc>,
}

#[derive(Clone, Debug)]
pub struct NewFormSubmission {
    pub guild_id: String,
    pub form_id: String,
    pub submitter_id: Option<String>,
    pub answers: serde_json::Value,
}

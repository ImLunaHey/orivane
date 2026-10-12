use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sqlx::FromRow;

#[derive(Clone, Debug, Serialize, Deserialize, FromRow)]
pub struct AuditEvent {
    pub id: i64,
    pub occurred_at: DateTime<Utc>,
    pub scope: String,
    pub guild_id: Option<String>,
    pub actor_id: String,
    pub actor_kind: String,
    pub action: String,
    pub target_type: String,
    pub target_id: Option<String>,
    pub outcome: String,
    pub source: String,
    pub request_id: Option<String>,
    pub reason: Option<String>,
    pub metadata: Value,
    pub before_state: Option<Value>,
    pub after_state: Option<Value>,
}

#[derive(Clone, Debug)]
pub struct NewAuditEvent {
    pub scope: String,
    pub guild_id: Option<String>,
    pub actor_id: String,
    pub actor_kind: String,
    pub action: String,
    pub target_type: String,
    pub target_id: Option<String>,
    pub outcome: String,
    pub source: String,
    pub request_id: Option<String>,
    pub reason: Option<String>,
    pub metadata: Value,
    pub before_state: Option<Value>,
    pub after_state: Option<Value>,
}

impl NewAuditEvent {
    pub fn global(actor_id: impl Into<String>, action: impl Into<String>) -> Self {
        Self {
            scope: "global".to_owned(),
            guild_id: None,
            actor_id: actor_id.into(),
            actor_kind: "user".to_owned(),
            action: action.into(),
            target_type: "bot".to_owned(),
            target_id: None,
            outcome: "success".to_owned(),
            source: "unknown".to_owned(),
            request_id: None,
            reason: None,
            metadata: serde_json::json!({}),
            before_state: None,
            after_state: None,
        }
    }

    pub fn guild(
        guild_id: impl Into<String>,
        actor_id: impl Into<String>,
        action: impl Into<String>,
    ) -> Self {
        Self {
            scope: "guild".to_owned(),
            guild_id: Some(guild_id.into()),
            actor_id: actor_id.into(),
            actor_kind: "user".to_owned(),
            action: action.into(),
            target_type: "guild".to_owned(),
            target_id: None,
            outcome: "success".to_owned(),
            source: "unknown".to_owned(),
            request_id: None,
            reason: None,
            metadata: serde_json::json!({}),
            before_state: None,
            after_state: None,
        }
    }
}

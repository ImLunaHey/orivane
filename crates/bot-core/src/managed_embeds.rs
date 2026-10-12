use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Clone, Debug, Default, Deserialize, Serialize, PartialEq, Eq)]
pub struct EmbedField {
    pub name: String,
    pub value: String,
    #[serde(default)]
    pub inline: bool,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct EmbedPayload {
    #[serde(default)]
    pub content: String,
    #[serde(default)]
    pub title: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub url: String,
    #[serde(default = "default_color")]
    pub color: u32,
    #[serde(default)]
    pub author_name: String,
    #[serde(default)]
    pub author_url: String,
    #[serde(default)]
    pub author_icon_url: String,
    #[serde(default)]
    pub footer_text: String,
    #[serde(default)]
    pub footer_icon_url: String,
    #[serde(default)]
    pub thumbnail_url: String,
    #[serde(default)]
    pub image_url: String,
    #[serde(default)]
    pub timestamp: bool,
    #[serde(default)]
    pub fields: Vec<EmbedField>,
}

impl Default for EmbedPayload {
    fn default() -> Self {
        Self {
            content: String::new(),
            title: String::new(),
            description: String::new(),
            url: String::new(),
            color: default_color(),
            author_name: String::new(),
            author_url: String::new(),
            author_icon_url: String::new(),
            footer_text: String::new(),
            footer_icon_url: String::new(),
            thumbnail_url: String::new(),
            image_url: String::new(),
            timestamp: false,
            fields: Vec::new(),
        }
    }
}

const fn default_color() -> u32 {
    5_797_266
}

#[derive(Clone, Debug)]
pub struct NewEmbedTemplate {
    pub guild_id: String,
    pub name: String,
    pub payload: EmbedPayload,
    pub created_by: String,
}

#[derive(Clone, Debug, FromRow, Serialize)]
pub struct EmbedTemplate {
    pub id: i64,
    pub guild_id: String,
    pub name: String,
    pub payload: serde_json::Value,
    pub created_by: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Clone, Debug)]
pub struct NewEmbedDelivery {
    pub guild_id: String,
    pub template_id: i64,
    pub channel_id: String,
    pub message_id: Option<String>,
    pub actor_id: String,
    pub action: String,
    pub status: String,
    pub error: Option<String>,
}

#[derive(Clone, Debug, FromRow, Serialize)]
pub struct EmbedDelivery {
    pub id: i64,
    pub guild_id: String,
    pub template_id: i64,
    pub channel_id: String,
    pub message_id: Option<String>,
    pub actor_id: String,
    pub action: String,
    pub status: String,
    pub error: Option<String>,
    pub created_at: DateTime<Utc>,
}

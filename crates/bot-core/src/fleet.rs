use chrono::{DateTime, Utc};
use serde::Serialize;
use sqlx::FromRow;

#[derive(Clone, Debug, Serialize, FromRow)]
pub struct ConnectedGuild {
    pub id: String,
    pub name: String,
    pub members: i64,
    pub commands: i64,
    pub status: String,
    pub shard: i64,
    pub installer_id: Option<String>,
    pub installer_name: Option<String>,
    pub owner_id: Option<String>,
    pub installed_at: DateTime<Utc>,
}

#[derive(Clone, Debug, Serialize, FromRow)]
pub struct GlobalUser {
    pub id: String,
    pub username: Option<String>,
    pub display_name: Option<String>,
    pub avatar_url: Option<String>,
    pub servers: i64,
    pub installed_servers: i64,
    pub install_commands_30d: i64,
    pub commands: i64,
    pub cases: i64,
    pub last_seen: Option<DateTime<Utc>>,
}

#[derive(Clone, Debug, Serialize, FromRow)]
pub struct UserGuildContext {
    pub server_id: String,
    pub server_name: String,
    pub relationship: String,
    pub commands: i64,
    pub cases: i64,
    pub last_seen: Option<DateTime<Utc>>,
}

#[derive(Clone, Debug, Serialize, FromRow)]
pub struct GuildDashboardStats {
    pub commands_today: i64,
    pub active_members_30d: i64,
    pub automod_actions_today: i64,
    pub open_cases: i64,
    pub p95_latency_ms: Option<f64>,
}

#[derive(Clone, Debug, Serialize, FromRow)]
pub struct CommandVolumeBucket {
    pub occurred_at: DateTime<Utc>,
    pub value: i64,
}

#[derive(Clone, Debug, Serialize)]
pub struct DependencyHealth {
    pub name: &'static str,
    pub status: &'static str,
    pub latency_ms: u64,
}

#[derive(Clone, Debug, Serialize)]
pub struct InfrastructureHealth {
    pub status: &'static str,
    pub services: Vec<DependencyHealth>,
}

#[derive(Clone, Debug, Serialize)]
pub struct RuntimeMetrics {
    pub database_pool_size: u32,
    pub database_pool_idle: usize,
    pub due_job_backlog: i64,
}

#[derive(Clone, Debug, Serialize, FromRow)]
pub struct GuildMemberSummary {
    pub id: String,
    pub username: Option<String>,
    pub display_name: Option<String>,
    pub avatar_url: Option<String>,
    pub commands_30d: i64,
    pub cases: i64,
    pub last_seen: Option<DateTime<Utc>>,
}

#[derive(Clone, Debug, Serialize)]
pub struct MemberXpAward {
    pub awarded: bool,
    pub total_xp: u64,
    pub level: u32,
    pub leveled_up: bool,
}

#[derive(Clone, Debug, Serialize, FromRow)]
pub struct MemberXpProfile {
    pub user_id: String,
    pub total_xp: i64,
    pub message_count: i64,
    pub rank: i64,
}

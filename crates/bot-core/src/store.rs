use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use redis::{AsyncCommands as _, aio::ConnectionManager};
use sqlx::{PgPool, postgres::PgPoolOptions};
use thiserror::Error;

use crate::{
    AuditEvent, CommandVolumeBucket, ConnectedGuild, DependencyHealth, GlobalUser, GuildConfig,
    GuildDashboardStats, GuildMemberSummary, InfrastructureHealth, MemberXpAward, MemberXpProfile,
    ModerationCase, NewAuditEvent, NewModerationCase, UserGuildContext,
    config::ConfigValidationError,
};

const CONFIG_CACHE_TTL_SECONDS: u64 = 300;

#[derive(Clone)]
pub struct Infrastructure {
    pool: PgPool,
    redis: ConnectionManager,
}

#[derive(Debug, Error)]
pub enum StoreError {
    #[error("database error: {0}")]
    Database(#[from] sqlx::Error),
    #[error("database migration error: {0}")]
    Migration(#[from] sqlx::migrate::MigrateError),
    #[error("cache error: {0}")]
    Cache(#[from] redis::RedisError),
    #[error("configuration is invalid: {0}")]
    InvalidConfig(#[from] ConfigValidationError),
    #[error("stored configuration is malformed: {0}")]
    Serialization(#[from] serde_json::Error),
}

impl Infrastructure {
    pub async fn record_raid_join(
        &self,
        guild_id: &str,
        user_id: &str,
        window_seconds: u16,
    ) -> Result<u64, StoreError> {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;
        let key = format!("anti-raid:joins:{guild_id}");
        let member = format!("{now}:{user_id}");
        let cutoff = now.saturating_sub(u64::from(window_seconds) * 1_000);
        let mut cache = self.redis.clone();
        let _: () = redis::cmd("ZADD")
            .arg(&key)
            .arg(now)
            .arg(member)
            .query_async(&mut cache)
            .await?;
        let _: () = redis::cmd("ZREMRANGEBYSCORE")
            .arg(&key)
            .arg("-inf")
            .arg(cutoff)
            .query_async(&mut cache)
            .await?;
        let count: u64 = redis::cmd("ZCARD")
            .arg(&key)
            .query_async(&mut cache)
            .await?;
        let _: () = redis::cmd("EXPIRE")
            .arg(&key)
            .arg(u64::from(window_seconds) + 5)
            .query_async(&mut cache)
            .await?;
        Ok(count)
    }

    pub async fn set_raid_lockdown(
        &self,
        guild_id: &str,
        duration_seconds: u64,
    ) -> Result<(), StoreError> {
        let mut cache = self.redis.clone();
        let _: () = redis::cmd("SET")
            .arg(format!("anti-raid:lockdown:{guild_id}"))
            .arg(1)
            .arg("EX")
            .arg(duration_seconds)
            .query_async(&mut cache)
            .await?;
        Ok(())
    }

    pub async fn clear_raid_lockdown(&self, guild_id: &str) -> Result<(), StoreError> {
        let mut cache = self.redis.clone();
        let _: () = cache.del(format!("anti-raid:lockdown:{guild_id}")).await?;
        Ok(())
    }

    pub async fn raid_lockdown_active(&self, guild_id: &str) -> Result<bool, StoreError> {
        let mut cache = self.redis.clone();
        cache
            .exists(format!("anti-raid:lockdown:{guild_id}"))
            .await
            .map_err(StoreError::from)
    }

    pub async fn connect(database_url: &str, redis_url: &str) -> Result<Self, StoreError> {
        let pool = PgPoolOptions::new()
            .max_connections(20)
            .acquire_timeout(Duration::from_secs(5))
            .connect(database_url)
            .await?;
        sqlx::migrate!("./migrations").run(&pool).await?;

        let client = redis::Client::open(redis_url)?;
        let redis = ConnectionManager::new(client).await?;

        Ok(Self { pool, redis })
    }

    pub async fn guild_config(&self, guild_id: &str) -> Result<GuildConfig, StoreError> {
        let cache_key = format!("guild-config:{guild_id}");
        let mut cache = self.redis.clone();
        let cached: Option<String> = cache.get(&cache_key).await?;
        if let Some(cached) = cached {
            return Ok(serde_json::from_str(&cached)?);
        }

        let stored: Option<serde_json::Value> =
            sqlx::query_scalar("SELECT config FROM guild_configs WHERE guild_id = $1")
                .bind(guild_id)
                .fetch_optional(&self.pool)
                .await?;
        let config = match stored {
            Some(value) => serde_json::from_value(value)?,
            None => GuildConfig::for_guild(guild_id),
        };

        let serialized = serde_json::to_string(&config)?;
        let _: () = cache
            .set_ex(cache_key, serialized, CONFIG_CACHE_TTL_SECONDS)
            .await?;
        Ok(config)
    }

    pub async fn put_guild_config(&self, config: &GuildConfig) -> Result<(), StoreError> {
        config.validate()?;
        let value = serde_json::to_value(config)?;
        sqlx::query(
            "INSERT INTO guild_configs (guild_id, config) VALUES ($1, $2) \
             ON CONFLICT (guild_id) DO UPDATE SET config = EXCLUDED.config, updated_at = now()",
        )
        .bind(&config.guild_id)
        .bind(value)
        .execute(&self.pool)
        .await?;

        let mut cache = self.redis.clone();
        let _: () = cache
            .del(format!("guild-config:{}", config.guild_id))
            .await?;
        Ok(())
    }

    pub async fn put_guild_config_audited(
        &self,
        config: &GuildConfig,
        event: &NewAuditEvent,
    ) -> Result<AuditEvent, StoreError> {
        config.validate()?;
        let value = serde_json::to_value(config)?;
        let mut transaction = self.pool.begin().await?;
        sqlx::query(
            "INSERT INTO guild_configs (guild_id, config) VALUES ($1, $2) \
             ON CONFLICT (guild_id) DO UPDATE SET config = EXCLUDED.config, updated_at = now()",
        )
        .bind(&config.guild_id)
        .bind(value)
        .execute(&mut *transaction)
        .await?;
        let stored = insert_audit_event(&mut transaction, event).await?;
        transaction.commit().await?;

        let mut cache = self.redis.clone();
        let _: () = cache
            .del(format!("guild-config:{}", config.guild_id))
            .await?;
        Ok(stored)
    }

    pub async fn record_audit_event(
        &self,
        event: &NewAuditEvent,
    ) -> Result<AuditEvent, StoreError> {
        let mut transaction = self.pool.begin().await?;
        let stored = insert_audit_event(&mut transaction, event).await?;
        transaction.commit().await?;
        Ok(stored)
    }

    pub async fn audit_events(
        &self,
        guild_id: Option<&str>,
        limit: i64,
        before_id: Option<i64>,
    ) -> Result<Vec<AuditEvent>, StoreError> {
        let limit = limit.clamp(1, 200);
        sqlx::query_as::<_, AuditEvent>(
            "SELECT id, occurred_at, scope, guild_id, actor_id, actor_kind, action, target_type, \
                    target_id, outcome, source, request_id, reason, metadata, before_state, after_state \
             FROM audit_events \
             WHERE ($1::text IS NULL OR guild_id = $1) AND ($2::bigint IS NULL OR id < $2) \
             ORDER BY id DESC LIMIT $3",
        )
        .bind(guild_id)
        .bind(before_id)
        .bind(limit)
        .fetch_all(&self.pool)
        .await
        .map_err(StoreError::from)
    }

    pub async fn audit_event(
        &self,
        event_id: i64,
        guild_id: Option<&str>,
    ) -> Result<Option<AuditEvent>, StoreError> {
        sqlx::query_as::<_, AuditEvent>(
            "SELECT id, occurred_at, scope, guild_id, actor_id, actor_kind, action, target_type, \
                    target_id, outcome, source, request_id, reason, metadata, before_state, after_state \
             FROM audit_events WHERE id = $1 AND ($2::text IS NULL OR guild_id = $2)",
        )
        .bind(event_id)
        .bind(guild_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(StoreError::from)
    }

    pub async fn moderation_cases(
        &self,
        guild_id: &str,
        limit: i64,
    ) -> Result<Vec<ModerationCase>, StoreError> {
        sqlx::query_as::<_, ModerationCase>(
            "SELECT id, guild_id, member_id, moderator_id, action, reason, status, created_at, updated_at \
             FROM moderation_cases WHERE guild_id = $1 ORDER BY id DESC LIMIT $2",
        )
        .bind(guild_id)
        .bind(limit.clamp(1, 200))
        .fetch_all(&self.pool)
        .await
        .map_err(StoreError::from)
    }

    pub async fn moderation_case(
        &self,
        guild_id: &str,
        case_id: i64,
    ) -> Result<Option<ModerationCase>, StoreError> {
        sqlx::query_as::<_, ModerationCase>(
            "SELECT id, guild_id, member_id, moderator_id, action, reason, status, created_at, updated_at \
             FROM moderation_cases WHERE guild_id = $1 AND id = $2",
        )
        .bind(guild_id)
        .bind(case_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(StoreError::from)
    }

    pub async fn create_moderation_case_audited(
        &self,
        guild_id: &str,
        moderator_id: &str,
        input: &NewModerationCase,
        event: &NewAuditEvent,
    ) -> Result<ModerationCase, StoreError> {
        let mut transaction = self.pool.begin().await?;
        let case = sqlx::query_as::<_, ModerationCase>(
            "INSERT INTO moderation_cases (guild_id, member_id, moderator_id, action, reason) \
             VALUES ($1, $2, $3, $4, $5) \
             RETURNING id, guild_id, member_id, moderator_id, action, reason, status, created_at, updated_at",
        )
        .bind(guild_id)
        .bind(&input.member_id)
        .bind(moderator_id)
        .bind(&input.action)
        .bind(input.reason.trim())
        .fetch_one(&mut *transaction)
        .await?;
        insert_audit_event(&mut transaction, event).await?;
        transaction.commit().await?;
        Ok(case)
    }

    pub async fn award_member_xp(
        &self,
        guild_id: &str,
        user_id: &str,
        amount: u16,
        cooldown_seconds: u16,
    ) -> Result<MemberXpAward, StoreError> {
        let cooldown_key = format!("xp-cooldown:{guild_id}:{user_id}");
        let mut cache = self.redis.clone();
        let acquired: Option<String> = redis::cmd("SET")
            .arg(&cooldown_key)
            .arg("1")
            .arg("EX")
            .arg(cooldown_seconds.max(1))
            .arg("NX")
            .query_async(&mut cache)
            .await?;
        if acquired.is_none() {
            let current: Option<i64> =
                sqlx::query_scalar("SELECT xp FROM member_xp WHERE guild_id = $1 AND user_id = $2")
                    .bind(guild_id)
                    .bind(user_id)
                    .fetch_optional(&self.pool)
                    .await?;
            let total_xp = u64::try_from(current.unwrap_or_default()).unwrap_or_default();
            return Ok(MemberXpAward {
                awarded: false,
                total_xp,
                level: level_for_xp(total_xp),
                leveled_up: false,
            });
        }

        let total: i64 = sqlx::query_scalar(
            "INSERT INTO member_xp (guild_id, user_id, xp, message_count) VALUES ($1, $2, $3, 1) \
             ON CONFLICT (guild_id, user_id) DO UPDATE SET \
                xp = member_xp.xp + EXCLUDED.xp, \
                message_count = member_xp.message_count + 1, updated_at = now() \
             RETURNING xp",
        )
        .bind(guild_id)
        .bind(user_id)
        .bind(i64::from(amount))
        .fetch_one(&self.pool)
        .await?;
        let total_xp = u64::try_from(total).unwrap_or_default();
        let previous_xp = total_xp.saturating_sub(u64::from(amount));
        let level = level_for_xp(total_xp);
        Ok(MemberXpAward {
            awarded: true,
            total_xp,
            level,
            leveled_up: level > level_for_xp(previous_xp),
        })
    }

    pub async fn member_xp_profile(
        &self,
        guild_id: &str,
        user_id: &str,
    ) -> Result<MemberXpProfile, StoreError> {
        let profile = sqlx::query_as::<_, MemberXpProfile>(
            "SELECT user_id, xp AS total_xp, message_count, \
                (SELECT count(*) + 1 FROM member_xp ranked \
                 WHERE ranked.guild_id = member_xp.guild_id AND ranked.xp > member_xp.xp)::bigint AS rank \
             FROM member_xp WHERE guild_id = $1 AND user_id = $2",
        )
        .bind(guild_id)
        .bind(user_id)
        .fetch_optional(&self.pool)
        .await?;
        Ok(profile.unwrap_or_else(|| MemberXpProfile {
            user_id: user_id.to_owned(),
            total_xp: 0,
            message_count: 0,
            rank: 0,
        }))
    }

    pub async fn xp_leaderboard(
        &self,
        guild_id: &str,
        limit: i64,
    ) -> Result<Vec<MemberXpProfile>, StoreError> {
        sqlx::query_as::<_, MemberXpProfile>(
            "SELECT user_id, xp AS total_xp, message_count, \
                    row_number() OVER (ORDER BY xp DESC, updated_at ASC)::bigint AS rank \
             FROM member_xp WHERE guild_id = $1 ORDER BY xp DESC, updated_at ASC LIMIT $2",
        )
        .bind(guild_id)
        .bind(limit.clamp(1, 25))
        .fetch_all(&self.pool)
        .await
        .map_err(StoreError::from)
    }

    pub async fn open_ticket_channel(
        &self,
        guild_id: &str,
        opener_id: &str,
    ) -> Result<Option<String>, StoreError> {
        sqlx::query_scalar(
            "SELECT channel_id FROM tickets WHERE guild_id = $1 AND opener_id = $2 AND status = 'open'",
        )
        .bind(guild_id)
        .bind(opener_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(StoreError::from)
    }

    pub async fn record_open_ticket(
        &self,
        guild_id: &str,
        opener_id: &str,
        channel_id: &str,
    ) -> Result<(), StoreError> {
        sqlx::query("INSERT INTO tickets (guild_id, opener_id, channel_id) VALUES ($1, $2, $3)")
            .bind(guild_id)
            .bind(opener_id)
            .bind(channel_id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    pub async fn close_ticket_channel(
        &self,
        guild_id: &str,
        channel_id: &str,
        closed_by: &str,
    ) -> Result<bool, StoreError> {
        let result = sqlx::query(
            "UPDATE tickets SET status = 'closed', closed_at = now(), closed_by = $3 \
             WHERE guild_id = $1 AND channel_id = $2 AND status = 'open'",
        )
        .bind(guild_id)
        .bind(channel_id)
        .bind(closed_by)
        .execute(&self.pool)
        .await?;
        Ok(result.rows_affected() == 1)
    }

    pub async fn claim_starboard_post(
        &self,
        guild_id: &str,
        source_message_id: &str,
        starboard_channel_id: &str,
    ) -> Result<bool, StoreError> {
        let result = sqlx::query(
            "INSERT INTO starboard_posts (guild_id, source_message_id, starboard_channel_id) \
             VALUES ($1, $2, $3) ON CONFLICT (guild_id, source_message_id) DO NOTHING",
        )
        .bind(guild_id)
        .bind(source_message_id)
        .bind(starboard_channel_id)
        .execute(&self.pool)
        .await?;
        Ok(result.rows_affected() == 1)
    }

    pub async fn health(&self) -> Result<(), StoreError> {
        sqlx::query("SELECT 1").execute(&self.pool).await?;
        let mut cache = self.redis.clone();
        let _: String = redis::cmd("PING").query_async(&mut cache).await?;
        Ok(())
    }

    pub async fn health_report(&self) -> InfrastructureHealth {
        let database_started = Instant::now();
        let database_ok = sqlx::query("SELECT 1").execute(&self.pool).await.is_ok();
        let database_latency = database_started
            .elapsed()
            .as_millis()
            .min(u128::from(u64::MAX)) as u64;

        let redis_started = Instant::now();
        let mut cache = self.redis.clone();
        let redis_ok = redis::cmd("PING")
            .query_async::<String>(&mut cache)
            .await
            .is_ok();
        let redis_latency = redis_started
            .elapsed()
            .as_millis()
            .min(u128::from(u64::MAX)) as u64;

        InfrastructureHealth {
            status: if database_ok && redis_ok {
                "operational"
            } else {
                "degraded"
            },
            services: vec![
                DependencyHealth {
                    name: "PostgreSQL",
                    status: if database_ok {
                        "operational"
                    } else {
                        "unavailable"
                    },
                    latency_ms: database_latency,
                },
                DependencyHealth {
                    name: "Redis",
                    status: if redis_ok {
                        "operational"
                    } else {
                        "unavailable"
                    },
                    latency_ms: redis_latency,
                },
            ],
        }
    }

    pub async fn upsert_connected_guild(
        &self,
        guild_id: &str,
        name: &str,
        member_count: u64,
        shard_id: u64,
        owner_id: &str,
    ) -> Result<(), StoreError> {
        sqlx::query(
            "INSERT INTO connected_guilds (guild_id, name, member_count, shard_id, owner_id) \
             VALUES ($1, $2, $3, $4, $5) \
             ON CONFLICT (guild_id) DO UPDATE SET name = EXCLUDED.name, member_count = EXCLUDED.member_count, \
             shard_id = EXCLUDED.shard_id, owner_id = EXCLUDED.owner_id, connected = true, \
             removed_at = NULL, updated_at = now()",
        )
        .bind(guild_id)
        .bind(name)
        .bind(i64::try_from(member_count).unwrap_or(i64::MAX))
        .bind(i64::try_from(shard_id).unwrap_or(i64::MAX))
        .bind(owner_id)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn mark_guild_disconnected(&self, guild_id: &str) -> Result<(), StoreError> {
        sqlx::query(
            "UPDATE connected_guilds SET connected = false, removed_at = now(), updated_at = now() WHERE guild_id = $1",
        )
        .bind(guild_id)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn attribute_guild_installer(
        &self,
        guild_id: &str,
        installer_id: &str,
    ) -> Result<(), StoreError> {
        sqlx::query(
            "UPDATE connected_guilds SET installer_id = COALESCE(installer_id, $2), updated_at = now() WHERE guild_id = $1",
        )
        .bind(guild_id)
        .bind(installer_id)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn connected_guilds(&self) -> Result<Vec<ConnectedGuild>, StoreError> {
        sqlx::query_as::<_, ConnectedGuild>(
            "SELECT guild_id AS id, name, member_count AS members, \
                    (SELECT count(*) FROM audit_events a WHERE a.guild_id = g.guild_id \
                     AND a.action LIKE 'slash_command.%' AND a.occurred_at >= date_trunc('day', now())) AS commands, \
                    'Connected'::text AS status, shard_id AS shard, installer_id, \
                    COALESCE((SELECT d.display_name FROM discord_identities d WHERE d.user_id = g.installer_id), \
                             (SELECT d.username FROM discord_identities d WHERE d.user_id = g.installer_id), \
                             (SELECT u.name FROM account oauth_account JOIN \"user\" u ON u.id = oauth_account.\"userId\" \
                              WHERE oauth_account.\"providerId\" = 'discord' AND oauth_account.\"accountId\" = g.installer_id LIMIT 1)) \
                    AS installer_name, owner_id, installed_at \
             FROM connected_guilds g WHERE connected ORDER BY member_count DESC",
        )
        .fetch_all(&self.pool)
        .await
        .map_err(StoreError::from)
    }

    pub async fn connected_guild_name(&self, guild_id: &str) -> Result<Option<String>, StoreError> {
        sqlx::query_scalar("SELECT name FROM connected_guilds WHERE guild_id = $1")
            .bind(guild_id)
            .fetch_optional(&self.pool)
            .await
            .map_err(StoreError::from)
    }

    pub async fn global_users(&self) -> Result<Vec<GlobalUser>, StoreError> {
        sqlx::query_as::<_, GlobalUser>(
            "WITH identities AS ( \
                 SELECT actor_id AS user_id FROM audit_events WHERE actor_kind = 'user' AND actor_id ~ '^[1-9][0-9]{0,19}$' \
                 UNION SELECT installer_id FROM connected_guilds WHERE installer_id IS NOT NULL \
             ) \
             SELECT i.user_id AS id, \
                    (SELECT d.username FROM discord_identities d WHERE d.user_id = i.user_id) AS username, \
                    COALESCE((SELECT d.display_name FROM discord_identities d WHERE d.user_id = i.user_id), \
                             (SELECT u.name FROM account oauth_account JOIN \"user\" u ON u.id = oauth_account.\"userId\" \
                              WHERE oauth_account.\"providerId\" = 'discord' AND oauth_account.\"accountId\" = i.user_id LIMIT 1)) \
                    AS display_name, \
                    count(DISTINCT a.guild_id) AS servers, \
                    (SELECT count(*) FROM connected_guilds g WHERE g.installer_id = i.user_id AND g.connected) AS installed_servers, \
                    count(*) FILTER (WHERE a.action LIKE 'slash_command.%' AND a.occurred_at >= now() - interval '30 days' \
                     AND EXISTS (SELECT 1 FROM connected_guilds g WHERE g.installer_id = i.user_id AND g.guild_id = a.guild_id)) AS install_commands_30d, \
                    count(*) FILTER (WHERE a.action LIKE 'slash_command.%' AND a.occurred_at >= now() - interval '30 days') AS commands, \
                    count(*) FILTER (WHERE a.action LIKE 'moderation.case.%') AS cases, \
                    max(a.occurred_at) AS last_seen \
             FROM identities i LEFT JOIN audit_events a ON a.actor_id = i.user_id \
             GROUP BY i.user_id ORDER BY last_seen DESC NULLS LAST",
        )
        .fetch_all(&self.pool)
        .await
        .map_err(StoreError::from)
    }

    pub async fn user_guild_contexts(
        &self,
        user_id: &str,
    ) -> Result<Vec<UserGuildContext>, StoreError> {
        sqlx::query_as::<_, UserGuildContext>(
            "WITH relevant AS ( \
                 SELECT guild_id FROM audit_events WHERE actor_id = $1 AND guild_id IS NOT NULL \
                 UNION SELECT guild_id FROM connected_guilds WHERE installer_id = $1 \
             ) \
             SELECT r.guild_id AS server_id, COALESCE(g.name, r.guild_id) AS server_name, \
                    CASE WHEN g.installer_id = $1 THEN 'Installer' ELSE 'Member' END AS relationship, \
                    count(a.id) FILTER (WHERE a.action LIKE 'slash_command.%' AND a.occurred_at >= now() - interval '30 days') AS commands, \
                    count(a.id) FILTER (WHERE a.action LIKE 'moderation.case.%') AS cases, \
                    max(a.occurred_at) AS last_seen \
             FROM relevant r LEFT JOIN connected_guilds g ON g.guild_id = r.guild_id \
             LEFT JOIN audit_events a ON a.guild_id = r.guild_id AND a.actor_id = $1 \
             GROUP BY r.guild_id, g.name, g.installer_id ORDER BY last_seen DESC NULLS LAST",
        )
        .bind(user_id)
        .fetch_all(&self.pool)
        .await
        .map_err(StoreError::from)
    }

    pub async fn guild_members(
        &self,
        guild_id: &str,
    ) -> Result<Vec<GuildMemberSummary>, StoreError> {
        sqlx::query_as::<_, GuildMemberSummary>(
            "WITH identities AS ( \
               SELECT actor_id AS user_id FROM audit_events WHERE guild_id = $1 AND actor_kind = 'user' AND actor_id ~ '^[1-9][0-9]{0,19}$' \
               UNION SELECT member_id FROM moderation_cases WHERE guild_id = $1 \
             ) \
             SELECT i.user_id AS id, \
                    count(a.id) FILTER (WHERE a.action LIKE 'slash_command.%' AND a.occurred_at >= now() - interval '30 days') AS commands_30d, \
                    (SELECT count(*) FROM moderation_cases c WHERE c.guild_id = $1 AND c.member_id = i.user_id) AS cases, \
                    max(a.occurred_at) AS last_seen \
             FROM identities i LEFT JOIN audit_events a ON a.guild_id = $1 AND a.actor_id = i.user_id \
             GROUP BY i.user_id ORDER BY last_seen DESC NULLS LAST",
        )
        .bind(guild_id)
        .fetch_all(&self.pool)
        .await
        .map_err(StoreError::from)
    }

    pub async fn guild_dashboard_stats(
        &self,
        guild_id: &str,
    ) -> Result<GuildDashboardStats, StoreError> {
        sqlx::query_as::<_, GuildDashboardStats>(
            "SELECT \
               count(*) FILTER (WHERE action LIKE 'slash_command.%' AND occurred_at >= date_trunc('day', now())) AS commands_today, \
               count(DISTINCT actor_id) FILTER (WHERE action LIKE 'slash_command.%' AND occurred_at >= now() - interval '30 days') AS active_members_30d, \
               count(*) FILTER (WHERE action LIKE 'automod.%' AND occurred_at >= date_trunc('day', now())) AS automod_actions_today, \
               (SELECT count(*) FROM moderation_cases WHERE guild_id = $1 AND status = 'open') AS open_cases, \
               percentile_cont(0.95) WITHIN GROUP (ORDER BY (metadata->>'latency_ms')::double precision) \
                 FILTER (WHERE action LIKE 'slash_command.%' AND metadata ? 'latency_ms' AND occurred_at >= now() - interval '24 hours') AS p95_latency_ms \
             FROM audit_events WHERE guild_id = $1",
        )
        .bind(guild_id)
        .fetch_one(&self.pool)
        .await
        .map_err(StoreError::from)
    }

    pub async fn command_volume(
        &self,
        guild_id: Option<&str>,
    ) -> Result<Vec<CommandVolumeBucket>, StoreError> {
        sqlx::query_as::<_, CommandVolumeBucket>(
            "WITH buckets AS ( \
               SELECT generate_series(date_trunc('hour', now()) - interval '24 hours', date_trunc('hour', now()), interval '4 hours') AS occurred_at \
             ) \
             SELECT b.occurred_at, count(a.id) AS value FROM buckets b \
             LEFT JOIN audit_events a ON a.action LIKE 'slash_command.%' \
               AND a.occurred_at >= b.occurred_at AND a.occurred_at < b.occurred_at + interval '4 hours' \
               AND ($1::text IS NULL OR a.guild_id = $1) \
             GROUP BY b.occurred_at ORDER BY b.occurred_at",
        )
        .bind(guild_id)
        .fetch_all(&self.pool)
        .await
        .map_err(StoreError::from)
    }
}

pub fn level_for_xp(xp: u64) -> u32 {
    let mut level = 0_u32;
    while xp >= 100_u64.saturating_mul(u64::from(level + 1).pow(2)) {
        level = level.saturating_add(1);
    }
    level
}

async fn insert_audit_event(
    transaction: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    event: &NewAuditEvent,
) -> Result<AuditEvent, sqlx::Error> {
    sqlx::query_as::<_, AuditEvent>(
        "INSERT INTO audit_events \
         (scope, guild_id, actor_id, actor_kind, action, target_type, target_id, outcome, source, \
          request_id, reason, metadata, before_state, after_state) \
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14) \
         RETURNING id, occurred_at, scope, guild_id, actor_id, actor_kind, action, target_type, \
                   target_id, outcome, source, request_id, reason, metadata, before_state, after_state",
    )
    .bind(&event.scope)
    .bind(&event.guild_id)
    .bind(&event.actor_id)
    .bind(&event.actor_kind)
    .bind(&event.action)
    .bind(&event.target_type)
    .bind(&event.target_id)
    .bind(&event.outcome)
    .bind(&event.source)
    .bind(&event.request_id)
    .bind(&event.reason)
    .bind(&event.metadata)
    .bind(&event.before_state)
    .bind(&event.after_state)
    .fetch_one(&mut **transaction)
    .await
}

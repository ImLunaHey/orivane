use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use redis::{AsyncCommands as _, aio::ConnectionManager};
use sqlx::{AssertSqlSafe, PgPool, postgres::PgPoolOptions};
use thiserror::Error;

use crate::{
    AfkStatus, AuditEvent, CommandVolumeBucket, ConnectedGuild, CounterRun, DependencyHealth,
    EconomyAccount, EconomyTransaction, EmbedDelivery, EmbedTemplate, FormSubmission, Giveaway,
    GiveawayEntry, GlobalUser, GuildConfig, GuildDashboardStats, GuildMemberSummary,
    InfrastructureHealth, InviteAttribution, InviteLeaderboardEntry, MemberXpAward,
    MemberXpProfile, MessageDeletionJob, ModerationCase, NewAuditEvent, NewEmbedDelivery,
    NewEmbedTemplate, NewFormSubmission, NewGiveaway, NewModerationCase, NewReminder,
    NewSuggestion, NewTicket, NewTicketPanel, Poll, Reminder, RuntimeMetrics, Suggestion,
    TemporaryVoiceChannel, Ticket, TicketAnalytics, TicketEvent, TicketNote, TicketPanel,
    UserGuildContext, VerificationAttempt, VoiceLinkEvent, config::ConfigValidationError,
};

const CONFIG_CACHE_TTL_SECONDS: u64 = 300;
const GIVEAWAY_SELECT_LIST: &str = "SELECT g.id, g.guild_id, g.channel_id, g.message_id, g.creator_id, d.username AS creator_username, d.display_name AS creator_display_name, g.title, g.description, g.status, g.winner_count, g.minimum_account_age_days, g.required_role_ids, g.bonus_role_id, g.bonus_entries, g.winner_ids, (SELECT count(*) FROM giveaway_entries e WHERE e.giveaway_id = g.id)::bigint AS entrant_count, COALESCE((SELECT sum(weight) FROM giveaway_entries e WHERE e.giveaway_id = g.id), 0)::bigint AS total_entries, g.ends_at, g.ended_at, g.announcement_sent_at, g.reroll_count, g.created_at, g.updated_at FROM giveaways g LEFT JOIN discord_identities d ON d.user_id = g.creator_id WHERE g.guild_id = $1 ORDER BY g.id DESC LIMIT $2";
const GIVEAWAY_SELECT_ONE: &str = "SELECT g.id, g.guild_id, g.channel_id, g.message_id, g.creator_id, d.username AS creator_username, d.display_name AS creator_display_name, g.title, g.description, g.status, g.winner_count, g.minimum_account_age_days, g.required_role_ids, g.bonus_role_id, g.bonus_entries, g.winner_ids, (SELECT count(*) FROM giveaway_entries e WHERE e.giveaway_id = g.id)::bigint AS entrant_count, COALESCE((SELECT sum(weight) FROM giveaway_entries e WHERE e.giveaway_id = g.id), 0)::bigint AS total_entries, g.ends_at, g.ended_at, g.announcement_sent_at, g.reroll_count, g.created_at, g.updated_at FROM giveaways g LEFT JOIN discord_identities d ON d.user_id = g.creator_id WHERE g.guild_id = $1 AND g.id = $2";
const TICKET_SELECT_LIST: &str = "SELECT t.id,t.guild_id,t.panel_id,t.opener_id,d.username AS opener_username,d.display_name AS opener_display_name,t.channel_id,t.subject,t.opening_note,t.status,t.claimed_by,t.assigned_to,t.closed_by,t.closed_reason,t.transcript,t.priority,t.tags,t.first_response_at,t.last_activity_at,t.opened_at,t.closed_at,t.reopened_at,t.updated_at FROM tickets t LEFT JOIN discord_identities d ON d.user_id=t.opener_id WHERE t.guild_id=$1 ORDER BY t.updated_at DESC,t.id DESC LIMIT $2";
const TICKET_SELECT_ONE: &str = "SELECT t.id,t.guild_id,t.panel_id,t.opener_id,d.username AS opener_username,d.display_name AS opener_display_name,t.channel_id,t.subject,t.opening_note,t.status,t.claimed_by,t.assigned_to,t.closed_by,t.closed_reason,t.transcript,t.priority,t.tags,t.first_response_at,t.last_activity_at,t.opened_at,t.closed_at,t.reopened_at,t.updated_at FROM tickets t LEFT JOIN discord_identities d ON d.user_id=t.opener_id WHERE t.guild_id=$1 AND t.id=$2";
const TICKET_SELECT_CHANNEL: &str = "SELECT t.id,t.guild_id,t.panel_id,t.opener_id,d.username AS opener_username,d.display_name AS opener_display_name,t.channel_id,t.subject,t.opening_note,t.status,t.claimed_by,t.assigned_to,t.closed_by,t.closed_reason,t.transcript,t.priority,t.tags,t.first_response_at,t.last_activity_at,t.opened_at,t.closed_at,t.reopened_at,t.updated_at FROM tickets t LEFT JOIN discord_identities d ON d.user_id=t.opener_id WHERE t.guild_id=$1 AND t.channel_id=$2 ORDER BY t.id DESC LIMIT 1";
const POLL_SELECT: &str = "SELECT p.id,p.guild_id,p.channel_id,p.creator_id,d.username AS creator_username,d.display_name AS creator_display_name,p.question,p.options,(SELECT COALESCE(jsonb_agg(COALESCE(v.votes,0) ORDER BY series.option_index),'[]'::jsonb) FROM generate_series(0,jsonb_array_length(p.options)-1) AS series(option_index) LEFT JOIN (SELECT option_index,count(*)::bigint AS votes FROM poll_votes WHERE poll_id=p.id GROUP BY option_index) v ON v.option_index=series.option_index) AS counts,(SELECT count(*) FROM poll_votes WHERE poll_id=p.id)::bigint AS total_votes,p.anonymous,p.status,p.ends_at,p.closed_at,p.created_at FROM polls p LEFT JOIN discord_identities d ON d.user_id=p.creator_id";

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
    #[error("insufficient economy balance")]
    InsufficientFunds,
}

impl Infrastructure {
    pub async fn claim_scheduler_tick(&self, tick: u64) -> Result<bool, StoreError> {
        let mut cache = self.redis.clone();
        let acquired: Option<String> = redis::cmd("SET")
            .arg(format!("scheduler-tick:{tick}"))
            .arg("1")
            .arg("EX")
            .arg(30_u64)
            .arg("NX")
            .query_async(&mut cache)
            .await?;
        Ok(acquired.is_some())
    }
    pub async fn set_afk_status(
        &self,
        guild_id: &str,
        user_id: &str,
        reason: &str,
        set_at_unix: u64,
        ttl_seconds: u64,
    ) -> Result<(), StoreError> {
        let set_at =
            chrono::DateTime::from_timestamp(i64::try_from(set_at_unix).unwrap_or(i64::MAX), 0)
                .unwrap_or_else(chrono::Utc::now);
        let expires_at = set_at
            + chrono::Duration::seconds(i64::try_from(ttl_seconds.max(1)).unwrap_or(i64::MAX));
        sqlx::query("INSERT INTO afk_statuses(guild_id,user_id,reason,set_at,expires_at) VALUES($1,$2,$3,$4,$5) ON CONFLICT(guild_id,user_id) DO UPDATE SET reason=EXCLUDED.reason,set_at=EXCLUDED.set_at,expires_at=EXCLUDED.expires_at")
            .bind(guild_id).bind(user_id).bind(reason).bind(set_at).bind(expires_at).execute(&self.pool).await?;
        Ok(())
    }
    pub async fn afk_status(
        &self,
        guild_id: &str,
        user_id: &str,
    ) -> Result<Option<(String, u64)>, StoreError> {
        sqlx::query(
            "DELETE FROM afk_statuses WHERE guild_id=$1 AND user_id=$2 AND expires_at <= now()",
        )
        .bind(guild_id)
        .bind(user_id)
        .execute(&self.pool)
        .await?;
        let value: Option<(String, chrono::DateTime<chrono::Utc>)> = sqlx::query_as(
            "SELECT reason,set_at FROM afk_statuses WHERE guild_id=$1 AND user_id=$2 AND expires_at > now()",
        )
        .bind(guild_id)
        .bind(user_id)
        .fetch_optional(&self.pool)
        .await?;
        Ok(value.map(|(reason, set_at)| {
            (
                reason,
                u64::try_from(set_at.timestamp()).unwrap_or_default(),
            )
        }))
    }
    pub async fn clear_afk_status(
        &self,
        guild_id: &str,
        user_id: &str,
    ) -> Result<bool, StoreError> {
        let result = sqlx::query("DELETE FROM afk_statuses WHERE guild_id=$1 AND user_id=$2")
            .bind(guild_id)
            .bind(user_id)
            .execute(&self.pool)
            .await?;
        Ok(result.rows_affected() > 0)
    }
    #[allow(clippy::too_many_arguments)]
    pub async fn create_poll(
        &self,
        guild_id: &str,
        channel_id: &str,
        creator_id: &str,
        question: &str,
        options: &[String],
        anonymous: bool,
        ttl_seconds: u64,
    ) -> Result<i64, StoreError> {
        let ends_at = chrono::Utc::now()
            + chrono::Duration::seconds(i64::try_from(ttl_seconds.max(60)).unwrap_or(i64::MAX));
        let id: i64 = sqlx::query_scalar("INSERT INTO polls(guild_id,channel_id,creator_id,question,options,anonymous,ends_at) VALUES($1,$2,$3,$4,$5,$6,$7) RETURNING id")
            .bind(guild_id).bind(channel_id).bind(creator_id).bind(question).bind(serde_json::to_value(options)?).bind(anonymous).bind(ends_at).fetch_one(&self.pool).await?;
        Ok(id)
    }
    pub async fn vote_poll(
        &self,
        guild_id: &str,
        poll_id: i64,
        user_id: &str,
        option: usize,
    ) -> Result<Option<(String, Vec<String>, Vec<u64>, bool)>, StoreError> {
        let mut tx = self.pool.begin().await?;
        sqlx::query("UPDATE polls SET status='expired',closed_at=COALESCE(closed_at,now()) WHERE guild_id=$1 AND id=$2 AND status='active' AND ends_at <= now()")
            .bind(guild_id).bind(poll_id).execute(&mut *tx).await?;
        let value: Option<(String, serde_json::Value, bool)> = sqlx::query_as(
            "SELECT question,options,anonymous FROM polls WHERE guild_id=$1 AND id=$2 AND status='active' AND ends_at > now() FOR UPDATE",
        ).bind(guild_id).bind(poll_id).fetch_optional(&mut *tx).await?;
        let Some((question, options_value, anonymous)) = value else {
            tx.commit().await?;
            return Ok(None);
        };
        let options: Vec<String> = serde_json::from_value(options_value)?;
        if option >= options.len() {
            tx.rollback().await?;
            return Ok(None);
        }
        sqlx::query("INSERT INTO poll_votes(poll_id,user_id,option_index) VALUES($1,$2,$3) ON CONFLICT(poll_id,user_id) DO UPDATE SET option_index=EXCLUDED.option_index,updated_at=now()")
            .bind(poll_id).bind(user_id).bind(i32::try_from(option).unwrap_or(i32::MAX)).execute(&mut *tx).await?;
        let raw: Vec<i32> =
            sqlx::query_scalar("SELECT option_index FROM poll_votes WHERE poll_id=$1")
                .bind(poll_id)
                .fetch_all(&mut *tx)
                .await?;
        tx.commit().await?;
        let mut counts = vec![0; options.len()];
        for item in raw {
            if let Ok(index) = usize::try_from(item)
                && let Some(count) = counts.get_mut(index)
            {
                *count += 1;
            }
        }
        Ok(Some((question, options, counts, anonymous)))
    }
    #[allow(clippy::type_complexity)]
    pub async fn close_poll(
        &self,
        guild_id: &str,
        poll_id: i64,
    ) -> Result<Option<(i64, String, String, String, Vec<String>, Vec<u64>, bool)>, StoreError>
    {
        let mut transaction = self.pool.begin().await?;
        let row: Option<(i64, String, String, String, serde_json::Value, bool)> = sqlx::query_as(
            "UPDATE polls SET status='closed',closed_at=now() WHERE guild_id=$1 AND id=$2 AND status='active' AND message_id IS NOT NULL RETURNING id,channel_id,message_id,question,options,anonymous",
        )
        .bind(guild_id)
        .bind(poll_id)
        .fetch_optional(&mut *transaction)
        .await?;
        let Some((id, channel_id, message_id, question, options, anonymous)) = row else {
            transaction.commit().await?;
            return Ok(None);
        };
        let options: Vec<String> = serde_json::from_value(options)?;
        let raw: Vec<i32> =
            sqlx::query_scalar("SELECT option_index FROM poll_votes WHERE poll_id=$1")
                .bind(id)
                .fetch_all(&mut *transaction)
                .await?;
        let mut counts = vec![0; options.len()];
        for option in raw {
            if let Ok(index) = usize::try_from(option)
                && let Some(count) = counts.get_mut(index)
            {
                *count += 1;
            }
        }
        transaction.commit().await?;
        Ok(Some((
            id, channel_id, message_id, question, options, counts, anonymous,
        )))
    }
    pub async fn attach_poll_message(
        &self,
        guild_id: &str,
        poll_id: i64,
        message_id: &str,
    ) -> Result<(), StoreError> {
        sqlx::query(
            "UPDATE polls SET message_id=$3 WHERE guild_id=$1 AND id=$2 AND status='active'",
        )
        .bind(guild_id)
        .bind(poll_id)
        .bind(message_id)
        .execute(&self.pool)
        .await?;
        Ok(())
    }
    #[allow(clippy::type_complexity)]
    pub async fn claim_due_polls(
        &self,
        guild_id: &str,
        now_unix: u64,
        limit: u8,
    ) -> Result<Vec<(i64, String, String, String, Vec<String>, Vec<u64>, bool)>, StoreError> {
        let now = chrono::DateTime::from_timestamp(i64::try_from(now_unix).unwrap_or(i64::MAX), 0)
            .unwrap_or_else(chrono::Utc::now);
        let mut transaction = self.pool.begin().await?;
        let rows: Vec<(i64, String, String, String, serde_json::Value, bool)> = sqlx::query_as(
            "UPDATE polls SET status='expired',closed_at=$2 WHERE id IN (SELECT id FROM polls WHERE guild_id=$1 AND status='active' AND message_id IS NOT NULL AND ends_at <= $2 ORDER BY ends_at FOR UPDATE SKIP LOCKED LIMIT $3) RETURNING id,channel_id,message_id,question,options,anonymous",
        )
        .bind(guild_id)
        .bind(now)
        .bind(i64::from(limit.clamp(1, 100)))
        .fetch_all(&mut *transaction)
        .await?;
        let mut due = Vec::with_capacity(rows.len());
        for (id, channel_id, message_id, question, options, anonymous) in rows {
            let options: Vec<String> = serde_json::from_value(options)?;
            let raw: Vec<i32> =
                sqlx::query_scalar("SELECT option_index FROM poll_votes WHERE poll_id=$1")
                    .bind(id)
                    .fetch_all(&mut *transaction)
                    .await?;
            let mut counts = vec![0; options.len()];
            for option in raw {
                if let Ok(index) = usize::try_from(option)
                    && let Some(count) = counts.get_mut(index)
                {
                    *count += 1;
                }
            }
            due.push((
                id, channel_id, message_id, question, options, counts, anonymous,
            ));
        }
        transaction.commit().await?;
        Ok(due)
    }
    pub async fn attribute_invite(
        &self,
        guild_id: &str,
        joined_user_id: &str,
        invites: &[(String, Option<u64>, u64)],
    ) -> Result<Option<(String, Option<u64>, u64)>, StoreError> {
        let mut cache = self.redis.clone();
        let key = format!("invite-snapshot:{guild_id}");
        let previous: Option<String> = redis::cmd("GET").arg(&key).query_async(&mut cache).await?;
        redis::cmd("SET")
            .arg(&key)
            .arg(serde_json::to_string(invites)?)
            .query_async::<()>(&mut cache)
            .await?;
        let Some(previous) = previous else {
            return Ok(None);
        };
        let previous: Vec<(String, Option<u64>, u64)> = serde_json::from_str(&previous)?;
        let chosen = invites
            .iter()
            .filter_map(|(code, inviter, uses)| {
                let before = previous
                    .iter()
                    .find(|(old, _, _)| old == code)
                    .map(|(_, _, uses)| *uses)
                    .unwrap_or_default();
                uses.checked_sub(before)
                    .filter(|delta| *delta > 0)
                    .map(|delta| (delta, code, inviter))
            })
            .max_by_key(|(delta, _, _)| *delta);
        let Some((_, code, inviter)) = chosen else {
            return Ok(None);
        };
        let inviter_id = inviter.map(|id| id.to_string());
        sqlx::query("INSERT INTO invite_attributions(guild_id,joined_user_id,invite_code,inviter_id) VALUES($1,$2,$3,$4)")
            .bind(guild_id).bind(joined_user_id).bind(code).bind(&inviter_id).execute(&self.pool).await?;
        let total: i64 = if let Some(inviter_id) = &inviter_id {
            sqlx::query_scalar(
                "SELECT count(*) FROM invite_attributions WHERE guild_id=$1 AND inviter_id=$2",
            )
            .bind(guild_id)
            .bind(inviter_id)
            .fetch_one(&self.pool)
            .await?
        } else {
            0
        };
        Ok(Some((
            code.clone(),
            *inviter,
            u64::try_from(total).unwrap_or_default(),
        )))
    }
    pub async fn ensure_invite_snapshot(
        &self,
        guild_id: &str,
        invites: &[(String, Option<u64>, u64)],
    ) -> Result<bool, StoreError> {
        let mut cache = self.redis.clone();
        let created: Option<String> = redis::cmd("SET")
            .arg(format!("invite-snapshot:{guild_id}"))
            .arg(serde_json::to_string(invites)?)
            .arg("NX")
            .query_async(&mut cache)
            .await?;
        Ok(created.is_some())
    }
    pub async fn invite_snapshot_ready(&self, guild_id: &str) -> Result<bool, StoreError> {
        let mut cache = self.redis.clone();
        Ok(redis::cmd("EXISTS")
            .arg(format!("invite-snapshot:{guild_id}"))
            .query_async(&mut cache)
            .await?)
    }
    pub async fn polls(&self, guild_id: &str, limit: i64) -> Result<Vec<Poll>, StoreError> {
        sqlx::query("UPDATE polls SET status='expired',closed_at=COALESCE(closed_at,now()) WHERE guild_id=$1 AND status='active' AND ends_at <= now()")
            .bind(guild_id).execute(&self.pool).await?;
        sqlx::query_as::<_, Poll>(AssertSqlSafe(format!(
            "{POLL_SELECT} WHERE p.guild_id=$1 ORDER BY p.id DESC LIMIT $2"
        )))
        .bind(guild_id)
        .bind(limit.clamp(1, 500))
        .fetch_all(&self.pool)
        .await
        .map_err(StoreError::from)
    }
    pub async fn poll(&self, guild_id: &str, poll_id: i64) -> Result<Option<Poll>, StoreError> {
        sqlx::query("UPDATE polls SET status='expired',closed_at=COALESCE(closed_at,now()) WHERE guild_id=$1 AND id=$2 AND status='active' AND ends_at <= now()")
            .bind(guild_id).bind(poll_id).execute(&self.pool).await?;
        sqlx::query_as::<_, Poll>(AssertSqlSafe(format!(
            "{POLL_SELECT} WHERE p.guild_id=$1 AND p.id=$2"
        )))
        .bind(guild_id)
        .bind(poll_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(StoreError::from)
    }
    pub async fn invite_attributions(
        &self,
        guild_id: &str,
        limit: i64,
    ) -> Result<Vec<InviteAttribution>, StoreError> {
        sqlx::query_as::<_,InviteAttribution>("SELECT a.id,a.guild_id,a.joined_user_id,j.username AS joined_username,j.display_name AS joined_display_name,a.invite_code,a.inviter_id,i.username AS inviter_username,i.display_name AS inviter_display_name,a.attributed_at FROM invite_attributions a LEFT JOIN discord_identities j ON j.user_id=a.joined_user_id LEFT JOIN discord_identities i ON i.user_id=a.inviter_id WHERE a.guild_id=$1 ORDER BY a.id DESC LIMIT $2")
            .bind(guild_id).bind(limit.clamp(1,500)).fetch_all(&self.pool).await.map_err(StoreError::from)
    }
    pub async fn invite_leaderboard(
        &self,
        guild_id: &str,
        limit: i64,
    ) -> Result<Vec<InviteLeaderboardEntry>, StoreError> {
        sqlx::query_as::<_,InviteLeaderboardEntry>("SELECT a.inviter_id,d.username,d.display_name,count(*)::bigint AS attributed_joins,max(a.attributed_at) AS latest_attribution FROM invite_attributions a LEFT JOIN discord_identities d ON d.user_id=a.inviter_id WHERE a.guild_id=$1 AND a.inviter_id IS NOT NULL GROUP BY a.inviter_id,d.username,d.display_name ORDER BY attributed_joins DESC,latest_attribution DESC LIMIT $2")
            .bind(guild_id).bind(limit.clamp(1,500)).fetch_all(&self.pool).await.map_err(StoreError::from)
    }
    pub async fn afk_statuses(
        &self,
        guild_id: &str,
        limit: i64,
    ) -> Result<Vec<AfkStatus>, StoreError> {
        sqlx::query("DELETE FROM afk_statuses WHERE expires_at <= now()")
            .execute(&self.pool)
            .await?;
        sqlx::query_as::<_,AfkStatus>("SELECT a.guild_id,a.user_id,d.username,d.display_name,a.reason,a.set_at,a.expires_at FROM afk_statuses a LEFT JOIN discord_identities d ON d.user_id=a.user_id WHERE a.guild_id=$1 ORDER BY a.set_at DESC LIMIT $2")
            .bind(guild_id).bind(limit.clamp(1,500)).fetch_all(&self.pool).await.map_err(StoreError::from)
    }
    pub async fn record_verification_attempt(
        &self,
        guild_id: &str,
        user_id: &str,
        outcome: &str,
        reason: Option<&str>,
        account_age_days: u64,
        has_avatar: bool,
    ) -> Result<(), StoreError> {
        sqlx::query("INSERT INTO verification_attempts(guild_id,user_id,outcome,reason,account_age_days,has_avatar) VALUES($1,$2,$3,$4,$5,$6)")
            .bind(guild_id).bind(user_id).bind(outcome).bind(reason).bind(i32::try_from(account_age_days).unwrap_or(i32::MAX)).bind(has_avatar).execute(&self.pool).await?;
        Ok(())
    }
    pub async fn verification_attempts(
        &self,
        guild_id: &str,
        limit: i64,
    ) -> Result<Vec<VerificationAttempt>, StoreError> {
        sqlx::query_as::<_,VerificationAttempt>("SELECT v.id,v.guild_id,v.user_id,d.username,d.display_name,v.outcome,v.reason,v.account_age_days,v.has_avatar,v.occurred_at FROM verification_attempts v LEFT JOIN discord_identities d ON d.user_id=v.user_id WHERE v.guild_id=$1 ORDER BY v.id DESC LIMIT $2")
            .bind(guild_id).bind(limit.clamp(1,500)).fetch_all(&self.pool).await.map_err(StoreError::from)
    }
    #[allow(clippy::too_many_arguments)]
    pub async fn record_counter_run(
        &self,
        guild_id: &str,
        counter_id: &str,
        channel_id: &str,
        value: u64,
        rendered_name: &str,
        outcome: &str,
        error: Option<&str>,
    ) -> Result<(), StoreError> {
        sqlx::query("INSERT INTO counter_runs(guild_id,counter_id,channel_id,value,rendered_name,outcome,error) VALUES($1,$2,$3,$4,$5,$6,$7)")
            .bind(guild_id).bind(counter_id).bind(channel_id).bind(i64::try_from(value).unwrap_or(i64::MAX)).bind(rendered_name).bind(outcome).bind(error).execute(&self.pool).await?;
        Ok(())
    }
    pub async fn counter_runs(
        &self,
        guild_id: &str,
        limit: i64,
    ) -> Result<Vec<CounterRun>, StoreError> {
        sqlx::query_as::<_, CounterRun>(
            "SELECT * FROM counter_runs WHERE guild_id=$1 ORDER BY id DESC LIMIT $2",
        )
        .bind(guild_id)
        .bind(limit.clamp(1, 500))
        .fetch_all(&self.pool)
        .await
        .map_err(StoreError::from)
    }
    pub async fn record_voice_link_event(
        &self,
        guild_id: &str,
        user_id: &str,
        voice_channel_id: &str,
        text_channel_id: &str,
        action: &str,
    ) -> Result<(), StoreError> {
        sqlx::query("INSERT INTO voice_link_events(guild_id,user_id,voice_channel_id,text_channel_id,action) VALUES($1,$2,$3,$4,$5)").bind(guild_id).bind(user_id).bind(voice_channel_id).bind(text_channel_id).bind(action).execute(&self.pool).await?;
        Ok(())
    }
    pub async fn voice_link_events(
        &self,
        guild_id: &str,
        limit: i64,
    ) -> Result<Vec<VoiceLinkEvent>, StoreError> {
        sqlx::query_as::<_,VoiceLinkEvent>("SELECT v.id,v.guild_id,v.user_id,d.username,d.display_name,v.voice_channel_id,v.text_channel_id,v.action,v.occurred_at FROM voice_link_events v LEFT JOIN discord_identities d ON d.user_id=v.user_id WHERE v.guild_id=$1 ORDER BY v.id DESC LIMIT $2").bind(guild_id).bind(limit.clamp(1,500)).fetch_all(&self.pool).await.map_err(StoreError::from)
    }
    pub async fn claim_appeal_cooldown(
        &self,
        guild_id: &str,
        user_id: &str,
        seconds: u64,
    ) -> Result<bool, StoreError> {
        let mut cache = self.redis.clone();
        let result: Option<String> = redis::cmd("SET")
            .arg(format!("appeal-cooldown:{guild_id}:{user_id}"))
            .arg("1")
            .arg("EX")
            .arg(seconds.max(1))
            .arg("NX")
            .query_async(&mut cache)
            .await?;
        Ok(result.is_some())
    }
    pub async fn economy_account(
        &self,
        guild_id: &str,
        user_id: &str,
    ) -> Result<EconomyAccount, StoreError> {
        let account =
            sqlx::query_as("SELECT * FROM economy_accounts WHERE guild_id=$1 AND user_id=$2")
                .bind(guild_id)
                .bind(user_id)
                .fetch_optional(&self.pool)
                .await?;
        Ok(account.unwrap_or_else(|| EconomyAccount {
            guild_id: guild_id.into(),
            user_id: user_id.into(),
            balance: 0,
            lifetime_earned: 0,
            lifetime_spent: 0,
            updated_at: chrono::Utc::now(),
        }))
    }
    pub async fn economy_accounts(
        &self,
        guild_id: &str,
        limit: i64,
    ) -> Result<Vec<EconomyAccount>, StoreError> {
        sqlx::query_as("SELECT * FROM economy_accounts WHERE guild_id=$1 ORDER BY balance DESC,updated_at ASC LIMIT $2").bind(guild_id).bind(limit.clamp(1,500)).fetch_all(&self.pool).await.map_err(StoreError::from)
    }
    pub async fn economy_transactions(
        &self,
        guild_id: &str,
        limit: i64,
    ) -> Result<Vec<EconomyTransaction>, StoreError> {
        sqlx::query_as("SELECT * FROM economy_transactions WHERE guild_id=$1 ORDER BY created_at DESC,id DESC LIMIT $2").bind(guild_id).bind(limit.clamp(1,500)).fetch_all(&self.pool).await.map_err(StoreError::from)
    }
    #[allow(clippy::too_many_arguments)]
    pub async fn economy_change_balance(
        &self,
        guild_id: &str,
        user_id: &str,
        amount: i64,
        kind: &str,
        reference: &str,
        counterparty_id: Option<&str>,
        metadata: &serde_json::Value,
    ) -> Result<EconomyAccount, StoreError> {
        let mut tx = self.pool.begin().await?;
        sqlx::query(
            "INSERT INTO economy_accounts(guild_id,user_id) VALUES($1,$2) ON CONFLICT DO NOTHING",
        )
        .bind(guild_id)
        .bind(user_id)
        .execute(&mut *tx)
        .await?;
        let current: i64 = sqlx::query_scalar(
            "SELECT balance FROM economy_accounts WHERE guild_id=$1 AND user_id=$2 FOR UPDATE",
        )
        .bind(guild_id)
        .bind(user_id)
        .fetch_one(&mut *tx)
        .await?;
        if sqlx::query_scalar::<_, i64>("SELECT id FROM economy_transactions WHERE reference=$1")
            .bind(reference)
            .fetch_optional(&mut *tx)
            .await?
            .is_none()
        {
            let next = current
                .checked_add(amount)
                .ok_or(StoreError::InsufficientFunds)?;
            if next < 0 {
                return Err(StoreError::InsufficientFunds);
            }
            sqlx::query("UPDATE economy_accounts SET balance=$3,lifetime_earned=lifetime_earned+GREATEST($4,0),lifetime_spent=lifetime_spent+GREATEST(-$4,0),updated_at=now() WHERE guild_id=$1 AND user_id=$2").bind(guild_id).bind(user_id).bind(next).bind(amount).execute(&mut *tx).await?;
            sqlx::query("INSERT INTO economy_transactions(guild_id,user_id,counterparty_id,amount,kind,reference,metadata) VALUES($1,$2,$3,$4,$5,$6,$7)").bind(guild_id).bind(user_id).bind(counterparty_id).bind(amount).bind(kind).bind(reference).bind(metadata).execute(&mut *tx).await?;
        }
        tx.commit().await?;
        self.economy_account(guild_id, user_id).await
    }
    pub async fn economy_claim(
        &self,
        guild_id: &str,
        user_id: &str,
        kind: &str,
        amount: u64,
        cooldown_seconds: u64,
        reference: &str,
    ) -> Result<(Option<EconomyAccount>, u64), StoreError> {
        let amount = i64::try_from(amount).map_err(|_| StoreError::InsufficientFunds)?;
        let mut tx = self.pool.begin().await?;
        if sqlx::query_scalar::<_, i64>("SELECT id FROM economy_transactions WHERE reference=$1")
            .bind(reference)
            .fetch_optional(&mut *tx)
            .await?
            .is_some()
        {
            let account =
                sqlx::query_as("SELECT * FROM economy_accounts WHERE guild_id=$1 AND user_id=$2")
                    .bind(guild_id)
                    .bind(user_id)
                    .fetch_one(&mut *tx)
                    .await?;
            tx.commit().await?;
            return Ok((Some(account), cooldown_seconds));
        }
        let now = chrono::Utc::now();
        let next:Option<chrono::DateTime<chrono::Utc>>=sqlx::query_scalar("SELECT next_claim_at FROM economy_claims WHERE guild_id=$1 AND user_id=$2 AND kind=$3 FOR UPDATE").bind(guild_id).bind(user_id).bind(kind).fetch_optional(&mut *tx).await?;
        if let Some(next) = next
            && next > now
        {
            return Ok((
                None,
                u64::try_from((next - now).num_seconds()).unwrap_or_default(),
            ));
        }
        let next =
            now + chrono::Duration::seconds(i64::try_from(cooldown_seconds).unwrap_or(i64::MAX));
        sqlx::query("INSERT INTO economy_claims(guild_id,user_id,kind,next_claim_at) VALUES($1,$2,$3,$4) ON CONFLICT(guild_id,user_id,kind) DO UPDATE SET next_claim_at=EXCLUDED.next_claim_at").bind(guild_id).bind(user_id).bind(kind).bind(next).execute(&mut *tx).await?;
        sqlx::query(
            "INSERT INTO economy_accounts(guild_id,user_id) VALUES($1,$2) ON CONFLICT DO NOTHING",
        )
        .bind(guild_id)
        .bind(user_id)
        .execute(&mut *tx)
        .await?;
        sqlx::query("UPDATE economy_accounts SET balance=balance+$3,lifetime_earned=lifetime_earned+$3,updated_at=now() WHERE guild_id=$1 AND user_id=$2")
            .bind(guild_id).bind(user_id).bind(amount).execute(&mut *tx).await?;
        sqlx::query("INSERT INTO economy_transactions(guild_id,user_id,amount,kind,reference) VALUES($1,$2,$3,$4,$5)")
            .bind(guild_id).bind(user_id).bind(amount).bind(kind).bind(reference).execute(&mut *tx).await?;
        let account =
            sqlx::query_as("SELECT * FROM economy_accounts WHERE guild_id=$1 AND user_id=$2")
                .bind(guild_id)
                .bind(user_id)
                .fetch_one(&mut *tx)
                .await?;
        tx.commit().await?;
        Ok((Some(account), cooldown_seconds))
    }
    pub async fn economy_transfer(
        &self,
        guild_id: &str,
        from: &str,
        to: &str,
        amount: u64,
        tax_percent: u8,
        reference: &str,
    ) -> Result<(EconomyAccount, EconomyAccount, u64), StoreError> {
        let amount = i64::try_from(amount).map_err(|_| StoreError::InsufficientFunds)?;
        let tax = amount.saturating_mul(i64::from(tax_percent)) / 100;
        let received = amount.saturating_sub(tax);
        let mut tx = self.pool.begin().await?;
        for user in [from, to] {
            sqlx::query("INSERT INTO economy_accounts(guild_id,user_id) VALUES($1,$2) ON CONFLICT DO NOTHING").bind(guild_id).bind(user).execute(&mut *tx).await?;
        }
        let mut users = vec![from, to];
        users.sort_unstable();
        for user in users {
            let _: i64 = sqlx::query_scalar(
                "SELECT balance FROM economy_accounts WHERE guild_id=$1 AND user_id=$2 FOR UPDATE",
            )
            .bind(guild_id)
            .bind(user)
            .fetch_one(&mut *tx)
            .await?;
        }
        let debit_ref = format!("{reference}:debit");
        if sqlx::query_scalar::<_, i64>("SELECT id FROM economy_transactions WHERE reference=$1")
            .bind(&debit_ref)
            .fetch_optional(&mut *tx)
            .await?
            .is_none()
        {
            let balance: i64 = sqlx::query_scalar(
                "SELECT balance FROM economy_accounts WHERE guild_id=$1 AND user_id=$2",
            )
            .bind(guild_id)
            .bind(from)
            .fetch_one(&mut *tx)
            .await?;
            if balance < amount {
                return Err(StoreError::InsufficientFunds);
            }
            sqlx::query("UPDATE economy_accounts SET balance=balance-$3,lifetime_spent=lifetime_spent+$3,updated_at=now() WHERE guild_id=$1 AND user_id=$2").bind(guild_id).bind(from).bind(amount).execute(&mut *tx).await?;
            sqlx::query("UPDATE economy_accounts SET balance=balance+$3,lifetime_earned=lifetime_earned+$3,updated_at=now() WHERE guild_id=$1 AND user_id=$2").bind(guild_id).bind(to).bind(received).execute(&mut *tx).await?;
            sqlx::query("INSERT INTO economy_transactions(guild_id,user_id,counterparty_id,amount,kind,reference,metadata) VALUES($1,$2,$3,$4,'transfer_sent',$5,$6),($1,$3,$2,$7,'transfer_received',$8,$6)").bind(guild_id).bind(from).bind(to).bind(-amount).bind(&debit_ref).bind(serde_json::json!({"tax":tax})).bind(received).bind(format!("{reference}:credit")).execute(&mut *tx).await?;
        }
        tx.commit().await?;
        Ok((
            self.economy_account(guild_id, from).await?,
            self.economy_account(guild_id, to).await?,
            u64::try_from(tax).unwrap_or_default(),
        ))
    }
    pub async fn temporary_voice_channels(
        &self,
        guild_id: &str,
    ) -> Result<Vec<TemporaryVoiceChannel>, StoreError> {
        sqlx::query_as("SELECT * FROM temporary_voice_channels WHERE guild_id=$1 AND status='active' ORDER BY created_at DESC")
            .bind(guild_id).fetch_all(&self.pool).await.map_err(StoreError::from)
    }

    pub async fn temporary_voice_channel(
        &self,
        guild_id: &str,
        channel_id: &str,
    ) -> Result<Option<TemporaryVoiceChannel>, StoreError> {
        sqlx::query_as("SELECT * FROM temporary_voice_channels WHERE guild_id=$1 AND channel_id=$2 AND status='active'")
            .bind(guild_id).bind(channel_id).fetch_optional(&self.pool).await.map_err(StoreError::from)
    }

    pub async fn temporary_voice_owner_channel(
        &self,
        guild_id: &str,
        owner_id: &str,
    ) -> Result<Option<TemporaryVoiceChannel>, StoreError> {
        sqlx::query_as("SELECT * FROM temporary_voice_channels WHERE guild_id=$1 AND owner_id=$2 AND status='active'")
            .bind(guild_id).bind(owner_id).fetch_optional(&self.pool).await.map_err(StoreError::from)
    }

    pub async fn register_temporary_voice_channel(
        &self,
        guild_id: &str,
        channel_id: &str,
        owner_id: &str,
        name: &str,
        user_limit: u8,
    ) -> Result<TemporaryVoiceChannel, StoreError> {
        sqlx::query_as("INSERT INTO temporary_voice_channels (guild_id,channel_id,owner_id,name,user_limit) VALUES ($1,$2,$3,$4,$5) RETURNING *")
            .bind(guild_id).bind(channel_id).bind(owner_id).bind(name).bind(i16::from(user_limit)).fetch_one(&self.pool).await.map_err(StoreError::from)
    }

    pub async fn update_temporary_voice_channel(
        &self,
        guild_id: &str,
        channel_id: &str,
        owner_id: Option<&str>,
        name: Option<&str>,
        user_limit: Option<u8>,
        locked: Option<bool>,
    ) -> Result<Option<TemporaryVoiceChannel>, StoreError> {
        sqlx::query_as("UPDATE temporary_voice_channels SET owner_id=COALESCE($3,owner_id),name=COALESCE($4,name),user_limit=COALESCE($5,user_limit),locked=COALESCE($6,locked),updated_at=now() WHERE guild_id=$1 AND channel_id=$2 AND status='active' RETURNING *")
            .bind(guild_id).bind(channel_id).bind(owner_id).bind(name).bind(user_limit.map(i16::from)).bind(locked).fetch_optional(&self.pool).await.map_err(StoreError::from)
    }

    pub async fn unregister_temporary_voice_channel(
        &self,
        guild_id: &str,
        channel_id: &str,
    ) -> Result<bool, StoreError> {
        let result=sqlx::query("UPDATE temporary_voice_channels SET status='deleted',deleted_at=now(),updated_at=now() WHERE guild_id=$1 AND channel_id=$2 AND status='active'").bind(guild_id).bind(channel_id).execute(&self.pool).await?;
        let mut cache = self.redis.clone();
        let _: u64 = redis::cmd("DEL")
            .arg(format!("temporary-voice:members:{channel_id}"))
            .query_async(&mut cache)
            .await?;
        Ok(result.rows_affected() == 1)
    }

    pub async fn temporary_voice_member_join(
        &self,
        channel_id: &str,
        user_id: &str,
    ) -> Result<u64, StoreError> {
        let key = format!("temporary-voice:members:{channel_id}");
        let mut cache = self.redis.clone();
        let _: u64 = redis::cmd("SADD")
            .arg(&key)
            .arg(user_id)
            .query_async(&mut cache)
            .await?;
        let _: () = redis::cmd("EXPIRE")
            .arg(&key)
            .arg(86_400_u64)
            .query_async(&mut cache)
            .await?;
        redis::cmd("SCARD")
            .arg(&key)
            .query_async(&mut cache)
            .await
            .map_err(StoreError::from)
    }

    pub async fn temporary_voice_member_leave(
        &self,
        channel_id: &str,
        user_id: &str,
    ) -> Result<u64, StoreError> {
        let key = format!("temporary-voice:members:{channel_id}");
        let mut cache = self.redis.clone();
        let _: u64 = redis::cmd("SREM")
            .arg(&key)
            .arg(user_id)
            .query_async(&mut cache)
            .await?;
        redis::cmd("SCARD")
            .arg(&key)
            .query_async(&mut cache)
            .await
            .map_err(StoreError::from)
    }

    pub async fn schedule_temporary_voice_delete(
        &self,
        guild_id: &str,
        channel_id: &str,
        delay_seconds: u16,
    ) -> Result<(), StoreError> {
        sqlx::query("UPDATE temporary_voice_channels SET delete_after=now()+($3::bigint * interval '1 second'),updated_at=now() WHERE guild_id=$1 AND channel_id=$2 AND status='active'").bind(guild_id).bind(channel_id).bind(i64::from(delay_seconds)).execute(&self.pool).await?;
        Ok(())
    }
    pub async fn cancel_temporary_voice_delete(
        &self,
        guild_id: &str,
        channel_id: &str,
    ) -> Result<(), StoreError> {
        sqlx::query("UPDATE temporary_voice_channels SET delete_after=NULL,updated_at=now() WHERE guild_id=$1 AND channel_id=$2 AND status='active'").bind(guild_id).bind(channel_id).execute(&self.pool).await?;
        Ok(())
    }
    pub async fn due_temporary_voice_deletes(
        &self,
        guild_id: &str,
        now_unix: u64,
    ) -> Result<Vec<TemporaryVoiceChannel>, StoreError> {
        let now = chrono::DateTime::from_timestamp(i64::try_from(now_unix).unwrap_or(i64::MAX), 0)
            .unwrap_or_else(chrono::Utc::now);
        sqlx::query_as("SELECT * FROM temporary_voice_channels WHERE guild_id=$1 AND status='active' AND delete_after IS NOT NULL AND delete_after <= $2").bind(guild_id).bind(now).fetch_all(&self.pool).await.map_err(StoreError::from)
    }

    pub async fn create_embed_template(
        &self,
        input: &NewEmbedTemplate,
    ) -> Result<EmbedTemplate, StoreError> {
        let payload = serde_json::to_value(&input.payload)?;
        let id: i64 = sqlx::query_scalar("INSERT INTO embed_templates (guild_id,name,payload,created_by) VALUES ($1,$2,$3,$4) RETURNING id")
            .bind(&input.guild_id).bind(input.name.trim()).bind(payload).bind(&input.created_by).fetch_one(&self.pool).await?;
        self.embed_template(&input.guild_id, id)
            .await?
            .ok_or(sqlx::Error::RowNotFound.into())
    }

    pub async fn embed_templates(
        &self,
        guild_id: &str,
        limit: i64,
    ) -> Result<Vec<EmbedTemplate>, StoreError> {
        sqlx::query_as::<_, EmbedTemplate>("SELECT id,guild_id,name,payload,created_by,created_at,updated_at FROM embed_templates WHERE guild_id=$1 AND deleted_at IS NULL ORDER BY updated_at DESC,id DESC LIMIT $2")
            .bind(guild_id).bind(limit.clamp(1,500)).fetch_all(&self.pool).await.map_err(StoreError::from)
    }

    pub async fn embed_template(
        &self,
        guild_id: &str,
        id: i64,
    ) -> Result<Option<EmbedTemplate>, StoreError> {
        sqlx::query_as::<_, EmbedTemplate>("SELECT id,guild_id,name,payload,created_by,created_at,updated_at FROM embed_templates WHERE guild_id=$1 AND id=$2 AND deleted_at IS NULL")
            .bind(guild_id).bind(id).fetch_optional(&self.pool).await.map_err(StoreError::from)
    }

    pub async fn update_embed_template(
        &self,
        guild_id: &str,
        id: i64,
        name: &str,
        payload: &crate::EmbedPayload,
    ) -> Result<Option<EmbedTemplate>, StoreError> {
        let payload = serde_json::to_value(payload)?;
        let updated: Option<i64> = sqlx::query_scalar("UPDATE embed_templates SET name=$3,payload=$4,updated_at=now() WHERE guild_id=$1 AND id=$2 AND deleted_at IS NULL RETURNING id")
            .bind(guild_id).bind(id).bind(name.trim()).bind(payload).fetch_optional(&self.pool).await?;
        match updated {
            Some(_) => self.embed_template(guild_id, id).await,
            None => Ok(None),
        }
    }

    pub async fn delete_embed_template(&self, guild_id: &str, id: i64) -> Result<bool, StoreError> {
        Ok(sqlx::query("UPDATE embed_templates SET deleted_at=now(),updated_at=now() WHERE guild_id=$1 AND id=$2 AND deleted_at IS NULL").bind(guild_id).bind(id).execute(&self.pool).await?.rows_affected()==1)
    }

    pub async fn record_embed_delivery(
        &self,
        input: &NewEmbedDelivery,
    ) -> Result<EmbedDelivery, StoreError> {
        sqlx::query_as::<_, EmbedDelivery>("INSERT INTO embed_deliveries (guild_id,template_id,channel_id,message_id,actor_id,action,status,error) VALUES ($1,$2,$3,$4,$5,$6,$7,$8) RETURNING *")
            .bind(&input.guild_id).bind(input.template_id).bind(&input.channel_id).bind(&input.message_id).bind(&input.actor_id).bind(&input.action).bind(&input.status).bind(&input.error).fetch_one(&self.pool).await.map_err(StoreError::from)
    }

    pub async fn embed_deliveries(
        &self,
        guild_id: &str,
        template_id: Option<i64>,
        limit: i64,
    ) -> Result<Vec<EmbedDelivery>, StoreError> {
        sqlx::query_as::<_, EmbedDelivery>("SELECT * FROM embed_deliveries WHERE guild_id=$1 AND ($2::bigint IS NULL OR template_id=$2) ORDER BY created_at DESC,id DESC LIMIT $3")
            .bind(guild_id).bind(template_id).bind(limit.clamp(1,500)).fetch_all(&self.pool).await.map_err(StoreError::from)
    }

    pub async fn embed_delivery(
        &self,
        guild_id: &str,
        template_id: i64,
        delivery_id: i64,
    ) -> Result<Option<EmbedDelivery>, StoreError> {
        sqlx::query_as::<_, EmbedDelivery>(
            "SELECT * FROM embed_deliveries WHERE guild_id=$1 AND template_id=$2 AND id=$3",
        )
        .bind(guild_id)
        .bind(template_id)
        .bind(delivery_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(StoreError::from)
    }

    pub async fn schedule_message_deletion(
        &self,
        guild_id: &str,
        channel_id: &str,
        message_id: &str,
        rule_id: &str,
        author_id: &str,
        due_unix: u64,
    ) -> Result<bool, StoreError> {
        let due = chrono::DateTime::from_timestamp(i64::try_from(due_unix).unwrap_or(i64::MAX), 0)
            .unwrap_or_else(chrono::Utc::now);
        let result = sqlx::query("INSERT INTO message_deletion_jobs (guild_id,channel_id,message_id,rule_id,author_id,source,due_at) VALUES ($1,$2,$3,$4,$5,'rule',$6) ON CONFLICT DO NOTHING")
            .bind(guild_id).bind(channel_id).bind(message_id).bind(rule_id).bind(author_id).bind(due).execute(&self.pool).await?;
        Ok(result.rows_affected() == 1)
    }

    pub async fn claim_due_message_deletions(
        &self,
        guild_id: &str,
        now_unix: u64,
        limit: i64,
    ) -> Result<Vec<MessageDeletionJob>, StoreError> {
        let now = chrono::DateTime::from_timestamp(i64::try_from(now_unix).unwrap_or(i64::MAX), 0)
            .unwrap_or_else(chrono::Utc::now);
        sqlx::query("UPDATE message_deletion_jobs SET status='failed',updated_at=now() WHERE guild_id=$1 AND status='delivering' AND attempts>=5 AND updated_at < $2 - interval '5 minutes'").bind(guild_id).bind(now).execute(&self.pool).await?;
        sqlx::query_as::<_, MessageDeletionJob>("WITH due AS (SELECT guild_id,channel_id,message_id FROM message_deletion_jobs WHERE guild_id=$1 AND ((status='scheduled' AND due_at <= $2) OR (status='delivering' AND attempts<5 AND updated_at < $2 - interval '5 minutes')) ORDER BY due_at LIMIT $3 FOR UPDATE SKIP LOCKED) UPDATE message_deletion_jobs j SET status='delivering',attempts=attempts+1,updated_at=now() FROM due WHERE j.guild_id=due.guild_id AND j.channel_id=due.channel_id AND j.message_id=due.message_id RETURNING j.*")
            .bind(guild_id).bind(now).bind(limit.clamp(1,100)).fetch_all(&self.pool).await.map_err(StoreError::from)
    }

    pub async fn finish_message_deletion(
        &self,
        guild_id: &str,
        channel_id: &str,
        message_id: &str,
        deleted: bool,
        retry_unix: u64,
    ) -> Result<bool, StoreError> {
        let retry =
            chrono::DateTime::from_timestamp(i64::try_from(retry_unix).unwrap_or(i64::MAX), 0)
                .unwrap_or_else(chrono::Utc::now);
        let terminal: Option<bool> = sqlx::query_scalar("UPDATE message_deletion_jobs SET status=CASE WHEN $4 THEN 'deleted' WHEN attempts>=5 THEN 'failed' ELSE 'scheduled' END,deleted_at=CASE WHEN $4 THEN now() ELSE NULL END,due_at=CASE WHEN NOT $4 AND attempts<5 THEN $5 ELSE due_at END,updated_at=now() WHERE guild_id=$1 AND channel_id=$2 AND message_id=$3 AND status='delivering' RETURNING status='failed'")
            .bind(guild_id).bind(channel_id).bind(message_id).bind(deleted).bind(retry).fetch_optional(&self.pool).await?;
        Ok(terminal.unwrap_or(false))
    }

    pub async fn message_deletion_jobs(
        &self,
        guild_id: &str,
        limit: i64,
    ) -> Result<Vec<MessageDeletionJob>, StoreError> {
        sqlx::query_as::<_, MessageDeletionJob>("SELECT * FROM message_deletion_jobs WHERE guild_id=$1 ORDER BY created_at DESC LIMIT $2").bind(guild_id).bind(limit.clamp(1,500)).fetch_all(&self.pool).await.map_err(StoreError::from)
    }
    pub async fn create_reminder(
        &self,
        input: &NewReminder,
        max_active_per_user: u8,
    ) -> Result<Option<Reminder>, StoreError> {
        let id: Option<i64> = sqlx::query_scalar(
            "INSERT INTO reminders (guild_id, channel_id, user_id, message, delivery_mode, remind_at, next_attempt_at) \
             SELECT $1, $2, $3, $4, $5, $6, $6 WHERE (SELECT count(*) FROM reminders WHERE guild_id = $1 AND user_id = $3 AND status IN ('scheduled', 'delivering')) < $7 \
             RETURNING id",
        )
        .bind(&input.guild_id)
        .bind(&input.channel_id)
        .bind(&input.user_id)
        .bind(input.message.trim())
        .bind(&input.delivery_mode)
        .bind(input.remind_at)
        .bind(i64::from(max_active_per_user))
        .fetch_optional(&self.pool)
        .await?;
        match id {
            Some(id) => self.reminder(&input.guild_id, id).await,
            None => Ok(None),
        }
    }

    pub async fn reminders(&self, guild_id: &str, limit: i64) -> Result<Vec<Reminder>, StoreError> {
        sqlx::query_as::<_, Reminder>("SELECT r.id, r.guild_id, r.channel_id, r.user_id, d.username, d.display_name, r.message, r.delivery_mode, r.status, r.remind_at, r.next_attempt_at, r.delivered_at, r.attempts, r.created_at, r.updated_at FROM reminders r LEFT JOIN discord_identities d ON d.user_id = r.user_id WHERE r.guild_id = $1 ORDER BY r.id DESC LIMIT $2")
            .bind(guild_id)
            .bind(limit.clamp(1, 500))
            .fetch_all(&self.pool)
            .await
            .map_err(StoreError::from)
    }

    pub async fn message_deletion_job(
        &self,
        guild_id: &str,
        channel_id: &str,
        message_id: &str,
    ) -> Result<Option<MessageDeletionJob>, StoreError> {
        sqlx::query_as::<_, MessageDeletionJob>("SELECT * FROM message_deletion_jobs WHERE guild_id=$1 AND channel_id=$2 AND message_id=$3")
            .bind(guild_id).bind(channel_id).bind(message_id).fetch_optional(&self.pool).await.map_err(StoreError::from)
    }

    pub async fn reminder(
        &self,
        guild_id: &str,
        reminder_id: i64,
    ) -> Result<Option<Reminder>, StoreError> {
        sqlx::query_as::<_, Reminder>("SELECT r.id, r.guild_id, r.channel_id, r.user_id, d.username, d.display_name, r.message, r.delivery_mode, r.status, r.remind_at, r.next_attempt_at, r.delivered_at, r.attempts, r.created_at, r.updated_at FROM reminders r LEFT JOIN discord_identities d ON d.user_id = r.user_id WHERE r.guild_id = $1 AND r.id = $2")
            .bind(guild_id)
            .bind(reminder_id)
            .fetch_optional(&self.pool)
            .await
            .map_err(StoreError::from)
    }

    pub async fn user_reminders(
        &self,
        guild_id: &str,
        user_id: &str,
        limit: i64,
    ) -> Result<Vec<Reminder>, StoreError> {
        sqlx::query_as::<_, Reminder>("SELECT r.id, r.guild_id, r.channel_id, r.user_id, d.username, d.display_name, r.message, r.delivery_mode, r.status, r.remind_at, r.next_attempt_at, r.delivered_at, r.attempts, r.created_at, r.updated_at FROM reminders r LEFT JOIN discord_identities d ON d.user_id = r.user_id WHERE r.guild_id = $1 AND r.user_id = $2 AND r.status IN ('scheduled', 'delivering') ORDER BY r.remind_at LIMIT $3")
            .bind(guild_id)
            .bind(user_id)
            .bind(limit.clamp(1, 50))
            .fetch_all(&self.pool)
            .await
            .map_err(StoreError::from)
    }

    pub async fn cancel_reminder(
        &self,
        guild_id: &str,
        reminder_id: i64,
        user_id: &str,
    ) -> Result<bool, StoreError> {
        let result = sqlx::query("UPDATE reminders SET status = 'cancelled', updated_at = now() WHERE guild_id = $1 AND id = $2 AND user_id = $3 AND status = 'scheduled'")
            .bind(guild_id).bind(reminder_id).bind(user_id).execute(&self.pool).await?;
        Ok(result.rows_affected() == 1)
    }

    pub async fn cancel_reminder_as_admin(
        &self,
        guild_id: &str,
        reminder_id: i64,
    ) -> Result<bool, StoreError> {
        let result = sqlx::query("UPDATE reminders SET status = 'cancelled', updated_at = now() WHERE guild_id = $1 AND id = $2 AND status = 'scheduled'")
            .bind(guild_id).bind(reminder_id).execute(&self.pool).await?;
        Ok(result.rows_affected() == 1)
    }

    pub async fn claim_due_reminders(
        &self,
        guild_id: &str,
        now_unix: u64,
        limit: i64,
    ) -> Result<Vec<Reminder>, StoreError> {
        let now = chrono::DateTime::from_timestamp(i64::try_from(now_unix).unwrap_or(i64::MAX), 0)
            .unwrap_or_else(chrono::Utc::now);
        sqlx::query(
            "UPDATE reminders SET status = 'failed', updated_at = now() WHERE guild_id = $1 AND status = 'delivering' AND attempts >= 5 AND updated_at <= $2 - interval '5 minutes'",
        )
        .bind(guild_id)
        .bind(now)
        .execute(&self.pool)
        .await?;
        sqlx::query_as::<_, Reminder>(
            "WITH due AS (SELECT id FROM reminders WHERE guild_id = $1 AND ((status = 'scheduled' AND next_attempt_at <= $2) OR (status = 'delivering' AND attempts < 5 AND updated_at <= $2 - interval '5 minutes')) ORDER BY next_attempt_at, id FOR UPDATE SKIP LOCKED LIMIT $3), \
             claimed AS (UPDATE reminders r SET status = 'delivering', attempts = attempts + 1, updated_at = now() FROM due WHERE r.id = due.id RETURNING r.*) \
             SELECT c.id, c.guild_id, c.channel_id, c.user_id, d.username, d.display_name, c.message, c.delivery_mode, c.status, c.remind_at, c.next_attempt_at, c.delivered_at, c.attempts, c.created_at, c.updated_at FROM claimed c LEFT JOIN discord_identities d ON d.user_id = c.user_id ORDER BY c.next_attempt_at, c.id",
        )
        .bind(guild_id).bind(now).bind(limit.clamp(1, 100)).fetch_all(&self.pool).await.map_err(StoreError::from)
    }

    pub async fn complete_reminder(
        &self,
        guild_id: &str,
        reminder_id: i64,
    ) -> Result<(), StoreError> {
        sqlx::query("UPDATE reminders SET status = 'delivered', delivered_at = now(), updated_at = now() WHERE guild_id = $1 AND id = $2 AND status = 'delivering'")
            .bind(guild_id).bind(reminder_id).execute(&self.pool).await?;
        Ok(())
    }

    pub async fn retry_reminder(
        &self,
        guild_id: &str,
        reminder_id: i64,
        next_attempt_unix: u64,
    ) -> Result<bool, StoreError> {
        let next_attempt = chrono::DateTime::from_timestamp(
            i64::try_from(next_attempt_unix).unwrap_or(i64::MAX),
            0,
        )
        .unwrap_or_else(chrono::Utc::now);
        let terminal: Option<bool> = sqlx::query_scalar(
            "UPDATE reminders SET status = CASE WHEN attempts >= 5 THEN 'failed' ELSE 'scheduled' END, next_attempt_at = CASE WHEN attempts >= 5 THEN next_attempt_at ELSE $3 END, updated_at = now() WHERE guild_id = $1 AND id = $2 AND status = 'delivering' RETURNING status = 'failed'",
        ).bind(guild_id).bind(reminder_id).bind(next_attempt).fetch_optional(&self.pool).await?;
        Ok(terminal.unwrap_or(false))
    }

    pub async fn create_giveaway(&self, input: &NewGiveaway) -> Result<Giveaway, StoreError> {
        let id: i64 = sqlx::query_scalar(
            "INSERT INTO giveaways (guild_id, channel_id, creator_id, title, description, winner_count, minimum_account_age_days, required_role_ids, bonus_role_id, bonus_entries, ends_at) \
             VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11) RETURNING id",
        )
        .bind(&input.guild_id).bind(&input.channel_id).bind(&input.creator_id)
        .bind(input.title.trim()).bind(input.description.trim()).bind(input.winner_count)
        .bind(input.minimum_account_age_days).bind(&input.required_role_ids)
        .bind(&input.bonus_role_id).bind(input.bonus_entries).bind(input.ends_at)
        .fetch_one(&self.pool).await?;
        self.giveaway(&input.guild_id, id)
            .await?
            .ok_or(sqlx::Error::RowNotFound.into())
    }

    pub async fn activate_giveaway(
        &self,
        guild_id: &str,
        giveaway_id: i64,
        message_id: &str,
    ) -> Result<bool, StoreError> {
        let result = sqlx::query("UPDATE giveaways SET status = 'active', message_id = $3, updated_at = now() WHERE guild_id = $1 AND id = $2 AND status = 'pending_delivery'")
            .bind(guild_id).bind(giveaway_id).bind(message_id).execute(&self.pool).await?;
        Ok(result.rows_affected() == 1)
    }

    pub async fn fail_giveaway_delivery(
        &self,
        guild_id: &str,
        giveaway_id: i64,
    ) -> Result<(), StoreError> {
        sqlx::query("UPDATE giveaways SET status = 'delivery_failed', updated_at = now() WHERE guild_id = $1 AND id = $2 AND status = 'pending_delivery'")
            .bind(guild_id).bind(giveaway_id).execute(&self.pool).await?;
        Ok(())
    }

    pub async fn enter_giveaway(
        &self,
        guild_id: &str,
        giveaway_id: i64,
        user_id: &str,
        weight: i16,
    ) -> Result<Option<bool>, StoreError> {
        let result = sqlx::query(
            "INSERT INTO giveaway_entries (giveaway_id, user_id, weight) \
             SELECT id, $3, $4 FROM giveaways WHERE guild_id = $1 AND id = $2 AND status = 'active' AND ends_at > now() \
             ON CONFLICT DO NOTHING",
        )
        .bind(guild_id).bind(giveaway_id).bind(user_id).bind(weight).execute(&self.pool).await?;
        if result.rows_affected() == 1 {
            return Ok(Some(true));
        }
        let available: bool = sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM giveaways WHERE guild_id = $1 AND id = $2 AND status = 'active' AND ends_at > now())")
            .bind(guild_id).bind(giveaway_id).fetch_one(&self.pool).await?;
        Ok(available.then_some(false))
    }

    pub async fn giveaways(&self, guild_id: &str, limit: i64) -> Result<Vec<Giveaway>, StoreError> {
        sqlx::query_as::<_, Giveaway>(GIVEAWAY_SELECT_LIST)
            .bind(guild_id)
            .bind(limit.clamp(1, 500))
            .fetch_all(&self.pool)
            .await
            .map_err(StoreError::from)
    }

    pub async fn giveaway(
        &self,
        guild_id: &str,
        giveaway_id: i64,
    ) -> Result<Option<Giveaway>, StoreError> {
        sqlx::query_as::<_, Giveaway>(GIVEAWAY_SELECT_ONE)
            .bind(guild_id)
            .bind(giveaway_id)
            .fetch_optional(&self.pool)
            .await
            .map_err(StoreError::from)
    }

    pub async fn giveaway_entries(
        &self,
        guild_id: &str,
        giveaway_id: i64,
        limit: i64,
    ) -> Result<Vec<GiveawayEntry>, StoreError> {
        sqlx::query_as::<_, GiveawayEntry>(
            "SELECT e.user_id, d.username, d.display_name, e.weight, e.created_at FROM giveaway_entries e \
             JOIN giveaways g ON g.id = e.giveaway_id LEFT JOIN discord_identities d ON d.user_id = e.user_id \
             WHERE g.guild_id = $1 AND g.id = $2 ORDER BY e.created_at DESC LIMIT $3")
            .bind(guild_id).bind(giveaway_id).bind(limit.clamp(1, 5_000)).fetch_all(&self.pool).await.map_err(StoreError::from)
    }

    pub async fn due_giveaway_ids(
        &self,
        guild_id: &str,
        now_unix: u64,
    ) -> Result<Vec<i64>, StoreError> {
        let now = chrono::DateTime::from_timestamp(i64::try_from(now_unix).unwrap_or(i64::MAX), 0)
            .unwrap_or_else(chrono::Utc::now);
        sqlx::query_scalar("SELECT id FROM giveaways WHERE guild_id = $1 AND status = 'active' AND ends_at <= $2 ORDER BY ends_at LIMIT 100")
            .bind(guild_id).bind(now).fetch_all(&self.pool).await.map_err(StoreError::from)
    }

    pub async fn pending_giveaway_announcement_ids(
        &self,
        guild_id: &str,
    ) -> Result<Vec<i64>, StoreError> {
        sqlx::query_scalar("SELECT id FROM giveaways WHERE guild_id = $1 AND status = 'ended' AND announcement_sent_at IS NULL ORDER BY ended_at LIMIT 100")
            .bind(guild_id).fetch_all(&self.pool).await.map_err(StoreError::from)
    }

    pub async fn mark_giveaway_announced(
        &self,
        guild_id: &str,
        giveaway_id: i64,
    ) -> Result<(), StoreError> {
        sqlx::query("UPDATE giveaways SET announcement_sent_at = now(), updated_at = now() WHERE guild_id = $1 AND id = $2 AND status = 'ended'")
            .bind(guild_id).bind(giveaway_id).execute(&self.pool).await?;
        Ok(())
    }

    pub async fn end_giveaway(
        &self,
        guild_id: &str,
        giveaway_id: i64,
        force: bool,
    ) -> Result<Option<Giveaway>, StoreError> {
        let mut tx = self.pool.begin().await?;
        let winner_count: Option<i16> = sqlx::query_scalar(
            "SELECT winner_count FROM giveaways WHERE guild_id = $1 AND id = $2 AND status = 'active' AND ($3 OR ends_at <= now()) FOR UPDATE")
            .bind(guild_id).bind(giveaway_id).bind(force).fetch_optional(&mut *tx).await?;
        let Some(winner_count) = winner_count else {
            tx.rollback().await?;
            return Ok(None);
        };
        let winners: Vec<String> = sqlx::query_scalar(
            "SELECT user_id FROM giveaway_entries WHERE giveaway_id = $1 ORDER BY (-ln(greatest(random(), 0.000000001)) / weight) LIMIT $2")
            .bind(giveaway_id).bind(i64::from(winner_count)).fetch_all(&mut *tx).await?;
        sqlx::query("UPDATE giveaways SET status = 'ended', winner_ids = $3, ended_at = now(), updated_at = now() WHERE guild_id = $1 AND id = $2")
            .bind(guild_id).bind(giveaway_id).bind(serde_json::json!(winners)).execute(&mut *tx).await?;
        tx.commit().await?;
        self.giveaway(guild_id, giveaway_id).await
    }

    pub async fn reroll_giveaway(
        &self,
        guild_id: &str,
        giveaway_id: i64,
    ) -> Result<Option<Giveaway>, StoreError> {
        let mut tx = self.pool.begin().await?;
        let row: Option<(i16, serde_json::Value)> = sqlx::query_as(
            "SELECT winner_count, winner_ids FROM giveaways WHERE guild_id = $1 AND id = $2 AND status = 'ended' FOR UPDATE")
            .bind(guild_id).bind(giveaway_id).fetch_optional(&mut *tx).await?;
        let Some((winner_count, previous)) = row else {
            tx.rollback().await?;
            return Ok(None);
        };
        let excluded: Vec<String> = serde_json::from_value(previous).unwrap_or_default();
        let winners: Vec<String> = sqlx::query_scalar(
            "SELECT user_id FROM giveaway_entries WHERE giveaway_id = $1 AND NOT (user_id = ANY($2::text[])) \
             ORDER BY (-ln(greatest(random(), 0.000000001)) / weight) LIMIT $3")
            .bind(giveaway_id).bind(&excluded).bind(i64::from(winner_count)).fetch_all(&mut *tx).await?;
        if winners.is_empty() {
            tx.rollback().await?;
            return Ok(None);
        }
        sqlx::query("UPDATE giveaways SET winner_ids = $3, reroll_count = reroll_count + 1, announcement_sent_at = NULL, updated_at = now() WHERE guild_id = $1 AND id = $2")
            .bind(guild_id).bind(giveaway_id).bind(serde_json::json!(winners)).execute(&mut *tx).await?;
        tx.commit().await?;
        self.giveaway(guild_id, giveaway_id).await
    }

    pub async fn create_suggestion(&self, input: &NewSuggestion) -> Result<Suggestion, StoreError> {
        sqlx::query_as::<_, Suggestion>(
            "WITH inserted AS ( \
               INSERT INTO suggestions (guild_id, channel_id, author_id, content) \
               VALUES ($1, $2, $3, $4) RETURNING * \
             ) SELECT i.id, i.guild_id, i.channel_id, i.message_id, i.author_id, \
                      d.username, d.display_name, i.content, i.status, i.staff_note, \
                      0::bigint AS upvotes, 0::bigint AS downvotes, i.created_at, i.updated_at \
               FROM inserted i LEFT JOIN discord_identities d ON d.user_id = i.author_id",
        )
        .bind(&input.guild_id)
        .bind(&input.channel_id)
        .bind(&input.author_id)
        .bind(input.content.trim())
        .fetch_one(&self.pool)
        .await
        .map_err(StoreError::from)
    }

    pub async fn claim_suggestion_cooldown(
        &self,
        guild_id: &str,
        user_id: &str,
        cooldown_seconds: u32,
    ) -> Result<bool, StoreError> {
        if cooldown_seconds == 0 {
            return Ok(true);
        }
        let mut cache = self.redis.clone();
        let acquired: Option<String> = redis::cmd("SET")
            .arg(format!("suggestion-cooldown:{guild_id}:{user_id}"))
            .arg("1")
            .arg("EX")
            .arg(cooldown_seconds)
            .arg("NX")
            .query_async(&mut cache)
            .await?;
        Ok(acquired.is_some())
    }

    pub async fn claim_module_cooldown(
        &self,
        module: &str,
        rule: &str,
        guild_id: &str,
        user_id: &str,
        cooldown_seconds: u32,
    ) -> Result<bool, StoreError> {
        if cooldown_seconds == 0 {
            return Ok(true);
        }
        let safe = |value: &str| {
            value
                .chars()
                .filter(|character| character.is_ascii_alphanumeric() || *character == '-')
                .take(64)
                .collect::<String>()
        };
        let mut cache = self.redis.clone();
        let acquired: Option<String> = redis::cmd("SET")
            .arg(format!(
                "module-cooldown:{}:{}:{guild_id}:{user_id}",
                safe(module),
                safe(rule)
            ))
            .arg("1")
            .arg("EX")
            .arg(cooldown_seconds)
            .arg("NX")
            .query_async(&mut cache)
            .await?;
        Ok(acquired.is_some())
    }

    pub async fn set_suggestion_message_id(
        &self,
        guild_id: &str,
        suggestion_id: i64,
        message_id: &str,
    ) -> Result<(), StoreError> {
        sqlx::query(
            "UPDATE suggestions SET message_id = $3, updated_at = now() WHERE guild_id = $1 AND id = $2",
        )
        .bind(guild_id)
        .bind(suggestion_id)
        .bind(message_id)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn vote_suggestion(
        &self,
        guild_id: &str,
        suggestion_id: i64,
        user_id: &str,
        direction: i16,
    ) -> Result<Option<Suggestion>, StoreError> {
        let mut transaction = self.pool.begin().await?;
        let pending: bool = sqlx::query_scalar(
            "SELECT EXISTS(SELECT 1 FROM suggestions WHERE guild_id = $1 AND id = $2 AND status = 'pending')",
        )
        .bind(guild_id)
        .bind(suggestion_id)
        .fetch_one(&mut *transaction)
        .await?;
        if !pending {
            transaction.rollback().await?;
            return Ok(None);
        }
        sqlx::query(
            "INSERT INTO suggestion_votes (suggestion_id, user_id, direction) VALUES ($1, $2, $3) \
             ON CONFLICT (suggestion_id, user_id) DO UPDATE SET direction = EXCLUDED.direction, updated_at = now()",
        )
        .bind(suggestion_id)
        .bind(user_id)
        .bind(direction)
        .execute(&mut *transaction)
        .await?;
        transaction.commit().await?;
        self.suggestion(guild_id, suggestion_id).await
    }

    pub async fn suggestions(
        &self,
        guild_id: &str,
        limit: i64,
    ) -> Result<Vec<Suggestion>, StoreError> {
        sqlx::query_as::<_, Suggestion>(
            "SELECT s.id, s.guild_id, s.channel_id, s.message_id, s.author_id, d.username, d.display_name, \
                    s.content, s.status, s.staff_note, \
                    count(v.user_id) FILTER (WHERE v.direction = 1) AS upvotes, \
                    count(v.user_id) FILTER (WHERE v.direction = -1) AS downvotes, s.created_at, s.updated_at \
             FROM suggestions s LEFT JOIN discord_identities d ON d.user_id = s.author_id \
             LEFT JOIN suggestion_votes v ON v.suggestion_id = s.id WHERE s.guild_id = $1 \
             GROUP BY s.id, d.username, d.display_name ORDER BY s.id DESC LIMIT $2",
        )
        .bind(guild_id)
        .bind(limit.clamp(1, 500))
        .fetch_all(&self.pool)
        .await
        .map_err(StoreError::from)
    }

    pub async fn suggestion(
        &self,
        guild_id: &str,
        suggestion_id: i64,
    ) -> Result<Option<Suggestion>, StoreError> {
        sqlx::query_as::<_, Suggestion>(
            "SELECT s.id, s.guild_id, s.channel_id, s.message_id, s.author_id, d.username, d.display_name, \
                    s.content, s.status, s.staff_note, \
                    count(v.user_id) FILTER (WHERE v.direction = 1) AS upvotes, \
                    count(v.user_id) FILTER (WHERE v.direction = -1) AS downvotes, s.created_at, s.updated_at \
             FROM suggestions s LEFT JOIN discord_identities d ON d.user_id = s.author_id \
             LEFT JOIN suggestion_votes v ON v.suggestion_id = s.id \
             WHERE s.guild_id = $1 AND s.id = $2 GROUP BY s.id, d.username, d.display_name",
        )
        .bind(guild_id)
        .bind(suggestion_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(StoreError::from)
    }

    pub async fn update_suggestion_status_audited(
        &self,
        guild_id: &str,
        suggestion_id: i64,
        status: &str,
        staff_note: Option<&str>,
        event: &NewAuditEvent,
    ) -> Result<Option<Suggestion>, StoreError> {
        let mut transaction = self.pool.begin().await?;
        let updated = sqlx::query_scalar::<_, i64>(
            "UPDATE suggestions SET status = $3, staff_note = $4, updated_at = now() \
             WHERE guild_id = $1 AND id = $2 RETURNING id",
        )
        .bind(guild_id)
        .bind(suggestion_id)
        .bind(status)
        .bind(staff_note)
        .fetch_optional(&mut *transaction)
        .await?;
        if updated.is_none() {
            transaction.rollback().await?;
            return Ok(None);
        }
        insert_audit_event(&mut transaction, event).await?;
        transaction.commit().await?;
        self.suggestion(guild_id, suggestion_id).await
    }

    pub async fn create_form_submission(
        &self,
        input: &NewFormSubmission,
    ) -> Result<FormSubmission, StoreError> {
        sqlx::query_as::<_, FormSubmission>(
            "WITH inserted AS ( \
               INSERT INTO form_submissions (guild_id, form_id, submitter_id, answers) \
               VALUES ($1, $2, $3, $4) \
               RETURNING id, guild_id, form_id, submitter_id, answers, status, reviewer_id, review_note, reviewed_at, created_at \
             ) SELECT i.id, i.guild_id, i.form_id, i.submitter_id, d.username, d.display_name, \
                      i.answers, i.status, i.reviewer_id, i.review_note, i.reviewed_at, i.created_at FROM inserted i \
               LEFT JOIN discord_identities d ON d.user_id = i.submitter_id",
        )
        .bind(&input.guild_id)
        .bind(&input.form_id)
        .bind(&input.submitter_id)
        .bind(&input.answers)
        .fetch_one(&self.pool)
        .await
        .map_err(StoreError::from)
    }

    pub async fn form_submissions(
        &self,
        guild_id: &str,
        form_id: &str,
        limit: i64,
    ) -> Result<Vec<FormSubmission>, StoreError> {
        sqlx::query_as::<_, FormSubmission>(
            "SELECT s.id, s.guild_id, s.form_id, s.submitter_id, d.username, d.display_name, \
                    s.answers, s.status, s.reviewer_id, s.review_note, s.reviewed_at, s.created_at FROM form_submissions s \
             LEFT JOIN discord_identities d ON d.user_id = s.submitter_id \
             WHERE s.guild_id = $1 AND s.form_id = $2 ORDER BY s.id DESC LIMIT $3",
        )
        .bind(guild_id)
        .bind(form_id)
        .bind(limit.clamp(1, 500))
        .fetch_all(&self.pool)
        .await
        .map_err(StoreError::from)
    }

    pub async fn form_submission(
        &self,
        guild_id: &str,
        form_id: &str,
        submission_id: i64,
    ) -> Result<Option<FormSubmission>, StoreError> {
        sqlx::query_as::<_, FormSubmission>(
            "SELECT s.id, s.guild_id, s.form_id, s.submitter_id, d.username, d.display_name, \
                    s.answers, s.status, s.reviewer_id, s.review_note, s.reviewed_at, s.created_at FROM form_submissions s \
             LEFT JOIN discord_identities d ON d.user_id = s.submitter_id \
             WHERE s.guild_id = $1 AND s.form_id = $2 AND s.id = $3",
        )
        .bind(guild_id)
        .bind(form_id)
        .bind(submission_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(StoreError::from)
    }

    #[allow(clippy::too_many_arguments)]
    pub async fn review_form_submission_audited(
        &self,
        guild_id: &str,
        form_id: &str,
        submission_id: i64,
        status: &str,
        reviewer_id: &str,
        note: Option<&str>,
        event: &NewAuditEvent,
    ) -> Result<Option<FormSubmission>, StoreError> {
        let mut tx = self.pool.begin().await?;
        let updated=sqlx::query_scalar::<_,i64>("UPDATE form_submissions SET status=$4,reviewer_id=$5,review_note=$6,reviewed_at=now() WHERE guild_id=$1 AND form_id=$2 AND id=$3 RETURNING id").bind(guild_id).bind(form_id).bind(submission_id).bind(status).bind(reviewer_id).bind(note).fetch_optional(&mut *tx).await?;
        if updated.is_none() {
            tx.rollback().await?;
            return Ok(None);
        }
        insert_audit_event(&mut tx, event).await?;
        tx.commit().await?;
        self.form_submission(guild_id, form_id, submission_id).await
    }

    pub async fn purge_form_submissions(
        &self,
        guild_id: &str,
        retention_days: u16,
    ) -> Result<u64, StoreError> {
        let result = sqlx::query("DELETE FROM form_submissions WHERE guild_id=$1 AND created_at < now() - make_interval(days => $2)")
            .bind(guild_id).bind(i32::from(retention_days)).execute(&self.pool).await?;
        Ok(result.rows_affected())
    }

    pub async fn scheduled_guild_configs(&self) -> Result<Vec<GuildConfig>, StoreError> {
        let values: Vec<serde_json::Value> = sqlx::query_scalar(
            "SELECT config FROM guild_configs WHERE config #>> '{modules,scheduled_messages,enabled}' = 'true' OR config #>> '{modules,giveaways,enabled}' = 'true' OR config #>> '{modules,reminders,enabled}' = 'true' OR config #>> '{modules,auto_delete,enabled}' = 'true' OR config #>> '{modules,tickets,enabled}' = 'true' OR config #>> '{modules,temporary_voice,enabled}' = 'true' OR config #>> '{modules,counters,enabled}' = 'true' OR config #>> '{modules,forms,enabled}' = 'true' OR config #>> '{modules,invite_tracking,enabled}' = 'true' OR config #>> '{modules,polls,enabled}' = 'true'",
        )
        .fetch_all(&self.pool)
        .await?;
        values
            .into_iter()
            .map(serde_json::from_value)
            .collect::<Result<_, _>>()
            .map_err(StoreError::from)
    }

    pub async fn claim_scheduled_message(
        &self,
        guild_id: &str,
        schedule_id: &str,
        occurrence_unix: u64,
    ) -> Result<bool, StoreError> {
        let occurrence = i64::try_from(occurrence_unix).unwrap_or(i64::MAX);
        let result = sqlx::query(
            "INSERT INTO scheduled_message_runs (guild_id, schedule_id, occurrence_unix) VALUES ($1, $2, $3) ON CONFLICT DO NOTHING",
        )
        .bind(guild_id)
        .bind(schedule_id)
        .bind(occurrence)
        .execute(&self.pool)
        .await?;
        Ok(result.rows_affected() == 1)
    }

    pub async fn release_scheduled_message(
        &self,
        guild_id: &str,
        schedule_id: &str,
        occurrence_unix: u64,
    ) -> Result<(), StoreError> {
        sqlx::query("DELETE FROM scheduled_message_runs WHERE guild_id = $1 AND schedule_id = $2 AND occurrence_unix = $3")
            .bind(guild_id)
            .bind(schedule_id)
            .bind(i64::try_from(occurrence_unix).unwrap_or(i64::MAX))
            .execute(&self.pool)
            .await?;
        Ok(())
    }

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

    pub async fn automod_rate_count(
        &self,
        guild_id: &str,
        user_id: &str,
        rule_id: &str,
        fingerprint: Option<&str>,
        message_id: &str,
        window_seconds: u16,
    ) -> Result<u64, StoreError> {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;
        let key = format!(
            "automod:rate:{guild_id}:{user_id}:{rule_id}:{}",
            fingerprint.unwrap_or("all")
        );
        let cutoff = now.saturating_sub(u64::from(window_seconds) * 1_000);
        let mut cache = self.redis.clone();
        let _: () = redis::cmd("ZADD")
            .arg(&key)
            .arg(now)
            .arg(message_id)
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

    pub async fn purge_guild_audit_events(
        &self,
        guild_id: &str,
        retention_days: u16,
    ) -> Result<u64, StoreError> {
        let result = sqlx::query(
            "DELETE FROM audit_events WHERE guild_id=$1 \
             AND occurred_at < now() - make_interval(days => $2)",
        )
        .bind(guild_id)
        .bind(i32::from(retention_days))
        .execute(&self.pool)
        .await?;
        Ok(result.rows_affected())
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
            "SELECT id, guild_id, member_id, moderator_id, action, reason, evidence_urls, status, created_at, updated_at \
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
            "SELECT id, guild_id, member_id, moderator_id, action, reason, evidence_urls, status, created_at, updated_at \
             FROM moderation_cases WHERE guild_id = $1 AND id = $2",
        )
        .bind(guild_id)
        .bind(case_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(StoreError::from)
    }

    pub async fn moderation_warning_count(
        &self,
        guild_id: &str,
        member_id: &str,
    ) -> Result<u64, StoreError> {
        let count: i64 = sqlx::query_scalar(
            "SELECT count(*)::bigint FROM moderation_cases WHERE guild_id=$1 AND member_id=$2 AND action='warn'",
        )
        .bind(guild_id)
        .bind(member_id)
        .fetch_one(&self.pool)
        .await?;
        Ok(u64::try_from(count).unwrap_or_default())
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
            "INSERT INTO moderation_cases (guild_id, member_id, moderator_id, action, reason, evidence_urls) \
             VALUES ($1, $2, $3, $4, $5, $6) \
             RETURNING id, guild_id, member_id, moderator_id, action, reason, evidence_urls, status, created_at, updated_at",
        )
        .bind(guild_id)
        .bind(&input.member_id)
        .bind(moderator_id)
        .bind(&input.action)
        .bind(input.reason.trim())
        .bind(serde_json::json!(input.evidence_urls))
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

    pub async fn adjust_member_xp(
        &self,
        guild_id: &str,
        user_id: &str,
        amount: i64,
        set: bool,
    ) -> Result<MemberXpProfile, StoreError> {
        if set {
            sqlx::query("INSERT INTO member_xp (guild_id,user_id,xp,message_count) VALUES ($1,$2,$3,0) ON CONFLICT (guild_id,user_id) DO UPDATE SET xp=EXCLUDED.xp,updated_at=now()")
                .bind(guild_id).bind(user_id).bind(amount.max(0)).execute(&self.pool).await?;
        } else {
            sqlx::query("INSERT INTO member_xp (guild_id,user_id,xp,message_count) VALUES ($1,$2,$3,0) ON CONFLICT (guild_id,user_id) DO UPDATE SET xp=GREATEST(0,member_xp.xp + EXCLUDED.xp),updated_at=now()")
                .bind(guild_id).bind(user_id).bind(amount).execute(&self.pool).await?;
        }
        self.member_xp_profile(guild_id, user_id).await
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

    pub async fn ticket_panels(&self, guild_id: &str) -> Result<Vec<TicketPanel>, StoreError> {
        sqlx::query_as::<_, TicketPanel>("SELECT id,guild_id,name,title,description,panel_channel_id,panel_message_id,category_id,support_role_ids,button_label,welcome_message,channel_name,intake_enabled,intake_label,max_open_per_user,auto_close_hours,transcript_retention_days,default_priority,enabled,created_by,created_at,updated_at FROM ticket_panels WHERE guild_id=$1 AND deleted_at IS NULL ORDER BY updated_at DESC,id DESC")
            .bind(guild_id).fetch_all(&self.pool).await.map_err(StoreError::from)
    }

    pub async fn ticket_panel(
        &self,
        guild_id: &str,
        id: i64,
    ) -> Result<Option<TicketPanel>, StoreError> {
        sqlx::query_as::<_, TicketPanel>("SELECT id,guild_id,name,title,description,panel_channel_id,panel_message_id,category_id,support_role_ids,button_label,welcome_message,channel_name,intake_enabled,intake_label,max_open_per_user,auto_close_hours,transcript_retention_days,default_priority,enabled,created_by,created_at,updated_at FROM ticket_panels WHERE guild_id=$1 AND id=$2 AND deleted_at IS NULL")
            .bind(guild_id).bind(id).fetch_optional(&self.pool).await.map_err(StoreError::from)
    }

    pub async fn create_ticket_panel(
        &self,
        input: &NewTicketPanel,
    ) -> Result<TicketPanel, StoreError> {
        let id: i64 = sqlx::query_scalar("INSERT INTO ticket_panels (guild_id,name,title,description,panel_channel_id,category_id,support_role_ids,button_label,welcome_message,channel_name,intake_enabled,intake_label,max_open_per_user,auto_close_hours,transcript_retention_days,default_priority,enabled,created_by) VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13,$14,$15,$16,$17,$18) RETURNING id")
            .bind(&input.guild_id).bind(input.name.trim()).bind(input.title.trim()).bind(input.description.trim()).bind(&input.panel_channel_id).bind(&input.category_id).bind(&input.support_role_ids).bind(input.button_label.trim()).bind(input.welcome_message.trim()).bind(input.channel_name.trim()).bind(input.intake_enabled).bind(input.intake_label.trim()).bind(input.max_open_per_user).bind(input.auto_close_hours).bind(input.transcript_retention_days).bind(&input.default_priority).bind(input.enabled).bind(&input.created_by).fetch_one(&self.pool).await?;
        self.ticket_panel(&input.guild_id, id)
            .await?
            .ok_or(sqlx::Error::RowNotFound.into())
    }

    pub async fn update_ticket_panel(
        &self,
        guild_id: &str,
        id: i64,
        input: &NewTicketPanel,
    ) -> Result<Option<TicketPanel>, StoreError> {
        let updated: Option<i64> = sqlx::query_scalar("UPDATE ticket_panels SET name=$3,title=$4,description=$5,panel_channel_id=$6,category_id=$7,support_role_ids=$8,button_label=$9,welcome_message=$10,channel_name=$11,intake_enabled=$12,intake_label=$13,max_open_per_user=$14,auto_close_hours=$15,transcript_retention_days=$16,default_priority=$17,enabled=$18,updated_at=now() WHERE guild_id=$1 AND id=$2 AND deleted_at IS NULL RETURNING id")
            .bind(guild_id).bind(id).bind(input.name.trim()).bind(input.title.trim()).bind(input.description.trim()).bind(&input.panel_channel_id).bind(&input.category_id).bind(&input.support_role_ids).bind(input.button_label.trim()).bind(input.welcome_message.trim()).bind(input.channel_name.trim()).bind(input.intake_enabled).bind(input.intake_label.trim()).bind(input.max_open_per_user).bind(input.auto_close_hours).bind(input.transcript_retention_days).bind(&input.default_priority).bind(input.enabled).fetch_optional(&self.pool).await?;
        match updated {
            Some(_) => self.ticket_panel(guild_id, id).await,
            None => Ok(None),
        }
    }

    pub async fn delete_ticket_panel(&self, guild_id: &str, id: i64) -> Result<bool, StoreError> {
        Ok(sqlx::query("UPDATE ticket_panels SET deleted_at=now(),enabled=false,updated_at=now() WHERE guild_id=$1 AND id=$2 AND deleted_at IS NULL").bind(guild_id).bind(id).execute(&self.pool).await?.rows_affected()==1)
    }

    pub async fn mark_ticket_panel_deployed(
        &self,
        guild_id: &str,
        id: i64,
        channel_id: &str,
        message_id: &str,
    ) -> Result<Option<TicketPanel>, StoreError> {
        let updated: Option<i64> = sqlx::query_scalar("UPDATE ticket_panels SET panel_channel_id=$3,panel_message_id=$4,updated_at=now() WHERE guild_id=$1 AND id=$2 AND deleted_at IS NULL RETURNING id").bind(guild_id).bind(id).bind(channel_id).bind(message_id).fetch_optional(&self.pool).await?;
        match updated {
            Some(_) => self.ticket_panel(guild_id, id).await,
            None => Ok(None),
        }
    }

    pub async fn tickets(&self, guild_id: &str, limit: i64) -> Result<Vec<Ticket>, StoreError> {
        sqlx::query_as::<_, Ticket>(TICKET_SELECT_LIST)
            .bind(guild_id)
            .bind(limit.clamp(1, 500))
            .fetch_all(&self.pool)
            .await
            .map_err(StoreError::from)
    }

    pub async fn ticket(&self, guild_id: &str, id: i64) -> Result<Option<Ticket>, StoreError> {
        sqlx::query_as::<_, Ticket>(TICKET_SELECT_ONE)
            .bind(guild_id)
            .bind(id)
            .fetch_optional(&self.pool)
            .await
            .map_err(StoreError::from)
    }

    pub async fn ticket_by_channel(
        &self,
        guild_id: &str,
        channel_id: &str,
    ) -> Result<Option<Ticket>, StoreError> {
        sqlx::query_as::<_, Ticket>(TICKET_SELECT_CHANNEL)
            .bind(guild_id)
            .bind(channel_id)
            .fetch_optional(&self.pool)
            .await
            .map_err(StoreError::from)
    }

    pub async fn open_ticket_count(
        &self,
        guild_id: &str,
        panel_id: Option<i64>,
        opener_id: &str,
    ) -> Result<i64, StoreError> {
        sqlx::query_scalar("SELECT count(*) FROM tickets WHERE guild_id=$1 AND opener_id=$2 AND status='open' AND panel_id IS NOT DISTINCT FROM $3")
            .bind(guild_id).bind(opener_id).bind(panel_id).fetch_one(&self.pool).await.map_err(StoreError::from)
    }

    pub async fn create_ticket(&self, input: &NewTicket) -> Result<Ticket, StoreError> {
        let mut transaction = self.pool.begin().await?;
        let id:i64=sqlx::query_scalar("INSERT INTO tickets (guild_id,panel_id,opener_id,channel_id,subject,opening_note,priority) VALUES ($1,$2,$3,$4,$5,$6,COALESCE((SELECT default_priority FROM ticket_panels WHERE id=$2),'normal')) RETURNING id")
            .bind(&input.guild_id).bind(input.panel_id).bind(&input.opener_id).bind(&input.channel_id).bind(input.subject.trim()).bind(input.opening_note.trim()).fetch_one(&mut *transaction).await?;
        sqlx::query("INSERT INTO ticket_events (ticket_id,guild_id,actor_id,action,metadata) VALUES ($1,$2,$3,'opened',$4)").bind(id).bind(&input.guild_id).bind(&input.opener_id).bind(serde_json::json!({"channel_id":input.channel_id,"panel_id":input.panel_id})).execute(&mut *transaction).await?;
        transaction.commit().await?;
        self.ticket(&input.guild_id, id)
            .await?
            .ok_or(sqlx::Error::RowNotFound.into())
    }

    pub async fn claim_ticket(
        &self,
        guild_id: &str,
        id: i64,
        actor_id: &str,
    ) -> Result<Option<Ticket>, StoreError> {
        let mut tx = self.pool.begin().await?;
        let changed:Option<i64>=sqlx::query_scalar("UPDATE tickets SET claimed_by=$3,assigned_to=$3,first_response_at=COALESCE(first_response_at,now()),updated_at=now() WHERE guild_id=$1 AND id=$2 AND status='open' RETURNING id").bind(guild_id).bind(id).bind(actor_id).fetch_optional(&mut *tx).await?;
        if changed.is_some() {
            sqlx::query("INSERT INTO ticket_events(ticket_id,guild_id,actor_id,action,metadata) VALUES($1,$2,$3,'claimed','{}')").bind(id).bind(guild_id).bind(actor_id).execute(&mut *tx).await?;
        }
        tx.commit().await?;
        match changed {
            Some(_) => self.ticket(guild_id, id).await,
            None => Ok(None),
        }
    }

    pub async fn assign_ticket(
        &self,
        guild_id: &str,
        id: i64,
        actor_id: &str,
        assignee_id: &str,
    ) -> Result<Option<Ticket>, StoreError> {
        let mut tx = self.pool.begin().await?;
        let changed:Option<i64>=sqlx::query_scalar("UPDATE tickets SET assigned_to=$3,updated_at=now() WHERE guild_id=$1 AND id=$2 AND status='open' RETURNING id").bind(guild_id).bind(id).bind(assignee_id).fetch_optional(&mut *tx).await?;
        if changed.is_some() {
            sqlx::query("INSERT INTO ticket_events(ticket_id,guild_id,actor_id,action,metadata) VALUES($1,$2,$3,'transferred',$4)").bind(id).bind(guild_id).bind(actor_id).bind(serde_json::json!({"assigned_to":assignee_id})).execute(&mut *tx).await?;
        }
        tx.commit().await?;
        match changed {
            Some(_) => self.ticket(guild_id, id).await,
            None => Ok(None),
        }
    }

    pub async fn close_ticket(
        &self,
        guild_id: &str,
        id: i64,
        actor_id: &str,
        reason: &str,
        transcript: &str,
    ) -> Result<Option<Ticket>, StoreError> {
        let mut tx = self.pool.begin().await?;
        let changed:Option<i64>=sqlx::query_scalar("UPDATE tickets SET status='closed',closed_at=now(),closed_by=$3,closed_reason=$4,transcript=$5,updated_at=now() WHERE guild_id=$1 AND id=$2 AND status='open' RETURNING id").bind(guild_id).bind(id).bind(actor_id).bind(reason).bind(transcript).fetch_optional(&mut *tx).await?;
        if changed.is_some() {
            sqlx::query("INSERT INTO ticket_events(ticket_id,guild_id,actor_id,action,metadata) VALUES($1,$2,$3,'closed',$4)").bind(id).bind(guild_id).bind(actor_id).bind(serde_json::json!({"reason":reason})).execute(&mut *tx).await?;
        }
        tx.commit().await?;
        match changed {
            Some(_) => self.ticket(guild_id, id).await,
            None => Ok(None),
        }
    }

    pub async fn reopen_ticket(
        &self,
        guild_id: &str,
        id: i64,
        actor_id: &str,
    ) -> Result<Option<Ticket>, StoreError> {
        let mut tx = self.pool.begin().await?;
        let changed:Option<i64>=sqlx::query_scalar("UPDATE tickets SET status='open',closed_at=NULL,closed_by=NULL,closed_reason=NULL,reopened_at=now(),updated_at=now() WHERE guild_id=$1 AND id=$2 AND status='closed' RETURNING id").bind(guild_id).bind(id).fetch_optional(&mut *tx).await?;
        if changed.is_some() {
            sqlx::query("INSERT INTO ticket_events(ticket_id,guild_id,actor_id,action,metadata) VALUES($1,$2,$3,'reopened','{}')").bind(id).bind(guild_id).bind(actor_id).execute(&mut *tx).await?;
        }
        tx.commit().await?;
        match changed {
            Some(_) => self.ticket(guild_id, id).await,
            None => Ok(None),
        }
    }

    pub async fn ticket_events(
        &self,
        guild_id: &str,
        id: i64,
    ) -> Result<Vec<TicketEvent>, StoreError> {
        sqlx::query_as::<_,TicketEvent>("SELECT id,ticket_id,guild_id,actor_id,action,metadata,created_at FROM ticket_events WHERE guild_id=$1 AND ticket_id=$2 ORDER BY created_at,id").bind(guild_id).bind(id).fetch_all(&self.pool).await.map_err(StoreError::from)
    }

    pub async fn update_ticket_metadata(
        &self,
        guild_id: &str,
        id: i64,
        actor_id: &str,
        priority: &str,
        tags: &serde_json::Value,
    ) -> Result<Option<Ticket>, StoreError> {
        let mut tx = self.pool.begin().await?;
        let changed:Option<i64>=sqlx::query_scalar("UPDATE tickets SET priority=$3,tags=$4,updated_at=now() WHERE guild_id=$1 AND id=$2 RETURNING id").bind(guild_id).bind(id).bind(priority).bind(tags).fetch_optional(&mut *tx).await?;
        if changed.is_some() {
            sqlx::query("INSERT INTO ticket_events(ticket_id,guild_id,actor_id,action,metadata) VALUES($1,$2,$3,'metadata_updated',$4)").bind(id).bind(guild_id).bind(actor_id).bind(serde_json::json!({"priority":priority,"tags":tags})).execute(&mut *tx).await?;
        }
        tx.commit().await?;
        match changed {
            Some(_) => self.ticket(guild_id, id).await,
            None => Ok(None),
        }
    }
    pub async fn add_ticket_note(
        &self,
        guild_id: &str,
        id: i64,
        author_id: &str,
        body: &str,
    ) -> Result<TicketNote, StoreError> {
        let mut tx = self.pool.begin().await?;
        let note=sqlx::query_as::<_,TicketNote>("INSERT INTO ticket_notes(ticket_id,guild_id,author_id,body) SELECT id,guild_id,$3,$4 FROM tickets WHERE guild_id=$1 AND id=$2 RETURNING id,ticket_id,guild_id,author_id,body,created_at").bind(guild_id).bind(id).bind(author_id).bind(body.trim()).fetch_one(&mut *tx).await?;
        sqlx::query("INSERT INTO ticket_events(ticket_id,guild_id,actor_id,action,metadata) VALUES($1,$2,$3,'note_added','{}')").bind(id).bind(guild_id).bind(author_id).execute(&mut *tx).await?;
        tx.commit().await?;
        Ok(note)
    }
    pub async fn ticket_notes(
        &self,
        guild_id: &str,
        id: i64,
    ) -> Result<Vec<TicketNote>, StoreError> {
        sqlx::query_as::<_,TicketNote>("SELECT id,ticket_id,guild_id,author_id,body,created_at FROM ticket_notes WHERE guild_id=$1 AND ticket_id=$2 ORDER BY created_at DESC,id DESC").bind(guild_id).bind(id).fetch_all(&self.pool).await.map_err(StoreError::from)
    }
    pub async fn touch_ticket_channel(
        &self,
        guild_id: &str,
        channel_id: &str,
    ) -> Result<bool, StoreError> {
        Ok(sqlx::query("UPDATE tickets SET last_activity_at=now(),updated_at=now() WHERE guild_id=$1 AND channel_id=$2 AND status='open'").bind(guild_id).bind(channel_id).execute(&self.pool).await?.rows_affected()==1)
    }
    pub async fn due_inactive_tickets(
        &self,
        guild_id: &str,
        limit: i64,
    ) -> Result<Vec<Ticket>, StoreError> {
        sqlx::query_as::<_,Ticket>("SELECT t.id,t.guild_id,t.panel_id,t.opener_id,d.username AS opener_username,d.display_name AS opener_display_name,t.channel_id,t.subject,t.opening_note,t.status,t.claimed_by,t.assigned_to,t.closed_by,t.closed_reason,t.transcript,t.priority,t.tags,t.first_response_at,t.last_activity_at,t.opened_at,t.closed_at,t.reopened_at,t.updated_at FROM tickets t LEFT JOIN discord_identities d ON d.user_id=t.opener_id JOIN ticket_panels p ON p.id=t.panel_id WHERE t.guild_id=$1 AND t.status='open' AND p.enabled AND p.auto_close_hours>0 AND t.last_activity_at < now()-(p.auto_close_hours * interval '1 hour') ORDER BY t.last_activity_at LIMIT $2")
            .bind(guild_id)
            .bind(limit.clamp(1, 100))
            .fetch_all(&self.pool)
            .await
            .map_err(StoreError::from)
    }
    pub async fn ticket_analytics(&self, guild_id: &str) -> Result<TicketAnalytics, StoreError> {
        sqlx::query_as::<_,TicketAnalytics>("SELECT count(*)::bigint total,count(*) FILTER(WHERE t.status='open')::bigint open,count(*) FILTER(WHERE t.status='closed')::bigint closed,count(*) FILTER(WHERE t.status='open' AND t.assigned_to IS NULL)::bigint unassigned,count(*) FILTER(WHERE t.status='open' AND p.auto_close_hours>0 AND t.last_activity_at < now()-(p.auto_close_hours * interval '1 hour'))::bigint overdue,percentile_cont(0.5) WITHIN GROUP(ORDER BY extract(epoch FROM(t.first_response_at-t.opened_at))/60) FILTER(WHERE t.first_response_at IS NOT NULL) median_first_response_minutes,percentile_cont(0.5) WITHIN GROUP(ORDER BY extract(epoch FROM(t.closed_at-t.opened_at))/60) FILTER(WHERE t.closed_at IS NOT NULL) median_resolution_minutes FROM tickets t LEFT JOIN ticket_panels p ON p.id=t.panel_id WHERE t.guild_id=$1").bind(guild_id).fetch_one(&self.pool).await.map_err(StoreError::from)
    }
    pub async fn purge_expired_ticket_transcripts(&self) -> Result<u64, StoreError> {
        Ok(sqlx::query("UPDATE tickets t SET transcript=NULL,updated_at=now() FROM ticket_panels p WHERE t.panel_id=p.id AND t.transcript IS NOT NULL AND t.closed_at < now()-(p.transcript_retention_days * interval '1 day')").execute(&self.pool).await?.rows_affected())
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

    pub async fn starboard_post(
        &self,
        guild_id: &str,
        source_message_id: &str,
    ) -> Result<Option<(String, String, String, u64)>, StoreError> {
        let row: Option<(Option<String>, String, Option<String>, i64)> = sqlx::query_as(
            "SELECT source_channel_id, starboard_channel_id, starboard_message_id, reaction_count \
             FROM starboard_posts WHERE guild_id = $1 AND source_message_id = $2",
        )
        .bind(guild_id)
        .bind(source_message_id)
        .fetch_optional(&self.pool)
        .await?;
        Ok(row.and_then(
            |(source_channel_id, starboard_channel_id, starboard_message_id, reaction_count)| {
                Some((
                    source_channel_id?,
                    starboard_channel_id,
                    starboard_message_id?,
                    u64::try_from(reaction_count).unwrap_or_default(),
                ))
            },
        ))
    }

    pub async fn complete_starboard_post(
        &self,
        guild_id: &str,
        source_message_id: &str,
        source_channel_id: &str,
        starboard_message_id: &str,
        reaction_count: u64,
    ) -> Result<(), StoreError> {
        sqlx::query(
            "UPDATE starboard_posts SET source_channel_id=$3, starboard_message_id=$4, \
             reaction_count=$5 WHERE guild_id=$1 AND source_message_id=$2",
        )
        .bind(guild_id)
        .bind(source_message_id)
        .bind(source_channel_id)
        .bind(starboard_message_id)
        .bind(i64::try_from(reaction_count).unwrap_or(i64::MAX))
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn remove_starboard_post(
        &self,
        guild_id: &str,
        source_message_id: &str,
    ) -> Result<(), StoreError> {
        sqlx::query("DELETE FROM starboard_posts WHERE guild_id=$1 AND source_message_id=$2")
            .bind(guild_id)
            .bind(source_message_id)
            .execute(&self.pool)
            .await?;
        Ok(())
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

    pub async fn runtime_metrics(&self) -> Result<RuntimeMetrics, StoreError> {
        let due_job_backlog: i64 = sqlx::query_scalar(
            "SELECT \
                (SELECT count(*) FROM message_deletion_jobs WHERE status='scheduled' AND due_at <= now()) + \
                (SELECT count(*) FROM reminders WHERE status='scheduled' AND next_attempt_at <= now()) + \
                (SELECT count(*) FROM giveaways WHERE status='active' AND ends_at <= now()) + \
                (SELECT count(*) FROM polls WHERE status='active' AND ends_at <= now()) + \
                (SELECT count(*) FROM autorole_jobs WHERE status='scheduled' AND due_at <= now()) + \
                (SELECT count(*) FROM moderation_jobs WHERE status='scheduled' AND due_at <= now())"
        )
        .fetch_one(&self.pool)
        .await?;
        Ok(RuntimeMetrics {
            database_pool_size: self.pool.size(),
            database_pool_idle: self.pool.num_idle(),
            due_job_backlog,
        })
    }

    pub async fn schedule_autorole_job(
        &self,
        guild_id: &str,
        user_id: &str,
        role_id: &str,
        action: &str,
        due_unix: u64,
    ) -> Result<bool, StoreError> {
        let due = chrono::DateTime::from_timestamp(i64::try_from(due_unix).unwrap_or(i64::MAX), 0)
            .unwrap_or_else(chrono::Utc::now);
        let result = sqlx::query("INSERT INTO autorole_jobs(guild_id,user_id,role_id,action,due_at) VALUES($1,$2,$3,$4,$5) ON CONFLICT DO NOTHING")
            .bind(guild_id).bind(user_id).bind(role_id).bind(action).bind(due).execute(&self.pool).await?;
        Ok(result.rows_affected() == 1)
    }

    pub async fn claim_due_autorole_jobs(
        &self,
        guild_id: &str,
        now_unix: u64,
        limit: i64,
    ) -> Result<Vec<crate::AutoroleJob>, StoreError> {
        let now = chrono::DateTime::from_timestamp(i64::try_from(now_unix).unwrap_or(i64::MAX), 0)
            .unwrap_or_else(chrono::Utc::now);
        sqlx::query("UPDATE autorole_jobs SET status='failed',last_error=COALESCE(last_error,'delivery lease expired'),updated_at=now() WHERE guild_id=$1 AND status='delivering' AND attempts>=5 AND updated_at < $2 - interval '5 minutes'").bind(guild_id).bind(now).execute(&self.pool).await?;
        sqlx::query_as::<_, crate::AutoroleJob>("WITH due AS (SELECT id FROM autorole_jobs WHERE guild_id=$1 AND ((status='scheduled' AND due_at <= $2) OR (status='delivering' AND attempts<5 AND updated_at < $2 - interval '5 minutes')) ORDER BY due_at,id LIMIT $3 FOR UPDATE SKIP LOCKED) UPDATE autorole_jobs j SET status='delivering',attempts=attempts+1,updated_at=now() FROM due WHERE j.id=due.id RETURNING j.*")
            .bind(guild_id).bind(now).bind(limit.clamp(1,100)).fetch_all(&self.pool).await.map_err(StoreError::from)
    }

    pub async fn finish_autorole_job(
        &self,
        guild_id: &str,
        job_id: i64,
        completed: bool,
        error: Option<&str>,
        retry_unix: u64,
    ) -> Result<bool, StoreError> {
        let retry =
            chrono::DateTime::from_timestamp(i64::try_from(retry_unix).unwrap_or(i64::MAX), 0)
                .unwrap_or_else(chrono::Utc::now);
        let terminal: Option<bool> = sqlx::query_scalar("UPDATE autorole_jobs SET status=CASE WHEN $3 THEN 'completed' WHEN attempts>=5 THEN 'failed' ELSE 'scheduled' END,completed_at=CASE WHEN $3 THEN now() ELSE NULL END,last_error=$4,due_at=CASE WHEN NOT $3 AND attempts<5 THEN $5 ELSE due_at END,updated_at=now() WHERE guild_id=$1 AND id=$2 AND status='delivering' RETURNING status='failed'")
            .bind(guild_id).bind(job_id).bind(completed).bind(error).bind(retry).fetch_optional(&self.pool).await?;
        Ok(terminal.unwrap_or(false))
    }

    pub async fn schedule_moderation_job(
        &self,
        guild_id: &str,
        member_id: &str,
        action: &str,
        reason: &str,
        due_unix: u64,
    ) -> Result<i64, StoreError> {
        let due = chrono::DateTime::from_timestamp(i64::try_from(due_unix).unwrap_or(i64::MAX), 0)
            .unwrap_or_else(chrono::Utc::now);
        sqlx::query_scalar("INSERT INTO moderation_jobs(guild_id,member_id,action,reason,due_at) VALUES($1,$2,$3,$4,$5) RETURNING id")
            .bind(guild_id).bind(member_id).bind(action).bind(reason).bind(due).fetch_one(&self.pool).await.map_err(StoreError::from)
    }

    pub async fn claim_due_moderation_jobs(
        &self,
        guild_id: &str,
        now_unix: u64,
        limit: i64,
    ) -> Result<Vec<crate::ModerationJob>, StoreError> {
        let now = chrono::DateTime::from_timestamp(i64::try_from(now_unix).unwrap_or(i64::MAX), 0)
            .unwrap_or_else(chrono::Utc::now);
        sqlx::query("UPDATE moderation_jobs SET status='failed',last_error=COALESCE(last_error,'delivery lease expired'),updated_at=now() WHERE guild_id=$1 AND status='delivering' AND attempts>=5 AND updated_at < $2 - interval '5 minutes'").bind(guild_id).bind(now).execute(&self.pool).await?;
        sqlx::query_as::<_, crate::ModerationJob>("WITH due AS (SELECT id FROM moderation_jobs WHERE guild_id=$1 AND ((status='scheduled' AND due_at <= $2) OR (status='delivering' AND attempts<5 AND updated_at < $2 - interval '5 minutes')) ORDER BY due_at,id LIMIT $3 FOR UPDATE SKIP LOCKED) UPDATE moderation_jobs j SET status='delivering',attempts=attempts+1,updated_at=now() FROM due WHERE j.id=due.id RETURNING j.*")
            .bind(guild_id).bind(now).bind(limit.clamp(1,100)).fetch_all(&self.pool).await.map_err(StoreError::from)
    }

    pub async fn finish_moderation_job(
        &self,
        guild_id: &str,
        job_id: i64,
        completed: bool,
        error: Option<&str>,
        retry_unix: u64,
    ) -> Result<bool, StoreError> {
        let retry =
            chrono::DateTime::from_timestamp(i64::try_from(retry_unix).unwrap_or(i64::MAX), 0)
                .unwrap_or_else(chrono::Utc::now);
        let terminal: Option<bool> = sqlx::query_scalar("UPDATE moderation_jobs SET status=CASE WHEN $3 THEN 'completed' WHEN attempts>=5 THEN 'failed' ELSE 'scheduled' END,completed_at=CASE WHEN $3 THEN now() ELSE NULL END,last_error=$4,due_at=CASE WHEN NOT $3 AND attempts<5 THEN $5 ELSE due_at END,updated_at=now() WHERE guild_id=$1 AND id=$2 AND status='delivering' RETURNING status='failed'")
            .bind(guild_id).bind(job_id).bind(completed).bind(error).bind(retry).fetch_optional(&self.pool).await?;
        Ok(terminal.unwrap_or(false))
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

    pub async fn upsert_discord_identity(
        &self,
        user_id: &str,
        username: &str,
        display_name: Option<&str>,
        avatar_url: Option<&str>,
    ) -> Result<(), StoreError> {
        sqlx::query(
            "INSERT INTO discord_identities (user_id, username, display_name, avatar_url) \
             VALUES ($1, $2, $3, $4) \
             ON CONFLICT (user_id) DO UPDATE SET username = EXCLUDED.username, \
             display_name = EXCLUDED.display_name, avatar_url = EXCLUDED.avatar_url, updated_at = now()",
        )
        .bind(user_id)
        .bind(username)
        .bind(display_name)
        .bind(avatar_url)
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
                    COALESCE((SELECT d.avatar_url FROM discord_identities d WHERE d.user_id = i.user_id), \
                             (SELECT u.image FROM account oauth_account JOIN \"user\" u ON u.id = oauth_account.\"userId\" \
                              WHERE oauth_account.\"providerId\" = 'discord' AND oauth_account.\"accountId\" = i.user_id LIMIT 1)) \
                    AS avatar_url, \
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
             SELECT i.user_id AS id, d.username, d.display_name, d.avatar_url, \
                    count(a.id) FILTER (WHERE a.action LIKE 'slash_command.%' AND a.occurred_at >= now() - interval '30 days') AS commands_30d, \
                    (SELECT count(*) FROM moderation_cases c WHERE c.guild_id = $1 AND c.member_id = i.user_id) AS cases, \
                    max(a.occurred_at) AS last_seen \
             FROM identities i LEFT JOIN discord_identities d ON d.user_id = i.user_id \
             LEFT JOIN audit_events a ON a.guild_id = $1 AND a.actor_id = i.user_id \
             GROUP BY i.user_id, d.username, d.display_name, d.avatar_url ORDER BY last_seen DESC NULLS LAST",
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

use std::sync::Arc;

use anyhow::Result;
use async_trait::async_trait;
use bot_core::GuildConfig;
use bot_discord::{
    CommandDefinition, CommandInteraction, ComponentInteraction, DiscordApi, ModalSubmitInteraction,
};

#[derive(Clone)]
pub struct ModuleContext {
    pub discord: Arc<dyn DiscordApi>,
    pub services: Arc<dyn ModuleServices>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ModerationCaseInput {
    pub guild_id: u64,
    pub moderator_id: u64,
    pub member_id: u64,
    pub action: String,
    pub reason: String,
    pub evidence_urls: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct XpAwardInput {
    pub guild_id: u64,
    pub user_id: u64,
    pub amount: u16,
    pub cooldown_seconds: u16,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct XpAwardResult {
    pub awarded: bool,
    pub total_xp: u64,
    pub level: u32,
    pub leveled_up: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct XpProfile {
    pub user_id: u64,
    pub total_xp: u64,
    pub level: u32,
    pub message_count: u64,
    pub rank: u64,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct XpAdjustmentInput {
    pub guild_id: u64,
    pub user_id: u64,
    pub amount: i64,
    pub operation: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TemporaryVoiceResult {
    pub channel_id: u64,
    pub owner_id: u64,
    pub name: String,
    pub user_limit: u8,
    pub locked: bool,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EconomyProfile {
    pub user_id: u64,
    pub balance: u64,
    pub lifetime_earned: u64,
    pub lifetime_spent: u64,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EconomyClaimResult {
    pub profile: Option<EconomyProfile>,
    pub retry_after_seconds: u64,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AfkStatusResult {
    pub user_id: u64,
    pub reason: String,
    pub set_at_unix: u64,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PollResult {
    pub id: i64,
    pub question: String,
    pub options: Vec<String>,
    pub counts: Vec<u64>,
    pub total_votes: u64,
    pub anonymous: bool,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DuePollResult {
    pub id: i64,
    pub channel_id: u64,
    pub message_id: u64,
    pub question: String,
    pub options: Vec<String>,
    pub counts: Vec<u64>,
    pub total_votes: u64,
    pub anonymous: bool,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct InviteUsageInput {
    pub code: String,
    pub inviter_id: Option<u64>,
    pub uses: u64,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct InviteAttributionResult {
    pub code: String,
    pub inviter_id: Option<u64>,
    pub inviter_total: u64,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TicketRecordInput {
    pub guild_id: u64,
    pub opener_id: u64,
    pub channel_id: u64,
    pub panel_id: Option<i64>,
    pub subject: String,
    pub opening_note: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TicketPanelResult {
    pub id: i64,
    pub category_id: u64,
    pub support_role_ids: Vec<u64>,
    pub welcome_message: String,
    pub channel_name: String,
    pub intake_enabled: bool,
    pub intake_label: String,
    pub max_open_per_user: u8,
    pub enabled: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TicketResult {
    pub id: i64,
    pub panel_id: Option<i64>,
    pub opener_id: u64,
    pub channel_id: u64,
    pub status: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FormAnswerInput {
    pub question_id: String,
    pub label: String,
    pub answer: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FormSubmissionInput {
    pub guild_id: u64,
    pub form_id: String,
    pub submitter_id: Option<u64>,
    pub answers: Vec<FormAnswerInput>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SuggestionInput {
    pub guild_id: u64,
    pub channel_id: u64,
    pub author_id: Option<u64>,
    pub content: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SuggestionResult {
    pub id: i64,
    pub channel_id: u64,
    pub message_id: Option<u64>,
    pub author_id: Option<u64>,
    pub content: String,
    pub status: String,
    pub staff_note: Option<String>,
    pub upvotes: u64,
    pub downvotes: u64,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GiveawayInput {
    pub guild_id: u64,
    pub channel_id: u64,
    pub creator_id: u64,
    pub title: String,
    pub description: String,
    pub winner_count: u8,
    pub minimum_account_age_days: u16,
    pub required_role_ids: Vec<u64>,
    pub bonus_role_id: Option<u64>,
    pub bonus_entries: u8,
    pub ends_at_unix: u64,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GiveawayResult {
    pub id: i64,
    pub channel_id: u64,
    pub title: String,
    pub status: String,
    pub winner_count: u8,
    pub minimum_account_age_days: u16,
    pub required_role_ids: Vec<u64>,
    pub bonus_role_id: Option<u64>,
    pub bonus_entries: u8,
    pub winner_ids: Vec<u64>,
    pub entrant_count: u64,
    pub total_entries: u64,
    pub ends_at_unix: u64,
    pub reroll_count: u32,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ReminderInput {
    pub guild_id: u64,
    pub channel_id: Option<u64>,
    pub user_id: u64,
    pub message: String,
    pub delivery_mode: String,
    pub remind_at_unix: u64,
    pub max_active_per_user: u8,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ReminderResult {
    pub id: i64,
    pub channel_id: Option<u64>,
    pub user_id: u64,
    pub message: String,
    pub delivery_mode: String,
    pub status: String,
    pub remind_at_unix: u64,
    pub attempts: u8,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MessageDeletionJobResult {
    pub channel_id: u64,
    pub message_id: u64,
    pub attempts: u8,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AutoroleJobResult {
    pub id: i64,
    pub user_id: u64,
    pub role_id: u64,
    pub action: String,
    pub attempts: u8,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ModerationJobResult {
    pub id: i64,
    pub member_id: u64,
    pub action: String,
    pub reason: String,
    pub attempts: u8,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StarboardPostResult {
    pub source_channel_id: u64,
    pub starboard_message_id: u64,
    pub reaction_count: u64,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ManagedEmbedResult {
    pub id: i64,
    pub name: String,
    pub payload: bot_core::EmbedPayload,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ManagedEmbedDeliveryInput {
    pub guild_id: u64,
    pub template_id: i64,
    pub channel_id: u64,
    pub message_id: Option<u64>,
    pub actor_id: u64,
    pub status: String,
    pub error: Option<String>,
}

#[allow(clippy::too_many_arguments)]
#[async_trait]
pub trait ModuleServices: Send + Sync {
    async fn purge_guild_audit_events(&self, guild_id: u64, retention_days: u16) -> Result<u64>;
    async fn set_afk_status(
        &self,
        _guild_id: u64,
        _user_id: u64,
        _reason: &str,
        _now_unix: u64,
        _ttl_seconds: u64,
    ) -> Result<()> {
        anyhow::bail!("AFK service unavailable")
    }
    async fn afk_status(&self, _guild_id: u64, _user_id: u64) -> Result<Option<AfkStatusResult>> {
        Ok(None)
    }
    async fn clear_afk_status(&self, _guild_id: u64, _user_id: u64) -> Result<bool> {
        Ok(false)
    }
    async fn create_poll(
        &self,
        _guild_id: u64,
        _channel_id: u64,
        _creator_id: u64,
        _question: &str,
        _options: &[String],
        _anonymous: bool,
        _ttl_seconds: u64,
    ) -> Result<PollResult> {
        anyhow::bail!("poll service unavailable")
    }
    async fn vote_poll(
        &self,
        _guild_id: u64,
        _poll_id: i64,
        _user_id: u64,
        _option: usize,
    ) -> Result<Option<PollResult>> {
        Ok(None)
    }
    async fn close_poll(&self, _guild_id: u64, _poll_id: i64) -> Result<Option<DuePollResult>> {
        Ok(None)
    }
    async fn attach_poll_message(
        &self,
        _guild_id: u64,
        _poll_id: i64,
        _message_id: u64,
    ) -> Result<()> {
        Ok(())
    }
    async fn claim_due_polls(
        &self,
        _guild_id: u64,
        _now_unix: u64,
        _limit: u8,
    ) -> Result<Vec<DuePollResult>> {
        Ok(Vec::new())
    }
    async fn attribute_invite(
        &self,
        _guild_id: u64,
        _joined_user_id: u64,
        _invites: &[InviteUsageInput],
    ) -> Result<Option<InviteAttributionResult>> {
        Ok(None)
    }
    async fn ensure_invite_snapshot(
        &self,
        _guild_id: u64,
        _invites: &[InviteUsageInput],
    ) -> Result<bool> {
        Ok(false)
    }
    async fn invite_snapshot_ready(&self, _guild_id: u64) -> Result<bool> {
        Ok(false)
    }
    async fn record_verification_attempt(
        &self,
        _guild_id: u64,
        _user_id: u64,
        _outcome: &str,
        _reason: Option<&str>,
        _account_age_days: u64,
        _has_avatar: bool,
    ) -> Result<()> {
        Ok(())
    }
    #[allow(clippy::too_many_arguments)]
    async fn record_counter_run(
        &self,
        _guild_id: u64,
        _counter_id: &str,
        _channel_id: u64,
        _value: u64,
        _rendered_name: &str,
        _outcome: &str,
        _error: Option<&str>,
    ) -> Result<()> {
        Ok(())
    }
    async fn record_voice_link_event(
        &self,
        _guild_id: u64,
        _user_id: u64,
        _voice_channel_id: u64,
        _text_channel_id: u64,
        _action: &str,
    ) -> Result<()> {
        Ok(())
    }
    async fn claim_appeal_cooldown(
        &self,
        _guild_id: u64,
        _user_id: u64,
        _seconds: u64,
    ) -> Result<bool> {
        Ok(true)
    }
    async fn guild_name(&self, _guild_id: u64) -> Result<Option<String>> {
        Ok(None)
    }
    async fn managed_embed(
        &self,
        _guild_id: u64,
        _template_id: i64,
    ) -> Result<Option<ManagedEmbedResult>> {
        Ok(None)
    }
    async fn record_managed_embed_delivery(&self, _input: ManagedEmbedDeliveryInput) -> Result<()> {
        Ok(())
    }
    async fn purge_form_submissions(&self, _guild_id: u64, _retention_days: u16) -> Result<u64> {
        Ok(0)
    }
    async fn create_moderation_case(&self, input: ModerationCaseInput) -> Result<i64>;
    async fn moderation_warning_count(&self, _guild_id: u64, _member_id: u64) -> Result<u64> {
        Ok(0)
    }
    async fn automod_rate_count(
        &self,
        _guild_id: u64,
        _user_id: u64,
        _rule_id: &str,
        _fingerprint: Option<&str>,
        _message_id: u64,
        _window_seconds: u16,
    ) -> Result<u64> {
        Ok(1)
    }
    async fn award_xp(&self, input: XpAwardInput) -> Result<XpAwardResult>;
    async fn xp_profile(&self, guild_id: u64, user_id: u64) -> Result<XpProfile>;
    async fn xp_leaderboard(&self, guild_id: u64, limit: u8) -> Result<Vec<XpProfile>>;
    async fn adjust_xp(&self, input: XpAdjustmentInput) -> Result<XpProfile>;
    async fn temporary_voice_channel(
        &self,
        guild_id: u64,
        channel_id: u64,
    ) -> Result<Option<TemporaryVoiceResult>>;
    async fn temporary_voice_owner_channel(
        &self,
        guild_id: u64,
        owner_id: u64,
    ) -> Result<Option<TemporaryVoiceResult>>;
    async fn register_temporary_voice(
        &self,
        guild_id: u64,
        channel_id: u64,
        owner_id: u64,
        name: &str,
        user_limit: u8,
    ) -> Result<TemporaryVoiceResult>;
    async fn update_temporary_voice(
        &self,
        guild_id: u64,
        channel_id: u64,
        owner_id: Option<u64>,
        name: Option<&str>,
        user_limit: Option<u8>,
        locked: Option<bool>,
    ) -> Result<Option<TemporaryVoiceResult>>;
    async fn unregister_temporary_voice(&self, guild_id: u64, channel_id: u64) -> Result<bool>;
    async fn temporary_voice_member_join(&self, channel_id: u64, user_id: u64) -> Result<u64>;
    async fn temporary_voice_member_leave(&self, channel_id: u64, user_id: u64) -> Result<u64>;
    async fn schedule_temporary_voice_delete(
        &self,
        guild_id: u64,
        channel_id: u64,
        delay_seconds: u16,
    ) -> Result<()>;
    async fn cancel_temporary_voice_delete(&self, guild_id: u64, channel_id: u64) -> Result<()>;
    async fn due_temporary_voice_deletes(
        &self,
        guild_id: u64,
        now_unix: u64,
    ) -> Result<Vec<TemporaryVoiceResult>>;
    async fn economy_profile(&self, guild_id: u64, user_id: u64) -> Result<EconomyProfile>;
    async fn economy_leaderboard(&self, guild_id: u64, limit: u8) -> Result<Vec<EconomyProfile>>;
    async fn economy_change_balance(
        &self,
        guild_id: u64,
        user_id: u64,
        amount: i64,
        kind: &str,
        reference: &str,
    ) -> Result<EconomyProfile>;
    async fn economy_claim(
        &self,
        guild_id: u64,
        user_id: u64,
        kind: &str,
        amount: u64,
        cooldown_seconds: u64,
        reference: &str,
    ) -> Result<EconomyClaimResult>;
    async fn economy_transfer(
        &self,
        guild_id: u64,
        from: u64,
        to: u64,
        amount: u64,
        tax_percent: u8,
        reference: &str,
    ) -> Result<(EconomyProfile, EconomyProfile, u64)>;
    async fn open_ticket_channel(&self, guild_id: u64, opener_id: u64) -> Result<Option<u64>>;
    async fn ticket_panel(
        &self,
        _guild_id: u64,
        _panel_id: i64,
    ) -> Result<Option<TicketPanelResult>> {
        Ok(None)
    }
    async fn open_ticket_count(
        &self,
        _guild_id: u64,
        _panel_id: Option<i64>,
        _opener_id: u64,
    ) -> Result<u8> {
        Ok(0)
    }
    async fn record_open_ticket(&self, input: TicketRecordInput) -> Result<()>;
    async fn ticket_by_channel(
        &self,
        _guild_id: u64,
        _channel_id: u64,
    ) -> Result<Option<TicketResult>> {
        Ok(None)
    }
    async fn ticket(&self, _guild_id: u64, _ticket_id: i64) -> Result<Option<TicketResult>> {
        Ok(None)
    }
    async fn claim_ticket(&self, _guild_id: u64, _ticket_id: i64, _actor_id: u64) -> Result<bool> {
        Ok(false)
    }
    async fn close_ticket(
        &self,
        _guild_id: u64,
        _ticket_id: i64,
        _actor_id: u64,
        _reason: &str,
        _transcript: &str,
    ) -> Result<bool> {
        Ok(false)
    }
    async fn touch_ticket_activity(&self, _guild_id: u64, _channel_id: u64) -> Result<bool> {
        Ok(false)
    }
    async fn due_inactive_tickets(&self, _guild_id: u64, _limit: u8) -> Result<Vec<TicketResult>> {
        Ok(Vec::new())
    }
    async fn purge_expired_ticket_transcripts(&self) -> Result<u64> {
        Ok(0)
    }
    async fn close_ticket_channel(
        &self,
        guild_id: u64,
        channel_id: u64,
        closed_by: u64,
    ) -> Result<bool>;
    async fn claim_starboard_post(
        &self,
        guild_id: u64,
        source_message_id: u64,
        starboard_channel_id: u64,
    ) -> Result<bool>;
    async fn starboard_post(
        &self,
        guild_id: u64,
        source_message_id: u64,
    ) -> Result<Option<StarboardPostResult>>;
    async fn complete_starboard_post(
        &self,
        guild_id: u64,
        source_message_id: u64,
        source_channel_id: u64,
        starboard_message_id: u64,
        reaction_count: u64,
    ) -> Result<()>;
    async fn remove_starboard_post(&self, guild_id: u64, source_message_id: u64) -> Result<()>;
    async fn record_raid_join(
        &self,
        guild_id: u64,
        user_id: u64,
        window_seconds: u16,
    ) -> Result<u64>;
    async fn set_raid_lockdown(&self, guild_id: u64, duration_seconds: u64) -> Result<()>;
    async fn clear_raid_lockdown(&self, guild_id: u64) -> Result<()>;
    async fn raid_lockdown_active(&self, guild_id: u64) -> Result<bool>;
    async fn claim_scheduled_message(
        &self,
        guild_id: u64,
        schedule_id: &str,
        occurrence_unix: u64,
    ) -> Result<bool>;
    async fn release_scheduled_message(
        &self,
        guild_id: u64,
        schedule_id: &str,
        occurrence_unix: u64,
    ) -> Result<()>;
    async fn create_form_submission(&self, input: FormSubmissionInput) -> Result<i64>;
    async fn create_suggestion(&self, input: SuggestionInput) -> Result<SuggestionResult>;
    async fn claim_suggestion_cooldown(
        &self,
        guild_id: u64,
        user_id: u64,
        cooldown_seconds: u32,
    ) -> Result<bool>;
    async fn claim_module_cooldown(
        &self,
        module: &str,
        rule: &str,
        guild_id: u64,
        user_id: u64,
        cooldown_seconds: u32,
    ) -> Result<bool>;
    async fn set_suggestion_message_id(
        &self,
        guild_id: u64,
        suggestion_id: i64,
        message_id: u64,
    ) -> Result<()>;
    async fn vote_suggestion(
        &self,
        guild_id: u64,
        suggestion_id: i64,
        user_id: u64,
        direction: i8,
    ) -> Result<Option<SuggestionResult>>;
    async fn update_suggestion_status(
        &self,
        guild_id: u64,
        suggestion_id: i64,
        actor_id: u64,
        status: &str,
        staff_note: Option<&str>,
    ) -> Result<Option<SuggestionResult>>;
    async fn create_giveaway(&self, input: GiveawayInput) -> Result<GiveawayResult>;
    async fn activate_giveaway(
        &self,
        guild_id: u64,
        giveaway_id: i64,
        message_id: u64,
    ) -> Result<bool>;
    async fn fail_giveaway_delivery(&self, guild_id: u64, giveaway_id: i64) -> Result<()>;
    async fn giveaway(&self, guild_id: u64, giveaway_id: i64) -> Result<Option<GiveawayResult>>;
    async fn enter_giveaway(
        &self,
        guild_id: u64,
        giveaway_id: i64,
        user_id: u64,
        weight: u8,
    ) -> Result<Option<bool>>;
    async fn due_giveaway_ids(&self, guild_id: u64, now_unix: u64) -> Result<Vec<i64>>;
    async fn pending_giveaway_announcement_ids(&self, guild_id: u64) -> Result<Vec<i64>>;
    async fn mark_giveaway_announced(&self, guild_id: u64, giveaway_id: i64) -> Result<()>;
    async fn end_giveaway(
        &self,
        guild_id: u64,
        giveaway_id: i64,
        force: bool,
    ) -> Result<Option<GiveawayResult>>;
    async fn reroll_giveaway(
        &self,
        guild_id: u64,
        giveaway_id: i64,
    ) -> Result<Option<GiveawayResult>>;
    async fn create_reminder(&self, input: ReminderInput) -> Result<Option<ReminderResult>>;
    async fn user_reminders(
        &self,
        guild_id: u64,
        user_id: u64,
        limit: u8,
    ) -> Result<Vec<ReminderResult>>;
    async fn cancel_reminder(&self, guild_id: u64, reminder_id: i64, user_id: u64) -> Result<bool>;
    async fn claim_due_reminders(
        &self,
        guild_id: u64,
        now_unix: u64,
        limit: u8,
    ) -> Result<Vec<ReminderResult>>;
    async fn complete_reminder(&self, guild_id: u64, reminder_id: i64) -> Result<()>;
    async fn retry_reminder(
        &self,
        guild_id: u64,
        reminder_id: i64,
        next_attempt_unix: u64,
    ) -> Result<bool>;
    async fn schedule_message_deletion(
        &self,
        _guild_id: u64,
        _channel_id: u64,
        _message_id: u64,
        _rule_id: &str,
        _author_id: u64,
        _due_unix: u64,
    ) -> Result<bool> {
        anyhow::bail!("message deletion service unavailable")
    }
    async fn claim_due_message_deletions(
        &self,
        _guild_id: u64,
        _now_unix: u64,
        _limit: u8,
    ) -> Result<Vec<MessageDeletionJobResult>> {
        anyhow::bail!("message deletion service unavailable")
    }
    async fn finish_message_deletion(
        &self,
        _guild_id: u64,
        _channel_id: u64,
        _message_id: u64,
        _deleted: bool,
        _retry_unix: u64,
    ) -> Result<bool> {
        anyhow::bail!("message deletion service unavailable")
    }
    async fn schedule_autorole_job(
        &self,
        _guild_id: u64,
        _user_id: u64,
        _role_id: u64,
        _action: &str,
        _due_unix: u64,
    ) -> Result<bool> {
        anyhow::bail!("autorole job service unavailable")
    }
    async fn claim_due_autorole_jobs(
        &self,
        _guild_id: u64,
        _now_unix: u64,
        _limit: u8,
    ) -> Result<Vec<AutoroleJobResult>> {
        anyhow::bail!("autorole job service unavailable")
    }
    async fn finish_autorole_job(
        &self,
        _guild_id: u64,
        _job_id: i64,
        _completed: bool,
        _error: Option<&str>,
        _retry_unix: u64,
    ) -> Result<bool> {
        anyhow::bail!("autorole job service unavailable")
    }
    async fn schedule_moderation_job(
        &self,
        _guild_id: u64,
        _member_id: u64,
        _action: &str,
        _reason: &str,
        _due_unix: u64,
    ) -> Result<i64> {
        anyhow::bail!("moderation job service unavailable")
    }
    async fn claim_due_moderation_jobs(
        &self,
        _guild_id: u64,
        _now_unix: u64,
        _limit: u8,
    ) -> Result<Vec<ModerationJobResult>> {
        anyhow::bail!("moderation job service unavailable")
    }
    async fn finish_moderation_job(
        &self,
        _guild_id: u64,
        _job_id: i64,
        _completed: bool,
        _error: Option<&str>,
        _retry_unix: u64,
    ) -> Result<bool> {
        anyhow::bail!("moderation job service unavailable")
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MemberJoin {
    pub guild_id: u64,
    pub guild_name: String,
    pub user_id: u64,
    pub username: String,
    pub display_name: String,
    pub account_age_seconds: u64,
    pub has_avatar: bool,
    pub is_bot: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MemberLeave {
    pub guild_id: u64,
    pub guild_name: String,
    pub user_id: u64,
    pub username: String,
    pub display_name: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AuditNotice {
    pub guild_id: u64,
    pub action: String,
    pub actor_id: String,
    pub outcome: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DiscordLogEvent {
    pub guild_id: u64,
    pub category: String,
    pub action: String,
    pub title: String,
    pub summary: String,
    pub actor_id: Option<u64>,
    pub target_type: String,
    pub target_id: Option<String>,
    pub channel_id: Option<u64>,
    pub details: Vec<(String, String)>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MessageCreate {
    pub guild_id: u64,
    pub channel_id: u64,
    pub message_id: u64,
    pub author_id: u64,
    pub author_is_bot: bool,
    pub role_ids: Vec<u64>,
    pub mention_count: usize,
    pub content: String,
    pub attachment_filenames: Vec<String>,
    pub sticker_count: usize,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct VoiceStateChange {
    pub guild_id: u64,
    pub user_id: u64,
    pub previous_channel_id: Option<u64>,
    pub channel_id: Option<u64>,
    pub previous_session_seconds: u64,
    pub previous_session_eligible: bool,
    pub user_is_bot: bool,
    pub username: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ReactionAdd {
    pub guild_id: u64,
    pub channel_id: u64,
    pub message_id: u64,
    pub reactor_id: u64,
    pub message_author_id: Option<u64>,
    pub emoji: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ModuleEventOutcome {
    pub action: String,
    pub target_type: String,
    pub target_id: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ScheduledTick {
    pub now_unix: u64,
}

impl ModuleEventOutcome {
    pub fn message(action: impl Into<String>, message_id: u64) -> Self {
        Self {
            action: action.into(),
            target_type: "message".to_owned(),
            target_id: Some(message_id.to_string()),
        }
    }
}

#[async_trait]
pub trait BotModule: Send + Sync {
    fn name(&self) -> &'static str;
    fn enabled(&self, config: &GuildConfig) -> bool;
    fn commands(&self) -> &'static [CommandDefinition] {
        &[]
    }
    fn handles_member_join(&self) -> bool {
        false
    }
    fn handles_member_leave(&self) -> bool {
        false
    }
    fn handles_audit_notice(&self) -> bool {
        false
    }
    fn handles_discord_log_event(&self) -> bool {
        false
    }
    fn handles_message_create(&self) -> bool {
        false
    }
    fn handles_voice_state_change(&self) -> bool {
        false
    }
    fn handles_reaction_add(&self) -> bool {
        false
    }
    fn handles_reaction_remove(&self) -> bool {
        false
    }
    fn handles_scheduled_tick(&self) -> bool {
        false
    }
    fn handles_modal_submit(&self, _custom_id: &str) -> bool {
        false
    }
    fn handles_component(&self, _custom_id: &str) -> bool {
        false
    }

    async fn on_command(
        &self,
        _context: &ModuleContext,
        _config: &GuildConfig,
        _interaction: &CommandInteraction,
    ) -> Result<()> {
        Ok(())
    }

    async fn on_member_join(
        &self,
        _context: &ModuleContext,
        _config: &GuildConfig,
        _event: &MemberJoin,
    ) -> Result<()> {
        Ok(())
    }

    async fn on_member_leave(
        &self,
        _context: &ModuleContext,
        _config: &GuildConfig,
        _event: &MemberLeave,
    ) -> Result<()> {
        Ok(())
    }

    async fn on_audit_notice(
        &self,
        _context: &ModuleContext,
        _config: &GuildConfig,
        _event: &AuditNotice,
    ) -> Result<()> {
        Ok(())
    }

    async fn on_discord_log_event(
        &self,
        _context: &ModuleContext,
        _config: &GuildConfig,
        _event: &DiscordLogEvent,
    ) -> Result<()> {
        Ok(())
    }

    async fn on_message_create(
        &self,
        _context: &ModuleContext,
        _config: &GuildConfig,
        _event: &MessageCreate,
    ) -> Result<Option<ModuleEventOutcome>> {
        Ok(None)
    }

    async fn on_voice_state_change(
        &self,
        _context: &ModuleContext,
        _config: &GuildConfig,
        _event: &VoiceStateChange,
    ) -> Result<Option<ModuleEventOutcome>> {
        Ok(None)
    }

    async fn on_reaction_add(
        &self,
        _context: &ModuleContext,
        _config: &GuildConfig,
        _event: &ReactionAdd,
    ) -> Result<Option<ModuleEventOutcome>> {
        Ok(None)
    }

    async fn on_reaction_remove(
        &self,
        _context: &ModuleContext,
        _config: &GuildConfig,
        _event: &ReactionAdd,
    ) -> Result<Option<ModuleEventOutcome>> {
        Ok(None)
    }

    async fn on_scheduled_tick(
        &self,
        _context: &ModuleContext,
        _config: &GuildConfig,
        _event: &ScheduledTick,
    ) -> Result<Vec<ModuleEventOutcome>> {
        Ok(Vec::new())
    }

    async fn on_modal_submit(
        &self,
        _context: &ModuleContext,
        _config: &GuildConfig,
        _interaction: &ModalSubmitInteraction,
    ) -> Result<Option<ModuleEventOutcome>> {
        Ok(None)
    }

    async fn on_component(
        &self,
        _context: &ModuleContext,
        _config: &GuildConfig,
        _interaction: &ComponentInteraction,
    ) -> Result<Option<ModuleEventOutcome>> {
        Ok(None)
    }
}

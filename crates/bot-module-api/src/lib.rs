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
pub struct TicketRecordInput {
    pub guild_id: u64,
    pub opener_id: u64,
    pub channel_id: u64,
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
    pub status: String,
    pub upvotes: u64,
    pub downvotes: u64,
}

#[async_trait]
pub trait ModuleServices: Send + Sync {
    async fn create_moderation_case(&self, input: ModerationCaseInput) -> Result<i64>;
    async fn award_xp(&self, input: XpAwardInput) -> Result<XpAwardResult>;
    async fn xp_profile(&self, guild_id: u64, user_id: u64) -> Result<XpProfile>;
    async fn xp_leaderboard(&self, guild_id: u64, limit: u8) -> Result<Vec<XpProfile>>;
    async fn open_ticket_channel(&self, guild_id: u64, opener_id: u64) -> Result<Option<u64>>;
    async fn record_open_ticket(&self, input: TicketRecordInput) -> Result<()>;
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
pub struct MessageCreate {
    pub guild_id: u64,
    pub channel_id: u64,
    pub message_id: u64,
    pub author_id: u64,
    pub author_is_bot: bool,
    pub role_ids: Vec<u64>,
    pub mention_count: usize,
    pub content: String,
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
    fn handles_message_create(&self) -> bool {
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

    async fn on_message_create(
        &self,
        _context: &ModuleContext,
        _config: &GuildConfig,
        _event: &MessageCreate,
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

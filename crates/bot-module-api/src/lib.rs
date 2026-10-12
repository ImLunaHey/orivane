use std::sync::Arc;

use anyhow::Result;
use async_trait::async_trait;
use bot_core::GuildConfig;
use bot_discord::{CommandDefinition, CommandInteraction, DiscordApi};

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
pub struct TicketRecordInput {
    pub guild_id: u64,
    pub opener_id: u64,
    pub channel_id: u64,
}

#[async_trait]
pub trait ModuleServices: Send + Sync {
    async fn create_moderation_case(&self, input: ModerationCaseInput) -> Result<i64>;
    async fn award_xp(&self, input: XpAwardInput) -> Result<XpAwardResult>;
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
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MemberJoin {
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
    fn handles_audit_notice(&self) -> bool {
        false
    }
    fn handles_message_create(&self) -> bool {
        false
    }
    fn handles_reaction_add(&self) -> bool {
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
}

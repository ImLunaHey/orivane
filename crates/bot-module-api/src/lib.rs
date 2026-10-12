use std::sync::Arc;

use anyhow::Result;
use async_trait::async_trait;
use bot_core::GuildConfig;
use bot_discord::{CommandDefinition, CommandInteraction, DiscordApi};

#[derive(Clone)]
pub struct ModuleContext {
    pub discord: Arc<dyn DiscordApi>,
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
}

use anyhow::{Context, Result};
use async_trait::async_trait;
use bot_core::GuildConfig;
use bot_module_api::{AuditNotice, BotModule, ModuleContext};

pub struct LoggingModule;

#[async_trait]
impl BotModule for LoggingModule {
    fn name(&self) -> &'static str {
        "logging"
    }
    fn enabled(&self, config: &GuildConfig) -> bool {
        config.modules.logging.enabled
    }
    fn handles_audit_notice(&self) -> bool {
        true
    }

    async fn on_audit_notice(
        &self,
        context: &ModuleContext,
        config: &GuildConfig,
        event: &AuditNotice,
    ) -> Result<()> {
        let channel_id = config
            .modules
            .logging
            .channel_id
            .as_deref()
            .context("logging channel is not configured")?
            .parse::<u64>()
            .context("logging channel is invalid")?;
        let message = format!(
            "**{}** · actor `{}` · outcome `{}`",
            event.action, event.actor_id, event.outcome
        );
        context.discord.send_message(channel_id, &message).await?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use super::*;
    use bot_test_support::{DiscordCall, FakeDiscord};

    #[tokio::test]
    async fn sends_a_content_safe_audit_notice() {
        let discord = FakeDiscord::new();
        let context = ModuleContext {
            discord: Arc::new(discord.clone()),
        };
        let mut config = GuildConfig::for_guild("10");
        config.modules.logging.enabled = true;
        config.modules.logging.channel_id = Some("20".to_owned());
        let notice = AuditNotice {
            guild_id: 10,
            action: "slash_command.ping".to_owned(),
            actor_id: "30".to_owned(),
            outcome: "success".to_owned(),
        };
        LoggingModule
            .on_audit_notice(&context, &config, &notice)
            .await
            .unwrap();
        assert_eq!(
            discord.calls(),
            vec![DiscordCall::SendMessage {
                channel_id: 20,
                content: "**slash_command.ping** · actor `30` · outcome `success`".to_owned()
            }]
        );
    }
}

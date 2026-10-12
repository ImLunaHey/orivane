use anyhow::{Context, Result};
use async_trait::async_trait;
use bot_core::{GuildConfig, LoggingConfig};
use bot_discord::{RichEmbed, RichEmbedField};
use bot_module_api::{
    AuditNotice, BotModule, DiscordLogEvent, ModuleContext, ModuleEventOutcome, ScheduledTick,
};

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
    fn handles_discord_log_event(&self) -> bool {
        true
    }
    fn handles_scheduled_tick(&self) -> bool {
        true
    }

    async fn on_scheduled_tick(
        &self,
        context: &ModuleContext,
        config: &GuildConfig,
        _event: &ScheduledTick,
    ) -> Result<Vec<ModuleEventOutcome>> {
        let deleted = context
            .services
            .purge_guild_audit_events(
                config.guild_id.parse().context("guild ID is invalid")?,
                config.modules.logging.retention_days,
            )
            .await?;
        if deleted == 0 {
            return Ok(Vec::new());
        }
        Ok(vec![ModuleEventOutcome {
            action: "logging.retention_purged".into(),
            target_type: "audit_events".into(),
            target_id: Some(deleted.to_string()),
        }])
    }

    async fn on_audit_notice(
        &self,
        context: &ModuleContext,
        config: &GuildConfig,
        event: &AuditNotice,
    ) -> Result<()> {
        if !config.modules.logging.command_events {
            return Ok(());
        }
        let channel_id = category_channel(&config.modules.logging, "commands")?;
        context
            .discord
            .send_message(
                channel_id,
                &format!(
                    "**{}** · actor `{}` · outcome `{}`",
                    event.action, event.actor_id, event.outcome
                ),
            )
            .await?;
        Ok(())
    }

    async fn on_discord_log_event(
        &self,
        context: &ModuleContext,
        config: &GuildConfig,
        event: &DiscordLogEvent,
    ) -> Result<()> {
        let logging = &config.modules.logging;
        if !category_enabled(logging, &event.category)
            || event.channel_id.is_some_and(|channel_id| {
                logging
                    .ignored_channel_ids
                    .iter()
                    .any(|ignored| ignored == &channel_id.to_string())
            })
        {
            return Ok(());
        }
        let channel_id = category_channel(logging, &event.category)?;
        let mut fields = Vec::with_capacity(event.details.len() + 3);
        if let Some(actor_id) = event.actor_id {
            fields.push(field("Actor", format!("<@{actor_id}>\n`{actor_id}`"), true));
        }
        if let Some(target_id) = &event.target_id {
            fields.push(field(
                "Target",
                format!("{}\n`{target_id}`", event.target_type),
                true,
            ));
        }
        if let Some(source_channel_id) = event.channel_id {
            fields.push(field(
                "Channel",
                format!("<#{source_channel_id}>\n`{source_channel_id}`"),
                true,
            ));
        }
        fields.extend(
            event
                .details
                .iter()
                .take(22)
                .map(|(name, value)| field(name, value, true)),
        );
        context
            .discord
            .send_rich_embed(
                channel_id,
                &RichEmbed {
                    title: event.title.clone(),
                    description: event.summary.clone(),
                    color: category_color(&event.category),
                    footer_text: format!("{} · {}", event.category, event.action),
                    timestamp: true,
                    fields,
                    ..RichEmbed::default()
                },
            )
            .await?;
        Ok(())
    }
}

fn field(name: impl Into<String>, value: impl Into<String>, inline: bool) -> RichEmbedField {
    RichEmbedField {
        name: name.into(),
        value: value.into(),
        inline,
    }
}
fn default_channel(config: &LoggingConfig) -> Result<u64> {
    config
        .channel_id
        .as_deref()
        .context("logging channel is not configured")?
        .parse::<u64>()
        .context("logging channel is invalid")
}
fn category_channel(config: &LoggingConfig, category: &str) -> Result<u64> {
    let override_id = match category {
        "commands" => config.command_channel_id.as_deref(),
        "messages" => config.message_channel_id.as_deref(),
        "members" => config.member_channel_id.as_deref(),
        "moderation" => config.moderation_channel_id.as_deref(),
        "roles" => config.role_channel_id.as_deref(),
        "channels" => config.channel_event_channel_id.as_deref(),
        "voice" => config.voice_channel_id.as_deref(),
        _ => None,
    };
    override_id.map_or_else(
        || default_channel(config),
        |id| {
            id.parse::<u64>()
                .context("category logging channel is invalid")
        },
    )
}
fn category_enabled(config: &LoggingConfig, category: &str) -> bool {
    match category {
        "messages" => config.message_events,
        "members" => config.member_events,
        "moderation" => config.moderation_events,
        "roles" => config.role_events,
        "channels" => config.channel_events,
        "voice" => config.voice_events,
        _ => false,
    }
}
fn category_color(category: &str) -> u32 {
    match category {
        "messages" => 0x5865f2,
        "members" => 0x57f287,
        "moderation" => 0xed4245,
        "roles" => 0xfee75c,
        "channels" => 0x3498db,
        "voice" => 0x9b59b6,
        _ => 0x99aab5,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bot_test_support::{DiscordCall, FakeDiscord, FakeModuleServices};
    use std::sync::Arc;

    fn context(discord: &FakeDiscord) -> ModuleContext {
        ModuleContext {
            discord: Arc::new(discord.clone()),
            services: Arc::new(FakeModuleServices::default()),
        }
    }

    #[tokio::test]
    async fn sends_a_content_safe_audit_notice() {
        let discord = FakeDiscord::new();
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
            .on_audit_notice(&context(&discord), &config, &notice)
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

    #[tokio::test]
    async fn routes_rich_events_to_category_channels_without_message_content() {
        let discord = FakeDiscord::new();
        let mut config = GuildConfig::for_guild("10");
        config.modules.logging.enabled = true;
        config.modules.logging.channel_id = Some("20".to_owned());
        config.modules.logging.message_channel_id = Some("21".to_owned());
        let event = DiscordLogEvent {
            guild_id: 10,
            category: "messages".into(),
            action: "discord.message.deleted".into(),
            title: "Message deleted".into(),
            summary: "A message was removed. Content was not retained.".into(),
            actor_id: None,
            target_type: "message".into(),
            target_id: Some("40".into()),
            channel_id: Some(50),
            details: vec![("Privacy".into(), "Content not retained".into())],
        };
        LoggingModule
            .on_discord_log_event(&context(&discord), &config, &event)
            .await
            .unwrap();
        let calls = discord.calls();
        let DiscordCall::SendRichEmbed { channel_id, embed } = &calls[0] else {
            panic!("expected a rich log message")
        };
        assert_eq!(*channel_id, 21);
        assert_eq!(embed.title, "Message deleted");
        assert!(!embed.description.contains("secret"));
    }

    #[tokio::test]
    async fn skips_ignored_channels() {
        let discord = FakeDiscord::new();
        let mut config = GuildConfig::for_guild("10");
        config.modules.logging.enabled = true;
        config.modules.logging.channel_id = Some("20".to_owned());
        config.modules.logging.ignored_channel_ids = vec!["50".into()];
        let event = DiscordLogEvent {
            guild_id: 10,
            category: "messages".into(),
            action: "discord.message.updated".into(),
            title: "Message edited".into(),
            summary: "Metadata changed.".into(),
            actor_id: Some(30),
            target_type: "message".into(),
            target_id: Some("40".into()),
            channel_id: Some(50),
            details: vec![],
        };
        LoggingModule
            .on_discord_log_event(&context(&discord), &config, &event)
            .await
            .unwrap();
        assert!(discord.calls().is_empty());
    }

    #[tokio::test]
    async fn purges_records_older_than_the_configured_retention() {
        let discord = FakeDiscord::new();
        let services = FakeModuleServices::default();
        let context = ModuleContext {
            discord: Arc::new(discord),
            services: Arc::new(services.clone()),
        };
        let mut config = GuildConfig::for_guild("10");
        config.modules.logging.retention_days = 90;
        let outcomes = LoggingModule
            .on_scheduled_tick(&context, &config, &ScheduledTick { now_unix: 1 })
            .await
            .unwrap();
        assert_eq!(outcomes[0].action, "logging.retention_purged");
        assert!(services.calls().iter().any(|call| matches!(
            call,
            bot_test_support::ServiceCall::PurgeGuildAuditEvents {
                guild_id: 10,
                retention_days: 90,
            }
        )));
    }
}

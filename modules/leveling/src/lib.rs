use anyhow::{Context, Result};
use async_trait::async_trait;
use bot_core::GuildConfig;
use bot_module_api::{BotModule, MessageCreate, ModuleContext, ModuleEventOutcome, XpAwardInput};

pub struct LevelingModule;

#[async_trait]
impl BotModule for LevelingModule {
    fn name(&self) -> &'static str {
        "leveling"
    }
    fn enabled(&self, config: &GuildConfig) -> bool {
        config.modules.leveling.enabled
    }
    fn handles_message_create(&self) -> bool {
        true
    }

    async fn on_message_create(
        &self,
        context: &ModuleContext,
        config: &GuildConfig,
        event: &MessageCreate,
    ) -> Result<Option<ModuleEventOutcome>> {
        if event.author_is_bot || event.content.trim().is_empty() {
            return Ok(None);
        }
        let settings = &config.modules.leveling;
        if settings
            .ignored_channel_ids
            .iter()
            .any(|id| id == &event.channel_id.to_string())
            || event.role_ids.iter().any(|role_id| {
                settings
                    .ignored_role_ids
                    .iter()
                    .any(|id| id == &role_id.to_string())
            })
        {
            return Ok(None);
        }
        let range = settings
            .xp_per_message_max
            .saturating_sub(settings.xp_per_message_min)
            .saturating_add(1);
        let amount = settings.xp_per_message_min
            + u16::try_from(event.message_id % u64::from(range)).unwrap_or_default();
        let result = context
            .services
            .award_xp(XpAwardInput {
                guild_id: event.guild_id,
                user_id: event.author_id,
                amount,
                cooldown_seconds: settings.cooldown_seconds,
            })
            .await?;
        if !result.awarded {
            return Ok(None);
        }
        if result.leveled_up && settings.announce_level_up {
            let channel_id = settings
                .announcement_channel_id
                .as_deref()
                .map(str::parse)
                .transpose()
                .context("level announcement channel is invalid")?
                .unwrap_or(event.channel_id);
            let message = settings
                .announcement_message
                .replace("{user}", &format!("<@{}>", event.author_id))
                .replace("{level}", &result.level.to_string())
                .replace("{xp}", &result.total_xp.to_string());
            context.discord.send_message(channel_id, &message).await?;
        }
        Ok(Some(ModuleEventOutcome::message(
            "leveling.xp_awarded",
            event.message_id,
        )))
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use bot_test_support::{FakeDiscord, FakeModuleServices, ServiceCall};

    use super::*;

    #[tokio::test]
    async fn awards_deterministic_xp_through_the_service_boundary() {
        let discord = FakeDiscord::new();
        let services = FakeModuleServices::default();
        let context = ModuleContext {
            discord: Arc::new(discord.clone()),
            services: Arc::new(services.clone()),
        };
        let mut config = GuildConfig::for_guild("10");
        config.modules.leveling.enabled = true;
        config.modules.leveling.xp_per_message_min = 10;
        config.modules.leveling.xp_per_message_max = 20;
        let event = MessageCreate {
            guild_id: 10,
            channel_id: 20,
            message_id: 33,
            author_id: 40,
            author_is_bot: false,
            role_ids: vec![],
            mention_count: 0,
            content: "A useful message".to_owned(),
        };
        assert!(
            LevelingModule
                .on_message_create(&context, &config, &event)
                .await
                .unwrap()
                .is_some()
        );
        assert_eq!(
            services.calls(),
            vec![ServiceCall::AwardXp(XpAwardInput {
                guild_id: 10,
                user_id: 40,
                amount: 10,
                cooldown_seconds: 60,
            })]
        );
        assert!(discord.calls().is_empty());
    }
}

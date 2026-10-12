use anyhow::{Context, Result};
use async_trait::async_trait;
use bot_core::GuildConfig;
use bot_module_api::{BotModule, ModuleContext, ModuleEventOutcome, ReactionAdd};

pub struct StarboardModule;

#[async_trait]
impl BotModule for StarboardModule {
    fn name(&self) -> &'static str {
        "starboard"
    }
    fn enabled(&self, config: &GuildConfig) -> bool {
        config.modules.starboard.enabled
    }

    fn handles_reaction_add(&self) -> bool {
        true
    }

    async fn on_reaction_add(
        &self,
        context: &ModuleContext,
        config: &GuildConfig,
        event: &ReactionAdd,
    ) -> Result<Option<ModuleEventOutcome>> {
        let settings = &config.modules.starboard;
        if event.emoji != settings.emoji
            || settings
                .ignored_channel_ids
                .iter()
                .any(|id| id == &event.channel_id.to_string())
            || (!settings.allow_self_stars && event.message_author_id == Some(event.reactor_id))
        {
            return Ok(None);
        }
        let summary = context
            .discord
            .message_reaction_summary(event.channel_id, event.message_id, &settings.emoji)
            .await?;
        if !settings.allow_self_stars && summary.author_id == event.reactor_id {
            return Ok(None);
        }
        if summary.count < u64::from(settings.threshold) {
            return Ok(None);
        }
        let destination = settings
            .channel_id
            .as_deref()
            .context("starboard channel is not configured")?
            .parse::<u64>()
            .context("starboard channel is invalid")?;
        if !context
            .services
            .claim_starboard_post(event.guild_id, event.message_id, destination)
            .await?
        {
            return Ok(None);
        }
        context
            .discord
            .send_message(
                destination,
                &format!(
                    "{} **{}** · <@{}> · https://discord.com/channels/{}/{}/{}",
                    settings.emoji,
                    summary.count,
                    summary.author_id,
                    event.guild_id,
                    event.channel_id,
                    event.message_id
                ),
            )
            .await?;
        Ok(Some(ModuleEventOutcome::message(
            "starboard.message_posted",
            event.message_id,
        )))
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use bot_discord::MessageReactionSummary;
    use bot_test_support::{DiscordCall, FakeDiscord, FakeModuleServices, ServiceCall};

    use super::*;

    #[tokio::test]
    async fn posts_once_after_the_threshold() {
        let discord = FakeDiscord::new();
        discord.set_message_reaction_summary(MessageReactionSummary {
            author_id: 50,
            count: 3,
        });
        let services = FakeModuleServices::default();
        let context = ModuleContext {
            discord: Arc::new(discord.clone()),
            services: Arc::new(services.clone()),
        };
        let mut config = GuildConfig::for_guild("10");
        config.modules.starboard.enabled = true;
        config.modules.starboard.channel_id = Some("90".to_owned());
        let event = ReactionAdd {
            guild_id: 10,
            channel_id: 20,
            message_id: 30,
            reactor_id: 40,
            message_author_id: Some(50),
            emoji: "⭐".to_owned(),
        };
        assert!(
            StarboardModule
                .on_reaction_add(&context, &config, &event)
                .await
                .unwrap()
                .is_some()
        );
        assert_eq!(
            services.calls(),
            vec![ServiceCall::ClaimStarboardPost {
                guild_id: 10,
                source_message_id: 30,
                starboard_channel_id: 90,
            }]
        );
        assert_eq!(
            discord.calls(),
            vec![
                DiscordCall::MessageReactionSummary {
                    channel_id: 20,
                    message_id: 30,
                    emoji: "⭐".to_owned(),
                },
                DiscordCall::SendMessage {
                    channel_id: 90,
                    content: "⭐ **3** · <@50> · https://discord.com/channels/10/20/30".to_owned(),
                },
            ]
        );
    }
}

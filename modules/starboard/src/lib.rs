use anyhow::{Context, Result};
use async_trait::async_trait;
use bot_core::GuildConfig;
use bot_discord::{MessageReactionSummary, RichEmbed};
use bot_module_api::{BotModule, DiscordLogEvent, ModuleContext, ModuleEventOutcome, ReactionAdd};

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
    fn handles_reaction_remove(&self) -> bool {
        true
    }
    fn handles_discord_log_event(&self) -> bool {
        true
    }

    async fn on_reaction_add(
        &self,
        context: &ModuleContext,
        config: &GuildConfig,
        event: &ReactionAdd,
    ) -> Result<Option<ModuleEventOutcome>> {
        let settings = &config.modules.starboard;
        let destination = destination(config)?;
        if event.emoji != settings.emoji
            || event.channel_id == destination
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
        if summary.count < u64::from(channel_threshold(config, event.channel_id)) {
            return Ok(None);
        }
        let embed = starboard_embed(config, event, &summary);
        if let Some(post) = context
            .services
            .starboard_post(event.guild_id, event.message_id)
            .await?
        {
            context
                .discord
                .edit_rich_embed(destination, post.starboard_message_id, &embed)
                .await?;
            return Ok(Some(ModuleEventOutcome::message(
                "starboard.message_updated",
                event.message_id,
            )));
        }
        if !context
            .services
            .claim_starboard_post(event.guild_id, event.message_id, destination)
            .await?
        {
            return Ok(None);
        }
        let starboard_message_id = context.discord.send_rich_embed(destination, &embed).await?;
        context
            .services
            .complete_starboard_post(
                event.guild_id,
                event.message_id,
                event.channel_id,
                starboard_message_id,
                summary.count,
            )
            .await?;
        Ok(Some(ModuleEventOutcome::message(
            "starboard.message_posted",
            event.message_id,
        )))
    }

    async fn on_reaction_remove(
        &self,
        context: &ModuleContext,
        config: &GuildConfig,
        event: &ReactionAdd,
    ) -> Result<Option<ModuleEventOutcome>> {
        if event.emoji != config.modules.starboard.emoji {
            return Ok(None);
        }
        let Some(post) = context
            .services
            .starboard_post(event.guild_id, event.message_id)
            .await?
        else {
            return Ok(None);
        };
        let summary = context
            .discord
            .message_reaction_summary(event.channel_id, event.message_id, &event.emoji)
            .await?;
        let destination = destination(config)?;
        if summary.count < u64::from(channel_threshold(config, event.channel_id)) {
            context
                .discord
                .delete_message(
                    destination,
                    post.starboard_message_id,
                    "Starboard count fell below the configured threshold",
                )
                .await?;
            context
                .services
                .remove_starboard_post(event.guild_id, event.message_id)
                .await?;
            return Ok(Some(ModuleEventOutcome::message(
                "starboard.message_removed",
                event.message_id,
            )));
        }
        context
            .discord
            .edit_rich_embed(
                destination,
                post.starboard_message_id,
                &starboard_embed(config, event, &summary),
            )
            .await?;
        Ok(Some(ModuleEventOutcome::message(
            "starboard.message_updated",
            event.message_id,
        )))
    }

    async fn on_discord_log_event(
        &self,
        context: &ModuleContext,
        config: &GuildConfig,
        event: &DiscordLogEvent,
    ) -> Result<()> {
        if !matches!(
            event.action.as_str(),
            "discord.message.updated" | "discord.message.deleted"
        ) {
            return Ok(());
        }
        let Some(source_message_id) = event
            .target_id
            .as_deref()
            .and_then(|value| value.parse::<u64>().ok())
        else {
            return Ok(());
        };
        let Some(post) = context
            .services
            .starboard_post(event.guild_id, source_message_id)
            .await?
        else {
            return Ok(());
        };
        if event.action == "discord.message.deleted" {
            context
                .discord
                .delete_message(
                    post.starboard_channel_id,
                    post.starboard_message_id,
                    "Starboard source message was deleted",
                )
                .await?;
            context
                .services
                .remove_starboard_post(event.guild_id, source_message_id)
                .await?;
            return Ok(());
        }

        let summary = context
            .discord
            .message_reaction_summary(
                post.source_channel_id,
                source_message_id,
                &config.modules.starboard.emoji,
            )
            .await?;
        let source = ReactionAdd {
            guild_id: event.guild_id,
            channel_id: post.source_channel_id,
            message_id: source_message_id,
            reactor_id: 0,
            message_author_id: Some(summary.author_id),
            emoji: config.modules.starboard.emoji.clone(),
        };
        context
            .discord
            .edit_rich_embed(
                post.starboard_channel_id,
                post.starboard_message_id,
                &starboard_embed(config, &source, &summary),
            )
            .await?;
        context
            .services
            .complete_starboard_post(
                event.guild_id,
                source_message_id,
                post.source_channel_id,
                post.starboard_message_id,
                summary.count,
            )
            .await?;
        Ok(())
    }
}

fn destination(config: &GuildConfig) -> Result<u64> {
    config
        .modules
        .starboard
        .channel_id
        .as_deref()
        .context("starboard channel is not configured")?
        .parse::<u64>()
        .context("starboard channel is invalid")
}

fn channel_threshold(config: &GuildConfig, channel_id: u64) -> u16 {
    config
        .modules
        .starboard
        .channel_thresholds
        .iter()
        .find(|override_| override_.channel_id == channel_id.to_string())
        .map_or(config.modules.starboard.threshold, |override_| {
            override_.threshold
        })
}

fn starboard_embed(
    config: &GuildConfig,
    event: &ReactionAdd,
    summary: &MessageReactionSummary,
) -> RichEmbed {
    let jump_url = format!(
        "https://discord.com/channels/{}/{}/{}",
        event.guild_id, event.channel_id, event.message_id
    );
    let description = if summary.content.trim().is_empty() {
        "*This message has no text content.*".into()
    } else {
        summary
            .content
            .replace('@', "@\u{200b}")
            .chars()
            .take(3_900)
            .collect()
    };
    RichEmbed {
        title: format!(
            "{} {} star{}",
            config.modules.starboard.emoji,
            summary.count,
            if summary.count == 1 { "" } else { "s" }
        ),
        description,
        url: jump_url,
        color: 0xfee75c,
        author_name: summary.author_name.clone(),
        author_url: format!("https://discord.com/users/{}", summary.author_id),
        image_url: summary.attachment_url.clone().unwrap_or_default(),
        footer_text: format!(
            "Source: #{} · message {}",
            event.channel_id, event.message_id
        ),
        timestamp: true,
        ..RichEmbed::default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bot_discord::MessageReactionSummary;
    use bot_module_api::StarboardPostResult;
    use bot_test_support::{DiscordCall, FakeDiscord, FakeModuleServices, ServiceCall};
    use std::sync::Arc;

    #[tokio::test]
    async fn posts_a_rich_source_embed_and_records_destination() {
        let discord = FakeDiscord::new();
        discord.set_next_message_id(99);
        discord.set_message_reaction_summary(MessageReactionSummary {
            author_id: 50,
            author_name: "luna".into(),
            content: "A useful message".into(),
            attachment_url: Some("https://cdn.example/image.png".into()),
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
        assert!(services.calls().iter().any(|call| matches!(
            call,
            ServiceCall::CompleteStarboardPost {
                source_message_id: 30,
                starboard_message_id: 99,
                reaction_count: 3,
                ..
            }
        )));
        assert!(discord.calls().iter().any(|call| matches!(call, DiscordCall::SendRichEmbed { channel_id: 90, embed } if embed.title == "⭐ 3 stars" && embed.description == "A useful message" && embed.image_url == "https://cdn.example/image.png")));
    }

    #[tokio::test]
    async fn removes_the_destination_when_count_falls_below_channel_threshold() {
        let discord = FakeDiscord::new();
        discord.set_message_reaction_summary(MessageReactionSummary {
            author_id: 50,
            author_name: "luna".into(),
            content: "A useful message".into(),
            attachment_url: None,
            count: 2,
        });
        let services = FakeModuleServices::default();
        services.set_starboard_post(
            10,
            30,
            StarboardPostResult {
                source_channel_id: 20,
                starboard_channel_id: 90,
                starboard_message_id: 99,
                reaction_count: 3,
            },
        );
        let context = ModuleContext {
            discord: Arc::new(discord.clone()),
            services: Arc::new(services.clone()),
        };
        let mut config = GuildConfig::for_guild("10");
        config.modules.starboard.enabled = true;
        config.modules.starboard.channel_id = Some("90".into());
        config.modules.starboard.channel_thresholds = vec![bot_core::StarboardChannelThreshold {
            channel_id: "20".into(),
            threshold: 3,
        }];
        let event = ReactionAdd {
            guild_id: 10,
            channel_id: 20,
            message_id: 30,
            reactor_id: 40,
            message_author_id: Some(50),
            emoji: "⭐".into(),
        };
        let outcome = StarboardModule
            .on_reaction_remove(&context, &config, &event)
            .await
            .unwrap()
            .unwrap();
        assert_eq!(outcome.action, "starboard.message_removed");
        assert!(discord.calls().iter().any(|call| matches!(
            call,
            DiscordCall::DeleteMessage {
                channel_id: 90,
                message_id: 99,
                ..
            }
        )));
        assert!(services.calls().iter().any(|call| matches!(
            call,
            ServiceCall::RemoveStarboardPost {
                guild_id: 10,
                source_message_id: 30,
            }
        )));
    }

    #[tokio::test]
    async fn deleting_a_source_removes_its_starboard_copy() {
        let discord = FakeDiscord::new();
        let services = FakeModuleServices::default();
        services.set_starboard_post(
            10,
            30,
            StarboardPostResult {
                source_channel_id: 20,
                starboard_channel_id: 90,
                starboard_message_id: 99,
                reaction_count: 3,
            },
        );
        let context = ModuleContext {
            discord: Arc::new(discord.clone()),
            services: Arc::new(services.clone()),
        };
        let mut config = GuildConfig::for_guild("10");
        config.modules.starboard.enabled = true;
        config.modules.starboard.channel_id = Some("90".into());
        StarboardModule
            .on_discord_log_event(
                &context,
                &config,
                &DiscordLogEvent {
                    guild_id: 10,
                    category: "messages".into(),
                    action: "discord.message.deleted".into(),
                    title: "Message deleted".into(),
                    summary: String::new(),
                    actor_id: None,
                    target_type: "message".into(),
                    target_id: Some("30".into()),
                    channel_id: Some(20),
                    details: vec![],
                },
            )
            .await
            .unwrap();
        assert!(discord.calls().iter().any(|call| matches!(
            call,
            DiscordCall::DeleteMessage {
                channel_id: 90,
                message_id: 99,
                ..
            }
        )));
        assert!(services.calls().iter().any(|call| matches!(
            call,
            ServiceCall::RemoveStarboardPost {
                guild_id: 10,
                source_message_id: 30,
            }
        )));
    }
}

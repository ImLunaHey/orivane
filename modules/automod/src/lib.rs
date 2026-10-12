use anyhow::{Context, Result};
use async_trait::async_trait;
use bot_core::GuildConfig;
use bot_module_api::{BotModule, MessageCreate, ModuleContext, ModuleEventOutcome};

pub struct AutomodModule;

#[async_trait]
impl BotModule for AutomodModule {
    fn name(&self) -> &'static str {
        "automod"
    }
    fn enabled(&self, config: &GuildConfig) -> bool {
        config.modules.automod.enabled
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
        if event.author_is_bot {
            return Ok(None);
        }
        let settings = &config.modules.automod;
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

        let Some(rule) = matched_rule(settings, event) else {
            return Ok(None);
        };
        let reason = format!("Orivane AutoMod: {rule}");
        if settings.action != "warn" {
            context
                .discord
                .delete_message(event.channel_id, event.message_id, &reason)
                .await?;
        }
        match settings.action.as_str() {
            "delete" | "warn" => {}
            "timeout" => {
                context
                    .discord
                    .timeout_member(
                        event.guild_id,
                        event.author_id,
                        u64::from(settings.timeout_seconds),
                        &reason,
                    )
                    .await?;
            }
            "ban" => {
                context
                    .discord
                    .ban_member(event.guild_id, event.author_id, 0, &reason)
                    .await?;
            }
            _ => anyhow::bail!("unsupported automod action"),
        }
        if !settings.response.trim().is_empty() {
            let response = settings
                .response
                .replace("{user}", &format!("<@{}>", event.author_id))
                .replace("{rule}", rule);
            context
                .discord
                .send_message(event.channel_id, &response)
                .await?;
        }
        if let Some(channel_id) = settings.log_channel_id.as_deref() {
            context
                .discord
                .send_message(
                    channel_id
                        .parse()
                        .context("automod log channel is invalid")?,
                    &format!(
                        "AutoMod `{}` matched for <@{}> in <#{}>; action `{}`.",
                        rule, event.author_id, event.channel_id, settings.action
                    ),
                )
                .await?;
        }
        Ok(Some(ModuleEventOutcome::message(
            format!("automod.{rule}.{}", settings.action),
            event.message_id,
        )))
    }
}

fn matched_rule(settings: &bot_core::AutomodConfig, event: &MessageCreate) -> Option<&'static str> {
    let normalized = event.content.to_lowercase();
    if settings
        .blocked_words
        .iter()
        .any(|term| !term.is_empty() && normalized.contains(&term.to_lowercase()))
    {
        return Some("blocked_term");
    }
    if event.mention_count > usize::from(settings.max_mentions) {
        return Some("mention_spam");
    }
    if settings.block_invites
        && (normalized.contains("discord.gg/") || normalized.contains("discord.com/invite/"))
    {
        return Some("invite_link");
    }
    if settings.block_links && (normalized.contains("http://") || normalized.contains("https://")) {
        return Some("external_link");
    }
    if event.content.lines().count() > usize::from(settings.max_lines) {
        return Some("excessive_lines");
    }
    let letters = event
        .content
        .chars()
        .filter(|character| character.is_alphabetic());
    let (uppercase, total) = letters.fold((0_u32, 0_u32), |(uppercase, total), character| {
        (uppercase + u32::from(character.is_uppercase()), total + 1)
    });
    if total >= 8 && uppercase.saturating_mul(100) / total > u32::from(settings.max_caps_percentage)
    {
        return Some("excessive_caps");
    }
    None
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use bot_test_support::{DiscordCall, FakeDiscord, FakeModuleServices};

    use super::*;

    #[tokio::test]
    async fn deletes_times_out_responds_and_logs_without_retaining_content() {
        let discord = FakeDiscord::new();
        let context = ModuleContext {
            discord: Arc::new(discord.clone()),
            services: Arc::new(FakeModuleServices::default()),
        };
        let mut config = GuildConfig::for_guild("10");
        config.modules.automod.enabled = true;
        config.modules.automod.blocked_words = vec!["secretword".to_owned()];
        config.modules.automod.action = "timeout".to_owned();
        config.modules.automod.timeout_seconds = 300;
        config.modules.automod.log_channel_id = Some("99".to_owned());
        let event = MessageCreate {
            guild_id: 10,
            channel_id: 20,
            message_id: 30,
            author_id: 40,
            author_is_bot: false,
            role_ids: vec![],
            mention_count: 0,
            content: "contains SECRETWORD".to_owned(),
        };
        let outcome = AutomodModule
            .on_message_create(&context, &config, &event)
            .await
            .unwrap();
        assert_eq!(
            outcome,
            Some(ModuleEventOutcome::message(
                "automod.blocked_term.timeout",
                30
            ))
        );
        assert_eq!(
            discord.calls(),
            vec![
                DiscordCall::DeleteMessage {
                    channel_id: 20,
                    message_id: 30,
                    reason: "Orivane AutoMod: blocked_term".to_owned(),
                },
                DiscordCall::TimeoutMember {
                    guild_id: 10,
                    user_id: 40,
                    duration_seconds: 300,
                    reason: "Orivane AutoMod: blocked_term".to_owned(),
                },
                DiscordCall::SendMessage {
                    channel_id: 20,
                    content: "<@40>, that message was removed by AutoMod (blocked_term)."
                        .to_owned(),
                },
                DiscordCall::SendMessage {
                    channel_id: 99,
                    content: "AutoMod `blocked_term` matched for <@40> in <#20>; action `timeout`."
                        .to_owned(),
                },
            ]
        );
        assert!(!format!("{:?}", discord.calls()).contains("SECRETWORD"));
    }

    #[tokio::test]
    async fn ignored_channels_have_no_side_effects() {
        let discord = FakeDiscord::new();
        let context = ModuleContext {
            discord: Arc::new(discord.clone()),
            services: Arc::new(FakeModuleServices::default()),
        };
        let mut config = GuildConfig::for_guild("10");
        config.modules.automod.ignored_channel_ids = vec!["20".to_owned()];
        let event = MessageCreate {
            guild_id: 10,
            channel_id: 20,
            message_id: 30,
            author_id: 40,
            author_is_bot: false,
            role_ids: vec![],
            mention_count: 99,
            content: "spam".to_owned(),
        };
        assert_eq!(
            AutomodModule
                .on_message_create(&context, &config, &event)
                .await
                .unwrap(),
            None
        );
        assert!(discord.calls().is_empty());
    }
}

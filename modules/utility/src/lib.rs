use anyhow::Result;
use async_trait::async_trait;
use bot_core::GuildConfig;
use bot_discord::{
    CommandDefinition, CommandInteraction, CommandOptionDefinition, CommandOptionKind,
    CommandResponse, RichEmbed, RichEmbedField,
};
use bot_module_api::{BotModule, ModuleContext};

pub struct UtilityModule;

const OPTIONAL_USER: &[CommandOptionDefinition] = &[CommandOptionDefinition {
    name: "user",
    description: "Member to inspect; defaults to you",
    kind: CommandOptionKind::User,
    required: false,
    min_value: None,
    max_value: None,
    min_length: None,
    max_length: None,
}];

pub const COMMANDS: &[CommandDefinition] = &[
    CommandDefinition {
        name: "ping",
        description: "Check whether the bot is responding",
        guild_only: true,
        default_permissions: None,
        options: &[],
    },
    CommandDefinition {
        name: "userinfo",
        description: "Show details about a Discord user",
        guild_only: true,
        default_permissions: None,
        options: OPTIONAL_USER,
    },
    CommandDefinition {
        name: "avatar",
        description: "Show a Discord user's avatar",
        guild_only: true,
        default_permissions: None,
        options: OPTIONAL_USER,
    },
    CommandDefinition {
        name: "help",
        description: "Show the commands enabled in this server",
        guild_only: true,
        default_permissions: None,
        options: &[],
    },
    CommandDefinition {
        name: "serverinfo",
        description: "Show this server's bot configuration summary",
        guild_only: true,
        default_permissions: None,
        options: &[],
    },
];

#[async_trait]
impl BotModule for UtilityModule {
    fn name(&self) -> &'static str {
        "utility"
    }
    fn enabled(&self, config: &GuildConfig) -> bool {
        config.modules.utility.enabled
    }
    fn commands(&self) -> &'static [CommandDefinition] {
        COMMANDS
    }

    async fn on_command(
        &self,
        context: &ModuleContext,
        config: &GuildConfig,
        interaction: &CommandInteraction,
    ) -> Result<()> {
        let enabled_count = [
            config.modules.automod.enabled,
            config.modules.leveling.enabled,
            config.modules.welcome.enabled,
            config.modules.logging.enabled,
            config.modules.utility.enabled,
            config.modules.moderation.enabled,
            config.modules.autorole.enabled,
            config.modules.starboard.enabled,
            config.modules.tickets.enabled,
            config.modules.custom_commands.enabled,
            config.modules.reaction_roles.enabled,
            config.modules.autoresponder.enabled,
            config.modules.anti_raid.enabled,
            config.modules.scheduled_messages.enabled,
            config.modules.forms.enabled,
            config.modules.suggestions.enabled,
            config.modules.giveaways.enabled,
            config.modules.reminders.enabled,
            config.modules.auto_delete.enabled,
            config.modules.managed_embeds.enabled,
            config.modules.temporary_voice.enabled,
            config.modules.economy.enabled,
            config.modules.verification.enabled,
            config.modules.moderation_appeals.enabled,
            config.modules.invite_tracking.enabled,
            config.modules.polls.enabled,
            config.modules.counters.enabled,
            config.modules.afk.enabled,
            config.modules.voice_text_linking.enabled,
        ]
        .into_iter()
        .filter(|enabled| *enabled)
        .count();
        if matches!(interaction.command_name.as_str(), "userinfo" | "avatar") {
            let user_id = interaction.user("user").unwrap_or(interaction.actor_id);
            let profile = context.discord.user_profile(user_id).await?;
            let created_unix = ((user_id >> 22) + 1_420_070_400_000) / 1_000;
            let embed = if interaction.command_name == "avatar" {
                RichEmbed {
                    title: format!("{}'s avatar", profile.display_name),
                    url: profile.avatar_url.clone(),
                    image_url: profile.avatar_url,
                    color: 0x5865f2,
                    footer_text: format!("User ID: {}", profile.id),
                    ..RichEmbed::default()
                }
            } else {
                RichEmbed {
                    title: profile.display_name.clone(),
                    description: format!("<@{}> · @{}", profile.id, profile.username),
                    color: 0x5865f2,
                    thumbnail_url: profile.avatar_url,
                    footer_text: "Discord user information".into(),
                    timestamp: true,
                    fields: vec![
                        RichEmbedField {
                            name: "User ID".into(),
                            value: profile.id.to_string(),
                            inline: true,
                        },
                        RichEmbedField {
                            name: "Account type".into(),
                            value: if profile.bot {
                                "Bot".into()
                            } else {
                                "User".into()
                            },
                            inline: true,
                        },
                        RichEmbedField {
                            name: "Created".into(),
                            value: format!("<t:{created_unix}:F>\n<t:{created_unix}:R>"),
                            inline: false,
                        },
                    ],
                    ..RichEmbed::default()
                }
            };
            let mut response = CommandResponse::public_embed(embed);
            response.ephemeral = config.modules.utility.ephemeral_responses;
            context
                .discord
                .respond_to_command(interaction, response)
                .await?;
            return Ok(());
        }
        let content = match interaction.command_name.as_str() {
            "ping" => "Pong!".to_owned(),
            "help" => "Available utility commands: `/ping`, `/help`, `/serverinfo`, `/userinfo`, and `/avatar`. Other commands appear when their modules are enabled.".to_owned(),
            "serverinfo" => {
                let metrics = context.discord.guild_metrics(interaction.guild_id).await.ok();
                metrics.map_or_else(
                    || format!("Server `{}` has {enabled_count} Orivane modules enabled.", interaction.guild_id),
                    |metrics| format!("Server `{}` · {} members · {} online · {} boosts · {enabled_count} Orivane modules enabled.", interaction.guild_id, metrics.members, metrics.online, metrics.boosts),
                )
            }
            _ => return Ok(()),
        };
        let response = if config.modules.utility.ephemeral_responses {
            CommandResponse::ephemeral(content)
        } else {
            CommandResponse::public(content)
        };
        context
            .discord
            .respond_to_command(interaction, response)
            .await?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bot_discord::{CommandValue, UserProfile};
    use bot_test_support::{DiscordCall, FakeDiscord, FakeModuleServices};
    use std::collections::BTreeMap;
    use std::sync::Arc;

    #[tokio::test]
    async fn ping_uses_the_server_response_setting() {
        let discord = FakeDiscord::new();
        let context = ModuleContext {
            discord: Arc::new(discord.clone()),
            services: Arc::new(FakeModuleServices::default()),
        };
        let mut config = GuildConfig::for_guild("200");
        config.modules.utility.ephemeral_responses = true;
        let interaction = CommandInteraction {
            id: 100,
            token: "test".into(),
            guild_id: 200,
            channel_id: Some(300),
            actor_id: 400,
            actor_role_ids: vec![],
            actor_permissions: u64::MAX,
            command_name: "ping".into(),
            options: Default::default(),
        };
        UtilityModule
            .on_command(&context, &config, &interaction)
            .await
            .unwrap();
        assert_eq!(
            discord.calls(),
            vec![DiscordCall::RespondToCommand {
                interaction_id: 100,
                response: CommandResponse::ephemeral("Pong!")
            }]
        );
    }

    #[tokio::test]
    async fn userinfo_uses_resolved_profiles_and_account_age() {
        let discord = FakeDiscord::new();
        discord.set_user_profile(UserProfile {
            id: 500,
            username: "imlunahey".into(),
            display_name: "luna".into(),
            avatar_url: "https://cdn.example/avatar.png".into(),
            bot: false,
        });
        let context = ModuleContext {
            discord: Arc::new(discord.clone()),
            services: Arc::new(FakeModuleServices::default()),
        };
        let interaction = CommandInteraction {
            id: 100,
            token: "test".into(),
            guild_id: 200,
            channel_id: Some(300),
            actor_id: 400,
            actor_role_ids: vec![],
            actor_permissions: 0,
            command_name: "userinfo".into(),
            options: BTreeMap::from([("user".into(), CommandValue::User(500))]),
        };
        UtilityModule
            .on_command(&context, &GuildConfig::for_guild("200"), &interaction)
            .await
            .unwrap();
        assert!(discord.calls().iter().any(|call| matches!(
            call,
            DiscordCall::RespondToCommand { response, .. }
                if response.embed.as_ref().is_some_and(|embed|
                    embed.description.contains("@imlunahey")
                        && embed.thumbnail_url == "https://cdn.example/avatar.png")
        )));
    }
}

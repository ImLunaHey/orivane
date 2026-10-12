use anyhow::Result;
use async_trait::async_trait;
use bot_core::GuildConfig;
use bot_discord::{CommandDefinition, CommandInteraction, CommandResponse};
use bot_module_api::{BotModule, ModuleContext};

pub struct UtilityModule;

pub const COMMANDS: &[CommandDefinition] = &[
    CommandDefinition {
        name: "ping",
        description: "Check whether the bot is responding",
        guild_only: true,
        default_permissions: None,
        options: &[],
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
        ]
        .into_iter()
        .filter(|enabled| *enabled)
        .count();
        let content = match interaction.command_name.as_str() {
            "ping" => "Pong!".to_owned(),
            "help" => "Available utility commands: `/ping`, `/help`, `/serverinfo`. Other commands appear when their modules are enabled.".to_owned(),
            "serverinfo" => format!("Server `{}` has {enabled_count} bot modules enabled.", interaction.guild_id),
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
    use bot_test_support::{DiscordCall, FakeDiscord, FakeModuleServices};
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
}

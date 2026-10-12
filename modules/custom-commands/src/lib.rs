use anyhow::{Context, Result};
use async_trait::async_trait;
use bot_core::GuildConfig;
use bot_discord::{
    CommandDefinition, CommandInteraction, CommandOptionDefinition, CommandOptionKind,
    CommandResponse,
};
use bot_module_api::{BotModule, ModuleContext};

pub struct CustomCommandsModule;

pub const COMMANDS: &[CommandDefinition] = &[CommandDefinition {
    name: "custom",
    description: "Run a server-defined custom response",
    guild_only: true,
    default_permissions: None,
    options: &[CommandOptionDefinition {
        name: "name",
        description: "Name of the custom command",
        kind: CommandOptionKind::String,
        required: true,
        min_value: None,
        max_value: None,
        min_length: Some(1),
        max_length: Some(32),
    }],
}];

#[async_trait]
impl BotModule for CustomCommandsModule {
    fn name(&self) -> &'static str {
        "custom_commands"
    }
    fn enabled(&self, config: &GuildConfig) -> bool {
        config.modules.custom_commands.enabled
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
        if interaction.command_name != "custom" {
            return Ok(());
        }
        let name = interaction
            .string("name")
            .context("custom command name is required")?;
        let Some(command) = config
            .modules
            .custom_commands
            .commands
            .iter()
            .find(|command| command.enabled && command.name.eq_ignore_ascii_case(name))
        else {
            context
                .discord
                .respond_to_command(
                    interaction,
                    CommandResponse::ephemeral(format!(
                        "No enabled custom command named {name} exists in this server."
                    )),
                )
                .await?;
            return Ok(());
        };
        if !command.allowed_channel_ids.is_empty()
            && interaction.channel_id.is_none_or(|channel_id| {
                !command
                    .allowed_channel_ids
                    .iter()
                    .any(|id| id == &channel_id.to_string())
            })
        {
            context
                .discord
                .respond_to_command(
                    interaction,
                    CommandResponse::ephemeral(
                        "That custom command is not available in this channel.",
                    ),
                )
                .await?;
            return Ok(());
        }
        let response = command
            .response
            .replace("{user}", &format!("<@{}>", interaction.actor_id))
            .replace("{user_id}", &interaction.actor_id.to_string())
            .replace(
                "{channel}",
                &interaction
                    .channel_id
                    .map(|id| format!("<#{id}>"))
                    .unwrap_or_else(|| "this channel".to_owned()),
            )
            .replace("{server_id}", &interaction.guild_id.to_string());
        context
            .discord
            .respond_to_command(
                interaction,
                if command.ephemeral {
                    CommandResponse::ephemeral(response)
                } else {
                    CommandResponse::public(response)
                },
            )
            .await?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::{collections::BTreeMap, sync::Arc};

    use bot_core::CustomCommandConfig;
    use bot_discord::CommandValue;
    use bot_test_support::{DiscordCall, FakeDiscord, FakeModuleServices};

    use super::*;

    #[tokio::test]
    async fn renders_variables_for_an_enabled_custom_command() {
        let discord = FakeDiscord::new();
        let context = ModuleContext {
            discord: Arc::new(discord.clone()),
            services: Arc::new(FakeModuleServices::default()),
        };
        let mut config = GuildConfig::for_guild("20");
        config.modules.custom_commands.enabled = true;
        config
            .modules
            .custom_commands
            .commands
            .push(CustomCommandConfig {
                enabled: true,
                name: "hello".to_owned(),
                description: "Say hello".to_owned(),
                response: "Hello {user} in {channel}".to_owned(),
                ephemeral: false,
                allowed_channel_ids: vec!["30".to_owned()],
            });
        let interaction = CommandInteraction {
            id: 10,
            token: "token".to_owned(),
            guild_id: 20,
            channel_id: Some(30),
            actor_id: 40,
            actor_role_ids: vec![],
            command_name: "custom".to_owned(),
            options: BTreeMap::from([(
                "name".to_owned(),
                CommandValue::String("hello".to_owned()),
            )]),
        };
        CustomCommandsModule
            .on_command(&context, &config, &interaction)
            .await
            .unwrap();
        assert_eq!(
            discord.calls(),
            vec![DiscordCall::RespondToCommand {
                interaction_id: 10,
                response: CommandResponse::public("Hello <@40> in <#30>"),
            }]
        );
    }
}

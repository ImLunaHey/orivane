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
        if !command.allowed_role_ids.is_empty()
            && !interaction.actor_role_ids.iter().any(|role_id| {
                command
                    .allowed_role_ids
                    .iter()
                    .any(|allowed| allowed == &role_id.to_string())
            })
        {
            context
                .discord
                .respond_to_command(
                    interaction,
                    CommandResponse::ephemeral(
                        "You do not have a role allowed to run that custom command.",
                    ),
                )
                .await?;
            return Ok(());
        }
        if !context
            .services
            .claim_module_cooldown(
                "custom-commands",
                &command.name,
                interaction.guild_id,
                interaction.actor_id,
                command.cooldown_seconds,
            )
            .await?
        {
            context
                .discord
                .respond_to_command(
                    interaction,
                    CommandResponse::ephemeral(format!(
                        "That command is on cooldown. Try again in up to {} seconds.",
                        command.cooldown_seconds
                    )),
                )
                .await?;
            return Ok(());
        }
        let responses = std::iter::once(&command.response)
            .chain(
                command
                    .additional_responses
                    .iter()
                    .filter(|response| !response.trim().is_empty()),
            )
            .collect::<Vec<_>>();
        let response = responses[(interaction.id as usize) % responses.len()]
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
                additional_responses: vec![],
                ephemeral: false,
                allowed_channel_ids: vec!["30".to_owned()],
                allowed_role_ids: vec![],
                cooldown_seconds: 0,
            });
        let interaction = CommandInteraction {
            id: 10,
            token: "token".to_owned(),
            guild_id: 20,
            channel_id: Some(30),
            actor_id: 40,
            actor_role_ids: vec![],
            actor_permissions: u64::MAX,
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

    #[tokio::test]
    async fn enforces_roles_and_selects_from_multiple_responses() {
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
                name: "answer".into(),
                description: String::new(),
                response: "First".into(),
                additional_responses: vec!["Second".into()],
                ephemeral: false,
                allowed_channel_ids: vec![],
                allowed_role_ids: vec!["55".into()],
                cooldown_seconds: 10,
            });
        let interaction = CommandInteraction {
            id: 11,
            token: "token".into(),
            guild_id: 20,
            channel_id: Some(30),
            actor_id: 40,
            actor_role_ids: vec![55],
            actor_permissions: 0,
            command_name: "custom".into(),
            options: BTreeMap::from([("name".into(), CommandValue::String("answer".into()))]),
        };
        CustomCommandsModule
            .on_command(&context, &config, &interaction)
            .await
            .unwrap();
        assert!(discord.calls().iter().any(|call| matches!(
            call,
            DiscordCall::RespondToCommand { response, .. } if response.content == "Second"
        )));
    }
}

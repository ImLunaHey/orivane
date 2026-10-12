use anyhow::{Context, Result, bail};
use async_trait::async_trait;
use bot_core::{EmbedPayload, GuildConfig};
use bot_discord::{
    CommandDefinition, CommandInteraction, CommandOptionDefinition, CommandOptionKind,
    CommandResponse, RichEmbed, RichEmbedField,
};
use bot_module_api::{BotModule, ManagedEmbedDeliveryInput, ModuleContext};

pub struct ManagedEmbedsModule;
const MANAGE_MESSAGES: u64 = 1 << 13;

pub const COMMANDS: &[CommandDefinition] = &[CommandDefinition {
    name: "embed-send",
    description: "Send a saved embed template",
    guild_only: true,
    default_permissions: Some(MANAGE_MESSAGES),
    options: &[
        CommandOptionDefinition {
            name: "template",
            description: "Dashboard template ID",
            kind: CommandOptionKind::Integer,
            required: true,
            min_value: Some(1),
            max_value: None,
            min_length: None,
            max_length: None,
        },
        CommandOptionDefinition {
            name: "channel",
            description: "Destination channel (defaults to this channel)",
            kind: CommandOptionKind::Channel,
            required: false,
            min_value: None,
            max_value: None,
            min_length: None,
            max_length: None,
        },
    ],
}];

#[async_trait]
impl BotModule for ManagedEmbedsModule {
    fn name(&self) -> &'static str {
        "managed_embeds"
    }
    fn enabled(&self, config: &GuildConfig) -> bool {
        config.modules.managed_embeds.enabled
    }
    fn commands(&self) -> &'static [CommandDefinition] {
        COMMANDS
    }

    async fn on_command(
        &self,
        context: &ModuleContext,
        _config: &GuildConfig,
        interaction: &CommandInteraction,
    ) -> Result<()> {
        if interaction.command_name != "embed-send" {
            return Ok(());
        }
        if interaction.actor_permissions & MANAGE_MESSAGES == 0 {
            bail!("Manage Messages permission is required");
        }
        let template_id = interaction
            .integer("template")
            .context("template is required")?;
        let channel_id = interaction
            .channel("channel")
            .or(interaction.channel_id)
            .context("channel is required")?;
        let template = context
            .services
            .managed_embed(interaction.guild_id, template_id)
            .await?
            .context("embed template not found")?;
        let server_name = context
            .services
            .guild_name(interaction.guild_id)
            .await?
            .unwrap_or_else(|| "this server".into());
        let embed = rich_embed(
            &template.payload,
            interaction.guild_id,
            &server_name,
            interaction.actor_id,
        );
        match context.discord.send_rich_embed(channel_id, &embed).await {
            Ok(message_id) => {
                context
                    .services
                    .record_managed_embed_delivery(ManagedEmbedDeliveryInput {
                        guild_id: interaction.guild_id,
                        template_id,
                        channel_id,
                        message_id: Some(message_id),
                        actor_id: interaction.actor_id,
                        status: "delivered".into(),
                        error: None,
                    })
                    .await?;
                context
                    .discord
                    .respond_to_command(
                        interaction,
                        CommandResponse::ephemeral(format!(
                            "Sent **{}** to <#{channel_id}>.",
                            template.name
                        )),
                    )
                    .await?;
                Ok(())
            }
            Err(error) => {
                context
                    .services
                    .record_managed_embed_delivery(ManagedEmbedDeliveryInput {
                        guild_id: interaction.guild_id,
                        template_id,
                        channel_id,
                        message_id: None,
                        actor_id: interaction.actor_id,
                        status: "failed".into(),
                        error: Some(error.to_string()),
                    })
                    .await?;
                Err(error.into())
            }
        }
    }
}

pub fn rich_embed(
    value: &EmbedPayload,
    guild_id: u64,
    guild_name: &str,
    actor_id: u64,
) -> RichEmbed {
    let render = |text: &str| {
        text.replace("{user}", &format!("<@{actor_id}>"))
            .replace("{display_name}", &format!("<@{actor_id}>"))
            .replace("{username}", &format!("<@{actor_id}>"))
            .replace("{user_id}", &actor_id.to_string())
            .replace("{server}", guild_name)
            .replace("{server_id}", &guild_id.to_string())
    };
    RichEmbed {
        content: render(&value.content),
        title: render(&value.title),
        description: render(&value.description),
        url: value.url.clone(),
        color: value.color,
        author_name: render(&value.author_name),
        author_url: value.author_url.clone(),
        author_icon_url: value.author_icon_url.clone(),
        footer_text: render(&value.footer_text),
        footer_icon_url: value.footer_icon_url.clone(),
        thumbnail_url: value.thumbnail_url.clone(),
        image_url: value.image_url.clone(),
        timestamp: value.timestamp,
        fields: value
            .fields
            .iter()
            .map(|field| RichEmbedField {
                name: render(&field.name),
                value: render(&field.value),
                inline: field.inline,
            })
            .collect(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bot_discord::CommandValue;
    use bot_test_support::{DiscordCall, FakeDiscord, FakeModuleServices};
    use std::{collections::BTreeMap, sync::Arc};

    #[tokio::test]
    async fn sends_a_saved_template() {
        let discord = FakeDiscord::new();
        let context = ModuleContext {
            discord: Arc::new(discord.clone()),
            services: Arc::new(FakeModuleServices::default()),
        };
        let interaction = CommandInteraction {
            id: 1,
            token: "x".into(),
            guild_id: 10,
            channel_id: Some(20),
            actor_id: 30,
            actor_role_ids: vec![],
            actor_permissions: MANAGE_MESSAGES,
            command_name: "embed-send".into(),
            options: BTreeMap::from([("template".into(), CommandValue::Integer(4))]),
        };
        ManagedEmbedsModule
            .on_command(&context, &GuildConfig::for_guild("10"), &interaction)
            .await
            .unwrap();
        assert!(
            matches!(&discord.calls()[0], DiscordCall::SendRichEmbed { channel_id:20, embed } if embed.title=="Welcome <@30> to this server")
        );
    }
}

use anyhow::{Context, Result};
use async_trait::async_trait;
use bot_core::GuildConfig;
use bot_discord::{
    CommandDefinition, CommandInteraction, CommandOptionDefinition, CommandOptionKind,
    CommandResponse,
};
use bot_module_api::{BotModule, ModuleContext, TicketRecordInput};

pub struct TicketsModule;

const REASON_OPTION: &[CommandOptionDefinition] = &[CommandOptionDefinition {
    name: "reason",
    description: "What you need help with or why the ticket is closing",
    kind: CommandOptionKind::String,
    required: false,
    min_value: None,
    max_value: None,
    min_length: Some(1),
    max_length: Some(500),
}];

pub const COMMANDS: &[CommandDefinition] = &[
    CommandDefinition {
        name: "ticket",
        description: "Open a private support ticket",
        guild_only: true,
        default_permissions: None,
        options: REASON_OPTION,
    },
    CommandDefinition {
        name: "close",
        description: "Close the current support ticket",
        guild_only: true,
        default_permissions: None,
        options: REASON_OPTION,
    },
];

#[async_trait]
impl BotModule for TicketsModule {
    fn name(&self) -> &'static str {
        "tickets"
    }

    fn enabled(&self, config: &GuildConfig) -> bool {
        config.modules.tickets.enabled
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
        match interaction.command_name.as_str() {
            "ticket" => open_ticket(context, config, interaction).await,
            "close" => close_ticket(context, config, interaction).await,
            _ => Ok(()),
        }
    }
}

async fn open_ticket(
    context: &ModuleContext,
    config: &GuildConfig,
    interaction: &CommandInteraction,
) -> Result<()> {
    if let Some(channel_id) = context
        .services
        .open_ticket_channel(interaction.guild_id, interaction.actor_id)
        .await?
    {
        context
            .discord
            .respond_to_command(
                interaction,
                CommandResponse::ephemeral(format!(
                    "You already have an open ticket: <#{channel_id}>."
                )),
            )
            .await?;
        return Ok(());
    }

    let settings = &config.modules.tickets;
    let category_id = settings
        .category_id
        .as_deref()
        .context("ticket category is not configured")?
        .parse::<u64>()
        .context("ticket category is invalid")?;
    let support_role_ids = settings
        .support_role_ids
        .iter()
        .map(|id| id.parse::<u64>().context("ticket support role is invalid"))
        .collect::<Result<Vec<_>>>()?;
    let channel_name = settings
        .channel_name
        .replace("{user_id}", &interaction.actor_id.to_string())
        .chars()
        .take(100)
        .collect::<String>();
    let channel_id = context
        .discord
        .create_ticket_channel(
            interaction.guild_id,
            category_id,
            interaction.actor_id,
            &support_role_ids,
            &channel_name,
            "Orivane support ticket opened",
        )
        .await?;

    if let Err(error) = context
        .services
        .record_open_ticket(TicketRecordInput {
            guild_id: interaction.guild_id,
            opener_id: interaction.actor_id,
            channel_id,
        })
        .await
    {
        let _ = context
            .discord
            .delete_channel(channel_id, "Ticket persistence failed; rolling back")
            .await;
        return Err(error);
    }

    let roles = support_role_ids
        .iter()
        .map(|role_id| format!("<@&{role_id}>"))
        .collect::<Vec<_>>()
        .join(" ");
    let mut welcome = settings
        .welcome_message
        .replace("{user}", &format!("<@{}>", interaction.actor_id))
        .replace("{user_id}", &interaction.actor_id.to_string())
        .replace("{support_roles}", &roles);
    if let Some(reason) = interaction
        .string("reason")
        .map(str::trim)
        .filter(|value| !value.is_empty())
    {
        welcome.push_str(&format!("\n\n**Opening note:** {reason}"));
    }
    context.discord.send_message(channel_id, &welcome).await?;
    context
        .discord
        .respond_to_command(
            interaction,
            CommandResponse::ephemeral(format!("Your private ticket is ready: <#{channel_id}>.")),
        )
        .await?;
    Ok(())
}

async fn close_ticket(
    context: &ModuleContext,
    config: &GuildConfig,
    interaction: &CommandInteraction,
) -> Result<()> {
    let channel_id = interaction
        .channel_id
        .context("this command requires a channel")?;
    if !context
        .services
        .close_ticket_channel(interaction.guild_id, channel_id, interaction.actor_id)
        .await?
    {
        context
            .discord
            .respond_to_command(
                interaction,
                CommandResponse::ephemeral("This channel is not an open Orivane ticket."),
            )
            .await?;
        return Ok(());
    }
    let reason = interaction.string("reason").unwrap_or("No reason provided");
    if let Some(transcript_channel_id) = config.modules.tickets.transcript_channel_id.as_deref() {
        context
            .discord
            .send_message(
                transcript_channel_id
                    .parse()
                    .context("transcript channel is invalid")?,
                &format!(
                    "Ticket <#{}> closed by <@{}>. Reason: {}",
                    channel_id, interaction.actor_id, reason
                ),
            )
            .await?;
    }
    context
        .discord
        .respond_to_command(
            interaction,
            CommandResponse::ephemeral("Ticket closed. This channel is being removed."),
        )
        .await?;
    context
        .discord
        .delete_channel(
            channel_id,
            &format!("Ticket closed by {}", interaction.actor_id),
        )
        .await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::{collections::BTreeMap, sync::Arc};

    use bot_discord::CommandValue;
    use bot_test_support::{DiscordCall, FakeDiscord, FakeModuleServices, ServiceCall};

    use super::*;

    #[tokio::test]
    async fn opens_private_channel_records_it_and_sends_welcome() {
        let discord = FakeDiscord::new();
        discord.set_next_channel_id(90);
        let services = FakeModuleServices::default();
        let context = ModuleContext {
            discord: Arc::new(discord.clone()),
            services: Arc::new(services.clone()),
        };
        let mut config = GuildConfig::for_guild("10");
        config.modules.tickets.enabled = true;
        config.modules.tickets.category_id = Some("20".to_owned());
        config.modules.tickets.support_role_ids = vec!["30".to_owned()];
        let command = CommandInteraction {
            id: 1,
            token: "token".to_owned(),
            guild_id: 10,
            channel_id: Some(50),
            actor_id: 40,
            actor_role_ids: vec![],
            command_name: "ticket".to_owned(),
            options: BTreeMap::from([(
                "reason".to_owned(),
                CommandValue::String("Billing help".to_owned()),
            )]),
        };

        TicketsModule
            .on_command(&context, &config, &command)
            .await
            .unwrap();

        assert_eq!(
            services.calls(),
            vec![
                ServiceCall::FindOpenTicket {
                    guild_id: 10,
                    opener_id: 40,
                },
                ServiceCall::RecordOpenTicket(TicketRecordInput {
                    guild_id: 10,
                    opener_id: 40,
                    channel_id: 90,
                }),
            ]
        );
        assert_eq!(
            discord.calls()[0],
            DiscordCall::CreateTicketChannel {
                guild_id: 10,
                category_id: 20,
                opener_id: 40,
                support_role_ids: vec![30],
                name: "ticket-40".to_owned(),
                reason: "Orivane support ticket opened".to_owned(),
            }
        );
        assert!(matches!(
            discord.calls().last(),
            Some(DiscordCall::RespondToCommand { .. })
        ));
    }
}

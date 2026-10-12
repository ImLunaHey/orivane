use anyhow::{Context, Result};
use async_trait::async_trait;
use bot_core::GuildConfig;
use bot_discord::{
    CommandDefinition, CommandInteraction, CommandOptionDefinition, CommandOptionKind,
    CommandResponse, ComponentInteraction, MessageButton, MessageButtonStyle, ModalDefinition,
    ModalFieldDefinition, ModalFieldStyle, ModalSubmitInteraction,
};
use bot_module_api::{
    BotModule, MessageCreate, ModuleContext, ModuleEventOutcome, ScheduledTick, TicketPanelResult,
    TicketRecordInput,
};

pub struct TicketsModule;
const OPEN_PREFIX: &str = "orivane:ticket:open:";
const INTAKE_PREFIX: &str = "orivane:ticket:intake:";
const CLAIM_PREFIX: &str = "orivane:ticket:claim:";
const CLOSE_PREFIX: &str = "orivane:ticket:close:";
const CLOSE_MODAL_PREFIX: &str = "orivane:ticket:close-modal:";

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
    fn handles_component(&self, custom_id: &str) -> bool {
        custom_id.starts_with(OPEN_PREFIX)
            || custom_id.starts_with(CLAIM_PREFIX)
            || custom_id.starts_with(CLOSE_PREFIX)
    }
    fn handles_modal_submit(&self, custom_id: &str) -> bool {
        custom_id.starts_with(INTAKE_PREFIX) || custom_id.starts_with(CLOSE_MODAL_PREFIX)
    }
    fn handles_message_create(&self) -> bool {
        true
    }
    fn handles_scheduled_tick(&self) -> bool {
        true
    }

    async fn on_message_create(
        &self,
        context: &ModuleContext,
        _config: &GuildConfig,
        event: &MessageCreate,
    ) -> Result<Option<ModuleEventOutcome>> {
        if context
            .services
            .touch_ticket_activity(event.guild_id, event.channel_id)
            .await?
        {
            return Ok(Some(ModuleEventOutcome {
                action: "ticket.activity.recorded".into(),
                target_type: "ticket_channel".into(),
                target_id: Some(event.channel_id.to_string()),
            }));
        }
        Ok(None)
    }

    async fn on_scheduled_tick(
        &self,
        context: &ModuleContext,
        config: &GuildConfig,
        _event: &ScheduledTick,
    ) -> Result<Vec<ModuleEventOutcome>> {
        let guild_id = config
            .guild_id
            .parse::<u64>()
            .context("guild ID is invalid")?;
        let tickets = context.services.due_inactive_tickets(guild_id, 25).await?;
        let mut outcomes = Vec::with_capacity(tickets.len());
        for ticket in tickets {
            let transcript = context
                .discord
                .channel_transcript(ticket.channel_id)
                .await
                .unwrap_or_else(|error| format!("Transcript capture failed: {error}"));
            if !context
                .services
                .close_ticket(
                    guild_id,
                    ticket.id,
                    0,
                    "Automatically closed after inactivity",
                    &transcript,
                )
                .await?
            {
                continue;
            }
            context
                .discord
                .set_ticket_member_writable(
                    ticket.channel_id,
                    ticket.opener_id,
                    false,
                    "Ticket automatically closed after inactivity",
                )
                .await?;
            if let Some(channel) = config.modules.tickets.transcript_channel_id.as_deref() {
                context
                    .discord
                    .send_message(
                        channel.parse().context("transcript channel is invalid")?,
                        &format!(
                            "Ticket #{} (<#{}>) automatically closed after inactivity.",
                            ticket.id, ticket.channel_id
                        ),
                    )
                    .await?;
            }
            outcomes.push(ModuleEventOutcome {
                action: "ticket.auto_closed".into(),
                target_type: "ticket".into(),
                target_id: Some(ticket.id.to_string()),
            });
        }
        context.services.purge_expired_ticket_transcripts().await?;
        Ok(outcomes)
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

    async fn on_component(
        &self,
        context: &ModuleContext,
        config: &GuildConfig,
        interaction: &ComponentInteraction,
    ) -> Result<Option<ModuleEventOutcome>> {
        if let Some(id) = interaction.custom_id.strip_prefix(OPEN_PREFIX) {
            let panel_id = id.parse::<i64>().context("invalid ticket panel ID")?;
            let panel = context
                .services
                .ticket_panel(interaction.guild_id, panel_id)
                .await?
                .context("ticket panel no longer exists")?;
            if !panel.enabled {
                context
                    .discord
                    .respond_to_component(
                        interaction,
                        CommandResponse::ephemeral("This ticket panel is disabled."),
                    )
                    .await?;
                return Ok(None);
            }
            if panel.intake_enabled {
                context
                    .discord
                    .open_component_modal(
                        interaction,
                        ModalDefinition {
                            custom_id: format!("{INTAKE_PREFIX}{panel_id}"),
                            title: "Open a support ticket".into(),
                            fields: vec![
                                ModalFieldDefinition {
                                    custom_id: "subject".into(),
                                    label: "Subject".into(),
                                    style: ModalFieldStyle::Short,
                                    required: true,
                                    min_length: 1,
                                    max_length: 100,
                                    placeholder: panel.intake_label,
                                },
                                ModalFieldDefinition {
                                    custom_id: "details".into(),
                                    label: "Details".into(),
                                    style: ModalFieldStyle::Paragraph,
                                    required: false,
                                    min_length: 0,
                                    max_length: 1000,
                                    placeholder: "Add any context that will help the team.".into(),
                                },
                            ],
                        },
                    )
                    .await?;
            } else {
                let channel = open_from_panel(
                    context,
                    interaction.guild_id,
                    interaction.actor_id,
                    &panel,
                    "Support request",
                    "",
                )
                .await?;
                context
                    .discord
                    .respond_to_component(
                        interaction,
                        CommandResponse::ephemeral(format!(
                            "Your private ticket is ready: <#{channel}>."
                        )),
                    )
                    .await?;
            }
            return Ok(Some(ModuleEventOutcome {
                action: "ticket.open.requested".into(),
                target_type: "ticket_panel".into(),
                target_id: Some(panel_id.to_string()),
            }));
        }
        if let Some(parts) = interaction.custom_id.strip_prefix(CLAIM_PREFIX) {
            let mut parts = parts.split(':');
            let panel_id = parts.next().context("missing panel")?.parse::<i64>()?;
            let ticket_id = parts.next().context("missing ticket")?.parse::<i64>()?;
            let panel = context
                .services
                .ticket_panel(interaction.guild_id, panel_id)
                .await?
                .context("ticket panel no longer exists")?;
            if !interaction
                .actor_role_ids
                .iter()
                .any(|r| panel.support_role_ids.contains(r))
            {
                context
                    .discord
                    .respond_to_component(
                        interaction,
                        CommandResponse::ephemeral(
                            "Only a configured support role can claim this ticket.",
                        ),
                    )
                    .await?;
                return Ok(None);
            }
            let claimed = context
                .services
                .claim_ticket(interaction.guild_id, ticket_id, interaction.actor_id)
                .await?;
            context
                .discord
                .respond_to_component(
                    interaction,
                    CommandResponse::ephemeral(if claimed {
                        "Ticket assigned to you."
                    } else {
                        "This ticket is no longer open."
                    }),
                )
                .await?;
            return Ok(Some(ModuleEventOutcome {
                action: "ticket.claimed".into(),
                target_type: "ticket".into(),
                target_id: Some(ticket_id.to_string()),
            }));
        }
        if let Some(id) = interaction.custom_id.strip_prefix(CLOSE_PREFIX) {
            let ticket_id = id.parse::<i64>().context("invalid ticket ID")?;
            let ticket = context
                .services
                .ticket(interaction.guild_id, ticket_id)
                .await?
                .context("ticket no longer exists")?;
            if ticket.status != "open" {
                context
                    .discord
                    .respond_to_component(
                        interaction,
                        CommandResponse::ephemeral("This ticket is already closed."),
                    )
                    .await?;
                return Ok(None);
            }
            if !can_manage_ticket(
                context,
                config,
                interaction.guild_id,
                ticket.panel_id,
                ticket.opener_id,
                interaction.actor_id,
                &interaction.actor_role_ids,
            )
            .await?
            {
                context.discord.respond_to_component(interaction,CommandResponse::ephemeral("Only the ticket opener or a configured support role can close this ticket.")).await?;
                return Ok(None);
            }
            context
                .discord
                .open_component_modal(
                    interaction,
                    ModalDefinition {
                        custom_id: format!("{CLOSE_MODAL_PREFIX}{ticket_id}"),
                        title: "Close support ticket".into(),
                        fields: vec![ModalFieldDefinition {
                            custom_id: "reason".into(),
                            label: "Closure reason".into(),
                            style: ModalFieldStyle::Paragraph,
                            required: true,
                            min_length: 1,
                            max_length: 500,
                            placeholder: "Explain why this ticket is being closed.".into(),
                        }],
                    },
                )
                .await?;
            return Ok(None);
        }
        Ok(None)
    }

    async fn on_modal_submit(
        &self,
        context: &ModuleContext,
        config: &GuildConfig,
        interaction: &ModalSubmitInteraction,
    ) -> Result<Option<ModuleEventOutcome>> {
        if let Some(id) = interaction.custom_id.strip_prefix(CLOSE_MODAL_PREFIX) {
            let ticket_id = id.parse::<i64>().context("invalid ticket ID")?;
            let ticket = context
                .services
                .ticket(interaction.guild_id, ticket_id)
                .await?
                .context("ticket no longer exists")?;
            if !can_manage_ticket(
                context,
                config,
                interaction.guild_id,
                ticket.panel_id,
                ticket.opener_id,
                interaction.actor_id,
                &interaction.actor_role_ids,
            )
            .await?
            {
                anyhow::bail!("member is not allowed to close this ticket")
            }
            let reason = interaction
                .values
                .get("reason")
                .map(String::as_str)
                .unwrap_or("Closed from Discord");
            let transcript = context
                .discord
                .channel_transcript(ticket.channel_id)
                .await?;
            if !context
                .services
                .close_ticket(
                    interaction.guild_id,
                    ticket.id,
                    interaction.actor_id,
                    reason,
                    &transcript,
                )
                .await?
            {
                context
                    .discord
                    .respond_to_modal(
                        interaction,
                        CommandResponse::ephemeral("This ticket is no longer open."),
                    )
                    .await?;
                return Ok(None);
            }
            context
                .discord
                .set_ticket_member_writable(
                    ticket.channel_id,
                    ticket.opener_id,
                    false,
                    &format!("Ticket closed by {}", interaction.actor_id),
                )
                .await?;
            if let Some(channel) = config.modules.tickets.transcript_channel_id.as_deref() {
                context
                    .discord
                    .send_message(
                        channel.parse().context("transcript channel is invalid")?,
                        &format!(
                            "Ticket #{} (<#{}>) closed by <@{}>. Reason: {}",
                            ticket.id, ticket.channel_id, interaction.actor_id, reason
                        ),
                    )
                    .await?
            }
            context.discord.respond_to_modal(interaction,CommandResponse::ephemeral("Ticket closed. The channel is now read-only and its transcript was retained.")).await?;
            return Ok(Some(ModuleEventOutcome {
                action: "ticket.closed".into(),
                target_type: "ticket".into(),
                target_id: Some(ticket.id.to_string()),
            }));
        }
        let panel_id = interaction
            .custom_id
            .strip_prefix(INTAKE_PREFIX)
            .context("invalid ticket modal ID")?
            .parse::<i64>()?;
        let panel = context
            .services
            .ticket_panel(interaction.guild_id, panel_id)
            .await?
            .context("ticket panel no longer exists")?;
        let subject = interaction
            .values
            .get("subject")
            .map(String::as_str)
            .unwrap_or("Support request");
        let details = interaction
            .values
            .get("details")
            .map(String::as_str)
            .unwrap_or("");
        let channel = open_from_panel(
            context,
            interaction.guild_id,
            interaction.actor_id,
            &panel,
            subject,
            details,
        )
        .await?;
        context
            .discord
            .respond_to_modal(
                interaction,
                CommandResponse::ephemeral(format!("Your private ticket is ready: <#{channel}>.")),
            )
            .await?;
        Ok(Some(ModuleEventOutcome {
            action: "ticket.opened".into(),
            target_type: "ticket_panel".into(),
            target_id: Some(panel_id.to_string()),
        }))
    }
}

async fn open_from_panel(
    context: &ModuleContext,
    guild_id: u64,
    actor_id: u64,
    panel: &TicketPanelResult,
    subject: &str,
    note: &str,
) -> Result<u64> {
    if context
        .services
        .open_ticket_count(guild_id, Some(panel.id), actor_id)
        .await?
        >= panel.max_open_per_user
    {
        anyhow::bail!("you have reached this panel's open-ticket limit")
    }
    let name = panel
        .channel_name
        .replace("{user_id}", &actor_id.to_string())
        .chars()
        .take(100)
        .collect::<String>();
    let channel = context
        .discord
        .create_ticket_channel(
            guild_id,
            panel.category_id,
            actor_id,
            &panel.support_role_ids,
            &name,
            "Orivane support ticket opened",
        )
        .await?;
    if let Err(error) = context
        .services
        .record_open_ticket(TicketRecordInput {
            guild_id,
            opener_id: actor_id,
            channel_id: channel,
            panel_id: Some(panel.id),
            subject: subject.into(),
            opening_note: note.into(),
        })
        .await
    {
        let _ = context
            .discord
            .delete_channel(channel, "Ticket persistence failed; rolling back")
            .await;
        return Err(error);
    }
    let ticket = context
        .services
        .ticket_by_channel(guild_id, channel)
        .await?
        .context("created ticket could not be loaded")?;
    let roles = panel
        .support_role_ids
        .iter()
        .map(|id| format!("<@&{id}>"))
        .collect::<Vec<_>>()
        .join(" ");
    let mut welcome = panel
        .welcome_message
        .replace("{user}", &format!("<@{actor_id}>"))
        .replace("{user_id}", &actor_id.to_string())
        .replace("{support_roles}", &roles);
    if !note.trim().is_empty() {
        welcome.push_str(&format!("\n\n**Opening note:** {}", note.trim()))
    }
    context
        .discord
        .send_message_with_buttons(
            channel,
            &welcome,
            &[
                MessageButton {
                    custom_id: format!("{CLAIM_PREFIX}{}:{}", panel.id, ticket.id),
                    label: "Claim ticket".into(),
                    style: MessageButtonStyle::Primary,
                },
                MessageButton {
                    custom_id: format!("{CLOSE_PREFIX}{}", ticket.id),
                    label: "Close ticket".into(),
                    style: MessageButtonStyle::Danger,
                },
            ],
        )
        .await?;
    Ok(channel)
}

async fn can_manage_ticket(
    context: &ModuleContext,
    config: &GuildConfig,
    guild_id: u64,
    panel_id: Option<i64>,
    opener_id: u64,
    actor_id: u64,
    actor_roles: &[u64],
) -> Result<bool> {
    if actor_id == opener_id {
        return Ok(true);
    }
    let roles = if let Some(panel_id) = panel_id {
        context
            .services
            .ticket_panel(guild_id, panel_id)
            .await?
            .map(|panel| panel.support_role_ids)
            .unwrap_or_default()
    } else {
        config
            .modules
            .tickets
            .support_role_ids
            .iter()
            .filter_map(|id| id.parse().ok())
            .collect()
    };
    Ok(actor_roles.iter().any(|role| roles.contains(role)))
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
            panel_id: None,
            subject: interaction
                .string("reason")
                .unwrap_or("Support request")
                .to_owned(),
            opening_note: interaction.string("reason").unwrap_or_default().to_owned(),
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
    let Some(ticket) = context
        .services
        .ticket_by_channel(interaction.guild_id, channel_id)
        .await?
    else {
        context
            .discord
            .respond_to_command(
                interaction,
                CommandResponse::ephemeral("This channel is not an open Orivane ticket."),
            )
            .await?;
        return Ok(());
    };
    if ticket.status != "open" {
        context
            .discord
            .respond_to_command(
                interaction,
                CommandResponse::ephemeral("This ticket is already closed."),
            )
            .await?;
        return Ok(());
    }
    let reason = interaction.string("reason").unwrap_or("No reason provided");
    let transcript = context
        .discord
        .channel_transcript(channel_id)
        .await
        .unwrap_or_else(|error| format!("Transcript capture failed: {error}"));
    if !context
        .services
        .close_ticket(
            interaction.guild_id,
            ticket.id,
            interaction.actor_id,
            reason,
            &transcript,
        )
        .await?
    {
        context
            .discord
            .respond_to_command(
                interaction,
                CommandResponse::ephemeral("This ticket could not be closed."),
            )
            .await?;
        return Ok(());
    }
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
            CommandResponse::ephemeral(
                "Ticket closed. The channel is now read-only and its transcript was retained.",
            ),
        )
        .await?;
    context
        .discord
        .set_ticket_member_writable(
            channel_id,
            ticket.opener_id,
            false,
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
            actor_permissions: u64::MAX,
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
                    panel_id: None,
                    subject: "Billing help".to_owned(),
                    opening_note: "Billing help".to_owned(),
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

    #[tokio::test]
    async fn opens_a_close_modal_and_retains_the_transcript() {
        let discord = FakeDiscord::new();
        let services = FakeModuleServices::default();
        let context = ModuleContext {
            discord: Arc::new(discord.clone()),
            services: Arc::new(services),
        };
        let mut config = GuildConfig::for_guild("10");
        config.modules.tickets.enabled = true;
        let button = ComponentInteraction {
            id: 2,
            token: "button".into(),
            guild_id: 10,
            channel_id: Some(90),
            actor_id: 40,
            actor_role_ids: vec![],
            custom_id: format!("{CLOSE_PREFIX}7"),
        };
        TicketsModule
            .on_component(&context, &config, &button)
            .await
            .unwrap();
        assert!(matches!(
            discord.calls().last(),
            Some(DiscordCall::OpenComponentModal { .. })
        ));
        let submit = ModalSubmitInteraction {
            id: 3,
            token: "modal".into(),
            guild_id: 10,
            channel_id: Some(90),
            actor_id: 40,
            actor_role_ids: vec![],
            custom_id: format!("{CLOSE_MODAL_PREFIX}7"),
            values: BTreeMap::from([("reason".into(), "Resolved".into())]),
        };
        TicketsModule
            .on_modal_submit(&context, &config, &submit)
            .await
            .unwrap();
        assert!(
            discord
                .calls()
                .iter()
                .any(|call| matches!(call, DiscordCall::ChannelTranscript { channel_id: 90 }))
        );
        assert!(discord.calls().iter().any(|call| matches!(
            call,
            DiscordCall::SetTicketMemberWritable {
                channel_id: 90,
                member_id: 40,
                writable: false,
                ..
            }
        )));
    }

    #[tokio::test]
    async fn records_ticket_channel_activity() {
        let services = FakeModuleServices::default();
        let context = ModuleContext {
            discord: Arc::new(FakeDiscord::new()),
            services: Arc::new(services.clone()),
        };
        let outcome = TicketsModule
            .on_message_create(
                &context,
                &GuildConfig::for_guild("10"),
                &MessageCreate {
                    guild_id: 10,
                    channel_id: 90,
                    message_id: 100,
                    author_id: 40,
                    author_is_bot: false,
                    role_ids: vec![],
                    mention_count: 0,
                    content: "Still need help".into(),
                },
            )
            .await
            .unwrap();
        assert_eq!(outcome.unwrap().action, "ticket.activity.recorded");
        assert_eq!(
            services.calls(),
            vec![ServiceCall::TouchTicketActivity {
                guild_id: 10,
                channel_id: 90,
            }]
        );
    }

    #[tokio::test]
    async fn auto_closes_inactive_tickets_and_purges_expired_transcripts() {
        let discord = FakeDiscord::new();
        let services = FakeModuleServices::default();
        services.set_due_tickets(vec![bot_module_api::TicketResult {
            id: 7,
            panel_id: Some(1),
            opener_id: 40,
            channel_id: 90,
            status: "open".into(),
        }]);
        let context = ModuleContext {
            discord: Arc::new(discord.clone()),
            services: Arc::new(services.clone()),
        };
        let mut config = GuildConfig::for_guild("10");
        config.modules.tickets.enabled = true;
        config.modules.tickets.transcript_channel_id = Some("70".into());
        let outcomes = TicketsModule
            .on_scheduled_tick(&context, &config, &ScheduledTick { now_unix: 100 })
            .await
            .unwrap();
        assert_eq!(outcomes.len(), 1);
        assert_eq!(outcomes[0].action, "ticket.auto_closed");
        assert!(discord.calls().iter().any(|call| matches!(
            call,
            DiscordCall::SetTicketMemberWritable {
                channel_id: 90,
                member_id: 40,
                writable: false,
                ..
            }
        )));
        assert!(
            services
                .calls()
                .contains(&ServiceCall::PurgeExpiredTicketTranscripts)
        );
    }
}

use anyhow::{Context, Result, bail};
use async_trait::async_trait;
use bot_core::GuildConfig;
use bot_discord::{
    CommandDefinition, CommandInteraction, CommandOptionDefinition, CommandOptionKind,
    CommandResponse,
};
use bot_module_api::{
    BotModule, ModuleContext, ModuleEventOutcome, ReminderInput, ReminderResult, ScheduledTick,
};

pub struct RemindersModule;

pub const COMMANDS: &[CommandDefinition] = &[
    CommandDefinition {
        name: "remind",
        description: "Schedule a personal reminder",
        guild_only: true,
        default_permissions: None,
        options: &[
            option(
                "message",
                "What should Orivane remind you about?",
                CommandOptionKind::String,
                true,
                None,
                None,
                Some(1),
                Some(1000),
            ),
            option(
                "minutes",
                "How many minutes from now?",
                CommandOptionKind::Integer,
                true,
                Some(1),
                Some(5_256_000),
                None,
                None,
            ),
            option(
                "dm",
                "Deliver this reminder privately",
                CommandOptionKind::Boolean,
                false,
                None,
                None,
                None,
                None,
            ),
        ],
    },
    CommandDefinition {
        name: "reminders",
        description: "List your active reminders",
        guild_only: true,
        default_permissions: None,
        options: &[],
    },
    CommandDefinition {
        name: "reminder-cancel",
        description: "Cancel one of your reminders",
        guild_only: true,
        default_permissions: None,
        options: &[option(
            "id",
            "Reminder ID",
            CommandOptionKind::Integer,
            true,
            Some(1),
            None,
            None,
            None,
        )],
    },
];

#[allow(clippy::too_many_arguments)]
const fn option(
    name: &'static str,
    description: &'static str,
    kind: CommandOptionKind,
    required: bool,
    min_value: Option<i64>,
    max_value: Option<i64>,
    min_length: Option<u16>,
    max_length: Option<u16>,
) -> CommandOptionDefinition {
    CommandOptionDefinition {
        name,
        description,
        kind,
        required,
        min_value,
        max_value,
        min_length,
        max_length,
    }
}

#[async_trait]
impl BotModule for RemindersModule {
    fn name(&self) -> &'static str {
        "reminders"
    }
    fn enabled(&self, config: &GuildConfig) -> bool {
        config.modules.reminders.enabled
    }
    fn commands(&self) -> &'static [CommandDefinition] {
        COMMANDS
    }
    fn handles_scheduled_tick(&self) -> bool {
        true
    }

    async fn on_command(
        &self,
        context: &ModuleContext,
        config: &GuildConfig,
        interaction: &CommandInteraction,
    ) -> Result<()> {
        match interaction.command_name.as_str() {
            "remind" => create(context, config, interaction).await,
            "reminders" => list(context, interaction).await,
            "reminder-cancel" => cancel(context, interaction).await,
            _ => Ok(()),
        }
    }

    async fn on_scheduled_tick(
        &self,
        context: &ModuleContext,
        config: &GuildConfig,
        event: &ScheduledTick,
    ) -> Result<Vec<ModuleEventOutcome>> {
        let guild_id = config
            .guild_id
            .parse::<u64>()
            .context("reminder guild ID is invalid")?;
        let mut outcomes = Vec::new();
        for reminder in context
            .services
            .claim_due_reminders(guild_id, event.now_unix, 100)
            .await?
        {
            let result = deliver(context, &reminder).await;
            let outcome = match result {
                Ok(()) => {
                    context
                        .services
                        .complete_reminder(guild_id, reminder.id)
                        .await?;
                    ModuleEventOutcome {
                        action: "reminder.delivered".into(),
                        target_type: "reminder".into(),
                        target_id: Some(reminder.id.to_string()),
                    }
                }
                Err(_) => {
                    let backoff = 60_u64
                        .saturating_mul(2_u64.pow(u32::from(reminder.attempts.saturating_sub(1))));
                    let terminal = context
                        .services
                        .retry_reminder(
                            guild_id,
                            reminder.id,
                            event.now_unix.saturating_add(backoff),
                        )
                        .await?;
                    ModuleEventOutcome {
                        action: if terminal {
                            "reminder.failed"
                        } else {
                            "reminder.retry_scheduled"
                        }
                        .into(),
                        target_type: "reminder".into(),
                        target_id: Some(reminder.id.to_string()),
                    }
                }
            };
            outcomes.push(outcome);
        }
        Ok(outcomes)
    }
}

async fn create(
    context: &ModuleContext,
    config: &GuildConfig,
    interaction: &CommandInteraction,
) -> Result<()> {
    let settings = &config.modules.reminders;
    let message = interaction
        .string("message")
        .context("reminder message is required")?
        .trim();
    let minutes = interaction
        .integer("minutes")
        .context("reminder delay is required")?;
    let minutes = u64::try_from(minutes).context("reminder delay is invalid")?;
    if minutes < u64::from(settings.minimum_delay_minutes)
        || minutes > u64::from(settings.maximum_delay_days) * 1_440
    {
        bail!("reminder delay is outside this server's configured limits");
    }
    let dm = matches!(
        interaction.options.get("dm"),
        Some(bot_discord::CommandValue::Boolean(true))
    );
    if dm && !settings.allow_direct_messages {
        bail!("direct-message reminders are disabled in this server");
    }
    let channel_id = if dm {
        None
    } else {
        Some(
            interaction
                .channel_id
                .context("reminder channel is missing")?,
        )
    };
    let remind_at_unix = now_unix().saturating_add(minutes.saturating_mul(60));
    let created = context
        .services
        .create_reminder(ReminderInput {
            guild_id: interaction.guild_id,
            channel_id,
            user_id: interaction.actor_id,
            message: message.into(),
            delivery_mode: if dm { "dm".into() } else { "channel".into() },
            remind_at_unix,
            max_active_per_user: settings.max_active_per_user,
        })
        .await?;
    let response = match created {
        Some(reminder) => format!(
            "Reminder #{} scheduled for <t:{}:F> (<t:{}:R>).",
            reminder.id, reminder.remind_at_unix, reminder.remind_at_unix
        ),
        None => format!(
            "You already have {} active reminders in this server. Cancel one before adding another.",
            settings.max_active_per_user
        ),
    };
    context
        .discord
        .respond_to_command(interaction, CommandResponse::ephemeral(response))
        .await?;
    Ok(())
}

async fn list(context: &ModuleContext, interaction: &CommandInteraction) -> Result<()> {
    let reminders = context
        .services
        .user_reminders(interaction.guild_id, interaction.actor_id, 25)
        .await?;
    let content = if reminders.is_empty() {
        "You have no active reminders in this server.".into()
    } else {
        let rows = reminders
            .iter()
            .map(|item| {
                format!(
                    "**#{}** · <t:{}:R> · {}",
                    item.id,
                    item.remind_at_unix,
                    truncate(&item.message, 80)
                )
            })
            .collect::<Vec<_>>()
            .join("\n");
        format!("Your active reminders:\n{rows}")
    };
    context
        .discord
        .respond_to_command(interaction, CommandResponse::ephemeral(content))
        .await?;
    Ok(())
}

async fn cancel(context: &ModuleContext, interaction: &CommandInteraction) -> Result<()> {
    let id = interaction
        .integer("id")
        .context("reminder ID is required")?;
    let cancelled = context
        .services
        .cancel_reminder(interaction.guild_id, id, interaction.actor_id)
        .await?;
    let content = if cancelled {
        format!("Reminder #{id} was cancelled.")
    } else {
        "That active reminder was not found or does not belong to you.".into()
    };
    context
        .discord
        .respond_to_command(interaction, CommandResponse::ephemeral(content))
        .await?;
    Ok(())
}

async fn deliver(context: &ModuleContext, reminder: &ReminderResult) -> Result<()> {
    let safe_message = reminder.message.replace('@', "@\u{200b}");
    if reminder.delivery_mode == "dm" {
        context
            .discord
            .send_direct_message(reminder.user_id, &format!("Reminder: {safe_message}"))
            .await?;
    } else {
        let channel_id = reminder
            .channel_id
            .context("channel reminder has no channel")?;
        context
            .discord
            .send_message(
                channel_id,
                &format!("<@{}>, reminder: {safe_message}", reminder.user_id),
            )
            .await?;
    }
    Ok(())
}

fn now_unix() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}
fn truncate(value: &str, max: usize) -> String {
    let mut chars = value.chars();
    let shortened = chars.by_ref().take(max).collect::<String>();
    if chars.next().is_some() {
        format!("{shortened}…")
    } else {
        shortened
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use bot_discord::{CommandValue, DiscordError};
    use bot_test_support::{DiscordCall, FakeDiscord, FakeModuleServices, ServiceCall};
    use std::{collections::BTreeMap, sync::Arc};

    fn context() -> (ModuleContext, FakeDiscord, FakeModuleServices) {
        let discord = FakeDiscord::new();
        let services = FakeModuleServices::default();
        (
            ModuleContext {
                discord: Arc::new(discord.clone()),
                services: Arc::new(services.clone()),
            },
            discord,
            services,
        )
    }
    fn command(name: &str, options: BTreeMap<String, CommandValue>) -> CommandInteraction {
        CommandInteraction {
            id: 1,
            token: "token".into(),
            guild_id: 10,
            channel_id: Some(20),
            actor_id: 30,
            actor_role_ids: vec![],
            actor_permissions: 0,
            command_name: name.into(),
            options,
        }
    }

    #[tokio::test]
    async fn creates_a_private_reminder() {
        let (context, discord, services) = context();
        let mut config = GuildConfig::for_guild("10");
        config.modules.reminders.enabled = true;
        let interaction = command(
            "remind",
            BTreeMap::from([
                ("message".into(), CommandValue::String("check oven".into())),
                ("minutes".into(), CommandValue::Integer(15)),
                ("dm".into(), CommandValue::Boolean(true)),
            ]),
        );
        RemindersModule
            .on_command(&context, &config, &interaction)
            .await
            .unwrap();
        assert!(
            matches!(&services.calls()[0], ServiceCall::CreateReminder(input) if input.delivery_mode == "dm" && input.channel_id.is_none())
        );
        assert!(
            matches!(&discord.calls()[0], DiscordCall::RespondToCommand { response, .. } if response.ephemeral)
        );
    }

    #[tokio::test]
    async fn delivers_due_channel_reminders() {
        let (context, discord, services) = context();
        let mut config = GuildConfig::for_guild("10");
        config.modules.reminders.enabled = true;
        let outcomes = RemindersModule
            .on_scheduled_tick(&context, &config, &ScheduledTick { now_unix: 100 })
            .await
            .unwrap();
        assert_eq!(outcomes[0].action, "reminder.delivered");
        assert!(
            matches!(&discord.calls()[0], DiscordCall::SendMessage { channel_id: 20, content } if content.contains("<@30>"))
        );
        assert!(services.calls().iter().any(|call| matches!(
            call,
            ServiceCall::CompleteReminder {
                reminder_id: 14,
                ..
            }
        )));
    }

    #[tokio::test]
    async fn retries_failed_delivery() {
        let (context, discord, services) = context();
        discord.fail_next(DiscordError::Request("no access".into()));
        let mut config = GuildConfig::for_guild("10");
        config.modules.reminders.enabled = true;
        let outcomes = RemindersModule
            .on_scheduled_tick(&context, &config, &ScheduledTick { now_unix: 100 })
            .await
            .unwrap();
        assert_eq!(outcomes[0].action, "reminder.retry_scheduled");
        assert!(services.calls().iter().any(|call| matches!(
            call,
            ServiceCall::RetryReminder {
                reminder_id: 14,
                next_attempt_unix: 160,
                ..
            }
        )));
    }
}

use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::{Context, Result, bail};
use async_trait::async_trait;
use bot_core::GuildConfig;
use bot_discord::{
    CommandDefinition, CommandInteraction, CommandOptionDefinition, CommandOptionKind,
    CommandResponse, ComponentInteraction, MessageButton, MessageButtonStyle,
};
use bot_module_api::{
    BotModule, GiveawayInput, GiveawayResult, ModuleContext, ModuleEventOutcome, ScheduledTick,
};

pub struct GiveawaysModule;
const ENTRY_PREFIX: &str = "orivane:giveaway:enter:";

pub const COMMANDS: &[CommandDefinition] = &[
    CommandDefinition {
        name: "giveaway-create",
        description: "Create a scheduled giveaway",
        guild_only: true,
        default_permissions: None,
        options: &[
            option(
                "title",
                "Prize or giveaway title",
                CommandOptionKind::String,
                true,
                None,
                None,
                Some(1),
                Some(100),
            ),
            option(
                "duration",
                "Duration in minutes",
                CommandOptionKind::Integer,
                false,
                Some(1),
                Some(525_600),
                None,
                None,
            ),
            option(
                "winners",
                "Number of winners",
                CommandOptionKind::Integer,
                false,
                Some(1),
                Some(25),
                None,
                None,
            ),
            option(
                "channel",
                "Channel for the giveaway",
                CommandOptionKind::Channel,
                false,
                None,
                None,
                None,
                None,
            ),
            option(
                "description",
                "Additional giveaway details",
                CommandOptionKind::String,
                false,
                None,
                None,
                Some(1),
                Some(1800),
            ),
            option(
                "required-role",
                "Role required to enter",
                CommandOptionKind::Role,
                false,
                None,
                None,
                None,
                None,
            ),
            option(
                "minimum-account-age",
                "Minimum account age in days",
                CommandOptionKind::Integer,
                false,
                Some(0),
                Some(3650),
                None,
                None,
            ),
            option(
                "bonus-role",
                "Role that receives bonus entries",
                CommandOptionKind::Role,
                false,
                None,
                None,
                None,
                None,
            ),
            option(
                "bonus-entries",
                "Extra entries granted to the bonus role",
                CommandOptionKind::Integer,
                false,
                Some(0),
                Some(25),
                None,
                None,
            ),
        ],
    },
    CommandDefinition {
        name: "giveaway-end",
        description: "End an active giveaway now",
        guild_only: true,
        default_permissions: None,
        options: &[option(
            "id",
            "Giveaway ID",
            CommandOptionKind::Integer,
            true,
            Some(1),
            None,
            None,
            None,
        )],
    },
    CommandDefinition {
        name: "giveaway-reroll",
        description: "Draw replacement giveaway winners",
        guild_only: true,
        default_permissions: None,
        options: &[option(
            "id",
            "Giveaway ID",
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
impl BotModule for GiveawaysModule {
    fn name(&self) -> &'static str {
        "giveaways"
    }
    fn enabled(&self, config: &GuildConfig) -> bool {
        config.modules.giveaways.enabled
    }
    fn commands(&self) -> &'static [CommandDefinition] {
        COMMANDS
    }
    fn handles_component(&self, custom_id: &str) -> bool {
        custom_id.starts_with(ENTRY_PREFIX)
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
            "giveaway-create" => create(context, config, interaction).await,
            "giveaway-end" => finish(context, config, interaction, false).await,
            "giveaway-reroll" => finish(context, config, interaction, true).await,
            _ => Ok(()),
        }
    }

    async fn on_component(
        &self,
        context: &ModuleContext,
        _config: &GuildConfig,
        interaction: &ComponentInteraction,
    ) -> Result<Option<ModuleEventOutcome>> {
        let giveaway_id = interaction
            .custom_id
            .strip_prefix(ENTRY_PREFIX)
            .context("invalid giveaway entry button")?
            .parse::<i64>()
            .context("invalid giveaway ID")?;
        let Some(giveaway) = context
            .services
            .giveaway(interaction.guild_id, giveaway_id)
            .await?
        else {
            respond_component(context, interaction, "This giveaway no longer exists.").await?;
            return Ok(None);
        };
        if giveaway.status != "active" || giveaway.ends_at_unix <= now_unix() {
            respond_component(context, interaction, "This giveaway has ended.").await?;
            return Ok(None);
        }
        if !giveaway.required_role_ids.is_empty()
            && !giveaway
                .required_role_ids
                .iter()
                .any(|required| interaction.actor_role_ids.contains(required))
        {
            respond_component(
                context,
                interaction,
                "You do not have a role required for this giveaway.",
            )
            .await?;
            return Ok(None);
        }
        if account_age_days(interaction.actor_id, now_unix())
            < u64::from(giveaway.minimum_account_age_days)
        {
            respond_component(
                context,
                interaction,
                "Your Discord account is too new to enter this giveaway.",
            )
            .await?;
            return Ok(None);
        }
        let bonus = giveaway
            .bonus_role_id
            .is_some_and(|role| interaction.actor_role_ids.contains(&role));
        let weight = 1_u8.saturating_add(if bonus { giveaway.bonus_entries } else { 0 });
        match context
            .services
            .enter_giveaway(
                interaction.guild_id,
                giveaway_id,
                interaction.actor_id,
                weight,
            )
            .await?
        {
            None => {
                respond_component(
                    context,
                    interaction,
                    "This giveaway is no longer accepting entries.",
                )
                .await?;
                Ok(None)
            }
            Some(false) => {
                respond_component(
                    context,
                    interaction,
                    "You are already entered in this giveaway.",
                )
                .await?;
                Ok(None)
            }
            Some(true) => {
                let message = if weight > 1 {
                    format!("You are entered with {weight} chances.")
                } else {
                    "You are entered in the giveaway.".into()
                };
                respond_component(context, interaction, &message).await?;
                Ok(Some(ModuleEventOutcome {
                    action: "giveaway.entered".into(),
                    target_type: "giveaway".into(),
                    target_id: Some(giveaway_id.to_string()),
                }))
            }
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
            .context("giveaway guild ID is invalid")?;
        let mut outcomes = Vec::new();
        for id in context
            .services
            .due_giveaway_ids(guild_id, event.now_unix)
            .await?
        {
            if let Some(result) = context.services.end_giveaway(guild_id, id, false).await? {
                announce_result(context, &result, result.reroll_count > 0).await?;
                context
                    .services
                    .mark_giveaway_announced(guild_id, id)
                    .await?;
                outcomes.push(ModuleEventOutcome {
                    action: "giveaway.ended".into(),
                    target_type: "giveaway".into(),
                    target_id: Some(id.to_string()),
                });
            }
        }
        for id in context
            .services
            .pending_giveaway_announcement_ids(guild_id)
            .await?
        {
            if let Some(result) = context.services.giveaway(guild_id, id).await? {
                announce_result(context, &result, result.reroll_count > 0).await?;
                context
                    .services
                    .mark_giveaway_announced(guild_id, id)
                    .await?;
                outcomes.push(ModuleEventOutcome {
                    action: "giveaway.announcement.retried".into(),
                    target_type: "giveaway".into(),
                    target_id: Some(id.to_string()),
                });
            }
        }
        Ok(outcomes)
    }
}

async fn create(
    context: &ModuleContext,
    config: &GuildConfig,
    interaction: &CommandInteraction,
) -> Result<()> {
    authorize(config, interaction)?;
    let settings = &config.modules.giveaways;
    let title = interaction
        .string("title")
        .context("giveaway title is required")?
        .trim();
    if title.is_empty() {
        bail!("giveaway title is empty");
    }
    let duration = interaction
        .integer("duration")
        .map_or(u64::from(settings.default_duration_minutes), |value| {
            u64::try_from(value).unwrap_or_default()
        });
    let winner_count = interaction
        .integer("winners")
        .map_or(settings.default_winner_count, |value| {
            u8::try_from(value).unwrap_or_default()
        });
    let channel_id = command_channel(interaction, "channel")
        .or_else(|| {
            settings
                .default_channel_id
                .as_deref()
                .and_then(|id| id.parse().ok())
        })
        .context("giveaway channel is required")?;
    let required_role_ids = interaction.role("required-role").map_or_else(
        || parse_roles(&settings.required_role_ids),
        |role| vec![role],
    );
    let minimum_account_age_days = interaction
        .integer("minimum-account-age")
        .map_or(settings.minimum_account_age_days, |value| {
            u16::try_from(value).unwrap_or_default()
        });
    let bonus_role_id = interaction.role("bonus-role").or_else(|| {
        settings
            .bonus_role_id
            .as_deref()
            .and_then(|id| id.parse().ok())
    });
    let bonus_entries = interaction
        .integer("bonus-entries")
        .map_or(settings.bonus_entries, |value| {
            u8::try_from(value).unwrap_or_default()
        });
    let giveaway = context
        .services
        .create_giveaway(GiveawayInput {
            guild_id: interaction.guild_id,
            channel_id,
            creator_id: interaction.actor_id,
            title: title.into(),
            description: interaction
                .string("description")
                .unwrap_or_default()
                .trim()
                .into(),
            winner_count,
            minimum_account_age_days,
            required_role_ids,
            bonus_role_id,
            bonus_entries,
            ends_at_unix: now_unix().saturating_add(duration.saturating_mul(60)),
        })
        .await?;
    let message_id = match context
        .discord
        .send_message_with_buttons(
            channel_id,
            &giveaway_message(
                &giveaway,
                interaction.string("description").unwrap_or_default(),
            ),
            &[MessageButton {
                custom_id: format!("{ENTRY_PREFIX}{}", giveaway.id),
                label: "Enter giveaway".into(),
                style: MessageButtonStyle::Primary,
            }],
        )
        .await
    {
        Ok(id) => id,
        Err(error) => {
            context
                .services
                .fail_giveaway_delivery(interaction.guild_id, giveaway.id)
                .await?;
            return Err(error.into());
        }
    };
    context
        .services
        .activate_giveaway(interaction.guild_id, giveaway.id, message_id)
        .await?;
    context
        .discord
        .respond_to_command(
            interaction,
            CommandResponse::ephemeral(format!("Giveaway #{} was created.", giveaway.id)),
        )
        .await?;
    Ok(())
}

async fn finish(
    context: &ModuleContext,
    config: &GuildConfig,
    interaction: &CommandInteraction,
    reroll: bool,
) -> Result<()> {
    authorize(config, interaction)?;
    let id = interaction
        .integer("id")
        .context("giveaway ID is required")?;
    let result = if reroll {
        context
            .services
            .reroll_giveaway(interaction.guild_id, id)
            .await?
    } else {
        context
            .services
            .end_giveaway(interaction.guild_id, id, true)
            .await?
    };
    let Some(result) = result else {
        context
            .discord
            .respond_to_command(
                interaction,
                CommandResponse::ephemeral(if reroll {
                    "No replacement entrants are available for that giveaway."
                } else {
                    "That giveaway is not active."
                }),
            )
            .await?;
        return Ok(());
    };
    announce_result(context, &result, reroll).await?;
    context
        .services
        .mark_giveaway_announced(interaction.guild_id, id)
        .await?;
    context
        .discord
        .respond_to_command(
            interaction,
            CommandResponse::ephemeral(if reroll {
                format!("Giveaway #{id} was rerolled.")
            } else {
                format!("Giveaway #{id} was ended.")
            }),
        )
        .await?;
    Ok(())
}

fn authorize(config: &GuildConfig, interaction: &CommandInteraction) -> Result<()> {
    let roles = &config.modules.giveaways.manager_role_ids;
    let configured = roles.iter().any(|role| {
        interaction
            .actor_role_ids
            .iter()
            .any(|id| role == &id.to_string())
    });
    if (!roles.is_empty() && !configured)
        || (roles.is_empty() && interaction.actor_permissions & 32 != 32)
    {
        bail!("you are not allowed to manage giveaways");
    }
    Ok(())
}

fn giveaway_message(giveaway: &GiveawayResult, description: &str) -> String {
    let description = safe(description.trim());
    let details = if description.is_empty() {
        String::new()
    } else {
        format!("\n{description}\n")
    };
    format!(
        "**Giveaway #{} · {}**\n{}\nEnds <t:{}:R> · {} winner{}\nPress the button below to enter.",
        giveaway.id,
        safe(&giveaway.title),
        details,
        giveaway.ends_at_unix,
        giveaway.winner_count,
        if giveaway.winner_count == 1 { "" } else { "s" }
    )
}

async fn announce_result(
    context: &ModuleContext,
    result: &GiveawayResult,
    reroll: bool,
) -> Result<()> {
    let winners = if result.winner_ids.is_empty() {
        "No eligible entries were received.".into()
    } else {
        result
            .winner_ids
            .iter()
            .map(|id| format!("<@{id}>"))
            .collect::<Vec<_>>()
            .join(", ")
    };
    let prefix = if reroll { "Rerolled" } else { "Ended" };
    context
        .discord
        .send_message(
            result.channel_id,
            &format!(
                "**{prefix}: Giveaway #{} · {}**\nWinner{}: {winners}",
                result.id,
                safe(&result.title),
                if result.winner_ids.len() == 1 {
                    ""
                } else {
                    "s"
                }
            ),
        )
        .await?;
    Ok(())
}

async fn respond_component(
    context: &ModuleContext,
    interaction: &ComponentInteraction,
    message: &str,
) -> Result<()> {
    context
        .discord
        .respond_to_component(interaction, CommandResponse::ephemeral(message))
        .await?;
    Ok(())
}
fn command_channel(interaction: &CommandInteraction, name: &str) -> Option<u64> {
    match interaction.options.get(name) {
        Some(bot_discord::CommandValue::Channel(id)) => Some(*id),
        _ => None,
    }
}
fn parse_roles(values: &[String]) -> Vec<u64> {
    values.iter().filter_map(|id| id.parse().ok()).collect()
}
fn now_unix() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}
fn account_age_days(user_id: u64, now: u64) -> u64 {
    now.saturating_sub((user_id >> 22) / 1_000 + 1_420_070_400) / 86_400
}
fn safe(value: &str) -> String {
    value.replace('@', "@\u{200b}")
}

#[cfg(test)]
mod tests {
    use super::*;
    use bot_core::GiveawaysConfig;
    use bot_discord::CommandValue;
    use bot_test_support::{DiscordCall, FakeDiscord, FakeModuleServices, ServiceCall};
    use std::{collections::BTreeMap, sync::Arc};

    fn config() -> GuildConfig {
        let mut config = GuildConfig::for_guild("10");
        config.modules.giveaways = GiveawaysConfig {
            enabled: true,
            default_channel_id: Some("20".into()),
            manager_role_ids: vec!["30".into()],
            default_duration_minutes: 60,
            default_winner_count: 1,
            minimum_account_age_days: 0,
            required_role_ids: vec![],
            bonus_role_id: Some("31".into()),
            bonus_entries: 2,
        };
        config
    }
    fn context(discord: &FakeDiscord, services: &FakeModuleServices) -> ModuleContext {
        ModuleContext {
            discord: Arc::new(discord.clone()),
            services: Arc::new(services.clone()),
        }
    }

    #[tokio::test]
    async fn creates_and_activates_a_button_giveaway() {
        let discord = FakeDiscord::new();
        discord.set_next_message_id(99);
        let services = FakeModuleServices::default();
        let interaction = CommandInteraction {
            id: 1,
            token: "x".into(),
            guild_id: 10,
            channel_id: Some(20),
            actor_id: 40,
            actor_role_ids: vec![30],
            actor_permissions: 0,
            command_name: "giveaway-create".into(),
            options: BTreeMap::from([(
                "title".into(),
                CommandValue::String("Nitro @everyone".into()),
            )]),
        };
        GiveawaysModule
            .on_command(&context(&discord, &services), &config(), &interaction)
            .await
            .unwrap();
        assert!(discord.calls().iter().any(|call| matches!(call, DiscordCall::SendMessageWithButtons { content, buttons, .. } if content.contains("@\u{200b}everyone") && buttons.len() == 1)));
        assert!(
            services
                .calls()
                .iter()
                .any(|call| matches!(call, ServiceCall::ActivateGiveaway { message_id: 99, .. }))
        );
    }

    #[tokio::test]
    async fn records_weighted_button_entries() {
        let discord = FakeDiscord::new();
        let services = FakeModuleServices::default();
        let interaction = ComponentInteraction {
            id: 2,
            token: "y".into(),
            guild_id: 10,
            channel_id: Some(20),
            actor_id: 1,
            actor_role_ids: vec![],
            custom_id: "orivane:giveaway:enter:12".into(),
        };
        GiveawaysModule
            .on_component(&context(&discord, &services), &config(), &interaction)
            .await
            .unwrap();
        assert!(services.calls().iter().any(|call| matches!(
            call,
            ServiceCall::EnterGiveaway {
                giveaway_id: 12,
                weight: 1,
                ..
            }
        )));
    }

    #[tokio::test]
    async fn announces_manual_winners() {
        let discord = FakeDiscord::new();
        let services = FakeModuleServices::default();
        let interaction = CommandInteraction {
            id: 3,
            token: "z".into(),
            guild_id: 10,
            channel_id: Some(11),
            actor_id: 40,
            actor_role_ids: vec![30],
            actor_permissions: 0,
            command_name: "giveaway-end".into(),
            options: BTreeMap::from([("id".into(), CommandValue::Integer(12))]),
        };
        GiveawaysModule
            .on_command(&context(&discord, &services), &config(), &interaction)
            .await
            .unwrap();
        assert!(discord.calls().iter().any(|call| matches!(call, DiscordCall::SendMessage { content, .. } if content.contains("<@50>"))));
    }
}

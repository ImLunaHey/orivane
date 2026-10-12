use anyhow::{Context, Result, bail};
use async_trait::async_trait;
use bot_core::GuildConfig;
use bot_discord::{
    CommandDefinition, CommandInteraction, CommandOptionDefinition, CommandOptionKind,
    CommandResponse,
};
use bot_module_api::{
    BotModule, ModerationCaseInput, ModuleContext, ModuleEventOutcome, ScheduledTick,
};

pub struct ModerationModule;

const KICK_MEMBERS: u64 = 1 << 1;
const BAN_MEMBERS: u64 = 1 << 2;
const MODERATE_MEMBERS: u64 = 1 << 40;
const ADMINISTRATOR: u64 = 1 << 3;
const MANAGE_CHANNELS: u64 = 1 << 4;
const MANAGE_MESSAGES: u64 = 1 << 13;

const USER_REASON_OPTIONS: &[CommandOptionDefinition] =
    &[user_option(), reason_option(), evidence_option()];
const TIMEOUT_OPTIONS: &[CommandOptionDefinition] = &[
    user_option(),
    CommandOptionDefinition {
        name: "minutes",
        description: "Timeout length in minutes (maximum 28 days)",
        kind: CommandOptionKind::Integer,
        required: true,
        min_value: Some(1),
        max_value: Some(40_320),
        min_length: None,
        max_length: None,
    },
    reason_option(),
    evidence_option(),
];
const BAN_OPTIONS: &[CommandOptionDefinition] = &[
    user_option(),
    reason_option(),
    CommandOptionDefinition {
        name: "delete_days",
        description: "Delete this many days of recent messages",
        kind: CommandOptionKind::Integer,
        required: false,
        min_value: Some(0),
        max_value: Some(7),
        min_length: None,
        max_length: None,
    },
    CommandOptionDefinition {
        name: "duration_hours",
        description: "Automatically unban after this many hours",
        kind: CommandOptionKind::Integer,
        required: false,
        min_value: Some(1),
        max_value: Some(8_760),
        min_length: None,
        max_length: None,
    },
    evidence_option(),
];
const PURGE_OPTIONS: &[CommandOptionDefinition] = &[
    CommandOptionDefinition {
        name: "count",
        description: "Number of recent messages to delete",
        kind: CommandOptionKind::Integer,
        required: true,
        min_value: Some(1),
        max_value: Some(100),
        min_length: None,
        max_length: None,
    },
    reason_option(),
];
const CHANNEL_REASON_OPTIONS: &[CommandOptionDefinition] = &[reason_option()];
const SLOWMODE_OPTIONS: &[CommandOptionDefinition] = &[
    CommandOptionDefinition {
        name: "seconds",
        description: "Delay between member messages; use 0 to disable",
        kind: CommandOptionKind::Integer,
        required: true,
        min_value: Some(0),
        max_value: Some(21_600),
        min_length: None,
        max_length: None,
    },
    reason_option(),
];

const fn user_option() -> CommandOptionDefinition {
    CommandOptionDefinition {
        name: "member",
        description: "Member to moderate",
        kind: CommandOptionKind::User,
        required: true,
        min_value: None,
        max_value: None,
        min_length: None,
        max_length: None,
    }
}

const fn reason_option() -> CommandOptionDefinition {
    CommandOptionDefinition {
        name: "reason",
        description: "Reason recorded in the moderation case",
        kind: CommandOptionKind::String,
        required: false,
        min_value: None,
        max_value: None,
        min_length: Some(1),
        max_length: Some(512),
    }
}

const fn evidence_option() -> CommandOptionDefinition {
    CommandOptionDefinition {
        name: "evidence_url",
        description: "Optional HTTP(S) link to supporting evidence",
        kind: CommandOptionKind::String,
        required: false,
        min_value: None,
        max_value: None,
        min_length: Some(8),
        max_length: Some(2_000),
    }
}

pub const COMMANDS: &[CommandDefinition] = &[
    command(
        "note",
        "Add a private moderation note for a member",
        MODERATE_MEMBERS,
        USER_REASON_OPTIONS,
    ),
    command(
        "warn",
        "Warn a member and create a moderation case",
        MODERATE_MEMBERS,
        USER_REASON_OPTIONS,
    ),
    command(
        "timeout",
        "Temporarily prevent a member from interacting",
        MODERATE_MEMBERS,
        TIMEOUT_OPTIONS,
    ),
    command(
        "kick",
        "Remove a member from the server",
        KICK_MEMBERS,
        USER_REASON_OPTIONS,
    ),
    command(
        "ban",
        "Ban a member from the server",
        BAN_MEMBERS,
        BAN_OPTIONS,
    ),
    command(
        "unban",
        "Remove a member's server ban",
        BAN_MEMBERS,
        USER_REASON_OPTIONS,
    ),
    command(
        "purge",
        "Delete recent messages in this channel",
        MANAGE_MESSAGES,
        PURGE_OPTIONS,
    ),
    command(
        "lock",
        "Prevent members from sending in this channel",
        MANAGE_CHANNELS,
        CHANNEL_REASON_OPTIONS,
    ),
    command(
        "unlock",
        "Allow members to send in this channel again",
        MANAGE_CHANNELS,
        CHANNEL_REASON_OPTIONS,
    ),
    command(
        "slowmode",
        "Set this channel's message cooldown",
        MANAGE_CHANNELS,
        SLOWMODE_OPTIONS,
    ),
];

const fn command(
    name: &'static str,
    description: &'static str,
    _permission: u64,
    options: &'static [CommandOptionDefinition],
) -> CommandDefinition {
    CommandDefinition {
        name,
        description,
        guild_only: true,
        default_permissions: None,
        options,
    }
}

#[async_trait]
impl BotModule for ModerationModule {
    fn name(&self) -> &'static str {
        "moderation"
    }
    fn enabled(&self, config: &GuildConfig) -> bool {
        config.modules.moderation.enabled
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
        authorize(config, interaction)?;
        let reason = interaction
            .string("reason")
            .map(str::trim)
            .filter(|reason| !reason.is_empty())
            .unwrap_or(&config.modules.moderation.default_reason)
            .to_owned();
        let evidence_urls = interaction
            .string("evidence_url")
            .filter(|url| url.starts_with("https://") || url.starts_with("http://"))
            .map(|url| vec![url.to_owned()])
            .unwrap_or_default();
        if matches!(
            interaction.command_name.as_str(),
            "purge" | "lock" | "unlock" | "slowmode"
        ) {
            return run_channel_command(context, interaction, &reason).await;
        }
        let member_id = interaction.user("member").context("member is required")?;
        if member_id == interaction.actor_id {
            bail!("moderators cannot target themselves");
        }
        match interaction.command_name.as_str() {
            "note" | "warn" => {}
            "timeout" => {
                let minutes = interaction
                    .integer("minutes")
                    .context("minutes is required")?;
                let seconds = u64::try_from(minutes)
                    .ok()
                    .and_then(|value| value.checked_mul(60))
                    .context("timeout duration is invalid")?;
                context
                    .discord
                    .timeout_member(interaction.guild_id, member_id, seconds, &reason)
                    .await?;
            }
            "kick" => {
                context
                    .discord
                    .kick_member(interaction.guild_id, member_id, &reason)
                    .await?
            }
            "ban" => {
                let delete_days = interaction.integer("delete_days").unwrap_or(0).clamp(0, 7);
                context
                    .discord
                    .ban_member(
                        interaction.guild_id,
                        member_id,
                        u32::try_from(delete_days * 86_400).unwrap_or_default(),
                        &reason,
                    )
                    .await?;
            }
            "unban" => {
                context
                    .discord
                    .unban_member(interaction.guild_id, member_id, &reason)
                    .await?
            }
            _ => return Ok(()),
        }

        let case_id = context
            .services
            .create_moderation_case(ModerationCaseInput {
                guild_id: interaction.guild_id,
                moderator_id: interaction.actor_id,
                member_id,
                action: interaction.command_name.clone(),
                reason: reason.clone(),
                evidence_urls,
            })
            .await?;

        if interaction.command_name == "ban"
            && let Some(hours) = interaction.integer("duration_hours")
        {
            let now = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs();
            let duration = u64::try_from(hours)
                .unwrap_or_default()
                .saturating_mul(3_600);
            context
                .services
                .schedule_moderation_job(
                    interaction.guild_id,
                    member_id,
                    "unban",
                    &format!("Temporary ban case #{case_id} expired"),
                    now.saturating_add(duration),
                )
                .await?;
        }

        let escalation = if interaction.command_name == "warn" {
            let warnings = context
                .services
                .moderation_warning_count(interaction.guild_id, member_id)
                .await?;
            if let Some(threshold) = config
                .modules
                .moderation
                .warning_thresholds
                .iter()
                .find(|threshold| u64::from(threshold.warnings) == warnings)
            {
                let escalation_reason = format!(
                    "Automatic {} after {} warnings: {}",
                    threshold.action, warnings, reason
                );
                match threshold.action.as_str() {
                    "timeout" => {
                        context
                            .discord
                            .timeout_member(
                                interaction.guild_id,
                                member_id,
                                u64::from(threshold.timeout_minutes) * 60,
                                &escalation_reason,
                            )
                            .await?
                    }
                    "kick" => {
                        context
                            .discord
                            .kick_member(interaction.guild_id, member_id, &escalation_reason)
                            .await?
                    }
                    "ban" => {
                        context
                            .discord
                            .ban_member(interaction.guild_id, member_id, 0, &escalation_reason)
                            .await?
                    }
                    _ => {}
                }
                context
                    .services
                    .create_moderation_case(ModerationCaseInput {
                        guild_id: interaction.guild_id,
                        moderator_id: interaction.actor_id,
                        member_id,
                        action: format!("auto_{}", threshold.action),
                        reason: escalation_reason,
                        evidence_urls: Vec::new(),
                    })
                    .await?;
                Some(threshold.action.as_str())
            } else {
                None
            }
        } else {
            None
        };

        let notified =
            if config.modules.moderation.dm_on_action && interaction.command_name != "note" {
                context
                    .discord
                    .send_direct_message(
                        member_id,
                        &format!(
                            "A moderation action (`{}`) was applied in server `{}`. Reason: {}",
                            interaction.command_name, interaction.guild_id, reason
                        ),
                    )
                    .await
                    .is_ok()
            } else {
                false
            };
        let notification = if notified { " Member notified." } else { "" };
        let escalation = escalation
            .map(|action| format!(" Warning threshold applied: {action}."))
            .unwrap_or_default();
        context
            .discord
            .respond_to_command(
                interaction,
                CommandResponse::ephemeral(format!(
                    "{} recorded for <@{}> as case #{}.{}{}",
                    title(&interaction.command_name),
                    member_id,
                    case_id,
                    notification,
                    escalation
                )),
            )
            .await?;
        Ok(())
    }

    async fn on_scheduled_tick(
        &self,
        context: &ModuleContext,
        config: &GuildConfig,
        tick: &ScheduledTick,
    ) -> Result<Vec<ModuleEventOutcome>> {
        let guild_id = config.guild_id.parse::<u64>().context("invalid guild ID")?;
        let jobs = context
            .services
            .claim_due_moderation_jobs(guild_id, tick.now_unix, 50)
            .await?;
        let mut outcomes = Vec::new();
        for job in jobs {
            let result = match job.action.as_str() {
                "unban" => {
                    context
                        .discord
                        .unban_member(guild_id, job.member_id, &job.reason)
                        .await
                }
                _ => continue,
            };
            let error = result.as_ref().err().map(ToString::to_string);
            let retry_delay = 30_u64.saturating_mul(2_u64.pow(u32::from(job.attempts.min(5))));
            let terminal = context
                .services
                .finish_moderation_job(
                    guild_id,
                    job.id,
                    result.is_ok(),
                    error.as_deref(),
                    tick.now_unix.saturating_add(retry_delay),
                )
                .await?;
            if result.is_ok() {
                context
                    .services
                    .create_moderation_case(ModerationCaseInput {
                        guild_id,
                        moderator_id: 0,
                        member_id: job.member_id,
                        action: job.action.clone(),
                        reason: job.reason.clone(),
                        evidence_urls: Vec::new(),
                    })
                    .await?;
            }
            if result.is_ok() || terminal {
                outcomes.push(ModuleEventOutcome {
                    action: if result.is_ok() {
                        "moderation.temporary_ban.expired".into()
                    } else {
                        "moderation.temporary_ban.failed".into()
                    },
                    target_type: "member".into(),
                    target_id: Some(job.member_id.to_string()),
                });
            }
        }
        Ok(outcomes)
    }
}

fn authorize(config: &GuildConfig, interaction: &CommandInteraction) -> Result<()> {
    if interaction.actor_permissions & ADMINISTRATOR != 0 {
        return Ok(());
    }
    let roles = &config.modules.moderation.moderator_role_ids;
    if !roles.is_empty()
        && interaction.actor_role_ids.iter().any(|role| {
            roles
                .iter()
                .any(|configured| configured == &role.to_string())
        })
    {
        return Ok(());
    }
    if roles.is_empty()
        && interaction.actor_permissions & required_permission(&interaction.command_name) != 0
    {
        return Ok(());
    }
    bail!("you are not allowed to use this moderation command")
}

fn required_permission(command: &str) -> u64 {
    match command {
        "kick" => KICK_MEMBERS,
        "ban" | "unban" => BAN_MEMBERS,
        "purge" => MANAGE_MESSAGES,
        "lock" | "unlock" | "slowmode" => MANAGE_CHANNELS,
        _ => MODERATE_MEMBERS,
    }
}

async fn run_channel_command(
    context: &ModuleContext,
    interaction: &CommandInteraction,
    reason: &str,
) -> Result<()> {
    let channel_id = interaction
        .channel_id
        .context("this command requires a channel")?;
    let response = match interaction.command_name.as_str() {
        "purge" => {
            let count = u16::try_from(interaction.integer("count").context("count is required")?)
                .context("purge count is invalid")?;
            let deleted = context
                .discord
                .purge_messages(channel_id, count, reason)
                .await?;
            format!("Deleted {deleted} recent messages.")
        }
        "lock" | "unlock" => {
            let locked = interaction.command_name == "lock";
            context
                .discord
                .set_channel_locked(interaction.guild_id, channel_id, locked, reason)
                .await?;
            if locked {
                "Channel locked.".into()
            } else {
                "Channel unlocked.".into()
            }
        }
        "slowmode" => {
            let seconds = u16::try_from(
                interaction
                    .integer("seconds")
                    .context("seconds is required")?,
            )
            .context("slowmode delay is invalid")?;
            context
                .discord
                .set_channel_slowmode(channel_id, seconds, reason)
                .await?;
            if seconds == 0 {
                "Slowmode disabled.".into()
            } else {
                format!("Slowmode set to {seconds} seconds.")
            }
        }
        _ => return Ok(()),
    };
    context
        .discord
        .respond_to_command(interaction, CommandResponse::ephemeral(response))
        .await?;
    Ok(())
}

fn title(action: &str) -> &str {
    match action {
        "note" => "Note",
        "warn" => "Warning",
        "timeout" => "Timeout",
        "kick" => "Kick",
        "ban" => "Ban",
        "unban" => "Unban",
        _ => "Action",
    }
}

#[cfg(test)]
mod tests {
    use std::{collections::BTreeMap, sync::Arc};

    use bot_discord::CommandValue;
    use bot_module_api::ModerationJobResult;
    use bot_test_support::{DiscordCall, FakeDiscord, FakeModuleServices, ServiceCall};

    use super::*;

    fn interaction(command: &str, options: BTreeMap<String, CommandValue>) -> CommandInteraction {
        CommandInteraction {
            id: 10,
            token: "token".to_owned(),
            guild_id: 20,
            channel_id: Some(30),
            actor_id: 40,
            actor_role_ids: vec![],
            actor_permissions: u64::MAX,
            command_name: command.to_owned(),
            options,
        }
    }

    #[tokio::test]
    async fn timeout_applies_action_persists_case_and_notifies() {
        let discord = FakeDiscord::new();
        let services = FakeModuleServices::default();
        let context = ModuleContext {
            discord: Arc::new(discord.clone()),
            services: Arc::new(services.clone()),
        };
        let mut config = GuildConfig::for_guild("20");
        config.modules.moderation.enabled = true;
        let command = interaction(
            "timeout",
            BTreeMap::from([
                ("member".to_owned(), CommandValue::User(50)),
                ("minutes".to_owned(), CommandValue::Integer(15)),
                ("reason".to_owned(), CommandValue::String("spam".to_owned())),
            ]),
        );

        ModerationModule
            .on_command(&context, &config, &command)
            .await
            .unwrap();

        assert_eq!(
            services.calls(),
            vec![ServiceCall::CreateModerationCase(ModerationCaseInput {
                guild_id: 20,
                moderator_id: 40,
                member_id: 50,
                action: "timeout".to_owned(),
                reason: "spam".to_owned(),
                evidence_urls: Vec::new(),
            })]
        );
        assert_eq!(
            discord.calls(),
            vec![
                DiscordCall::TimeoutMember {
                    guild_id: 20,
                    user_id: 50,
                    duration_seconds: 900,
                    reason: "spam".to_owned(),
                },
                DiscordCall::SendDirectMessage {
                    user_id: 50,
                    content:
                        "A moderation action (`timeout`) was applied in server `20`. Reason: spam"
                            .to_owned(),
                },
                DiscordCall::RespondToCommand {
                    interaction_id: 10,
                    response: CommandResponse::ephemeral(
                        "Timeout recorded for <@50> as case #1. Member notified."
                    ),
                },
            ]
        );
    }

    #[tokio::test]
    async fn rejects_self_moderation_before_side_effects() {
        let discord = FakeDiscord::new();
        let services = FakeModuleServices::default();
        let context = ModuleContext {
            discord: Arc::new(discord.clone()),
            services: Arc::new(services.clone()),
        };
        let config = GuildConfig::for_guild("20");
        let command = interaction(
            "ban",
            BTreeMap::from([("member".to_owned(), CommandValue::User(40))]),
        );
        assert!(
            ModerationModule
                .on_command(&context, &config, &command)
                .await
                .is_err()
        );
        assert!(discord.calls().is_empty());
        assert!(services.calls().is_empty());
    }

    #[tokio::test]
    async fn temporary_ban_schedules_and_executes_a_durable_unban() {
        let discord = FakeDiscord::new();
        let services = FakeModuleServices::default();
        let context = ModuleContext {
            discord: Arc::new(discord.clone()),
            services: Arc::new(services.clone()),
        };
        let mut config = GuildConfig::for_guild("20");
        config.modules.moderation.enabled = true;
        let command = interaction(
            "ban",
            BTreeMap::from([
                ("member".into(), CommandValue::User(50)),
                ("duration_hours".into(), CommandValue::Integer(24)),
            ]),
        );
        ModerationModule
            .on_command(&context, &config, &command)
            .await
            .unwrap();
        assert!(services.calls().iter().any(|call| matches!(
            call,
            ServiceCall::ScheduleModerationJob { guild_id: 20, member_id: 50, action, .. } if action == "unban"
        )));

        discord.take_calls();
        services.set_due_moderation_jobs(vec![ModerationJobResult {
            id: 9,
            member_id: 50,
            action: "unban".into(),
            reason: "Temporary ban expired".into(),
            attempts: 1,
        }]);
        let outcomes = ModerationModule
            .on_scheduled_tick(&context, &config, &ScheduledTick { now_unix: 2_000 })
            .await
            .unwrap();
        assert_eq!(outcomes[0].action, "moderation.temporary_ban.expired");
        assert_eq!(
            discord.calls(),
            vec![DiscordCall::UnbanMember {
                guild_id: 20,
                user_id: 50,
                reason: "Temporary ban expired".into(),
            }]
        );
    }

    #[tokio::test]
    async fn configured_moderator_role_can_warn_and_trigger_escalation() {
        let discord = FakeDiscord::new();
        let services = FakeModuleServices::default();
        let context = ModuleContext {
            discord: Arc::new(discord.clone()),
            services: Arc::new(services.clone()),
        };
        let mut config = GuildConfig::for_guild("20");
        config.modules.moderation.moderator_role_ids = vec!["70".into()];
        config.modules.moderation.warning_thresholds = vec![bot_core::WarningThresholdConfig {
            warnings: 1,
            action: "timeout".into(),
            timeout_minutes: 30,
        }];
        let mut command = interaction(
            "warn",
            BTreeMap::from([("member".to_owned(), CommandValue::User(50))]),
        );
        command.actor_permissions = 0;
        command.actor_role_ids = vec![70];

        ModerationModule
            .on_command(&context, &config, &command)
            .await
            .unwrap();

        assert!(discord.calls().iter().any(|call| matches!(
            call,
            DiscordCall::TimeoutMember {
                user_id: 50,
                duration_seconds: 1800,
                ..
            }
        )));
        assert_eq!(services.calls().len(), 3);
        assert!(matches!(
            services.calls()[1],
            ServiceCall::ModerationWarningCount {
                guild_id: 20,
                member_id: 50
            }
        ));
        assert!(
            matches!(&services.calls()[2], ServiceCall::CreateModerationCase(input) if input.action == "auto_timeout")
        );
    }

    #[tokio::test]
    async fn rejects_actor_without_permission_or_configured_role() {
        let discord = FakeDiscord::new();
        let services = FakeModuleServices::default();
        let context = ModuleContext {
            discord: Arc::new(discord.clone()),
            services: Arc::new(services.clone()),
        };
        let mut config = GuildConfig::for_guild("20");
        config.modules.moderation.moderator_role_ids = vec!["70".into()];
        let mut command = interaction(
            "warn",
            BTreeMap::from([("member".to_owned(), CommandValue::User(50))]),
        );
        command.actor_permissions = 0;

        let error = ModerationModule
            .on_command(&context, &config, &command)
            .await
            .unwrap_err();
        assert!(error.to_string().contains("not allowed"));
        assert!(discord.calls().is_empty());
        assert!(services.calls().is_empty());
    }

    #[tokio::test]
    async fn channel_commands_use_the_current_channel() {
        let discord = FakeDiscord::new();
        let services = FakeModuleServices::default();
        let context = ModuleContext {
            discord: Arc::new(discord.clone()),
            services: Arc::new(services),
        };
        let config = GuildConfig::for_guild("20");

        ModerationModule
            .on_command(
                &context,
                &config,
                &interaction(
                    "purge",
                    BTreeMap::from([("count".into(), CommandValue::Integer(25))]),
                ),
            )
            .await
            .unwrap();
        ModerationModule
            .on_command(&context, &config, &interaction("lock", BTreeMap::new()))
            .await
            .unwrap();
        ModerationModule
            .on_command(&context, &config, &interaction("unlock", BTreeMap::new()))
            .await
            .unwrap();
        ModerationModule
            .on_command(
                &context,
                &config,
                &interaction(
                    "slowmode",
                    BTreeMap::from([("seconds".into(), CommandValue::Integer(30))]),
                ),
            )
            .await
            .unwrap();

        let calls = discord.calls();
        assert!(calls.iter().any(|call| matches!(
            call,
            DiscordCall::PurgeMessages {
                channel_id: 30,
                count: 25,
                ..
            }
        )));
        assert!(calls.iter().any(|call| matches!(
            call,
            DiscordCall::SetChannelLocked {
                channel_id: 30,
                locked: true,
                ..
            }
        )));
        assert!(calls.iter().any(|call| matches!(
            call,
            DiscordCall::SetChannelLocked {
                channel_id: 30,
                locked: false,
                ..
            }
        )));
        assert!(calls.iter().any(|call| matches!(
            call,
            DiscordCall::SetChannelSlowmode {
                channel_id: 30,
                seconds: 30,
                ..
            }
        )));
    }
}

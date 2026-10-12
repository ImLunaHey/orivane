use anyhow::{Context, Result, bail};
use async_trait::async_trait;
use bot_core::GuildConfig;
use bot_discord::{
    CommandDefinition, CommandInteraction, CommandOptionDefinition, CommandOptionKind,
    CommandResponse,
};
use bot_module_api::{BotModule, ModerationCaseInput, ModuleContext};

pub struct ModerationModule;

const KICK_MEMBERS: u64 = 1 << 1;
const BAN_MEMBERS: u64 = 1 << 2;
const MODERATE_MEMBERS: u64 = 1 << 40;

const USER_REASON_OPTIONS: &[CommandOptionDefinition] = &[user_option(), reason_option()];
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
];

const fn command(
    name: &'static str,
    description: &'static str,
    permission: u64,
    options: &'static [CommandOptionDefinition],
) -> CommandDefinition {
    CommandDefinition {
        name,
        description,
        guild_only: true,
        default_permissions: Some(permission),
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

    async fn on_command(
        &self,
        context: &ModuleContext,
        config: &GuildConfig,
        interaction: &CommandInteraction,
    ) -> Result<()> {
        let member_id = interaction.user("member").context("member is required")?;
        if member_id == interaction.actor_id {
            bail!("moderators cannot target themselves");
        }
        let reason = interaction
            .string("reason")
            .map(str::trim)
            .filter(|reason| !reason.is_empty())
            .unwrap_or(&config.modules.moderation.default_reason)
            .to_owned();

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
            })
            .await?;

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
        context
            .discord
            .respond_to_command(
                interaction,
                CommandResponse::ephemeral(format!(
                    "{} recorded for <@{}> as case #{}.{}",
                    title(&interaction.command_name),
                    member_id,
                    case_id,
                    notification
                )),
            )
            .await?;
        Ok(())
    }
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
    use bot_test_support::{DiscordCall, FakeDiscord, FakeModuleServices, ServiceCall};

    use super::*;

    fn interaction(command: &str, options: BTreeMap<String, CommandValue>) -> CommandInteraction {
        CommandInteraction {
            id: 10,
            token: "token".to_owned(),
            guild_id: 20,
            channel_id: Some(30),
            actor_id: 40,
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
}

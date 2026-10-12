use anyhow::{Context, Result, bail};
use async_trait::async_trait;
use bot_core::GuildConfig;
use bot_discord::{
    CommandDefinition, CommandInteraction, CommandOptionDefinition, CommandOptionKind,
    CommandResponse,
};
use bot_module_api::{
    BotModule, ModuleContext, ModuleEventOutcome, ScheduledTick, VoiceStateChange,
};

pub struct TemporaryVoiceModule;

const RENAME_OPTIONS: &[CommandOptionDefinition] = &[CommandOptionDefinition {
    name: "name",
    description: "New channel name",
    kind: CommandOptionKind::String,
    required: true,
    min_value: None,
    max_value: None,
    min_length: Some(1),
    max_length: Some(100),
}];
const LIMIT_OPTIONS: &[CommandOptionDefinition] = &[CommandOptionDefinition {
    name: "limit",
    description: "Maximum members; use 0 for unlimited",
    kind: CommandOptionKind::Integer,
    required: true,
    min_value: Some(0),
    max_value: Some(99),
    min_length: None,
    max_length: None,
}];
const MEMBER_OPTIONS: &[CommandOptionDefinition] = &[CommandOptionDefinition {
    name: "member",
    description: "Member to target",
    kind: CommandOptionKind::User,
    required: true,
    min_value: None,
    max_value: None,
    min_length: None,
    max_length: None,
}];
const fn command(
    name: &'static str,
    description: &'static str,
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
pub const COMMANDS: &[CommandDefinition] = &[
    command(
        "voice-rename",
        "Rename your temporary voice channel",
        RENAME_OPTIONS,
    ),
    command(
        "voice-limit",
        "Set your temporary voice channel's user limit",
        LIMIT_OPTIONS,
    ),
    command("voice-lock", "Lock your temporary voice channel", &[]),
    command("voice-unlock", "Unlock your temporary voice channel", &[]),
    command(
        "voice-transfer",
        "Transfer temporary voice channel ownership",
        MEMBER_OPTIONS,
    ),
    command(
        "voice-kick",
        "Disconnect a member from your temporary voice channel",
        MEMBER_OPTIONS,
    ),
];

#[async_trait]
impl BotModule for TemporaryVoiceModule {
    fn name(&self) -> &'static str {
        "temporary_voice"
    }
    fn enabled(&self, config: &GuildConfig) -> bool {
        config.modules.temporary_voice.enabled
    }
    fn commands(&self) -> &'static [CommandDefinition] {
        COMMANDS
    }
    fn handles_voice_state_change(&self) -> bool {
        true
    }
    fn handles_scheduled_tick(&self) -> bool {
        true
    }

    async fn on_voice_state_change(
        &self,
        context: &ModuleContext,
        config: &GuildConfig,
        event: &VoiceStateChange,
    ) -> Result<Option<ModuleEventOutcome>> {
        if event.user_is_bot {
            return Ok(None);
        }
        if let Some(channel_id) = event.previous_channel_id
            && context
                .services
                .temporary_voice_channel(event.guild_id, channel_id)
                .await?
                .is_some()
        {
            let remaining = context
                .services
                .temporary_voice_member_leave(channel_id, event.user_id)
                .await?;
            if remaining == 0 {
                context
                    .services
                    .schedule_temporary_voice_delete(
                        event.guild_id,
                        channel_id,
                        config.modules.temporary_voice.delete_delay_seconds,
                    )
                    .await?;
            }
        }
        if let Some(channel_id) = event.channel_id
            && context
                .services
                .temporary_voice_channel(event.guild_id, channel_id)
                .await?
                .is_some()
        {
            context
                .services
                .temporary_voice_member_join(channel_id, event.user_id)
                .await?;
            context
                .services
                .cancel_temporary_voice_delete(event.guild_id, channel_id)
                .await?;
        }
        let lobby_id = config
            .modules
            .temporary_voice
            .lobby_channel_id
            .as_deref()
            .map(str::parse)
            .transpose()
            .context("temporary voice lobby is invalid")?;
        if event.channel_id != lobby_id {
            return Ok(None);
        }
        if let Some(existing) = context
            .services
            .temporary_voice_owner_channel(event.guild_id, event.user_id)
            .await?
        {
            context
                .discord
                .move_member(
                    event.guild_id,
                    event.user_id,
                    Some(existing.channel_id),
                    "Return to owned temporary voice channel",
                )
                .await?;
            return Ok(Some(outcome("temporary_voice.reused", existing.channel_id)));
        }
        let category_id = config
            .modules
            .temporary_voice
            .category_id
            .as_deref()
            .context("temporary voice category is required")?
            .parse()
            .context("temporary voice category is invalid")?;
        let name = config
            .modules
            .temporary_voice
            .default_name
            .replace("{username}", &event.username)
            .replace("{user_id}", &event.user_id.to_string());
        let channel_id = context
            .discord
            .create_temporary_voice_channel(
                event.guild_id,
                category_id,
                event.user_id,
                &name,
                config.modules.temporary_voice.user_limit,
                "Temporary voice lobby joined",
            )
            .await?;
        if let Err(error) = context
            .services
            .register_temporary_voice(
                event.guild_id,
                channel_id,
                event.user_id,
                &name,
                config.modules.temporary_voice.user_limit,
            )
            .await
        {
            let _ = context
                .discord
                .delete_channel(channel_id, "Temporary voice registration failed")
                .await;
            return Err(error);
        }
        context
            .services
            .temporary_voice_member_join(channel_id, event.user_id)
            .await?;
        context
            .discord
            .move_member(
                event.guild_id,
                event.user_id,
                Some(channel_id),
                "Move to new temporary voice channel",
            )
            .await?;
        Ok(Some(outcome("temporary_voice.created", channel_id)))
    }

    async fn on_command(
        &self,
        context: &ModuleContext,
        config: &GuildConfig,
        interaction: &CommandInteraction,
    ) -> Result<()> {
        if !config.modules.temporary_voice.owner_controls {
            bail!("temporary voice owner controls are disabled")
        }
        let channel = context
            .services
            .temporary_voice_owner_channel(interaction.guild_id, interaction.actor_id)
            .await?
            .context("you do not own an active temporary voice channel")?;
        let reason = format!("Temporary voice control by {}", interaction.actor_id);
        let response = match interaction.command_name.as_str() {
            "voice-rename" => {
                let name = interaction
                    .string("name")
                    .context("name is required")?
                    .trim();
                context
                    .discord
                    .update_voice_channel(
                        interaction.guild_id,
                        channel.channel_id,
                        Some(name),
                        None,
                        None,
                        &reason,
                    )
                    .await?;
                context
                    .services
                    .update_temporary_voice(
                        interaction.guild_id,
                        channel.channel_id,
                        None,
                        Some(name),
                        None,
                        None,
                    )
                    .await?;
                format!("Channel renamed to **{name}**.")
            }
            "voice-limit" => {
                let limit =
                    u8::try_from(interaction.integer("limit").context("limit is required")?)
                        .context("limit is invalid")?;
                context
                    .discord
                    .update_voice_channel(
                        interaction.guild_id,
                        channel.channel_id,
                        None,
                        Some(limit),
                        None,
                        &reason,
                    )
                    .await?;
                context
                    .services
                    .update_temporary_voice(
                        interaction.guild_id,
                        channel.channel_id,
                        None,
                        None,
                        Some(limit),
                        None,
                    )
                    .await?;
                if limit == 0 {
                    "User limit removed.".into()
                } else {
                    format!("User limit set to {limit}.")
                }
            }
            "voice-lock" | "voice-unlock" => {
                let locked = interaction.command_name == "voice-lock";
                context
                    .discord
                    .update_voice_channel(
                        interaction.guild_id,
                        channel.channel_id,
                        None,
                        None,
                        Some(locked),
                        &reason,
                    )
                    .await?;
                context
                    .services
                    .update_temporary_voice(
                        interaction.guild_id,
                        channel.channel_id,
                        None,
                        None,
                        None,
                        Some(locked),
                    )
                    .await?;
                if locked {
                    "Channel locked.".into()
                } else {
                    "Channel unlocked.".into()
                }
            }
            "voice-transfer" => {
                let owner_id = interaction.user("member").context("member is required")?;
                if owner_id == interaction.actor_id {
                    bail!("that member already owns the channel")
                };
                context
                    .discord
                    .transfer_voice_owner(
                        channel.channel_id,
                        interaction.actor_id,
                        owner_id,
                        &reason,
                    )
                    .await?;
                context
                    .services
                    .update_temporary_voice(
                        interaction.guild_id,
                        channel.channel_id,
                        Some(owner_id),
                        None,
                        None,
                        None,
                    )
                    .await?;
                format!("Ownership transferred to <@{owner_id}>.")
            }
            "voice-kick" => {
                let member_id = interaction.user("member").context("member is required")?;
                if member_id == interaction.actor_id {
                    bail!("owners cannot kick themselves")
                };
                context
                    .discord
                    .move_member(interaction.guild_id, member_id, None, &reason)
                    .await?;
                format!("Disconnected <@{member_id}>.")
            }
            _ => return Ok(()),
        };
        context
            .discord
            .respond_to_command(interaction, CommandResponse::ephemeral(response))
            .await?;
        Ok(())
    }

    async fn on_scheduled_tick(
        &self,
        context: &ModuleContext,
        config: &GuildConfig,
        event: &ScheduledTick,
    ) -> Result<Vec<ModuleEventOutcome>> {
        let guild_id = config.guild_id.parse().context("guild ID is invalid")?;
        let mut outcomes = Vec::new();
        for channel in context
            .services
            .due_temporary_voice_deletes(guild_id, event.now_unix)
            .await?
        {
            context
                .discord
                .delete_channel(channel.channel_id, "Temporary voice channel became empty")
                .await?;
            context
                .services
                .unregister_temporary_voice(guild_id, channel.channel_id)
                .await?;
            outcomes.push(outcome("temporary_voice.deleted", channel.channel_id));
        }
        Ok(outcomes)
    }
}

fn outcome(action: &str, channel_id: u64) -> ModuleEventOutcome {
    ModuleEventOutcome {
        action: action.into(),
        target_type: "channel".into(),
        target_id: Some(channel_id.to_string()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bot_module_api::ModuleServices;
    use bot_test_support::{DiscordCall, FakeDiscord, FakeModuleServices, ServiceCall};
    use std::sync::Arc;
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
    fn config() -> GuildConfig {
        let mut config = GuildConfig::for_guild("10");
        config.modules.temporary_voice.enabled = true;
        config.modules.temporary_voice.lobby_channel_id = Some("20".into());
        config.modules.temporary_voice.category_id = Some("30".into());
        config
    }
    #[tokio::test]
    async fn lobby_join_creates_registers_and_moves() {
        let (context, discord, services) = context();
        let event = VoiceStateChange {
            guild_id: 10,
            user_id: 40,
            previous_channel_id: None,
            channel_id: Some(20),
            previous_session_seconds: 0,
            previous_session_eligible: true,
            user_is_bot: false,
            username: "luna".into(),
        };
        let result = TemporaryVoiceModule
            .on_voice_state_change(&context, &config(), &event)
            .await
            .unwrap();
        assert_eq!(result.unwrap().action, "temporary_voice.created");
        assert!(discord.calls().iter().any(|call| matches!(
            call,
            DiscordCall::MoveMember {
                channel_id: Some(9001),
                ..
            }
        )));
        assert!(services.calls().iter().any(|call| matches!(
            call,
            ServiceCall::RegisterTemporaryVoice { owner_id: 40, .. }
        )));
    }
    #[tokio::test]
    async fn owner_can_lock_and_transfer() {
        let (context, discord, services) = context();
        services
            .register_temporary_voice(10, 50, 40, "luna", 0)
            .await
            .unwrap();
        let mut interaction = CommandInteraction {
            id: 1,
            token: "t".into(),
            guild_id: 10,
            channel_id: Some(60),
            actor_id: 40,
            actor_role_ids: vec![],
            actor_permissions: 0,
            command_name: "voice-lock".into(),
            options: Default::default(),
        };
        TemporaryVoiceModule
            .on_command(&context, &config(), &interaction)
            .await
            .unwrap();
        interaction.command_name = "voice-transfer".into();
        interaction
            .options
            .insert("member".into(), bot_discord::CommandValue::User(41));
        TemporaryVoiceModule
            .on_command(&context, &config(), &interaction)
            .await
            .unwrap();
        assert!(discord.calls().iter().any(|call| matches!(
            call,
            DiscordCall::UpdateVoiceChannel {
                locked: Some(true),
                ..
            }
        )));
        assert!(
            discord
                .calls()
                .iter()
                .any(|call| matches!(call, DiscordCall::TransferVoiceOwner { owner_id: 41, .. }))
        );
    }

    #[tokio::test]
    async fn empty_channels_are_deleted_by_the_scheduled_worker() {
        let (context, discord, services) = context();
        services
            .register_temporary_voice(10, 50, 40, "luna", 0)
            .await
            .unwrap();
        services.temporary_voice_member_join(50, 40).await.unwrap();
        let event = VoiceStateChange {
            guild_id: 10,
            user_id: 40,
            previous_channel_id: Some(50),
            channel_id: None,
            previous_session_seconds: 60,
            previous_session_eligible: true,
            user_is_bot: false,
            username: "luna".into(),
        };
        TemporaryVoiceModule
            .on_voice_state_change(&context, &config(), &event)
            .await
            .unwrap();
        let outcomes = TemporaryVoiceModule
            .on_scheduled_tick(&context, &config(), &ScheduledTick { now_unix: 100 })
            .await
            .unwrap();
        assert_eq!(outcomes[0].action, "temporary_voice.deleted");
        assert!(
            discord
                .calls()
                .iter()
                .any(|call| matches!(call, DiscordCall::DeleteChannel { channel_id: 50, .. }))
        );
        assert!(services.calls().iter().any(|call| matches!(
            call,
            ServiceCall::UnregisterTemporaryVoice { channel_id: 50, .. }
        )));
    }
}

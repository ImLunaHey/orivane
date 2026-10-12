use anyhow::{Context, Result};
use async_trait::async_trait;
use bot_core::GuildConfig;
use bot_discord::{
    CommandDefinition, CommandInteraction, CommandOptionDefinition, CommandOptionKind,
    CommandResponse,
};
use bot_module_api::{BotModule, MemberJoin, ModerationCaseInput, ModuleContext};

pub const COMMANDS: &[CommandDefinition] = &[
    CommandDefinition {
        name: "raid-lockdown",
        description: "Temporarily contain every new member",
        guild_only: true,
        default_permissions: Some(32),
        options: &[CommandOptionDefinition {
            name: "minutes",
            description: "Lockdown duration in minutes",
            kind: CommandOptionKind::Integer,
            required: true,
            min_value: Some(1),
            max_value: Some(1440),
            min_length: None,
            max_length: None,
        }],
    },
    CommandDefinition {
        name: "raid-unlock",
        description: "End the manual join lockdown",
        guild_only: true,
        default_permissions: Some(32),
        options: &[],
    },
];
#[derive(Default)]
pub struct AntiRaidModule;

#[async_trait]
impl BotModule for AntiRaidModule {
    fn name(&self) -> &'static str {
        "anti_raid"
    }
    fn enabled(&self, config: &GuildConfig) -> bool {
        config.modules.anti_raid.enabled
    }
    fn handles_member_join(&self) -> bool {
        true
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
        match interaction.command_name.as_str() {
            "raid-lockdown" => {
                let minutes = interaction
                    .integer("minutes")
                    .context("lockdown duration is required")? as u64;
                context
                    .services
                    .set_raid_lockdown(interaction.guild_id, minutes * 60)
                    .await?;
                context
                    .discord
                    .respond_to_command(
                        interaction,
                        CommandResponse::ephemeral(format!(
                            "Join lockdown enabled for {minutes} minute(s)."
                        )),
                    )
                    .await?;
            }
            "raid-unlock" => {
                context
                    .services
                    .clear_raid_lockdown(interaction.guild_id)
                    .await?;
                context
                    .discord
                    .respond_to_command(
                        interaction,
                        CommandResponse::ephemeral("Join lockdown disabled."),
                    )
                    .await?;
            }
            _ => {}
        }
        Ok(())
    }
    async fn on_member_join(
        &self,
        context: &ModuleContext,
        config: &GuildConfig,
        event: &MemberJoin,
    ) -> Result<()> {
        let policy = &config.modules.anti_raid;
        let count = context
            .services
            .record_raid_join(event.guild_id, event.user_id, policy.join_window_seconds)
            .await?;
        let forced = context
            .services
            .raid_lockdown_active(event.guild_id)
            .await?;
        let raid = forced || count >= u64::from(policy.join_threshold);
        let suspicious = event.is_bot
            || event.account_age_seconds < u64::from(policy.minimum_account_age_days) * 86_400
            || (policy.require_avatar && !event.has_avatar);
        if !raid || (!forced && !suspicious && !policy.punish_all_during_raid) {
            return Ok(());
        }
        let reason = format!(
            "Anti-raid: {count} joins in {} seconds",
            policy.join_window_seconds
        );
        match policy.action.as_str() {
            "quarantine" => {
                context
                    .discord
                    .add_member_role(
                        event.guild_id,
                        event.user_id,
                        policy
                            .quarantine_role_id
                            .as_deref()
                            .context("quarantine role missing")?
                            .parse()
                            .context("invalid quarantine role")?,
                    )
                    .await?
            }
            "timeout" => {
                context
                    .discord
                    .timeout_member(
                        event.guild_id,
                        event.user_id,
                        policy.timeout_seconds.into(),
                        &reason,
                    )
                    .await?
            }
            "kick" => {
                context
                    .discord
                    .kick_member(event.guild_id, event.user_id, &reason)
                    .await?
            }
            "ban" => {
                context
                    .discord
                    .ban_member(event.guild_id, event.user_id, 0, &reason)
                    .await?
            }
            _ => {}
        }
        context
            .services
            .create_moderation_case(ModerationCaseInput {
                guild_id: event.guild_id,
                moderator_id: 0,
                member_id: event.user_id,
                action: format!("anti_raid_{}", policy.action),
                reason: reason.clone(),
            })
            .await?;
        if let Some(channel) = &policy.alert_channel_id {
            context
                .discord
                .send_message(
                    channel.parse().context("invalid alert channel")?,
                    &format!(
                        "🚨 **Raid protection triggered** · {reason} · action `{}` · member <@{}>",
                        policy.action, event.user_id
                    ),
                )
                .await?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bot_discord::CommandValue;
    use bot_module_api::ModuleServices;
    use bot_test_support::{DiscordCall, FakeDiscord, FakeModuleServices};
    use std::{collections::BTreeMap, sync::Arc};

    #[tokio::test]
    async fn quarantines_a_risky_member_when_the_join_burst_triggers() {
        let discord = FakeDiscord::new();
        let context = ModuleContext {
            discord: Arc::new(discord.clone()),
            services: Arc::new(FakeModuleServices::default()),
        };
        let mut config = GuildConfig::for_guild("10");
        config.modules.anti_raid.enabled = true;
        config.modules.anti_raid.join_threshold = 2;
        config.modules.anti_raid.join_window_seconds = 60;
        config.modules.anti_raid.minimum_account_age_days = 7;
        config.modules.anti_raid.quarantine_role_id = Some("90".into());
        config.modules.anti_raid.alert_channel_id = Some("80".into());
        let module = AntiRaidModule;
        for user_id in [40, 41] {
            module
                .on_member_join(
                    &context,
                    &config,
                    &MemberJoin {
                        guild_id: 10,
                        guild_name: "Guild".into(),
                        user_id,
                        username: "new".into(),
                        display_name: "New".into(),
                        account_age_seconds: 60,
                        has_avatar: false,
                        is_bot: false,
                    },
                )
                .await
                .unwrap();
        }
        assert_eq!(discord.calls(), vec![
            DiscordCall::AddMemberRole { guild_id: 10, user_id: 41, role_id: 90 },
            DiscordCall::SendMessage { channel_id: 80, content: "🚨 **Raid protection triggered** · Anti-raid: 2 joins in 60 seconds · action `quarantine` · member <@41>".into() }
        ]);
    }

    #[tokio::test]
    async fn manual_lockdown_is_shared_through_services() {
        let discord = FakeDiscord::new();
        let services = FakeModuleServices::default();
        let context = ModuleContext {
            discord: Arc::new(discord.clone()),
            services: Arc::new(services.clone()),
        };
        let command = CommandInteraction {
            id: 1,
            token: "token".into(),
            guild_id: 10,
            channel_id: Some(20),
            actor_id: 30,
            actor_role_ids: vec![],
            actor_permissions: u64::MAX,
            command_name: "raid-lockdown".into(),
            options: BTreeMap::from([("minutes".into(), CommandValue::Integer(15))]),
        };
        AntiRaidModule
            .on_command(&context, &GuildConfig::for_guild("10"), &command)
            .await
            .unwrap();
        assert!(services.raid_lockdown_active(10).await.unwrap());
        assert_eq!(
            discord.calls(),
            vec![DiscordCall::RespondToCommand {
                interaction_id: 1,
                response: CommandResponse::ephemeral("Join lockdown enabled for 15 minute(s).")
            }]
        );
    }
}

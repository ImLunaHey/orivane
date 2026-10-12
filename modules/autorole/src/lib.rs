use anyhow::{Context, Result};
use async_trait::async_trait;
use bot_core::GuildConfig;
use bot_discord::{
    CommandDefinition, CommandInteraction, CommandOptionDefinition, CommandOptionKind,
    CommandResponse,
};
use bot_module_api::{BotModule, MemberJoin, ModuleContext, ModuleEventOutcome, ScheduledTick};

pub struct AutoroleModule;

pub const COMMANDS: &[CommandDefinition] = &[CommandDefinition {
    name: "role",
    description: "Add or remove a self-assignable server role",
    guild_only: true,
    default_permissions: None,
    options: &[CommandOptionDefinition {
        name: "role",
        description: "The role to toggle",
        kind: CommandOptionKind::Role,
        required: true,
        min_value: None,
        max_value: None,
        min_length: None,
        max_length: None,
    }],
}];

#[async_trait]
impl BotModule for AutoroleModule {
    fn name(&self) -> &'static str {
        "autorole"
    }
    fn enabled(&self, config: &GuildConfig) -> bool {
        config.modules.autorole.enabled
    }
    fn handles_member_join(&self) -> bool {
        true
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
        if interaction.command_name != "role" {
            return Ok(());
        }
        let role_id = interaction.role("role").context("role is required")?;
        let allowed = config
            .modules
            .autorole
            .joinable_role_ids
            .iter()
            .any(|id| id == &role_id.to_string());
        if !allowed {
            context
                .discord
                .respond_to_command(
                    interaction,
                    CommandResponse::ephemeral("That role is not self-assignable."),
                )
                .await?;
            return Ok(());
        }

        if interaction.actor_role_ids.contains(&role_id) {
            context
                .discord
                .remove_member_role(
                    interaction.guild_id,
                    interaction.actor_id,
                    role_id,
                    "Self-assignable role removed",
                )
                .await?;
            context
                .discord
                .respond_to_command(
                    interaction,
                    CommandResponse::ephemeral(format!("Removed <@&{role_id}>.")),
                )
                .await?;
            return Ok(());
        }

        if config.modules.autorole.single_joinable_role {
            for existing in config
                .modules
                .autorole
                .joinable_role_ids
                .iter()
                .filter_map(|id| id.parse::<u64>().ok())
                .filter(|id| interaction.actor_role_ids.contains(id))
            {
                context
                    .discord
                    .remove_member_role(
                        interaction.guild_id,
                        interaction.actor_id,
                        existing,
                        "Replaced self-assignable role",
                    )
                    .await?;
            }
        }
        context
            .discord
            .add_member_role(interaction.guild_id, interaction.actor_id, role_id)
            .await?;
        context
            .discord
            .respond_to_command(
                interaction,
                CommandResponse::ephemeral(format!("Added <@&{role_id}>.")),
            )
            .await?;
        Ok(())
    }

    async fn on_member_join(
        &self,
        context: &ModuleContext,
        config: &GuildConfig,
        event: &MemberJoin,
    ) -> Result<()> {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        for role_id in &config.modules.autorole.role_ids {
            let role_id = role_id.parse::<u64>().context("autorole ID is invalid")?;
            if config.modules.autorole.delay_seconds == 0 {
                context
                    .discord
                    .add_member_role(event.guild_id, event.user_id, role_id)
                    .await?;
            } else {
                context
                    .services
                    .schedule_autorole_job(
                        event.guild_id,
                        event.user_id,
                        role_id,
                        "add",
                        now + u64::from(config.modules.autorole.delay_seconds),
                    )
                    .await?;
            }
        }
        for role_id in &config.modules.autorole.remove_role_ids {
            let role_id = role_id
                .parse::<u64>()
                .context("autorole removal ID is invalid")?;
            if config.modules.autorole.remove_delay_seconds == 0 {
                context
                    .discord
                    .remove_member_role(
                        event.guild_id,
                        event.user_id,
                        role_id,
                        "Configured autorole removal",
                    )
                    .await?;
            } else {
                context
                    .services
                    .schedule_autorole_job(
                        event.guild_id,
                        event.user_id,
                        role_id,
                        "remove",
                        now + u64::from(config.modules.autorole.remove_delay_seconds),
                    )
                    .await?;
            }
        }
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
            .claim_due_autorole_jobs(guild_id, tick.now_unix, 50)
            .await?;
        let mut outcomes = Vec::new();
        for job in jobs {
            let delivery = match job.action.as_str() {
                "add" => {
                    context
                        .discord
                        .add_member_role(guild_id, job.user_id, job.role_id)
                        .await
                }
                "remove" => {
                    context
                        .discord
                        .remove_member_role(
                            guild_id,
                            job.user_id,
                            job.role_id,
                            "Configured delayed autorole removal",
                        )
                        .await
                }
                _ => continue,
            };
            let error = delivery.as_ref().err().map(ToString::to_string);
            let retry_delay = 30_u64.saturating_mul(2_u64.pow(u32::from(job.attempts.min(5))));
            let terminal = context
                .services
                .finish_autorole_job(
                    guild_id,
                    job.id,
                    delivery.is_ok(),
                    error.as_deref(),
                    tick.now_unix.saturating_add(retry_delay),
                )
                .await?;
            if delivery.is_ok() || terminal {
                outcomes.push(ModuleEventOutcome {
                    action: if delivery.is_ok() {
                        format!("autorole.{}.completed", job.action)
                    } else {
                        format!("autorole.{}.failed", job.action)
                    },
                    target_type: "member".into(),
                    target_id: Some(job.user_id.to_string()),
                });
            }
        }
        Ok(outcomes)
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use super::*;
    use bot_discord::CommandValue;
    use bot_module_api::AutoroleJobResult;
    use bot_test_support::{DiscordCall, FakeDiscord, FakeModuleServices, ServiceCall};
    use std::collections::BTreeMap;

    #[tokio::test]
    async fn assigns_every_configured_role() {
        let discord = FakeDiscord::new();
        let context = ModuleContext {
            discord: Arc::new(discord.clone()),
            services: Arc::new(FakeModuleServices::default()),
        };
        let mut config = GuildConfig::for_guild("10");
        config.modules.autorole.enabled = true;
        config.modules.autorole.role_ids = vec!["40".to_owned(), "41".to_owned()];
        AutoroleModule
            .on_member_join(
                &context,
                &config,
                &MemberJoin {
                    guild_id: 10,
                    guild_name: "Guild".to_owned(),
                    user_id: 30,
                    username: "member".to_owned(),
                    display_name: "Member".to_owned(),
                    account_age_seconds: 86_400 * 30,
                    has_avatar: true,
                    is_bot: false,
                },
            )
            .await
            .unwrap();
        assert_eq!(
            discord.calls(),
            vec![
                DiscordCall::AddMemberRole {
                    guild_id: 10,
                    user_id: 30,
                    role_id: 40
                },
                DiscordCall::AddMemberRole {
                    guild_id: 10,
                    user_id: 30,
                    role_id: 41
                }
            ]
        );
    }

    #[tokio::test]
    async fn replaces_an_exclusive_self_assignable_role() {
        let discord = FakeDiscord::new();
        let context = ModuleContext {
            discord: Arc::new(discord.clone()),
            services: Arc::new(FakeModuleServices::default()),
        };
        let mut config = GuildConfig::for_guild("10");
        config.modules.autorole.enabled = true;
        config.modules.autorole.joinable_role_ids = vec!["40".into(), "41".into()];
        config.modules.autorole.single_joinable_role = true;
        let interaction = CommandInteraction {
            id: 1,
            token: "test".into(),
            guild_id: 10,
            channel_id: Some(20),
            actor_id: 30,
            actor_role_ids: vec![40],
            actor_permissions: u64::MAX,
            command_name: "role".into(),
            options: BTreeMap::from([("role".into(), CommandValue::Role(41))]),
        };
        AutoroleModule
            .on_command(&context, &config, &interaction)
            .await
            .unwrap();
        assert_eq!(
            discord.calls(),
            vec![
                DiscordCall::RemoveMemberRole {
                    guild_id: 10,
                    user_id: 30,
                    role_id: 40,
                    reason: "Replaced self-assignable role".into()
                },
                DiscordCall::AddMemberRole {
                    guild_id: 10,
                    user_id: 30,
                    role_id: 41
                },
                DiscordCall::RespondToCommand {
                    interaction_id: 1,
                    response: CommandResponse::ephemeral("Added <@&41>.")
                },
            ]
        );
    }

    #[tokio::test]
    async fn delayed_roles_are_durable_and_processed_by_the_scheduler() {
        let discord = FakeDiscord::new();
        let services = FakeModuleServices::default();
        let context = ModuleContext {
            discord: Arc::new(discord.clone()),
            services: Arc::new(services.clone()),
        };
        let mut config = GuildConfig::for_guild("10");
        config.modules.autorole.enabled = true;
        config.modules.autorole.role_ids = vec!["40".into()];
        config.modules.autorole.delay_seconds = 60;
        AutoroleModule
            .on_member_join(
                &context,
                &config,
                &MemberJoin {
                    guild_id: 10,
                    guild_name: "Guild".into(),
                    user_id: 30,
                    username: "member".into(),
                    display_name: "Member".into(),
                    account_age_seconds: 86_400,
                    has_avatar: true,
                    is_bot: false,
                },
            )
            .await
            .unwrap();
        assert!(matches!(
            services.calls().first(),
            Some(ServiceCall::ScheduleAutoroleJob { guild_id: 10, user_id: 30, role_id: 40, action, .. }) if action == "add"
        ));
        assert!(discord.calls().is_empty());

        services.set_due_autorole_jobs(vec![AutoroleJobResult {
            id: 7,
            user_id: 30,
            role_id: 40,
            action: "add".into(),
            attempts: 1,
        }]);
        let outcomes = AutoroleModule
            .on_scheduled_tick(&context, &config, &ScheduledTick { now_unix: 1_000 })
            .await
            .unwrap();
        assert_eq!(outcomes[0].action, "autorole.add.completed");
        assert_eq!(
            discord.calls(),
            vec![DiscordCall::AddMemberRole {
                guild_id: 10,
                user_id: 30,
                role_id: 40,
            }]
        );
        assert!(services.calls().iter().any(|call| matches!(
            call,
            ServiceCall::FinishAutoroleJob {
                job_id: 7,
                completed: true,
                ..
            }
        )));
    }
}

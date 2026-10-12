use anyhow::{Context, Result};
use async_trait::async_trait;
use bot_core::GuildConfig;
use bot_module_api::{BotModule, ModuleContext, ModuleEventOutcome, ReactionAdd};

pub struct ReactionRolesModule;

#[async_trait]
impl BotModule for ReactionRolesModule {
    fn name(&self) -> &'static str {
        "reaction_roles"
    }

    fn enabled(&self, config: &GuildConfig) -> bool {
        config.modules.reaction_roles.enabled
    }

    fn handles_reaction_add(&self) -> bool {
        true
    }

    fn handles_reaction_remove(&self) -> bool {
        true
    }

    async fn on_reaction_add(
        &self,
        context: &ModuleContext,
        config: &GuildConfig,
        event: &ReactionAdd,
    ) -> Result<Option<ModuleEventOutcome>> {
        let Some(role_id) = mapped_role(config, event) else {
            return Ok(None);
        };
        context
            .discord
            .add_member_role(event.guild_id, event.reactor_id, role_id)
            .await?;
        Ok(Some(role_outcome("reaction_roles.role_added", role_id)))
    }

    async fn on_reaction_remove(
        &self,
        context: &ModuleContext,
        config: &GuildConfig,
        event: &ReactionAdd,
    ) -> Result<Option<ModuleEventOutcome>> {
        let Some(role_id) = mapped_role(config, event) else {
            return Ok(None);
        };
        context
            .discord
            .remove_member_role(
                event.guild_id,
                event.reactor_id,
                role_id,
                "Reaction role removed",
            )
            .await?;
        Ok(Some(role_outcome("reaction_roles.role_removed", role_id)))
    }
}

fn mapped_role(config: &GuildConfig, event: &ReactionAdd) -> Option<u64> {
    config
        .modules
        .reaction_roles
        .bindings
        .iter()
        .find(|binding| {
            binding.message_id == event.message_id.to_string() && binding.emoji == event.emoji
        })
        .map(|binding| {
            binding
                .role_id
                .parse::<u64>()
                .context("reaction role ID is invalid")
        })
        .transpose()
        .ok()
        .flatten()
}

fn role_outcome(action: &str, role_id: u64) -> ModuleEventOutcome {
    ModuleEventOutcome {
        action: action.to_owned(),
        target_type: "role".to_owned(),
        target_id: Some(role_id.to_string()),
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use bot_core::ReactionRoleBinding;
    use bot_test_support::{DiscordCall, FakeDiscord, FakeModuleServices};

    use super::*;

    fn setup() -> (FakeDiscord, ModuleContext, GuildConfig, ReactionAdd) {
        let discord = FakeDiscord::new();
        let context = ModuleContext {
            discord: Arc::new(discord.clone()),
            services: Arc::new(FakeModuleServices::default()),
        };
        let mut config = GuildConfig::for_guild("10");
        config.modules.reaction_roles.enabled = true;
        config.modules.reaction_roles.bindings = vec![ReactionRoleBinding {
            message_id: "30".into(),
            emoji: "🌙".into(),
            role_id: "40".into(),
        }];
        let event = ReactionAdd {
            guild_id: 10,
            channel_id: 20,
            message_id: 30,
            reactor_id: 50,
            message_author_id: None,
            emoji: "🌙".into(),
        };
        (discord, context, config, event)
    }

    #[tokio::test]
    async fn mirrors_a_reaction_as_a_role() {
        let (discord, context, config, event) = setup();
        let module = ReactionRolesModule;
        module
            .on_reaction_add(&context, &config, &event)
            .await
            .unwrap();
        module
            .on_reaction_remove(&context, &config, &event)
            .await
            .unwrap();
        assert_eq!(
            discord.calls(),
            vec![
                DiscordCall::AddMemberRole {
                    guild_id: 10,
                    user_id: 50,
                    role_id: 40
                },
                DiscordCall::RemoveMemberRole {
                    guild_id: 10,
                    user_id: 50,
                    role_id: 40,
                    reason: "Reaction role removed".into()
                },
            ]
        );
    }
}

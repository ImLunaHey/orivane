use anyhow::{Context, Result};
use async_trait::async_trait;
use bot_core::{GuildConfig, ReactionRolePanel};
use bot_discord::{CommandResponse, ComponentInteraction};
use bot_module_api::{BotModule, ModuleContext, ModuleEventOutcome, ReactionAdd};

pub struct ReactionRolesModule;
const BUTTON_PREFIX: &str = "orivane:rr:";
const SELECT_PREFIX: &str = "orivane:rr-select:";

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

    fn handles_component(&self, custom_id: &str) -> bool {
        custom_id.starts_with(BUTTON_PREFIX) || custom_id.starts_with(SELECT_PREFIX)
    }

    async fn on_component(
        &self,
        context: &ModuleContext,
        config: &GuildConfig,
        interaction: &ComponentInteraction,
    ) -> Result<Option<ModuleEventOutcome>> {
        let (panel_id, option_ids) =
            if let Some(value) = interaction.custom_id.strip_prefix(BUTTON_PREFIX) {
                let (panel, option) = value
                    .split_once(':')
                    .context("invalid reaction-role button")?;
                (panel, vec![option.to_owned()])
            } else {
                (
                    interaction
                        .custom_id
                        .strip_prefix(SELECT_PREFIX)
                        .context("invalid reaction-role selector")?,
                    interaction.values.clone(),
                )
            };
        let panel = config
            .modules
            .reaction_roles
            .panels
            .iter()
            .find(|panel| panel.enabled && panel.id == panel_id)
            .context("reaction-role panel no longer exists")?;
        let required = panel
            .required_role_ids
            .iter()
            .filter_map(|role| role.parse::<u64>().ok())
            .collect::<Vec<_>>();
        let blocked = panel
            .blocked_role_ids
            .iter()
            .filter_map(|role| role.parse::<u64>().ok())
            .collect::<Vec<_>>();
        if (!required.is_empty()
            && !interaction
                .actor_role_ids
                .iter()
                .any(|role| required.contains(role)))
            || interaction
                .actor_role_ids
                .iter()
                .any(|role| blocked.contains(role))
        {
            context
                .discord
                .respond_to_component(
                    interaction,
                    CommandResponse::ephemeral("You do not meet this role panel's requirements."),
                )
                .await?;
            return Ok(None);
        }
        apply_panel_selection(context, interaction, panel, &option_ids).await?;
        context
            .discord
            .respond_to_component(
                interaction,
                CommandResponse::ephemeral("Your roles were updated."),
            )
            .await?;
        Ok(Some(ModuleEventOutcome {
            action: "reaction_roles.selection.updated".into(),
            target_type: "reaction_role_panel".into(),
            target_id: Some(panel.id.clone()),
        }))
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

async fn apply_panel_selection(
    context: &ModuleContext,
    interaction: &ComponentInteraction,
    panel: &ReactionRolePanel,
    option_ids: &[String],
) -> Result<()> {
    let selected = panel
        .options
        .iter()
        .filter(|option| option_ids.contains(&option.id))
        .map(|option| {
            option
                .role_id
                .parse::<u64>()
                .context("reaction role ID is invalid")
        })
        .collect::<Result<Vec<_>>>()?;
    let panel_roles = panel
        .options
        .iter()
        .map(|option| {
            option
                .role_id
                .parse::<u64>()
                .context("reaction role ID is invalid")
        })
        .collect::<Result<Vec<_>>>()?;
    let held = &interaction.actor_role_ids;

    if interaction.custom_id.starts_with(SELECT_PREFIX) && panel.mode == "toggle" {
        for role in panel_roles
            .iter()
            .filter(|role| held.contains(role) && !selected.contains(role))
        {
            context
                .discord
                .remove_member_role(
                    interaction.guild_id,
                    interaction.actor_id,
                    *role,
                    "Reaction-role selection changed",
                )
                .await?;
        }
    }
    if panel.exclusive && panel.mode != "remove_only" {
        for role in panel_roles
            .iter()
            .filter(|role| held.contains(role) && !selected.contains(role))
        {
            context
                .discord
                .remove_member_role(
                    interaction.guild_id,
                    interaction.actor_id,
                    *role,
                    "Exclusive reaction role changed",
                )
                .await?;
        }
    }
    for role in selected {
        let has_role = held.contains(&role);
        let should_remove = panel.mode == "remove_only"
            || (panel.mode == "toggle"
                && interaction.custom_id.starts_with(BUTTON_PREFIX)
                && has_role);
        if should_remove {
            if has_role {
                context
                    .discord
                    .remove_member_role(
                        interaction.guild_id,
                        interaction.actor_id,
                        role,
                        "Reaction role removed",
                    )
                    .await?;
            }
        } else if !has_role {
            context
                .discord
                .add_member_role(interaction.guild_id, interaction.actor_id, role)
                .await?;
        }
    }
    Ok(())
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

    use bot_core::{ReactionRoleBinding, ReactionRoleOption, ReactionRolePanel};
    use bot_discord::ComponentInteraction;
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

    fn panel_config(component_type: &str, mode: &str, exclusive: bool) -> GuildConfig {
        let mut config = GuildConfig::for_guild("10");
        config.modules.reaction_roles.enabled = true;
        config.modules.reaction_roles.panels = vec![ReactionRolePanel {
            id: "colors".into(),
            enabled: true,
            name: "Colors".into(),
            title: "Choose a color".into(),
            description: String::new(),
            channel_id: "20".into(),
            message_id: Some("30".into()),
            component_type: component_type.into(),
            mode: mode.into(),
            exclusive,
            required_role_ids: vec![],
            blocked_role_ids: vec![],
            options: vec![
                ReactionRoleOption {
                    id: "red".into(),
                    label: "Red".into(),
                    description: String::new(),
                    emoji: "🔴".into(),
                    role_id: "40".into(),
                },
                ReactionRoleOption {
                    id: "blue".into(),
                    label: "Blue".into(),
                    description: String::new(),
                    emoji: "🔵".into(),
                    role_id: "41".into(),
                },
            ],
        }];
        config
    }

    #[tokio::test]
    async fn exclusive_button_replaces_the_previous_role() {
        let discord = FakeDiscord::new();
        let context = ModuleContext {
            discord: Arc::new(discord.clone()),
            services: Arc::new(FakeModuleServices::default()),
        };
        let interaction = ComponentInteraction {
            id: 1,
            token: "x".into(),
            guild_id: 10,
            channel_id: Some(20),
            actor_id: 50,
            actor_role_ids: vec![40],
            custom_id: "orivane:rr:colors:blue".into(),
            values: vec![],
        };
        ReactionRolesModule
            .on_component(
                &context,
                &panel_config("buttons", "toggle", true),
                &interaction,
            )
            .await
            .unwrap();
        assert!(
            discord
                .calls()
                .iter()
                .any(|call| matches!(call, DiscordCall::RemoveMemberRole { role_id: 40, .. }))
        );
        assert!(
            discord
                .calls()
                .iter()
                .any(|call| matches!(call, DiscordCall::AddMemberRole { role_id: 41, .. }))
        );
    }

    #[tokio::test]
    async fn dropdown_synchronizes_all_roles_in_the_panel() {
        let discord = FakeDiscord::new();
        let context = ModuleContext {
            discord: Arc::new(discord.clone()),
            services: Arc::new(FakeModuleServices::default()),
        };
        let interaction = ComponentInteraction {
            id: 1,
            token: "x".into(),
            guild_id: 10,
            channel_id: Some(20),
            actor_id: 50,
            actor_role_ids: vec![40],
            custom_id: "orivane:rr-select:colors".into(),
            values: vec!["blue".into()],
        };
        ReactionRolesModule
            .on_component(
                &context,
                &panel_config("select", "toggle", false),
                &interaction,
            )
            .await
            .unwrap();
        assert!(
            discord
                .calls()
                .iter()
                .any(|call| matches!(call, DiscordCall::RemoveMemberRole { role_id: 40, .. }))
        );
        assert!(
            discord
                .calls()
                .iter()
                .any(|call| matches!(call, DiscordCall::AddMemberRole { role_id: 41, .. }))
        );
    }

    #[tokio::test]
    async fn rejects_members_who_do_not_meet_panel_role_policy() {
        let discord = FakeDiscord::new();
        let context = ModuleContext {
            discord: Arc::new(discord.clone()),
            services: Arc::new(FakeModuleServices::default()),
        };
        let mut config = panel_config("buttons", "add_only", false);
        config.modules.reaction_roles.panels[0].required_role_ids = vec!["99".into()];
        let interaction = ComponentInteraction {
            id: 1,
            token: "x".into(),
            guild_id: 10,
            channel_id: Some(20),
            actor_id: 50,
            actor_role_ids: vec![],
            custom_id: "orivane:rr:colors:red".into(),
            values: vec![],
        };
        let outcome = ReactionRolesModule
            .on_component(&context, &config, &interaction)
            .await
            .unwrap();
        assert!(outcome.is_none());
        assert!(
            !discord
                .calls()
                .iter()
                .any(|call| matches!(call, DiscordCall::AddMemberRole { .. }))
        );
        assert!(discord.calls().iter().any(|call| matches!(
            call,
            DiscordCall::RespondToComponent { response, .. }
                if response.content.contains("requirements")
        )));
    }
}

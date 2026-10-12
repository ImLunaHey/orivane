use anyhow::{Context, Result};
use async_trait::async_trait;
use bot_core::GuildConfig;
use bot_discord::{
    CommandDefinition, CommandResponse, ComponentInteraction, MessageButton, MessageButtonStyle,
};
use bot_module_api::{BotModule, MemberJoin, ModuleContext, ModuleEventOutcome};

pub struct VerificationModule;
pub const COMMANDS: &[CommandDefinition] = &[];
const PREFIX: &str = "orivane:verification:";

#[async_trait]
impl BotModule for VerificationModule {
    fn name(&self) -> &'static str {
        "verification"
    }
    fn enabled(&self, config: &GuildConfig) -> bool {
        config.modules.verification.enabled
    }
    fn commands(&self) -> &'static [CommandDefinition] {
        COMMANDS
    }
    fn handles_member_join(&self) -> bool {
        true
    }
    fn handles_component(&self, custom_id: &str) -> bool {
        custom_id.starts_with(PREFIX)
    }

    async fn on_member_join(
        &self,
        context: &ModuleContext,
        config: &GuildConfig,
        event: &MemberJoin,
    ) -> Result<()> {
        if event.is_bot {
            return Ok(());
        }
        let settings = &config.modules.verification;
        if let Some(role) = settings.pending_role_id.as_deref() {
            context
                .discord
                .add_member_role(
                    event.guild_id,
                    event.user_id,
                    role.parse().context("invalid pending role")?,
                )
                .await?;
        }
        let eligible = event.account_age_seconds
            >= u64::from(settings.minimum_account_age_days) * 86_400
            && (!settings.require_avatar || event.has_avatar);
        let channel = settings
            .channel_id
            .as_deref()
            .context("verification channel missing")?
            .parse()
            .context("invalid verification channel")?;
        if eligible {
            let content = render(&settings.message, event);
            context
                .discord
                .send_message_with_buttons(
                    channel,
                    &content,
                    &[MessageButton {
                        custom_id: format!("{PREFIX}{}", event.user_id),
                        label: "Verify".into(),
                        style: MessageButtonStyle::Success,
                    }],
                )
                .await?;
        } else {
            context
                .discord
                .send_message(
                    channel,
                    &format!("<@{}> {}", event.user_id, settings.rejection_message),
                )
                .await?;
        }
        Ok(())
    }

    async fn on_component(
        &self,
        context: &ModuleContext,
        config: &GuildConfig,
        interaction: &ComponentInteraction,
    ) -> Result<Option<ModuleEventOutcome>> {
        let expected: u64 = interaction
            .custom_id
            .strip_prefix(PREFIX)
            .context("invalid verification action")?
            .parse()
            .context("invalid verification member")?;
        if expected != interaction.actor_id {
            context
                .discord
                .respond_to_component(
                    interaction,
                    CommandResponse::ephemeral(
                        "This verification prompt belongs to another member.",
                    ),
                )
                .await?;
            return Ok(None);
        }
        let settings = &config.modules.verification;
        let verified: u64 = settings
            .verified_role_id
            .as_deref()
            .context("verified role missing")?
            .parse()
            .context("invalid verified role")?;
        context
            .discord
            .add_member_role(interaction.guild_id, interaction.actor_id, verified)
            .await?;
        if let Some(pending) = settings.pending_role_id.as_deref() {
            context
                .discord
                .remove_member_role(
                    interaction.guild_id,
                    interaction.actor_id,
                    pending.parse().context("invalid pending role")?,
                    "Member completed verification",
                )
                .await?;
        }
        context
            .discord
            .respond_to_component(
                interaction,
                CommandResponse::ephemeral(
                    "Verification complete. You now have access to the server.",
                ),
            )
            .await?;
        Ok(Some(ModuleEventOutcome {
            action: "verification.completed".into(),
            target_type: "member".into(),
            target_id: Some(interaction.actor_id.to_string()),
        }))
    }
}

fn render(template: &str, event: &MemberJoin) -> String {
    template
        .replace("{user}", &format!("<@{}>", event.user_id))
        .replace("{username}", &event.username)
        .replace("{server}", &event.guild_name)
}

#[cfg(test)]
mod tests {
    use super::*;
    use bot_test_support::{DiscordCall, FakeDiscord, FakeModuleServices};
    use std::sync::Arc;
    fn setup() -> (ModuleContext, FakeDiscord, GuildConfig) {
        let discord = FakeDiscord::new();
        let mut config = GuildConfig::for_guild("10");
        config.modules.verification.enabled = true;
        config.modules.verification.channel_id = Some("20".into());
        config.modules.verification.verified_role_id = Some("30".into());
        config.modules.verification.pending_role_id = Some("31".into());
        (
            ModuleContext {
                discord: Arc::new(discord.clone()),
                services: Arc::new(FakeModuleServices::default()),
            },
            discord,
            config,
        )
    }
    #[tokio::test]
    async fn eligible_member_receives_prompt_and_can_verify() {
        let (ctx, discord, config) = setup();
        let event = MemberJoin {
            guild_id: 10,
            guild_name: "Lab".into(),
            user_id: 40,
            username: "luna".into(),
            display_name: "Luna".into(),
            account_age_seconds: 86400,
            has_avatar: true,
            is_bot: false,
        };
        VerificationModule
            .on_member_join(&ctx, &config, &event)
            .await
            .unwrap();
        assert!(discord.calls().iter().any(|call|matches!(call,DiscordCall::SendMessageWithButtons{channel_id:20,buttons,..} if buttons[0].custom_id=="orivane:verification:40")));
        let interaction = ComponentInteraction {
            id: 1,
            token: "t".into(),
            guild_id: 10,
            channel_id: Some(20),
            actor_id: 40,
            actor_role_ids: vec![31],
            custom_id: "orivane:verification:40".into(),
            values: vec![],
        };
        assert!(
            VerificationModule
                .on_component(&ctx, &config, &interaction)
                .await
                .unwrap()
                .is_some()
        );
        assert!(
            discord
                .calls()
                .iter()
                .any(|call| matches!(call, DiscordCall::AddMemberRole { role_id: 30, .. }))
        );
        assert!(
            discord
                .calls()
                .iter()
                .any(|call| matches!(call, DiscordCall::RemoveMemberRole { role_id: 31, .. }))
        );
    }
    #[tokio::test]
    async fn risky_member_is_rejected_without_a_button() {
        let (ctx, discord, mut config) = setup();
        config.modules.verification.minimum_account_age_days = 7;
        let event = MemberJoin {
            guild_id: 10,
            guild_name: "Lab".into(),
            user_id: 40,
            username: "new".into(),
            display_name: "New".into(),
            account_age_seconds: 60,
            has_avatar: false,
            is_bot: false,
        };
        VerificationModule
            .on_member_join(&ctx, &config, &event)
            .await
            .unwrap();
        assert!(
            !discord
                .calls()
                .iter()
                .any(|call| matches!(call, DiscordCall::SendMessageWithButtons { .. }))
        );
    }
}

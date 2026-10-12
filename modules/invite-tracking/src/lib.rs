use anyhow::Result;
use async_trait::async_trait;
use bot_core::GuildConfig;
use bot_discord::CommandDefinition;
use bot_module_api::{
    BotModule, InviteUsageInput, MemberJoin, ModuleContext, ModuleEventOutcome, ScheduledTick,
};
pub struct InviteTrackingModule;
pub const COMMANDS: &[CommandDefinition] = &[];
#[async_trait]
impl BotModule for InviteTrackingModule {
    fn name(&self) -> &'static str {
        "invite_tracking"
    }
    fn enabled(&self, c: &GuildConfig) -> bool {
        c.modules.invite_tracking.enabled
    }
    fn handles_member_join(&self) -> bool {
        true
    }
    fn handles_scheduled_tick(&self) -> bool {
        true
    }
    async fn on_scheduled_tick(
        &self,
        ctx: &ModuleContext,
        c: &GuildConfig,
        _event: &ScheduledTick,
    ) -> Result<Vec<ModuleEventOutcome>> {
        let guild_id = c.guild_id.parse()?;
        if ctx.services.invite_snapshot_ready(guild_id).await? {
            return Ok(Vec::new());
        }
        let invites = ctx
            .discord
            .guild_invites(guild_id)
            .await?
            .into_iter()
            .map(|invite| InviteUsageInput {
                code: invite.code,
                inviter_id: invite.inviter_id,
                uses: invite.uses,
            })
            .collect::<Vec<_>>();
        ctx.services
            .ensure_invite_snapshot(guild_id, &invites)
            .await?;
        Ok(Vec::new())
    }
    async fn on_member_join(
        &self,
        ctx: &ModuleContext,
        c: &GuildConfig,
        e: &MemberJoin,
    ) -> Result<()> {
        let Ok(invites) = ctx.discord.guild_invites(e.guild_id).await else {
            return Ok(());
        };
        let usages = invites
            .into_iter()
            .map(|i| InviteUsageInput {
                code: i.code,
                inviter_id: i.inviter_id,
                uses: i.uses,
            })
            .collect::<Vec<_>>();
        let attribution = ctx
            .services
            .attribute_invite(e.guild_id, e.user_id, &usages)
            .await?;
        if let (Some(channel), Some(a)) = (
            c.modules
                .invite_tracking
                .log_channel_id
                .as_deref()
                .and_then(|id| id.parse().ok()),
            attribution,
        ) {
            let inviter = a
                .inviter_id
                .map(|id| format!("<@{id}>"))
                .unwrap_or_else(|| "Unknown inviter".into());
            ctx.discord
                .send_message(
                    channel,
                    &format!(
                        "<@{}> joined using `{}` from {inviter}. Recorded invites: {}.",
                        e.user_id, a.code, a.inviter_total
                    ),
                )
                .await?;
            if let Some(inviter_id) = a.inviter_id {
                for reward in c
                    .modules
                    .invite_tracking
                    .rewards
                    .iter()
                    .filter(|reward| reward.invites <= a.inviter_total)
                {
                    ctx.discord
                        .add_member_role(e.guild_id, inviter_id, reward.role_id.parse()?)
                        .await?;
                }
            }
        }
        Ok(())
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use bot_discord::GuildInvite;
    use bot_test_support::{DiscordCall, FakeDiscord, FakeModuleServices, ServiceCall};
    use std::sync::Arc;
    #[test]
    fn module_uses_dedicated_config_switch() {
        let mut c = GuildConfig::for_guild("1");
        let module = InviteTrackingModule;
        assert!(!module.enabled(&c));
        c.modules.invite_tracking.enabled = true;
        assert!(module.enabled(&c));
    }

    #[tokio::test]
    async fn scheduler_primes_invite_attribution_snapshot() {
        let discord = FakeDiscord::new();
        discord.set_guild_invites(vec![GuildInvite {
            code: "hello".into(),
            inviter_id: Some(30),
            uses: 4,
        }]);
        let services = FakeModuleServices::default();
        let context = ModuleContext {
            discord: Arc::new(discord.clone()),
            services: Arc::new(services.clone()),
        };
        let mut config = GuildConfig::for_guild("10");
        config.modules.invite_tracking.enabled = true;

        InviteTrackingModule
            .on_scheduled_tick(&context, &config, &ScheduledTick { now_unix: 1 })
            .await
            .unwrap();

        assert_eq!(
            discord.calls(),
            vec![DiscordCall::GuildInvites { guild_id: 10 }]
        );
        assert_eq!(
            services.calls(),
            vec![
                ServiceCall::InviteSnapshotReady { guild_id: 10 },
                ServiceCall::EnsureInviteSnapshot {
                    guild_id: 10,
                    invites: vec![InviteUsageInput {
                        code: "hello".into(),
                        inviter_id: Some(30),
                        uses: 4,
                    }],
                }
            ]
        );
    }
}

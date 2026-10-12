use anyhow::Result;
use async_trait::async_trait;
use bot_core::GuildConfig;
use bot_discord::CommandDefinition;
use bot_module_api::{BotModule, InviteUsageInput, MemberJoin, ModuleContext};
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
        let attribution = ctx.services.attribute_invite(e.guild_id, &usages).await?;
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
        }
        Ok(())
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn module_uses_dedicated_config_switch() {
        let mut c = GuildConfig::for_guild("1");
        let module = InviteTrackingModule;
        assert!(!module.enabled(&c));
        c.modules.invite_tracking.enabled = true;
        assert!(module.enabled(&c));
    }
}

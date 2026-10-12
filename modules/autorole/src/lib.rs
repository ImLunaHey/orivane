use anyhow::{Context, Result};
use async_trait::async_trait;
use bot_core::GuildConfig;
use bot_module_api::{BotModule, MemberJoin, ModuleContext};

pub struct AutoroleModule;

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

    async fn on_member_join(
        &self,
        context: &ModuleContext,
        config: &GuildConfig,
        event: &MemberJoin,
    ) -> Result<()> {
        if config.modules.autorole.delay_seconds > 0 {
            tokio::time::sleep(std::time::Duration::from_secs(
                config.modules.autorole.delay_seconds.into(),
            ))
            .await;
        }
        for role_id in &config.modules.autorole.role_ids {
            context
                .discord
                .add_member_role(
                    event.guild_id,
                    event.user_id,
                    role_id.parse::<u64>().context("autorole ID is invalid")?,
                )
                .await?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use super::*;
    use bot_test_support::{DiscordCall, FakeDiscord};

    #[tokio::test]
    async fn assigns_every_configured_role() {
        let discord = FakeDiscord::new();
        let context = ModuleContext {
            discord: Arc::new(discord.clone()),
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
}

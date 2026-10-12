use anyhow::{Context, Result};
use async_trait::async_trait;
use bot_core::GuildConfig;
use bot_module_api::{BotModule, MemberJoin, ModuleContext};

pub struct WelcomeModule;

#[async_trait]
impl BotModule for WelcomeModule {
    fn name(&self) -> &'static str {
        "welcome"
    }
    fn enabled(&self, config: &GuildConfig) -> bool {
        config.modules.welcome.enabled
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
        let channel_id = config
            .modules
            .welcome
            .channel_id
            .as_deref()
            .context("welcome channel is not configured")?
            .parse::<u64>()
            .context("welcome channel is invalid")?;
        let message = config
            .modules
            .welcome
            .message
            .replace("{user}", &format!("<@{}>", event.user_id))
            .replace("{username}", &event.username)
            .replace("{display_name}", &event.display_name)
            .replace("{user_id}", &event.user_id.to_string())
            .replace("{server}", &event.guild_name)
            .replace("{server_id}", &event.guild_id.to_string());
        context.discord.send_message(channel_id, &message).await?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use super::*;
    use bot_test_support::{DiscordCall, FakeDiscord};

    #[tokio::test]
    async fn renders_member_and_server_variables() {
        let discord = FakeDiscord::new();
        let context = ModuleContext {
            discord: Arc::new(discord.clone()),
        };
        let mut config = GuildConfig::for_guild("10");
        config.modules.welcome.enabled = true;
        config.modules.welcome.channel_id = Some("20".to_owned());
        config.modules.welcome.message =
            "Welcome {user} ({display_name} / @{username}, {user_id}) to {server} ({server_id})!"
                .to_owned();
        WelcomeModule
            .on_member_join(
                &context,
                &config,
                &MemberJoin {
                    guild_id: 10,
                    guild_name: "Guild".to_owned(),
                    user_id: 30,
                    username: "imlunahey".to_owned(),
                    display_name: "luna".to_owned(),
                },
            )
            .await
            .unwrap();
        assert_eq!(
            discord.calls(),
            vec![DiscordCall::SendMessage {
                channel_id: 20,
                content: "Welcome <@30> (luna / @imlunahey, 30) to Guild (10)!".to_owned()
            }]
        );
    }
}

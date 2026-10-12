use anyhow::{Context, Result};
use async_trait::async_trait;
use bot_core::GuildConfig;
use bot_module_api::{BotModule, MemberJoin, MemberLeave, ModuleContext};

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
    fn handles_member_leave(&self) -> bool {
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
        let message = render(
            &config.modules.welcome.message,
            event.guild_id,
            &event.guild_name,
            event.user_id,
            &event.username,
            &event.display_name,
        );
        context.discord.send_message(channel_id, &message).await?;
        if config.modules.welcome.dm_enabled {
            let direct_message = render(
                &config.modules.welcome.dm_message,
                event.guild_id,
                &event.guild_name,
                event.user_id,
                &event.username,
                &event.display_name,
            );
            let _ = context
                .discord
                .send_direct_message(event.user_id, &direct_message)
                .await;
        }
        Ok(())
    }

    async fn on_member_leave(
        &self,
        context: &ModuleContext,
        config: &GuildConfig,
        event: &MemberLeave,
    ) -> Result<()> {
        if !config.modules.welcome.goodbye_enabled {
            return Ok(());
        }
        let channel_id = config
            .modules
            .welcome
            .goodbye_channel_id
            .as_deref()
            .context("goodbye channel is not configured")?
            .parse::<u64>()
            .context("goodbye channel is invalid")?;
        let message = render(
            &config.modules.welcome.goodbye_message,
            event.guild_id,
            &event.guild_name,
            event.user_id,
            &event.username,
            &event.display_name,
        );
        context.discord.send_message(channel_id, &message).await?;
        Ok(())
    }
}

fn render(
    template: &str,
    guild_id: u64,
    guild_name: &str,
    user_id: u64,
    username: &str,
    display_name: &str,
) -> String {
    template
        .replace("{user}", &format!("<@{user_id}>"))
        .replace("{username}", username)
        .replace("{display_name}", display_name)
        .replace("{user_id}", &user_id.to_string())
        .replace("{server}", guild_name)
        .replace("{server_id}", &guild_id.to_string())
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use super::*;
    use bot_test_support::{DiscordCall, FakeDiscord, FakeModuleServices};

    #[tokio::test]
    async fn renders_member_and_server_variables() {
        let discord = FakeDiscord::new();
        let context = ModuleContext {
            discord: Arc::new(discord.clone()),
            services: Arc::new(FakeModuleServices::default()),
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

    #[tokio::test]
    async fn sends_optional_direct_welcome_and_goodbye() {
        let discord = FakeDiscord::new();
        let context = ModuleContext {
            discord: Arc::new(discord.clone()),
            services: Arc::new(FakeModuleServices::default()),
        };
        let mut config = GuildConfig::for_guild("10");
        config.modules.welcome.channel_id = Some("20".to_owned());
        config.modules.welcome.dm_enabled = true;
        config.modules.welcome.dm_message = "Private welcome to {server}, @{username}".to_owned();
        config.modules.welcome.goodbye_enabled = true;
        config.modules.welcome.goodbye_channel_id = Some("21".to_owned());
        config.modules.welcome.goodbye_message = "Goodbye {display_name}".to_owned();
        let joined = MemberJoin {
            guild_id: 10,
            guild_name: "Guild".to_owned(),
            user_id: 30,
            username: "luna".to_owned(),
            display_name: "Luna".to_owned(),
        };
        WelcomeModule
            .on_member_join(&context, &config, &joined)
            .await
            .unwrap();
        WelcomeModule
            .on_member_leave(
                &context,
                &config,
                &MemberLeave {
                    guild_id: joined.guild_id,
                    guild_name: joined.guild_name,
                    user_id: joined.user_id,
                    username: joined.username,
                    display_name: joined.display_name,
                },
            )
            .await
            .unwrap();
        assert_eq!(
            discord.calls(),
            vec![
                DiscordCall::SendMessage {
                    channel_id: 20,
                    content: "Welcome <@30> to Guild!".to_owned(),
                },
                DiscordCall::SendDirectMessage {
                    user_id: 30,
                    content: "Private welcome to Guild, @luna".to_owned(),
                },
                DiscordCall::SendMessage {
                    channel_id: 21,
                    content: "Goodbye Luna".to_owned(),
                },
            ]
        );
    }
}

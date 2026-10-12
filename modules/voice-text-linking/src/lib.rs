use anyhow::Result;
use async_trait::async_trait;
use bot_core::GuildConfig;
use bot_discord::CommandDefinition;
use bot_module_api::{BotModule, ModuleContext, ModuleEventOutcome, VoiceStateChange};

pub struct VoiceTextLinkingModule;
pub const COMMANDS: &[CommandDefinition] = &[];

#[async_trait]
impl BotModule for VoiceTextLinkingModule {
    fn name(&self) -> &'static str {
        "voice_text_linking"
    }
    fn enabled(&self, config: &GuildConfig) -> bool {
        config.modules.voice_text_linking.enabled
    }
    fn handles_voice_state_change(&self) -> bool {
        true
    }
    async fn on_voice_state_change(
        &self,
        context: &ModuleContext,
        config: &GuildConfig,
        event: &VoiceStateChange,
    ) -> Result<Option<ModuleEventOutcome>> {
        if event.user_is_bot || event.previous_channel_id == event.channel_id {
            return Ok(None);
        }
        let settings = &config.modules.voice_text_linking;
        if let Some(joined) = event.channel_id
            && let Some(link) = settings
                .links
                .iter()
                .find(|link| link.voice_channel_id == joined.to_string() && link.announce_joins)
            && let Ok(channel) = link.text_channel_id.parse()
        {
            context
                .discord
                .send_message(
                    channel,
                    &format!("<@{}> joined <#{}>.", event.user_id, joined),
                )
                .await?;
            context
                .services
                .record_voice_link_event(event.guild_id, event.user_id, joined, channel, "joined")
                .await?;
        }
        if let Some(left) = event.previous_channel_id
            && let Some(link) = settings
                .links
                .iter()
                .find(|link| link.voice_channel_id == left.to_string() && link.announce_leaves)
            && let Ok(channel) = link.text_channel_id.parse()
        {
            context
                .discord
                .send_message(channel, &format!("<@{}> left <#{}>.", event.user_id, left))
                .await?;
            context
                .services
                .record_voice_link_event(event.guild_id, event.user_id, left, channel, "left")
                .await?;
        }
        Ok(Some(ModuleEventOutcome {
            action: "voice_text_link.activity".into(),
            target_type: "member".into(),
            target_id: Some(event.user_id.to_string()),
        }))
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use bot_core::VoiceTextLinkConfig;
    use bot_test_support::{DiscordCall, FakeDiscord, FakeModuleServices, ServiceCall};
    use std::sync::Arc;
    #[test]
    fn module_uses_dedicated_config_switch() {
        let mut c = GuildConfig::for_guild("1");
        let module = VoiceTextLinkingModule;
        assert!(!module.enabled(&c));
        c.modules.voice_text_linking.enabled = true;
        assert!(module.enabled(&c));
    }

    #[tokio::test]
    async fn announces_channel_moves_and_records_both_sides() {
        let discord = FakeDiscord::new();
        let services = FakeModuleServices::default();
        let context = ModuleContext {
            discord: Arc::new(discord.clone()),
            services: Arc::new(services.clone()),
        };
        let mut config = GuildConfig::for_guild("10");
        config.modules.voice_text_linking.enabled = true;
        config.modules.voice_text_linking.links = vec![
            VoiceTextLinkConfig {
                voice_channel_id: "20".into(),
                text_channel_id: "21".into(),
                announce_joins: false,
                announce_leaves: true,
            },
            VoiceTextLinkConfig {
                voice_channel_id: "30".into(),
                text_channel_id: "31".into(),
                announce_joins: true,
                announce_leaves: false,
            },
        ];

        VoiceTextLinkingModule
            .on_voice_state_change(
                &context,
                &config,
                &VoiceStateChange {
                    guild_id: 10,
                    user_id: 40,
                    user_is_bot: false,
                    previous_channel_id: Some(20),
                    channel_id: Some(30),
                    previous_session_seconds: 120,
                    previous_session_eligible: true,
                    username: "luna".into(),
                },
            )
            .await
            .unwrap();

        assert_eq!(
            discord.calls(),
            vec![
                DiscordCall::SendMessage {
                    channel_id: 31,
                    content: "<@40> joined <#30>.".into(),
                },
                DiscordCall::SendMessage {
                    channel_id: 21,
                    content: "<@40> left <#20>.".into(),
                }
            ]
        );
        assert_eq!(
            services.calls(),
            vec![
                ServiceCall::RecordVoiceLinkEvent {
                    guild_id: 10,
                    user_id: 40,
                    voice_channel_id: 30,
                    text_channel_id: 31,
                    action: "joined".into(),
                },
                ServiceCall::RecordVoiceLinkEvent {
                    guild_id: 10,
                    user_id: 40,
                    voice_channel_id: 20,
                    text_channel_id: 21,
                    action: "left".into(),
                }
            ]
        );
    }
}

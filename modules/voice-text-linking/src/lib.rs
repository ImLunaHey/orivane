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
        if let Some(joined) = event.channel_id {
            if let Some(link) = settings
                .links
                .iter()
                .find(|link| link.voice_channel_id == joined.to_string() && link.announce_joins)
            {
                if let Ok(channel) = link.text_channel_id.parse() {
                    context
                        .discord
                        .send_message(
                            channel,
                            &format!("<@{}> joined <#{}>.", event.user_id, joined),
                        )
                        .await?;
                }
            }
        }
        if let Some(left) = event.previous_channel_id {
            if let Some(link) = settings
                .links
                .iter()
                .find(|link| link.voice_channel_id == left.to_string() && link.announce_leaves)
            {
                if let Ok(channel) = link.text_channel_id.parse() {
                    context
                        .discord
                        .send_message(channel, &format!("<@{}> left <#{}>.", event.user_id, left))
                        .await?;
                }
            }
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
    #[test]
    fn module_uses_dedicated_config_switch() {
        let mut c = GuildConfig::for_guild("1");
        let module = VoiceTextLinkingModule;
        assert!(!module.enabled(&c));
        c.modules.voice_text_linking.enabled = true;
        assert!(module.enabled(&c));
    }
}

use async_trait::async_trait;
use bot_core::GuildConfig;
use bot_module_api::BotModule;

pub struct ModerationModule;

#[async_trait]
impl BotModule for ModerationModule {
    fn name(&self) -> &'static str {
        "moderation"
    }
    fn enabled(&self, config: &GuildConfig) -> bool {
        config.modules.moderation.enabled
    }
}

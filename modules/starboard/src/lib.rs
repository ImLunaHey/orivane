use async_trait::async_trait;
use bot_core::GuildConfig;
use bot_module_api::BotModule;

pub struct StarboardModule;

#[async_trait]
impl BotModule for StarboardModule {
    fn name(&self) -> &'static str {
        "starboard"
    }
    fn enabled(&self, config: &GuildConfig) -> bool {
        config.modules.starboard.enabled
    }
}

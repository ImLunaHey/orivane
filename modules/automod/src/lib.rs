use async_trait::async_trait;
use bot_core::GuildConfig;
use bot_module_api::BotModule;

pub struct AutomodModule;

#[async_trait]
impl BotModule for AutomodModule {
    fn name(&self) -> &'static str {
        "automod"
    }
    fn enabled(&self, config: &GuildConfig) -> bool {
        config.modules.automod.enabled
    }
}

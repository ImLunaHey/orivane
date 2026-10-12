use async_trait::async_trait;
use bot_core::GuildConfig;
use bot_module_api::BotModule;

pub struct LevelingModule;

#[async_trait]
impl BotModule for LevelingModule {
    fn name(&self) -> &'static str {
        "leveling"
    }
    fn enabled(&self, config: &GuildConfig) -> bool {
        config.modules.leveling.enabled
    }
}

use async_trait::async_trait;
use bot_core::GuildConfig;
use bot_module_api::BotModule;

pub struct CustomCommandsModule;

#[async_trait]
impl BotModule for CustomCommandsModule {
    fn name(&self) -> &'static str {
        "custom_commands"
    }
    fn enabled(&self, config: &GuildConfig) -> bool {
        config.modules.custom_commands.enabled
    }
}

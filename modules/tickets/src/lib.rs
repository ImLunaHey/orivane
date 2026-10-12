use async_trait::async_trait;
use bot_core::GuildConfig;
use bot_module_api::BotModule;

pub struct TicketsModule;

#[async_trait]
impl BotModule for TicketsModule {
    fn name(&self) -> &'static str {
        "tickets"
    }
    fn enabled(&self, config: &GuildConfig) -> bool {
        config.modules.tickets.enabled
    }
}

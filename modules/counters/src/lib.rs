use anyhow::{Context, Result};
use async_trait::async_trait;
use bot_core::GuildConfig;
use bot_discord::CommandDefinition;
use bot_module_api::{BotModule, ModuleContext, ModuleEventOutcome, ScheduledTick};
pub struct CountersModule;
pub const COMMANDS: &[CommandDefinition] = &[];
#[async_trait]
impl BotModule for CountersModule {
    fn name(&self) -> &'static str {
        "counters"
    }
    fn enabled(&self, c: &GuildConfig) -> bool {
        c.modules.counters.enabled
    }
    fn handles_scheduled_tick(&self) -> bool {
        true
    }
    async fn on_scheduled_tick(
        &self,
        ctx: &ModuleContext,
        c: &GuildConfig,
        _: &ScheduledTick,
    ) -> Result<Vec<ModuleEventOutcome>> {
        let guild = c.guild_id.parse::<u64>().context("invalid counter guild")?;
        let metrics = ctx.discord.guild_metrics(guild).await?;
        let mut outcomes = Vec::new();
        let mut first_error = None;
        for counter in c
            .modules
            .counters
            .counters
            .iter()
            .filter(|counter| counter.enabled)
        {
            let count = match counter.kind.as_str() {
                "members" => metrics.members,
                "online" => metrics.online,
                "boosts" => metrics.boosts,
                "goal" => counter.value,
                _ => continue,
            };
            let goal = counter.goal.unwrap_or_default();
            let remaining = goal.saturating_sub(count);
            let name = counter
                .label
                .replace("{count}", &count.to_string())
                .replace("{goal}", &goal.to_string())
                .replace("{remaining}", &remaining.to_string())
                .chars()
                .take(100)
                .collect::<String>();
            let channel_id = counter.channel_id.parse()?;
            let result = ctx
                .discord
                .rename_channel(channel_id, &name, "Orivane counter update")
                .await;
            let error = result.as_ref().err().map(ToString::to_string);
            ctx.services
                .record_counter_run(
                    guild,
                    &counter.id,
                    channel_id,
                    count,
                    &name,
                    if result.is_ok() { "success" } else { "failure" },
                    error.as_deref(),
                )
                .await?;
            match result {
                Ok(()) => outcomes.push(ModuleEventOutcome {
                    action: "counter.updated".into(),
                    target_type: "channel".into(),
                    target_id: Some(counter.channel_id.clone()),
                }),
                Err(error) => {
                    first_error.get_or_insert(error);
                }
            }
        }
        if let Some(error) = first_error {
            return Err(error.into());
        }
        Ok(outcomes)
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use bot_core::CounterConfig;
    use bot_discord::GuildMetrics;
    use bot_test_support::{DiscordCall, FakeDiscord, FakeModuleServices, ServiceCall};
    use std::sync::Arc;
    #[test]
    fn module_uses_dedicated_config_switch() {
        let mut c = GuildConfig::for_guild("1");
        let module = CountersModule;
        assert!(!module.enabled(&c));
        c.modules.counters.enabled = true;
        assert!(module.enabled(&c));
    }

    #[tokio::test]
    async fn renders_metrics_and_records_successful_updates() {
        let discord = FakeDiscord::new();
        discord.set_guild_metrics(GuildMetrics {
            members: 123,
            online: 45,
            boosts: 6,
        });
        let services = FakeModuleServices::default();
        let context = ModuleContext {
            discord: Arc::new(discord.clone()),
            services: Arc::new(services.clone()),
        };
        let mut config = GuildConfig::for_guild("10");
        config.modules.counters.enabled = true;
        config.modules.counters.counters.push(CounterConfig {
            id: "members".into(),
            enabled: true,
            channel_id: "20".into(),
            label: "Members: {count} / {goal} ({remaining} left)".into(),
            kind: "members".into(),
            value: 0,
            goal: Some(200),
        });

        let outcomes = CountersModule
            .on_scheduled_tick(&context, &config, &ScheduledTick { now_unix: 1 })
            .await
            .unwrap();

        assert_eq!(outcomes.len(), 1);
        assert_eq!(
            discord.calls(),
            vec![
                DiscordCall::GuildMetrics { guild_id: 10 },
                DiscordCall::RenameChannel {
                    channel_id: 20,
                    name: "Members: 123 / 200 (77 left)".into(),
                    reason: "Orivane counter update".into(),
                }
            ]
        );
        assert!(matches!(
            services.calls().as_slice(),
            [ServiceCall::RecordCounterRun { value: 123, status, .. }] if status == "success"
        ));
    }
}

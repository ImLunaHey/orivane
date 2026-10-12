use anyhow::{Context, Result};
use async_trait::async_trait;
use bot_core::{GuildConfig, ScheduledMessageConfig};
use bot_module_api::{BotModule, ModuleContext, ModuleEventOutcome, ScheduledTick};

pub struct ScheduledMessagesModule;

#[async_trait]
impl BotModule for ScheduledMessagesModule {
    fn name(&self) -> &'static str {
        "scheduled_messages"
    }
    fn enabled(&self, config: &GuildConfig) -> bool {
        config.modules.scheduled_messages.enabled
    }
    fn handles_scheduled_tick(&self) -> bool {
        true
    }

    async fn on_scheduled_tick(
        &self,
        context: &ModuleContext,
        config: &GuildConfig,
        event: &ScheduledTick,
    ) -> Result<Vec<ModuleEventOutcome>> {
        let guild_id = config
            .guild_id
            .parse::<u64>()
            .context("scheduled-message guild ID is invalid")?;
        let mut outcomes = Vec::new();
        for schedule in config
            .modules
            .scheduled_messages
            .schedules
            .iter()
            .filter(|schedule| schedule.enabled)
        {
            let Some(occurrence) = due_occurrence(schedule, event.now_unix) else {
                continue;
            };
            if !context
                .services
                .claim_scheduled_message(guild_id, &schedule.id, occurrence)
                .await?
            {
                continue;
            }
            let channel_id = schedule
                .channel_id
                .parse::<u64>()
                .context("scheduled-message channel ID is invalid")?;
            let content = schedule.message.replace("{server_id}", &config.guild_id);
            if let Err(error) = context.discord.send_message(channel_id, &content).await {
                context
                    .services
                    .release_scheduled_message(guild_id, &schedule.id, occurrence)
                    .await?;
                return Err(anyhow::Error::from(error)
                    .context(format!("schedule {} delivery failed", schedule.id)));
            }
            outcomes.push(ModuleEventOutcome {
                action: "scheduled_message.sent".into(),
                target_type: "scheduled_message".into(),
                target_id: Some(schedule.id.clone()),
            });
        }
        Ok(outcomes)
    }
}

fn due_occurrence(schedule: &ScheduledMessageConfig, now: u64) -> Option<u64> {
    if now < schedule.starts_at_unix {
        return None;
    }
    let interval = u64::from(schedule.interval_minutes) * 60;
    let occurrence =
        schedule.starts_at_unix + ((now - schedule.starts_at_unix) / interval) * interval;
    if schedule.ends_at_unix.is_some_and(|end| occurrence > end) {
        None
    } else {
        Some(occurrence)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bot_test_support::{DiscordCall, FakeDiscord, FakeModuleServices};
    use std::sync::Arc;

    #[tokio::test]
    async fn sends_and_claims_the_current_occurrence_once() {
        let discord = FakeDiscord::new();
        let context = ModuleContext {
            discord: Arc::new(discord.clone()),
            services: Arc::new(FakeModuleServices::default()),
        };
        let mut config = GuildConfig::for_guild("10");
        config.modules.scheduled_messages.enabled = true;
        config.modules.scheduled_messages.schedules = vec![ScheduledMessageConfig {
            enabled: true,
            id: "rules".into(),
            channel_id: "20".into(),
            message: "Read the rules for {server_id}".into(),
            starts_at_unix: 1_000,
            interval_minutes: 60,
            ends_at_unix: None,
        }];
        let outcomes = ScheduledMessagesModule
            .on_scheduled_tick(&context, &config, &ScheduledTick { now_unix: 4_601 })
            .await
            .unwrap();
        let duplicate = ScheduledMessagesModule
            .on_scheduled_tick(&context, &config, &ScheduledTick { now_unix: 4_601 })
            .await
            .unwrap();
        assert_eq!(outcomes.len(), 1);
        assert!(duplicate.is_empty());
        assert_eq!(
            discord.calls(),
            vec![DiscordCall::SendMessage {
                channel_id: 20,
                content: "Read the rules for 10".into()
            }]
        );
    }

    #[test]
    fn respects_start_and_end_boundaries() {
        let schedule = ScheduledMessageConfig {
            enabled: true,
            id: "x".into(),
            channel_id: "20".into(),
            message: "x".into(),
            starts_at_unix: 1_000,
            interval_minutes: 10,
            ends_at_unix: Some(1_600),
        };
        assert_eq!(due_occurrence(&schedule, 999), None);
        assert_eq!(due_occurrence(&schedule, 1_599), Some(1_000));
        assert_eq!(due_occurrence(&schedule, 1_601), Some(1_600));
        assert_eq!(due_occurrence(&schedule, 2_201), None);
    }
}

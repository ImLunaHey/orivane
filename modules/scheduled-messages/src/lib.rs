use anyhow::{Context, Result};
use async_trait::async_trait;
use bot_core::{GuildConfig, ScheduledMessageConfig};
use bot_module_api::ManagedEmbedDeliveryInput;
use bot_module_api::{BotModule, ModuleContext, ModuleEventOutcome, ScheduledTick};
use bot_module_managed_embeds::rich_embed;

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
            let delivery = if let Some(template_id) = schedule.embed_template_id {
                let template = context
                    .services
                    .managed_embed(guild_id, template_id)
                    .await?
                    .context("scheduled embed template not found")?;
                let server_name = context
                    .services
                    .guild_name(guild_id)
                    .await?
                    .unwrap_or_else(|| "this server".into());
                context
                    .discord
                    .send_rich_embed(
                        channel_id,
                        &rich_embed(&template.payload, guild_id, &server_name, 0),
                    )
                    .await
                    .map(Some)
            } else {
                let content = schedule.message.replace("{server_id}", &config.guild_id);
                context
                    .discord
                    .send_message(channel_id, &content)
                    .await
                    .map(|()| None)
            };
            let message_id = match delivery {
                Ok(message_id) => message_id,
                Err(error) => {
                    context
                        .services
                        .release_scheduled_message(guild_id, &schedule.id, occurrence)
                        .await?;
                    return Err(anyhow::Error::from(error)
                        .context(format!("schedule {} delivery failed", schedule.id)));
                }
            };
            if let Some(template_id) = schedule.embed_template_id {
                context
                    .services
                    .record_managed_embed_delivery(ManagedEmbedDeliveryInput {
                        guild_id,
                        template_id,
                        channel_id,
                        message_id,
                        actor_id: 0,
                        status: "delivered".into(),
                        error: None,
                    })
                    .await?;
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
    let occurrence = match schedule.kind.as_str() {
        "once" => schedule.starts_at_unix,
        "weekdays" => weekday_occurrence(schedule, now)?,
        _ => {
            let interval = u64::from(schedule.interval_minutes) * 60;
            schedule.starts_at_unix + ((now - schedule.starts_at_unix) / interval) * interval
        }
    };
    if schedule.ends_at_unix.is_some_and(|end| occurrence > end) {
        None
    } else {
        Some(occurrence)
    }
}

fn weekday_occurrence(schedule: &ScheduledMessageConfig, now: u64) -> Option<u64> {
    const DAY: i64 = 86_400;
    let offset = i64::from(schedule.timezone_offset_minutes) * 60;
    let local_now = i64::try_from(now).ok()?.checked_add(offset)?;
    let local_start = i64::try_from(schedule.starts_at_unix)
        .ok()?
        .checked_add(offset)?;
    let time_of_day = local_start.rem_euclid(DAY);
    let today = local_now.div_euclid(DAY);
    for days_back in 0..=7 {
        let day = today - days_back;
        let iso_weekday = u8::try_from((day + 3).rem_euclid(7) + 1).ok()?;
        if !schedule.weekdays.contains(&iso_weekday) {
            continue;
        }
        let local_occurrence = day.checked_mul(DAY)?.checked_add(time_of_day)?;
        let occurrence = u64::try_from(local_occurrence.checked_sub(offset)?).ok()?;
        if occurrence <= now && occurrence >= schedule.starts_at_unix {
            return Some(occurrence);
        }
    }
    None
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
            kind: "interval".into(),
            weekdays: Vec::new(),
            timezone_offset_minutes: 0,
            embed_template_id: None,
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

    #[tokio::test]
    async fn sends_a_managed_embed_schedule() {
        let discord = FakeDiscord::new();
        let context = ModuleContext {
            discord: Arc::new(discord.clone()),
            services: Arc::new(FakeModuleServices::default()),
        };
        let mut config = GuildConfig::for_guild("10");
        config.modules.scheduled_messages.enabled = true;
        config.modules.scheduled_messages.schedules = vec![ScheduledMessageConfig {
            enabled: true,
            id: "embed".into(),
            channel_id: "20".into(),
            message: "fallback".into(),
            kind: "once".into(),
            weekdays: Vec::new(),
            timezone_offset_minutes: 0,
            embed_template_id: Some(1),
            starts_at_unix: 1_000,
            interval_minutes: 1,
            ends_at_unix: None,
        }];
        ScheduledMessagesModule
            .on_scheduled_tick(&context, &config, &ScheduledTick { now_unix: 1_000 })
            .await
            .unwrap();
        assert!(matches!(
            discord.calls().first(),
            Some(DiscordCall::SendRichEmbed { channel_id: 20, .. })
        ));
    }

    #[test]
    fn respects_start_and_end_boundaries() {
        let schedule = ScheduledMessageConfig {
            enabled: true,
            id: "x".into(),
            channel_id: "20".into(),
            message: "x".into(),
            kind: "interval".into(),
            weekdays: Vec::new(),
            timezone_offset_minutes: 0,
            embed_template_id: None,
            starts_at_unix: 1_000,
            interval_minutes: 10,
            ends_at_unix: Some(1_600),
        };
        assert_eq!(due_occurrence(&schedule, 999), None);
        assert_eq!(due_occurrence(&schedule, 1_599), Some(1_000));
        assert_eq!(due_occurrence(&schedule, 1_601), Some(1_600));
        assert_eq!(due_occurrence(&schedule, 2_201), None);
    }

    #[test]
    fn supports_one_time_and_weekday_schedules() {
        let mut schedule = ScheduledMessageConfig {
            enabled: true,
            id: "x".into(),
            channel_id: "20".into(),
            message: "x".into(),
            kind: "once".into(),
            weekdays: Vec::new(),
            timezone_offset_minutes: 0,
            embed_template_id: None,
            starts_at_unix: 1_000,
            interval_minutes: 1,
            ends_at_unix: None,
        };
        assert_eq!(due_occurrence(&schedule, 999), None);
        assert_eq!(due_occurrence(&schedule, 50_000), Some(1_000));

        // 1970-01-05 10:00 UTC was a Monday (ISO weekday 1).
        schedule.kind = "weekdays".into();
        schedule.weekdays = vec![1, 3, 5];
        schedule.starts_at_unix = 4 * 86_400 + 10 * 3_600;
        assert_eq!(
            due_occurrence(&schedule, 4 * 86_400 + 11 * 3_600),
            Some(4 * 86_400 + 10 * 3_600)
        );
        // Tuesday still resolves to the most recent Monday occurrence.
        assert_eq!(
            due_occurrence(&schedule, 5 * 86_400 + 11 * 3_600),
            Some(4 * 86_400 + 10 * 3_600)
        );
    }
}

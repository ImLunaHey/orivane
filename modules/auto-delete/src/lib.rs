use anyhow::{Context, Result};
use async_trait::async_trait;
use bot_core::GuildConfig;
use bot_discord::CommandDefinition;
use bot_module_api::{BotModule, MessageCreate, ModuleContext, ModuleEventOutcome, ScheduledTick};

pub struct AutoDeleteModule;
pub const COMMANDS: &[CommandDefinition] = &[];

#[async_trait]
impl BotModule for AutoDeleteModule {
    fn name(&self) -> &'static str {
        "auto_delete"
    }
    fn enabled(&self, c: &GuildConfig) -> bool {
        c.modules.auto_delete.enabled
    }
    fn commands(&self) -> &'static [CommandDefinition] {
        COMMANDS
    }
    fn handles_message_create(&self) -> bool {
        true
    }
    fn handles_scheduled_tick(&self) -> bool {
        true
    }
    async fn on_message_create(
        &self,
        ctx: &ModuleContext,
        cfg: &GuildConfig,
        event: &MessageCreate,
    ) -> Result<Option<ModuleEventOutcome>> {
        let Some(rule) = cfg.modules.auto_delete.rules.iter().find(|r| {
            r.enabled
                && r.channel_id == event.channel_id.to_string()
                && !(r.ignore_bot_messages && event.author_is_bot)
                && !r
                    .excluded_role_ids
                    .iter()
                    .any(|id| event.role_ids.iter().any(|role| id == &role.to_string()))
        }) else {
            return Ok(None);
        };
        let due = now().saturating_add(u64::from(rule.delay_seconds));
        if !ctx
            .services
            .schedule_message_deletion(
                event.guild_id,
                event.channel_id,
                event.message_id,
                &rule.id,
                event.author_id,
                due,
            )
            .await?
        {
            return Ok(None);
        }
        Ok(Some(ModuleEventOutcome {
            action: "auto_delete.scheduled".into(),
            target_type: "message".into(),
            target_id: Some(event.message_id.to_string()),
        }))
    }
    async fn on_scheduled_tick(
        &self,
        ctx: &ModuleContext,
        cfg: &GuildConfig,
        event: &ScheduledTick,
    ) -> Result<Vec<ModuleEventOutcome>> {
        let guild = cfg
            .guild_id
            .parse()
            .context("invalid auto-delete guild ID")?;
        let mut out = vec![];
        for job in ctx
            .services
            .claim_due_message_deletions(guild, event.now_unix, 100)
            .await?
        {
            let deleted = ctx
                .discord
                .delete_message(job.channel_id, job.message_id, "Orivane auto-delete rule")
                .await
                .is_ok();
            let backoff = 60u64.saturating_mul(2u64.pow(u32::from(job.attempts.saturating_sub(1))));
            let failed = ctx
                .services
                .finish_message_deletion(
                    guild,
                    job.channel_id,
                    job.message_id,
                    deleted,
                    event.now_unix.saturating_add(backoff),
                )
                .await?;
            out.push(ModuleEventOutcome {
                action: if deleted {
                    "auto_delete.deleted"
                } else if failed {
                    "auto_delete.failed"
                } else {
                    "auto_delete.retry_scheduled"
                }
                .into(),
                target_type: "message".into(),
                target_id: Some(job.message_id.to_string()),
            });
        }
        Ok(out)
    }
}
fn now() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

#[cfg(test)]
mod tests {
    use super::*;
    use bot_test_support::{DiscordCall, FakeDiscord, FakeModuleServices};
    use std::sync::Arc;
    fn context() -> (ModuleContext, FakeDiscord) {
        let d = FakeDiscord::new();
        (
            ModuleContext {
                discord: Arc::new(d.clone()),
                services: Arc::new(FakeModuleServices::default()),
            },
            d,
        )
    }
    #[tokio::test]
    async fn schedules_matching_messages_without_content() {
        let (ctx, _) = context();
        let mut c = GuildConfig::for_guild("10");
        c.modules.auto_delete.enabled = true;
        c.modules.auto_delete.rules.push(bot_core::AutoDeleteRule {
            enabled: true,
            id: "chat".into(),
            channel_id: "20".into(),
            delay_seconds: 60,
            ignore_bot_messages: false,
            excluded_role_ids: vec![],
        });
        let out = AutoDeleteModule
            .on_message_create(
                &ctx,
                &c,
                &MessageCreate {
                    guild_id: 10,
                    channel_id: 20,
                    message_id: 40,
                    author_id: 30,
                    author_is_bot: false,
                    role_ids: vec![],
                    mention_count: 0,
                    content: "private text".into(),
                    attachment_filenames: vec![],
                    sticker_count: 0,
                },
            )
            .await
            .unwrap()
            .unwrap();
        assert_eq!(out.action, "auto_delete.scheduled")
    }
    #[tokio::test]
    async fn deletes_claimed_jobs() {
        let (ctx, d) = context();
        let mut c = GuildConfig::for_guild("10");
        c.modules.auto_delete.enabled = true;
        let out = AutoDeleteModule
            .on_scheduled_tick(&ctx, &c, &ScheduledTick { now_unix: 1 })
            .await
            .unwrap();
        assert_eq!(out[0].action, "auto_delete.deleted");
        assert!(matches!(
            &d.calls()[0],
            DiscordCall::DeleteMessage { message_id: 40, .. }
        ))
    }
}

use anyhow::Result;
use async_trait::async_trait;
use bot_core::GuildConfig;
use bot_discord::{
    CommandDefinition, CommandInteraction, CommandOptionDefinition, CommandOptionKind,
    CommandResponse,
};
use bot_module_api::{BotModule, MessageCreate, ModuleContext, ModuleEventOutcome};
use std::time::{SystemTime, UNIX_EPOCH};
pub struct AfkModule;
pub const COMMANDS: &[CommandDefinition] = &[CommandDefinition {
    name: "afk",
    description: "Set or clear your away status",
    guild_only: true,
    default_permissions: None,
    options: &[CommandOptionDefinition {
        name: "reason",
        description: "Why you are away; omit to clear your status",
        kind: CommandOptionKind::String,
        required: false,
        min_value: None,
        max_value: None,
        min_length: Some(1),
        max_length: Some(300),
    }],
}];
#[async_trait]
impl BotModule for AfkModule {
    fn name(&self) -> &'static str {
        "afk"
    }
    fn enabled(&self, c: &GuildConfig) -> bool {
        c.modules.afk.enabled
    }
    fn commands(&self) -> &'static [CommandDefinition] {
        COMMANDS
    }
    fn handles_message_create(&self) -> bool {
        true
    }
    async fn on_command(
        &self,
        ctx: &ModuleContext,
        c: &GuildConfig,
        i: &CommandInteraction,
    ) -> Result<()> {
        if i.command_name != "afk" {
            return Ok(());
        }
        if let Some(reason) = i.string("reason") {
            let now = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs();
            ctx.services
                .set_afk_status(
                    i.guild_id,
                    i.actor_id,
                    reason,
                    now,
                    u64::from(c.modules.afk.maximum_hours) * 3600,
                )
                .await?;
            ctx.discord
                .respond_to_command(
                    i,
                    CommandResponse::ephemeral(format!("You are now AFK: {}", safe(reason))),
                )
                .await?;
        } else {
            let cleared = ctx
                .services
                .clear_afk_status(i.guild_id, i.actor_id)
                .await?;
            ctx.discord
                .respond_to_command(
                    i,
                    CommandResponse::ephemeral(if cleared {
                        "Your AFK status was cleared."
                    } else {
                        "You are not currently AFK."
                    }),
                )
                .await?;
        }
        Ok(())
    }
    async fn on_message_create(
        &self,
        ctx: &ModuleContext,
        c: &GuildConfig,
        e: &MessageCreate,
    ) -> Result<Option<ModuleEventOutcome>> {
        if e.author_is_bot {
            return Ok(None);
        }
        let mut acted = false;
        if c.modules.afk.clear_on_message
            && ctx
                .services
                .clear_afk_status(e.guild_id, e.author_id)
                .await?
        {
            ctx.discord
                .send_message(
                    e.channel_id,
                    &format!(
                        "Welcome back <@{}>; your AFK status was cleared.",
                        e.author_id
                    ),
                )
                .await?;
            acted = true;
        }
        if c.modules.afk.notify_mentions {
            for id in mentioned_users(&e.content) {
                if id == e.author_id {
                    continue;
                }
                if let Some(status) = ctx.services.afk_status(e.guild_id, id).await? {
                    ctx.discord
                        .send_message(
                            e.channel_id,
                            &format!("<@{id}> is AFK: {}", safe(&status.reason)),
                        )
                        .await?;
                    acted = true;
                }
            }
        }
        Ok(acted.then(|| ModuleEventOutcome {
            action: "afk.message_handled".into(),
            target_type: "message".into(),
            target_id: Some(e.message_id.to_string()),
        }))
    }
}
fn mentioned_users(content: &str) -> Vec<u64> {
    content
        .split('<')
        .filter_map(|part| part.strip_prefix('@'))
        .filter_map(|part| part.strip_prefix('!').unwrap_or(part).split('>').next())
        .filter_map(|id| id.parse().ok())
        .collect()
}
fn safe(v: &str) -> String {
    v.replace('@', "@\u{200b}").chars().take(300).collect()
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn extracts_standard_and_nickname_mentions() {
        assert_eq!(mentioned_users("hi <@123> and <@!456>"), vec![123, 456]);
    }
    #[test]
    fn neutralizes_mass_mentions() {
        assert_eq!(safe("@everyone away"), "@\u{200b}everyone away");
    }
}

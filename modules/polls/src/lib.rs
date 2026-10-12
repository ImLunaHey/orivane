use anyhow::{Context, Result, bail};
use async_trait::async_trait;
use bot_core::GuildConfig;
use bot_discord::{
    CommandDefinition, CommandInteraction, CommandOptionDefinition, CommandOptionKind,
    CommandResponse, ComponentInteraction, MessageButton, MessageButtonStyle,
};
use bot_module_api::{BotModule, DuePollResult, ModuleContext, ModuleEventOutcome, ScheduledTick};
pub struct PollsModule;
const PREFIX: &str = "orivane:poll:";
const fn text(
    name: &'static str,
    description: &'static str,
    required: bool,
    max: u16,
) -> CommandOptionDefinition {
    CommandOptionDefinition {
        name,
        description,
        kind: CommandOptionKind::String,
        required,
        min_value: None,
        max_value: None,
        min_length: Some(1),
        max_length: Some(max),
    }
}
pub const COMMANDS: &[CommandDefinition] = &[
    CommandDefinition {
        name: "poll",
        description: "Create a timed poll",
        guild_only: true,
        default_permissions: None,
        options: &[
            text("question", "Question to ask", true, 300),
            text("option-1", "First choice", true, 80),
            text("option-2", "Second choice", true, 80),
            text("option-3", "Optional third choice", false, 80),
            text("option-4", "Optional fourth choice", false, 80),
            text("option-5", "Optional fifth choice", false, 80),
            CommandOptionDefinition {
                name: "minutes",
                description: "Duration in minutes",
                kind: CommandOptionKind::Integer,
                required: false,
                min_value: Some(1),
                max_value: Some(525600),
                min_length: None,
                max_length: None,
            },
            CommandOptionDefinition {
                name: "anonymous",
                description: "Hide individual voter identities",
                kind: CommandOptionKind::Boolean,
                required: false,
                min_value: None,
                max_value: None,
                min_length: None,
                max_length: None,
            },
        ],
    },
    CommandDefinition {
        name: "poll-close",
        description: "Close a poll early",
        guild_only: true,
        default_permissions: None,
        options: &[CommandOptionDefinition {
            name: "id",
            description: "Poll ID",
            kind: CommandOptionKind::Integer,
            required: true,
            min_value: Some(1),
            max_value: None,
            min_length: None,
            max_length: None,
        }],
    },
];
#[async_trait]
impl BotModule for PollsModule {
    fn name(&self) -> &'static str {
        "polls"
    }
    fn enabled(&self, c: &GuildConfig) -> bool {
        c.modules.polls.enabled
    }
    fn commands(&self) -> &'static [CommandDefinition] {
        COMMANDS
    }
    fn handles_component(&self, id: &str) -> bool {
        id.starts_with(PREFIX)
    }
    fn handles_scheduled_tick(&self) -> bool {
        true
    }
    async fn on_command(
        &self,
        ctx: &ModuleContext,
        c: &GuildConfig,
        i: &CommandInteraction,
    ) -> Result<()> {
        if i.command_name == "poll-close" {
            let allowed = c.modules.polls.manager_role_ids.is_empty()
                || c.modules
                    .polls
                    .manager_role_ids
                    .iter()
                    .any(|role| i.actor_role_ids.iter().any(|id| role == &id.to_string()));
            if !allowed {
                bail!("you do not have a poll manager role")
            }
            let id = i.integer("id").context("poll ID required")?;
            let closed = ctx.services.close_poll(i.guild_id, id).await?;
            if let Some(poll) = &closed {
                ctx.discord
                    .edit_message_with_buttons(
                        poll.channel_id,
                        poll.message_id,
                        &closed_poll_content(poll),
                        &[],
                    )
                    .await?;
            }
            ctx.discord
                .respond_to_command(
                    i,
                    CommandResponse::ephemeral(if closed.is_some() {
                        format!("Poll #{id} closed.")
                    } else {
                        format!("Poll #{id} was not active.")
                    }),
                )
                .await?;
            return Ok(());
        }
        if i.command_name != "poll" {
            return Ok(());
        }
        let q = i.string("question").context("question required")?;
        let options = (1..=5)
            .filter_map(|n| {
                i.string(&format!("option-{n}"))
                    .map(str::trim)
                    .filter(|v| !v.is_empty())
                    .map(str::to_owned)
            })
            .collect::<Vec<_>>();
        if options.len() < 2 || options.len() > usize::from(c.modules.polls.maximum_options) {
            bail!("poll option count is outside configured limits")
        }
        let minutes = i
            .integer("minutes")
            .and_then(|v| u64::try_from(v).ok())
            .unwrap_or(u64::from(c.modules.polls.default_duration_minutes));
        let anonymous = matches!(
            i.options.get("anonymous"),
            Some(bot_discord::CommandValue::Boolean(true))
        );
        if anonymous && !c.modules.polls.allow_anonymous {
            bail!("anonymous polls are disabled")
        }
        let channel = i.channel_id.context("poll channel missing")?;
        let poll = ctx
            .services
            .create_poll(
                i.guild_id,
                channel,
                i.actor_id,
                q,
                &options,
                anonymous,
                minutes.saturating_mul(60),
            )
            .await?;
        let buttons = options
            .iter()
            .enumerate()
            .map(|(index, label)| MessageButton {
                custom_id: format!("{PREFIX}{}:{index}", poll.id),
                label: label.clone(),
                style: MessageButtonStyle::Primary,
            })
            .collect::<Vec<_>>();
        let message_id = ctx
            .discord
            .send_message_with_buttons(
                channel,
                &format!(
                    "**{}**\nVote below · closes in {minutes} minute(s){}",
                    safe(q),
                    if anonymous { " · anonymous" } else { "" }
                ),
                &buttons,
            )
            .await?;
        ctx.services
            .attach_poll_message(i.guild_id, poll.id, message_id)
            .await?;
        ctx.discord
            .respond_to_command(
                i,
                CommandResponse::ephemeral(format!("Poll #{} created.", poll.id)),
            )
            .await?;
        Ok(())
    }
    async fn on_component(
        &self,
        ctx: &ModuleContext,
        _: &GuildConfig,
        i: &ComponentInteraction,
    ) -> Result<Option<ModuleEventOutcome>> {
        let value = i
            .custom_id
            .strip_prefix(PREFIX)
            .context("invalid poll vote")?;
        let (id, option) = value.split_once(':').context("invalid poll vote")?;
        let id = id.parse::<i64>()?;
        let option = option.parse::<usize>()?;
        let Some(poll) = ctx
            .services
            .vote_poll(i.guild_id, id, i.actor_id, option)
            .await?
        else {
            ctx.discord
                .respond_to_component(i, CommandResponse::ephemeral("This poll has closed."))
                .await?;
            return Ok(None);
        };
        let tally = poll
            .options
            .iter()
            .zip(poll.counts.iter())
            .map(|(name, count)| format!("{}: {count}", safe(name)))
            .collect::<Vec<_>>()
            .join(" · ");
        ctx.discord
            .respond_to_component(
                i,
                CommandResponse::ephemeral(format!(
                    "Vote recorded. {tally} · {} total",
                    poll.total_votes
                )),
            )
            .await?;
        Ok(Some(ModuleEventOutcome {
            action: "poll.voted".into(),
            target_type: "poll".into(),
            target_id: Some(id.to_string()),
        }))
    }
    async fn on_scheduled_tick(
        &self,
        ctx: &ModuleContext,
        c: &GuildConfig,
        event: &ScheduledTick,
    ) -> Result<Vec<ModuleEventOutcome>> {
        let guild_id = c.guild_id.parse()?;
        let due = ctx
            .services
            .claim_due_polls(guild_id, event.now_unix, 25)
            .await?;
        let mut outcomes = Vec::with_capacity(due.len());
        for poll in due {
            ctx.discord
                .edit_message_with_buttons(
                    poll.channel_id,
                    poll.message_id,
                    &closed_poll_content(&poll),
                    &[],
                )
                .await?;
            outcomes.push(ModuleEventOutcome {
                action: "poll.expired".into(),
                target_type: "poll".into(),
                target_id: Some(poll.id.to_string()),
            });
        }
        Ok(outcomes)
    }
}
fn closed_poll_content(poll: &DuePollResult) -> String {
    let results = poll
        .options
        .iter()
        .zip(&poll.counts)
        .map(|(option, count)| format!("• {} — {count}", safe(option)))
        .collect::<Vec<_>>()
        .join("\n");
    format!(
        "**{}**\nPoll closed · {} total vote(s){}\n{}",
        safe(&poll.question),
        poll.total_votes,
        if poll.anonymous { " · anonymous" } else { "" },
        results
    )
}
fn safe(v: &str) -> String {
    v.replace('@', "@\u{200b}")
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn exposes_create_and_close_commands() {
        assert_eq!(
            COMMANDS.iter().map(|c| c.name).collect::<Vec<_>>(),
            vec!["poll", "poll-close"]
        );
    }
    #[test]
    fn neutralizes_mentions() {
        assert!(safe("@here").contains('\u{200b}'));
    }
    #[test]
    fn closed_results_include_every_axis_of_the_vote() {
        let content = closed_poll_content(&DuePollResult {
            id: 1,
            channel_id: 2,
            message_id: 3,
            question: "Ship it?".into(),
            options: vec!["Yes".into(), "No".into()],
            counts: vec![4, 1],
            total_votes: 5,
            anonymous: true,
        });
        assert!(content.contains("Poll closed · 5 total vote(s) · anonymous"));
        assert!(content.contains("• Yes — 4"));
        assert!(content.contains("• No — 1"));
    }
}

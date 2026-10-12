use anyhow::{Context, Result};
use async_trait::async_trait;
use bot_core::GuildConfig;
use bot_discord::{
    CommandDefinition, CommandInteraction, CommandResponse, ModalDefinition, ModalFieldDefinition,
    ModalFieldStyle, ModalSubmitInteraction,
};
use bot_module_api::{
    BotModule, FormAnswerInput, FormSubmissionInput, ModuleContext, ModuleEventOutcome,
};

pub struct ModerationAppealsModule;
const MODAL_ID: &str = "orivane:moderation-appeal";
pub const COMMANDS: &[CommandDefinition] = &[CommandDefinition {
    name: "appeal",
    description: "Ask the moderation team to review an action",
    guild_only: true,
    default_permissions: None,
    options: &[],
}];

#[async_trait]
impl BotModule for ModerationAppealsModule {
    fn name(&self) -> &'static str {
        "moderation_appeals"
    }
    fn enabled(&self, config: &GuildConfig) -> bool {
        config.modules.moderation_appeals.enabled
    }
    fn commands(&self) -> &'static [CommandDefinition] {
        COMMANDS
    }
    fn handles_modal_submit(&self, custom_id: &str) -> bool {
        custom_id == MODAL_ID
    }
    async fn on_command(
        &self,
        context: &ModuleContext,
        config: &GuildConfig,
        interaction: &CommandInteraction,
    ) -> Result<()> {
        if interaction.command_name != "appeal" {
            return Ok(());
        }
        context
            .discord
            .open_modal(
                interaction,
                ModalDefinition {
                    custom_id: MODAL_ID.into(),
                    title: "Moderation appeal".into(),
                    fields: vec![
                        ModalFieldDefinition {
                            custom_id: "case".into(),
                            label: "Case number or action".into(),
                            style: ModalFieldStyle::Short,
                            required: true,
                            min_length: 1,
                            max_length: 100,
                            placeholder: "Case #42, ban, timeout…".into(),
                        },
                        ModalFieldDefinition {
                            custom_id: "reason".into(),
                            label: "Why should this be reviewed?".into(),
                            style: ModalFieldStyle::Paragraph,
                            required: true,
                            min_length: 10,
                            max_length: 1000,
                            placeholder: config.modules.moderation_appeals.prompt.clone(),
                        },
                        ModalFieldDefinition {
                            custom_id: "context".into(),
                            label: "Additional context".into(),
                            style: ModalFieldStyle::Paragraph,
                            required: false,
                            min_length: 0,
                            max_length: 800,
                            placeholder: "Evidence, links, or anything the team should know".into(),
                        },
                    ],
                },
            )
            .await?;
        Ok(())
    }
    async fn on_modal_submit(
        &self,
        context: &ModuleContext,
        config: &GuildConfig,
        interaction: &ModalSubmitInteraction,
    ) -> Result<Option<ModuleEventOutcome>> {
        if !context
            .services
            .claim_appeal_cooldown(
                interaction.guild_id,
                interaction.actor_id,
                u64::from(config.modules.moderation_appeals.cooldown_hours) * 3600,
            )
            .await?
        {
            context
                .discord
                .respond_to_modal(
                    interaction,
                    CommandResponse::ephemeral(format!(
                        "You may submit one appeal every {} hour(s).",
                        config.modules.moderation_appeals.cooldown_hours
                    )),
                )
                .await?;
            return Ok(None);
        }
        let case = interaction
            .values
            .get("case")
            .context("case is required")?
            .trim();
        let reason = interaction
            .values
            .get("reason")
            .context("appeal reason is required")?
            .trim();
        let extra = interaction
            .values
            .get("context")
            .map(String::as_str)
            .unwrap_or("")
            .trim();
        let channel: u64 = config
            .modules
            .moderation_appeals
            .review_channel_id
            .as_deref()
            .context("appeal review channel is required")?
            .parse()
            .context("invalid appeal review channel")?;
        let id = context
            .services
            .create_form_submission(FormSubmissionInput {
                guild_id: interaction.guild_id,
                form_id: "moderation-appeals".into(),
                submitter_id: Some(interaction.actor_id),
                answers: vec![
                    FormAnswerInput {
                        question_id: "case".into(),
                        label: "Case or action".into(),
                        answer: case.into(),
                    },
                    FormAnswerInput {
                        question_id: "reason".into(),
                        label: "Appeal".into(),
                        answer: reason.into(),
                    },
                    FormAnswerInput {
                        question_id: "context".into(),
                        label: "Additional context".into(),
                        answer: extra.into(),
                    },
                ],
            })
            .await?;
        context.discord.send_message(channel, &format!("**Moderation appeal #{id}**\nSubmitted by: <@{}>\n**Case or action**\n{}\n**Appeal**\n{}\n**Additional context**\n{}", interaction.actor_id, safe(case), safe(reason), if extra.is_empty(){"—".into()}else{safe(extra)})).await?;
        context
            .discord
            .respond_to_modal(
                interaction,
                CommandResponse::ephemeral(format!(
                    "Appeal #{id} was submitted to the moderation team."
                )),
            )
            .await?;
        Ok(Some(ModuleEventOutcome {
            action: "moderation_appeal.submitted".into(),
            target_type: "moderation_appeal".into(),
            target_id: Some(id.to_string()),
        }))
    }
}
fn safe(value: &str) -> String {
    value
        .replace('@', "@\u{200b}")
        .replace("```", "`\u{200b}`\u{200b}`")
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn appeal_is_slash_only_and_guild_scoped() {
        assert_eq!(COMMANDS.len(), 1);
        assert_eq!(COMMANDS[0].name, "appeal");
        assert!(COMMANDS[0].guild_only);
    }
    #[test]
    fn sanitizes_mentions_and_fences() {
        let value = safe("@everyone ```x```");
        assert!(!value.contains("@everyone"));
        assert!(!value.contains("```"));
    }
}

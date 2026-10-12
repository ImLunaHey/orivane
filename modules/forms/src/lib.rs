use anyhow::{Context, Result, bail};
use async_trait::async_trait;
use bot_core::{FormConfig, GuildConfig};
use bot_discord::{
    CommandDefinition, CommandInteraction, CommandOptionDefinition, CommandOptionKind,
    CommandResponse, ModalDefinition, ModalFieldDefinition, ModalFieldStyle,
    ModalSubmitInteraction,
};
use bot_module_api::{
    BotModule, FormAnswerInput, FormSubmissionInput, ModuleContext, ModuleEventOutcome,
};

pub struct FormsModule;

const MODAL_PREFIX: &str = "orivane:form:";

pub const COMMANDS: &[CommandDefinition] = &[CommandDefinition {
    name: "form",
    description: "Open a server form",
    guild_only: true,
    default_permissions: None,
    options: &[CommandOptionDefinition {
        name: "name",
        description: "Form name or ID",
        kind: CommandOptionKind::String,
        required: true,
        min_value: None,
        max_value: None,
        min_length: Some(1),
        max_length: Some(64),
    }],
}];

#[async_trait]
impl BotModule for FormsModule {
    fn name(&self) -> &'static str {
        "forms"
    }
    fn enabled(&self, config: &GuildConfig) -> bool {
        config.modules.forms.enabled
    }
    fn commands(&self) -> &'static [CommandDefinition] {
        COMMANDS
    }
    fn handles_modal_submit(&self, custom_id: &str) -> bool {
        custom_id.starts_with(MODAL_PREFIX)
    }

    async fn on_command(
        &self,
        context: &ModuleContext,
        config: &GuildConfig,
        interaction: &CommandInteraction,
    ) -> Result<()> {
        if interaction.command_name != "form" {
            return Ok(());
        }
        let query = interaction
            .string("name")
            .context("form name is required")?;
        let Some(form) = config.modules.forms.forms.iter().find(|form| {
            form.enabled
                && (form.id.eq_ignore_ascii_case(query) || form.name.eq_ignore_ascii_case(query))
        }) else {
            context
                .discord
                .respond_to_command(
                    interaction,
                    CommandResponse::ephemeral(
                        "No enabled form with that name exists in this server.",
                    ),
                )
                .await?;
            return Ok(());
        };
        if !allowed(form, &interaction.actor_role_ids) {
            context
                .discord
                .respond_to_command(
                    interaction,
                    CommandResponse::ephemeral("You do not have a role allowed to use this form."),
                )
                .await?;
            return Ok(());
        }
        context
            .discord
            .open_modal(
                interaction,
                ModalDefinition {
                    custom_id: format!("{MODAL_PREFIX}{}", form.id),
                    title: form.name.clone(),
                    fields: form
                        .questions
                        .iter()
                        .map(|question| ModalFieldDefinition {
                            custom_id: question.id.clone(),
                            label: question.label.clone(),
                            style: if question.style == "paragraph" {
                                ModalFieldStyle::Paragraph
                            } else {
                                ModalFieldStyle::Short
                            },
                            required: question.required,
                            min_length: question.min_length,
                            max_length: question.max_length,
                            placeholder: question.placeholder.clone(),
                        })
                        .collect(),
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
        let id = interaction
            .custom_id
            .strip_prefix(MODAL_PREFIX)
            .context("invalid form modal ID")?;
        let form = config
            .modules
            .forms
            .forms
            .iter()
            .find(|form| form.enabled && form.id == id)
            .context("form is no longer enabled")?;
        if !allowed(form, &interaction.actor_role_ids) {
            bail!("member no longer has a role allowed to use this form");
        }
        for question in &form.questions {
            let value = interaction
                .values
                .get(&question.id)
                .map(String::as_str)
                .unwrap_or_default();
            let length = value.chars().count();
            if (question.required && value.trim().is_empty())
                || length < usize::from(question.min_length)
                || length > usize::from(question.max_length)
            {
                bail!("submitted form field is invalid");
            }
        }
        let channel_id = form
            .destination_channel_id
            .parse::<u64>()
            .context("form destination channel is invalid")?;
        for chunk in submission_chunks(form, interaction) {
            context.discord.send_message(channel_id, &chunk).await?;
        }
        let submission_id = context
            .services
            .create_form_submission(FormSubmissionInput {
                guild_id: interaction.guild_id,
                form_id: form.id.clone(),
                submitter_id: (!form.anonymous).then_some(interaction.actor_id),
                answers: form
                    .questions
                    .iter()
                    .map(|question| FormAnswerInput {
                        question_id: question.id.clone(),
                        label: question.label.clone(),
                        answer: interaction
                            .values
                            .get(&question.id)
                            .cloned()
                            .unwrap_or_default(),
                    })
                    .collect(),
            })
            .await?;
        context
            .discord
            .respond_to_modal(
                interaction,
                CommandResponse::ephemeral("Your form was submitted successfully."),
            )
            .await?;
        Ok(Some(ModuleEventOutcome {
            action: "form.submitted".into(),
            target_type: "form_submission".into(),
            target_id: Some(submission_id.to_string()),
        }))
    }
}

fn allowed(form: &FormConfig, actor_roles: &[u64]) -> bool {
    form.allowed_role_ids.is_empty()
        || form
            .allowed_role_ids
            .iter()
            .any(|role| actor_roles.iter().any(|id| role == &id.to_string()))
}

fn submission_chunks(form: &FormConfig, interaction: &ModalSubmitInteraction) -> Vec<String> {
    let submitter = if form.anonymous {
        "Anonymous".to_owned()
    } else {
        format!("<@{}>", interaction.actor_id)
    };
    let mut sections = vec![format!(
        "**Form submission: {}**\nSubmitted by: {submitter}",
        safe(&form.name)
    )];
    sections.extend(form.questions.iter().map(|question| {
        let answer = interaction
            .values
            .get(&question.id)
            .map(String::as_str)
            .unwrap_or("—");
        format!("**{}**\n{}", safe(&question.label), safe(answer))
    }));
    let mut chunks = Vec::new();
    let mut current = String::new();
    for section in sections {
        for part in split_chars(&section, 1800) {
            if !current.is_empty() && current.chars().count() + part.chars().count() + 2 > 1900 {
                chunks.push(std::mem::take(&mut current));
            }
            if !current.is_empty() {
                current.push_str("\n\n");
            }
            current.push_str(&part);
        }
    }
    if !current.is_empty() {
        chunks.push(current);
    }
    chunks
}

fn safe(value: &str) -> String {
    value.replace('@', "@\u{200b}")
}
fn split_chars(value: &str, limit: usize) -> Vec<String> {
    let chars = value.chars().collect::<Vec<_>>();
    chars
        .chunks(limit)
        .map(|chunk| chunk.iter().collect())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use bot_core::{FormQuestionConfig, FormsConfig};
    use bot_discord::CommandValue;
    use bot_test_support::{DiscordCall, FakeDiscord, FakeModuleServices, ServiceCall};
    use std::{collections::BTreeMap, sync::Arc};

    fn config() -> GuildConfig {
        let mut config = GuildConfig::for_guild("10");
        config.modules.forms = FormsConfig {
            enabled: true,
            forms: vec![FormConfig {
                enabled: true,
                id: "apply".into(),
                name: "Staff application".into(),
                description: "Apply".into(),
                destination_channel_id: "20".into(),
                allowed_role_ids: vec!["30".into()],
                anonymous: false,
                questions: vec![FormQuestionConfig {
                    id: "why".into(),
                    label: "Why?".into(),
                    style: "paragraph".into(),
                    required: true,
                    min_length: 2,
                    max_length: 4000,
                    placeholder: "Tell us".into(),
                }],
            }],
        };
        config
    }

    #[tokio::test]
    async fn opens_native_modal_and_routes_a_safe_submission() {
        let discord = FakeDiscord::new();
        let services = FakeModuleServices::default();
        let context = ModuleContext {
            discord: Arc::new(discord.clone()),
            services: Arc::new(services.clone()),
        };
        let command = CommandInteraction {
            id: 1,
            token: "x".into(),
            guild_id: 10,
            channel_id: Some(11),
            actor_id: 40,
            actor_role_ids: vec![30],
            command_name: "form".into(),
            options: BTreeMap::from([("name".into(), CommandValue::String("apply".into()))]),
        };
        FormsModule
            .on_command(&context, &config(), &command)
            .await
            .unwrap();
        assert!(
            matches!(&discord.calls()[0], DiscordCall::OpenModal { modal, .. } if modal.custom_id == "orivane:form:apply" && modal.fields.len() == 1)
        );
        discord.take_calls();
        let submit = ModalSubmitInteraction {
            id: 2,
            token: "y".into(),
            guild_id: 10,
            actor_id: 40,
            actor_role_ids: vec![30],
            custom_id: "orivane:form:apply".into(),
            values: BTreeMap::from([("why".into(), "Hi @everyone".into())]),
        };
        let outcome = FormsModule
            .on_modal_submit(&context, &config(), &submit)
            .await
            .unwrap();
        let outcome = outcome.unwrap();
        assert_eq!(outcome.action, "form.submitted");
        assert_eq!(outcome.target_type, "form_submission");
        assert!(outcome.target_id.is_some());
        assert!(matches!(
            &services.calls()[0],
            ServiceCall::CreateFormSubmission(input)
                if input.guild_id == 10
                    && input.form_id == "apply"
                    && input.submitter_id == Some(40)
                    && input.answers[0].answer == "Hi @everyone"
        ));
        assert!(
            matches!(&discord.calls()[0], DiscordCall::SendMessage { channel_id: 20, content } if content.contains("@\u{200b}everyone"))
        );
        assert!(matches!(
            &discord.calls()[1],
            DiscordCall::RespondToModal {
                interaction_id: 2,
                ..
            }
        ));
    }

    #[tokio::test]
    async fn denies_members_without_an_allowed_role() {
        let discord = FakeDiscord::new();
        let context = ModuleContext {
            discord: Arc::new(discord.clone()),
            services: Arc::new(FakeModuleServices::default()),
        };
        let command = CommandInteraction {
            id: 1,
            token: "x".into(),
            guild_id: 10,
            channel_id: None,
            actor_id: 40,
            actor_role_ids: vec![],
            command_name: "form".into(),
            options: BTreeMap::from([("name".into(), CommandValue::String("apply".into()))]),
        };
        FormsModule
            .on_command(&context, &config(), &command)
            .await
            .unwrap();
        assert!(
            matches!(&discord.calls()[0], DiscordCall::RespondToCommand { response, .. } if response.ephemeral)
        );
    }
}

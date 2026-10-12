use anyhow::{Context, Result, bail};
use async_trait::async_trait;
use bot_core::GuildConfig;
use bot_discord::{
    CommandDefinition, CommandInteraction, CommandOptionDefinition, CommandOptionKind,
    CommandResponse, ComponentInteraction, MessageButton, MessageButtonStyle,
};
use bot_module_api::{
    BotModule, ModuleContext, ModuleEventOutcome, SuggestionInput, SuggestionResult,
};

pub struct SuggestionsModule;

const COMPONENT_PREFIX: &str = "orivane:suggestion:vote:";

pub const COMMANDS: &[CommandDefinition] = &[
    CommandDefinition {
        name: "suggest",
        description: "Submit a suggestion for this server",
        guild_only: true,
        default_permissions: None,
        options: &[
            CommandOptionDefinition {
                name: "content",
                description: "Your suggestion",
                kind: CommandOptionKind::String,
                required: true,
                min_value: None,
                max_value: None,
                min_length: Some(1),
                max_length: Some(1800),
            },
            CommandOptionDefinition {
                name: "anonymous",
                description: "Hide your identity from the suggestion",
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
        name: "suggestion-status",
        description: "Update a suggestion's staff status",
        guild_only: true,
        default_permissions: None,
        options: &[
            CommandOptionDefinition {
                name: "id",
                description: "Suggestion ID",
                kind: CommandOptionKind::Integer,
                required: true,
                min_value: Some(1),
                max_value: None,
                min_length: None,
                max_length: None,
            },
            CommandOptionDefinition {
                name: "status",
                description: "approved, rejected, implemented, duplicate, or pending",
                kind: CommandOptionKind::String,
                required: true,
                min_value: None,
                max_value: None,
                min_length: Some(1),
                max_length: Some(16),
            },
            CommandOptionDefinition {
                name: "note",
                description: "Optional staff context",
                kind: CommandOptionKind::String,
                required: false,
                min_value: None,
                max_value: None,
                min_length: Some(1),
                max_length: Some(500),
            },
        ],
    },
];

#[async_trait]
impl BotModule for SuggestionsModule {
    fn name(&self) -> &'static str {
        "suggestions"
    }
    fn enabled(&self, config: &GuildConfig) -> bool {
        config.modules.suggestions.enabled
    }
    fn commands(&self) -> &'static [CommandDefinition] {
        COMMANDS
    }
    fn handles_component(&self, custom_id: &str) -> bool {
        custom_id.starts_with(COMPONENT_PREFIX)
    }

    async fn on_command(
        &self,
        context: &ModuleContext,
        config: &GuildConfig,
        interaction: &CommandInteraction,
    ) -> Result<()> {
        match interaction.command_name.as_str() {
            "suggest" => submit(context, config, interaction).await,
            "suggestion-status" => update_status(context, config, interaction).await,
            _ => Ok(()),
        }
    }

    async fn on_component(
        &self,
        context: &ModuleContext,
        _config: &GuildConfig,
        interaction: &ComponentInteraction,
    ) -> Result<Option<ModuleEventOutcome>> {
        let value = interaction
            .custom_id
            .strip_prefix(COMPONENT_PREFIX)
            .context("invalid suggestion vote")?;
        let (id, direction) = value.rsplit_once(':').context("invalid suggestion vote")?;
        let suggestion_id = id.parse::<i64>().context("invalid suggestion ID")?;
        let direction = match direction {
            "up" => 1,
            "down" => -1,
            _ => bail!("invalid suggestion vote direction"),
        };
        let Some(result) = context
            .services
            .vote_suggestion(
                interaction.guild_id,
                suggestion_id,
                interaction.actor_id,
                direction,
            )
            .await?
        else {
            context
                .discord
                .respond_to_component(
                    interaction,
                    CommandResponse::ephemeral("This suggestion is no longer accepting votes."),
                )
                .await?;
            return Ok(None);
        };
        if let Some(message_id) = result.message_id {
            context
                .discord
                .edit_message_with_buttons(
                    result.channel_id,
                    message_id,
                    &render_suggestion(&result),
                    &suggestion_buttons(&result),
                )
                .await?;
        }
        context
            .discord
            .respond_to_component(
                interaction,
                CommandResponse::ephemeral(format!(
                    "Vote recorded. {} up · {} down",
                    result.upvotes, result.downvotes
                )),
            )
            .await?;
        Ok(Some(ModuleEventOutcome {
            action: "suggestion.voted".into(),
            target_type: "suggestion".into(),
            target_id: Some(suggestion_id.to_string()),
        }))
    }
}

async fn submit(
    context: &ModuleContext,
    config: &GuildConfig,
    interaction: &CommandInteraction,
) -> Result<()> {
    let settings = &config.modules.suggestions;
    let content = interaction
        .string("content")
        .context("suggestion content is required")?
        .trim();
    if content.is_empty() {
        bail!("suggestion content is empty");
    }
    let anonymous = interaction
        .options
        .get("anonymous")
        .is_some_and(|value| matches!(value, bot_discord::CommandValue::Boolean(true)));
    if anonymous && !settings.allow_anonymous {
        context
            .discord
            .respond_to_command(
                interaction,
                CommandResponse::ephemeral("Anonymous suggestions are disabled in this server."),
            )
            .await?;
        return Ok(());
    }
    if !context
        .services
        .claim_suggestion_cooldown(
            interaction.guild_id,
            interaction.actor_id,
            settings.cooldown_seconds,
        )
        .await?
    {
        context
            .discord
            .respond_to_command(
                interaction,
                CommandResponse::ephemeral(format!(
                    "Please wait {} seconds between suggestions.",
                    settings.cooldown_seconds
                )),
            )
            .await?;
        return Ok(());
    }
    let channel_id = settings
        .channel_id
        .as_deref()
        .context("suggestions channel is required")?
        .parse::<u64>()
        .context("suggestions channel is invalid")?;
    let suggestion = context
        .services
        .create_suggestion(SuggestionInput {
            guild_id: interaction.guild_id,
            channel_id,
            author_id: (!anonymous).then_some(interaction.actor_id),
            content: content.to_owned(),
        })
        .await?;
    let message = render_suggestion(&suggestion);
    let message_id = context
        .discord
        .send_message_with_buttons(channel_id, &message, &suggestion_buttons(&suggestion))
        .await?;
    context
        .services
        .set_suggestion_message_id(interaction.guild_id, suggestion.id, message_id)
        .await?;
    context
        .discord
        .respond_to_command(
            interaction,
            CommandResponse::ephemeral(format!("Suggestion #{} was submitted.", suggestion.id)),
        )
        .await?;
    Ok(())
}

async fn update_status(
    context: &ModuleContext,
    config: &GuildConfig,
    interaction: &CommandInteraction,
) -> Result<()> {
    let staff_roles = &config.modules.suggestions.staff_role_ids;
    let is_configured_staff = staff_roles.iter().any(|role| {
        interaction
            .actor_role_ids
            .iter()
            .any(|id| role == &id.to_string())
    });
    let can_manage_guild = interaction.actor_permissions & 32 == 32;
    if (!staff_roles.is_empty() && !is_configured_staff)
        || (staff_roles.is_empty() && !can_manage_guild)
    {
        context
            .discord
            .respond_to_command(
                interaction,
                CommandResponse::ephemeral("You do not have a configured suggestions staff role."),
            )
            .await?;
        return Ok(());
    }
    let id = interaction
        .integer("id")
        .context("suggestion ID is required")?;
    let status = interaction
        .string("status")
        .context("suggestion status is required")?
        .to_ascii_lowercase();
    if !matches!(
        status.as_str(),
        "pending" | "approved" | "rejected" | "implemented" | "duplicate"
    ) {
        bail!("invalid suggestion status");
    }
    let note = interaction.string("note");
    let Some(result) = context
        .services
        .update_suggestion_status(
            interaction.guild_id,
            id,
            interaction.actor_id,
            &status,
            note,
        )
        .await?
    else {
        context
            .discord
            .respond_to_command(
                interaction,
                CommandResponse::ephemeral("Suggestion not found."),
            )
            .await?;
        return Ok(());
    };
    if let Some(message_id) = result.message_id {
        context
            .discord
            .edit_message_with_buttons(
                result.channel_id,
                message_id,
                &render_suggestion(&result),
                &suggestion_buttons(&result),
            )
            .await?;
    }
    context
        .discord
        .respond_to_command(
            interaction,
            CommandResponse::ephemeral(format!(
                "Suggestion #{} is now {}.",
                result.id, result.status
            )),
        )
        .await?;
    Ok(())
}

fn safe(value: &str) -> String {
    value.replace('@', "@\u{200b}")
}

fn render_suggestion(suggestion: &SuggestionResult) -> String {
    let author = suggestion
        .author_id
        .map_or_else(|| "Anonymous".into(), |id| format!("<@{id}>"));
    let note = suggestion
        .staff_note
        .as_deref()
        .filter(|note| !note.is_empty())
        .map(|note| format!("\nStaff note: {}", safe(note)))
        .unwrap_or_default();
    format!(
        "**Suggestion #{}**\n{}\n\nSubmitted by: {author}\nStatus: {}\nVotes: {} up · {} down{note}",
        suggestion.id,
        safe(&suggestion.content),
        title_status(&suggestion.status),
        suggestion.upvotes,
        suggestion.downvotes,
    )
}

fn suggestion_buttons(suggestion: &SuggestionResult) -> Vec<MessageButton> {
    if suggestion.status != "pending" {
        return Vec::new();
    }
    vec![
        MessageButton {
            custom_id: format!("{COMPONENT_PREFIX}{}:up", suggestion.id),
            label: format!("Upvote · {}", suggestion.upvotes),
            style: MessageButtonStyle::Success,
        },
        MessageButton {
            custom_id: format!("{COMPONENT_PREFIX}{}:down", suggestion.id),
            label: format!("Downvote · {}", suggestion.downvotes),
            style: MessageButtonStyle::Danger,
        },
    ]
}

fn title_status(status: &str) -> String {
    let mut chars = status.chars();
    chars
        .next()
        .map(|first| first.to_uppercase().collect::<String>() + chars.as_str())
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;
    use bot_core::SuggestionsConfig;
    use bot_discord::CommandValue;
    use bot_test_support::{DiscordCall, FakeDiscord, FakeModuleServices, ServiceCall};
    use std::{collections::BTreeMap, sync::Arc};

    fn config() -> GuildConfig {
        let mut config = GuildConfig::for_guild("10");
        config.modules.suggestions = SuggestionsConfig {
            enabled: true,
            channel_id: Some("20".into()),
            staff_role_ids: vec!["30".into()],
            allow_anonymous: true,
            cooldown_seconds: 60,
        };
        config
    }

    #[tokio::test]
    async fn submits_with_vote_buttons_and_records_message() {
        let discord = FakeDiscord::new();
        discord.set_next_message_id(99);
        let services = FakeModuleServices::default();
        let context = ModuleContext {
            discord: Arc::new(discord.clone()),
            services: Arc::new(services.clone()),
        };
        let command = CommandInteraction {
            id: 1,
            token: "x".into(),
            guild_id: 10,
            channel_id: Some(20),
            actor_id: 40,
            actor_role_ids: vec![],
            actor_permissions: u64::MAX,
            command_name: "suggest".into(),
            options: BTreeMap::from([(
                "content".into(),
                CommandValue::String("Add @everyone movie nights".into()),
            )]),
        };
        SuggestionsModule
            .on_command(&context, &config(), &command)
            .await
            .unwrap();
        assert!(discord.calls().iter().any(|call| matches!(call, DiscordCall::SendMessageWithButtons { channel_id: 20, content, buttons } if content.contains("@\u{200b}everyone") && buttons.len() == 2)));
        assert!(services.calls().iter().any(|call| matches!(
            call,
            ServiceCall::SetSuggestionMessageId { message_id: 99, .. }
        )));
    }

    #[tokio::test]
    async fn records_button_votes() {
        let discord = FakeDiscord::new();
        let services = FakeModuleServices::default();
        let context = ModuleContext {
            discord: Arc::new(discord.clone()),
            services: Arc::new(services.clone()),
        };
        let interaction = ComponentInteraction {
            id: 2,
            token: "y".into(),
            guild_id: 10,
            channel_id: Some(20),
            actor_id: 50,
            actor_role_ids: vec![],
            custom_id: "orivane:suggestion:vote:7:up".into(),
            values: vec![],
        };
        let outcome = SuggestionsModule
            .on_component(&context, &config(), &interaction)
            .await
            .unwrap()
            .unwrap();
        assert_eq!(outcome.action, "suggestion.voted");
        assert!(matches!(
            &services.calls()[0],
            ServiceCall::VoteSuggestion {
                suggestion_id: 7,
                direction: 1,
                ..
            }
        ));
        assert!(discord.calls().iter().any(|call| matches!(
            call,
            DiscordCall::EditMessageWithButtons {
                channel_id: 20,
                message_id: 50,
                content,
                buttons,
            } if content.contains("Votes: 1 up · 0 down")
                && buttons.iter().any(|button| button.label == "Upvote · 1")
        )));
    }

    #[tokio::test]
    async fn limits_status_changes_to_configured_staff_roles() {
        let discord = FakeDiscord::new();
        let services = FakeModuleServices::default();
        let context = ModuleContext {
            discord: Arc::new(discord.clone()),
            services: Arc::new(services.clone()),
        };
        let mut command = CommandInteraction {
            id: 3,
            token: "z".into(),
            guild_id: 10,
            channel_id: Some(11),
            actor_id: 40,
            actor_role_ids: vec![],
            actor_permissions: u64::MAX,
            command_name: "suggestion-status".into(),
            options: BTreeMap::from([
                ("id".into(), CommandValue::Integer(7)),
                ("status".into(), CommandValue::String("approved".into())),
            ]),
        };
        SuggestionsModule
            .on_command(&context, &config(), &command)
            .await
            .unwrap();
        assert!(
            !services
                .calls()
                .iter()
                .any(|call| matches!(call, ServiceCall::UpdateSuggestionStatus { .. }))
        );
        command.actor_role_ids.push(30);
        SuggestionsModule
            .on_command(&context, &config(), &command)
            .await
            .unwrap();
        assert!(services.calls().iter().any(|call| matches!(call, ServiceCall::UpdateSuggestionStatus { suggestion_id: 7, status, .. } if status == "approved")));
        assert!(discord.calls().iter().any(|call| matches!(
            call,
            DiscordCall::EditMessageWithButtons {
                channel_id: 20,
                message_id: 50,
                content,
                buttons,
            } if content.contains("Status: Approved") && buttons.is_empty()
        )));
    }
}

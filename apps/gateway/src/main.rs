use std::{
    collections::BTreeMap,
    env,
    sync::Arc,
    time::{Instant, SystemTime, UNIX_EPOCH},
};

use anyhow::{Context, Result};
use async_trait::async_trait;
use bot_core::{Infrastructure, NewAuditEvent, NewFormSubmission, NewModerationCase, level_for_xp};
use bot_discord::{
    CommandDefinition, CommandInteraction, CommandResponse, CommandValue, DiscordApi,
    ModalSubmitInteraction, TwilightDiscord,
};
use bot_module_anti_raid::AntiRaidModule;
use bot_module_api::{
    AuditNotice, BotModule, FormSubmissionInput, MemberJoin, MemberLeave, MessageCreate,
    ModerationCaseInput, ModuleContext, ModuleServices, ReactionAdd, ScheduledTick,
    TicketRecordInput, XpAwardInput, XpAwardResult, XpProfile,
};
use bot_module_automod::AutomodModule;
use bot_module_autoresponder::AutoresponderModule;
use bot_module_autorole::AutoroleModule;
use bot_module_custom_commands::CustomCommandsModule;
use bot_module_forms::FormsModule;
use bot_module_leveling::LevelingModule;
use bot_module_logging::LoggingModule;
use bot_module_moderation::ModerationModule;
use bot_module_reaction_roles::ReactionRolesModule;
use bot_module_scheduled_messages::ScheduledMessagesModule;
use bot_module_starboard::StarboardModule;
use bot_module_tickets::TicketsModule;
use bot_module_utility::UtilityModule;
use bot_module_welcome::WelcomeModule;
use twilight_gateway::{Event, EventTypeFlags, Intents, Shard, ShardId, StreamExt as _};
use twilight_model::application::interaction::{
    InteractionData,
    application_command::{CommandDataOption, CommandOptionValue as TwilightCommandValue},
    modal::ModalInteractionComponent,
};
use twilight_model::channel::message::EmojiReactionType;
use twilight_model::gateway::payload::incoming::GuildCreate;

#[derive(Clone)]
struct GatewayModuleServices {
    infrastructure: Infrastructure,
}

#[async_trait]
impl ModuleServices for GatewayModuleServices {
    async fn create_form_submission(&self, input: FormSubmissionInput) -> Result<i64> {
        let answers = serde_json::to_value(
            input
                .answers
                .iter()
                .map(|answer| {
                    serde_json::json!({
                        "question_id": answer.question_id,
                        "label": answer.label,
                        "answer": answer.answer,
                    })
                })
                .collect::<Vec<_>>(),
        )?;
        let submission = self
            .infrastructure
            .create_form_submission(&NewFormSubmission {
                guild_id: input.guild_id.to_string(),
                form_id: input.form_id,
                submitter_id: input.submitter_id.map(|id| id.to_string()),
                answers,
            })
            .await?;
        Ok(submission.id)
    }

    async fn create_moderation_case(&self, input: ModerationCaseInput) -> Result<i64> {
        let guild_id = input.guild_id.to_string();
        let moderator_id = input.moderator_id.to_string();
        let member_id = input.member_id.to_string();
        let case_input = NewModerationCase {
            member_id: member_id.clone(),
            action: input.action.clone(),
            reason: input.reason.clone(),
        };
        let mut audit = NewAuditEvent::guild(
            &guild_id,
            &moderator_id,
            format!("moderation.{}", input.action),
        );
        audit.source = "discord_gateway".to_owned();
        audit.target_type = "member".to_owned();
        audit.target_id = Some(member_id);
        audit.reason = Some(input.reason);
        audit.metadata = serde_json::json!({ "action": input.action });
        let case = self
            .infrastructure
            .create_moderation_case_audited(&guild_id, &moderator_id, &case_input, &audit)
            .await?;
        Ok(case.id)
    }

    async fn award_xp(&self, input: XpAwardInput) -> Result<XpAwardResult> {
        let result = self
            .infrastructure
            .award_member_xp(
                &input.guild_id.to_string(),
                &input.user_id.to_string(),
                input.amount,
                input.cooldown_seconds,
            )
            .await?;
        Ok(XpAwardResult {
            awarded: result.awarded,
            total_xp: result.total_xp,
            level: result.level,
            leveled_up: result.leveled_up,
        })
    }

    async fn xp_profile(&self, guild_id: u64, user_id: u64) -> Result<XpProfile> {
        let profile = self
            .infrastructure
            .member_xp_profile(&guild_id.to_string(), &user_id.to_string())
            .await?;
        let total_xp = u64::try_from(profile.total_xp).unwrap_or_default();
        Ok(XpProfile {
            user_id,
            total_xp,
            level: level_for_xp(total_xp),
            message_count: u64::try_from(profile.message_count).unwrap_or_default(),
            rank: u64::try_from(profile.rank).unwrap_or_default(),
        })
    }

    async fn xp_leaderboard(&self, guild_id: u64, limit: u8) -> Result<Vec<XpProfile>> {
        self.infrastructure
            .xp_leaderboard(&guild_id.to_string(), i64::from(limit))
            .await?
            .into_iter()
            .map(|profile| {
                let user_id = profile
                    .user_id
                    .parse::<u64>()
                    .context("stored XP user is invalid")?;
                let total_xp = u64::try_from(profile.total_xp).unwrap_or_default();
                Ok(XpProfile {
                    user_id,
                    total_xp,
                    level: level_for_xp(total_xp),
                    message_count: u64::try_from(profile.message_count).unwrap_or_default(),
                    rank: u64::try_from(profile.rank).unwrap_or_default(),
                })
            })
            .collect()
    }

    async fn open_ticket_channel(&self, guild_id: u64, opener_id: u64) -> Result<Option<u64>> {
        self.infrastructure
            .open_ticket_channel(&guild_id.to_string(), &opener_id.to_string())
            .await?
            .map(|channel_id| {
                channel_id
                    .parse()
                    .context("stored ticket channel is invalid")
            })
            .transpose()
    }

    async fn record_open_ticket(&self, input: TicketRecordInput) -> Result<()> {
        self.infrastructure
            .record_open_ticket(
                &input.guild_id.to_string(),
                &input.opener_id.to_string(),
                &input.channel_id.to_string(),
            )
            .await?;
        Ok(())
    }

    async fn close_ticket_channel(
        &self,
        guild_id: u64,
        channel_id: u64,
        closed_by: u64,
    ) -> Result<bool> {
        Ok(self
            .infrastructure
            .close_ticket_channel(
                &guild_id.to_string(),
                &channel_id.to_string(),
                &closed_by.to_string(),
            )
            .await?)
    }

    async fn claim_starboard_post(
        &self,
        guild_id: u64,
        source_message_id: u64,
        starboard_channel_id: u64,
    ) -> Result<bool> {
        Ok(self
            .infrastructure
            .claim_starboard_post(
                &guild_id.to_string(),
                &source_message_id.to_string(),
                &starboard_channel_id.to_string(),
            )
            .await?)
    }

    async fn record_raid_join(
        &self,
        guild_id: u64,
        user_id: u64,
        window_seconds: u16,
    ) -> Result<u64> {
        Ok(self
            .infrastructure
            .record_raid_join(&guild_id.to_string(), &user_id.to_string(), window_seconds)
            .await?)
    }

    async fn set_raid_lockdown(&self, guild_id: u64, duration_seconds: u64) -> Result<()> {
        Ok(self
            .infrastructure
            .set_raid_lockdown(&guild_id.to_string(), duration_seconds)
            .await?)
    }

    async fn clear_raid_lockdown(&self, guild_id: u64) -> Result<()> {
        Ok(self
            .infrastructure
            .clear_raid_lockdown(&guild_id.to_string())
            .await?)
    }

    async fn raid_lockdown_active(&self, guild_id: u64) -> Result<bool> {
        Ok(self
            .infrastructure
            .raid_lockdown_active(&guild_id.to_string())
            .await?)
    }

    async fn claim_scheduled_message(
        &self,
        guild_id: u64,
        schedule_id: &str,
        occurrence_unix: u64,
    ) -> Result<bool> {
        Ok(self
            .infrastructure
            .claim_scheduled_message(&guild_id.to_string(), schedule_id, occurrence_unix)
            .await?)
    }

    async fn release_scheduled_message(
        &self,
        guild_id: u64,
        schedule_id: &str,
        occurrence_unix: u64,
    ) -> Result<()> {
        Ok(self
            .infrastructure
            .release_scheduled_message(&guild_id.to_string(), schedule_id, occurrence_unix)
            .await?)
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    init_tracing();
    let token = required_env("DISCORD_TOKEN")?;
    let database_url = required_env("DATABASE_URL")?;
    let redis_url = required_env("REDIS_URL")?;

    let modules: Vec<Arc<dyn BotModule>> = vec![
        Arc::new(UtilityModule),
        Arc::new(WelcomeModule),
        Arc::new(AutoroleModule),
        Arc::new(LoggingModule),
        Arc::new(ModerationModule),
        Arc::new(AutomodModule),
        Arc::new(LevelingModule),
        Arc::new(StarboardModule),
        Arc::new(TicketsModule),
        Arc::new(CustomCommandsModule),
        Arc::new(ReactionRolesModule),
        Arc::new(AutoresponderModule),
        Arc::new(AntiRaidModule::default()),
        Arc::new(ScheduledMessagesModule),
        Arc::new(FormsModule),
    ];
    let discord: Arc<dyn DiscordApi> = Arc::new(TwilightDiscord::connect(token.clone()).await?);
    register_commands(discord.as_ref(), &modules).await?;
    let infrastructure = Infrastructure::connect(&database_url, &redis_url).await?;
    let context = ModuleContext {
        discord,
        services: Arc::new(GatewayModuleServices {
            infrastructure: infrastructure.clone(),
        }),
    };
    let intents = Intents::GUILDS
        | Intents::GUILD_MEMBERS
        | Intents::GUILD_MESSAGES
        | Intents::MESSAGE_CONTENT
        | Intents::GUILD_MESSAGE_REACTIONS;
    let mut shard = Shard::new(ShardId::ONE, token, intents);

    tokio::spawn(run_scheduler(
        context.clone(),
        modules.clone(),
        infrastructure.clone(),
    ));

    tracing::info!("gateway worker starting");
    while let Some(item) = shard.next_event(EventTypeFlags::all()).await {
        let event = match item {
            Ok(event) => event,
            Err(error) => {
                tracing::warn!(%error, "failed to receive gateway event");
                continue;
            }
        };

        match event {
            Event::GatewayClose(frame) => {
                tracing::error!(?frame, "Discord closed the gateway connection");
            }
            Event::GuildCreate(guild) => {
                if let GuildCreate::Available(guild) = *guild {
                    let guild_id = guild.id.to_string();
                    if let Err(error) = infrastructure
                        .upsert_connected_guild(
                            &guild_id,
                            &guild.name,
                            guild.member_count.unwrap_or(guild.members.len() as u64),
                            shard.id().number().into(),
                            &guild.owner_id.to_string(),
                        )
                        .await
                    {
                        tracing::error!(%guild_id, %error, "failed to update connected guild registry");
                    }
                }
            }
            Event::GuildDelete(guild) => {
                let guild_id = guild.id.to_string();
                if guild.unavailable != Some(true)
                    && let Err(error) = infrastructure.mark_guild_disconnected(&guild_id).await
                {
                    tracing::error!(%guild_id, %error, "failed to mark guild disconnected");
                }
            }
            Event::MemberAdd(member) => {
                let guild_id = member.guild_id.to_string();
                let username = member.user.name.clone();
                let display_name = member
                    .nick
                    .clone()
                    .or_else(|| member.user.global_name.clone())
                    .unwrap_or_else(|| username.clone());
                let config = match infrastructure.guild_config(&guild_id).await {
                    Ok(config) => config,
                    Err(error) => {
                        tracing::error!(%guild_id, %error, "could not load guild configuration for member join");
                        continue;
                    }
                };
                let event = MemberJoin {
                    guild_id: member.guild_id.get(),
                    guild_name: infrastructure
                        .connected_guild_name(&guild_id)
                        .await
                        .ok()
                        .flatten()
                        .unwrap_or_else(|| "the server".to_owned()),
                    user_id: member.user.id.get(),
                    username,
                    display_name,
                    account_age_seconds: account_age_seconds(member.user.id.get()),
                    has_avatar: member.user.avatar.is_some(),
                    is_bot: member.user.bot,
                };
                for module in modules
                    .iter()
                    .filter(|module| module.handles_member_join() && module.enabled(&config))
                {
                    let module = Arc::clone(module);
                    let context = context.clone();
                    let config = config.clone();
                    let event = event.clone();
                    let infrastructure = infrastructure.clone();
                    let guild_id = guild_id.clone();
                    tokio::spawn(async move {
                        let result = module.on_member_join(&context, &config, &event).await;
                        let mut audit = NewAuditEvent::guild(
                            &guild_id,
                            "system",
                            format!("module.{}.member_join", module.name()),
                        );
                        audit.actor_kind = "system".to_owned();
                        audit.source = "discord_gateway".to_owned();
                        audit.target_type = "member".to_owned();
                        audit.target_id = Some(event.user_id.to_string());
                        audit.metadata = serde_json::json!({ "module": module.name() });
                        if let Err(error) = result {
                            audit.outcome = "failure".to_owned();
                            audit.reason = Some(error.to_string());
                        }
                        if let Err(error) = infrastructure.record_audit_event(&audit).await {
                            tracing::error!(%error, "failed to persist member join module audit event");
                        }
                    });
                }
            }
            Event::MemberRemove(member) => {
                let guild_id = member.guild_id.to_string();
                let config = match infrastructure.guild_config(&guild_id).await {
                    Ok(config) => config,
                    Err(error) => {
                        tracing::error!(%guild_id, %error, "could not load guild configuration for member leave");
                        continue;
                    }
                };
                let username = member.user.name.clone();
                let display_name = member
                    .user
                    .global_name
                    .clone()
                    .unwrap_or_else(|| username.clone());
                let event = MemberLeave {
                    guild_id: member.guild_id.get(),
                    guild_name: infrastructure
                        .connected_guild_name(&guild_id)
                        .await
                        .ok()
                        .flatten()
                        .unwrap_or_else(|| "the server".to_owned()),
                    user_id: member.user.id.get(),
                    username,
                    display_name,
                };
                for module in modules
                    .iter()
                    .filter(|module| module.handles_member_leave() && module.enabled(&config))
                {
                    let result = module.on_member_leave(&context, &config, &event).await;
                    let mut audit = NewAuditEvent::guild(
                        &guild_id,
                        "system",
                        format!("module.{}.member_leave", module.name()),
                    );
                    audit.actor_kind = "system".to_owned();
                    audit.source = "discord_gateway".to_owned();
                    audit.target_type = "member".to_owned();
                    audit.target_id = Some(event.user_id.to_string());
                    audit.metadata = serde_json::json!({ "module": module.name() });
                    if let Err(error) = result {
                        audit.outcome = "failure".to_owned();
                        audit.reason = Some(error.to_string());
                    }
                    if let Err(error) = infrastructure.record_audit_event(&audit).await {
                        tracing::error!(%error, "failed to persist member leave module audit event");
                    }
                }
            }
            Event::MessageCreate(message) => {
                let Some(guild_id) = message.guild_id else {
                    continue;
                };
                let guild_id_string = guild_id.to_string();
                let config = match infrastructure.guild_config(&guild_id_string).await {
                    Ok(config) => config,
                    Err(error) => {
                        tracing::error!(%guild_id, %error, "could not load guild configuration for message");
                        continue;
                    }
                };
                let normalized = MessageCreate {
                    guild_id: guild_id.get(),
                    channel_id: message.channel_id.get(),
                    message_id: message.id.get(),
                    author_id: message.author.id.get(),
                    author_is_bot: message.author.bot,
                    role_ids: message
                        .member
                        .as_ref()
                        .map(|member| member.roles.iter().map(|id| id.get()).collect())
                        .unwrap_or_default(),
                    mention_count: message.mentions.len() + message.mention_roles.len(),
                    content: message.content.clone(),
                };
                for module in modules
                    .iter()
                    .filter(|module| module.handles_message_create() && module.enabled(&config))
                {
                    let result = module
                        .on_message_create(&context, &config, &normalized)
                        .await;
                    let outcome = match result {
                        Ok(Some(outcome)) => outcome,
                        Ok(None) => continue,
                        Err(error) => {
                            let mut audit = NewAuditEvent::guild(
                                &guild_id_string,
                                "system",
                                format!("module.{}.message.failed", module.name()),
                            );
                            audit.actor_kind = "system".to_owned();
                            audit.source = "discord_gateway".to_owned();
                            audit.target_type = "message".to_owned();
                            audit.target_id = Some(normalized.message_id.to_string());
                            audit.outcome = "failure".to_owned();
                            audit.reason = Some(error.to_string());
                            audit.metadata = serde_json::json!({ "module": module.name(), "channel_id": normalized.channel_id.to_string() });
                            if let Err(store_error) =
                                infrastructure.record_audit_event(&audit).await
                            {
                                tracing::error!(%store_error, "failed to persist message module failure");
                            }
                            continue;
                        }
                    };
                    let mut audit = NewAuditEvent::guild(
                        &guild_id_string,
                        normalized.author_id.to_string(),
                        outcome.action,
                    );
                    audit.source = "discord_gateway".to_owned();
                    audit.target_type = outcome.target_type;
                    audit.target_id = outcome.target_id;
                    audit.metadata = serde_json::json!({ "module": module.name(), "channel_id": normalized.channel_id.to_string() });
                    if let Err(error) = infrastructure.record_audit_event(&audit).await {
                        tracing::error!(%error, "failed to persist message module audit event");
                    }
                }
            }
            Event::ReactionAdd(reaction) => {
                let Some(guild_id) = reaction.guild_id else {
                    continue;
                };
                let guild_id_string = guild_id.to_string();
                let config = match infrastructure.guild_config(&guild_id_string).await {
                    Ok(config) => config,
                    Err(error) => {
                        tracing::error!(%guild_id, %error, "could not load guild configuration for reaction");
                        continue;
                    }
                };
                let event = ReactionAdd {
                    guild_id: guild_id.get(),
                    channel_id: reaction.channel_id.get(),
                    message_id: reaction.message_id.get(),
                    reactor_id: reaction.user_id.get(),
                    message_author_id: reaction.message_author_id.map(|id| id.get()),
                    emoji: match &reaction.emoji {
                        EmojiReactionType::Unicode { name } => name.clone(),
                        EmojiReactionType::Custom { id, .. } => id.to_string(),
                    },
                };
                for module in modules
                    .iter()
                    .filter(|module| module.handles_reaction_add() && module.enabled(&config))
                {
                    match module.on_reaction_add(&context, &config, &event).await {
                        Ok(Some(outcome)) => {
                            let mut audit = NewAuditEvent::guild(
                                &guild_id_string,
                                event.reactor_id.to_string(),
                                outcome.action,
                            );
                            audit.source = "discord_gateway".to_owned();
                            audit.target_type = outcome.target_type;
                            audit.target_id = outcome.target_id;
                            audit.metadata = serde_json::json!({ "module": module.name(), "channel_id": event.channel_id.to_string() });
                            if let Err(error) = infrastructure.record_audit_event(&audit).await {
                                tracing::error!(%error, "failed to persist reaction module audit event");
                            }
                        }
                        Ok(None) => {}
                        Err(error) => {
                            tracing::error!(module = module.name(), %error, "reaction module failed");
                            let mut audit = NewAuditEvent::guild(
                                &guild_id_string,
                                event.reactor_id.to_string(),
                                format!("module.{}.reaction.failed", module.name()),
                            );
                            audit.source = "discord_gateway".to_owned();
                            audit.target_type = "message".to_owned();
                            audit.target_id = Some(event.message_id.to_string());
                            audit.outcome = "failure".to_owned();
                            audit.reason = Some(error.to_string());
                            if let Err(store_error) =
                                infrastructure.record_audit_event(&audit).await
                            {
                                tracing::error!(%store_error, "failed to persist reaction module failure");
                            }
                        }
                    }
                }
            }
            Event::ReactionRemove(reaction) => {
                let Some(guild_id) = reaction.guild_id else {
                    continue;
                };
                let guild_id_string = guild_id.to_string();
                let config = match infrastructure.guild_config(&guild_id_string).await {
                    Ok(config) => config,
                    Err(error) => {
                        tracing::error!(%guild_id, %error, "could not load guild configuration for removed reaction");
                        continue;
                    }
                };
                let event = ReactionAdd {
                    guild_id: guild_id.get(),
                    channel_id: reaction.channel_id.get(),
                    message_id: reaction.message_id.get(),
                    reactor_id: reaction.user_id.get(),
                    message_author_id: None,
                    emoji: match &reaction.emoji {
                        EmojiReactionType::Unicode { name } => name.clone(),
                        EmojiReactionType::Custom { id, .. } => id.to_string(),
                    },
                };
                for module in modules
                    .iter()
                    .filter(|module| module.handles_reaction_remove() && module.enabled(&config))
                {
                    match module.on_reaction_remove(&context, &config, &event).await {
                        Ok(Some(outcome)) => {
                            let mut audit = NewAuditEvent::guild(
                                &guild_id_string,
                                event.reactor_id.to_string(),
                                outcome.action,
                            );
                            audit.source = "discord_gateway".to_owned();
                            audit.target_type = outcome.target_type;
                            audit.target_id = outcome.target_id;
                            audit.metadata = serde_json::json!({ "module": module.name(), "channel_id": event.channel_id.to_string() });
                            if let Err(error) = infrastructure.record_audit_event(&audit).await {
                                tracing::error!(%error, "failed to persist removed-reaction module audit event");
                            }
                        }
                        Ok(None) => {}
                        Err(error) => {
                            tracing::error!(module = module.name(), %error, "removed-reaction module failed");
                            let mut audit = NewAuditEvent::guild(
                                &guild_id_string,
                                event.reactor_id.to_string(),
                                format!("module.{}.reaction_remove.failed", module.name()),
                            );
                            audit.source = "discord_gateway".to_owned();
                            audit.target_type = "message".to_owned();
                            audit.target_id = Some(event.message_id.to_string());
                            audit.outcome = "failure".to_owned();
                            audit.reason = Some(error.to_string());
                            if let Err(store_error) =
                                infrastructure.record_audit_event(&audit).await
                            {
                                tracing::error!(%store_error, "failed to persist removed-reaction module failure");
                            }
                        }
                    }
                }
            }
            Event::InteractionCreate(interaction) => {
                if let Some(InteractionData::ModalSubmit(modal)) = interaction.data.as_ref() {
                    let Some(guild_id) = interaction.guild_id else {
                        continue;
                    };
                    let config = match infrastructure.guild_config(&guild_id.to_string()).await {
                        Ok(config) => config,
                        Err(error) => {
                            tracing::error!(%guild_id, %error, "could not load guild configuration for modal");
                            continue;
                        }
                    };
                    let actor_id = interaction
                        .author_id()
                        .map(|id| id.get())
                        .unwrap_or_default();
                    let modal_interaction = ModalSubmitInteraction {
                        id: interaction.id.get(),
                        token: interaction.token.clone(),
                        guild_id: guild_id.get(),
                        actor_id,
                        actor_role_ids: interaction
                            .member
                            .as_ref()
                            .map(|member| member.roles.iter().map(|role| role.get()).collect())
                            .unwrap_or_default(),
                        custom_id: modal.custom_id.clone(),
                        values: modal_values(&modal.components),
                    };
                    for module in modules
                        .iter()
                        .filter(|module| module.handles_modal_submit(&modal.custom_id))
                    {
                        let module_enabled = module.enabled(&config);
                        let result = if module_enabled {
                            module
                                .on_modal_submit(&context, &config, &modal_interaction)
                                .await
                        } else {
                            context
                                .discord
                                .respond_to_modal(
                                    &modal_interaction,
                                    CommandResponse::ephemeral(format!(
                                        "The {} module is disabled in this server.",
                                        module.name()
                                    )),
                                )
                                .await
                                .map_err(anyhow::Error::from)
                                .map(|()| None)
                        };
                        if let Err(error) = &result {
                            tracing::error!(module = module.name(), %error, "modal submission failed");
                            if let Err(response_error) = context.discord.respond_to_modal(&modal_interaction, CommandResponse::ephemeral("That form could not be submitted. Please try again or contact a server administrator.")).await {
                                tracing::error!(%response_error, "failed to acknowledge failed modal submission");
                            }
                        }
                        let outcome = result.as_ref().ok().and_then(|outcome| outcome.as_ref());
                        let mut audit = NewAuditEvent::guild(
                            guild_id.to_string(),
                            actor_id.to_string(),
                            outcome.map_or_else(
                                || format!("module.{}.modal_submit", module.name()),
                                |value| value.action.clone(),
                            ),
                        );
                        audit.source = "discord_gateway".into();
                        audit.target_type = outcome
                            .map_or_else(|| "form".into(), |value| value.target_type.clone());
                        audit.target_id = outcome
                            .and_then(|value| value.target_id.clone())
                            .or_else(|| {
                                modal
                                    .custom_id
                                    .strip_prefix("orivane:form:")
                                    .map(str::to_owned)
                            });
                        audit.metadata = serde_json::json!({ "module": module.name(), "interaction_id": modal_interaction.id.to_string() });
                        if !module_enabled {
                            audit.outcome = "denied".into();
                            audit.reason = Some("module is disabled for this server".into());
                        }
                        if let Err(error) = &result {
                            audit.outcome = "failure".into();
                            audit.reason = Some(error.to_string());
                        }
                        if let Err(error) = infrastructure.record_audit_event(&audit).await {
                            tracing::error!(%error, "failed to persist modal audit event");
                        }
                        let notice = AuditNotice {
                            guild_id: guild_id.get(),
                            action: audit.action.clone(),
                            actor_id: audit.actor_id.clone(),
                            outcome: audit.outcome.clone(),
                        };
                        for logging_module in modules.iter().filter(|candidate| {
                            candidate.handles_audit_notice() && candidate.enabled(&config)
                        }) {
                            if let Err(error) = logging_module
                                .on_audit_notice(&context, &config, &notice)
                                .await
                            {
                                tracing::error!(module = logging_module.name(), %error, "failed to deliver modal server log notice");
                            }
                        }
                    }
                    continue;
                }
                let Some(InteractionData::ApplicationCommand(command)) = interaction.data.as_ref()
                else {
                    continue;
                };
                let Some(guild_id) = interaction.guild_id else {
                    continue;
                };
                let config = match infrastructure.guild_config(&guild_id.to_string()).await {
                    Ok(config) => config,
                    Err(error) => {
                        tracing::error!(%guild_id, %error, "could not load guild configuration");
                        continue;
                    }
                };

                for module in &modules {
                    if !module
                        .commands()
                        .iter()
                        .any(|definition| definition.name == command.name)
                    {
                        continue;
                    }
                    let actor_id = interaction
                        .author_id()
                        .map(|id| id.get())
                        .unwrap_or_default();
                    let command = CommandInteraction {
                        id: interaction.id.get(),
                        token: interaction.token.clone(),
                        guild_id: guild_id.get(),
                        channel_id: interaction.channel.as_ref().map(|channel| channel.id.get()),
                        actor_id,
                        actor_role_ids: interaction
                            .member
                            .as_ref()
                            .map(|member| member.roles.iter().map(|role| role.get()).collect())
                            .unwrap_or_default(),
                        command_name: command.name.clone(),
                        options: command_options(&command.options),
                    };
                    let started_at = Instant::now();
                    let module_enabled = module.enabled(&config);
                    let result = if module_enabled {
                        module.on_command(&context, &config, &command).await
                    } else {
                        context
                            .discord
                            .respond_to_command(
                                &command,
                                bot_discord::CommandResponse::ephemeral(format!(
                                    "The {} module is disabled in this server.",
                                    module.name()
                                )),
                            )
                            .await
                            .map_err(anyhow::Error::from)
                    };
                    let latency_ms =
                        started_at.elapsed().as_millis().min(u128::from(u64::MAX)) as u64;
                    let mut audit = NewAuditEvent::guild(
                        guild_id.to_string(),
                        actor_id.to_string(),
                        format!("slash_command.{}", command.command_name),
                    );
                    audit.source = "discord_gateway".to_owned();
                    audit.target_type = "slash_command".to_owned();
                    audit.target_id = Some(command.command_name.clone());
                    audit.metadata = serde_json::json!({ "module": module.name(), "interaction_id": command.id.to_string(), "latency_ms": latency_ms });
                    if !module_enabled {
                        audit.outcome = "denied".to_owned();
                        audit.reason = Some("module is disabled for this server".to_owned());
                    }
                    if let Err(error) = &result {
                        audit.outcome = "failure".to_owned();
                        audit.reason = Some(error.to_string());
                        tracing::error!(module = module.name(), %error, "module failed");
                    }
                    if let Err(error) = infrastructure.record_audit_event(&audit).await {
                        tracing::error!(%error, "failed to persist slash command audit event");
                    }
                    let notice = AuditNotice {
                        guild_id: guild_id.get(),
                        action: audit.action.clone(),
                        actor_id: audit.actor_id.clone(),
                        outcome: audit.outcome.clone(),
                    };
                    for logging_module in modules.iter().filter(|candidate| {
                        candidate.handles_audit_notice() && candidate.enabled(&config)
                    }) {
                        if let Err(error) = logging_module
                            .on_audit_notice(&context, &config, &notice)
                            .await
                        {
                            tracing::error!(module = logging_module.name(), %error, "failed to deliver server log notice");
                        }
                    }
                }
            }
            _ => {}
        }
    }

    anyhow::bail!("Discord gateway event stream closed")
}

fn modal_values(components: &[ModalInteractionComponent]) -> BTreeMap<String, String> {
    let mut values = BTreeMap::new();
    for component in components {
        match component {
            ModalInteractionComponent::TextInput(input) => {
                values.insert(input.custom_id.clone(), input.value.clone());
            }
            ModalInteractionComponent::ActionRow(row) => {
                values.extend(modal_values(&row.components))
            }
            ModalInteractionComponent::Label(label) => {
                values.extend(modal_values(std::slice::from_ref(&label.component)))
            }
            _ => {}
        }
    }
    values
}

async fn run_scheduler(
    context: ModuleContext,
    modules: Vec<Arc<dyn BotModule>>,
    infrastructure: Infrastructure,
) {
    let mut interval = tokio::time::interval(std::time::Duration::from_secs(15));
    loop {
        interval.tick().await;
        let now_unix = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        let configs = match infrastructure.scheduled_guild_configs().await {
            Ok(configs) => configs,
            Err(error) => {
                tracing::error!(%error, "failed to load scheduled-message guilds");
                continue;
            }
        };
        for config in configs {
            for module in modules
                .iter()
                .filter(|module| module.handles_scheduled_tick() && module.enabled(&config))
            {
                match module
                    .on_scheduled_tick(&context, &config, &ScheduledTick { now_unix })
                    .await
                {
                    Ok(outcomes) => {
                        for outcome in outcomes {
                            let mut audit =
                                NewAuditEvent::guild(&config.guild_id, "system", outcome.action);
                            audit.actor_kind = "system".into();
                            audit.source = "scheduler".into();
                            audit.target_type = outcome.target_type;
                            audit.target_id = outcome.target_id;
                            audit.metadata = serde_json::json!({ "module": module.name() });
                            if let Err(error) = infrastructure.record_audit_event(&audit).await {
                                tracing::error!(%error, "failed to audit scheduled message");
                            }
                        }
                    }
                    Err(error) => {
                        tracing::error!(module = module.name(), guild_id = %config.guild_id, %error, "scheduled module failed");
                        let mut audit = NewAuditEvent::guild(
                            &config.guild_id,
                            "system",
                            format!("module.{}.schedule.failed", module.name()),
                        );
                        audit.actor_kind = "system".into();
                        audit.source = "scheduler".into();
                        audit.outcome = "failure".into();
                        audit.reason = Some(error.to_string());
                        if let Err(store_error) = infrastructure.record_audit_event(&audit).await {
                            tracing::error!(%store_error, "failed to audit scheduler failure");
                        }
                    }
                }
            }
        }
    }
}

async fn register_commands(discord: &dyn DiscordApi, modules: &[Arc<dyn BotModule>]) -> Result<()> {
    let commands: Vec<CommandDefinition> = modules
        .iter()
        .flat_map(|module| module.commands().iter().cloned())
        .collect();
    discord.register_commands(&commands).await?;
    tracing::info!(
        count = commands.len(),
        "registered global application commands"
    );
    Ok(())
}

fn command_options(options: &[CommandDataOption]) -> BTreeMap<String, CommandValue> {
    options
        .iter()
        .filter_map(|option| {
            let value = match &option.value {
                TwilightCommandValue::String(value) => CommandValue::String(value.clone()),
                TwilightCommandValue::Integer(value) => CommandValue::Integer(*value),
                TwilightCommandValue::Boolean(value) => CommandValue::Boolean(*value),
                TwilightCommandValue::User(value) => CommandValue::User(value.get()),
                TwilightCommandValue::Channel(value) => CommandValue::Channel(value.get()),
                TwilightCommandValue::Role(value) => CommandValue::Role(value.get()),
                _ => return None,
            };
            Some((option.name.clone(), value))
        })
        .collect()
}

fn required_env(name: &str) -> Result<String> {
    env::var(name).with_context(|| format!("{name} is required"))
}

fn account_age_seconds(user_id: u64) -> u64 {
    const DISCORD_EPOCH_MS: u64 = 1_420_070_400_000;
    let created_ms = (user_id >> 22).saturating_add(DISCORD_EPOCH_MS);
    let now_ms = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64;
    now_ms.saturating_sub(created_ms) / 1_000
}

fn init_tracing() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info,sqlx=warn,twilight_gateway=warn".into()),
        )
        .init();
}

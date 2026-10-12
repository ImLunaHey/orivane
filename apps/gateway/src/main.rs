use std::{
    collections::BTreeMap,
    env,
    sync::Arc,
    time::{Instant, SystemTime, UNIX_EPOCH},
};

use anyhow::{Context, Result};
use async_trait::async_trait;
use bot_core::{
    Giveaway, Infrastructure, NewAuditEvent, NewFormSubmission, NewGiveaway, NewModerationCase,
    NewSuggestion, Suggestion, level_for_xp,
};
use bot_discord::{
    CommandDefinition, CommandInteraction, CommandResponse, CommandValue, ComponentInteraction,
    DiscordApi, ModalSubmitInteraction, TwilightDiscord,
};
use bot_module_anti_raid::AntiRaidModule;
use bot_module_api::{
    AuditNotice, BotModule, FormSubmissionInput, GiveawayInput, GiveawayResult, MemberJoin,
    MemberLeave, MessageCreate, ModerationCaseInput, ModuleContext, ModuleServices, ReactionAdd,
    ScheduledTick, SuggestionInput, SuggestionResult, TicketRecordInput, XpAwardInput,
    XpAwardResult, XpProfile,
};
use bot_module_automod::AutomodModule;
use bot_module_autoresponder::AutoresponderModule;
use bot_module_autorole::AutoroleModule;
use bot_module_custom_commands::CustomCommandsModule;
use bot_module_forms::FormsModule;
use bot_module_giveaways::GiveawaysModule;
use bot_module_leveling::LevelingModule;
use bot_module_logging::LoggingModule;
use bot_module_moderation::ModerationModule;
use bot_module_reaction_roles::ReactionRolesModule;
use bot_module_scheduled_messages::ScheduledMessagesModule;
use bot_module_starboard::StarboardModule;
use bot_module_suggestions::SuggestionsModule;
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
    async fn create_giveaway(&self, input: GiveawayInput) -> Result<GiveawayResult> {
        let ends_at = chrono::DateTime::from_timestamp(
            i64::try_from(input.ends_at_unix).unwrap_or(i64::MAX),
            0,
        )
        .context("giveaway end time is invalid")?;
        let giveaway = self
            .infrastructure
            .create_giveaway(&NewGiveaway {
                guild_id: input.guild_id.to_string(),
                channel_id: input.channel_id.to_string(),
                creator_id: input.creator_id.to_string(),
                title: input.title,
                description: input.description,
                winner_count: i16::from(input.winner_count),
                minimum_account_age_days: i32::from(input.minimum_account_age_days),
                required_role_ids: serde_json::json!(
                    input
                        .required_role_ids
                        .iter()
                        .map(u64::to_string)
                        .collect::<Vec<_>>()
                ),
                bonus_role_id: input.bonus_role_id.map(|id| id.to_string()),
                bonus_entries: i16::from(input.bonus_entries),
                ends_at,
            })
            .await?;
        Ok(giveaway_result(&giveaway))
    }

    async fn activate_giveaway(
        &self,
        guild_id: u64,
        giveaway_id: i64,
        message_id: u64,
    ) -> Result<bool> {
        Ok(self
            .infrastructure
            .activate_giveaway(&guild_id.to_string(), giveaway_id, &message_id.to_string())
            .await?)
    }

    async fn fail_giveaway_delivery(&self, guild_id: u64, giveaway_id: i64) -> Result<()> {
        self.infrastructure
            .fail_giveaway_delivery(&guild_id.to_string(), giveaway_id)
            .await?;
        Ok(())
    }

    async fn giveaway(&self, guild_id: u64, giveaway_id: i64) -> Result<Option<GiveawayResult>> {
        Ok(self
            .infrastructure
            .giveaway(&guild_id.to_string(), giveaway_id)
            .await?
            .as_ref()
            .map(giveaway_result))
    }

    async fn enter_giveaway(
        &self,
        guild_id: u64,
        giveaway_id: i64,
        user_id: u64,
        weight: u8,
    ) -> Result<Option<bool>> {
        Ok(self
            .infrastructure
            .enter_giveaway(
                &guild_id.to_string(),
                giveaway_id,
                &user_id.to_string(),
                i16::from(weight),
            )
            .await?)
    }

    async fn due_giveaway_ids(&self, guild_id: u64, now_unix: u64) -> Result<Vec<i64>> {
        Ok(self
            .infrastructure
            .due_giveaway_ids(&guild_id.to_string(), now_unix)
            .await?)
    }

    async fn pending_giveaway_announcement_ids(&self, guild_id: u64) -> Result<Vec<i64>> {
        Ok(self
            .infrastructure
            .pending_giveaway_announcement_ids(&guild_id.to_string())
            .await?)
    }

    async fn mark_giveaway_announced(&self, guild_id: u64, giveaway_id: i64) -> Result<()> {
        self.infrastructure
            .mark_giveaway_announced(&guild_id.to_string(), giveaway_id)
            .await?;
        Ok(())
    }

    async fn end_giveaway(
        &self,
        guild_id: u64,
        giveaway_id: i64,
        force: bool,
    ) -> Result<Option<GiveawayResult>> {
        Ok(self
            .infrastructure
            .end_giveaway(&guild_id.to_string(), giveaway_id, force)
            .await?
            .as_ref()
            .map(giveaway_result))
    }

    async fn reroll_giveaway(
        &self,
        guild_id: u64,
        giveaway_id: i64,
    ) -> Result<Option<GiveawayResult>> {
        Ok(self
            .infrastructure
            .reroll_giveaway(&guild_id.to_string(), giveaway_id)
            .await?
            .as_ref()
            .map(giveaway_result))
    }

    async fn claim_suggestion_cooldown(
        &self,
        guild_id: u64,
        user_id: u64,
        cooldown_seconds: u32,
    ) -> Result<bool> {
        Ok(self
            .infrastructure
            .claim_suggestion_cooldown(
                &guild_id.to_string(),
                &user_id.to_string(),
                cooldown_seconds,
            )
            .await?)
    }

    async fn create_suggestion(&self, input: SuggestionInput) -> Result<SuggestionResult> {
        let suggestion = self
            .infrastructure
            .create_suggestion(&NewSuggestion {
                guild_id: input.guild_id.to_string(),
                channel_id: input.channel_id.to_string(),
                author_id: input.author_id.map(|id| id.to_string()),
                content: input.content,
            })
            .await?;
        Ok(suggestion_result(&suggestion))
    }

    async fn set_suggestion_message_id(
        &self,
        guild_id: u64,
        suggestion_id: i64,
        message_id: u64,
    ) -> Result<()> {
        self.infrastructure
            .set_suggestion_message_id(
                &guild_id.to_string(),
                suggestion_id,
                &message_id.to_string(),
            )
            .await?;
        Ok(())
    }

    async fn vote_suggestion(
        &self,
        guild_id: u64,
        suggestion_id: i64,
        user_id: u64,
        direction: i8,
    ) -> Result<Option<SuggestionResult>> {
        Ok(self
            .infrastructure
            .vote_suggestion(
                &guild_id.to_string(),
                suggestion_id,
                &user_id.to_string(),
                i16::from(direction),
            )
            .await?
            .as_ref()
            .map(suggestion_result))
    }

    async fn update_suggestion_status(
        &self,
        guild_id: u64,
        suggestion_id: i64,
        actor_id: u64,
        status: &str,
        staff_note: Option<&str>,
    ) -> Result<Option<SuggestionResult>> {
        let mut audit = NewAuditEvent::guild(
            guild_id.to_string(),
            actor_id.to_string(),
            "suggestion.status.updated",
        );
        audit.source = "discord_gateway".into();
        audit.target_type = "suggestion".into();
        audit.target_id = Some(suggestion_id.to_string());
        audit.metadata = serde_json::json!({ "status": status });
        Ok(self
            .infrastructure
            .update_suggestion_status_audited(
                &guild_id.to_string(),
                suggestion_id,
                status,
                staff_note,
                &audit,
            )
            .await?
            .as_ref()
            .map(suggestion_result))
    }

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

fn suggestion_result(suggestion: &Suggestion) -> SuggestionResult {
    SuggestionResult {
        id: suggestion.id,
        status: suggestion.status.clone(),
        upvotes: u64::try_from(suggestion.upvotes).unwrap_or_default(),
        downvotes: u64::try_from(suggestion.downvotes).unwrap_or_default(),
    }
}

fn giveaway_result(giveaway: &Giveaway) -> GiveawayResult {
    GiveawayResult {
        id: giveaway.id,
        channel_id: giveaway.channel_id.parse().unwrap_or_default(),
        title: giveaway.title.clone(),
        status: giveaway.status.clone(),
        winner_count: u8::try_from(giveaway.winner_count).unwrap_or_default(),
        minimum_account_age_days: u16::try_from(giveaway.minimum_account_age_days)
            .unwrap_or_default(),
        required_role_ids: serde_json::from_value::<Vec<String>>(
            giveaway.required_role_ids.clone(),
        )
        .unwrap_or_default()
        .into_iter()
        .filter_map(|id| id.parse().ok())
        .collect(),
        bonus_role_id: giveaway
            .bonus_role_id
            .as_deref()
            .and_then(|id| id.parse().ok()),
        bonus_entries: u8::try_from(giveaway.bonus_entries).unwrap_or_default(),
        winner_ids: serde_json::from_value::<Vec<String>>(giveaway.winner_ids.clone())
            .unwrap_or_default()
            .into_iter()
            .filter_map(|id| id.parse().ok())
            .collect(),
        entrant_count: u64::try_from(giveaway.entrant_count).unwrap_or_default(),
        total_entries: u64::try_from(giveaway.total_entries).unwrap_or_default(),
        ends_at_unix: u64::try_from(giveaway.ends_at.timestamp()).unwrap_or_default(),
        reroll_count: u32::try_from(giveaway.reroll_count).unwrap_or_default(),
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
        Arc::new(AntiRaidModule),
        Arc::new(ScheduledMessagesModule),
        Arc::new(FormsModule),
        Arc::new(SuggestionsModule),
        Arc::new(GiveawaysModule),
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
                if let Some(author) = interaction.author() {
                    let display_name = interaction
                        .member
                        .as_ref()
                        .and_then(|member| member.nick.as_deref())
                        .or(author.global_name.as_deref());
                    let avatar_url = author.avatar.as_ref().map(|hash| {
                        let hash = hash.to_string();
                        let extension = if hash.starts_with("a_") { "gif" } else { "png" };
                        format!(
                            "https://cdn.discordapp.com/avatars/{}/{}.{}?size=128",
                            author.id, hash, extension
                        )
                    });
                    if let Err(error) = infrastructure
                        .upsert_discord_identity(
                            &author.id.to_string(),
                            &author.name,
                            display_name,
                            avatar_url.as_deref(),
                        )
                        .await
                    {
                        tracing::warn!(user_id = %author.id, %error, "could not refresh interaction identity");
                    }
                }
                if let Some(InteractionData::MessageComponent(component)) =
                    interaction.data.as_ref()
                {
                    let Some(guild_id) = interaction.guild_id else {
                        continue;
                    };
                    let config = match infrastructure.guild_config(&guild_id.to_string()).await {
                        Ok(config) => config,
                        Err(error) => {
                            tracing::error!(%guild_id, %error, "could not load guild configuration for component");
                            continue;
                        }
                    };
                    let actor_id = interaction
                        .author_id()
                        .map(|id| id.get())
                        .unwrap_or_default();
                    let component_interaction = ComponentInteraction {
                        id: interaction.id.get(),
                        token: interaction.token.clone(),
                        guild_id: guild_id.get(),
                        actor_id,
                        actor_role_ids: interaction
                            .member
                            .as_ref()
                            .map(|member| member.roles.iter().map(|role| role.get()).collect())
                            .unwrap_or_default(),
                        custom_id: component.custom_id.clone(),
                    };
                    for module in modules
                        .iter()
                        .filter(|module| module.handles_component(&component.custom_id))
                    {
                        let module_enabled = module.enabled(&config);
                        let result = if module_enabled {
                            module
                                .on_component(&context, &config, &component_interaction)
                                .await
                        } else {
                            context
                                .discord
                                .respond_to_component(
                                    &component_interaction,
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
                            tracing::error!(module = module.name(), %error, "component interaction failed");
                            if let Err(response_error) = context
                                .discord
                                .respond_to_component(
                                    &component_interaction,
                                    CommandResponse::ephemeral(
                                        "That action could not be completed. Please try again.",
                                    ),
                                )
                                .await
                            {
                                tracing::error!(%response_error, "failed to acknowledge component failure");
                            }
                        }
                        let outcome = result.as_ref().ok().and_then(|value| value.as_ref());
                        let mut audit = NewAuditEvent::guild(
                            guild_id.to_string(),
                            actor_id.to_string(),
                            outcome.map_or_else(
                                || format!("module.{}.component", module.name()),
                                |value| value.action.clone(),
                            ),
                        );
                        audit.source = "discord_gateway".into();
                        audit.target_type = outcome
                            .map_or_else(|| "component".into(), |value| value.target_type.clone());
                        audit.target_id = outcome.and_then(|value| value.target_id.clone());
                        audit.metadata = serde_json::json!({ "module": module.name(), "interaction_id": component_interaction.id.to_string() });
                        if !module_enabled {
                            audit.outcome = "denied".into();
                        }
                        if let Err(error) = &result {
                            audit.outcome = "failure".into();
                            audit.reason = Some(error.to_string());
                        }
                        if let Err(error) = infrastructure.record_audit_event(&audit).await {
                            tracing::error!(%error, "failed to persist component audit event");
                        }
                    }
                    continue;
                }
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
                        actor_permissions: interaction
                            .member
                            .as_ref()
                            .and_then(|member| member.permissions)
                            .map(|permissions| permissions.bits())
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

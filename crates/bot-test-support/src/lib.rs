use std::{
    collections::{HashMap, HashSet, VecDeque},
    sync::{Arc, Mutex},
};

use async_trait::async_trait;
use bot_discord::{
    CommandDefinition, CommandInteraction, CommandResponse, ComponentInteraction, DiscordApi,
    DiscordError, GuildChannel, GuildRole, MessageButton, MessageReactionSummary, ModalDefinition,
    ModalSubmitInteraction, RichEmbed,
};
use bot_module_api::{
    FormSubmissionInput, GiveawayInput, GiveawayResult, ManagedEmbedResult,
    MessageDeletionJobResult, ModerationCaseInput, ModuleServices, ReminderInput, ReminderResult,
    SuggestionInput, SuggestionResult, TicketRecordInput, XpAwardInput, XpAwardResult, XpProfile,
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DiscordCall {
    RegisterCommands {
        commands: Vec<CommandDefinition>,
    },
    RespondToCommand {
        interaction_id: u64,
        response: CommandResponse,
    },
    OpenModal {
        interaction_id: u64,
        modal: ModalDefinition,
    },
    RespondToModal {
        interaction_id: u64,
        response: CommandResponse,
    },
    RespondToComponent {
        interaction_id: u64,
        response: CommandResponse,
    },
    SendMessage {
        channel_id: u64,
        content: String,
    },
    SendMessageWithButtons {
        channel_id: u64,
        content: String,
        buttons: Vec<MessageButton>,
    },
    SendRichEmbed {
        channel_id: u64,
        embed: RichEmbed,
    },
    EditRichEmbed {
        channel_id: u64,
        message_id: u64,
        embed: RichEmbed,
    },
    SendDirectMessage {
        user_id: u64,
        content: String,
    },
    AddMemberRole {
        guild_id: u64,
        user_id: u64,
        role_id: u64,
    },
    RemoveMemberRole {
        guild_id: u64,
        user_id: u64,
        role_id: u64,
        reason: String,
    },
    TimeoutMember {
        guild_id: u64,
        user_id: u64,
        duration_seconds: u64,
        reason: String,
    },
    KickMember {
        guild_id: u64,
        user_id: u64,
        reason: String,
    },
    BanMember {
        guild_id: u64,
        user_id: u64,
        delete_message_seconds: u32,
        reason: String,
    },
    UnbanMember {
        guild_id: u64,
        user_id: u64,
        reason: String,
    },
    DeleteMessage {
        channel_id: u64,
        message_id: u64,
        reason: String,
    },
    GuildChannels {
        guild_id: u64,
    },
    GuildRoles {
        guild_id: u64,
    },
    CreateTicketChannel {
        guild_id: u64,
        category_id: u64,
        opener_id: u64,
        support_role_ids: Vec<u64>,
        name: String,
        reason: String,
    },
    DeleteChannel {
        channel_id: u64,
        reason: String,
    },
    MessageReactionSummary {
        channel_id: u64,
        message_id: u64,
        emoji: String,
    },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ServiceCall {
    CreateModerationCase(ModerationCaseInput),
    AwardXp(XpAwardInput),
    XpProfile {
        guild_id: u64,
        user_id: u64,
    },
    XpLeaderboard {
        guild_id: u64,
        limit: u8,
    },
    FindOpenTicket {
        guild_id: u64,
        opener_id: u64,
    },
    RecordOpenTicket(TicketRecordInput),
    CloseTicket {
        guild_id: u64,
        channel_id: u64,
        closed_by: u64,
    },
    ClaimStarboardPost {
        guild_id: u64,
        source_message_id: u64,
        starboard_channel_id: u64,
    },
    RecordRaidJoin {
        guild_id: u64,
        user_id: u64,
        window_seconds: u16,
    },
    SetRaidLockdown {
        guild_id: u64,
        duration_seconds: u64,
    },
    ClearRaidLockdown {
        guild_id: u64,
    },
    RaidLockdownActive {
        guild_id: u64,
    },
    ClaimScheduledMessage {
        guild_id: u64,
        schedule_id: String,
        occurrence_unix: u64,
    },
    ReleaseScheduledMessage {
        guild_id: u64,
        schedule_id: String,
        occurrence_unix: u64,
    },
    CreateFormSubmission(FormSubmissionInput),
    CreateSuggestion(SuggestionInput),
    ClaimSuggestionCooldown {
        guild_id: u64,
        user_id: u64,
        cooldown_seconds: u32,
    },
    SetSuggestionMessageId {
        guild_id: u64,
        suggestion_id: i64,
        message_id: u64,
    },
    VoteSuggestion {
        guild_id: u64,
        suggestion_id: i64,
        user_id: u64,
        direction: i8,
    },
    UpdateSuggestionStatus {
        guild_id: u64,
        suggestion_id: i64,
        actor_id: u64,
        status: String,
        staff_note: Option<String>,
    },
    CreateGiveaway(GiveawayInput),
    ActivateGiveaway {
        guild_id: u64,
        giveaway_id: i64,
        message_id: u64,
    },
    FailGiveawayDelivery {
        guild_id: u64,
        giveaway_id: i64,
    },
    Giveaway {
        guild_id: u64,
        giveaway_id: i64,
    },
    EnterGiveaway {
        guild_id: u64,
        giveaway_id: i64,
        user_id: u64,
        weight: u8,
    },
    DueGiveawayIds {
        guild_id: u64,
        now_unix: u64,
    },
    PendingGiveawayAnnouncementIds {
        guild_id: u64,
    },
    MarkGiveawayAnnounced {
        guild_id: u64,
        giveaway_id: i64,
    },
    EndGiveaway {
        guild_id: u64,
        giveaway_id: i64,
        force: bool,
    },
    RerollGiveaway {
        guild_id: u64,
        giveaway_id: i64,
    },
    CreateReminder(ReminderInput),
    UserReminders {
        guild_id: u64,
        user_id: u64,
        limit: u8,
    },
    CancelReminder {
        guild_id: u64,
        reminder_id: i64,
        user_id: u64,
    },
    ClaimDueReminders {
        guild_id: u64,
        now_unix: u64,
        limit: u8,
    },
    CompleteReminder {
        guild_id: u64,
        reminder_id: i64,
    },
    RetryReminder {
        guild_id: u64,
        reminder_id: i64,
        next_attempt_unix: u64,
    },
}

#[derive(Clone, Default)]
pub struct FakeModuleServices {
    calls: Arc<Mutex<Vec<ServiceCall>>>,
    raid_counts: Arc<Mutex<HashMap<u64, u64>>>,
    lockdowns: Arc<Mutex<HashSet<u64>>>,
    scheduled_claims: Arc<Mutex<HashSet<(u64, String, u64)>>>,
}

impl FakeModuleServices {
    pub fn calls(&self) -> Vec<ServiceCall> {
        self.calls.lock().expect("fake lock poisoned").clone()
    }
}

#[async_trait]
impl ModuleServices for FakeModuleServices {
    async fn managed_embed(
        &self,
        _guild_id: u64,
        template_id: i64,
    ) -> anyhow::Result<Option<ManagedEmbedResult>> {
        Ok(Some(ManagedEmbedResult {
            id: template_id,
            name: "Rules".into(),
            payload: bot_core::EmbedPayload {
                title: "Welcome {user} to {server}".into(),
                description: "Read the rules".into(),
                ..Default::default()
            },
        }))
    }
    async fn schedule_message_deletion(
        &self,
        _: u64,
        _: u64,
        _: u64,
        _: &str,
        _: u64,
        _: u64,
    ) -> anyhow::Result<bool> {
        Ok(true)
    }
    async fn claim_due_message_deletions(
        &self,
        _: u64,
        n: u64,
        _: u8,
    ) -> anyhow::Result<Vec<MessageDeletionJobResult>> {
        Ok(vec![MessageDeletionJobResult {
            channel_id: 20,
            message_id: 40,
            attempts: u8::try_from(n.min(1)).unwrap_or(1),
        }])
    }
    async fn finish_message_deletion(
        &self,
        _: u64,
        _: u64,
        _: u64,
        _: bool,
        _: u64,
    ) -> anyhow::Result<bool> {
        Ok(false)
    }
    async fn claim_suggestion_cooldown(
        &self,
        guild_id: u64,
        user_id: u64,
        cooldown_seconds: u32,
    ) -> anyhow::Result<bool> {
        self.calls
            .lock()
            .expect("fake lock poisoned")
            .push(ServiceCall::ClaimSuggestionCooldown {
                guild_id,
                user_id,
                cooldown_seconds,
            });
        Ok(true)
    }

    async fn create_suggestion(&self, input: SuggestionInput) -> anyhow::Result<SuggestionResult> {
        let mut calls = self.calls.lock().expect("fake lock poisoned");
        calls.push(ServiceCall::CreateSuggestion(input));
        Ok(SuggestionResult {
            id: i64::try_from(calls.len()).unwrap_or(i64::MAX),
            status: "pending".into(),
            upvotes: 0,
            downvotes: 0,
        })
    }

    async fn set_suggestion_message_id(
        &self,
        guild_id: u64,
        suggestion_id: i64,
        message_id: u64,
    ) -> anyhow::Result<()> {
        self.calls
            .lock()
            .expect("fake lock poisoned")
            .push(ServiceCall::SetSuggestionMessageId {
                guild_id,
                suggestion_id,
                message_id,
            });
        Ok(())
    }

    async fn vote_suggestion(
        &self,
        guild_id: u64,
        suggestion_id: i64,
        user_id: u64,
        direction: i8,
    ) -> anyhow::Result<Option<SuggestionResult>> {
        self.calls
            .lock()
            .expect("fake lock poisoned")
            .push(ServiceCall::VoteSuggestion {
                guild_id,
                suggestion_id,
                user_id,
                direction,
            });
        Ok(Some(SuggestionResult {
            id: suggestion_id,
            status: "pending".into(),
            upvotes: u64::from(direction == 1),
            downvotes: u64::from(direction == -1),
        }))
    }

    async fn update_suggestion_status(
        &self,
        guild_id: u64,
        suggestion_id: i64,
        actor_id: u64,
        status: &str,
        staff_note: Option<&str>,
    ) -> anyhow::Result<Option<SuggestionResult>> {
        self.calls
            .lock()
            .expect("fake lock poisoned")
            .push(ServiceCall::UpdateSuggestionStatus {
                guild_id,
                suggestion_id,
                actor_id,
                status: status.into(),
                staff_note: staff_note.map(str::to_owned),
            });
        Ok(Some(SuggestionResult {
            id: suggestion_id,
            status: status.into(),
            upvotes: 0,
            downvotes: 0,
        }))
    }

    async fn create_giveaway(&self, input: GiveawayInput) -> anyhow::Result<GiveawayResult> {
        self.calls
            .lock()
            .expect("fake lock poisoned")
            .push(ServiceCall::CreateGiveaway(input.clone()));
        Ok(fake_giveaway(12, &input))
    }

    async fn activate_giveaway(
        &self,
        guild_id: u64,
        giveaway_id: i64,
        message_id: u64,
    ) -> anyhow::Result<bool> {
        self.calls
            .lock()
            .expect("fake lock poisoned")
            .push(ServiceCall::ActivateGiveaway {
                guild_id,
                giveaway_id,
                message_id,
            });
        Ok(true)
    }

    async fn fail_giveaway_delivery(&self, guild_id: u64, giveaway_id: i64) -> anyhow::Result<()> {
        self.calls
            .lock()
            .expect("fake lock poisoned")
            .push(ServiceCall::FailGiveawayDelivery {
                guild_id,
                giveaway_id,
            });
        Ok(())
    }

    async fn giveaway(
        &self,
        guild_id: u64,
        giveaway_id: i64,
    ) -> anyhow::Result<Option<GiveawayResult>> {
        self.calls
            .lock()
            .expect("fake lock poisoned")
            .push(ServiceCall::Giveaway {
                guild_id,
                giveaway_id,
            });
        let mut result = fake_giveaway(
            giveaway_id,
            &GiveawayInput {
                guild_id,
                channel_id: 20,
                creator_id: 40,
                title: "Nitro".into(),
                description: String::new(),
                winner_count: 1,
                minimum_account_age_days: 0,
                required_role_ids: vec![],
                bonus_role_id: None,
                bonus_entries: 1,
                ends_at_unix: u64::MAX,
            },
        );
        result.status = "active".into();
        Ok(Some(result))
    }

    async fn enter_giveaway(
        &self,
        guild_id: u64,
        giveaway_id: i64,
        user_id: u64,
        weight: u8,
    ) -> anyhow::Result<Option<bool>> {
        self.calls
            .lock()
            .expect("fake lock poisoned")
            .push(ServiceCall::EnterGiveaway {
                guild_id,
                giveaway_id,
                user_id,
                weight,
            });
        Ok(Some(true))
    }

    async fn due_giveaway_ids(&self, guild_id: u64, now_unix: u64) -> anyhow::Result<Vec<i64>> {
        self.calls
            .lock()
            .expect("fake lock poisoned")
            .push(ServiceCall::DueGiveawayIds { guild_id, now_unix });
        Ok(vec![])
    }

    async fn pending_giveaway_announcement_ids(&self, guild_id: u64) -> anyhow::Result<Vec<i64>> {
        self.calls
            .lock()
            .expect("fake lock poisoned")
            .push(ServiceCall::PendingGiveawayAnnouncementIds { guild_id });
        Ok(vec![])
    }

    async fn mark_giveaway_announced(&self, guild_id: u64, giveaway_id: i64) -> anyhow::Result<()> {
        self.calls
            .lock()
            .expect("fake lock poisoned")
            .push(ServiceCall::MarkGiveawayAnnounced {
                guild_id,
                giveaway_id,
            });
        Ok(())
    }

    async fn end_giveaway(
        &self,
        guild_id: u64,
        giveaway_id: i64,
        force: bool,
    ) -> anyhow::Result<Option<GiveawayResult>> {
        self.calls
            .lock()
            .expect("fake lock poisoned")
            .push(ServiceCall::EndGiveaway {
                guild_id,
                giveaway_id,
                force,
            });
        let mut result = fake_giveaway(
            giveaway_id,
            &GiveawayInput {
                guild_id,
                channel_id: 20,
                creator_id: 40,
                title: "Nitro".into(),
                description: String::new(),
                winner_count: 1,
                minimum_account_age_days: 0,
                required_role_ids: vec![],
                bonus_role_id: None,
                bonus_entries: 1,
                ends_at_unix: 1,
            },
        );
        result.status = "ended".into();
        result.winner_ids = vec![50];
        result.entrant_count = 1;
        result.total_entries = 1;
        Ok(Some(result))
    }

    async fn reroll_giveaway(
        &self,
        guild_id: u64,
        giveaway_id: i64,
    ) -> anyhow::Result<Option<GiveawayResult>> {
        self.calls
            .lock()
            .expect("fake lock poisoned")
            .push(ServiceCall::RerollGiveaway {
                guild_id,
                giveaway_id,
            });
        let mut result = fake_giveaway(
            giveaway_id,
            &GiveawayInput {
                guild_id,
                channel_id: 20,
                creator_id: 40,
                title: "Nitro".into(),
                description: String::new(),
                winner_count: 1,
                minimum_account_age_days: 0,
                required_role_ids: vec![],
                bonus_role_id: None,
                bonus_entries: 1,
                ends_at_unix: 1,
            },
        );
        result.status = "ended".into();
        result.winner_ids = vec![60];
        result.reroll_count = 1;
        Ok(Some(result))
    }

    async fn create_reminder(
        &self,
        input: ReminderInput,
    ) -> anyhow::Result<Option<ReminderResult>> {
        self.calls
            .lock()
            .expect("fake lock poisoned")
            .push(ServiceCall::CreateReminder(input.clone()));
        Ok(Some(fake_reminder(14, &input)))
    }

    async fn user_reminders(
        &self,
        guild_id: u64,
        user_id: u64,
        limit: u8,
    ) -> anyhow::Result<Vec<ReminderResult>> {
        self.calls
            .lock()
            .expect("fake lock poisoned")
            .push(ServiceCall::UserReminders {
                guild_id,
                user_id,
                limit,
            });
        Ok(vec![])
    }

    async fn cancel_reminder(
        &self,
        guild_id: u64,
        reminder_id: i64,
        user_id: u64,
    ) -> anyhow::Result<bool> {
        self.calls
            .lock()
            .expect("fake lock poisoned")
            .push(ServiceCall::CancelReminder {
                guild_id,
                reminder_id,
                user_id,
            });
        Ok(true)
    }

    async fn claim_due_reminders(
        &self,
        guild_id: u64,
        now_unix: u64,
        limit: u8,
    ) -> anyhow::Result<Vec<ReminderResult>> {
        self.calls
            .lock()
            .expect("fake lock poisoned")
            .push(ServiceCall::ClaimDueReminders {
                guild_id,
                now_unix,
                limit,
            });
        Ok(vec![ReminderResult {
            id: 14,
            channel_id: Some(20),
            user_id: 30,
            message: "check the oven".into(),
            delivery_mode: "channel".into(),
            status: "delivering".into(),
            remind_at_unix: now_unix,
            attempts: 1,
        }])
    }

    async fn complete_reminder(&self, guild_id: u64, reminder_id: i64) -> anyhow::Result<()> {
        self.calls
            .lock()
            .expect("fake lock poisoned")
            .push(ServiceCall::CompleteReminder {
                guild_id,
                reminder_id,
            });
        Ok(())
    }

    async fn retry_reminder(
        &self,
        guild_id: u64,
        reminder_id: i64,
        next_attempt_unix: u64,
    ) -> anyhow::Result<bool> {
        self.calls
            .lock()
            .expect("fake lock poisoned")
            .push(ServiceCall::RetryReminder {
                guild_id,
                reminder_id,
                next_attempt_unix,
            });
        Ok(false)
    }

    async fn create_moderation_case(&self, input: ModerationCaseInput) -> anyhow::Result<i64> {
        let mut calls = self.calls.lock().expect("fake lock poisoned");
        calls.push(ServiceCall::CreateModerationCase(input));
        Ok(i64::try_from(calls.len()).unwrap_or(i64::MAX))
    }

    async fn award_xp(&self, input: XpAwardInput) -> anyhow::Result<XpAwardResult> {
        self.calls
            .lock()
            .expect("fake lock poisoned")
            .push(ServiceCall::AwardXp(input.clone()));
        Ok(XpAwardResult {
            awarded: true,
            total_xp: u64::from(input.amount),
            level: 0,
            leveled_up: false,
        })
    }

    async fn xp_profile(&self, guild_id: u64, user_id: u64) -> anyhow::Result<XpProfile> {
        self.calls
            .lock()
            .expect("fake lock poisoned")
            .push(ServiceCall::XpProfile { guild_id, user_id });
        Ok(XpProfile {
            user_id,
            total_xp: 625,
            level: 2,
            message_count: 25,
            rank: 3,
        })
    }

    async fn xp_leaderboard(&self, guild_id: u64, limit: u8) -> anyhow::Result<Vec<XpProfile>> {
        self.calls
            .lock()
            .expect("fake lock poisoned")
            .push(ServiceCall::XpLeaderboard { guild_id, limit });
        Ok(vec![XpProfile {
            user_id: 42,
            total_xp: 900,
            level: 3,
            message_count: 50,
            rank: 1,
        }])
    }

    async fn open_ticket_channel(
        &self,
        guild_id: u64,
        opener_id: u64,
    ) -> anyhow::Result<Option<u64>> {
        self.calls
            .lock()
            .expect("fake lock poisoned")
            .push(ServiceCall::FindOpenTicket {
                guild_id,
                opener_id,
            });
        Ok(None)
    }

    async fn record_open_ticket(&self, input: TicketRecordInput) -> anyhow::Result<()> {
        self.calls
            .lock()
            .expect("fake lock poisoned")
            .push(ServiceCall::RecordOpenTicket(input));
        Ok(())
    }

    async fn close_ticket_channel(
        &self,
        guild_id: u64,
        channel_id: u64,
        closed_by: u64,
    ) -> anyhow::Result<bool> {
        self.calls
            .lock()
            .expect("fake lock poisoned")
            .push(ServiceCall::CloseTicket {
                guild_id,
                channel_id,
                closed_by,
            });
        Ok(true)
    }

    async fn claim_starboard_post(
        &self,
        guild_id: u64,
        source_message_id: u64,
        starboard_channel_id: u64,
    ) -> anyhow::Result<bool> {
        self.calls
            .lock()
            .expect("fake lock poisoned")
            .push(ServiceCall::ClaimStarboardPost {
                guild_id,
                source_message_id,
                starboard_channel_id,
            });
        Ok(true)
    }

    async fn record_raid_join(
        &self,
        guild_id: u64,
        user_id: u64,
        window_seconds: u16,
    ) -> anyhow::Result<u64> {
        self.calls
            .lock()
            .expect("fake lock poisoned")
            .push(ServiceCall::RecordRaidJoin {
                guild_id,
                user_id,
                window_seconds,
            });
        let mut counts = self.raid_counts.lock().expect("fake lock poisoned");
        let count = counts.entry(guild_id).or_default();
        *count += 1;
        Ok(*count)
    }

    async fn set_raid_lockdown(&self, guild_id: u64, duration_seconds: u64) -> anyhow::Result<()> {
        self.calls
            .lock()
            .expect("fake lock poisoned")
            .push(ServiceCall::SetRaidLockdown {
                guild_id,
                duration_seconds,
            });
        self.lockdowns
            .lock()
            .expect("fake lock poisoned")
            .insert(guild_id);
        Ok(())
    }

    async fn clear_raid_lockdown(&self, guild_id: u64) -> anyhow::Result<()> {
        self.calls
            .lock()
            .expect("fake lock poisoned")
            .push(ServiceCall::ClearRaidLockdown { guild_id });
        self.lockdowns
            .lock()
            .expect("fake lock poisoned")
            .remove(&guild_id);
        Ok(())
    }

    async fn raid_lockdown_active(&self, guild_id: u64) -> anyhow::Result<bool> {
        self.calls
            .lock()
            .expect("fake lock poisoned")
            .push(ServiceCall::RaidLockdownActive { guild_id });
        Ok(self
            .lockdowns
            .lock()
            .expect("fake lock poisoned")
            .contains(&guild_id))
    }

    async fn claim_scheduled_message(
        &self,
        guild_id: u64,
        schedule_id: &str,
        occurrence_unix: u64,
    ) -> anyhow::Result<bool> {
        self.calls
            .lock()
            .expect("fake lock poisoned")
            .push(ServiceCall::ClaimScheduledMessage {
                guild_id,
                schedule_id: schedule_id.into(),
                occurrence_unix,
            });
        Ok(self
            .scheduled_claims
            .lock()
            .expect("fake lock poisoned")
            .insert((guild_id, schedule_id.into(), occurrence_unix)))
    }

    async fn release_scheduled_message(
        &self,
        guild_id: u64,
        schedule_id: &str,
        occurrence_unix: u64,
    ) -> anyhow::Result<()> {
        self.calls
            .lock()
            .expect("fake lock poisoned")
            .push(ServiceCall::ReleaseScheduledMessage {
                guild_id,
                schedule_id: schedule_id.into(),
                occurrence_unix,
            });
        self.scheduled_claims
            .lock()
            .expect("fake lock poisoned")
            .remove(&(guild_id, schedule_id.into(), occurrence_unix));
        Ok(())
    }

    async fn create_form_submission(&self, input: FormSubmissionInput) -> anyhow::Result<i64> {
        let mut calls = self.calls.lock().expect("fake lock poisoned");
        calls.push(ServiceCall::CreateFormSubmission(input));
        Ok(i64::try_from(calls.len()).unwrap_or(i64::MAX))
    }
}

fn fake_giveaway(id: i64, input: &GiveawayInput) -> GiveawayResult {
    GiveawayResult {
        id,
        channel_id: input.channel_id,
        title: input.title.clone(),
        status: "pending_delivery".into(),
        winner_count: input.winner_count,
        minimum_account_age_days: input.minimum_account_age_days,
        required_role_ids: input.required_role_ids.clone(),
        bonus_role_id: input.bonus_role_id,
        bonus_entries: input.bonus_entries,
        winner_ids: vec![],
        entrant_count: 0,
        total_entries: 0,
        ends_at_unix: input.ends_at_unix,
        reroll_count: 0,
    }
}

fn fake_reminder(id: i64, input: &ReminderInput) -> ReminderResult {
    ReminderResult {
        id,
        channel_id: input.channel_id,
        user_id: input.user_id,
        message: input.message.clone(),
        delivery_mode: input.delivery_mode.clone(),
        status: "scheduled".into(),
        remind_at_unix: input.remind_at_unix,
        attempts: 0,
    }
}

#[derive(Clone, Default)]
pub struct FakeDiscord {
    state: Arc<Mutex<FakeState>>,
}

#[derive(Default)]
struct FakeState {
    calls: Vec<DiscordCall>,
    failures: VecDeque<DiscordError>,
    guild_channels: Vec<GuildChannel>,
    guild_roles: Vec<GuildRole>,
    next_channel_id: u64,
    next_message_id: u64,
    message_reaction_summary: MessageReactionSummary,
}

impl FakeDiscord {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn calls(&self) -> Vec<DiscordCall> {
        self.state.lock().expect("fake lock poisoned").calls.clone()
    }

    pub fn take_calls(&self) -> Vec<DiscordCall> {
        std::mem::take(&mut self.state.lock().expect("fake lock poisoned").calls)
    }

    pub fn fail_next(&self, error: DiscordError) {
        self.state
            .lock()
            .expect("fake lock poisoned")
            .failures
            .push_back(error);
    }

    pub fn set_guild_channels(&self, channels: Vec<GuildChannel>) {
        self.state
            .lock()
            .expect("fake lock poisoned")
            .guild_channels = channels;
    }

    pub fn set_guild_roles(&self, roles: Vec<GuildRole>) {
        self.state.lock().expect("fake lock poisoned").guild_roles = roles;
    }

    pub fn set_next_channel_id(&self, channel_id: u64) {
        self.state
            .lock()
            .expect("fake lock poisoned")
            .next_channel_id = channel_id;
    }

    pub fn set_next_message_id(&self, message_id: u64) {
        self.state
            .lock()
            .expect("fake lock poisoned")
            .next_message_id = message_id;
    }

    pub fn set_message_reaction_summary(&self, summary: MessageReactionSummary) {
        self.state
            .lock()
            .expect("fake lock poisoned")
            .message_reaction_summary = summary;
    }

    fn record(&self, call: DiscordCall) -> Result<(), DiscordError> {
        let mut state = self.state.lock().expect("fake lock poisoned");
        state.calls.push(call);
        match state.failures.pop_front() {
            Some(error) => Err(error),
            None => Ok(()),
        }
    }
}

#[async_trait]
impl DiscordApi for FakeDiscord {
    async fn recent_message_ids(&self, _: u64, limit: u16) -> Result<Vec<u64>, DiscordError> {
        Ok((1..=u64::from(limit)).collect())
    }
    async fn register_commands(&self, commands: &[CommandDefinition]) -> Result<(), DiscordError> {
        self.record(DiscordCall::RegisterCommands {
            commands: commands.to_vec(),
        })
    }

    async fn respond_to_command(
        &self,
        interaction: &CommandInteraction,
        response: CommandResponse,
    ) -> Result<(), DiscordError> {
        self.record(DiscordCall::RespondToCommand {
            interaction_id: interaction.id,
            response,
        })
    }

    async fn open_modal(
        &self,
        interaction: &CommandInteraction,
        modal: ModalDefinition,
    ) -> Result<(), DiscordError> {
        self.record(DiscordCall::OpenModal {
            interaction_id: interaction.id,
            modal,
        })
    }

    async fn respond_to_modal(
        &self,
        interaction: &ModalSubmitInteraction,
        response: CommandResponse,
    ) -> Result<(), DiscordError> {
        self.record(DiscordCall::RespondToModal {
            interaction_id: interaction.id,
            response,
        })
    }

    async fn respond_to_component(
        &self,
        interaction: &ComponentInteraction,
        response: CommandResponse,
    ) -> Result<(), DiscordError> {
        self.record(DiscordCall::RespondToComponent {
            interaction_id: interaction.id,
            response,
        })
    }

    async fn send_message(&self, channel_id: u64, content: &str) -> Result<(), DiscordError> {
        self.record(DiscordCall::SendMessage {
            channel_id,
            content: content.to_owned(),
        })
    }

    async fn send_message_with_buttons(
        &self,
        channel_id: u64,
        content: &str,
        buttons: &[MessageButton],
    ) -> Result<u64, DiscordError> {
        self.record(DiscordCall::SendMessageWithButtons {
            channel_id,
            content: content.to_owned(),
            buttons: buttons.to_vec(),
        })?;
        Ok(self
            .state
            .lock()
            .expect("fake lock poisoned")
            .next_message_id)
    }

    async fn send_rich_embed(
        &self,
        channel_id: u64,
        embed: &RichEmbed,
    ) -> Result<u64, DiscordError> {
        self.record(DiscordCall::SendRichEmbed {
            channel_id,
            embed: embed.clone(),
        })?;
        Ok(self
            .state
            .lock()
            .expect("fake lock poisoned")
            .next_message_id)
    }

    async fn edit_rich_embed(
        &self,
        channel_id: u64,
        message_id: u64,
        embed: &RichEmbed,
    ) -> Result<(), DiscordError> {
        self.record(DiscordCall::EditRichEmbed {
            channel_id,
            message_id,
            embed: embed.clone(),
        })
    }

    async fn send_direct_message(&self, user_id: u64, content: &str) -> Result<(), DiscordError> {
        self.record(DiscordCall::SendDirectMessage {
            user_id,
            content: content.to_owned(),
        })
    }

    async fn add_member_role(
        &self,
        guild_id: u64,
        user_id: u64,
        role_id: u64,
    ) -> Result<(), DiscordError> {
        self.record(DiscordCall::AddMemberRole {
            guild_id,
            user_id,
            role_id,
        })
    }

    async fn remove_member_role(
        &self,
        guild_id: u64,
        user_id: u64,
        role_id: u64,
        reason: &str,
    ) -> Result<(), DiscordError> {
        self.record(DiscordCall::RemoveMemberRole {
            guild_id,
            user_id,
            role_id,
            reason: reason.to_owned(),
        })
    }

    async fn timeout_member(
        &self,
        guild_id: u64,
        user_id: u64,
        duration_seconds: u64,
        reason: &str,
    ) -> Result<(), DiscordError> {
        self.record(DiscordCall::TimeoutMember {
            guild_id,
            user_id,
            duration_seconds,
            reason: reason.to_owned(),
        })
    }

    async fn kick_member(
        &self,
        guild_id: u64,
        user_id: u64,
        reason: &str,
    ) -> Result<(), DiscordError> {
        self.record(DiscordCall::KickMember {
            guild_id,
            user_id,
            reason: reason.to_owned(),
        })
    }

    async fn ban_member(
        &self,
        guild_id: u64,
        user_id: u64,
        delete_message_seconds: u32,
        reason: &str,
    ) -> Result<(), DiscordError> {
        self.record(DiscordCall::BanMember {
            guild_id,
            user_id,
            delete_message_seconds,
            reason: reason.to_owned(),
        })
    }

    async fn unban_member(
        &self,
        guild_id: u64,
        user_id: u64,
        reason: &str,
    ) -> Result<(), DiscordError> {
        self.record(DiscordCall::UnbanMember {
            guild_id,
            user_id,
            reason: reason.to_owned(),
        })
    }

    async fn delete_message(
        &self,
        channel_id: u64,
        message_id: u64,
        reason: &str,
    ) -> Result<(), DiscordError> {
        self.record(DiscordCall::DeleteMessage {
            channel_id,
            message_id,
            reason: reason.to_owned(),
        })
    }

    async fn guild_channels(&self, guild_id: u64) -> Result<Vec<GuildChannel>, DiscordError> {
        self.record(DiscordCall::GuildChannels { guild_id })?;
        Ok(self
            .state
            .lock()
            .expect("fake lock poisoned")
            .guild_channels
            .clone())
    }

    async fn guild_roles(&self, guild_id: u64) -> Result<Vec<GuildRole>, DiscordError> {
        self.record(DiscordCall::GuildRoles { guild_id })?;
        Ok(self
            .state
            .lock()
            .expect("fake lock poisoned")
            .guild_roles
            .clone())
    }

    async fn create_ticket_channel(
        &self,
        guild_id: u64,
        category_id: u64,
        opener_id: u64,
        support_role_ids: &[u64],
        name: &str,
        reason: &str,
    ) -> Result<u64, DiscordError> {
        self.record(DiscordCall::CreateTicketChannel {
            guild_id,
            category_id,
            opener_id,
            support_role_ids: support_role_ids.to_vec(),
            name: name.to_owned(),
            reason: reason.to_owned(),
        })?;
        Ok(self
            .state
            .lock()
            .expect("fake lock poisoned")
            .next_channel_id)
    }

    async fn delete_channel(&self, channel_id: u64, reason: &str) -> Result<(), DiscordError> {
        self.record(DiscordCall::DeleteChannel {
            channel_id,
            reason: reason.to_owned(),
        })
    }

    async fn message_reaction_summary(
        &self,
        channel_id: u64,
        message_id: u64,
        emoji: &str,
    ) -> Result<MessageReactionSummary, DiscordError> {
        self.record(DiscordCall::MessageReactionSummary {
            channel_id,
            message_id,
            emoji: emoji.to_owned(),
        })?;
        Ok(self
            .state
            .lock()
            .expect("fake lock poisoned")
            .message_reaction_summary
            .clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn records_calls_and_can_inject_failures() {
        let fake = FakeDiscord::new();
        let interaction = CommandInteraction {
            id: 1,
            token: "test-token".to_owned(),
            guild_id: 2,
            channel_id: Some(3),
            actor_id: 4,
            actor_role_ids: vec![],
            actor_permissions: u64::MAX,
            command_name: "ping".to_owned(),
            options: Default::default(),
        };
        fake.respond_to_command(&interaction, CommandResponse::public("Pong!"))
            .await
            .unwrap();
        fake.fail_next(DiscordError::Request("rate limited".to_owned()));
        assert!(
            fake.respond_to_command(&interaction, CommandResponse::public("Pong!"))
                .await
                .is_err()
        );
        assert_eq!(fake.calls().len(), 2);
    }
}

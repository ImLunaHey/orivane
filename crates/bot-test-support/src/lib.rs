use std::{
    collections::{HashMap, HashSet, VecDeque},
    sync::{Arc, Mutex},
};

use async_trait::async_trait;
use bot_discord::{
    CommandDefinition, CommandInteraction, CommandResponse, DiscordApi, DiscordError, GuildChannel,
    GuildRole, MessageReactionSummary, ModalDefinition, ModalSubmitInteraction,
};
use bot_module_api::{
    FormSubmissionInput, ModerationCaseInput, ModuleServices, TicketRecordInput, XpAwardInput,
    XpAwardResult, XpProfile,
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
    SendMessage {
        channel_id: u64,
        content: String,
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

    async fn send_message(&self, channel_id: u64, content: &str) -> Result<(), DiscordError> {
        self.record(DiscordCall::SendMessage {
            channel_id,
            content: content.to_owned(),
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

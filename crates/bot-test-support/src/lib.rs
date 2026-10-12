use std::{
    collections::VecDeque,
    sync::{Arc, Mutex},
};

use async_trait::async_trait;
use bot_discord::{
    CommandDefinition, CommandInteraction, CommandResponse, DiscordApi, DiscordError, GuildChannel,
    GuildRole,
};
use bot_module_api::{ModerationCaseInput, ModuleServices};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DiscordCall {
    RegisterCommands {
        commands: Vec<CommandDefinition>,
    },
    RespondToCommand {
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
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ServiceCall {
    CreateModerationCase(ModerationCaseInput),
}

#[derive(Clone, Default)]
pub struct FakeModuleServices {
    calls: Arc<Mutex<Vec<ServiceCall>>>,
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

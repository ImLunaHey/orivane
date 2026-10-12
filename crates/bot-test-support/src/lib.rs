use std::{
    collections::VecDeque,
    sync::{Arc, Mutex},
};

use async_trait::async_trait;
use bot_discord::{
    CommandDefinition, CommandInteraction, CommandResponse, DiscordApi, DiscordError, GuildChannel,
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
    SendMessage {
        channel_id: u64,
        content: String,
    },
    AddMemberRole {
        guild_id: u64,
        user_id: u64,
        role_id: u64,
    },
    GuildChannels {
        guild_id: u64,
    },
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

    async fn guild_channels(&self, guild_id: u64) -> Result<Vec<GuildChannel>, DiscordError> {
        self.record(DiscordCall::GuildChannels { guild_id })?;
        Ok(self
            .state
            .lock()
            .expect("fake lock poisoned")
            .guild_channels
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
            command_name: "ping".to_owned(),
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

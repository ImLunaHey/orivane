use std::sync::Arc;

use async_trait::async_trait;
use serde::Serialize;
use thiserror::Error;
use twilight_http::Client;
use twilight_model::{
    channel::message::MessageFlags,
    http::interaction::{
        InteractionResponse as TwilightInteractionResponse, InteractionResponseType,
    },
    id::{
        Id,
        marker::{ApplicationMarker, ChannelMarker, GuildMarker, RoleMarker, UserMarker},
    },
};
use twilight_util::builder::InteractionResponseDataBuilder;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CommandDefinition {
    pub name: &'static str,
    pub description: &'static str,
    pub guild_only: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CommandInteraction {
    pub id: u64,
    pub token: String,
    pub guild_id: u64,
    pub command_name: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CommandResponse {
    pub content: String,
    pub ephemeral: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct GuildChannel {
    pub id: String,
    pub name: String,
    pub kind: String,
    pub position: i32,
    pub parent_id: Option<String>,
}

impl CommandResponse {
    pub fn public(content: impl Into<String>) -> Self {
        Self {
            content: content.into(),
            ephemeral: false,
        }
    }

    pub fn ephemeral(content: impl Into<String>) -> Self {
        Self {
            content: content.into(),
            ephemeral: true,
        }
    }
}

#[derive(Clone, Debug, Error, PartialEq, Eq)]
pub enum DiscordError {
    #[error("Discord API request failed: {0}")]
    Request(String),
}

#[async_trait]
pub trait DiscordApi: Send + Sync {
    async fn register_commands(&self, commands: &[CommandDefinition]) -> Result<(), DiscordError>;

    async fn respond_to_command(
        &self,
        interaction: &CommandInteraction,
        response: CommandResponse,
    ) -> Result<(), DiscordError>;

    async fn send_message(&self, channel_id: u64, content: &str) -> Result<(), DiscordError>;

    async fn add_member_role(
        &self,
        guild_id: u64,
        user_id: u64,
        role_id: u64,
    ) -> Result<(), DiscordError>;

    async fn guild_channels(&self, guild_id: u64) -> Result<Vec<GuildChannel>, DiscordError>;
}

pub struct TwilightDiscord {
    http: Arc<Client>,
    application_id: Id<ApplicationMarker>,
}

impl TwilightDiscord {
    pub async fn connect(token: String) -> Result<Self, DiscordError> {
        let http = Arc::new(Client::new(token));
        let application = http
            .current_user_application()
            .await
            .map_err(request_error)?
            .model()
            .await
            .map_err(request_error)?;
        Ok(Self {
            http,
            application_id: application.id,
        })
    }
}

#[async_trait]
impl DiscordApi for TwilightDiscord {
    async fn register_commands(&self, commands: &[CommandDefinition]) -> Result<(), DiscordError> {
        let client = self.http.interaction(self.application_id);
        for command in commands {
            client
                .create_global_command()
                .chat_input(command.name, command.description)
                .dm_permission(!command.guild_only)
                .await
                .map_err(request_error)?;
        }
        Ok(())
    }

    async fn respond_to_command(
        &self,
        interaction: &CommandInteraction,
        response: CommandResponse,
    ) -> Result<(), DiscordError> {
        let mut data = InteractionResponseDataBuilder::new().content(response.content);
        if response.ephemeral {
            data = data.flags(MessageFlags::EPHEMERAL);
        }
        let response = TwilightInteractionResponse {
            kind: InteractionResponseType::ChannelMessageWithSource,
            data: Some(data.build()),
        };
        self.http
            .interaction(self.application_id)
            .create_response(Id::new(interaction.id), &interaction.token, &response)
            .await
            .map_err(request_error)?;
        Ok(())
    }

    async fn send_message(&self, channel_id: u64, content: &str) -> Result<(), DiscordError> {
        self.http
            .create_message(Id::<ChannelMarker>::new(channel_id))
            .content(content)
            .await
            .map_err(request_error)?;
        Ok(())
    }

    async fn add_member_role(
        &self,
        guild_id: u64,
        user_id: u64,
        role_id: u64,
    ) -> Result<(), DiscordError> {
        self.http
            .add_guild_member_role(
                Id::<GuildMarker>::new(guild_id),
                Id::<UserMarker>::new(user_id),
                Id::<RoleMarker>::new(role_id),
            )
            .await
            .map_err(request_error)?;
        Ok(())
    }

    async fn guild_channels(&self, guild_id: u64) -> Result<Vec<GuildChannel>, DiscordError> {
        let channels = self
            .http
            .guild_channels(Id::<GuildMarker>::new(guild_id))
            .await
            .map_err(request_error)?
            .models()
            .await
            .map_err(request_error)?;
        let mut channels = channels
            .into_iter()
            .filter_map(|channel| {
                Some(GuildChannel {
                    id: channel.id.to_string(),
                    name: channel.name?,
                    kind: channel.kind.name().to_owned(),
                    position: channel.position.unwrap_or_default(),
                    parent_id: channel.parent_id.map(|id| id.to_string()),
                })
            })
            .collect::<Vec<_>>();
        channels.sort_by_key(|channel| channel.position);
        Ok(channels)
    }
}

fn request_error(error: impl std::fmt::Display) -> DiscordError {
    DiscordError::Request(error.to_string())
}

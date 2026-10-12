use std::{
    collections::BTreeMap,
    sync::Arc,
    time::{SystemTime, UNIX_EPOCH},
};

use async_trait::async_trait;
use serde::Serialize;
use thiserror::Error;
use twilight_http::{Client, request::AuditLogReason};
use twilight_model::{
    application::command::{
        CommandOption, CommandOptionType, CommandOptionValue as TwilightCommandOptionValue,
        CommandType,
    },
    application::interaction::InteractionContextType,
    channel::{
        message::MessageFlags,
        message::component::{
            ActionRow, Button, ButtonStyle, Component, Label, SelectMenu, SelectMenuOption,
            SelectMenuType, TextInput, TextInputStyle,
        },
        message::embed::{
            Embed, EmbedAuthor, EmbedField as TwilightEmbedField, EmbedFooter, EmbedImage,
            EmbedThumbnail,
        },
        permission_overwrite::{PermissionOverwrite, PermissionOverwriteType},
    },
    guild::Permissions,
    http::interaction::{
        InteractionResponse as TwilightInteractionResponse, InteractionResponseType,
    },
    http::permission_overwrite::{
        PermissionOverwrite as HttpPermissionOverwrite,
        PermissionOverwriteType as HttpPermissionOverwriteType,
    },
    id::{
        Id,
        marker::{
            ApplicationMarker, ChannelMarker, EmojiMarker, GuildMarker, MessageMarker, RoleMarker,
            UserMarker,
        },
    },
    util::Timestamp,
};
use twilight_util::builder::{InteractionResponseDataBuilder, command::CommandBuilder};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CommandDefinition {
    pub name: &'static str,
    pub description: &'static str,
    pub guild_only: bool,
    pub default_permissions: Option<u64>,
    pub options: &'static [CommandOptionDefinition],
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CommandOptionKind {
    String,
    Integer,
    Boolean,
    User,
    Channel,
    Role,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CommandOptionDefinition {
    pub name: &'static str,
    pub description: &'static str,
    pub kind: CommandOptionKind,
    pub required: bool,
    pub min_value: Option<i64>,
    pub max_value: Option<i64>,
    pub min_length: Option<u16>,
    pub max_length: Option<u16>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CommandValue {
    String(String),
    Integer(i64),
    Boolean(bool),
    User(u64),
    Channel(u64),
    Role(u64),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CommandInteraction {
    pub id: u64,
    pub token: String,
    pub guild_id: u64,
    pub channel_id: Option<u64>,
    pub actor_id: u64,
    pub actor_role_ids: Vec<u64>,
    pub actor_permissions: u64,
    pub command_name: String,
    pub options: BTreeMap<String, CommandValue>,
}

impl CommandInteraction {
    pub fn string(&self, name: &str) -> Option<&str> {
        match self.options.get(name) {
            Some(CommandValue::String(value)) => Some(value),
            _ => None,
        }
    }

    pub fn integer(&self, name: &str) -> Option<i64> {
        match self.options.get(name) {
            Some(CommandValue::Integer(value)) => Some(*value),
            _ => None,
        }
    }

    pub fn user(&self, name: &str) -> Option<u64> {
        match self.options.get(name) {
            Some(CommandValue::User(value)) => Some(*value),
            _ => None,
        }
    }

    pub fn role(&self, name: &str) -> Option<u64> {
        match self.options.get(name) {
            Some(CommandValue::Role(value)) => Some(*value),
            _ => None,
        }
    }

    pub fn channel(&self, name: &str) -> Option<u64> {
        match self.options.get(name) {
            Some(CommandValue::Channel(value)) => Some(*value),
            _ => None,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CommandResponse {
    pub content: String,
    pub ephemeral: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ModalFieldStyle {
    Short,
    Paragraph,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ModalFieldDefinition {
    pub custom_id: String,
    pub label: String,
    pub style: ModalFieldStyle,
    pub required: bool,
    pub min_length: u16,
    pub max_length: u16,
    pub placeholder: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ModalDefinition {
    pub custom_id: String,
    pub title: String,
    pub fields: Vec<ModalFieldDefinition>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ModalSubmitInteraction {
    pub id: u64,
    pub token: String,
    pub guild_id: u64,
    pub channel_id: Option<u64>,
    pub actor_id: u64,
    pub actor_role_ids: Vec<u64>,
    pub custom_id: String,
    pub values: BTreeMap<String, String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ComponentInteraction {
    pub id: u64,
    pub token: String,
    pub guild_id: u64,
    pub channel_id: Option<u64>,
    pub actor_id: u64,
    pub actor_role_ids: Vec<u64>,
    pub custom_id: String,
    pub values: Vec<String>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MessageButtonStyle {
    Primary,
    Secondary,
    Success,
    Danger,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MessageButton {
    pub custom_id: String,
    pub label: String,
    pub style: MessageButtonStyle,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RolePanelOption {
    pub id: String,
    pub label: String,
    pub description: String,
    pub emoji: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RolePanelComponents {
    pub custom_id: String,
    pub kind: String,
    pub exclusive: bool,
    pub options: Vec<RolePanelOption>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct RichEmbedField {
    pub name: String,
    pub value: String,
    pub inline: bool,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct RichEmbed {
    pub content: String,
    pub title: String,
    pub description: String,
    pub url: String,
    pub color: u32,
    pub author_name: String,
    pub author_url: String,
    pub author_icon_url: String,
    pub footer_text: String,
    pub footer_icon_url: String,
    pub thumbnail_url: String,
    pub image_url: String,
    pub timestamp: bool,
    pub fields: Vec<RichEmbedField>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct GuildChannel {
    pub id: String,
    pub name: String,
    pub kind: String,
    pub position: i32,
    pub parent_id: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct GuildRole {
    pub id: String,
    pub name: String,
    pub position: i64,
    pub managed: bool,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct MessageReactionSummary {
    pub author_id: u64,
    pub count: u64,
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

fn optional_text(value: &str) -> Option<String> {
    (!value.trim().is_empty()).then(|| value.trim().to_owned())
}

fn discord_embed(value: &RichEmbed) -> Result<Embed, DiscordError> {
    let timestamp = if value.timestamp {
        let seconds = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(request_error)?
            .as_secs();
        Some(
            Timestamp::from_secs(i64::try_from(seconds).unwrap_or(i64::MAX))
                .map_err(request_error)?,
        )
    } else {
        None
    };
    Ok(Embed {
        author: optional_text(&value.author_name).map(|name| EmbedAuthor {
            name,
            url: optional_text(&value.author_url),
            icon_url: optional_text(&value.author_icon_url),
            proxy_icon_url: None,
        }),
        color: Some(value.color),
        description: optional_text(&value.description),
        fields: value
            .fields
            .iter()
            .map(|field| TwilightEmbedField {
                name: field.name.clone(),
                value: field.value.clone(),
                inline: field.inline,
            })
            .collect(),
        footer: optional_text(&value.footer_text).map(|text| EmbedFooter {
            text,
            icon_url: optional_text(&value.footer_icon_url),
            proxy_icon_url: None,
        }),
        image: optional_text(&value.image_url).map(|url| EmbedImage {
            url,
            height: None,
            width: None,
            proxy_url: None,
        }),
        kind: "rich".to_owned(),
        provider: None,
        thumbnail: optional_text(&value.thumbnail_url).map(|url| EmbedThumbnail {
            url,
            height: None,
            width: None,
            proxy_url: None,
        }),
        timestamp,
        title: optional_text(&value.title),
        url: optional_text(&value.url),
        video: None,
    })
}

#[async_trait]
pub trait DiscordApi: Send + Sync {
    async fn register_commands(&self, commands: &[CommandDefinition]) -> Result<(), DiscordError>;

    async fn respond_to_command(
        &self,
        interaction: &CommandInteraction,
        response: CommandResponse,
    ) -> Result<(), DiscordError>;

    async fn open_modal(
        &self,
        interaction: &CommandInteraction,
        modal: ModalDefinition,
    ) -> Result<(), DiscordError>;

    async fn open_component_modal(
        &self,
        interaction: &ComponentInteraction,
        modal: ModalDefinition,
    ) -> Result<(), DiscordError>;

    async fn respond_to_modal(
        &self,
        interaction: &ModalSubmitInteraction,
        response: CommandResponse,
    ) -> Result<(), DiscordError>;

    async fn respond_to_component(
        &self,
        interaction: &ComponentInteraction,
        response: CommandResponse,
    ) -> Result<(), DiscordError>;

    async fn send_message(&self, channel_id: u64, content: &str) -> Result<(), DiscordError>;

    async fn send_message_with_buttons(
        &self,
        channel_id: u64,
        content: &str,
        buttons: &[MessageButton],
    ) -> Result<u64, DiscordError>;
    async fn edit_message_with_buttons(
        &self,
        channel_id: u64,
        message_id: u64,
        content: &str,
        buttons: &[MessageButton],
    ) -> Result<(), DiscordError>;
    async fn send_role_panel(
        &self,
        channel_id: u64,
        content: &str,
        components: &RolePanelComponents,
    ) -> Result<u64, DiscordError>;
    async fn edit_role_panel(
        &self,
        channel_id: u64,
        message_id: u64,
        content: &str,
        components: &RolePanelComponents,
    ) -> Result<(), DiscordError>;

    async fn send_rich_embed(
        &self,
        channel_id: u64,
        embed: &RichEmbed,
    ) -> Result<u64, DiscordError>;

    async fn edit_rich_embed(
        &self,
        channel_id: u64,
        message_id: u64,
        embed: &RichEmbed,
    ) -> Result<(), DiscordError>;

    async fn send_direct_message(&self, user_id: u64, content: &str) -> Result<(), DiscordError>;

    async fn add_member_role(
        &self,
        guild_id: u64,
        user_id: u64,
        role_id: u64,
    ) -> Result<(), DiscordError>;

    async fn remove_member_role(
        &self,
        guild_id: u64,
        user_id: u64,
        role_id: u64,
        reason: &str,
    ) -> Result<(), DiscordError>;

    async fn timeout_member(
        &self,
        guild_id: u64,
        user_id: u64,
        duration_seconds: u64,
        reason: &str,
    ) -> Result<(), DiscordError>;

    async fn kick_member(
        &self,
        guild_id: u64,
        user_id: u64,
        reason: &str,
    ) -> Result<(), DiscordError>;

    async fn ban_member(
        &self,
        guild_id: u64,
        user_id: u64,
        delete_message_seconds: u32,
        reason: &str,
    ) -> Result<(), DiscordError>;

    async fn unban_member(
        &self,
        guild_id: u64,
        user_id: u64,
        reason: &str,
    ) -> Result<(), DiscordError>;

    async fn delete_message(
        &self,
        channel_id: u64,
        message_id: u64,
        reason: &str,
    ) -> Result<(), DiscordError>;
    async fn recent_message_ids(
        &self,
        _channel_id: u64,
        _limit: u16,
    ) -> Result<Vec<u64>, DiscordError> {
        Err(DiscordError::Request("message history unavailable".into()))
    }

    async fn guild_channels(&self, guild_id: u64) -> Result<Vec<GuildChannel>, DiscordError>;

    async fn guild_roles(&self, guild_id: u64) -> Result<Vec<GuildRole>, DiscordError>;

    async fn create_ticket_channel(
        &self,
        guild_id: u64,
        category_id: u64,
        opener_id: u64,
        support_role_ids: &[u64],
        name: &str,
        reason: &str,
    ) -> Result<u64, DiscordError>;

    async fn delete_channel(&self, channel_id: u64, reason: &str) -> Result<(), DiscordError>;
    async fn channel_transcript(&self, _channel_id: u64) -> Result<String, DiscordError> {
        Err(DiscordError::Request(
            "transcript capture unavailable".into(),
        ))
    }
    async fn set_ticket_member_writable(
        &self,
        _channel_id: u64,
        _member_id: u64,
        _writable: bool,
        _reason: &str,
    ) -> Result<(), DiscordError> {
        Err(DiscordError::Request(
            "ticket permission updates unavailable".into(),
        ))
    }

    async fn message_reaction_summary(
        &self,
        channel_id: u64,
        message_id: u64,
        emoji: &str,
    ) -> Result<MessageReactionSummary, DiscordError>;
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
        let commands = commands
            .iter()
            .map(|command| {
                let mut builder =
                    CommandBuilder::new(command.name, command.description, CommandType::ChatInput);
                if command.guild_only {
                    builder = builder.contexts([InteractionContextType::Guild]);
                }
                if let Some(permissions) = command.default_permissions {
                    builder = builder
                        .default_member_permissions(Permissions::from_bits_truncate(permissions));
                }
                for option in command.options {
                    builder = builder.option(to_twilight_option(option));
                }
                builder.build()
            })
            .collect::<Vec<_>>();
        client
            .set_global_commands(&commands)
            .await
            .map_err(request_error)?;
        Ok(())
    }

    async fn send_role_panel(
        &self,
        channel_id: u64,
        content: &str,
        definition: &RolePanelComponents,
    ) -> Result<u64, DiscordError> {
        let components = role_panel_components(definition);
        let message = self
            .http
            .create_message(Id::<ChannelMarker>::new(channel_id))
            .content(content)
            .components(&components)
            .await
            .map_err(request_error)?
            .model()
            .await
            .map_err(request_error)?;
        Ok(message.id.get())
    }

    async fn edit_role_panel(
        &self,
        channel_id: u64,
        message_id: u64,
        content: &str,
        definition: &RolePanelComponents,
    ) -> Result<(), DiscordError> {
        let components = role_panel_components(definition);
        self.http
            .update_message(Id::new(channel_id), Id::new(message_id))
            .content(Some(content))
            .components(Some(&components))
            .await
            .map_err(request_error)?;
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

    #[allow(deprecated)]
    async fn open_modal(
        &self,
        interaction: &CommandInteraction,
        modal: ModalDefinition,
    ) -> Result<(), DiscordError> {
        let components = modal.fields.into_iter().map(|field| {
            Component::Label(Label {
                id: None,
                label: field.label,
                description: None,
                component: Box::new(Component::TextInput(TextInput {
                    id: None,
                    custom_id: field.custom_id,
                    label: None,
                    max_length: Some(field.max_length),
                    min_length: Some(field.min_length),
                    placeholder: (!field.placeholder.is_empty()).then_some(field.placeholder),
                    required: Some(field.required),
                    style: match field.style {
                        ModalFieldStyle::Short => TextInputStyle::Short,
                        ModalFieldStyle::Paragraph => TextInputStyle::Paragraph,
                    },
                    value: None,
                })),
            })
        });
        let response = TwilightInteractionResponse {
            kind: InteractionResponseType::Modal,
            data: Some(
                InteractionResponseDataBuilder::new()
                    .custom_id(modal.custom_id)
                    .title(modal.title)
                    .components(components)
                    .build(),
            ),
        };
        self.http
            .interaction(self.application_id)
            .create_response(Id::new(interaction.id), &interaction.token, &response)
            .await
            .map_err(request_error)?;
        Ok(())
    }

    #[allow(deprecated)]
    async fn open_component_modal(
        &self,
        interaction: &ComponentInteraction,
        modal: ModalDefinition,
    ) -> Result<(), DiscordError> {
        let components = modal.fields.into_iter().map(|field| {
            Component::Label(Label {
                id: None,
                label: field.label,
                description: None,
                component: Box::new(Component::TextInput(TextInput {
                    id: None,
                    custom_id: field.custom_id,
                    label: None,
                    max_length: Some(field.max_length),
                    min_length: Some(field.min_length),
                    placeholder: (!field.placeholder.is_empty()).then_some(field.placeholder),
                    required: Some(field.required),
                    style: match field.style {
                        ModalFieldStyle::Short => TextInputStyle::Short,
                        ModalFieldStyle::Paragraph => TextInputStyle::Paragraph,
                    },
                    value: None,
                })),
            })
        });
        let response = TwilightInteractionResponse {
            kind: InteractionResponseType::Modal,
            data: Some(
                InteractionResponseDataBuilder::new()
                    .custom_id(modal.custom_id)
                    .title(modal.title)
                    .components(components)
                    .build(),
            ),
        };
        self.http
            .interaction(self.application_id)
            .create_response(Id::new(interaction.id), &interaction.token, &response)
            .await
            .map_err(request_error)?;
        Ok(())
    }

    async fn respond_to_modal(
        &self,
        interaction: &ModalSubmitInteraction,
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

    async fn respond_to_component(
        &self,
        interaction: &ComponentInteraction,
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

    async fn send_message_with_buttons(
        &self,
        channel_id: u64,
        content: &str,
        buttons: &[MessageButton],
    ) -> Result<u64, DiscordError> {
        let components = vec![Component::ActionRow(ActionRow {
            id: None,
            components: buttons
                .iter()
                .map(|button| {
                    Component::Button(Button {
                        id: None,
                        custom_id: Some(button.custom_id.clone()),
                        disabled: false,
                        emoji: None,
                        label: Some(button.label.clone()),
                        style: match button.style {
                            MessageButtonStyle::Primary => ButtonStyle::Primary,
                            MessageButtonStyle::Secondary => ButtonStyle::Secondary,
                            MessageButtonStyle::Success => ButtonStyle::Success,
                            MessageButtonStyle::Danger => ButtonStyle::Danger,
                        },
                        url: None,
                        sku_id: None,
                    })
                })
                .collect(),
        })];
        let message = self
            .http
            .create_message(Id::<ChannelMarker>::new(channel_id))
            .content(content)
            .components(&components)
            .await
            .map_err(request_error)?
            .model()
            .await
            .map_err(request_error)?;
        Ok(message.id.get())
    }

    async fn edit_message_with_buttons(
        &self,
        channel_id: u64,
        message_id: u64,
        content: &str,
        buttons: &[MessageButton],
    ) -> Result<(), DiscordError> {
        let components = vec![Component::ActionRow(ActionRow {
            id: None,
            components: buttons
                .iter()
                .map(|button| {
                    Component::Button(Button {
                        id: None,
                        custom_id: Some(button.custom_id.clone()),
                        disabled: false,
                        emoji: None,
                        label: Some(button.label.clone()),
                        style: match button.style {
                            MessageButtonStyle::Primary => ButtonStyle::Primary,
                            MessageButtonStyle::Secondary => ButtonStyle::Secondary,
                            MessageButtonStyle::Success => ButtonStyle::Success,
                            MessageButtonStyle::Danger => ButtonStyle::Danger,
                        },
                        url: None,
                        sku_id: None,
                    })
                })
                .collect(),
        })];
        self.http
            .update_message(Id::new(channel_id), Id::new(message_id))
            .content(Some(content))
            .components(Some(&components))
            .await
            .map_err(request_error)?;
        Ok(())
    }

    async fn send_rich_embed(
        &self,
        channel_id: u64,
        embed: &RichEmbed,
    ) -> Result<u64, DiscordError> {
        let discord_embed = discord_embed(embed)?;
        let embeds = [discord_embed];
        let mut request = self
            .http
            .create_message(Id::<ChannelMarker>::new(channel_id))
            .embeds(&embeds);
        if !embed.content.trim().is_empty() {
            request = request.content(&embed.content);
        }
        let message = request
            .await
            .map_err(request_error)?
            .model()
            .await
            .map_err(request_error)?;
        Ok(message.id.get())
    }

    async fn edit_rich_embed(
        &self,
        channel_id: u64,
        message_id: u64,
        embed: &RichEmbed,
    ) -> Result<(), DiscordError> {
        let discord_embed = discord_embed(embed)?;
        let embeds = [discord_embed];
        self.http
            .update_message(
                Id::<ChannelMarker>::new(channel_id),
                Id::<MessageMarker>::new(message_id),
            )
            .content(Some(&embed.content))
            .embeds(Some(&embeds))
            .await
            .map_err(request_error)?;
        Ok(())
    }

    async fn send_direct_message(&self, user_id: u64, content: &str) -> Result<(), DiscordError> {
        let channel = self
            .http
            .create_private_channel(Id::<UserMarker>::new(user_id))
            .await
            .map_err(request_error)?
            .model()
            .await
            .map_err(request_error)?;
        self.http
            .create_message(channel.id)
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

    async fn remove_member_role(
        &self,
        guild_id: u64,
        user_id: u64,
        role_id: u64,
        reason: &str,
    ) -> Result<(), DiscordError> {
        self.http
            .remove_guild_member_role(Id::new(guild_id), Id::new(user_id), Id::new(role_id))
            .reason(reason)
            .await
            .map_err(request_error)?;
        Ok(())
    }

    async fn timeout_member(
        &self,
        guild_id: u64,
        user_id: u64,
        duration_seconds: u64,
        reason: &str,
    ) -> Result<(), DiscordError> {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(request_error)?
            .as_secs();
        let expires = now
            .checked_add(duration_seconds)
            .context("timeout duration overflow")?;
        let expires = i64::try_from(expires)
            .ok()
            .and_then(|seconds| Timestamp::from_secs(seconds).ok())
            .context("timeout expiry is invalid")?;
        self.http
            .update_guild_member(Id::new(guild_id), Id::new(user_id))
            .communication_disabled_until(Some(expires))
            .reason(reason)
            .await
            .map_err(request_error)?;
        Ok(())
    }

    async fn kick_member(
        &self,
        guild_id: u64,
        user_id: u64,
        reason: &str,
    ) -> Result<(), DiscordError> {
        self.http
            .remove_guild_member(Id::new(guild_id), Id::new(user_id))
            .reason(reason)
            .await
            .map_err(request_error)?;
        Ok(())
    }

    async fn ban_member(
        &self,
        guild_id: u64,
        user_id: u64,
        delete_message_seconds: u32,
        reason: &str,
    ) -> Result<(), DiscordError> {
        self.http
            .create_ban(Id::new(guild_id), Id::new(user_id))
            .delete_message_seconds(delete_message_seconds)
            .reason(reason)
            .await
            .map_err(request_error)?;
        Ok(())
    }

    async fn unban_member(
        &self,
        guild_id: u64,
        user_id: u64,
        reason: &str,
    ) -> Result<(), DiscordError> {
        self.http
            .delete_ban(Id::new(guild_id), Id::new(user_id))
            .reason(reason)
            .await
            .map_err(request_error)?;
        Ok(())
    }

    async fn delete_message(
        &self,
        channel_id: u64,
        message_id: u64,
        reason: &str,
    ) -> Result<(), DiscordError> {
        self.http
            .delete_message(
                Id::<ChannelMarker>::new(channel_id),
                Id::<MessageMarker>::new(message_id),
            )
            .reason(reason)
            .await
            .map_err(request_error)?;
        Ok(())
    }

    async fn recent_message_ids(
        &self,
        channel_id: u64,
        limit: u16,
    ) -> Result<Vec<u64>, DiscordError> {
        let messages = self
            .http
            .channel_messages(Id::<ChannelMarker>::new(channel_id))
            .limit(limit.clamp(1, 100))
            .await
            .map_err(request_error)?
            .models()
            .await
            .map_err(request_error)?;
        Ok(messages
            .into_iter()
            .map(|message| message.id.get())
            .collect())
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

    async fn guild_roles(&self, guild_id: u64) -> Result<Vec<GuildRole>, DiscordError> {
        let roles = self
            .http
            .roles(Id::<GuildMarker>::new(guild_id))
            .await
            .map_err(request_error)?
            .models()
            .await
            .map_err(request_error)?;
        Ok(roles
            .into_iter()
            .map(|role| GuildRole {
                id: role.id.to_string(),
                name: role.name,
                position: role.position,
                managed: role.managed,
            })
            .collect())
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
        let access = Permissions::VIEW_CHANNEL
            | Permissions::SEND_MESSAGES
            | Permissions::READ_MESSAGE_HISTORY;
        let mut overwrites = vec![
            PermissionOverwrite {
                allow: Permissions::empty(),
                deny: Permissions::VIEW_CHANNEL,
                id: Id::new(guild_id),
                kind: PermissionOverwriteType::Role,
            },
            PermissionOverwrite {
                allow: access,
                deny: Permissions::empty(),
                id: Id::new(opener_id),
                kind: PermissionOverwriteType::Member,
            },
        ];
        overwrites.extend(support_role_ids.iter().map(|role_id| PermissionOverwrite {
            allow: access,
            deny: Permissions::empty(),
            id: Id::new(*role_id),
            kind: PermissionOverwriteType::Role,
        }));
        let channel = self
            .http
            .create_guild_channel(Id::new(guild_id), name)
            .parent_id(Id::new(category_id))
            .permission_overwrites(&overwrites)
            .reason(reason)
            .await
            .map_err(request_error)?
            .model()
            .await
            .map_err(request_error)?;
        Ok(channel.id.get())
    }

    async fn delete_channel(&self, channel_id: u64, reason: &str) -> Result<(), DiscordError> {
        self.http
            .delete_channel(Id::new(channel_id))
            .reason(reason)
            .await
            .map_err(request_error)?;
        Ok(())
    }

    async fn channel_transcript(&self, channel_id: u64) -> Result<String, DiscordError> {
        let mut messages = Vec::new();
        let mut before = None;
        loop {
            let request = self
                .http
                .channel_messages(Id::<ChannelMarker>::new(channel_id));
            let page = match before {
                Some(id) => {
                    request
                        .before(Id::<MessageMarker>::new(id))
                        .limit(100)
                        .await
                }
                None => request.limit(100).await,
            }
            .map_err(request_error)?
            .models()
            .await
            .map_err(request_error)?;
            let count = page.len();
            before = page.last().map(|message| message.id.get());
            messages.extend(page);
            if count < 100 || messages.len() >= 10_000 {
                break;
            }
        }
        messages.reverse();
        Ok(messages
            .into_iter()
            .map(|message| {
                let content = if message.content.is_empty() {
                    "[attachment or embed]".to_owned()
                } else {
                    message.content
                };
                format!(
                    "[{}] {} ({}): {}",
                    message.timestamp.as_secs(),
                    message.author.name,
                    message.author.id,
                    content.replace('\n', " ")
                )
            })
            .collect::<Vec<_>>()
            .join("\n"))
    }

    async fn set_ticket_member_writable(
        &self,
        channel_id: u64,
        member_id: u64,
        writable: bool,
        reason: &str,
    ) -> Result<(), DiscordError> {
        let read = Permissions::VIEW_CHANNEL | Permissions::READ_MESSAGE_HISTORY;
        let overwrite = HttpPermissionOverwrite {
            allow: Some(if writable {
                read | Permissions::SEND_MESSAGES
            } else {
                read
            }),
            deny: Some(if writable {
                Permissions::empty()
            } else {
                Permissions::SEND_MESSAGES
            }),
            id: Id::<UserMarker>::new(member_id).cast(),
            kind: HttpPermissionOverwriteType::Member,
        };
        self.http
            .update_channel_permission(Id::new(channel_id), &overwrite)
            .reason(reason)
            .await
            .map_err(request_error)?;
        Ok(())
    }

    async fn message_reaction_summary(
        &self,
        channel_id: u64,
        message_id: u64,
        emoji: &str,
    ) -> Result<MessageReactionSummary, DiscordError> {
        let message = self
            .http
            .message(Id::new(channel_id), Id::new(message_id))
            .await
            .map_err(request_error)?
            .model()
            .await
            .map_err(request_error)?;
        let count = message
            .reactions
            .iter()
            .find(|reaction| emoji_key(&reaction.emoji) == emoji)
            .map(|reaction| reaction.count)
            .unwrap_or_default();
        Ok(MessageReactionSummary {
            author_id: message.author.id.get(),
            count,
        })
    }
}

fn role_panel_components(definition: &RolePanelComponents) -> Vec<Component> {
    if definition.kind == "select" {
        return vec![Component::ActionRow(ActionRow {
            id: None,
            components: vec![Component::SelectMenu(SelectMenu {
                id: None,
                channel_types: None,
                custom_id: definition.custom_id.clone(),
                default_values: None,
                disabled: false,
                kind: SelectMenuType::Text,
                max_values: Some(if definition.exclusive {
                    1
                } else {
                    u8::try_from(definition.options.len()).unwrap_or(25)
                }),
                min_values: Some(0),
                options: Some(
                    definition
                        .options
                        .iter()
                        .map(|option| SelectMenuOption {
                            default: false,
                            description: (!option.description.trim().is_empty())
                                .then(|| option.description.clone()),
                            emoji: component_emoji(&option.emoji),
                            label: option.label.clone(),
                            value: option.id.clone(),
                        })
                        .collect(),
                ),
                placeholder: Some("Choose your roles".into()),
                required: None,
            })],
        })];
    }
    definition
        .options
        .chunks(5)
        .map(|options| {
            Component::ActionRow(ActionRow {
                id: None,
                components: options
                    .iter()
                    .map(|option| {
                        Component::Button(Button {
                            id: None,
                            custom_id: Some(format!("{}:{}", definition.custom_id, option.id)),
                            disabled: false,
                            emoji: component_emoji(&option.emoji),
                            label: Some(option.label.clone()),
                            style: ButtonStyle::Secondary,
                            url: None,
                            sku_id: None,
                        })
                    })
                    .collect(),
            })
        })
        .collect()
}

fn component_emoji(value: &str) -> Option<twilight_model::channel::message::EmojiReactionType> {
    let value = value.trim();
    if value.is_empty() {
        return None;
    }
    if let Ok(id) = value.parse::<u64>() {
        return Some(
            twilight_model::channel::message::EmojiReactionType::Custom {
                animated: false,
                id: Id::<EmojiMarker>::new(id),
                name: None,
            },
        );
    }
    Some(
        twilight_model::channel::message::EmojiReactionType::Unicode {
            name: value.to_owned(),
        },
    )
}

fn emoji_key(emoji: &twilight_model::channel::message::EmojiReactionType) -> String {
    match emoji {
        twilight_model::channel::message::EmojiReactionType::Unicode { name } => name.clone(),
        twilight_model::channel::message::EmojiReactionType::Custom { id, .. } => id.to_string(),
    }
}

fn to_twilight_option(option: &CommandOptionDefinition) -> CommandOption {
    CommandOption {
        autocomplete: None,
        channel_types: None,
        choices: None,
        description: option.description.to_owned(),
        description_localizations: None,
        kind: match option.kind {
            CommandOptionKind::String => CommandOptionType::String,
            CommandOptionKind::Integer => CommandOptionType::Integer,
            CommandOptionKind::Boolean => CommandOptionType::Boolean,
            CommandOptionKind::User => CommandOptionType::User,
            CommandOptionKind::Channel => CommandOptionType::Channel,
            CommandOptionKind::Role => CommandOptionType::Role,
        },
        max_length: option.max_length,
        max_value: option.max_value.map(TwilightCommandOptionValue::Integer),
        min_length: option.min_length,
        min_value: option.min_value.map(TwilightCommandOptionValue::Integer),
        name: option.name.to_owned(),
        name_localizations: None,
        options: None,
        required: Some(option.required),
    }
}

trait OptionContext<T> {
    fn context(self, message: &str) -> Result<T, DiscordError>;
}

impl<T> OptionContext<T> for Option<T> {
    fn context(self, message: &str) -> Result<T, DiscordError> {
        self.ok_or_else(|| DiscordError::Request(message.to_owned()))
    }
}

fn request_error(error: impl std::fmt::Display) -> DiscordError {
    DiscordError::Request(error.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn options(count: usize) -> Vec<RolePanelOption> {
        (0..count)
            .map(|index| RolePanelOption {
                id: format!("role-{index}"),
                label: format!("Role {index}"),
                description: "Choose this role".into(),
                emoji: "✨".into(),
            })
            .collect()
    }

    #[test]
    fn role_panel_buttons_are_split_into_discord_safe_rows() {
        let components = role_panel_components(&RolePanelComponents {
            custom_id: "orivane:rr:colors".into(),
            kind: "buttons".into(),
            exclusive: false,
            options: options(7),
        });
        assert_eq!(components.len(), 2);
        assert!(matches!(&components[0], Component::ActionRow(row) if row.components.len() == 5));
        assert!(matches!(&components[1], Component::ActionRow(row) if row.components.len() == 2));
    }

    #[test]
    fn exclusive_select_panel_allows_one_value() {
        let components = role_panel_components(&RolePanelComponents {
            custom_id: "orivane:rr-select:colors".into(),
            kind: "select".into(),
            exclusive: true,
            options: options(2),
        });
        assert!(
            matches!(&components[0], Component::ActionRow(row) if matches!(&row.components[0], Component::SelectMenu(menu) if menu.max_values == Some(1) && menu.options.as_ref().is_some_and(|options| options.len() == 2)))
        );
    }
}

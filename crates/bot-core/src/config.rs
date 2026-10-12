use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct GuildConfig {
    pub guild_id: String,
    #[serde(default)]
    pub modules: ModulesConfig,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize, PartialEq, Eq)]
pub struct ModulesConfig {
    #[serde(default)]
    pub automod: AutomodConfig,
    #[serde(default)]
    pub leveling: LevelingConfig,
    #[serde(default)]
    pub welcome: WelcomeConfig,
    #[serde(default)]
    pub logging: LoggingConfig,
    #[serde(default)]
    pub utility: UtilityConfig,
    #[serde(default)]
    pub moderation: ModerationConfig,
    #[serde(default)]
    pub autorole: AutoroleConfig,
    #[serde(default)]
    pub starboard: StarboardConfig,
    #[serde(default)]
    pub tickets: TicketsConfig,
    #[serde(default)]
    pub custom_commands: CustomCommandsConfig,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct AutomodConfig {
    #[serde(default)]
    pub enabled: bool,
    #[serde(default)]
    pub blocked_words: Vec<String>,
    #[serde(default = "default_max_mentions")]
    pub max_mentions: u8,
    #[serde(default)]
    pub block_invites: bool,
    #[serde(default)]
    pub block_links: bool,
    #[serde(default = "default_max_caps_percentage")]
    pub max_caps_percentage: u8,
    #[serde(default = "default_max_lines")]
    pub max_lines: u16,
    #[serde(default = "default_automod_action")]
    pub action: String,
    #[serde(default = "default_automod_timeout")]
    pub timeout_seconds: u32,
    #[serde(default)]
    pub ignored_channel_ids: Vec<String>,
    #[serde(default)]
    pub ignored_role_ids: Vec<String>,
    pub log_channel_id: Option<String>,
    #[serde(default = "default_automod_response")]
    pub response: String,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct LevelingConfig {
    #[serde(default)]
    pub enabled: bool,
    #[serde(default = "default_xp_min")]
    pub xp_per_message_min: u16,
    #[serde(default = "default_xp_max")]
    pub xp_per_message_max: u16,
    #[serde(default = "default_xp_cooldown")]
    pub cooldown_seconds: u16,
    #[serde(default)]
    pub ignored_channel_ids: Vec<String>,
    #[serde(default)]
    pub ignored_role_ids: Vec<String>,
    #[serde(default)]
    pub announce_level_up: bool,
    pub announcement_channel_id: Option<String>,
    #[serde(default = "default_level_up_message")]
    pub announcement_message: String,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct WelcomeConfig {
    #[serde(default)]
    pub enabled: bool,
    pub channel_id: Option<String>,
    #[serde(default = "default_welcome_message")]
    pub message: String,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize, PartialEq, Eq)]
pub struct LoggingConfig {
    #[serde(default)]
    pub enabled: bool,
    pub channel_id: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct UtilityConfig {
    #[serde(default = "default_true")]
    pub enabled: bool,
    #[serde(default)]
    pub ephemeral_responses: bool,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct ModerationConfig {
    #[serde(default)]
    pub enabled: bool,
    #[serde(default = "default_true")]
    pub dm_on_action: bool,
    #[serde(default = "default_moderation_reason")]
    pub default_reason: String,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize, PartialEq, Eq)]
pub struct AutoroleConfig {
    #[serde(default)]
    pub enabled: bool,
    #[serde(default)]
    pub role_ids: Vec<String>,
    #[serde(default)]
    pub delay_seconds: u16,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct StarboardConfig {
    #[serde(default)]
    pub enabled: bool,
    pub channel_id: Option<String>,
    #[serde(default = "default_starboard_emoji")]
    pub emoji: String,
    #[serde(default = "default_starboard_threshold")]
    pub threshold: u16,
    #[serde(default)]
    pub allow_self_stars: bool,
    #[serde(default)]
    pub ignored_channel_ids: Vec<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct TicketsConfig {
    #[serde(default)]
    pub enabled: bool,
    pub category_id: Option<String>,
    #[serde(default)]
    pub support_role_ids: Vec<String>,
    pub transcript_channel_id: Option<String>,
    #[serde(default = "default_ticket_welcome")]
    pub welcome_message: String,
    #[serde(default = "default_ticket_name")]
    pub channel_name: String,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct CustomCommandsConfig {
    #[serde(default)]
    pub enabled: bool,
    #[serde(default = "default_custom_command_limit")]
    pub max_commands: u16,
    #[serde(default)]
    pub commands: Vec<CustomCommandConfig>,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct CustomCommandConfig {
    #[serde(default = "default_true")]
    pub enabled: bool,
    pub name: String,
    #[serde(default)]
    pub description: String,
    pub response: String,
    #[serde(default)]
    pub ephemeral: bool,
    #[serde(default)]
    pub allowed_channel_ids: Vec<String>,
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum ConfigValidationError {
    #[error("guild_id must be a non-zero Discord snowflake")]
    InvalidGuildId,
    #[error("blocked_words may contain at most 100 entries of 64 characters each")]
    InvalidBlockedWords,
    #[error("automod filters, action, timeout, or ignored targets are invalid")]
    InvalidAutomod,
    #[error("leveling XP minimum must be positive and no greater than its maximum")]
    InvalidXpRange,
    #[error("leveling ignored targets or announcement settings are invalid")]
    InvalidLeveling,
    #[error("a welcome channel is required when the welcome module is enabled")]
    MissingWelcomeChannel,
    #[error("a logging channel is required when the logging module is enabled")]
    MissingLoggingChannel,
    #[error("autorole role IDs must be valid Discord snowflakes and contain at most 10 roles")]
    InvalidAutoroles,
    #[error("a starboard channel is required when the starboard module is enabled")]
    MissingStarboardChannel,
    #[error("starboard threshold must be between 1 and 100")]
    InvalidStarboardThreshold,
    #[error(
        "a ticket category and at least one support role are required when tickets are enabled"
    )]
    InvalidTickets,
    #[error("custom command limit must be between 1 and 100")]
    InvalidCustomCommandLimit,
    #[error("custom command names, responses, channel restrictions, or count are invalid")]
    InvalidCustomCommands,
}

impl GuildConfig {
    pub fn for_guild(guild_id: impl Into<String>) -> Self {
        Self {
            guild_id: guild_id.into(),
            modules: ModulesConfig::default(),
        }
    }

    pub fn validate(&self) -> Result<(), ConfigValidationError> {
        if !self.guild_id.parse::<u64>().is_ok_and(|id| id > 0) {
            return Err(ConfigValidationError::InvalidGuildId);
        }

        if self.modules.automod.blocked_words.len() > 100
            || self
                .modules
                .automod
                .blocked_words
                .iter()
                .any(|word| word.chars().count() > 64)
        {
            return Err(ConfigValidationError::InvalidBlockedWords);
        }
        let automod = &self.modules.automod;
        if !(20..=100).contains(&automod.max_caps_percentage)
            || !(1..=100).contains(&automod.max_lines)
            || !matches!(
                automod.action.as_str(),
                "delete" | "warn" | "timeout" | "ban"
            )
            || (automod.action == "timeout" && !(60..=2_419_200).contains(&automod.timeout_seconds))
            || invalid_snowflake_list(&automod.ignored_channel_ids, 100)
            || invalid_snowflake_list(&automod.ignored_role_ids, 100)
            || automod
                .log_channel_id
                .as_deref()
                .is_some_and(|id| !valid_snowflake(id))
        {
            return Err(ConfigValidationError::InvalidAutomod);
        }

        let leveling = &self.modules.leveling;
        if leveling.xp_per_message_min == 0
            || leveling.xp_per_message_min > leveling.xp_per_message_max
        {
            return Err(ConfigValidationError::InvalidXpRange);
        }
        if invalid_snowflake_list(&leveling.ignored_channel_ids, 100)
            || invalid_snowflake_list(&leveling.ignored_role_ids, 100)
            || (leveling.announce_level_up
                && leveling
                    .announcement_channel_id
                    .as_deref()
                    .is_some_and(|id| !valid_snowflake(id)))
        {
            return Err(ConfigValidationError::InvalidLeveling);
        }

        if self.modules.welcome.enabled
            && self
                .modules
                .welcome
                .channel_id
                .as_deref()
                .is_none_or(|id| !valid_snowflake(id))
        {
            return Err(ConfigValidationError::MissingWelcomeChannel);
        }
        if self.modules.logging.enabled
            && self
                .modules
                .logging
                .channel_id
                .as_deref()
                .is_none_or(|id| !valid_snowflake(id))
        {
            return Err(ConfigValidationError::MissingLoggingChannel);
        }
        if (self.modules.autorole.enabled && self.modules.autorole.role_ids.is_empty())
            || self.modules.autorole.role_ids.len() > 10
            || self
                .modules
                .autorole
                .role_ids
                .iter()
                .any(|id| !valid_snowflake(id))
        {
            return Err(ConfigValidationError::InvalidAutoroles);
        }
        if self.modules.starboard.enabled
            && self
                .modules
                .starboard
                .channel_id
                .as_deref()
                .is_none_or(|id| !valid_snowflake(id))
        {
            return Err(ConfigValidationError::MissingStarboardChannel);
        }
        if !(1..=100).contains(&self.modules.starboard.threshold) {
            return Err(ConfigValidationError::InvalidStarboardThreshold);
        }
        if invalid_snowflake_list(&self.modules.starboard.ignored_channel_ids, 100) {
            return Err(ConfigValidationError::InvalidStarboardThreshold);
        }
        if self.modules.tickets.enabled
            && (self
                .modules
                .tickets
                .category_id
                .as_deref()
                .is_none_or(|id| !valid_snowflake(id))
                || self.modules.tickets.support_role_ids.is_empty()
                || self.modules.tickets.support_role_ids.len() > 10
                || self
                    .modules
                    .tickets
                    .support_role_ids
                    .iter()
                    .any(|id| !valid_snowflake(id))
                || self
                    .modules
                    .tickets
                    .transcript_channel_id
                    .as_deref()
                    .is_some_and(|id| !valid_snowflake(id))
                || self.modules.tickets.channel_name.trim().is_empty()
                || self.modules.tickets.channel_name.chars().count() > 100
                || self.modules.tickets.welcome_message.chars().count() > 2_000)
        {
            return Err(ConfigValidationError::InvalidTickets);
        }
        if !(1..=100).contains(&self.modules.custom_commands.max_commands) {
            return Err(ConfigValidationError::InvalidCustomCommandLimit);
        }
        let custom = &self.modules.custom_commands;
        let mut names = std::collections::HashSet::new();
        if custom.commands.len() > usize::from(custom.max_commands)
            || custom.commands.iter().any(|command| {
                let normalized = command.name.to_ascii_lowercase();
                command.name.is_empty()
                    || command.name.len() > 32
                    || !command.name.chars().all(|character| {
                        character.is_ascii_lowercase()
                            || character.is_ascii_digit()
                            || character == '-'
                            || character == '_'
                    })
                    || !names.insert(normalized)
                    || command.description.chars().count() > 100
                    || command.response.is_empty()
                    || command.response.chars().count() > 2_000
                    || invalid_snowflake_list(&command.allowed_channel_ids, 100)
            })
        {
            return Err(ConfigValidationError::InvalidCustomCommands);
        }

        Ok(())
    }
}

impl Default for UtilityConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            ephemeral_responses: false,
        }
    }
}

impl Default for ModerationConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            dm_on_action: true,
            default_reason: default_moderation_reason(),
        }
    }
}

impl Default for StarboardConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            channel_id: None,
            emoji: default_starboard_emoji(),
            threshold: default_starboard_threshold(),
            allow_self_stars: false,
            ignored_channel_ids: Vec::new(),
        }
    }
}

impl Default for CustomCommandsConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            max_commands: default_custom_command_limit(),
            commands: Vec::new(),
        }
    }
}

impl Default for TicketsConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            category_id: None,
            support_role_ids: Vec::new(),
            transcript_channel_id: None,
            welcome_message: default_ticket_welcome(),
            channel_name: default_ticket_name(),
        }
    }
}

impl Default for AutomodConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            blocked_words: Vec::new(),
            max_mentions: default_max_mentions(),
            block_invites: true,
            block_links: false,
            max_caps_percentage: default_max_caps_percentage(),
            max_lines: default_max_lines(),
            action: default_automod_action(),
            timeout_seconds: default_automod_timeout(),
            ignored_channel_ids: Vec::new(),
            ignored_role_ids: Vec::new(),
            log_channel_id: None,
            response: default_automod_response(),
        }
    }
}

impl Default for LevelingConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            xp_per_message_min: default_xp_min(),
            xp_per_message_max: default_xp_max(),
            cooldown_seconds: default_xp_cooldown(),
            ignored_channel_ids: Vec::new(),
            ignored_role_ids: Vec::new(),
            announce_level_up: true,
            announcement_channel_id: None,
            announcement_message: default_level_up_message(),
        }
    }
}

impl Default for WelcomeConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            channel_id: None,
            message: default_welcome_message(),
        }
    }
}

const fn default_max_mentions() -> u8 {
    5
}
const fn default_max_caps_percentage() -> u8 {
    80
}
const fn default_max_lines() -> u16 {
    12
}
const fn default_automod_timeout() -> u32 {
    600
}
fn default_automod_action() -> String {
    "delete".to_owned()
}
fn default_automod_response() -> String {
    "{user}, that message was removed by AutoMod ({rule}).".to_owned()
}
const fn default_xp_min() -> u16 {
    15
}
const fn default_xp_max() -> u16 {
    25
}
const fn default_xp_cooldown() -> u16 {
    60
}
fn default_level_up_message() -> String {
    "Congratulations {user}, you reached level {level}!".to_owned()
}
fn default_welcome_message() -> String {
    "Welcome {user} to {server}!".to_owned()
}
const fn default_true() -> bool {
    true
}
const fn default_starboard_threshold() -> u16 {
    3
}
const fn default_custom_command_limit() -> u16 {
    25
}
fn default_starboard_emoji() -> String {
    "⭐".to_owned()
}
fn default_moderation_reason() -> String {
    "No reason provided".to_owned()
}
fn default_ticket_welcome() -> String {
    "Welcome {user}. Please describe what you need help with. A member of {support_roles} will respond soon.".to_owned()
}
fn default_ticket_name() -> String {
    "ticket-{user_id}".to_owned()
}
fn valid_snowflake(value: &str) -> bool {
    value.parse::<u64>().is_ok_and(|id| id > 0)
}

fn invalid_snowflake_list(values: &[String], limit: usize) -> bool {
    values.len() > limit || values.iter().any(|value| !valid_snowflake(value))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_are_valid() {
        assert_eq!(GuildConfig::for_guild("123").validate(), Ok(()));
    }

    #[test]
    fn enabled_destination_modules_require_channels() {
        let mut config = GuildConfig::for_guild("123");
        config.modules.welcome.enabled = true;
        assert_eq!(
            config.validate(),
            Err(ConfigValidationError::MissingWelcomeChannel)
        );
    }

    #[test]
    fn enabled_autorole_requires_valid_roles() {
        let mut config = GuildConfig::for_guild("123");
        config.modules.autorole.enabled = true;
        assert_eq!(
            config.validate(),
            Err(ConfigValidationError::InvalidAutoroles)
        );
        config.modules.autorole.role_ids = vec!["456".to_owned()];
        assert_eq!(config.validate(), Ok(()));
    }

    #[test]
    fn older_guild_configs_receive_new_module_defaults() {
        let config: GuildConfig = serde_json::from_value(serde_json::json!({
            "guild_id": "123",
            "modules": {
                "automod": { "enabled": false, "blocked_words": [], "max_mentions": 5 },
                "leveling": { "enabled": false, "xp_per_message_min": 15, "xp_per_message_max": 25, "cooldown_seconds": 60 },
                "welcome": { "enabled": false, "channel_id": null, "message": "Welcome {user}!" },
                "logging": { "enabled": false, "channel_id": null }
            }
        })).unwrap();
        assert!(config.modules.utility.enabled);
        assert!(!config.modules.autorole.enabled);
        assert_eq!(config.modules.starboard.threshold, 3);
    }
}

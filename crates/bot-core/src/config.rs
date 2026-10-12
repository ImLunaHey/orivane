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
    #[serde(default)]
    pub reaction_roles: ReactionRolesConfig,
    #[serde(default)]
    pub autoresponder: AutoresponderConfig,
    #[serde(default)]
    pub anti_raid: AntiRaidConfig,
    #[serde(default)]
    pub scheduled_messages: ScheduledMessagesConfig,
    #[serde(default)]
    pub forms: FormsConfig,
    #[serde(default)]
    pub suggestions: SuggestionsConfig,
    #[serde(default)]
    pub giveaways: GiveawaysConfig,
    #[serde(default)]
    pub reminders: RemindersConfig,
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
    #[serde(default)]
    pub dm_enabled: bool,
    #[serde(default = "default_welcome_dm_message")]
    pub dm_message: String,
    #[serde(default)]
    pub goodbye_enabled: bool,
    pub goodbye_channel_id: Option<String>,
    #[serde(default = "default_goodbye_message")]
    pub goodbye_message: String,
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
    #[serde(default)]
    pub remove_role_ids: Vec<String>,
    #[serde(default)]
    pub remove_delay_seconds: u32,
    #[serde(default)]
    pub joinable_role_ids: Vec<String>,
    #[serde(default)]
    pub single_joinable_role: bool,
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

#[derive(Clone, Debug, Default, Deserialize, Serialize, PartialEq, Eq)]
pub struct ReactionRolesConfig {
    #[serde(default)]
    pub enabled: bool,
    #[serde(default)]
    pub bindings: Vec<ReactionRoleBinding>,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct ReactionRoleBinding {
    pub message_id: String,
    pub emoji: String,
    pub role_id: String,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize, PartialEq, Eq)]
pub struct AutoresponderConfig {
    #[serde(default)]
    pub enabled: bool,
    #[serde(default)]
    pub responses: Vec<AutoresponseConfig>,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct AutoresponseConfig {
    #[serde(default = "default_true")]
    pub enabled: bool,
    pub trigger: String,
    pub response: String,
    #[serde(default = "default_match_mode")]
    pub match_mode: String,
    #[serde(default)]
    pub case_sensitive: bool,
    #[serde(default)]
    pub allowed_channel_ids: Vec<String>,
    #[serde(default)]
    pub ignored_role_ids: Vec<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct AntiRaidConfig {
    #[serde(default)]
    pub enabled: bool,
    #[serde(default = "default_join_threshold")]
    pub join_threshold: u16,
    #[serde(default = "default_join_window")]
    pub join_window_seconds: u16,
    #[serde(default = "default_account_age")]
    pub minimum_account_age_days: u16,
    #[serde(default)]
    pub require_avatar: bool,
    #[serde(default)]
    pub punish_all_during_raid: bool,
    #[serde(default = "default_raid_action")]
    pub action: String,
    #[serde(default = "default_raid_timeout")]
    pub timeout_seconds: u32,
    pub quarantine_role_id: Option<String>,
    pub alert_channel_id: Option<String>,
}

impl Default for AntiRaidConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            join_threshold: default_join_threshold(),
            join_window_seconds: default_join_window(),
            minimum_account_age_days: default_account_age(),
            require_avatar: false,
            punish_all_during_raid: false,
            action: default_raid_action(),
            timeout_seconds: default_raid_timeout(),
            quarantine_role_id: None,
            alert_channel_id: None,
        }
    }
}

#[derive(Clone, Debug, Default, Deserialize, Serialize, PartialEq, Eq)]
pub struct ScheduledMessagesConfig {
    #[serde(default)]
    pub enabled: bool,
    #[serde(default)]
    pub schedules: Vec<ScheduledMessageConfig>,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct ScheduledMessageConfig {
    #[serde(default = "default_true")]
    pub enabled: bool,
    pub id: String,
    pub channel_id: String,
    pub message: String,
    pub starts_at_unix: u64,
    pub interval_minutes: u32,
    pub ends_at_unix: Option<u64>,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize, PartialEq, Eq)]
pub struct FormsConfig {
    #[serde(default)]
    pub enabled: bool,
    #[serde(default)]
    pub forms: Vec<FormConfig>,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct FormConfig {
    #[serde(default = "default_true")]
    pub enabled: bool,
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub description: String,
    pub destination_channel_id: String,
    #[serde(default)]
    pub allowed_role_ids: Vec<String>,
    #[serde(default)]
    pub anonymous: bool,
    #[serde(default)]
    pub questions: Vec<FormQuestionConfig>,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct FormQuestionConfig {
    pub id: String,
    pub label: String,
    #[serde(default = "default_form_question_style")]
    pub style: String,
    #[serde(default = "default_true")]
    pub required: bool,
    #[serde(default)]
    pub min_length: u16,
    #[serde(default = "default_form_max_length")]
    pub max_length: u16,
    #[serde(default)]
    pub placeholder: String,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct SuggestionsConfig {
    #[serde(default)]
    pub enabled: bool,
    pub channel_id: Option<String>,
    #[serde(default)]
    pub staff_role_ids: Vec<String>,
    #[serde(default)]
    pub allow_anonymous: bool,
    #[serde(default = "default_suggestion_cooldown")]
    pub cooldown_seconds: u32,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct GiveawaysConfig {
    #[serde(default)]
    pub enabled: bool,
    pub default_channel_id: Option<String>,
    #[serde(default)]
    pub manager_role_ids: Vec<String>,
    #[serde(default = "default_giveaway_duration")]
    pub default_duration_minutes: u32,
    #[serde(default = "default_giveaway_winners")]
    pub default_winner_count: u8,
    #[serde(default)]
    pub minimum_account_age_days: u16,
    #[serde(default)]
    pub required_role_ids: Vec<String>,
    pub bonus_role_id: Option<String>,
    #[serde(default = "default_bonus_entries")]
    pub bonus_entries: u8,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct RemindersConfig {
    #[serde(default)]
    pub enabled: bool,
    #[serde(default = "default_true")]
    pub allow_direct_messages: bool,
    #[serde(default = "default_max_active_reminders")]
    pub max_active_per_user: u8,
    #[serde(default = "default_minimum_reminder_delay")]
    pub minimum_delay_minutes: u16,
    #[serde(default = "default_maximum_reminder_days")]
    pub maximum_delay_days: u16,
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
    #[error("welcome direct-message or goodbye settings are invalid")]
    InvalidWelcomeMessages,
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
    #[error("reaction-role mappings must be unique and contain valid messages, roles, and emoji")]
    InvalidReactionRoles,
    #[error("autoresponses contain an invalid trigger, response, matching mode, or restriction")]
    InvalidAutoresponder,
    #[error("anti-raid thresholds, action, timeout, role, or alert channel are invalid")]
    InvalidAntiRaid,
    #[error("scheduled messages contain invalid IDs, channels, messages, intervals, or dates")]
    InvalidScheduledMessages,
    #[error("forms contain invalid IDs, destinations, roles, questions, or field limits")]
    InvalidForms,
    #[error("suggestions channel, staff roles, or cooldown are invalid")]
    InvalidSuggestions,
    #[error("giveaway channel, roles, duration, winner count, or bonus entries are invalid")]
    InvalidGiveaways,
    #[error("reminder limits or delay bounds are invalid")]
    InvalidReminders,
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
        if self.modules.welcome.dm_message.chars().count() > 2_000
            || self.modules.welcome.goodbye_message.chars().count() > 2_000
            || (self.modules.welcome.goodbye_enabled
                && self
                    .modules
                    .welcome
                    .goodbye_channel_id
                    .as_deref()
                    .is_none_or(|id| !valid_snowflake(id)))
        {
            return Err(ConfigValidationError::InvalidWelcomeMessages);
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
        let autorole = &self.modules.autorole;
        if (autorole.enabled
            && autorole.role_ids.is_empty()
            && autorole.remove_role_ids.is_empty()
            && autorole.joinable_role_ids.is_empty())
            || invalid_snowflake_list(&autorole.role_ids, 10)
            || invalid_snowflake_list(&autorole.remove_role_ids, 10)
            || invalid_snowflake_list(&autorole.joinable_role_ids, 25)
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

        let reaction_roles = &self.modules.reaction_roles;
        let mut mappings = std::collections::HashSet::new();
        if (reaction_roles.enabled && reaction_roles.bindings.is_empty())
            || reaction_roles.bindings.len() > 100
            || reaction_roles.bindings.iter().any(|binding| {
                !valid_snowflake(&binding.message_id)
                    || !valid_snowflake(&binding.role_id)
                    || binding.emoji.trim().is_empty()
                    || binding.emoji.chars().count() > 64
                    || !mappings.insert((binding.message_id.clone(), binding.emoji.clone()))
            })
        {
            return Err(ConfigValidationError::InvalidReactionRoles);
        }
        let autoresponder = &self.modules.autoresponder;
        if (autoresponder.enabled && autoresponder.responses.is_empty())
            || autoresponder.responses.len() > 100
            || autoresponder.responses.iter().any(|response| {
                response.trigger.trim().is_empty()
                    || response.trigger.chars().count() > 200
                    || response.response.trim().is_empty()
                    || response.response.chars().count() > 2_000
                    || !matches!(
                        response.match_mode.as_str(),
                        "exact" | "contains" | "starts_with"
                    )
                    || invalid_snowflake_list(&response.allowed_channel_ids, 100)
                    || invalid_snowflake_list(&response.ignored_role_ids, 100)
            })
        {
            return Err(ConfigValidationError::InvalidAutoresponder);
        }
        let anti_raid = &self.modules.anti_raid;
        if !(2..=500).contains(&anti_raid.join_threshold)
            || !(5..=3600).contains(&anti_raid.join_window_seconds)
            || anti_raid.minimum_account_age_days > 3650
            || !matches!(
                anti_raid.action.as_str(),
                "log" | "quarantine" | "timeout" | "kick" | "ban"
            )
            || (anti_raid.action == "timeout"
                && !(60..=2_419_200).contains(&anti_raid.timeout_seconds))
            || (anti_raid.enabled
                && anti_raid.action == "quarantine"
                && anti_raid
                    .quarantine_role_id
                    .as_deref()
                    .is_none_or(|id| !valid_snowflake(id)))
            || anti_raid
                .alert_channel_id
                .as_deref()
                .is_some_and(|id| !valid_snowflake(id))
        {
            return Err(ConfigValidationError::InvalidAntiRaid);
        }
        let scheduled = &self.modules.scheduled_messages;
        let mut schedule_ids = std::collections::HashSet::new();
        if (scheduled.enabled && scheduled.schedules.is_empty())
            || scheduled.schedules.len() > 100
            || scheduled.schedules.iter().any(|schedule| {
                schedule.id.is_empty()
                    || schedule.id.len() > 64
                    || !schedule.id.chars().all(|character| {
                        character.is_ascii_alphanumeric() || character == '-' || character == '_'
                    })
                    || !schedule_ids.insert(schedule.id.clone())
                    || !valid_snowflake(&schedule.channel_id)
                    || schedule.message.trim().is_empty()
                    || schedule.message.chars().count() > 2_000
                    || !(1..=525_600).contains(&schedule.interval_minutes)
                    || schedule.starts_at_unix == 0
                    || schedule
                        .ends_at_unix
                        .is_some_and(|end| end < schedule.starts_at_unix)
            })
        {
            return Err(ConfigValidationError::InvalidScheduledMessages);
        }

        let forms = &self.modules.forms;
        let mut form_ids = std::collections::HashSet::new();
        if (forms.enabled && forms.forms.is_empty())
            || forms.forms.len() > 25
            || forms.forms.iter().any(|form| {
                let mut question_ids = std::collections::HashSet::new();
                form.id.is_empty()
                    || form.id.len() > 64
                    || !form
                        .id
                        .chars()
                        .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
                    || !form_ids.insert(form.id.clone())
                    || form.name.trim().is_empty()
                    || form.name.chars().count() > 45
                    || form.description.chars().count() > 100
                    || !valid_snowflake(&form.destination_channel_id)
                    || invalid_snowflake_list(&form.allowed_role_ids, 25)
                    || form.questions.is_empty()
                    || form.questions.len() > 5
                    || form.questions.iter().any(|question| {
                        question.id.is_empty()
                            || question.id.len() > 64
                            || !question
                                .id
                                .chars()
                                .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
                            || !question_ids.insert(question.id.clone())
                            || question.label.trim().is_empty()
                            || question.label.chars().count() > 45
                            || !matches!(question.style.as_str(), "short" | "paragraph")
                            || question.min_length > question.max_length
                            || question.max_length == 0
                            || question.max_length > 4_000
                            || question.placeholder.chars().count() > 100
                    })
            })
        {
            return Err(ConfigValidationError::InvalidForms);
        }

        let suggestions = &self.modules.suggestions;
        if (suggestions.enabled
            && suggestions
                .channel_id
                .as_deref()
                .is_none_or(|id| !valid_snowflake(id)))
            || invalid_snowflake_list(&suggestions.staff_role_ids, 25)
            || suggestions.cooldown_seconds > 86_400
        {
            return Err(ConfigValidationError::InvalidSuggestions);
        }

        let giveaways = &self.modules.giveaways;
        if (giveaways.enabled
            && giveaways
                .default_channel_id
                .as_deref()
                .is_none_or(|id| !valid_snowflake(id)))
            || invalid_snowflake_list(&giveaways.manager_role_ids, 25)
            || invalid_snowflake_list(&giveaways.required_role_ids, 25)
            || giveaways
                .bonus_role_id
                .as_deref()
                .is_some_and(|id| !valid_snowflake(id))
            || !(1..=525_600).contains(&giveaways.default_duration_minutes)
            || !(1..=25).contains(&giveaways.default_winner_count)
            || giveaways.minimum_account_age_days > 3650
            || giveaways.bonus_entries > 25
        {
            return Err(ConfigValidationError::InvalidGiveaways);
        }

        let reminders = &self.modules.reminders;
        if !(1..=50).contains(&reminders.max_active_per_user)
            || !(1..=1_440).contains(&reminders.minimum_delay_minutes)
            || !(1..=3_650).contains(&reminders.maximum_delay_days)
            || u32::from(reminders.minimum_delay_minutes)
                > u32::from(reminders.maximum_delay_days) * 1_440
        {
            return Err(ConfigValidationError::InvalidReminders);
        }

        Ok(())
    }
}

fn default_form_question_style() -> String {
    "short".to_owned()
}
fn default_form_max_length() -> u16 {
    300
}
const fn default_suggestion_cooldown() -> u32 {
    300
}
const fn default_giveaway_duration() -> u32 {
    1_440
}
const fn default_giveaway_winners() -> u8 {
    1
}
const fn default_bonus_entries() -> u8 {
    1
}
const fn default_max_active_reminders() -> u8 {
    10
}
const fn default_minimum_reminder_delay() -> u16 {
    1
}
const fn default_maximum_reminder_days() -> u16 {
    365
}

impl Default for RemindersConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            allow_direct_messages: true,
            max_active_per_user: default_max_active_reminders(),
            minimum_delay_minutes: default_minimum_reminder_delay(),
            maximum_delay_days: default_maximum_reminder_days(),
        }
    }
}

impl Default for GiveawaysConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            default_channel_id: None,
            manager_role_ids: Vec::new(),
            default_duration_minutes: default_giveaway_duration(),
            default_winner_count: default_giveaway_winners(),
            minimum_account_age_days: 0,
            required_role_ids: Vec::new(),
            bonus_role_id: None,
            bonus_entries: default_bonus_entries(),
        }
    }
}

impl Default for SuggestionsConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            channel_id: None,
            staff_role_ids: Vec::new(),
            allow_anonymous: false,
            cooldown_seconds: default_suggestion_cooldown(),
        }
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
            dm_enabled: false,
            dm_message: default_welcome_dm_message(),
            goodbye_enabled: false,
            goodbye_channel_id: None,
            goodbye_message: default_goodbye_message(),
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
fn default_welcome_dm_message() -> String {
    "Welcome to {server}, {user}!".to_owned()
}
fn default_goodbye_message() -> String {
    "{username} has left {server}.".to_owned()
}
const fn default_true() -> bool {
    true
}

fn default_match_mode() -> String {
    "contains".to_owned()
}
const fn default_join_threshold() -> u16 {
    10
}
const fn default_join_window() -> u16 {
    60
}
const fn default_account_age() -> u16 {
    7
}
fn default_raid_action() -> String {
    "quarantine".to_owned()
}
const fn default_raid_timeout() -> u32 {
    3600
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

    #[test]
    fn forms_require_valid_unique_questions_and_a_destination() {
        let mut config = GuildConfig::for_guild("123");
        config.modules.forms.enabled = true;
        assert_eq!(config.validate(), Err(ConfigValidationError::InvalidForms));
        config.modules.forms.forms.push(FormConfig {
            enabled: true,
            id: "application".into(),
            name: "Application".into(),
            description: String::new(),
            destination_channel_id: "456".into(),
            allowed_role_ids: vec![],
            anonymous: false,
            questions: vec![FormQuestionConfig {
                id: "reason".into(),
                label: "Why are you applying?".into(),
                style: "paragraph".into(),
                required: true,
                min_length: 1,
                max_length: 1_000,
                placeholder: String::new(),
            }],
        });
        assert_eq!(config.validate(), Ok(()));
        let duplicate = config.modules.forms.forms[0].questions[0].clone();
        config.modules.forms.forms[0].questions.push(duplicate);
        assert_eq!(config.validate(), Err(ConfigValidationError::InvalidForms));
    }

    #[test]
    fn enabled_suggestions_require_a_channel_and_bounded_cooldown() {
        let mut config = GuildConfig::for_guild("123");
        config.modules.suggestions.enabled = true;
        assert_eq!(
            config.validate(),
            Err(ConfigValidationError::InvalidSuggestions)
        );
        config.modules.suggestions.channel_id = Some("456".into());
        assert_eq!(config.validate(), Ok(()));
        config.modules.suggestions.cooldown_seconds = 86_401;
        assert_eq!(
            config.validate(),
            Err(ConfigValidationError::InvalidSuggestions)
        );
    }

    #[test]
    fn enabled_giveaways_require_valid_defaults() {
        let mut config = GuildConfig::for_guild("123");
        config.modules.giveaways.enabled = true;
        assert_eq!(
            config.validate(),
            Err(ConfigValidationError::InvalidGiveaways)
        );
        config.modules.giveaways.default_channel_id = Some("456".into());
        assert_eq!(config.validate(), Ok(()));
        config.modules.giveaways.default_winner_count = 0;
        assert_eq!(
            config.validate(),
            Err(ConfigValidationError::InvalidGiveaways)
        );
    }

    #[test]
    fn reminders_require_bounded_limits_and_delays() {
        let mut config = GuildConfig::for_guild("123");
        config.modules.reminders.enabled = true;
        assert_eq!(config.validate(), Ok(()));
        config.modules.reminders.max_active_per_user = 0;
        assert_eq!(
            config.validate(),
            Err(ConfigValidationError::InvalidReminders)
        );
    }
}

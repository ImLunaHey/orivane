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
    #[serde(default)]
    pub auto_delete: AutoDeleteConfig,
    #[serde(default)]
    pub managed_embeds: ManagedEmbedsConfig,
    #[serde(default)]
    pub temporary_voice: TemporaryVoiceConfig,
    #[serde(default)]
    pub economy: EconomyConfig,
    #[serde(default)]
    pub verification: VerificationConfig,
    #[serde(default)]
    pub moderation_appeals: ModerationAppealsConfig,
    #[serde(default)]
    pub invite_tracking: InviteTrackingConfig,
    #[serde(default)]
    pub polls: PollsConfig,
    #[serde(default)]
    pub counters: CountersConfig,
    #[serde(default)]
    pub afk: AfkConfig,
    #[serde(default)]
    pub voice_text_linking: VoiceTextLinkingConfig,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct ModerationAppealsConfig {
    #[serde(default)]
    pub enabled: bool,
    pub review_channel_id: Option<String>,
    #[serde(default)]
    pub reviewer_role_ids: Vec<String>,
    #[serde(default = "default_appeal_cooldown")]
    pub cooldown_hours: u16,
    #[serde(default = "default_appeal_prompt")]
    pub prompt: String,
}
impl Default for ModerationAppealsConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            review_channel_id: None,
            reviewer_role_ids: vec![],
            cooldown_hours: 24,
            prompt: default_appeal_prompt(),
        }
    }
}
fn default_appeal_cooldown() -> u16 {
    24
}
fn default_appeal_prompt() -> String {
    "Explain why this moderation action should be reviewed.".into()
}

#[derive(Clone, Debug, Default, Deserialize, Serialize, PartialEq, Eq)]
pub struct InviteTrackingConfig {
    #[serde(default)]
    pub enabled: bool,
    pub log_channel_id: Option<String>,
    #[serde(default)]
    pub reward_role_ids: Vec<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct PollsConfig {
    #[serde(default)]
    pub enabled: bool,
    #[serde(default = "default_poll_duration")]
    pub default_duration_minutes: u32,
    #[serde(default = "default_poll_options")]
    pub maximum_options: u8,
    #[serde(default = "default_true")]
    pub allow_anonymous: bool,
    #[serde(default)]
    pub manager_role_ids: Vec<String>,
}
impl Default for PollsConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            default_duration_minutes: 1440,
            maximum_options: 10,
            allow_anonymous: true,
            manager_role_ids: vec![],
        }
    }
}
fn default_poll_duration() -> u32 {
    1440
}
fn default_poll_options() -> u8 {
    10
}

#[derive(Clone, Debug, Default, Deserialize, Serialize, PartialEq, Eq)]
pub struct CountersConfig {
    #[serde(default)]
    pub enabled: bool,
    #[serde(default)]
    pub counters: Vec<CounterConfig>,
}
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct CounterConfig {
    pub id: String,
    #[serde(default = "default_true")]
    pub enabled: bool,
    pub channel_id: String,
    pub label: String,
    pub kind: String,
    #[serde(default)]
    pub goal: Option<u64>,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct AfkConfig {
    #[serde(default)]
    pub enabled: bool,
    #[serde(default = "default_afk_max_hours")]
    pub maximum_hours: u16,
    #[serde(default = "default_true")]
    pub clear_on_message: bool,
    #[serde(default = "default_true")]
    pub notify_mentions: bool,
}
impl Default for AfkConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            maximum_hours: 168,
            clear_on_message: true,
            notify_mentions: true,
        }
    }
}
fn default_afk_max_hours() -> u16 {
    168
}

#[derive(Clone, Debug, Default, Deserialize, Serialize, PartialEq, Eq)]
pub struct VoiceTextLinkingConfig {
    #[serde(default)]
    pub enabled: bool,
    #[serde(default)]
    pub links: Vec<VoiceTextLinkConfig>,
}
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct VoiceTextLinkConfig {
    pub voice_channel_id: String,
    pub text_channel_id: String,
    #[serde(default = "default_true")]
    pub announce_joins: bool,
    #[serde(default)]
    pub announce_leaves: bool,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct VerificationConfig {
    #[serde(default)]
    pub enabled: bool,
    pub channel_id: Option<String>,
    pub verified_role_id: Option<String>,
    pub pending_role_id: Option<String>,
    #[serde(default)]
    pub minimum_account_age_days: u16,
    #[serde(default)]
    pub require_avatar: bool,
    #[serde(default = "default_verification_message")]
    pub message: String,
    #[serde(default = "default_verification_rejection")]
    pub rejection_message: String,
}

impl Default for VerificationConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            channel_id: None,
            verified_role_id: None,
            pending_role_id: None,
            minimum_account_age_days: 0,
            require_avatar: false,
            message: default_verification_message(),
            rejection_message: default_verification_rejection(),
        }
    }
}

fn default_verification_message() -> String {
    "Welcome {user}. Press Verify to access {server}.".into()
}
fn default_verification_rejection() -> String {
    "Your account does not meet this server's verification requirements.".into()
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct EconomyConfig {
    #[serde(default)]
    pub enabled: bool,
    #[serde(default = "default_currency_name")]
    pub currency_name: String,
    #[serde(default = "default_currency_symbol")]
    pub currency_symbol: String,
    #[serde(default = "default_daily_amount")]
    pub daily_amount: u64,
    #[serde(default = "default_work_min")]
    pub work_min: u64,
    #[serde(default = "default_work_max")]
    pub work_max: u64,
    #[serde(default = "default_work_cooldown")]
    pub work_cooldown_seconds: u32,
    #[serde(default)]
    pub transfer_tax_percent: u8,
    #[serde(default = "default_true")]
    pub games_enabled: bool,
    #[serde(default = "default_minimum_wager")]
    pub minimum_wager: u64,
    #[serde(default = "default_maximum_wager")]
    pub maximum_wager: u64,
    #[serde(default)]
    pub shop_items: Vec<EconomyShopItem>,
}
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct EconomyShopItem {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub description: String,
    pub price: u64,
    pub role_id: Option<String>,
}
impl Default for EconomyConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            currency_name: default_currency_name(),
            currency_symbol: default_currency_symbol(),
            daily_amount: default_daily_amount(),
            work_min: default_work_min(),
            work_max: default_work_max(),
            work_cooldown_seconds: default_work_cooldown(),
            transfer_tax_percent: 0,
            games_enabled: true,
            minimum_wager: default_minimum_wager(),
            maximum_wager: default_maximum_wager(),
            shop_items: Vec::new(),
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct TemporaryVoiceConfig {
    #[serde(default)]
    pub enabled: bool,
    pub lobby_channel_id: Option<String>,
    pub category_id: Option<String>,
    #[serde(default = "default_temporary_voice_name")]
    pub default_name: String,
    #[serde(default)]
    pub user_limit: u8,
    #[serde(default = "default_temporary_voice_delete_delay")]
    pub delete_delay_seconds: u16,
    #[serde(default = "default_true")]
    pub owner_controls: bool,
}

impl Default for TemporaryVoiceConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            lobby_channel_id: None,
            category_id: None,
            default_name: default_temporary_voice_name(),
            user_limit: 0,
            delete_delay_seconds: default_temporary_voice_delete_delay(),
            owner_controls: true,
        }
    }
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
    #[serde(default)]
    pub rules: Vec<AutomodRuleConfig>,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct AutomodRuleConfig {
    pub id: String,
    #[serde(default = "default_true")]
    pub enabled: bool,
    pub kind: String,
    #[serde(default = "default_automod_action")]
    pub action: String,
    #[serde(default = "default_automod_timeout")]
    pub timeout_seconds: u32,
    #[serde(default)]
    pub threshold: u16,
    #[serde(default)]
    pub window_seconds: u16,
    #[serde(default)]
    pub patterns: Vec<String>,
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
    #[serde(default)]
    pub role_rewards: Vec<LevelRewardConfig>,
    #[serde(default)]
    pub remove_previous_rewards: bool,
    #[serde(default = "default_minimum_message_length")]
    pub minimum_message_length: u16,
    #[serde(default = "default_duplicate_message_window")]
    pub duplicate_message_window_seconds: u16,
    #[serde(default)]
    pub voice_xp_enabled: bool,
    #[serde(default = "default_voice_xp_per_minute")]
    pub voice_xp_per_minute: u16,
    #[serde(default = "default_voice_xp_session_limit")]
    pub voice_xp_max_session_minutes: u16,
    #[serde(default)]
    pub ignored_voice_channel_ids: Vec<String>,
    #[serde(default = "default_rank_card_color")]
    pub rank_card_color: u32,
    #[serde(default = "default_rank_card_footer")]
    pub rank_card_footer: String,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct LevelRewardConfig {
    pub level: u32,
    pub role_id: String,
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

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct LoggingConfig {
    #[serde(default)]
    pub enabled: bool,
    pub channel_id: Option<String>,
    #[serde(default = "default_true")]
    pub command_events: bool,
    #[serde(default = "default_true")]
    pub message_events: bool,
    #[serde(default = "default_true")]
    pub member_events: bool,
    #[serde(default = "default_true")]
    pub moderation_events: bool,
    #[serde(default = "default_true")]
    pub role_events: bool,
    #[serde(default = "default_true")]
    pub channel_events: bool,
    #[serde(default = "default_true")]
    pub voice_events: bool,
    #[serde(default)]
    pub command_channel_id: Option<String>,
    #[serde(default)]
    pub message_channel_id: Option<String>,
    #[serde(default)]
    pub member_channel_id: Option<String>,
    #[serde(default)]
    pub moderation_channel_id: Option<String>,
    #[serde(default)]
    pub role_channel_id: Option<String>,
    #[serde(default)]
    pub channel_event_channel_id: Option<String>,
    #[serde(default)]
    pub voice_channel_id: Option<String>,
    #[serde(default)]
    pub ignored_channel_ids: Vec<String>,
}

impl Default for LoggingConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            channel_id: None,
            command_events: true,
            message_events: true,
            member_events: true,
            moderation_events: true,
            role_events: true,
            channel_events: true,
            voice_events: true,
            command_channel_id: None,
            message_channel_id: None,
            member_channel_id: None,
            moderation_channel_id: None,
            role_channel_id: None,
            channel_event_channel_id: None,
            voice_channel_id: None,
            ignored_channel_ids: Vec::new(),
        }
    }
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
    #[serde(default)]
    pub moderator_role_ids: Vec<String>,
    #[serde(default)]
    pub warning_thresholds: Vec<WarningThresholdConfig>,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct WarningThresholdConfig {
    pub warnings: u16,
    pub action: String,
    #[serde(default)]
    pub timeout_minutes: u32,
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
    #[serde(default)]
    pub panels: Vec<ReactionRolePanel>,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct ReactionRoleBinding {
    pub message_id: String,
    pub emoji: String,
    pub role_id: String,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct ReactionRolePanel {
    pub id: String,
    #[serde(default = "default_true")]
    pub enabled: bool,
    pub name: String,
    pub title: String,
    #[serde(default)]
    pub description: String,
    pub channel_id: String,
    #[serde(default)]
    pub message_id: Option<String>,
    pub component_type: String,
    pub mode: String,
    #[serde(default)]
    pub exclusive: bool,
    #[serde(default)]
    pub required_role_ids: Vec<String>,
    #[serde(default)]
    pub blocked_role_ids: Vec<String>,
    #[serde(default)]
    pub options: Vec<ReactionRoleOption>,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct ReactionRoleOption {
    pub id: String,
    pub label: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
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

#[derive(Clone, Debug, Default, Deserialize, Serialize, PartialEq, Eq)]
pub struct AutoDeleteConfig {
    #[serde(default)]
    pub enabled: bool,
    #[serde(default)]
    pub rules: Vec<AutoDeleteRule>,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize, PartialEq, Eq)]
pub struct ManagedEmbedsConfig {
    #[serde(default)]
    pub enabled: bool,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct AutoDeleteRule {
    #[serde(default = "default_true")]
    pub enabled: bool,
    pub id: String,
    pub channel_id: String,
    pub delay_seconds: u32,
    #[serde(default)]
    pub ignore_bot_messages: bool,
    #[serde(default)]
    pub excluded_role_ids: Vec<String>,
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
    #[error("moderation role policy or warning thresholds are invalid")]
    InvalidModeration,
    #[error("a welcome channel is required when the welcome module is enabled")]
    MissingWelcomeChannel,
    #[error("welcome direct-message or goodbye settings are invalid")]
    InvalidWelcomeMessages,
    #[error("a logging channel is required when the logging module is enabled")]
    MissingLoggingChannel,
    #[error("logging category channels or ignored channels are invalid")]
    InvalidLogging,
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
    #[error("auto-delete rules contain invalid IDs, channels, delays, or exclusions")]
    InvalidAutoDelete,
    #[error("temporary voice lobby, category, name, limit, or deletion delay is invalid")]
    InvalidTemporaryVoice,
    #[error("economy currency, rewards, cooldowns, wagers, tax, or shop items are invalid")]
    InvalidEconomy,
    #[error("verification channels, roles, account policy, or messages are invalid")]
    InvalidVerification,
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
            || automod.rules.len() > 50
            || automod.rules.iter().enumerate().any(|(index, rule)| {
                rule.id.is_empty()
                    || rule.id.len() > 32
                    || !rule.id.chars().all(|character| {
                        character.is_ascii_alphanumeric() || character == '-' || character == '_'
                    })
                    || automod.rules[..index]
                        .iter()
                        .any(|other| other.id == rule.id)
                    || !matches!(rule.action.as_str(), "delete" | "warn" | "timeout" | "ban")
                    || (rule.action == "timeout"
                        && !(60..=2_419_200).contains(&rule.timeout_seconds))
                    || !matches!(
                        rule.kind.as_str(),
                        "blocked_terms"
                            | "mention_spam"
                            | "invite_links"
                            | "external_links"
                            | "caps"
                            | "lines"
                            | "duplicate_spam"
                            | "rapid_spam"
                            | "attachments"
                            | "stickers"
                            | "emoji_spam"
                            | "zalgo"
                            | "phishing"
                    )
                    || rule.patterns.len() > 100
                    || rule
                        .patterns
                        .iter()
                        .any(|pattern| pattern.is_empty() || pattern.chars().count() > 128)
                    || matches!(
                        rule.kind.as_str(),
                        "mention_spam"
                            | "lines"
                            | "attachments"
                            | "stickers"
                            | "emoji_spam"
                            | "zalgo"
                    ) && !(1..=100).contains(&rule.threshold)
                    || rule.kind == "caps" && !(20..=100).contains(&rule.threshold)
                    || matches!(rule.kind.as_str(), "duplicate_spam" | "rapid_spam")
                        && (!(2..=100).contains(&rule.threshold)
                            || !(2..=3600).contains(&rule.window_seconds))
            })
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
            || invalid_snowflake_list(&leveling.ignored_voice_channel_ids, 100)
            || leveling.minimum_message_length > 500
            || leveling.duplicate_message_window_seconds > 3_600
            || leveling.role_rewards.len() > 100
            || leveling
                .role_rewards
                .iter()
                .enumerate()
                .any(|(index, reward)| {
                    reward.level == 0
                        || !valid_snowflake(&reward.role_id)
                        || leveling.role_rewards[..index].iter().any(|other| {
                            other.level == reward.level || other.role_id == reward.role_id
                        })
                })
            || (leveling.voice_xp_enabled
                && (!(1..=1_000).contains(&leveling.voice_xp_per_minute)
                    || !(1..=1_440).contains(&leveling.voice_xp_max_session_minutes)))
            || leveling.rank_card_color > 0xFF_FF_FF
            || leveling.rank_card_footer.chars().count() > 100
            || (leveling.announce_level_up
                && leveling
                    .announcement_channel_id
                    .as_deref()
                    .is_some_and(|id| !valid_snowflake(id)))
        {
            return Err(ConfigValidationError::InvalidLeveling);
        }
        let temporary_voice = &self.modules.temporary_voice;
        if temporary_voice.default_name.trim().is_empty()
            || temporary_voice.default_name.chars().count() > 100
            || temporary_voice.user_limit > 99
            || temporary_voice.delete_delay_seconds > 300
            || temporary_voice
                .lobby_channel_id
                .as_deref()
                .is_some_and(|id| !valid_snowflake(id))
            || temporary_voice
                .category_id
                .as_deref()
                .is_some_and(|id| !valid_snowflake(id))
            || (temporary_voice.enabled
                && (temporary_voice.lobby_channel_id.is_none()
                    || temporary_voice.category_id.is_none()
                    || temporary_voice.lobby_channel_id == temporary_voice.category_id))
        {
            return Err(ConfigValidationError::InvalidTemporaryVoice);
        }
        let economy = &self.modules.economy;
        if economy.currency_name.trim().is_empty()
            || economy.currency_name.chars().count() > 32
            || economy.currency_symbol.trim().is_empty()
            || economy.currency_symbol.chars().count() > 8
            || economy.daily_amount > 1_000_000
            || economy.work_min == 0
            || economy.work_min > economy.work_max
            || economy.work_max > 1_000_000
            || !(60..=86_400).contains(&economy.work_cooldown_seconds)
            || economy.transfer_tax_percent > 50
            || economy.minimum_wager == 0
            || economy.minimum_wager > economy.maximum_wager
            || economy.maximum_wager > 1_000_000_000
            || economy.shop_items.len() > 100
            || economy.shop_items.iter().enumerate().any(|(index, item)| {
                item.id.is_empty()
                    || item.id.len() > 32
                    || !item.id.chars().all(|character| {
                        character.is_ascii_lowercase()
                            || character.is_ascii_digit()
                            || character == '-'
                            || character == '_'
                    })
                    || economy.shop_items[..index]
                        .iter()
                        .any(|other| other.id == item.id)
                    || item.name.trim().is_empty()
                    || item.name.chars().count() > 100
                    || item.description.chars().count() > 500
                    || item.price == 0
                    || item.price > i64::MAX as u64
                    || item
                        .role_id
                        .as_deref()
                        .is_some_and(|id| !valid_snowflake(id))
            })
        {
            return Err(ConfigValidationError::InvalidEconomy);
        }
        let verification = &self.modules.verification;
        if verification.minimum_account_age_days > 3650
            || verification.message.trim().is_empty()
            || verification.message.chars().count() > 1800
            || verification.rejection_message.trim().is_empty()
            || verification.rejection_message.chars().count() > 500
            || verification
                .channel_id
                .as_deref()
                .is_some_and(|id| !valid_snowflake(id))
            || verification
                .verified_role_id
                .as_deref()
                .is_some_and(|id| !valid_snowflake(id))
            || verification
                .pending_role_id
                .as_deref()
                .is_some_and(|id| !valid_snowflake(id))
            || (verification.enabled
                && (verification.channel_id.is_none()
                    || verification.verified_role_id.is_none()
                    || verification.pending_role_id == verification.verified_role_id))
        {
            return Err(ConfigValidationError::InvalidVerification);
        }
        let moderation = &self.modules.moderation;
        let mut warning_counts = std::collections::HashSet::new();
        if moderation.default_reason.trim().is_empty()
            || moderation.default_reason.chars().count() > 512
            || invalid_snowflake_list(&moderation.moderator_role_ids, 100)
            || moderation.warning_thresholds.len() > 20
            || moderation.warning_thresholds.iter().any(|threshold| {
                threshold.warnings == 0
                    || !warning_counts.insert(threshold.warnings)
                    || !matches!(threshold.action.as_str(), "timeout" | "kick" | "ban")
                    || (threshold.action == "timeout"
                        && !(1..=40_320).contains(&threshold.timeout_minutes))
            })
        {
            return Err(ConfigValidationError::InvalidModeration);
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
        let logging = &self.modules.logging;
        if invalid_snowflake_list(&logging.ignored_channel_ids, 100)
            || [
                logging.command_channel_id.as_deref(),
                logging.message_channel_id.as_deref(),
                logging.member_channel_id.as_deref(),
                logging.moderation_channel_id.as_deref(),
                logging.role_channel_id.as_deref(),
                logging.channel_event_channel_id.as_deref(),
                logging.voice_channel_id.as_deref(),
            ]
            .into_iter()
            .flatten()
            .any(|id| !valid_snowflake(id))
        {
            return Err(ConfigValidationError::InvalidLogging);
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
        let mut panel_ids = std::collections::HashSet::new();
        if (reaction_roles.enabled && reaction_roles.bindings.is_empty())
            && reaction_roles.panels.is_empty()
            || reaction_roles.bindings.len() > 100
            || reaction_roles.panels.len() > 50
            || reaction_roles.bindings.iter().any(|binding| {
                !valid_snowflake(&binding.message_id)
                    || !valid_snowflake(&binding.role_id)
                    || binding.emoji.trim().is_empty()
                    || binding.emoji.chars().count() > 64
                    || !mappings.insert((binding.message_id.clone(), binding.emoji.clone()))
            })
            || reaction_roles.panels.iter().any(|panel| {
                let mut option_ids = std::collections::HashSet::new();
                let mut role_ids = std::collections::HashSet::new();
                panel.id.is_empty()
                    || panel.id.chars().count() > 64
                    || !panel_ids.insert(panel.id.clone())
                    || !panel.id.chars().all(|character| {
                        character.is_ascii_alphanumeric() || "_-".contains(character)
                    })
                    || panel.name.trim().is_empty()
                    || panel.name.chars().count() > 80
                    || panel.title.trim().is_empty()
                    || panel.title.chars().count() > 100
                    || panel.description.chars().count() > 1800
                    || !valid_snowflake(&panel.channel_id)
                    || panel
                        .message_id
                        .as_deref()
                        .is_some_and(|id| !valid_snowflake(id))
                    || !matches!(panel.component_type.as_str(), "buttons" | "select")
                    || !matches!(panel.mode.as_str(), "toggle" | "add_only" | "remove_only")
                    || invalid_snowflake_list(&panel.required_role_ids, 25)
                    || invalid_snowflake_list(&panel.blocked_role_ids, 25)
                    || panel.options.is_empty()
                    || panel.options.len() > 25
                    || panel.options.iter().any(|option| {
                        option.id.is_empty()
                            || option.id.chars().count() > 32
                            || !option_ids.insert(option.id.clone())
                            || !option.id.chars().all(|character| {
                                character.is_ascii_alphanumeric() || "_-".contains(character)
                            })
                            || option.label.trim().is_empty()
                            || option.label.chars().count() > 80
                            || option.description.chars().count() > 100
                            || option.emoji.chars().count() > 32
                            || !valid_snowflake(&option.role_id)
                            || !role_ids.insert(option.role_id.clone())
                    })
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
        let auto_delete = &self.modules.auto_delete;
        let mut auto_delete_ids = std::collections::HashSet::new();
        if auto_delete.rules.len() > 100
            || auto_delete.rules.iter().any(|rule| {
                rule.id.is_empty()
                    || rule.id.len() > 64
                    || !rule
                        .id
                        .chars()
                        .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
                    || !auto_delete_ids.insert(rule.id.clone())
                    || !valid_snowflake(&rule.channel_id)
                    || !(5..=2_592_000).contains(&rule.delay_seconds)
                    || invalid_snowflake_list(&rule.excluded_role_ids, 25)
            })
        {
            return Err(ConfigValidationError::InvalidAutoDelete);
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
            moderator_role_ids: Vec::new(),
            warning_thresholds: Vec::new(),
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
            rules: Vec::new(),
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
            role_rewards: Vec::new(),
            remove_previous_rewards: false,
            minimum_message_length: default_minimum_message_length(),
            duplicate_message_window_seconds: default_duplicate_message_window(),
            voice_xp_enabled: false,
            voice_xp_per_minute: default_voice_xp_per_minute(),
            voice_xp_max_session_minutes: default_voice_xp_session_limit(),
            ignored_voice_channel_ids: Vec::new(),
            rank_card_color: default_rank_card_color(),
            rank_card_footer: default_rank_card_footer(),
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
const fn default_minimum_message_length() -> u16 {
    5
}
const fn default_duplicate_message_window() -> u16 {
    300
}
const fn default_voice_xp_per_minute() -> u16 {
    2
}
const fn default_voice_xp_session_limit() -> u16 {
    240
}
const fn default_rank_card_color() -> u32 {
    0x58_65_F2
}
fn default_rank_card_footer() -> String {
    "Keep participating to reach the next level.".into()
}
fn default_temporary_voice_name() -> String {
    "{username}'s channel".into()
}
const fn default_temporary_voice_delete_delay() -> u16 {
    5
}
fn default_currency_name() -> String {
    "credits".into()
}
fn default_currency_symbol() -> String {
    "◈".into()
}
const fn default_daily_amount() -> u64 {
    250
}
const fn default_work_min() -> u64 {
    25
}
const fn default_work_max() -> u64 {
    100
}
const fn default_work_cooldown() -> u32 {
    3_600
}
const fn default_minimum_wager() -> u64 {
    10
}
const fn default_maximum_wager() -> u64 {
    100_000
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
        assert!(config.modules.logging.command_events);
        assert!(config.modules.logging.message_events);
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

    #[test]
    fn auto_delete_rules_require_safe_delays() {
        let mut config = GuildConfig::for_guild("123");
        config.modules.auto_delete.rules.push(AutoDeleteRule {
            enabled: true,
            id: "media".into(),
            channel_id: "456".into(),
            delay_seconds: 60,
            ignore_bot_messages: true,
            excluded_role_ids: vec![],
        });
        assert_eq!(config.validate(), Ok(()));
        config.modules.auto_delete.rules[0].delay_seconds = 1;
        assert_eq!(
            config.validate(),
            Err(ConfigValidationError::InvalidAutoDelete)
        );
    }

    #[test]
    fn economy_requires_bounded_wagers_and_unique_shop_items() {
        let mut config = GuildConfig::for_guild("123");
        config.modules.economy.shop_items.push(EconomyShopItem {
            id: "vip".into(),
            name: "VIP".into(),
            description: "Unlock the VIP role".into(),
            price: 5_000,
            role_id: Some("456".into()),
        });
        assert_eq!(config.validate(), Ok(()));
        config
            .modules
            .economy
            .shop_items
            .push(config.modules.economy.shop_items[0].clone());
        assert_eq!(
            config.validate(),
            Err(ConfigValidationError::InvalidEconomy)
        );
        config.modules.economy.shop_items.pop();
        config.modules.economy.maximum_wager = config.modules.economy.minimum_wager - 1;
        assert_eq!(
            config.validate(),
            Err(ConfigValidationError::InvalidEconomy)
        );
    }
}

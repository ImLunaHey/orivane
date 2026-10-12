export type GuildConfig = {
  guild_id: string
  modules: {
    automod: {
      enabled: boolean
      blocked_words: string[]
      max_mentions: number
      block_invites: boolean
      block_links: boolean
      max_caps_percentage: number
      max_lines: number
      action: 'delete' | 'warn' | 'timeout' | 'ban'
      timeout_seconds: number
      ignored_channel_ids: string[]
      ignored_role_ids: string[]
      log_channel_id: string | null
      response: string
      rules: AutomodRule[]
    }
    leveling: {
      enabled: boolean
      xp_per_message_min: number
      xp_per_message_max: number
      cooldown_seconds: number
      ignored_channel_ids: string[]
      ignored_role_ids: string[]
      announce_level_up: boolean
      announcement_channel_id: string | null
      announcement_message: string
      role_rewards: { level:number; role_id:string }[]
      remove_previous_rewards: boolean
      minimum_message_length: number
      duplicate_message_window_seconds: number
      voice_xp_enabled: boolean
      voice_xp_per_minute: number
      voice_xp_max_session_minutes: number
      ignored_voice_channel_ids: string[]
      rank_card_color: number
      rank_card_footer: string
    }
    welcome: {
      enabled: boolean
      channel_id: string | null
      message: string
      dm_enabled: boolean
      dm_message: string
      goodbye_enabled: boolean
      goodbye_channel_id: string | null
      goodbye_message: string
    }
    logging: LoggingConfig
    utility: { enabled: boolean; ephemeral_responses: boolean }
    moderation: { enabled: boolean; dm_on_action: boolean; default_reason: string; moderator_role_ids:string[]; warning_thresholds:WarningThreshold[] }
    autorole: { enabled: boolean; role_ids: string[]; delay_seconds: number; remove_role_ids: string[]; remove_delay_seconds: number; joinable_role_ids: string[]; single_joinable_role: boolean }
    starboard: { enabled: boolean; channel_id: string | null; emoji: string; threshold: number; allow_self_stars: boolean; ignored_channel_ids: string[] }
    tickets: { enabled: boolean; category_id: string | null; support_role_ids: string[]; transcript_channel_id: string | null; welcome_message: string; channel_name: string }
    custom_commands: { enabled: boolean; max_commands: number; commands: CustomCommand[] }
    reaction_roles: { enabled: boolean; bindings: ReactionRoleBinding[]; panels: ReactionRolePanel[] }
    autoresponder: { enabled: boolean; responses: Autoresponse[] }
    anti_raid: { enabled: boolean; join_threshold: number; join_window_seconds: number; minimum_account_age_days: number; require_avatar: boolean; punish_all_during_raid: boolean; action: 'log' | 'quarantine' | 'timeout' | 'kick' | 'ban'; timeout_seconds: number; quarantine_role_id: string | null; alert_channel_id: string | null }
    scheduled_messages: { enabled: boolean; schedules: ScheduledMessage[] }
    forms: { enabled: boolean; forms: ServerForm[] }
    suggestions: { enabled: boolean; channel_id: string | null; staff_role_ids: string[]; allow_anonymous: boolean; cooldown_seconds: number }
    giveaways: { enabled: boolean; default_channel_id: string | null; manager_role_ids: string[]; default_duration_minutes: number; default_winner_count: number; minimum_account_age_days: number; required_role_ids: string[]; bonus_role_id: string | null; bonus_entries: number }
    reminders: { enabled: boolean; allow_direct_messages: boolean; max_active_per_user: number; minimum_delay_minutes: number; maximum_delay_days: number }
    auto_delete: { enabled: boolean; rules: AutoDeleteRule[] }
    managed_embeds: { enabled: boolean }
    temporary_voice: { enabled:boolean;lobby_channel_id:string|null;category_id:string|null;default_name:string;user_limit:number;delete_delay_seconds:number;owner_controls:boolean }
  }
}

export type LoggingConfig = {
  enabled: boolean
  channel_id: string | null
  command_events: boolean
  message_events: boolean
  member_events: boolean
  moderation_events: boolean
  role_events: boolean
  channel_events: boolean
  voice_events: boolean
  command_channel_id: string | null
  message_channel_id: string | null
  member_channel_id: string | null
  moderation_channel_id: string | null
  role_channel_id: string | null
  channel_event_channel_id: string | null
  voice_channel_id: string | null
  ignored_channel_ids: string[]
}

export type ReactionRoleBinding = { message_id: string; emoji: string; role_id: string }
export type WarningThreshold = { warnings:number;action:'timeout'|'kick'|'ban';timeout_minutes:number }
export type AutomodRule = { id:string;enabled:boolean;kind:'blocked_terms'|'mention_spam'|'invite_links'|'external_links'|'caps'|'lines'|'duplicate_spam'|'rapid_spam'|'attachments'|'stickers'|'emoji_spam'|'zalgo'|'phishing';action:'delete'|'warn'|'timeout'|'ban';timeout_seconds:number;threshold:number;window_seconds:number;patterns:string[] }
export type ReactionRoleOption = { id:string;label:string;description:string;emoji:string;role_id:string }
export type ReactionRolePanel = { id:string;enabled:boolean;name:string;title:string;description:string;channel_id:string;message_id:string|null;component_type:'buttons'|'select';mode:'toggle'|'add_only'|'remove_only';exclusive:boolean;required_role_ids:string[];blocked_role_ids:string[];options:ReactionRoleOption[] }
export type Autoresponse = { enabled: boolean; trigger: string; response: string; match_mode: 'exact' | 'contains' | 'starts_with'; case_sensitive: boolean; allowed_channel_ids: string[]; ignored_role_ids: string[] }
export type ScheduledMessage = { enabled: boolean; id: string; channel_id: string; message: string; starts_at_unix: number; interval_minutes: number; ends_at_unix: number | null }
export type FormQuestion = { id: string; label: string; style: 'short' | 'paragraph'; required: boolean; min_length: number; max_length: number; placeholder: string }
export type ServerForm = { enabled: boolean; id: string; name: string; description: string; destination_channel_id: string; allowed_role_ids: string[]; anonymous: boolean; questions: FormQuestion[] }
export type FormSubmissionAnswer = { question_id: string; label: string; answer: string }
export type FormSubmission = { id: number; guild_id: string; form_id: string; submitter_id: string | null; username: string | null; display_name: string | null; answers: FormSubmissionAnswer[]; created_at: string }
export type SuggestionStatus = 'pending' | 'approved' | 'rejected' | 'implemented' | 'duplicate'
export type Suggestion = { id: number; guild_id: string; channel_id: string; message_id: string | null; author_id: string | null; username: string | null; display_name: string | null; content: string; status: SuggestionStatus; staff_note: string | null; upvotes: number; downvotes: number; created_at: string; updated_at: string }
export type GiveawayStatus = 'pending_delivery' | 'active' | 'ended' | 'cancelled' | 'delivery_failed'
export type Giveaway = { id: number; guild_id: string; channel_id: string; message_id: string | null; creator_id: string; creator_username: string | null; creator_display_name: string | null; title: string; description: string; status: GiveawayStatus; winner_count: number; minimum_account_age_days: number; required_role_ids: string[]; bonus_role_id: string | null; bonus_entries: number; winner_ids: string[]; entrant_count: number; total_entries: number; ends_at: string; ended_at: string | null; reroll_count: number; created_at: string; updated_at: string }
export type GiveawayEntry = { user_id: string; username: string | null; display_name: string | null; weight: number; created_at: string }
export type NewGiveaway = { channel_id: string; title: string; description: string; winner_count: number; minimum_account_age_days: number; required_role_ids: string[]; bonus_role_id: string | null; bonus_entries: number; ends_at_unix: number }
export type ReminderStatus = 'scheduled' | 'delivering' | 'delivered' | 'cancelled' | 'failed'
export type Reminder = { id: number; guild_id: string; channel_id: string | null; user_id: string; username: string | null; display_name: string | null; message: string; delivery_mode: 'channel' | 'dm'; status: ReminderStatus; remind_at: string; next_attempt_at: string; delivered_at: string | null; attempts: number; created_at: string; updated_at: string }
export type AutoDeleteRule = { enabled: boolean; id: string; channel_id: string; delay_seconds: number; ignore_bot_messages: boolean; excluded_role_ids: string[] }
export type MessageDeletionJob = { guild_id: string; channel_id: string; message_id: string; rule_id: string; author_id: string; source: 'rule'|'purge'; status: 'scheduled'|'delivering'|'deleted'|'failed'|'cancelled'; due_at: string; attempts: number; deleted_at: string|null; created_at: string; updated_at: string }
export type EmbedField = { name: string; value: string; inline: boolean }
export type EmbedPayload = { content: string; title: string; description: string; url: string; color: number; author_name: string; author_url: string; author_icon_url: string; footer_text: string; footer_icon_url: string; thumbnail_url: string; image_url: string; timestamp: boolean; fields: EmbedField[] }
export type EmbedTemplate = { id: number; guild_id: string; name: string; payload: EmbedPayload; created_by: string; created_at: string; updated_at: string }
export type EmbedDelivery = { id: number; guild_id: string; template_id: number; channel_id: string; message_id: string|null; actor_id: string; action: 'send'|'edit'; status: 'delivered'|'failed'; error: string|null; created_at: string }
export type TicketPanel = { id:number; guild_id:string; name:string; title:string; description:string; panel_channel_id:string|null; panel_message_id:string|null; category_id:string; support_role_ids:string[]; button_label:string; welcome_message:string; channel_name:string; intake_enabled:boolean; intake_label:string; max_open_per_user:number; auto_close_hours:number; transcript_retention_days:number; default_priority:'low'|'normal'|'high'|'urgent'; enabled:boolean; created_by:string; created_at:string; updated_at:string }
export type NewTicketPanel = Omit<TicketPanel,'id'|'guild_id'|'panel_channel_id'|'panel_message_id'|'created_by'|'created_at'|'updated_at'> & { channel_id:string }
export type Ticket = { id:number; guild_id:string; panel_id:number|null; opener_id:string; opener_username:string|null; opener_display_name:string|null; channel_id:string; subject:string; opening_note:string; status:'open'|'closed'; claimed_by:string|null; assigned_to:string|null; closed_by:string|null; closed_reason:string|null; transcript:string|null; priority:'low'|'normal'|'high'|'urgent'; tags:string[]; first_response_at:string|null; last_activity_at:string; opened_at:string; closed_at:string|null; reopened_at:string|null; updated_at:string }
export type TicketNote = { id:number;ticket_id:number;guild_id:string;author_id:string;body:string;created_at:string }
export type TicketAnalytics = { total:number;open:number;closed:number;unassigned:number;overdue:number;median_first_response_minutes:number|null;median_resolution_minutes:number|null }
export type TicketEvent = { id:number; ticket_id:number; guild_id:string; actor_id:string; action:string; metadata:Record<string,unknown>; created_at:string }

export type CustomCommand = {
  enabled: boolean
  name: string
  description: string
  response: string
  ephemeral: boolean
  allowed_channel_ids: string[]
}

export type GuildChannel = {
  id: string
  name: string
  kind: 'GuildText' | 'GuildAnnouncement' | 'GuildCategory' | string
  position: number
  parent_id: string | null
}

export type GuildRole = {
  id: string
  name: string
  position: number
  managed: boolean
}

export type AnalyticsEvent = {
  id: string
  event: string
  module: string
  outcome: 'success' | 'blocked' | 'failed'
  latency: number
  occurredAt: string
}

export type AuditEvent = {
  id: number
  occurred_at: string
  scope: 'global' | 'guild'
  guild_id: string | null
  actor_id: string
  actor_kind: 'user' | 'bot' | 'system' | 'api'
  action: string
  target_type: string
  target_id: string | null
  outcome: 'success' | 'failure' | 'denied'
  source: string
  request_id: string | null
  reason: string | null
  metadata: Record<string, unknown>
  before_state: unknown | null
  after_state: unknown | null
}

export type ModerationCase = {
  id: number
  guild_id: string
  member_id: string
  moderator_id: string
  action: 'note' | 'warn' | 'timeout' | 'kick' | 'ban'
  reason: string
  status: 'open' | 'resolved' | 'reversed'
  created_at: string
  updated_at: string
}

export type NewModerationCase = Pick<ModerationCase, 'member_id' | 'action' | 'reason'>
export type TemporaryVoiceChannel = { id:number;guild_id:string;channel_id:string;owner_id:string;name:string;user_limit:number;locked:boolean;status:string;created_at:string;updated_at:string;deleted_at:string|null;delete_after:string|null }

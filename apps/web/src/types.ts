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
    logging: {
      enabled: boolean
      channel_id: string | null
    }
    utility: { enabled: boolean; ephemeral_responses: boolean }
    moderation: { enabled: boolean; dm_on_action: boolean; default_reason: string }
    autorole: { enabled: boolean; role_ids: string[]; delay_seconds: number; remove_role_ids: string[]; remove_delay_seconds: number; joinable_role_ids: string[]; single_joinable_role: boolean }
    starboard: { enabled: boolean; channel_id: string | null; emoji: string; threshold: number; allow_self_stars: boolean; ignored_channel_ids: string[] }
    tickets: { enabled: boolean; category_id: string | null; support_role_ids: string[]; transcript_channel_id: string | null; welcome_message: string; channel_name: string }
    custom_commands: { enabled: boolean; max_commands: number; commands: CustomCommand[] }
    reaction_roles: { enabled: boolean; bindings: ReactionRoleBinding[] }
    autoresponder: { enabled: boolean; responses: Autoresponse[] }
    anti_raid: { enabled: boolean; join_threshold: number; join_window_seconds: number; minimum_account_age_days: number; require_avatar: boolean; punish_all_during_raid: boolean; action: 'log' | 'quarantine' | 'timeout' | 'kick' | 'ban'; timeout_seconds: number; quarantine_role_id: string | null; alert_channel_id: string | null }
    scheduled_messages: { enabled: boolean; schedules: ScheduledMessage[] }
    forms: { enabled: boolean; forms: ServerForm[] }
  }
}

export type ReactionRoleBinding = { message_id: string; emoji: string; role_id: string }
export type Autoresponse = { enabled: boolean; trigger: string; response: string; match_mode: 'exact' | 'contains' | 'starts_with'; case_sensitive: boolean; allowed_channel_ids: string[]; ignored_role_ids: string[] }
export type ScheduledMessage = { enabled: boolean; id: string; channel_id: string; message: string; starts_at_unix: number; interval_minutes: number; ends_at_unix: number | null }
export type FormQuestion = { id: string; label: string; style: 'short' | 'paragraph'; required: boolean; min_length: number; max_length: number; placeholder: string }
export type ServerForm = { enabled: boolean; id: string; name: string; description: string; destination_channel_id: string; allowed_role_ids: string[]; anonymous: boolean; questions: FormQuestion[] }
export type FormSubmissionAnswer = { question_id: string; label: string; answer: string }
export type FormSubmission = { id: number; guild_id: string; form_id: string; submitter_id: string | null; username: string | null; display_name: string | null; answers: FormSubmissionAnswer[]; created_at: string }

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

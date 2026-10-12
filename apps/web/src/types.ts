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
    }
    logging: {
      enabled: boolean
      channel_id: string | null
    }
    utility: { enabled: boolean; ephemeral_responses: boolean }
    moderation: { enabled: boolean; dm_on_action: boolean; default_reason: string }
    autorole: { enabled: boolean; role_ids: string[]; delay_seconds: number }
    starboard: { enabled: boolean; channel_id: string | null; emoji: string; threshold: number }
    tickets: { enabled: boolean; category_id: string | null; support_role_ids: string[]; transcript_channel_id: string | null }
    custom_commands: { enabled: boolean; max_commands: number }
  }
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

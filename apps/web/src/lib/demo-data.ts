import type { AnalyticsEvent, GuildConfig } from '../types'

export const demoGuildId = '901256781234567890'

export const defaultConfig: GuildConfig = {
  guild_id: demoGuildId,
  modules: {
    automod: {
      enabled: true,
      blocked_words: ['spoiler-link'],
      max_mentions: 5,
      block_invites: true,
      block_links: false,
      max_caps_percentage: 80,
      max_lines: 12,
      action: 'delete',
      timeout_seconds: 600,
      ignored_channel_ids: [],
      ignored_role_ids: [],
      log_channel_id: null,
      response: '{user}, that message was removed by AutoMod ({rule}).',
    },
    leveling: {
      enabled: true,
      xp_per_message_min: 15,
      xp_per_message_max: 25,
      cooldown_seconds: 60,
      ignored_channel_ids: [],
      ignored_role_ids: [],
      announce_level_up: true,
      announcement_channel_id: null,
      announcement_message: 'Congratulations {user}, you reached level {level}!',
    },
    welcome: {
      enabled: true,
      channel_id: '901256781234567891',
      message: 'Welcome {user} to {server}!',
      dm_enabled: false,
      dm_message: 'Welcome to {server}, {user}!',
      goodbye_enabled: false,
      goodbye_channel_id: null,
      goodbye_message: '{username} has left {server}.',
    },
    logging: { enabled: true, channel_id: '901256781234567892' },
    utility: { enabled: true, ephemeral_responses: false },
    moderation: { enabled: true, dm_on_action: true, default_reason: 'No reason provided' },
    autorole: { enabled: false, role_ids: [], delay_seconds: 0 },
    starboard: { enabled: false, channel_id: null, emoji: '⭐', threshold: 3, allow_self_stars: false, ignored_channel_ids: [] },
    tickets: { enabled: false, category_id: null, support_role_ids: [], transcript_channel_id: null, welcome_message: 'Welcome {user}. Please describe what you need help with. A member of {support_roles} will respond soon.', channel_name: 'ticket-{user_id}' },
    custom_commands: { enabled: false, max_commands: 25, commands: [] },
  },
}

export const analyticsEvents: AnalyticsEvent[] = [
  { id: 'evt_01', event: '/rank', module: 'Leveling', outcome: 'success', latency: 42, occurredAt: '2 min ago' },
  { id: 'evt_02', event: 'mention-spam', module: 'Automod', outcome: 'blocked', latency: 18, occurredAt: '4 min ago' },
  { id: 'evt_03', event: '/warn', module: 'Moderation', outcome: 'success', latency: 87, occurredAt: '9 min ago' },
  { id: 'evt_04', event: 'member.joined', module: 'Welcome', outcome: 'success', latency: 64, occurredAt: '12 min ago' },
  { id: 'evt_05', event: '/settings', module: 'Core', outcome: 'failed', latency: 312, occurredAt: '18 min ago' },
]

export const commandSeries = [38, 52, 45, 71, 64, 86, 96, 83, 112, 104, 129, 121, 145, 138, 166, 157, 184, 176, 194, 188, 213, 207, 226, 241]

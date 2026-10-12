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
    logging: { enabled: true, channel_id: '901256781234567892', command_events: true, message_events: true, member_events: true, moderation_events: true, role_events: true, channel_events: true, voice_events: true, command_channel_id: null, message_channel_id: null, member_channel_id: null, moderation_channel_id: null, role_channel_id: null, channel_event_channel_id: null, voice_channel_id: null, ignored_channel_ids: [] },
    utility: { enabled: true, ephemeral_responses: false },
    moderation: { enabled: true, dm_on_action: true, default_reason: 'No reason provided' },
    autorole: { enabled: false, role_ids: [], delay_seconds: 0, remove_role_ids: [], remove_delay_seconds: 0, joinable_role_ids: [], single_joinable_role: false },
    starboard: { enabled: false, channel_id: null, emoji: '⭐', threshold: 3, allow_self_stars: false, ignored_channel_ids: [] },
    tickets: { enabled: false, category_id: null, support_role_ids: [], transcript_channel_id: null, welcome_message: 'Welcome {user}. Please describe what you need help with. A member of {support_roles} will respond soon.', channel_name: 'ticket-{user_id}' },
    custom_commands: { enabled: false, max_commands: 25, commands: [] },
    reaction_roles: { enabled: false, bindings: [] },
    autoresponder: { enabled: false, responses: [] },
    anti_raid: { enabled: false, join_threshold: 10, join_window_seconds: 60, minimum_account_age_days: 7, require_avatar: false, punish_all_during_raid: false, action: 'quarantine', timeout_seconds: 3600, quarantine_role_id: null, alert_channel_id: null },
    scheduled_messages: { enabled: false, schedules: [] },
    forms: {
      enabled: false,
      forms: [{
        enabled: true,
        id: 'staff-application',
        name: 'Staff application',
        description: 'Applications for community staff roles.',
        destination_channel_id: '100000000000000003',
        allowed_role_ids: [],
        anonymous: false,
        questions: [{
          id: 'experience',
          label: 'What relevant experience do you have?',
          style: 'paragraph',
          required: true,
          min_length: 20,
          max_length: 1000,
          placeholder: 'Tell us about your experience',
        }],
      }],
    },
    suggestions: { enabled: false, channel_id: '100000000000000003', staff_role_ids: [], allow_anonymous: true, cooldown_seconds: 300 },
    giveaways: { enabled: false, default_channel_id: '100000000000000005', manager_role_ids: [], default_duration_minutes: 1440, default_winner_count: 1, minimum_account_age_days: 7, required_role_ids: [], bonus_role_id: null, bonus_entries: 1 },
    reminders: { enabled: false, allow_direct_messages: true, max_active_per_user: 10, minimum_delay_minutes: 1, maximum_delay_days: 365 },
    auto_delete: { enabled: false, rules: [] },
    managed_embeds: { enabled: false },
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

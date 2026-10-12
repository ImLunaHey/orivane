import type { AuditEvent } from '../types'

export const demoAuditEvents: AuditEvent[] = [
  { id: 1042, occurred_at: '2026-10-12T20:41:12.000Z', scope: 'guild', guild_id: '901234567890123456', actor_id: '312058643210567680', actor_kind: 'user', action: 'guild.config.updated', target_type: 'guild_config', target_id: '901234567890123456', outcome: 'success', source: 'dashboard_api', request_id: 'req_8af20d', reason: null, metadata: { changed_fields: ['modules.automod.max_mentions'] }, before_state: { modules: { automod: { max_mentions: 8 } } }, after_state: { modules: { automod: { max_mentions: 6 } } } },
  { id: 1041, occurred_at: '2026-10-12T20:36:03.000Z', scope: 'guild', guild_id: '588456789012345678', actor_id: '118742006320914432', actor_kind: 'user', action: 'moderation.member.warned', target_type: 'member', target_id: '227843117431025543', outcome: 'success', source: 'discord_gateway', request_id: null, reason: 'Repeated link spam', metadata: { case_id: 4821, command: '/warn' }, before_state: null, after_state: { warning_count: 2 } },
  { id: 1040, occurred_at: '2026-10-12T20:28:47.000Z', scope: 'global', guild_id: null, actor_id: 'system', actor_kind: 'system', action: 'commands.synchronised', target_type: 'application_commands', target_id: null, outcome: 'success', source: 'gateway_startup', request_id: null, reason: null, metadata: { registered: 18, removed: 1 }, before_state: null, after_state: { command_count: 18 } },
  { id: 1039, occurred_at: '2026-10-12T20:17:29.000Z', scope: 'guild', guild_id: '827345678901234567', actor_id: '572270865432789802', actor_kind: 'user', action: 'slash_command.ban', target_type: 'member', target_id: '227843117431025543', outcome: 'denied', source: 'discord_gateway', request_id: null, reason: 'Actor lacks Ban Members permission', metadata: { channel_id: '827345678901234590' }, before_state: null, after_state: null },
  { id: 1038, occurred_at: '2026-10-12T19:58:10.000Z', scope: 'guild', guild_id: '901234567890123456', actor_id: '336954228542136654', actor_kind: 'user', action: 'slash_command.ping', target_type: 'slash_command', target_id: 'ping', outcome: 'success', source: 'discord_gateway', request_id: null, reason: null, metadata: { latency_ms: 42 }, before_state: null, after_state: null },
  { id: 1037, occurred_at: '2026-10-12T19:40:54.000Z', scope: 'global', guild_id: null, actor_id: '312058643210567680', actor_kind: 'user', action: 'bot.installed', target_type: 'guild', target_id: '649567890123456789', outcome: 'success', source: 'discord_oauth', request_id: 'oauth_b6e1aa', reason: null, metadata: { permissions: '1099511627775' }, before_state: null, after_state: { guild_id: '649567890123456789' } },
  { id: 1036, occurred_at: '2026-10-12T19:31:08.000Z', scope: 'guild', guild_id: '763456789012345678', actor_id: '682381976543890913', actor_kind: 'api', action: 'guild.config.updated', target_type: 'guild_config', target_id: '763456789012345678', outcome: 'failure', source: 'dashboard_api', request_id: 'req_f881c2', reason: 'Welcome channel is not visible to the bot', metadata: { changed_fields: ['modules.welcome.channel_id'] }, before_state: null, after_state: { modules: { welcome: { channel_id: '763456789012345699' } } } },
  { id: 1035, occurred_at: '2026-10-12T18:55:22.000Z', scope: 'guild', guild_id: '901234567890123456', actor_id: '312058643210567680', actor_kind: 'api', action: 'module.automod.enabled', target_type: 'module', target_id: 'automod', outcome: 'success', source: 'dashboard_api', request_id: 'req_a17c90', reason: null, metadata: {}, before_state: { enabled: false }, after_state: { enabled: true } },
]

const key = 'orivane:demo-audit-events'

export function storedDemoAuditEvents(): AuditEvent[] {
  try {
    return JSON.parse(localStorage.getItem(key) ?? '[]') as AuditEvent[]
  } catch {
    return []
  }
}

export function appendDemoAuditEvent(event: AuditEvent) {
  localStorage.setItem(key, JSON.stringify([event, ...storedDemoAuditEvents()].slice(0, 200)))
}

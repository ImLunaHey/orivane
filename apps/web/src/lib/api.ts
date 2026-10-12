import { defaultConfig, demoGuildId } from './demo-data'
import { appendDemoAuditEvent, demoAuditEvents, storedDemoAuditEvents } from './audit-data'
import type { AuditEvent, GuildChannel, GuildConfig, GuildRole, ModerationCase, NewModerationCase } from '../types'
import { demoServers, type ConnectedServer } from './server-data'
import { globalUsers, type GlobalUser, type UserServerContext } from './user-data'
import { hourlyCommands, type ChartDatum } from './analytics-data'

const configuredApiBase = import.meta.env.VITE_API_BASE_URL as string | undefined
const apiBase = configuredApiBase === 'same-origin' ? window.location.origin : configuredApiBase
const demoActorId = '312058643210567680'

export const isDemoMode = !configuredApiBase
export const configuredGuildId = (import.meta.env.VITE_GUILD_ID as string | undefined) ?? demoGuildId

const demoKey = (guildId: string) => `orivane:config:${guildId}`

function demoConfig(guildId: string): GuildConfig {
  const saved = localStorage.getItem(demoKey(guildId))
  if (!saved) return { ...structuredClone(defaultConfig), guild_id: guildId }
  const parsed = JSON.parse(saved) as Partial<GuildConfig>
  const defaults = structuredClone(defaultConfig)
  return { ...defaults, ...parsed, guild_id: guildId, modules: { ...defaults.modules, ...parsed.modules } }
}

async function request<T>(path: string, init?: RequestInit): Promise<T> {
  const response = await fetch(`${apiBase}${path}`, {
    ...init,
    credentials: 'include',
    headers: {
      'content-type': 'application/json',
      ...init?.headers,
    },
  })
  if (!response.ok) {
    const body = (await response.json().catch(() => null)) as { error?: string } | null
    throw new Error(body?.error ?? `Request failed with status ${response.status}`)
  }
  if (response.status === 204) return undefined as T
  return response.json() as Promise<T>
}

export async function getGuildConfig(guildId = configuredGuildId): Promise<GuildConfig> {
  if (isDemoMode) {
    await new Promise((resolve) => setTimeout(resolve, 180))
    return demoConfig(guildId)
  }
  return request<GuildConfig>(`/v1/guilds/${guildId}/config`)
}

export async function getGuildChannels(guildId: string): Promise<GuildChannel[]> {
  if (isDemoMode) return [
    { id: '100000000000000001', name: 'Information', kind: 'GuildCategory', position: 0, parent_id: null },
    { id: '100000000000000002', name: 'welcome', kind: 'GuildText', position: 1, parent_id: '100000000000000001' },
    { id: '100000000000000003', name: 'bot-logs', kind: 'GuildText', position: 2, parent_id: '100000000000000001' },
    { id: '100000000000000004', name: 'Community', kind: 'GuildCategory', position: 3, parent_id: null },
    { id: '100000000000000005', name: 'general', kind: 'GuildText', position: 4, parent_id: '100000000000000004' },
  ]
  return request<GuildChannel[]>(`/v1/guilds/${guildId}/channels`)
}

export async function getGuildRoles(guildId: string): Promise<GuildRole[]> {
  if (isDemoMode) return []
  return request<GuildRole[]>(`/v1/guilds/${guildId}/roles`)
}

export async function saveGuildConfig(config: GuildConfig): Promise<GuildConfig> {
  if (isDemoMode) {
    await new Promise((resolve) => setTimeout(resolve, 260))
    const before = demoConfig(config.guild_id)
    localStorage.setItem(demoKey(config.guild_id), JSON.stringify(config))
    appendDemoAuditEvent({
      id: Date.now(), occurred_at: new Date().toISOString(), scope: 'guild', guild_id: config.guild_id,
      actor_id: demoActorId, actor_kind: 'user', action: 'guild.config.updated', target_type: 'guild_config',
      target_id: config.guild_id, outcome: 'success', source: 'dashboard_api', request_id: `demo_${Date.now()}`,
      reason: null, metadata: { demo: true }, before_state: before, after_state: config,
    })
    return config
  }
  return request<GuildConfig>(`/v1/guilds/${config.guild_id}/config`, {
    method: 'PUT',
    body: JSON.stringify(config),
  })
}

export async function getAuditEvents(guildId?: string): Promise<AuditEvent[]> {
  if (isDemoMode) {
    await new Promise((resolve) => setTimeout(resolve, 120))
    const events = [...storedDemoAuditEvents(), ...demoAuditEvents].sort((a, b) => b.occurred_at.localeCompare(a.occurred_at))
    return guildId ? events.filter((event) => event.guild_id === guildId) : events
  }
  return request<AuditEvent[]>(guildId ? `/v1/guilds/${guildId}/audit?limit=200` : '/v1/audit?limit=200')
}

export async function getAuditEvent(eventId: string, guildId?: string): Promise<AuditEvent> {
  if (isDemoMode) {
    await new Promise((resolve) => setTimeout(resolve, 90))
    const event = [...storedDemoAuditEvents(), ...demoAuditEvents].find((item) => String(item.id) === eventId && (!guildId || item.guild_id === guildId))
    if (!event) throw new Error('Audit event not found')
    return event
  }
  return request<AuditEvent>(guildId ? `/v1/guilds/${guildId}/audit/${eventId}` : `/v1/audit/${eventId}`)
}

const starterCases: ModerationCase[] = [
  { id: 187, guild_id: configuredGuildId, member_id: '118742006320914432', moderator_id: demoActorId, action: 'warn', reason: 'Repeated link spam', status: 'open', created_at: '2026-10-11T19:52:00Z', updated_at: '2026-10-11T19:52:00Z' },
  { id: 186, guild_id: configuredGuildId, member_id: '227843117431025543', moderator_id: 'automod', action: 'timeout', reason: 'Mention flood', status: 'open', created_at: '2026-10-11T19:26:00Z', updated_at: '2026-10-11T19:26:00Z' },
]

function demoCases(guildId: string): ModerationCase[] {
  const saved = localStorage.getItem(`orivane:cases:${guildId}`)
  return saved ? JSON.parse(saved) as ModerationCase[] : starterCases.map((item) => ({ ...item, guild_id: guildId }))
}

export async function getModerationCases(guildId: string): Promise<ModerationCase[]> {
  if (isDemoMode) return demoCases(guildId)
  return request<ModerationCase[]>(`/v1/guilds/${guildId}/cases`)
}

export async function getModerationCase(guildId: string, caseId: string): Promise<ModerationCase> {
  if (isDemoMode) {
    const item = demoCases(guildId).find((entry) => String(entry.id) === caseId)
    if (!item) throw new Error('Moderation case not found')
    return item
  }
  return request<ModerationCase>(`/v1/guilds/${guildId}/cases/${caseId}`)
}

export async function createModerationCase(guildId: string, input: NewModerationCase): Promise<ModerationCase> {
  if (isDemoMode) {
    const cases = demoCases(guildId)
    const now = new Date().toISOString()
    const item: ModerationCase = { id: Math.max(0, ...cases.map(({ id }) => id)) + 1, guild_id: guildId, moderator_id: demoActorId, status: 'open', created_at: now, updated_at: now, ...input }
    localStorage.setItem(`orivane:cases:${guildId}`, JSON.stringify([item, ...cases]))
    appendDemoAuditEvent({ id: Date.now(), occurred_at: now, scope: 'guild', guild_id: guildId, actor_id: demoActorId, actor_kind: 'user', action: 'moderation.case.created', target_type: 'member', target_id: input.member_id, outcome: 'success', source: 'dashboard_api', request_id: `demo_${Date.now()}`, reason: null, metadata: { action: input.action, case_id: item.id }, before_state: null, after_state: item })
    return item
  }
  return request<ModerationCase>(`/v1/guilds/${guildId}/cases`, { method: 'POST', body: JSON.stringify(input) })
}

export async function recordDashboardActivity(input: { action: 'dashboard.page.viewed' | 'dashboard.data.exported'; guild_id?: string; path?: string; export?: string }): Promise<void> {
  if (isDemoMode) {
    appendDemoAuditEvent({ id: Date.now(), occurred_at: new Date().toISOString(), scope: input.guild_id ? 'guild' : 'global', guild_id: input.guild_id ?? null, actor_id: demoActorId, actor_kind: 'user', action: input.action, target_type: 'dashboard_page', target_id: input.path ?? null, outcome: 'success', source: 'dashboard_web', request_id: `demo_${Date.now()}`, reason: null, metadata: { path: input.path, export: input.export }, before_state: null, after_state: null })
    return
  }
  await request<void>('/v1/activity', { method: 'POST', body: JSON.stringify(input) })
}

export async function syncCommands(): Promise<{ synchronized: number }> {
  if (isDemoMode) {
    await new Promise((resolve) => setTimeout(resolve, 500))
    appendDemoAuditEvent({ id: Date.now(), occurred_at: new Date().toISOString(), scope: 'global', guild_id: null, actor_id: demoActorId, actor_kind: 'user', action: 'commands.synchronized', target_type: 'slash_command_registry', target_id: null, outcome: 'success', source: 'dashboard_api', request_id: `demo_${Date.now()}`, reason: null, metadata: { command_count: 3 }, before_state: null, after_state: null })
    return { synchronized: 3 }
  }
  return request<{ synchronized: number }>('/v1/commands/sync', { method: 'POST' })
}

type ServerResponse = Omit<ConnectedServer, 'installer' | 'installerId'> & { installer_id: string | null; installer_name: string | null; owner_id: string | null; installed_at: string }

export async function getServers(): Promise<ConnectedServer[]> {
  if (isDemoMode) return demoServers
  const servers = await request<ServerResponse[]>('/v1/servers')
  return servers.map(({ installer_id, installer_name, owner_id, installed_at, ...server }) => ({
    ...server,
    installerId: installer_id ?? '',
    installer: installer_name ?? (installer_id ? `${installer_id.slice(0, 4)}…${installer_id.slice(-4)}` : 'Unattributed'),
    ownerId: owner_id ?? undefined,
    installedAt: installed_at,
  }))
}

type UserResponse = { id: string; username: string | null; display_name: string | null; servers: number; installed_servers: number; install_commands_30d: number; commands: number; cases: number; last_seen: string | null }
type UserContextResponse = { server_id: string; server_name: string; relationship: UserServerContext['relationship']; commands: number; cases: number; last_seen: string | null }

function displayUser(user: UserResponse, contexts: UserServerContext[] = []): GlobalUser {
  const username = user.username ?? user.id
  return {
    id: user.id,
    username,
    displayName: user.display_name ?? (user.username ? user.username : `Discord user ${user.id.slice(-4)}`),
    kind: user.installed_servers ? 'Installer' : 'Member',
    servers: user.servers,
    installedServers: user.installed_servers,
    installCommands30d: user.install_commands_30d,
    commands: user.commands,
    cases: user.cases,
    lastSeen: user.last_seen ? new Date(user.last_seen).toLocaleString() : 'No activity recorded',
    accountAge: 'Not available from telemetry',
    contexts,
  }
}

export async function getUsers(): Promise<GlobalUser[]> {
  if (isDemoMode) return globalUsers
  return (await request<UserResponse[]>('/v1/users')).map((user) => displayUser(user))
}

export async function getUser(userId: string): Promise<GlobalUser> {
  if (isDemoMode) {
    const user = globalUsers.find((item) => item.id === userId)
    if (!user) throw new Error('Global user not found')
    return user
  }
  const response = await request<{ user: UserResponse; contexts: UserContextResponse[] }>(`/v1/users/${userId}`)
  const contexts: UserServerContext[] = response.contexts.map((context) => ({
    serverId: context.server_id,
    serverName: context.server_name,
    relationship: context.relationship,
    commands: context.commands,
    cases: context.cases,
    lastSeen: context.last_seen ? new Date(context.last_seen).toLocaleString() : 'No activity recorded',
  }))
  return displayUser(response.user, contexts)
}

export type GuildDashboardStats = {
  commands_today: number
  active_members_30d: number
  automod_actions_today: number
  open_cases: number
  p95_latency_ms: number | null
}

export async function getGuildStats(guildId: string): Promise<GuildDashboardStats> {
  if (isDemoMode) return { commands_today: 3_842, active_members_30d: 4_208, automod_actions_today: 187, open_cases: 14, p95_latency_ms: 94 }
  return request<GuildDashboardStats>(`/v1/guilds/${guildId}/stats`)
}

export async function getCommandVolume(guildId?: string): Promise<ChartDatum[]> {
  if (isDemoMode) return hourlyCommands
  const path = guildId ? `/v1/guilds/${guildId}/analytics/commands` : '/v1/analytics/commands'
  const buckets = await request<Array<{ occurred_at: string; value: number }>>(path)
  return buckets.map((bucket) => ({
    label: new Intl.DateTimeFormat(undefined, {
      month: 'short',
      day: 'numeric',
      hour: '2-digit',
      minute: '2-digit',
      timeZone: 'UTC',
      timeZoneName: 'short',
    }).format(new Date(bucket.occurred_at)),
    value: bucket.value,
    detail: `${bucket.value.toLocaleString()} audited slash commands in this four-hour UTC window`,
  }))
}

export type InfrastructureHealth = {
  status: 'operational' | 'degraded'
  services: Array<{ name: string; status: 'operational' | 'unavailable'; latency_ms: number }>
}

export async function getHealthReport(): Promise<InfrastructureHealth> {
  if (isDemoMode) return { status: 'operational', services: [{ name: 'PostgreSQL', status: 'operational', latency_ms: 7 }, { name: 'Redis', status: 'operational', latency_ms: 2 }] }
  return request<InfrastructureHealth>('/v1/health')
}

export type GuildMemberSummary = {
  id: string
  commands_30d: number
  cases: number
  last_seen: string | null
}

export async function getGuildMembers(guildId: string): Promise<GuildMemberSummary[]> {
  if (isDemoMode) return []
  return request<GuildMemberSummary[]>(`/v1/guilds/${guildId}/members`)
}

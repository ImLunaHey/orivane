import { defaultConfig, demoGuildId } from './demo-data'
import { appendDemoAuditEvent, demoAuditEvents, storedDemoAuditEvents } from './audit-data'
import type { AuditEvent, FormSubmission, Giveaway, GiveawayEntry, GuildChannel, GuildConfig, GuildRole, MessageDeletionJob, ModerationCase, NewGiveaway, NewModerationCase, Reminder, Suggestion, SuggestionStatus } from '../types'
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

const demoSubmissions: FormSubmission[] = [{
  id: 1,
  guild_id: demoGuildId,
  form_id: 'staff-application',
  submitter_id: '118742006320914432',
  username: 'silver.arc',
  display_name: 'Silver',
  answers: [{ question_id: 'experience', label: 'What relevant experience do you have?', answer: 'I have moderated community servers for several years.' }],
  created_at: '2026-10-12T15:18:00Z',
}]

export async function getFormSubmissions(guildId: string, formId: string): Promise<FormSubmission[]> {
  if (isDemoMode) return demoSubmissions.filter((submission) => submission.form_id === formId).map((submission) => ({ ...submission, guild_id: guildId }))
  return request<FormSubmission[]>(`/v1/guilds/${guildId}/forms/${encodeURIComponent(formId)}/submissions`)
}

export async function getFormSubmission(guildId: string, formId: string, submissionId: string): Promise<FormSubmission> {
  if (isDemoMode) {
    const submission = (await getFormSubmissions(guildId, formId)).find((item) => String(item.id) === submissionId)
    if (!submission) throw new Error('Form submission not found')
    return submission
  }
  return request<FormSubmission>(`/v1/guilds/${guildId}/forms/${encodeURIComponent(formId)}/submissions/${submissionId}`)
}

const starterSuggestions: Suggestion[] = [
  { id: 3, guild_id: demoGuildId, channel_id: '100000000000000003', message_id: '200000000000000003', author_id: '118742006320914432', username: 'silver.arc', display_name: 'Silver', content: 'Add a monthly community showcase for member projects.', status: 'pending', staff_note: null, upvotes: 18, downvotes: 2, created_at: '2026-10-12T14:42:00Z', updated_at: '2026-10-12T14:42:00Z' },
  { id: 2, guild_id: demoGuildId, channel_id: '100000000000000003', message_id: '200000000000000002', author_id: null, username: null, display_name: null, content: 'Create a read-only channel for release notes.', status: 'approved', staff_note: 'Staff will include this in the next channel cleanup.', upvotes: 31, downvotes: 1, created_at: '2026-10-11T09:16:00Z', updated_at: '2026-10-12T10:05:00Z' },
  { id: 1, guild_id: demoGuildId, channel_id: '100000000000000003', message_id: '200000000000000001', author_id: '227843117431025543', username: 'northstar', display_name: 'Northstar', content: 'Run a weekly game night poll.', status: 'implemented', staff_note: 'Added to the events calendar.', upvotes: 24, downvotes: 4, created_at: '2026-10-08T18:30:00Z', updated_at: '2026-10-10T12:00:00Z' },
]

const suggestionKey = (guildId: string) => `orivane:suggestions:${guildId}`
function demoSuggestions(guildId: string): Suggestion[] {
  const saved = localStorage.getItem(suggestionKey(guildId))
  return saved ? JSON.parse(saved) as Suggestion[] : starterSuggestions.map((item) => ({ ...item, guild_id: guildId }))
}

export async function getSuggestions(guildId: string): Promise<Suggestion[]> {
  if (isDemoMode) return demoSuggestions(guildId)
  return request<Suggestion[]>(`/v1/guilds/${guildId}/suggestions`)
}

export async function getSuggestion(guildId: string, suggestionId: string): Promise<Suggestion> {
  if (isDemoMode) {
    const item = demoSuggestions(guildId).find((suggestion) => String(suggestion.id) === suggestionId)
    if (!item) throw new Error('Suggestion not found')
    return item
  }
  return request<Suggestion>(`/v1/guilds/${guildId}/suggestions/${suggestionId}`)
}

export async function updateSuggestionStatus(input: { guildId: string; suggestionId: string; status: SuggestionStatus; staff_note: string | null }): Promise<Suggestion> {
  if (isDemoMode) {
    const suggestions = demoSuggestions(input.guildId)
    const before = suggestions.find((item) => String(item.id) === input.suggestionId)
    if (!before) throw new Error('Suggestion not found')
    const updated = { ...before, status: input.status, staff_note: input.staff_note, updated_at: new Date().toISOString() }
    localStorage.setItem(suggestionKey(input.guildId), JSON.stringify(suggestions.map((item) => item.id === updated.id ? updated : item)))
    appendDemoAuditEvent({ id: Date.now(), occurred_at: updated.updated_at, scope: 'guild', guild_id: input.guildId, actor_id: demoActorId, actor_kind: 'user', action: 'suggestion.status.updated', target_type: 'suggestion', target_id: input.suggestionId, outcome: 'success', source: 'dashboard_api', request_id: `demo_${Date.now()}`, reason: null, metadata: { status: input.status, has_staff_note: Boolean(input.staff_note) }, before_state: { status: before.status }, after_state: { status: updated.status } })
    return updated
  }
  return request<Suggestion>(`/v1/guilds/${input.guildId}/suggestions/${input.suggestionId}`, { method: 'PATCH', body: JSON.stringify({ status: input.status, staff_note: input.staff_note }) })
}

const giveawayKey = (guildId: string) => `orivane:giveaways:${guildId}`
const entryKey = (guildId: string, giveawayId: string | number) => `orivane:giveaway-entries:${guildId}:${giveawayId}`
function starterGiveaways(guildId: string): Giveaway[] {
  const now = Date.now()
  return [
    { id: 2, guild_id: guildId, channel_id: '100000000000000005', message_id: '300000000000000002', creator_id: demoActorId, creator_username: 'imlunahey', creator_display_name: 'luna', title: 'One month of Discord Nitro', description: 'Community appreciation giveaway.', status: 'active', winner_count: 1, minimum_account_age_days: 7, required_role_ids: [], bonus_role_id: null, bonus_entries: 1, winner_ids: [], entrant_count: 2, total_entries: 3, ends_at: new Date(now + 3_600_000).toISOString(), ended_at: null, reroll_count: 0, created_at: new Date(now - 3_600_000).toISOString(), updated_at: new Date(now - 3_600_000).toISOString() },
    { id: 1, guild_id: guildId, channel_id: '100000000000000005', message_id: '300000000000000001', creator_id: demoActorId, creator_username: 'imlunahey', creator_display_name: 'luna', title: 'Community game key', description: '', status: 'ended', winner_count: 1, minimum_account_age_days: 0, required_role_ids: [], bonus_role_id: null, bonus_entries: 0, winner_ids: ['118742006320914432'], entrant_count: 2, total_entries: 2, ends_at: new Date(now - 86_400_000).toISOString(), ended_at: new Date(now - 86_400_000).toISOString(), reroll_count: 0, created_at: new Date(now - 172_800_000).toISOString(), updated_at: new Date(now - 86_400_000).toISOString() },
  ]
}
function demoGiveaways(guildId: string): Giveaway[] { const saved = localStorage.getItem(giveawayKey(guildId)); return saved ? JSON.parse(saved) as Giveaway[] : starterGiveaways(guildId) }
function demoGiveawayEntries(guildId: string, giveawayId: string): GiveawayEntry[] { const saved = localStorage.getItem(entryKey(guildId, giveawayId)); if (saved) return JSON.parse(saved) as GiveawayEntry[]; return [{ user_id: '118742006320914432', username: 'silver.arc', display_name: 'Silver', weight: 2, created_at: new Date(Date.now() - 1_800_000).toISOString() }, { user_id: '227843117431025543', username: 'northstar', display_name: 'Northstar', weight: 1, created_at: new Date(Date.now() - 900_000).toISOString() }] }

export async function getGiveaways(guildId: string): Promise<Giveaway[]> { if (isDemoMode) return demoGiveaways(guildId); return request<Giveaway[]>(`/v1/guilds/${guildId}/giveaways`) }
export async function getGiveaway(guildId: string, giveawayId: string): Promise<Giveaway> { if (isDemoMode) { const item = demoGiveaways(guildId).find((giveaway) => String(giveaway.id) === giveawayId); if (!item) throw new Error('Giveaway not found'); return item } return request<Giveaway>(`/v1/guilds/${guildId}/giveaways/${giveawayId}`) }
export async function getGiveawayEntries(guildId: string, giveawayId: string): Promise<GiveawayEntry[]> { if (isDemoMode) return demoGiveawayEntries(guildId, giveawayId); return request<GiveawayEntry[]>(`/v1/guilds/${guildId}/giveaways/${giveawayId}/entries`) }
export async function createGiveaway(input: { guildId: string; giveaway: NewGiveaway }): Promise<Giveaway> {
  if (isDemoMode) {
    const items = demoGiveaways(input.guildId); const now = new Date().toISOString(); const id = Math.max(0, ...items.map((item) => item.id)) + 1
    const item: Giveaway = { id, guild_id: input.guildId, message_id: `demo-${id}`, creator_id: demoActorId, creator_username: 'imlunahey', creator_display_name: 'luna', status: 'active', winner_ids: [], entrant_count: 0, total_entries: 0, ends_at: new Date(input.giveaway.ends_at_unix * 1000).toISOString(), ended_at: null, reroll_count: 0, created_at: now, updated_at: now, ...input.giveaway }
    localStorage.setItem(giveawayKey(input.guildId), JSON.stringify([item, ...items])); appendDemoAuditEvent({ id: Date.now(), occurred_at: now, scope: 'guild', guild_id: input.guildId, actor_id: demoActorId, actor_kind: 'user', action: 'giveaway.created', target_type: 'giveaway', target_id: String(id), outcome: 'success', source: 'dashboard_api', request_id: `demo_${Date.now()}`, reason: null, metadata: { winner_count: item.winner_count, ends_at_unix: input.giveaway.ends_at_unix }, before_state: null, after_state: null }); return item
  }
  return request<Giveaway>(`/v1/guilds/${input.guildId}/giveaways`, { method: 'POST', body: JSON.stringify(input.giveaway) })
}
export async function endGiveaway(input: { guildId: string; giveawayId: string }): Promise<Giveaway> { return changeDemoOrLiveGiveaway(input, false) }
export async function rerollGiveaway(input: { guildId: string; giveawayId: string }): Promise<Giveaway> { return changeDemoOrLiveGiveaway(input, true) }
async function changeDemoOrLiveGiveaway(input: { guildId: string; giveawayId: string }, reroll: boolean): Promise<Giveaway> {
  if (isDemoMode) {
    const items = demoGiveaways(input.guildId); const before = items.find((item) => String(item.id) === input.giveawayId); if (!before) throw new Error('Giveaway not found')
    const entries = demoGiveawayEntries(input.guildId, input.giveawayId); const prior = new Set(before.winner_ids); const winners = entries.filter((entry) => !reroll || !prior.has(entry.user_id)).slice(0, before.winner_count).map((entry) => entry.user_id); if (reroll && !winners.length) throw new Error('No replacement entrants are available')
    const now = new Date().toISOString(); const updated = { ...before, status: 'ended' as const, winner_ids: winners, ended_at: before.ended_at ?? now, reroll_count: before.reroll_count + Number(reroll), updated_at: now }; localStorage.setItem(giveawayKey(input.guildId), JSON.stringify(items.map((item) => item.id === updated.id ? updated : item))); appendDemoAuditEvent({ id: Date.now(), occurred_at: now, scope: 'guild', guild_id: input.guildId, actor_id: demoActorId, actor_kind: 'user', action: reroll ? 'giveaway.rerolled' : 'giveaway.ended', target_type: 'giveaway', target_id: input.giveawayId, outcome: 'success', source: 'dashboard_api', request_id: `demo_${Date.now()}`, reason: null, metadata: { winner_count: winners.length }, before_state: null, after_state: null }); return updated
  }
  return request<Giveaway>(`/v1/guilds/${input.guildId}/giveaways/${input.giveawayId}/${reroll ? 'reroll' : 'end'}`, { method: 'POST' })
}

const reminderKey = (guildId: string) => `orivane:reminders:${guildId}`
function demoReminders(guildId: string): Reminder[] {
  const saved = localStorage.getItem(reminderKey(guildId)); if (saved) return JSON.parse(saved) as Reminder[]
  const now = Date.now(); return [
    { id: 2, guild_id: guildId, channel_id: '100000000000000005', user_id: '118742006320914432', username: 'silver.arc', display_name: 'Silver', message: 'Community game night starts soon', delivery_mode: 'channel', status: 'scheduled', remind_at: new Date(now + 3_600_000).toISOString(), next_attempt_at: new Date(now + 3_600_000).toISOString(), delivered_at: null, attempts: 0, created_at: new Date(now - 600_000).toISOString(), updated_at: new Date(now - 600_000).toISOString() },
    { id: 1, guild_id: guildId, channel_id: null, user_id: demoActorId, username: 'imlunahey', display_name: 'luna', message: 'Review the moderation queue', delivery_mode: 'dm', status: 'delivered', remind_at: new Date(now - 3_600_000).toISOString(), next_attempt_at: new Date(now - 3_600_000).toISOString(), delivered_at: new Date(now - 3_600_000).toISOString(), attempts: 1, created_at: new Date(now - 7_200_000).toISOString(), updated_at: new Date(now - 3_600_000).toISOString() },
  ]
}
export async function getReminders(guildId: string): Promise<Reminder[]> { if (isDemoMode) return demoReminders(guildId); return request<Reminder[]>(`/v1/guilds/${guildId}/reminders`) }
export async function getReminder(guildId: string, reminderId: string): Promise<Reminder> { if (isDemoMode) { const item = demoReminders(guildId).find((reminder) => String(reminder.id) === reminderId); if (!item) throw new Error('Reminder not found'); return item } return request<Reminder>(`/v1/guilds/${guildId}/reminders/${reminderId}`) }
export async function cancelReminder(input: { guildId: string; reminderId: string }): Promise<Reminder> {
  if (isDemoMode) { const items = demoReminders(input.guildId); const before = items.find((item) => String(item.id) === input.reminderId); if (!before || before.status !== 'scheduled') throw new Error('Reminder is not active'); const updated = { ...before, status: 'cancelled' as const, updated_at: new Date().toISOString() }; localStorage.setItem(reminderKey(input.guildId), JSON.stringify(items.map((item) => item.id === updated.id ? updated : item))); return updated }
  return request<Reminder>(`/v1/guilds/${input.guildId}/reminders/${input.reminderId}/cancel`, { method: 'POST' })
}
export async function getMessageDeletions(guildId:string):Promise<MessageDeletionJob[]>{if(isDemoMode)return [{guild_id:guildId,channel_id:'100000000000000005',message_id:'300000000000000009',rule_id:'media-cleanup',author_id:'118742006320914432',source:'rule',status:'scheduled',due_at:new Date(Date.now()+60000).toISOString(),attempts:0,deleted_at:null,created_at:new Date().toISOString(),updated_at:new Date().toISOString()}];return request<MessageDeletionJob[]>(`/v1/guilds/${guildId}/message-deletions`)}
export async function getMessageDeletion(guildId:string,channelId:string,messageId:string):Promise<MessageDeletionJob>{if(isDemoMode){const item=(await getMessageDeletions(guildId)).find(x=>x.channel_id===channelId&&x.message_id===messageId);if(!item)throw new Error('Deletion job not found');return item}return request<MessageDeletionJob>(`/v1/guilds/${guildId}/message-deletions/${channelId}/${messageId}`)}

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
  username: string | null
  display_name: string | null
  commands_30d: number
  cases: number
  last_seen: string | null
}

export async function getGuildMembers(guildId: string): Promise<GuildMemberSummary[]> {
  if (isDemoMode) return []
  return request<GuildMemberSummary[]>(`/v1/guilds/${guildId}/members`)
}

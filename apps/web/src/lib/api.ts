import { defaultConfig, demoGuildId } from './demo-data'
import { appendDemoAuditEvent, demoAuditEvents, storedDemoAuditEvents } from './audit-data'
import type { AfkStatusRecord, AuditEvent, CounterRunRecord, EconomyAccount, EconomyTransaction, EmbedDelivery, EmbedPayload, EmbedTemplate, FormSubmission, Giveaway, GiveawayEntry, GuildChannel, GuildConfig, GuildRole, InviteAttributionRecord, InviteLeaderboardRecord, MessageDeletionJob, ModerationCase, NewGiveaway, NewModerationCase, NewTicketPanel, OperationalRecord, PollRecord, Reminder, ResourceDiagnostics, Suggestion, SuggestionStatus, TemporaryVoiceChannel, Ticket, TicketAnalytics, TicketEvent, TicketNote, TicketPanel, VerificationAttemptRecord, VoiceLinkEventRecord } from '../types'
import { demoServers, type ConnectedServer } from './server-data'
import { globalUsers, type GlobalUser, type UserServerContext } from './user-data'
import { discordAccountAge } from './discord'
import { hourlyCommands, type ChartDatum } from './analytics-data'

const configuredApiBase = import.meta.env.VITE_API_BASE_URL as string | undefined
const apiBase = (configuredApiBase === 'same-origin' || !configuredApiBase) ? window.location.origin : configuredApiBase
const demoActorId = '312058643210567680'

// Demo fixtures must be explicitly enabled outside the test runner. Falling back to
// them when an API URL is omitted makes sample guilds look like real installations.
export const isDemoMode = import.meta.env.VITE_DEMO_MODE === 'true' || import.meta.env.MODE === 'test'
export const configuredGuildId = (import.meta.env.VITE_GUILD_ID as string | undefined) ?? demoGuildId

const demoKey = (guildId: string) => `orivane:config:${guildId}`

function demoConfig(guildId: string): GuildConfig {
  const saved = localStorage.getItem(demoKey(guildId))
  if (!saved) return { ...structuredClone(defaultConfig), guild_id: guildId }
  const parsed = JSON.parse(saved) as Partial<GuildConfig>
  const defaults = structuredClone(defaultConfig)
  const modules = Object.fromEntries(Object.entries(defaults.modules).map(([key, value]) => [key, { ...value, ...parsed.modules?.[key as keyof GuildConfig['modules']] }])) as GuildConfig['modules']
  return { ...defaults, ...parsed, guild_id: guildId, modules }
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
    { id: '100000000000000006', name: 'Lounge', kind: 'GuildVoice', position: 5, parent_id: '100000000000000004' },
  ]
  return request<GuildChannel[]>(`/v1/guilds/${guildId}/channels`)
}

export async function getGuildRoles(guildId: string): Promise<GuildRole[]> {
  if (isDemoMode) return [
    { id: '100000000000000101', name: 'Support', position: 3, managed: false },
    { id: '100000000000000102', name: 'Announcements', position: 2, managed: false },
    { id: '100000000000000103', name: 'Member', position: 1, managed: false },
  ]
  return request<GuildRole[]>(`/v1/guilds/${guildId}/roles`)
}

export async function getResourceDiagnostics(guildId: string): Promise<ResourceDiagnostics> {
  if (isDemoMode) return { status: 'healthy', channels_available: true, roles_available: true, issues: [] }
  return request<ResourceDiagnostics>(`/v1/guilds/${guildId}/resources/diagnostics`)
}

export type OperationalModule = 'moderation_appeals'|'polls'|'invite_tracking'|'verification'|'counters'|'afk'|'voice_text_linking'
const demoOperationalNow = '2026-10-12T17:45:00Z'
let demoPolls: PollRecord[] = [{ id: 7, guild_id: demoGuildId, channel_id: '100000000000000005', creator_id: demoActorId, creator_username: 'imlunahey', creator_display_name: 'luna', question: 'Which event should we run next?', options: ['Game night', 'Project showcase'], counts: [18, 12], total_votes: 30, anonymous: false, status: 'active', ends_at: '2026-10-13T17:45:00Z', closed_at: null, created_at: demoOperationalNow }]
let demoAfkStatuses: AfkStatusRecord[] = [{ guild_id: demoGuildId, user_id: '118742006320914432', username: 'silver.arc', display_name: 'Silver', reason: 'Away for dinner', set_at: demoOperationalNow, expires_at: '2026-10-12T20:45:00Z' }]

function demoOperationalRecords(guildId: string, module: OperationalModule): OperationalRecord[] {
  switch (module) {
    case 'moderation_appeals': return demoSubmissions.filter((item) => item.form_id === 'moderation-appeals').map((item) => ({ ...item, guild_id: guildId }))
    case 'polls': return demoPolls.map((item) => ({ ...item, guild_id: guildId }))
    case 'invite_tracking': return [{ id: 3, guild_id: guildId, joined_user_id: '227843117431025543', joined_username: 'northstar', joined_display_name: 'Northstar', invite_code: 'orivane', inviter_id: '118742006320914432', inviter_username: 'silver.arc', inviter_display_name: 'Silver', attributed_at: demoOperationalNow }]
    case 'verification': return [{ id: 4, guild_id: guildId, user_id: '227843117431025543', username: 'northstar', display_name: 'Northstar', outcome: 'verified', reason: null, account_age_days: 820, has_avatar: true, occurred_at: demoOperationalNow }]
    case 'counters': return [{ id: 5, guild_id: guildId, counter_id: 'members', channel_id: '100000000000000006', value: 1284, rendered_name: 'Members: 1,284', outcome: 'success', error: null, occurred_at: demoOperationalNow }]
    case 'afk': return demoAfkStatuses.map((item) => ({ ...item, guild_id: guildId }))
    case 'voice_text_linking': return [{ id: 6, guild_id: guildId, user_id: '118742006320914432', username: 'silver.arc', display_name: 'Silver', voice_channel_id: '100000000000000006', text_channel_id: '100000000000000005', action: 'joined', occurred_at: demoOperationalNow }]
  }
}

export async function getOperationalRecords(guildId:string,module:OperationalModule):Promise<OperationalRecord[]>{
  if(isDemoMode)return demoOperationalRecords(guildId,module)
  switch(module){
    case 'moderation_appeals': return request<FormSubmission[]>(`/v1/guilds/${guildId}/appeals`)
    case 'polls': return request<PollRecord[]>(`/v1/guilds/${guildId}/polls`)
    case 'invite_tracking': return request<InviteAttributionRecord[]>(`/v1/guilds/${guildId}/invites/attributions`)
    case 'verification': return request<VerificationAttemptRecord[]>(`/v1/guilds/${guildId}/verification/attempts`)
    case 'counters': return request<CounterRunRecord[]>(`/v1/guilds/${guildId}/counters/runs`)
    case 'afk': return request<AfkStatusRecord[]>(`/v1/guilds/${guildId}/afk`)
    case 'voice_text_linking': return request<VoiceLinkEventRecord[]>(`/v1/guilds/${guildId}/voice-links/events`)
  }
}
export async function getInviteLeaderboard(guildId:string):Promise<InviteLeaderboardRecord[]>{if(isDemoMode)return [{ inviter_id: '118742006320914432', username: 'silver.arc', display_name: 'Silver', attributed_joins: 14, latest_attribution: demoOperationalNow }];return request(`/v1/guilds/${guildId}/invites/leaderboard`)}
export async function getAppeal(guildId:string,id:string):Promise<FormSubmission>{
  if(isDemoMode){const item=demoSubmissions.find((submission)=>String(submission.id)===id&&submission.form_id==='moderation-appeals');if(!item)throw new Error('Appeal not found');return {...item,guild_id:guildId}}
  return request(`/v1/guilds/${guildId}/appeals/${id}`)
}
export async function reviewAppeal(guildId:string,id:string,status:Exclude<FormSubmission['status'],'submitted'>,note:string):Promise<FormSubmission>{
  if(isDemoMode){const current=await getAppeal(guildId,id);const updated={...current,status,reviewer_id:demoActorId,review_note:note||null,reviewed_at:new Date().toISOString()};demoSubmissions=demoSubmissions.map((item)=>item.id===updated.id&&item.form_id==='moderation-appeals'?updated:item);return updated}
  return request(`/v1/guilds/${guildId}/appeals/${id}`,{method:'PATCH',body:JSON.stringify({status,note:note||null})})
}
export async function getPoll(guildId:string,id:string):Promise<PollRecord>{if(isDemoMode){const item=demoPolls.find((poll)=>String(poll.id)===id);if(!item)throw new Error('Poll not found');return {...item,guild_id:guildId}}return request(`/v1/guilds/${guildId}/polls/${id}`)}
export async function closePoll(guildId:string,id:string):Promise<void>{if(isDemoMode){demoPolls=demoPolls.map((poll)=>String(poll.id)===id?{...poll,status:'closed',closed_at:new Date().toISOString()}:poll);return}return request(`/v1/guilds/${guildId}/polls/${id}/close`,{method:'POST'})}
export async function clearAfkStatus(guildId:string,userId:string):Promise<void>{if(isDemoMode){demoAfkStatuses=demoAfkStatuses.filter((item)=>item.user_id!==userId);return}return request(`/v1/guilds/${guildId}/afk/${userId}`,{method:'DELETE'})}

const temporaryVoiceKey=(guildId:string)=>`orivane:temporary-voice:${guildId}`
export async function getTemporaryVoiceChannels(guildId:string):Promise<TemporaryVoiceChannel[]>{
  if(isDemoMode){const saved=localStorage.getItem(temporaryVoiceKey(guildId));return saved?JSON.parse(saved):[{id:1,guild_id:guildId,channel_id:'100000000000000006',owner_id:'118742006320914432',name:"Silver's channel",user_limit:5,locked:false,status:'active',created_at:new Date(Date.now()-900000).toISOString(),updated_at:new Date().toISOString(),deleted_at:null,delete_after:null}]}
  return request(`/v1/guilds/${guildId}/temporary-voice`)
}
export async function deleteTemporaryVoiceChannel(guildId:string,channelId:string):Promise<void>{
  if(isDemoMode){const channels=await getTemporaryVoiceChannels(guildId);localStorage.setItem(temporaryVoiceKey(guildId),JSON.stringify(channels.filter(channel=>channel.channel_id!==channelId)));return}
  return request(`/v1/guilds/${guildId}/temporary-voice/${channelId}`,{method:'DELETE'})
}

export async function getEconomyAccounts(guildId:string):Promise<EconomyAccount[]>{
  if(isDemoMode)return [{guild_id:guildId,user_id:'118742006320914432',balance:12500,lifetime_earned:18200,lifetime_spent:5700,updated_at:new Date().toISOString()},{guild_id:guildId,user_id:'312058643210567680',balance:8400,lifetime_earned:9100,lifetime_spent:700,updated_at:new Date(Date.now()-3600000).toISOString()}]
  return request(`/v1/guilds/${guildId}/economy/accounts`)
}
export async function getEconomyTransactions(guildId:string):Promise<EconomyTransaction[]>{
  if(isDemoMode)return [{id:1,guild_id:guildId,user_id:'118742006320914432',counterparty_id:null,amount:250,kind:'daily',reference:'daily:demo',metadata:{},created_at:new Date().toISOString()}]
  return request(`/v1/guilds/${guildId}/economy/transactions`)
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

const embedKey = (guildId: string) => `orivane:embeds:${guildId}`
const embedDeliveryKey = (guildId: string) => `orivane:embed-deliveries:${guildId}`
const starterEmbed: EmbedTemplate = { id: 1, guild_id: demoGuildId, name: 'Community rules', payload: { content: '', title: 'Welcome to {server}', description: 'Please read the rules before joining the conversation, {user}.', url: '', color: 5797266, author_name: 'Orivane', author_url: '', author_icon_url: '', footer_text: 'Thanks for helping keep the community welcoming.', footer_icon_url: '', thumbnail_url: '/orivane-avatar.png', image_url: '', timestamp: false, fields: [{ name: 'Be kind', value: 'Treat everyone with respect.', inline: true }, { name: 'Stay on topic', value: 'Use the right channel for each conversation.', inline: true }] }, created_by: demoActorId, created_at: '2026-10-12T12:00:00Z', updated_at: '2026-10-12T12:00:00Z' }
function demoEmbeds(guildId: string): EmbedTemplate[] { const saved=localStorage.getItem(embedKey(guildId)); return saved?JSON.parse(saved) as EmbedTemplate[]:[{...structuredClone(starterEmbed),guild_id:guildId}] }
function demoEmbedDeliveries(guildId: string): EmbedDelivery[] { const saved=localStorage.getItem(embedDeliveryKey(guildId)); return saved ? JSON.parse(saved) as EmbedDelivery[] : [] }

export async function getEmbedTemplates(guildId:string):Promise<EmbedTemplate[]>{if(isDemoMode)return demoEmbeds(guildId);return request(`/v1/guilds/${guildId}/embeds`)}
export async function getEmbedTemplate(guildId:string,templateId:string):Promise<EmbedTemplate>{if(isDemoMode){const item=demoEmbeds(guildId).find(x=>String(x.id)===templateId);if(!item)throw new Error('Embed template not found');return item}return request(`/v1/guilds/${guildId}/embeds/${templateId}`)}
export async function saveEmbedTemplate(input:{guildId:string;templateId?:string;name:string;payload:EmbedPayload}):Promise<EmbedTemplate>{if(isDemoMode){const items=demoEmbeds(input.guildId),now=new Date().toISOString(),existing=items.find(x=>String(x.id)===input.templateId);const saved:EmbedTemplate=existing?{...existing,name:input.name,payload:input.payload,updated_at:now}:{id:Math.max(0,...items.map(x=>x.id))+1,guild_id:input.guildId,name:input.name,payload:input.payload,created_by:demoActorId,created_at:now,updated_at:now};localStorage.setItem(embedKey(input.guildId),JSON.stringify(existing?items.map(x=>x.id===saved.id?saved:x):[saved,...items]));return saved}return request(`/v1/guilds/${input.guildId}/embeds${input.templateId?`/${input.templateId}`:''}`,{method:input.templateId?'PUT':'POST',body:JSON.stringify({name:input.name,payload:input.payload})})}
export async function deleteEmbedTemplate(guildId:string,templateId:string):Promise<void>{if(isDemoMode){localStorage.setItem(embedKey(guildId),JSON.stringify(demoEmbeds(guildId).filter(x=>String(x.id)!==templateId)));return}return request(`/v1/guilds/${guildId}/embeds/${templateId}`,{method:'DELETE'})}
export async function getEmbedDeliveries(guildId:string):Promise<EmbedDelivery[]>{if(isDemoMode)return demoEmbedDeliveries(guildId);return request(`/v1/guilds/${guildId}/embed-deliveries`)}
export async function sendEmbedTemplate(input:{guildId:string;templateId:string;channelId:string}):Promise<EmbedDelivery>{if(isDemoMode){const item:EmbedDelivery={id:Date.now(),guild_id:input.guildId,template_id:Number(input.templateId),channel_id:input.channelId,message_id:String(Date.now()+1),actor_id:demoActorId,action:'send',status:'delivered',error:null,created_at:new Date().toISOString()};localStorage.setItem(embedDeliveryKey(input.guildId),JSON.stringify([item,...demoEmbedDeliveries(input.guildId)]));return item}return request(`/v1/guilds/${input.guildId}/embeds/${input.templateId}/send`,{method:'POST',body:JSON.stringify({channel_id:input.channelId})})}
export async function syncEmbedDelivery(input:{guildId:string;templateId:string;deliveryId:string}):Promise<EmbedDelivery>{if(isDemoMode){const previous=demoEmbedDeliveries(input.guildId).find(x=>String(x.id)===input.deliveryId);if(!previous)throw new Error('Embed delivery not found');const item={...previous,id:Date.now(),action:'edit' as const,created_at:new Date().toISOString()};localStorage.setItem(embedDeliveryKey(input.guildId),JSON.stringify([item,...demoEmbedDeliveries(input.guildId)]));return item}return request(`/v1/guilds/${input.guildId}/embeds/${input.templateId}/deliveries/${input.deliveryId}/sync`,{method:'POST'})}

let demoSubmissions: FormSubmission[] = [{
  id: 1,
  guild_id: demoGuildId,
  form_id: 'staff-application',
  submitter_id: '118742006320914432',
  username: 'silver.arc',
  display_name: 'Silver',
  answers: [{ question_id: 'experience', label: 'What relevant experience do you have?', answer: 'I have moderated community servers for several years.' }],
  status:'submitted',reviewer_id:null,review_note:null,reviewed_at:null,
  created_at: '2026-10-12T15:18:00Z',
},{id:2,guild_id:demoGuildId,form_id:'moderation-appeals',submitter_id:'118742006320914432',username:'silver.arc',display_name:'Silver',answers:[{question_id:'case',label:'Case or action',answer:'Case #42'},{question_id:'reason',label:'Appeal',answer:'Please review the context for this warning.'}],status:'submitted',reviewer_id:null,review_note:null,reviewed_at:null,created_at:'2026-10-12T16:18:00Z'}]

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
export async function reviewFormSubmission(input:{guildId:string;formId:string;submissionId:string;status:'approved'|'rejected'|'closed';note:string|null}):Promise<FormSubmission>{if(isDemoMode){const current=await getFormSubmission(input.guildId,input.formId,input.submissionId);const updated={...current,status:input.status,reviewer_id:'784365843810222080',review_note:input.note,reviewed_at:new Date().toISOString()};demoSubmissions=demoSubmissions.map(item=>item.id===updated.id&&item.form_id===updated.form_id?updated:item);return updated}return request<FormSubmission>(`/v1/guilds/${input.guildId}/forms/${encodeURIComponent(input.formId)}/submissions/${input.submissionId}`,{method:'PATCH',headers:{'content-type':'application/json'},body:JSON.stringify({status:input.status,note:input.note})})}

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

const ticketPanelKey=(guildId:string)=>`orivane:ticket-panels:${guildId}`;const ticketKey=(guildId:string)=>`orivane:tickets:${guildId}`
function demoTicketPanels(guildId:string):TicketPanel[]{const saved=localStorage.getItem(ticketPanelKey(guildId));if(saved)return JSON.parse(saved) as TicketPanel[];return [{id:1,guild_id:guildId,name:'General support',title:'Need help?',description:'Open a private ticket and our team will get back to you.',panel_channel_id:'100000000000000002',panel_message_id:'demo-panel',category_id:'100000000000000001',support_role_ids:['784365843810222080'],button_label:'Open ticket',welcome_message:'Welcome {user}. {support_roles} will be with you shortly.',channel_name:'ticket-{user_id}',intake_enabled:true,intake_label:'What can we help with?',max_open_per_user:1,auto_close_hours:72,transcript_retention_days:90,default_priority:'normal',enabled:true,created_by:demoActorId,created_at:new Date(Date.now()-86400000).toISOString(),updated_at:new Date().toISOString()}]}
function demoTickets(guildId:string):Ticket[]{const saved=localStorage.getItem(ticketKey(guildId));if(saved)return JSON.parse(saved) as Ticket[];const opened=new Date(Date.now()-3600000).toISOString();return [{id:42,guild_id:guildId,panel_id:1,opener_id:'118742006320914432',opener_username:'silver.arc',opener_display_name:'Silver',channel_id:'100000000000000005',subject:'Verification help',opening_note:'I cannot complete verification.',status:'open',claimed_by:null,assigned_to:null,closed_by:null,closed_reason:null,transcript:null,priority:'normal',tags:[],first_response_at:null,last_activity_at:opened,opened_at:opened,closed_at:null,reopened_at:null,updated_at:opened}]}
export async function getTicketPanels(guildId:string):Promise<TicketPanel[]>{if(isDemoMode)return demoTicketPanels(guildId);return request(`/v1/guilds/${guildId}/ticket-panels`)}
export async function getTicketPanel(guildId:string,id:string):Promise<TicketPanel>{const item=(await getTicketPanels(guildId)).find(x=>String(x.id)===id);if(!item)throw new Error('Ticket panel not found');return item}
export async function saveTicketPanel(input:{guildId:string;panelId?:string;panel:NewTicketPanel}):Promise<TicketPanel>{if(isDemoMode){const items=demoTicketPanels(input.guildId),old=items.find(x=>String(x.id)===input.panelId),now=new Date().toISOString(),saved:TicketPanel=old?{...old,...input.panel,panel_channel_id:input.panel.channel_id,updated_at:now}:{...input.panel,id:Math.max(0,...items.map(x=>x.id))+1,guild_id:input.guildId,panel_channel_id:input.panel.channel_id,panel_message_id:null,created_by:demoActorId,created_at:now,updated_at:now};localStorage.setItem(ticketPanelKey(input.guildId),JSON.stringify(old?items.map(x=>x.id===saved.id?saved:x):[saved,...items]));return saved}return request(`/v1/guilds/${input.guildId}/ticket-panels${input.panelId?`/${input.panelId}`:''}`,{method:input.panelId?'PUT':'POST',body:JSON.stringify(input.panel)})}
export async function deleteTicketPanel(guildId:string,id:string):Promise<void>{if(isDemoMode){localStorage.setItem(ticketPanelKey(guildId),JSON.stringify(demoTicketPanels(guildId).filter(x=>String(x.id)!==id)));return}return request(`/v1/guilds/${guildId}/ticket-panels/${id}`,{method:'DELETE'})}
export async function deployTicketPanel(guildId:string,id:string):Promise<TicketPanel>{if(isDemoMode){const item=await getTicketPanel(guildId,id);return {...item,panel_message_id:`demo-${Date.now()}`}}return request(`/v1/guilds/${guildId}/ticket-panels/${id}/deploy`,{method:'POST'})}
export async function getTickets(guildId:string):Promise<Ticket[]>{if(isDemoMode)return demoTickets(guildId);return request(`/v1/guilds/${guildId}/tickets`)}
export async function getTicket(guildId:string,id:string):Promise<Ticket>{if(isDemoMode){const item=demoTickets(guildId).find(x=>String(x.id)===id);if(!item)throw new Error('Ticket not found');return item}return request(`/v1/guilds/${guildId}/tickets/${id}`)}
export async function getTicketEvents(guildId:string,id:string):Promise<TicketEvent[]>{if(isDemoMode)return [{id:1,ticket_id:Number(id),guild_id:guildId,actor_id:'118742006320914432',action:'opened',metadata:{},created_at:new Date(Date.now()-3600000).toISOString()}];return request(`/v1/guilds/${guildId}/tickets/${id}/events`)}
export async function changeTicket(input:{guildId:string;ticketId:string;action:'claim'|'close'|'reopen'|'transfer';reason?:string;user_id?:string}):Promise<Ticket>{if(isDemoMode){const items=demoTickets(input.guildId),item=items.find(x=>String(x.id)===input.ticketId);if(!item)throw new Error('Ticket not found');const updated={...item,status:input.action==='close'?'closed' as const:input.action==='reopen'?'open' as const:item.status,claimed_by:input.action==='claim'?demoActorId:item.claimed_by,assigned_to:input.action==='transfer'?(input.user_id??null):input.action==='claim'?demoActorId:item.assigned_to,closed_reason:input.action==='close'?(input.reason||'Closed from dashboard'):input.action==='reopen'?null:item.closed_reason,updated_at:new Date().toISOString()};localStorage.setItem(ticketKey(input.guildId),JSON.stringify(items.map(x=>x.id===updated.id?updated:x)));return updated}return request(`/v1/guilds/${input.guildId}/tickets/${input.ticketId}/${input.action}`,{method:'POST',body:JSON.stringify({reason:input.reason??'',user_id:input.user_id})})}
export async function getTicketAnalytics(guildId:string):Promise<TicketAnalytics>{if(isDemoMode){const items=demoTickets(guildId);return{total:items.length,open:items.filter(x=>x.status==='open').length,closed:items.filter(x=>x.status==='closed').length,unassigned:items.filter(x=>x.status==='open'&&!x.assigned_to).length,overdue:0,median_first_response_minutes:null,median_resolution_minutes:null}}return request(`/v1/guilds/${guildId}/tickets/analytics`)}
export async function getTicketNotes(guildId:string,id:string):Promise<TicketNote[]>{if(isDemoMode)return[];return request(`/v1/guilds/${guildId}/tickets/${id}/notes`)}
export async function addTicketNote(input:{guildId:string;ticketId:string;body:string}):Promise<TicketNote>{if(isDemoMode)return{id:Date.now(),ticket_id:Number(input.ticketId),guild_id:input.guildId,author_id:demoActorId,body:input.body,created_at:new Date().toISOString()};return request(`/v1/guilds/${input.guildId}/tickets/${input.ticketId}/notes`,{method:'POST',body:JSON.stringify({body:input.body})})}
export async function updateTicketMetadata(input:{guildId:string;ticketId:string;priority:Ticket['priority'];tags:string[]}):Promise<Ticket>{if(isDemoMode){const items=demoTickets(input.guildId),item=items.find(x=>String(x.id)===input.ticketId);if(!item)throw new Error('Ticket not found');const updated={...item,priority:input.priority,tags:input.tags,updated_at:new Date().toISOString()};localStorage.setItem(ticketKey(input.guildId),JSON.stringify(items.map(x=>x.id===updated.id?updated:x)));return updated}return request(`/v1/guilds/${input.guildId}/tickets/${input.ticketId}`,{method:'PATCH',body:JSON.stringify({priority:input.priority,tags:input.tags})})}
export function ticketTranscriptUrl(guildId:string,id:string):string{return `${apiBase??''}/v1/guilds/${guildId}/tickets/${id}/transcript`}
export async function deployReactionRolePanel(guildId:string,panelId:string):Promise<GuildConfig>{if(isDemoMode){const config=await getGuildConfig(guildId),panels=config.modules.reaction_roles.panels.map(panel=>panel.id===panelId?{...panel,message_id:panel.message_id??`demo-${Date.now()}`}:panel),updated={...config,modules:{...config.modules,reaction_roles:{...config.modules.reaction_roles,panels}}};localStorage.setItem(demoKey(guildId),JSON.stringify(updated));return updated}return request(`/v1/guilds/${guildId}/reaction-role-panels/${encodeURIComponent(panelId)}/deploy`,{method:'POST'})}

export async function getAuditEvents(guildId?: string): Promise<AuditEvent[]> {
  if (isDemoMode) {
    await new Promise((resolve) => setTimeout(resolve, 120))
    const events = [...storedDemoAuditEvents(), ...demoAuditEvents].sort((a, b) => b.occurred_at.localeCompare(a.occurred_at))
    return guildId ? events.filter((event) => event.guild_id === guildId) : events
  }
  return request<AuditEvent[]>(guildId ? `/v1/guilds/${guildId}/audit?limit=200` : '/v1/audit?limit=200')
}

export async function getUserAuditEvents(userId: string, guildId?: string, beforeId?: number): Promise<AuditEvent[]> {
  if (isDemoMode) {
    const events = await getAuditEvents(guildId)
    return events.filter((event) => (event.actor_id === userId || event.target_id === userId) && (!beforeId || event.id < beforeId)).slice(0, 25)
  }
  const query = new URLSearchParams({ limit: '25', user_id: userId })
  if (beforeId) query.set('before_id', String(beforeId))
  return request<AuditEvent[]>(`${guildId ? `/v1/guilds/${guildId}/audit` : '/v1/audit'}?${query}`)
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
  { id: 187, guild_id: configuredGuildId, member_id: '118742006320914432', moderator_id: demoActorId, action: 'warn', reason: 'Repeated link spam', evidence_urls: ['https://discord.com/channels/901234567890123456/100000000000000005/200000000000000001'], status: 'open', created_at: '2026-10-11T19:52:00Z', updated_at: '2026-10-11T19:52:00Z' },
  { id: 186, guild_id: configuredGuildId, member_id: '227843117431025543', moderator_id: 'automod', action: 'timeout', reason: 'Mention flood', evidence_urls: [], status: 'open', created_at: '2026-10-11T19:26:00Z', updated_at: '2026-10-11T19:26:00Z' },
]

function demoCases(guildId: string): ModerationCase[] {
  const saved = localStorage.getItem(`orivane:cases:${guildId}`)
  return saved ? JSON.parse(saved) as ModerationCase[] : starterCases.map((item) => ({ ...item, guild_id: guildId }))
}

export async function getModerationCases(guildId: string, options: { memberId?: string; beforeId?: number; limit?: number } = {}): Promise<ModerationCase[]> {
  if (isDemoMode) return demoCases(guildId)
    .filter((item) => (!options.memberId || item.member_id === options.memberId) && (!options.beforeId || item.id < options.beforeId))
    .slice(0, options.limit ?? 200)
  const query = new URLSearchParams({ limit: String(options.limit ?? 200) })
  if (options.memberId) query.set('member_id', options.memberId)
  if (options.beforeId) query.set('before_id', String(options.beforeId))
  return request<ModerationCase[]>(`/v1/guilds/${guildId}/cases?${query}`)
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
    const item: ModerationCase = { id: Math.max(0, ...cases.map(({ id }) => id)) + 1, guild_id: guildId, moderator_id: demoActorId, status: 'open', created_at: now, updated_at: now, ...input, evidence_urls: input.evidence_urls ?? [] }
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

type UserResponse = { id: string; username: string | null; display_name: string | null; avatar_url: string | null; servers: number; installed_servers: number; install_commands_30d: number; commands: number; cases: number; last_seen: string | null }
type UserContextResponse = { server_id: string; server_name: string; relationship: UserServerContext['relationship']; commands: number; cases: number; last_seen: string | null }

function displayUser(user: UserResponse, contexts: UserServerContext[] = []): GlobalUser {
  const username = user.username ?? user.id
  return {
    id: user.id,
    username,
    displayName: user.display_name ?? (user.username ? user.username : `Discord user ${user.id.slice(-4)}`),
    avatarUrl: user.avatar_url,
    kind: user.installed_servers ? 'Installer' : 'Member',
    servers: user.servers,
    installedServers: user.installed_servers,
    installCommands30d: user.install_commands_30d,
    commands: user.commands,
    cases: user.cases,
    lastSeen: user.last_seen ? new Date(user.last_seen).toLocaleString() : 'No activity recorded',
    accountAge: discordAccountAge(user.id),
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
  avatar_url: string | null
  commands_30d: number
  cases: number
  last_seen: string | null
}

export type GuildMemberDetail = GuildMemberSummary & {
  bot: boolean | null
  joined_at: string | null
  role_ids: string[]
  roles: GuildRole[]
  pending: boolean | null
  discord_profile_status: 'live' | 'unavailable'
  tracked_messages: number
  total_xp: number
  xp_rank: number
}

export async function getGuildMembers(guildId: string): Promise<GuildMemberSummary[]> {
  if (isDemoMode) return []
  return request<GuildMemberSummary[]>(`/v1/guilds/${guildId}/members`)
}

export async function getGuildMember(guildId: string, memberId: string): Promise<GuildMemberDetail> {
  if (isDemoMode) {
    const user = globalUsers.find((item) => item.id === memberId)
    return {
      id: memberId,
      username: user?.username ?? `member-${memberId.slice(-4)}`,
      display_name: user?.displayName ?? `Discord user ${memberId.slice(-4)}`,
      avatar_url: user?.avatarUrl ?? null,
      bot: false,
      joined_at: '2026-08-12T15:05:00.000Z',
      role_ids: ['100000000000000103'],
      roles: [{ id: '100000000000000103', name: 'Member', position: 1, managed: false }],
      pending: false,
      discord_profile_status: 'live',
      commands_30d: user?.commands ?? 0,
      cases: user?.cases ?? 0,
      last_seen: new Date().toISOString(),
      tracked_messages: 184,
      total_xp: 2_400,
      xp_rank: 12,
    }
  }
  return request<GuildMemberDetail>(`/v1/guilds/${guildId}/members/${memberId}`)
}

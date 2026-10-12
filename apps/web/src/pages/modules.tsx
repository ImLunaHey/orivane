import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query'
import { Link, useParams } from '@tanstack/react-router'
import { ArrowLeft, Bot, Check, Gavel, LoaderCircle, ScrollText, ShieldCheck, Sparkles, Star, TerminalSquare, Ticket, Trophy, UserPlus } from 'lucide-react'
import { useEffect, useState } from 'react'
import { DiscordChannelSelect } from '../components/discord-channel-select'
import { Badge, Button, Panel, Toggle } from '../components/ui'
import { WelcomeMessageEditor } from '../components/welcome-message-editor'
import { getGuildChannels, getGuildConfig, getServers, saveGuildConfig } from '../lib/api'
import type { GuildChannel, GuildConfig } from '../types'

type ModuleKey = keyof GuildConfig['modules']
type ModuleMeta = { title: string; description: string; icon: typeof Bot; tone: string; runtime: 'Live' | 'Configuration only' }

const moduleMeta: Record<ModuleKey, ModuleMeta> = {
  utility: { title: 'Utility', description: 'Ping, help, and server information slash commands.', icon: TerminalSquare, tone: 'blue', runtime: 'Live' },
  welcome: { title: 'Welcome', description: 'Greet new members with server-specific messages.', icon: Sparkles, tone: 'blue', runtime: 'Live' },
  autorole: { title: 'Autorole', description: 'Assign configured roles when a member joins.', icon: UserPlus, tone: 'green', runtime: 'Live' },
  logging: { title: 'Server logging', description: 'Send accountable command outcomes to a channel.', icon: ScrollText, tone: 'green', runtime: 'Live' },
  moderation: { title: 'Moderation', description: 'Cases and defaults for warn, timeout, kick, and ban.', icon: Gavel, tone: 'violet', runtime: 'Configuration only' },
  automod: { title: 'Automod', description: 'Block spam, mention floods, and unwanted terms.', icon: ShieldCheck, tone: 'violet', runtime: 'Configuration only' },
  leveling: { title: 'Leveling', description: 'Reward healthy participation with XP and ranks.', icon: Trophy, tone: 'amber', runtime: 'Configuration only' },
  starboard: { title: 'Starboard', description: 'Promote popular posts into a highlight channel.', icon: Star, tone: 'amber', runtime: 'Configuration only' },
  tickets: { title: 'Tickets', description: 'Private support channels with transcripts and roles.', icon: Ticket, tone: 'blue', runtime: 'Configuration only' },
  custom_commands: { title: 'Custom commands', description: 'Server-owned slash responses with safe limits.', icon: Bot, tone: 'violet', runtime: 'Configuration only' },
}

export function ModulesPage() {
  const { serverId } = useParams({ strict: false }) as { serverId: string }
  const client = useQueryClient()
  const queryKey = ['guild-config', serverId] as const
  const { data, isLoading } = useQuery({ queryKey, queryFn: () => getGuildConfig(serverId) })
  const [draft, setDraft] = useState<GuildConfig | null>(null)
  const mutation = useMutation({ mutationFn: saveGuildConfig, onSuccess: (saved) => { client.setQueryData(queryKey, saved); setDraft(saved) } })
  useEffect(() => { setDraft(data ?? null) }, [data, serverId])

  if (isLoading || !draft) return <LoadingState />
  const updateModule = <K extends ModuleKey>(key: K, value: GuildConfig['modules'][K]) => setDraft({ ...draft, modules: { ...draft.modules, [key]: value } })
  const hasChanges = JSON.stringify(data) !== JSON.stringify(draft)

  return <div className="page-stack">
    <section className="page-intro"><div><p className="eyebrow">Configuration</p><h2>Modules</h2><p>Enable only the features this server needs. Every saved change is validated, audited, and picked up through the shared configuration cache.</p></div><div className="save-row">{mutation.isSuccess && !hasChanges && <span className="saved-label"><Check size={15} />Saved</span>}<Button disabled={!hasChanges || mutation.isPending} onClick={() => mutation.mutate(draft)}>{mutation.isPending && <LoaderCircle className="spin" size={16} />}Save changes</Button></div></section>
    {mutation.isError && <div className="error-banner" role="alert">{mutation.error.message}</div>}
    <section className="module-grid">
      {(Object.keys(moduleMeta) as ModuleKey[]).map((key) => {
        const meta = moduleMeta[key]
        const Icon = meta.icon
        const module = draft.modules[key]
        return <Panel key={key} className={`module-card ${module.enabled ? 'module-enabled' : ''}`}>
          <div className="module-top"><span className={`module-icon ${meta.tone}`}><Icon size={21} /></span><Toggle label={`Enable ${meta.title}`} checked={module.enabled} onChange={(enabled) => updateModule(key, { ...module, enabled } as GuildConfig['modules'][typeof key])} /></div>
          <Link className="module-card-main" to="/servers/$serverId/modules/$moduleId" params={{ serverId, moduleId: key }}><div className="module-title-row"><h3>{meta.title}</h3><Badge tone={meta.runtime === 'Live' ? 'green' : 'neutral'}>{meta.runtime}</Badge></div><p>{meta.description}</p></Link>
          <div className="module-status"><Badge tone={module.enabled ? 'green' : 'neutral'}>{module.enabled ? 'Enabled' : 'Disabled'}</Badge><Link to="/servers/$serverId/modules/$moduleId" params={{ serverId, moduleId: key }}>View details →</Link></div>
        </Panel>
      })}
    </section>
  </div>
}

export function ModuleDetailPage() {
  const { serverId, moduleId } = useParams({ strict: false }) as { serverId: string; moduleId: string }
  const moduleKey = moduleId as ModuleKey
  const client = useQueryClient()
  const queryKey = ['guild-config', serverId] as const
  const { data, isLoading } = useQuery({ queryKey, queryFn: () => getGuildConfig(serverId) })
  const channels = useQuery({ queryKey: ['guild-channels', serverId], queryFn: () => getGuildChannels(serverId) })
  const servers = useQuery({ queryKey: ['servers'], queryFn: getServers })
  const [draft, setDraft] = useState<GuildConfig | null>(null)
  const mutation = useMutation({ mutationFn: saveGuildConfig, onSuccess: (saved) => { client.setQueryData(queryKey, saved); setDraft(saved) } })
  useEffect(() => { setDraft(data ?? null) }, [data, serverId])

  if (!(moduleKey in moduleMeta)) return <div className="empty-state"><h2>Module not found</h2><Link to="/servers/$serverId/modules" params={{ serverId }}>Return to modules</Link></div>
  if (isLoading || !draft) return <LoadingState />

  const meta = moduleMeta[moduleKey]
  const Icon = meta.icon
  const module = draft.modules[moduleKey]
  const updateModule = <K extends ModuleKey>(key: K, value: GuildConfig['modules'][K]) => setDraft({ ...draft, modules: { ...draft.modules, [key]: value } })
  const hasChanges = JSON.stringify(data) !== JSON.stringify(draft)

  return <div className="page-stack">
    <Link className="back-link" to="/servers/$serverId/modules" params={{ serverId }}><ArrowLeft size={16} />All modules</Link>
    <section className="page-intro module-detail-intro"><div><p className="eyebrow">{meta.runtime} module</p><h2><span className={`module-icon ${meta.tone}`}><Icon size={21} /></span>{meta.title}</h2><p>{meta.description}</p></div><div className="save-row"><Toggle label={`Enable ${meta.title}`} checked={module.enabled} onChange={(enabled) => updateModule(moduleKey, { ...module, enabled } as GuildConfig['modules'][typeof moduleKey])} />{mutation.isSuccess && !hasChanges && <span className="saved-label"><Check size={15} />Saved</span>}<Button disabled={!hasChanges || mutation.isPending} onClick={() => mutation.mutate(draft)}>{mutation.isPending && <LoaderCircle className="spin" size={16} />}Save changes</Button></div></section>
    {mutation.isError && <div className="error-banner" role="alert">{mutation.error.message}</div>}
    <Panel className="settings-panel">
      <div className="panel-heading"><div><p className="eyebrow">Configuration</p><h3>{meta.title} settings</h3></div><Badge tone={module.enabled ? 'green' : 'neutral'}>{module.enabled ? 'Active' : 'Disabled'}</Badge></div>
      {channels.isError && <div className="error-banner" role="alert"><span>Discord channels could not be loaded.</span><Button onClick={() => channels.refetch()}>Retry</Button></div>}
      <ModuleSettings moduleKey={moduleKey} config={draft} update={updateModule} channels={channels.data ?? []} channelsPending={channels.isLoading} channelsError={channels.isError} serverName={servers.data?.find((server) => server.id === serverId)?.name ?? `Server ${serverId.slice(-4)}`} />
      {meta.runtime === 'Configuration only' && <div className="module-readiness"><strong>Not active in Discord yet</strong><span>These settings can be saved and audited, but this module does not currently process Discord events or slash commands.</span></div>}
    </Panel>
  </div>
}

function ModuleSettings({ moduleKey, config, update, channels, channelsPending, channelsError, serverName }: { moduleKey: ModuleKey; config: GuildConfig; update: <K extends ModuleKey>(key: K, value: GuildConfig['modules'][K]) => void; channels: GuildChannel[]; channelsPending: boolean; channelsError: boolean; serverName: string }) {
  const modules = config.modules
  switch (moduleKey) {
    case 'utility': return <div className="form-grid"><ToggleField title="Ephemeral responses" detail="Only the invoking member sees utility command replies" checked={modules.utility.ephemeral_responses} onChange={(value) => update('utility', { ...modules.utility, ephemeral_responses: value })} /></div>
    case 'welcome': return <div className="form-grid"><DiscordChannelSelect title="Welcome channel" detail="Where new-member greetings are sent" value={modules.welcome.channel_id} kind="text" channels={channels} pending={channelsPending} error={channelsError} onChange={(value) => update('welcome', { ...modules.welcome, channel_id: value })} /><WelcomeMessageEditor value={modules.welcome.message} serverId={config.guild_id} serverName={serverName} onChange={(value) => update('welcome', { ...modules.welcome, message: value })} /></div>
    case 'autorole': return <div className="form-grid"><TextField wide title="Role IDs" detail="Comma-separated; up to 10 roles" value={modules.autorole.role_ids.join(', ')} onChange={(value) => update('autorole', { ...modules.autorole, role_ids: list(value) })} /><NumberField title="Delay" detail="Seconds after a member joins" min={0} max={3600} value={modules.autorole.delay_seconds} onChange={(value) => update('autorole', { ...modules.autorole, delay_seconds: value })} /></div>
    case 'logging': return <div className="form-grid"><DiscordChannelSelect title="Logging channel" detail="Command outcomes and module actions" value={modules.logging.channel_id} kind="text" channels={channels} pending={channelsPending} error={channelsError} onChange={(value) => update('logging', { ...modules.logging, channel_id: value })} /></div>
    case 'moderation': return <div className="form-grid"><ToggleField title="DM members" detail="Notify members when a moderation action is applied" checked={modules.moderation.dm_on_action} onChange={(value) => update('moderation', { ...modules.moderation, dm_on_action: value })} /><TextField wide title="Default reason" detail="Used when a moderator omits a reason" value={modules.moderation.default_reason} onChange={(value) => update('moderation', { ...modules.moderation, default_reason: value })} /></div>
    case 'automod': return <div className="form-grid"><NumberField title="Maximum mentions" detail="Allowed in one message" min={1} max={50} value={modules.automod.max_mentions} onChange={(value) => update('automod', { ...modules.automod, max_mentions: value })} /><TextField wide title="Blocked terms" detail="Comma-separated; message content is never retained in analytics" value={modules.automod.blocked_words.join(', ')} onChange={(value) => update('automod', { ...modules.automod, blocked_words: list(value) })} /></div>
    case 'leveling': return <div className="form-grid"><NumberField title="Minimum XP" detail="Lowest award per eligible message" min={1} max={1000} value={modules.leveling.xp_per_message_min} onChange={(value) => update('leveling', { ...modules.leveling, xp_per_message_min: value })} /><NumberField title="Maximum XP" detail="Highest award per eligible message" min={1} max={1000} value={modules.leveling.xp_per_message_max} onChange={(value) => update('leveling', { ...modules.leveling, xp_per_message_max: value })} /><NumberField title="XP cooldown" detail="Seconds between awards" min={1} max={3600} value={modules.leveling.cooldown_seconds} onChange={(value) => update('leveling', { ...modules.leveling, cooldown_seconds: value })} /></div>
    case 'starboard': return <div className="form-grid"><DiscordChannelSelect title="Starboard channel" detail="Destination for highlighted messages" value={modules.starboard.channel_id} kind="text" channels={channels} pending={channelsPending} error={channelsError} onChange={(value) => update('starboard', { ...modules.starboard, channel_id: value })} /><TextField title="Reaction emoji" detail="Unicode emoji used for nominations" value={modules.starboard.emoji} onChange={(value) => update('starboard', { ...modules.starboard, emoji: value })} /><NumberField title="Reaction threshold" detail="Required reactions before posting" min={1} max={100} value={modules.starboard.threshold} onChange={(value) => update('starboard', { ...modules.starboard, threshold: value })} /></div>
    case 'tickets': return <div className="form-grid"><DiscordChannelSelect title="Ticket category" detail="New private channels are created here" value={modules.tickets.category_id} kind="category" channels={channels} pending={channelsPending} error={channelsError} onChange={(value) => update('tickets', { ...modules.tickets, category_id: value })} /><DiscordChannelSelect title="Transcript channel" detail="Optional destination for closed-ticket transcripts" value={modules.tickets.transcript_channel_id} kind="text" channels={channels} pending={channelsPending} error={channelsError} onChange={(value) => update('tickets', { ...modules.tickets, transcript_channel_id: value })} /><TextField wide title="Support role IDs" detail="Comma-separated roles that can access tickets" value={modules.tickets.support_role_ids.join(', ')} onChange={(value) => update('tickets', { ...modules.tickets, support_role_ids: list(value) })} /></div>
    case 'custom_commands': return <div className="form-grid"><NumberField title="Command limit" detail="Maximum server-owned commands" min={1} max={100} value={modules.custom_commands.max_commands} onChange={(value) => update('custom_commands', { ...modules.custom_commands, max_commands: value })} /></div>
  }
}

function TextField({ title, detail, value, onChange, wide = false }: { title: string; detail: string; value: string; onChange: (value: string) => void; wide?: boolean }) { return <label className={wide ? 'wide' : undefined}><span>{title}</span><small>{detail}</small><input value={value} onChange={(event) => onChange(event.target.value)} /></label> }
function NumberField({ title, detail, value, min, max, onChange }: { title: string; detail: string; value: number; min: number; max: number; onChange: (value: number) => void }) { return <label><span>{title}</span><small>{detail}</small><input type="number" min={min} max={max} value={value} onChange={(event) => onChange(Number(event.target.value))} /></label> }
function ToggleField({ title, detail, checked, onChange }: { title: string; detail: string; checked: boolean; onChange: (value: boolean) => void }) { return <div className="module-toggle-field"><span><strong>{title}</strong><small>{detail}</small></span><Toggle label={title} checked={checked} onChange={onChange} /></div> }
function list(value: string) { return value.split(',').map((item) => item.trim()).filter(Boolean) }
function LoadingState() { return <div className="loading-state"><LoaderCircle className="spin" /><span>Loading configuration…</span></div> }

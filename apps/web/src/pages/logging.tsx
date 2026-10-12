import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query'
import { Link, useParams } from '@tanstack/react-router'
import type { ColumnDef } from '@tanstack/react-table'
import { ArrowLeft, Check, LoaderCircle } from 'lucide-react'
import { useEffect, useMemo, useState } from 'react'
import { DataTable } from '../components/data-table'
import { DiscordChannelMultiSelect } from '../components/discord-channel-multi-select'
import { DiscordChannelSelect } from '../components/discord-channel-select'
import { LoggingCategoryCard } from '../components/logging-category-card'
import { MetricChart } from '../components/metric-chart'
import { Badge, Button, Panel, SectionHeader, Toggle } from '../components/ui'
import { getAuditEvents, getGuildChannels, getGuildConfig, saveGuildConfig } from '../lib/api'
import type { AuditEvent, GuildConfig, LoggingConfig } from '../types'

type Category = {
  id: string
  title: string
  detail: string
  enabledKey: keyof LoggingConfig
  channelKey?: keyof LoggingConfig
}

const categories: Category[] = [
  { id: 'commands', title: 'Commands and modules', detail: 'Slash-command outcomes and module actions.', enabledKey: 'command_events', channelKey: 'command_channel_id' },
  { id: 'messages', title: 'Messages', detail: 'Edits, single deletions, and bulk deletions without retaining content.', enabledKey: 'message_events', channelKey: 'message_channel_id' },
  { id: 'members', title: 'Members', detail: 'Joins, departures, profile changes, role snapshots, and timeouts.', enabledKey: 'member_events', channelKey: 'member_channel_id' },
  { id: 'moderation', title: 'Moderation', detail: 'Attributed Discord audit-log actions, bans, kicks, and permission changes.', enabledKey: 'moderation_events', channelKey: 'moderation_channel_id' },
  { id: 'roles', title: 'Roles', detail: 'Attributed role creation, updates, deletion, and member-role changes.', enabledKey: 'role_events', channelKey: 'role_channel_id' },
  { id: 'channels', title: 'Channels and threads', detail: 'Attributed channel, thread, and overwrite administration.', enabledKey: 'channel_events', channelKey: 'channel_event_channel_id' },
  { id: 'voice', title: 'Voice', detail: 'Member joins, moves, and disconnects; mute toggles are excluded.', enabledKey: 'voice_events', channelKey: 'voice_channel_id' },
]

export function LoggingPage() {
  const { serverId } = useParams({ strict: false }) as { serverId: string }
  const queryClient = useQueryClient()
  const configKey = ['guild-config', serverId] as const
  const config = useQuery({ queryKey: configKey, queryFn: () => getGuildConfig(serverId) })
  const channels = useQuery({ queryKey: ['guild-channels', serverId], queryFn: () => getGuildChannels(serverId) })
  const audits = useQuery({ queryKey: ['audit-events', serverId], queryFn: () => getAuditEvents(serverId) })
  const [draft, setDraft] = useState<GuildConfig | null>(null)
  useEffect(() => setDraft(config.data ?? null), [config.data, serverId])
  const save = useMutation({ mutationFn: saveGuildConfig, onSuccess: (saved) => { queryClient.setQueryData(configKey, saved); setDraft(saved) } })
  const events = useMemo(() => (audits.data ?? []).filter((event) => event.action.startsWith('discord.') || event.action.startsWith('slash_command.') || event.action.startsWith('module.')), [audits.data])
  const columns = useMemo<Array<ColumnDef<AuditEvent>>>(() => [
    { accessorKey: 'occurred_at', header: 'Time', cell: ({ getValue }) => new Date(getValue<string>()).toLocaleString() },
    { accessorKey: 'action', header: 'Event', cell: ({ row }) => <span className="audit-action"><strong>{friendlyAction(row.original.action)}</strong><small>{eventCategory(row.original)}</small></span> },
    { accessorKey: 'actor_id', header: 'Actor', cell: ({ row }) => row.original.actor_kind === 'user' ? <Link className="table-user-link" to="/users/$userId" params={{ userId: row.original.actor_id }}>Discord user {row.original.actor_id.slice(-4)}<small>{row.original.actor_id}</small></Link> : <span>Orivane system</span> },
    { accessorKey: 'target_type', header: 'Target', cell: ({ row }) => <span className="audit-target"><strong>{row.original.target_type}</strong><small>{row.original.target_id ?? 'No target ID'}</small></span> },
    { id: 'details', header: 'Actions', cell: ({ row }) => <Link className="text-link" to="/servers/$serverId/audit/$eventId" params={{ serverId, eventId: String(row.original.id) }}>Inspect →</Link> },
  ], [serverId])

  if (!draft) return <div className="loading-state"><LoaderCircle className="spin" />Loading logging configuration…</div>
  const logging = draft.modules.logging
  const changed = JSON.stringify(config.data) !== JSON.stringify(draft)
  const update = (next: LoggingConfig) => setDraft({ ...draft, modules: { ...draft.modules, logging: next } })
  const chart = categories.map((category) => {
    const count = events.filter((event) => eventCategory(event) === category.id).length
    return { label: category.id, value: count, detail: `${count} recent ${category.title.toLowerCase()} events are available for inspection.` }
  })
  const attributed = events.filter((event) => event.actor_kind === 'user').length
  const failures = events.filter((event) => event.outcome === 'failure').length

  return <div className="page-stack logging-page">
    <Link className="back-link" to="/servers/$serverId/modules" params={{ serverId }}><ArrowLeft size={16} />All modules</Link>
    <section className="page-intro"><div><p className="eyebrow">Server accountability</p><h2>Discord event logging</h2><p>Route operational events by category, retain safe metadata, and inspect every durable record.</p></div><Badge tone="green">Live gateway events</Badge></section>
    <section className="forms-summary" aria-label="Logging summary"><Panel><small>Recent events</small><strong>{events.length}</strong></Panel><Panel><small>Attributed actors</small><strong>{attributed}</strong></Panel><Panel><small>Delivery or action failures</small><strong>{failures}</strong></Panel></section>
    <Panel className="form-detail-panel"><SectionHeader eyebrow="Destinations and retention" title="Logging policy" detail="Route events and control how long durable guild audit records remain available." /><div className="section-body logging-destination-grid"><DiscordChannelSelect title="Default logging channel" detail="Required while logging is enabled" value={logging.channel_id ?? ''} kind="text" channels={channels.data ?? []} pending={channels.isLoading} error={channels.isError} onChange={(channel_id) => update({ ...logging, channel_id: channel_id || null })} /><DiscordChannelMultiSelect title="Ignored channels" detail="Message and voice activity originating here is not logged" values={logging.ignored_channel_ids} channels={channels.data ?? []} pending={channels.isLoading} error={channels.isError} onChange={(ignored_channel_ids) => update({ ...logging, ignored_channel_ids })} /><label><span>Audit retention</span><small>Days to retain this server's durable audit history (30–3650)</small><input type="number" min={30} max={3650} value={logging.retention_days ?? 365} onChange={(event) => update({ ...logging, retention_days: Number(event.target.value) })} /></label></div></Panel>
    <Panel className="form-detail-panel"><SectionHeader eyebrow="Event routing" title="Logging categories" detail="Turn event families on independently and optionally route each one to a separate channel." /><div className="section-body logging-category-grid">{categories.map((category) => <LoggingCategoryCard key={category.id} title={category.title} detail={category.detail} enabled={Boolean(logging[category.enabledKey])} channelId={category.channelKey ? logging[category.channelKey] as string | null : undefined} channels={channels.data ?? []} pending={channels.isLoading} error={channels.isError} onEnabledChange={(enabled) => update({ ...logging, [category.enabledKey]: enabled })} onChannelChange={category.channelKey ? (channelId) => update({ ...logging, [category.channelKey as string]: channelId }) : undefined} />)}</div></Panel>
    <Panel className="analytics-chart-panel"><SectionHeader eyebrow="Event analytics" title="Recent Discord events by category" detail="Axes are labeled and each bar exposes contextual counts on hover." /><MetricChart data={chart} xLabel="Discord event category" yLabel="Recent events" ariaLabel="Recent Discord logging events grouped by category" type="bar" color="#57f287" /></Panel>
    <Panel className="table-panel"><SectionHeader eyebrow="Durable history" title="Recent logged events" detail="Open the complete audit record for metadata, target IDs, attribution, and outcome." /><DataTable data={events} columns={columns} getRowId={(event) => String(event.id)} ariaLabel="Recent Discord logging events" emptyMessage={audits.isLoading ? 'Loading Discord events…' : 'No Discord events have been recorded yet'} /></Panel>
    {save.isError && <div className="error-banner" role="alert">{save.error.message}</div>}
    <div className="bottom-save-bar"><div className="module-save-toggle"><Toggle label="Enable Discord event logging" checked={logging.enabled} onChange={(enabled) => update({ ...logging, enabled })} /><span><strong>Discord event logging</strong><small>View Audit Log permission is required for attributed administrative actions.</small></span></div><div className="save-row">{save.isSuccess && !changed && <span className="saved-label"><Check size={15} />Saved</span>}<Button disabled={!changed || save.isPending} onClick={() => save.mutate(draft)}>{save.isPending && <LoaderCircle className="spin" size={16} />}Save changes</Button></div></div>
  </div>
}

function eventCategory(event: AuditEvent) {
  if (event.action.startsWith('slash_command.') || event.action.startsWith('module.')) return 'commands'
  const metadataCategory = event.metadata.category
  return typeof metadataCategory === 'string' ? metadataCategory : 'moderation'
}

function friendlyAction(action: string) {
  return action.split('.').map((part) => part.charAt(0).toUpperCase() + part.slice(1).replaceAll('_', ' ')).join(' · ')
}

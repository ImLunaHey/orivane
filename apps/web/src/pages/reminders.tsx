import type { ColumnDef } from '@tanstack/react-table'
import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query'
import { Link, useParams } from '@tanstack/react-router'
import { ArrowLeft, Check, LoaderCircle, MessageSquare, Send, User } from 'lucide-react'
import { useEffect, useMemo, useState } from 'react'
import { DataTable } from '../components/data-table'
import { DiscordMemberLink } from '../components/discord-member-link'
import { MetricChart } from '../components/metric-chart'
import { Button, Panel, SectionHeader, Toggle } from '../components/ui'
import { cancelReminder, getGuildConfig, getReminder, getReminders, saveGuildConfig } from '../lib/api'
import type { GuildConfig, Reminder, ReminderStatus } from '../types'
import { LoadingState } from './reminders/loading-state'
import { NumberField } from './reminders/number-field'
import { ReminderStatusBadge, titleCase } from './reminders/reminder-status-badge'
import { ToggleField } from './reminders/toggle-field'

export function RemindersPage() {
  const { serverId } = useParams({ strict: false }) as { serverId: string }; const client = useQueryClient(); const configKey = ['guild-config', serverId] as const
  const config = useQuery({ queryKey: configKey, queryFn: () => getGuildConfig(serverId) }); const reminders = useQuery({ queryKey: ['reminders', serverId], queryFn: () => getReminders(serverId) })
  const [draft, setDraft] = useState<GuildConfig | null>(null); useEffect(() => setDraft(config.data ?? null), [config.data, serverId])
  const save = useMutation({ mutationFn: saveGuildConfig, onSuccess: (saved) => { client.setQueryData(configKey, saved); setDraft(saved) } })
  const columns = useMemo<Array<ColumnDef<Reminder>>>(() => [
    { accessorKey: 'user_id', header: 'Member', cell: ({ row }) => <DiscordMemberLink serverId={serverId} userId={row.original.user_id} username={row.original.username} displayName={row.original.display_name} /> },
    { accessorKey: 'message', header: 'Reminder', cell: ({ row }) => <Link to="/servers/$serverId/modules/reminders/$reminderId" params={{ serverId, reminderId: String(row.original.id) }}>{truncate(row.original.message, 72)}</Link> },
    { accessorKey: 'status', header: 'Status', cell: ({ row }) => <ReminderStatusBadge status={row.original.status} /> },
    { accessorKey: 'delivery_mode', header: 'Delivery', cell: ({ row }) => row.original.delivery_mode === 'dm' ? 'Direct message' : 'Channel' },
    { accessorKey: 'remind_at', header: 'Due', cell: ({ row }) => formatDate(row.original.remind_at) },
    { id: 'actions', header: '', enableSorting: false, cell: ({ row }) => <Link className="button table-action" to="/servers/$serverId/modules/reminders/$reminderId" params={{ serverId, reminderId: String(row.original.id) }}>Inspect</Link> },
  ], [serverId])
  if (config.isLoading || !draft) return <LoadingState />
  const module = draft.modules.reminders; const items = reminders.data ?? []; const hasChanges = JSON.stringify(config.data) !== JSON.stringify(draft); const updateModule = (value: GuildConfig['modules']['reminders']) => setDraft({ ...draft, modules: { ...draft.modules, reminders: value } })
  const chartData = (['scheduled', 'delivered', 'cancelled', 'failed'] as ReminderStatus[]).map((status) => ({ label: titleCase(status), value: items.filter((item) => item.status === status).length, detail: `${items.filter((item) => item.status === status).length} reminders are ${status}.` }))
  return <div className="page-stack reminders-page">
    <Link className="back-link" to="/servers/$serverId/modules" params={{ serverId }}><ArrowLeft size={16} />All modules</Link>
    <section className="page-intro"><div><p className="eyebrow">Personal scheduling</p><h2>Reminders</h2><p>Members schedule private or in-channel reminders using slash commands. Delivery is durable, retried, and fully audited.</p></div></section>
    <section className="forms-summary" aria-label="Reminder summary"><Panel><small>Total reminders</small><strong>{items.length}</strong></Panel><Panel><small>Scheduled</small><strong>{items.filter((item) => item.status === 'scheduled').length}</strong></Panel><Panel><small>Delivered</small><strong>{items.filter((item) => item.status === 'delivered').length}</strong></Panel><Panel><small>Failed</small><strong>{items.filter((item) => item.status === 'failed').length}</strong></Panel></section>
    <Panel className="form-detail-panel"><SectionHeader eyebrow="Configuration" title="Reminder limits" detail="These safeguards apply to /remind for every member in this server." /><div className="section-body form-grid"><ToggleField title="Allow direct messages" detail="Members may choose private delivery instead of the current channel" checked={module.allow_direct_messages} onChange={(allow_direct_messages) => updateModule({ ...module, allow_direct_messages })} /><NumberField title="Active reminders per member" detail="Maximum pending reminders per member" value={module.max_active_per_user} min={1} max={50} onChange={(max_active_per_user) => updateModule({ ...module, max_active_per_user })} /><NumberField title="Minimum delay" detail="Shortest delay in minutes" value={module.minimum_delay_minutes} min={1} max={1440} onChange={(minimum_delay_minutes) => updateModule({ ...module, minimum_delay_minutes })} /><NumberField title="Maximum delay" detail="Longest delay in days" value={module.maximum_delay_days} min={1} max={3650} onChange={(maximum_delay_days) => updateModule({ ...module, maximum_delay_days })} /></div></Panel>
    <Panel className="analytics-chart-panel"><SectionHeader eyebrow="Delivery analytics" title="Reminders by status" /><MetricChart data={chartData} xLabel="Reminder status" yLabel="Reminders" ariaLabel="Number of reminders grouped by delivery status" type="bar" color="#38bdf8" /></Panel>
    <Panel className="table-panel reminders-directory"><SectionHeader eyebrow="Directory" title="Server reminders" detail="Message contents are visible to authorized dashboard administrators." /><DataTable data={items} columns={columns} getRowId={(row) => String(row.id)} ariaLabel="Server reminders" emptyMessage={reminders.isLoading ? 'Loading reminders…' : 'No reminders have been scheduled yet'} /></Panel>
    {(save.isError || reminders.isError) && <div className="error-banner" role="alert">{save.error?.message ?? reminders.error?.message}</div>}
    <div className="bottom-save-bar"><div className="module-save-toggle"><Toggle label="Enable Reminders" checked={module.enabled} onChange={(enabled) => updateModule({ ...module, enabled })} /><span><strong>Reminders module</strong><small>{module.enabled ? 'Slash commands and scheduled delivery are active.' : 'Reminder commands and delivery are disabled.'}</small></span></div><div className="save-row">{save.isSuccess && !hasChanges && <span className="saved-label"><Check size={15} />Saved</span>}<Button disabled={!hasChanges || save.isPending} onClick={() => save.mutate(draft)}>{save.isPending && <LoaderCircle className="spin" size={16} />}Save changes</Button></div></div>
  </div>
}

export function ReminderDetailPage() {
  const { serverId, reminderId } = useParams({ strict: false }) as { serverId: string; reminderId: string }; const client = useQueryClient(); const key = ['reminder', serverId, reminderId] as const
  const reminder = useQuery({ queryKey: key, queryFn: () => getReminder(serverId, reminderId) }); const cancel = useMutation({ mutationFn: cancelReminder, onSuccess: (saved) => { client.setQueryData(key, saved); void client.invalidateQueries({ queryKey: ['reminders', serverId] }) } })
  if (reminder.isLoading) return <LoadingState />; if (reminder.isError || !reminder.data) return <div className="empty-state"><h2>Reminder not found</h2><Link to="/servers/$serverId/modules/reminders" params={{ serverId }}>Return to reminders</Link></div>
  const item = reminder.data
  return <div className="page-stack reminder-detail-page"><Link className="back-link" to="/servers/$serverId/modules/reminders" params={{ serverId }}><ArrowLeft size={16} />All reminders</Link><section className="page-intro"><div><p className="eyebrow">Reminder #{item.id}</p><h2>{truncate(item.message, 80)}</h2><p>Created {formatDate(item.created_at)}</p></div><ReminderStatusBadge status={item.status} /></section><section className="suggestion-detail-summary"><Panel><small>Member</small><strong><User size={20} /><DiscordMemberLink serverId={serverId} userId={item.user_id} username={item.username} displayName={item.display_name} /></strong></Panel><Panel><small>Delivery</small><strong>{item.delivery_mode === 'dm' ? <><Send size={20} />Direct message</> : <><MessageSquare size={20} />Channel</>}</strong></Panel><Panel><small>Attempts</small><strong>{item.attempts} / 5</strong></Panel></section><Panel className="form-detail-panel"><SectionHeader eyebrow="Contents" title="Reminder message" /><div className="section-body reminder-message"><p>{item.message}</p></div></Panel><Panel className="giveaway-rules"><SectionHeader eyebrow="Delivery lifecycle" title="Timing and destination" /><dl className="section-body"><div><dt>Scheduled for</dt><dd>{formatDate(item.remind_at)}</dd></div><div><dt>Next attempt</dt><dd>{formatDate(item.next_attempt_at)}</dd></div><div><dt>Delivered</dt><dd>{item.delivered_at ? formatDate(item.delivered_at) : 'Not delivered'}</dd></div><div><dt>Channel</dt><dd>{item.channel_id ?? 'Private message'}</dd></div></dl></Panel>{cancel.isError && <div className="error-banner" role="alert">{cancel.error.message}</div>}<div className="bottom-save-bar"><span>Cancellation is audited and immediately prevents scheduled delivery.</span>{item.status === 'scheduled' && <Button disabled={cancel.isPending} onClick={() => cancel.mutate({ guildId: serverId, reminderId })}>Cancel reminder</Button>}</div></div>
}

function truncate(value: string, max: number) { return value.length > max ? `${value.slice(0, max)}…` : value }
function formatDate(value: string) { return new Intl.DateTimeFormat(undefined, { dateStyle: 'medium', timeStyle: 'short' }).format(new Date(value)) }

import type { ColumnDef } from '@tanstack/react-table'
import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query'
import { Link, useParams } from '@tanstack/react-router'
import { ArrowLeft, Check, LoaderCircle, MessageSquareText, ThumbsDown, ThumbsUp } from 'lucide-react'
import { useEffect, useMemo, useState } from 'react'
import { DataTable } from '../components/data-table'
import { DiscordChannelSelect } from '../components/discord-channel-select'
import { DiscordMemberLink } from '../components/discord-member-link'
import { DiscordRoleSelect } from '../components/discord-role-select'
import { MetricChart } from '../components/metric-chart'
import { Badge, Button, Panel, SectionHeader, Toggle } from '../components/ui'
import { getGuildChannels, getGuildConfig, getGuildRoles, getSuggestion, getSuggestions, saveGuildConfig, updateSuggestionStatus } from '../lib/api'
import type { GuildConfig, Suggestion, SuggestionStatus } from '../types'

const statuses: SuggestionStatus[] = ['pending', 'approved', 'implemented', 'rejected', 'duplicate']

export function SuggestionsPage() {
  const { serverId } = useParams({ strict: false }) as { serverId: string }
  const client = useQueryClient()
  const configKey = ['guild-config', serverId] as const
  const config = useQuery({ queryKey: configKey, queryFn: () => getGuildConfig(serverId) })
  const suggestions = useQuery({ queryKey: ['suggestions', serverId], queryFn: () => getSuggestions(serverId) })
  const channels = useQuery({ queryKey: ['guild-channels', serverId], queryFn: () => getGuildChannels(serverId) })
  const roles = useQuery({ queryKey: ['guild-roles', serverId], queryFn: () => getGuildRoles(serverId) })
  const [draft, setDraft] = useState<GuildConfig | null>(null)
  const save = useMutation({ mutationFn: saveGuildConfig, onSuccess: (saved) => { client.setQueryData(configKey, saved); setDraft(saved) } })
  useEffect(() => setDraft(config.data ?? null), [config.data, serverId])

  const columns = useMemo<Array<ColumnDef<Suggestion>>>(() => [
    { accessorKey: 'id', header: 'ID', cell: ({ row }) => <Link to="/servers/$serverId/modules/suggestions/$suggestionId" params={{ serverId, suggestionId: String(row.original.id) }}>#{row.original.id}</Link> },
    { accessorKey: 'content', header: 'Suggestion', cell: ({ row }) => <div className="suggestion-content-cell"><strong>{row.original.content}</strong><small>{row.original.author_id ? <DiscordMemberLink serverId={serverId} userId={row.original.author_id} username={row.original.username} displayName={row.original.display_name} /> : 'Anonymous'}</small></div> },
    { accessorKey: 'status', header: 'Status', cell: ({ row }) => <StatusBadge status={row.original.status} /> },
    { id: 'votes', header: 'Votes', accessorFn: (row) => row.upvotes - row.downvotes, cell: ({ row }) => <span className="vote-total"><ThumbsUp size={15} />{row.original.upvotes}<ThumbsDown size={15} />{row.original.downvotes}</span> },
    { accessorKey: 'created_at', header: 'Received', cell: ({ row }) => formatDate(row.original.created_at) },
    { id: 'actions', header: '', enableSorting: false, cell: ({ row }) => <Link className="button table-action" to="/servers/$serverId/modules/suggestions/$suggestionId" params={{ serverId, suggestionId: String(row.original.id) }}>Inspect</Link> },
  ], [serverId])

  if (config.isLoading || !draft) return <LoadingState />
  const module = draft.modules.suggestions
  const items = suggestions.data ?? []
  const hasChanges = JSON.stringify(config.data) !== JSON.stringify(draft)
  const totalVotes = items.reduce((total, item) => total + item.upvotes + item.downvotes, 0)
  const statusData = statuses.map((status) => ({ label: labelStatus(status), value: items.filter((item) => item.status === status).length, detail: `${items.filter((item) => item.status === status).length} suggestions are ${labelStatus(status).toLowerCase()}.` }))
  const updateModule = (value: GuildConfig['modules']['suggestions']) => setDraft({ ...draft, modules: { ...draft.modules, suggestions: value } })

  return <div className="page-stack suggestions-page">
    <Link className="back-link" to="/servers/$serverId/modules" params={{ serverId }}><ArrowLeft size={16} />All modules</Link>
    <section className="page-intro"><div><p className="eyebrow">Community feedback</p><h2>Suggestions</h2><p>Collect member ideas, measure support, and move each proposal through a visible staff workflow.</p></div><Badge tone={module.enabled ? 'green' : 'neutral'}>{module.enabled ? 'Module active' : 'Module disabled'}</Badge></section>
    <section className="forms-summary" aria-label="Suggestions summary"><Panel><small>Total suggestions</small><strong>{items.length}</strong></Panel><Panel><small>Awaiting review</small><strong>{items.filter((item) => item.status === 'pending').length}</strong></Panel><Panel><small>Votes cast</small><strong>{totalVotes}</strong></Panel><Panel><small>Implemented</small><strong>{items.filter((item) => item.status === 'implemented').length}</strong></Panel></section>
    <Panel className="form-detail-panel"><SectionHeader eyebrow="Configuration" title="Submission settings" detail="Control where suggestions appear and who can manage their status." /><div className="section-body form-grid"><DiscordChannelSelect title="Suggestions channel" detail="New suggestions and vote buttons are posted here" value={module.channel_id} kind="text" channels={channels.data ?? []} pending={channels.isLoading} error={channels.isError} onChange={(channel_id) => updateModule({ ...module, channel_id })} /><DiscordRoleSelect title="Staff roles" detail="Members allowed to approve, reject, or implement suggestions" values={module.staff_role_ids} roles={roles.data ?? []} pending={roles.isLoading} error={roles.isError} max={25} onChange={(staff_role_ids) => updateModule({ ...module, staff_role_ids })} /><ToggleField title="Anonymous suggestions" detail="Allow members to hide their identity from the suggestion post and dashboard" checked={module.allow_anonymous} onChange={(allow_anonymous) => updateModule({ ...module, allow_anonymous })} /><NumberField title="Submission cooldown" detail="Seconds each member must wait before submitting again" value={module.cooldown_seconds} min={0} max={86400} onChange={(cooldown_seconds) => updateModule({ ...module, cooldown_seconds })} /></div></Panel>
    {(channels.isError || roles.isError || suggestions.isError) && <div className="error-banner" role="alert">{[channels.isError && `Channels: ${channels.error.message}`, roles.isError && `Roles: ${roles.error.message}`, suggestions.isError && `Suggestions: ${suggestions.error.message}`].filter(Boolean).join(' · ')}</div>}
    <Panel className="analytics-chart-panel"><SectionHeader eyebrow="Workflow analytics" title="Suggestions by status" detail="Every chart includes named axes and contextual hover details." /><MetricChart data={statusData} xLabel="Suggestion status" yLabel="Suggestions" ariaLabel="Number of suggestions grouped by workflow status" type="bar" color="#9b76f6" /></Panel>
    <Panel className="table-panel suggestions-directory"><SectionHeader eyebrow="Inbox" title="Current suggestions" detail="Open a suggestion to review its content, votes, author, and staff decision." /><DataTable data={items} columns={columns} getRowId={(row) => String(row.id)} ariaLabel="Server suggestions" emptyMessage={suggestions.isLoading ? 'Loading suggestions…' : 'No suggestions have been submitted yet'} /></Panel>
    {save.isError && <div className="error-banner" role="alert">{save.error.message}</div>}
    <div className="bottom-save-bar"><div className="module-save-toggle"><Toggle label="Enable Suggestions" checked={module.enabled} onChange={(enabled) => updateModule({ ...module, enabled })} /><span><strong>Suggestions module</strong><small>{module.enabled ? 'Members can submit and vote with slash commands and buttons.' : 'Suggestion commands and voting are disabled.'}</small></span></div><div className="save-row">{save.isSuccess && !hasChanges && <span className="saved-label"><Check size={15} />Saved</span>}<Button disabled={!hasChanges || save.isPending} onClick={() => save.mutate(draft)}>{save.isPending && <LoaderCircle className="spin" size={16} />}Save changes</Button></div></div>
  </div>
}

export function SuggestionDetailPage() {
  const { serverId, suggestionId } = useParams({ strict: false }) as { serverId: string; suggestionId: string }
  const client = useQueryClient()
  const queryKey = ['suggestion', serverId, suggestionId] as const
  const suggestion = useQuery({ queryKey, queryFn: () => getSuggestion(serverId, suggestionId) })
  const [status, setStatus] = useState<SuggestionStatus>('pending')
  const [staffNote, setStaffNote] = useState('')
  useEffect(() => { if (suggestion.data) { setStatus(suggestion.data.status); setStaffNote(suggestion.data.staff_note ?? '') } }, [suggestion.data])
  const update = useMutation({ mutationFn: updateSuggestionStatus, onSuccess: (saved) => { client.setQueryData(queryKey, saved); void client.invalidateQueries({ queryKey: ['suggestions', serverId] }) } })
  if (suggestion.isLoading) return <LoadingState />
  if (suggestion.isError || !suggestion.data) return <div className="empty-state"><h2>Suggestion not found</h2><Link to="/servers/$serverId/modules/suggestions" params={{ serverId }}>Return to suggestions</Link></div>
  const item = suggestion.data
  const hasChanges = item.status !== status || (item.staff_note ?? '') !== staffNote

  return <div className="page-stack suggestion-detail-page">
    <Link className="back-link" to="/servers/$serverId/modules/suggestions" params={{ serverId }}><ArrowLeft size={16} />All suggestions</Link>
    <section className="page-intro"><div><p className="eyebrow">Suggestion #{item.id}</p><h2>Review suggestion</h2><p>Submitted {formatDate(item.created_at)} and last updated {formatDate(item.updated_at)}.</p></div><StatusBadge status={item.status} /></section>
    <section className="suggestion-detail-summary"><Panel><small>Upvotes</small><strong><ThumbsUp size={20} />{item.upvotes}</strong></Panel><Panel><small>Downvotes</small><strong><ThumbsDown size={20} />{item.downvotes}</strong></Panel><Panel><small>Net score</small><strong>{item.upvotes - item.downvotes}</strong></Panel></section>
    <Panel className="suggestion-copy-panel"><SectionHeader eyebrow="Member proposal" title="Suggestion" action={item.author_id ? <DiscordMemberLink serverId={serverId} userId={item.author_id} username={item.username} displayName={item.display_name} /> : <Badge tone="neutral">Anonymous</Badge>} /><div className="section-body suggestion-copy"><MessageSquareText size={22} /><p>{item.content}</p></div></Panel>
    <Panel className="form-detail-panel"><SectionHeader eyebrow="Staff workflow" title="Decision" detail="Status changes are retained in the global and server audit logs." /><div className="section-body form-grid"><SelectField title="Status" detail="Current outcome visible to staff" value={status} options={statuses.map((value) => [value, labelStatus(value)])} onChange={(value) => setStatus(value as SuggestionStatus)} /><label className="wide"><span>Staff note</span><small>Internal decision context, up to 500 characters; note contents are not copied into audit metadata</small><textarea value={staffNote} onChange={(event) => setStaffNote(event.target.value)} maxLength={500} /></label></div></Panel>
    <Panel className="suggestion-metadata"><SectionHeader eyebrow="Discord record" title="Delivery metadata" /><dl className="section-body"><div><dt>Channel ID</dt><dd>{item.channel_id}</dd></div><div><dt>Message ID</dt><dd>{item.message_id ?? 'Not delivered'}</dd></div><div><dt>Author ID</dt><dd>{item.author_id ?? 'Anonymous'}</dd></div></dl></Panel>
    {update.isError && <div className="error-banner" role="alert">{update.error.message}</div>}
    <div className="bottom-save-bar"><span /> <div className="save-row">{update.isSuccess && !hasChanges && <span className="saved-label"><Check size={15} />Saved</span>}<Button disabled={!hasChanges || update.isPending} onClick={() => update.mutate({ guildId: serverId, suggestionId, status, staff_note: staffNote.trim() || null })}>{update.isPending && <LoaderCircle className="spin" size={16} />}Save decision</Button></div></div>
  </div>
}

function StatusBadge({ status }: { status: SuggestionStatus }) {
  const tone = status === 'implemented' || status === 'approved' ? 'green' : status === 'rejected' ? 'red' : status === 'pending' ? 'amber' : 'neutral'
  return <Badge tone={tone}>{labelStatus(status)}</Badge>
}
function labelStatus(status: SuggestionStatus) { return `${status.charAt(0).toUpperCase()}${status.slice(1)}` }
function NumberField({ title, detail, value, min, max, onChange }: { title: string; detail: string; value: number; min: number; max: number; onChange: (value: number) => void }) { return <label><span>{title}</span><small>{detail}</small><input type="number" min={min} max={max} value={value} onChange={(event) => onChange(Number(event.target.value))} /></label> }
function SelectField({ title, detail, value, options, onChange }: { title: string; detail: string; value: string; options: string[][]; onChange: (value: string) => void }) { return <label><span>{title}</span><small>{detail}</small><select value={value} onChange={(event) => onChange(event.target.value)}>{options.map(([option, label]) => <option value={option} key={option}>{label}</option>)}</select></label> }
function ToggleField({ title, detail, checked, onChange }: { title: string; detail: string; checked: boolean; onChange: (value: boolean) => void }) { return <div className="module-toggle-field"><span><strong>{title}</strong><small>{detail}</small></span><Toggle label={title} checked={checked} onChange={onChange} /></div> }
function formatDate(value: string) { return new Intl.DateTimeFormat(undefined, { dateStyle: 'medium', timeStyle: 'short' }).format(new Date(value)) }
function LoadingState() { return <div className="loading-state"><LoaderCircle className="spin" /><span>Loading suggestions…</span></div> }

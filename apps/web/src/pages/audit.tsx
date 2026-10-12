import { useQuery } from '@tanstack/react-query'
import { Link, useParams } from '@tanstack/react-router'
import type { ColumnDef } from '@tanstack/react-table'
import { AlertTriangle, ArrowLeft, CheckCircle2, Download, FileClock, Search, ShieldAlert } from 'lucide-react'
import { useMemo, useState } from 'react'
import { DataTable } from '../components/data-table'
import { Badge, Button, Panel } from '../components/ui'
import { getAuditEvent, getAuditEvents, getServers, isDemoMode } from '../lib/api'
import { demoServers } from '../lib/server-data'
import { globalUsers } from '../lib/user-data'
import type { AuditEvent } from '../types'

type OutcomeFilter = 'all' | AuditEvent['outcome']

export function GlobalAuditPage() {
  return <AuditLogPage />
}

export function ServerAuditPage() {
  const { serverId } = useParams({ strict: false }) as { serverId: string }
  return <AuditLogPage guildId={serverId} />
}

export function GlobalAuditDetailPage() {
  const { eventId } = useParams({ strict: false }) as { eventId: string }
  return <AuditEventPage eventId={eventId} />
}

export function ServerAuditDetailPage() {
  const { serverId, eventId } = useParams({ strict: false }) as { serverId: string; eventId: string }
  return <AuditEventPage eventId={eventId} guildId={serverId} />
}

function AuditLogPage({ guildId }: { guildId?: string }) {
  const [query, setQuery] = useState('')
  const [outcome, setOutcome] = useState<OutcomeFilter>('all')
  const auditQuery = useQuery({ queryKey: ['audit-events', guildId ?? 'global'], queryFn: () => getAuditEvents(guildId) })
  const serversQuery = useQuery({ queryKey: ['servers'], queryFn: getServers, enabled: !guildId })
  const servers = useMemo(() => serversQuery.data ?? (isDemoMode ? demoServers : []), [serversQuery.data])
  const events = useMemo(() => auditQuery.data ?? [], [auditQuery.data])
  const filtered = useMemo(() => events.filter((event) => {
    const haystack = `${event.action} ${event.actor_id} ${event.target_type} ${event.target_id ?? ''} ${event.source} ${event.reason ?? ''}`.toLowerCase()
    return (outcome === 'all' || event.outcome === outcome) && haystack.includes(query.toLowerCase())
  }), [events, outcome, query])
  const columns = useMemo<Array<ColumnDef<AuditEvent>>>(() => [
    { accessorKey: 'occurred_at', header: 'Time', cell: ({ getValue }) => <time dateTime={getValue<string>()}>{formatTimestamp(getValue<string>())}</time> },
    { accessorKey: 'action', header: 'Action', cell: ({ row }) => <span className="audit-action"><strong>{friendlyAction(row.original.action)}</strong><small>{row.original.source.replaceAll('_', ' ')}</small></span> },
    { accessorKey: 'actor_id', header: 'Actor', cell: ({ row }) => <Actor event={row.original} /> },
    ...(!guildId ? [{ accessorKey: 'guild_id', header: 'Scope', cell: ({ row }: { row: { original: AuditEvent } }) => <Scope event={row.original} serverName={servers.find((server) => server.id === row.original.guild_id)?.name} /> } as ColumnDef<AuditEvent>] : []),
    { accessorKey: 'target_type', header: 'Target', cell: ({ row }) => <span className="audit-target"><strong>{row.original.target_type.replaceAll('_', ' ')}</strong><small>{row.original.target_id ?? 'No specific target'}</small></span> },
    { accessorKey: 'outcome', header: 'Outcome', cell: ({ getValue }) => { const value = getValue<AuditEvent['outcome']>(); return <Badge tone={value === 'success' ? 'green' : value === 'denied' ? 'amber' : 'red'}>{value}</Badge> } },
    { id: 'details', header: 'Details', enableSorting: false, cell: ({ row }) => guildId ? <Link className="text-link audit-details-button" to="/servers/$serverId/audit/$eventId" params={{ serverId: guildId, eventId: String(row.original.id) }}>Inspect →</Link> : <Link className="text-link audit-details-button" to="/audit/$eventId" params={{ eventId: String(row.original.id) }}>Inspect →</Link> },
  ], [guildId, servers])
  const successes = events.filter((event) => event.outcome === 'success').length
  const denied = events.filter((event) => event.outcome === 'denied').length
  const failures = events.filter((event) => event.outcome === 'failure').length

  return <div className="page-stack">
    <section className="page-intro"><div><p className="eyebrow">{guildId ? 'Server accountability' : 'Fleet accountability'}</p><h2>{guildId ? 'Server audit log' : 'Global audit log'}</h2><p>{guildId ? 'Configuration, moderation, and command actions for this Discord server.' : 'Security and operational actions across the bot, every server, the API, and the gateway.'}</p></div><Button onClick={() => downloadAuditCsv(filtered, guildId)} disabled={!filtered.length}><Download size={16} />Export CSV</Button></section>
    <section className="system-metrics"><Metric icon={FileClock} label="Events loaded" value={events.length.toLocaleString()} note="Newest first" /><Metric icon={CheckCircle2} label="Successful" value={successes.toLocaleString()} note={rate(successes, events.length)} /><Metric icon={ShieldAlert} label="Denied" value={denied.toLocaleString()} note="Permission and policy blocks" /><Metric icon={AlertTriangle} label="Failed" value={failures.toLocaleString()} note="Needs investigation" /></section>
    <Panel className="system-panel audit-table-panel">
      <div className="table-toolbar audit-toolbar"><div><p className="eyebrow">Append-only history</p><h3>Recorded actions</h3></div><div className="audit-filters"><label className="search-box"><Search size={16} /><input aria-label="Search audit events" value={query} onChange={(event) => setQuery(event.target.value)} placeholder="Action, actor, target" /></label><label className="select-label"><span>Outcome</span><select aria-label="Filter by outcome" value={outcome} onChange={(event) => setOutcome(event.target.value as OutcomeFilter)}><option value="all">All outcomes</option><option value="success">Success</option><option value="denied">Denied</option><option value="failure">Failure</option></select></label></div></div>
      {auditQuery.isLoading ? <div className="audit-loading">Loading audit events…</div> : auditQuery.isError ? <div className="audit-loading audit-error">Could not load audit events: {auditQuery.error.message}</div> : <DataTable data={filtered} columns={columns} getRowId={(row) => String(row.id)} emptyMessage="No audit events match these filters" ariaLabel={guildId ? 'Server audit events' : 'Global audit events'} />}
    </Panel>
  </div>
}

function AuditEventPage({ eventId, guildId }: { eventId: string; guildId?: string }) {
  const eventQuery = useQuery({ queryKey: ['audit-event', guildId ?? 'global', eventId], queryFn: () => getAuditEvent(eventId, guildId) })
  const backLink = guildId ? <Link className="back-link" to="/servers/$serverId/audit" params={{ serverId: guildId }}><ArrowLeft size={14} />Back to server audit log</Link> : <Link className="back-link" to="/audit"><ArrowLeft size={14} />Back to global audit log</Link>
  if (eventQuery.isLoading) return <div className="page-stack">{backLink}<div className="audit-loading">Loading audit event…</div></div>
  if (eventQuery.isError || !eventQuery.data) return <div className="page-stack">{backLink}<div className="empty-state"><h2>Audit event not found</h2><p>{eventQuery.error?.message ?? 'This event is unavailable.'}</p></div></div>
  const event = eventQuery.data
  return <div className="page-stack">{backLink}<section className="page-intro audit-detail-intro"><div><p className="eyebrow">Audit event #{event.id}</p><h2>{friendlyAction(event.action)}</h2><p>Recorded {new Date(event.occurred_at).toLocaleString()} from {event.source.replaceAll('_', ' ')}.</p></div><Badge tone={event.outcome === 'success' ? 'green' : event.outcome === 'denied' ? 'amber' : 'red'}>{event.outcome}</Badge></section><AuditDetails event={event} /></div>
}

function AuditDetails({ event }: { event: AuditEvent }) {
  return <Panel className="audit-detail-panel"><div className="panel-heading"><div><p className="eyebrow">Event context</p><h3>Who did what, where, and why</h3></div></div>
    <dl className="audit-meta"><div><dt>Timestamp</dt><dd>{new Date(event.occurred_at).toLocaleString()}</dd></div><div><dt>Actor</dt><dd>{actorName(event.actor_id)} · {event.actor_kind}</dd></div><div><dt>Source</dt><dd>{event.source}</dd></div><div><dt>Request ID</dt><dd>{event.request_id ?? 'Not supplied'}</dd></div><div><dt>Target</dt><dd>{event.target_type} · {event.target_id ?? 'none'}</dd></div><div><dt>Outcome</dt><dd>{event.outcome}</dd></div></dl>
    {event.reason && <div className="audit-reason"><strong>Reason</strong><p>{event.reason}</p></div>}
    <div className="audit-json-grid"><JsonBlock label="Before" value={event.before_state} /><JsonBlock label="After" value={event.after_state} /><JsonBlock label="Metadata" value={event.metadata} /></div>
  </Panel>
}

function JsonBlock({ label, value }: { label: string; value: unknown }) {
  return <div><strong>{label}</strong><pre>{value == null ? 'No data recorded' : JSON.stringify(value, null, 2)}</pre></div>
}

function Actor({ event }: { event: AuditEvent }) {
  const user = globalUsers.find((item) => item.id === event.actor_id)
  const isDiscordUser = event.actor_kind === 'user' && /^[1-9][0-9]{0,19}$/.test(event.actor_id)
  return isDiscordUser ? <Link className="table-user-link" to="/users/$userId" params={{ userId: event.actor_id }}>{user?.displayName ?? `Discord user ${event.actor_id.slice(-4)}`}<small>{event.actor_kind}</small></Link> : <span className="audit-target"><strong>{actorName(event.actor_id)}</strong><small>{event.actor_kind}</small></span>
}

function Scope({ event, serverName }: { event: AuditEvent; serverName?: string }) {
  if (!event.guild_id) return <Badge tone="violet">Bot global</Badge>
  return <Link className="table-user-link" to="/servers/$serverId/audit" params={{ serverId: event.guild_id }}>{serverName ?? event.guild_id}<small>Server</small></Link>
}

function Metric({ icon: Icon, label, value, note }: { icon: typeof FileClock; label: string; value: string; note: string }) {
  return <Panel><span className="system-icon"><Icon size={17} /></span><div><small>{label}</small><strong>{value}</strong><em>{note}</em></div></Panel>
}

function friendlyAction(action: string) { return action.split('.').map((part) => part.charAt(0).toUpperCase() + part.slice(1).replaceAll('_', ' ')).join(' · ') }
function actorName(id: string) { return id === 'system' ? 'Orivane system' : id === 'api-client' ? 'API client' : id }
function formatTimestamp(value: string) { return new Date(value).toLocaleString([], { month: 'short', day: 'numeric', hour: '2-digit', minute: '2-digit' }) }
function rate(count: number, total: number) { return total ? `${Math.round((count / total) * 100)}% of loaded events` : 'No events loaded' }

function downloadAuditCsv(events: AuditEvent[], guildId?: string) {
  const rows = [['id', 'occurred_at', 'scope', 'guild_id', 'actor_id', 'actor_kind', 'action', 'target_type', 'target_id', 'outcome', 'source', 'request_id', 'reason'], ...events.map((event) => [event.id, event.occurred_at, event.scope, event.guild_id, event.actor_id, event.actor_kind, event.action, event.target_type, event.target_id, event.outcome, event.source, event.request_id, event.reason])]
  const csv = rows.map((row) => row.map((value) => `"${String(value ?? '').replaceAll('"', '""')}"`).join(',')).join('\n')
  const url = URL.createObjectURL(new Blob([csv], { type: 'text/csv;charset=utf-8' }))
  const link = document.createElement('a')
  link.href = url
  link.download = guildId ? `orivane-audit-${guildId}.csv` : 'orivane-global-audit.csv'
  link.click()
  URL.revokeObjectURL(url)
}

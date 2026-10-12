import { flexRender, getCoreRowModel, useReactTable, type ColumnDef } from '@tanstack/react-table'
import { Link, useParams } from '@tanstack/react-router'
import { ArrowDownToLine, ArrowLeft, Database, Filter, Gauge, Radio, Search, ShieldCheck, TerminalSquare } from 'lucide-react'
import { useMemo, useState, type ReactNode } from 'react'
import { useQuery } from '@tanstack/react-query'
import { MetricChart } from '../components/metric-chart'
import { Badge, Button, Panel } from '../components/ui'
import { automodActions, commandUsage, hourlyCommands, latencyTrend, type ChartDatum } from '../lib/analytics-data'
import { analyticsEvents } from '../lib/demo-data'
import type { AnalyticsEvent } from '../types'
import { downloadCsv } from '../lib/export'
import { getAuditEvents, getCommandVolume, getGuildStats, isDemoMode } from '../lib/api'

export function AnalyticsPage() {
  const { serverId } = useParams({ strict: false }) as { serverId: string }
  const [search, setSearch] = useState('')
  const summary = useQuery({ queryKey: ['guild-stats', serverId], queryFn: () => getGuildStats(serverId) })
  const volume = useQuery({ queryKey: ['command-volume', 'v2', serverId], queryFn: () => getCommandVolume(serverId) })
  const audit = useQuery({ queryKey: ['audit-events', serverId], queryFn: () => getAuditEvents(serverId) })
  const liveEvents = (audit.data ?? []).map(auditToAnalytics)
  const events = isDemoMode ? analyticsEvents : liveEvents
  const columns = useMemo<ColumnDef<AnalyticsEvent>[]>(() => [
    { accessorKey: 'event', header: 'Event', cell: ({ row }) => <span className="event-name"><Radio size={14} />{row.original.event}</span> },
    { accessorKey: 'module', header: 'Module' },
    { accessorKey: 'outcome', header: 'Outcome', cell: ({ row }) => <Badge tone={row.original.outcome === 'failed' ? 'red' : row.original.outcome === 'blocked' ? 'amber' : 'green'}>{row.original.outcome}</Badge> },
    { accessorKey: 'latency', header: 'Latency', cell: ({ row }) => `${row.original.latency} ms` },
    { accessorKey: 'occurredAt', header: 'Occurred' },
  ], [])
  const filtered = events.filter((event) => `${event.event} ${event.module} ${event.outcome}`.toLowerCase().includes(search.toLowerCase()))
  const table = useReactTable({ data: filtered, columns, getCoreRowModel: getCoreRowModel() })

  return <div className="page-stack">
    <section className="page-intro"><div><p className="eyebrow">Observability</p><h2>Analytics</h2><p>Understand command use, module outcomes, and latency without retaining message content.</p></div><Button onClick={() => downloadCsv(`orivane-${serverId}-analytics-events.csv`, events, serverId)}><ArrowDownToLine size={16} />Export</Button></section>
    <section className="analytics-summary">
      <Link to="/servers/$serverId/analytics/commands" params={{ serverId }}><SummaryMetric icon={<Radio size={17} />} label="Commands today" value={summary.data?.commands_today.toLocaleString() ?? '—'} detail="Open command report →" /></Link>
      <Link to="/servers/$serverId/analytics/latency" params={{ serverId }}><SummaryMetric icon={<Database size={17} />} label="p95 response" value={summary.data?.p95_latency_ms == null ? '—' : `${Math.round(summary.data.p95_latency_ms)} ms`} detail="Open latency report →" /></Link>
      <Link to="/servers/$serverId/analytics/automod" params={{ serverId }}><SummaryMetric icon={<Filter size={17} />} label="Automod actions" value={summary.data?.automod_actions_today.toLocaleString() ?? '—'} detail="Open automod report →" /></Link>
    </section>

    <section className="analytics-chart-grid">
      <ReportPanel title="Command volume" eyebrow="Traffic" detailTo="commands" serverId={serverId}><MetricChart data={volume.data ?? (isDemoMode ? hourlyCommands : [])} xLabel="Time (UTC)" yLabel="Commands / 4h" ariaLabel="Command volume over 24 hours" /></ReportPanel>
      <ReportPanel title="Response latency" eyebrow="Performance" detailTo="latency" serverId={serverId}><MetricChart data={isDemoMode ? latencyTrend : latencyFromEvents(events)} xLabel="Recent command" yLabel="Response latency" unit=" ms" color="#4aa8e8" ariaLabel="Recent response latency" /></ReportPanel>
      <ReportPanel title="Automod actions" eyebrow="Safety" detailTo="automod" serverId={serverId}><MetricChart data={isDemoMode ? automodActions : automodFromEvents(events)} xLabel="Rule category" yLabel="Actions today" type="bar" color="#e1a04b" ariaLabel="Automod actions by rule category" /></ReportPanel>
    </section>

    <Panel className="table-panel">
      <div className="table-toolbar"><div><h3>Event explorer</h3><Badge tone="violet">Live</Badge></div><label className="search-box"><Search size={15} /><input aria-label="Filter analytics events" value={search} onChange={(event) => setSearch(event.target.value)} placeholder="Filter events" /></label></div>
      <div className="table-scroll"><table><thead>{table.getHeaderGroups().map((group) => <tr key={group.id}>{group.headers.map((header) => <th key={header.id}>{flexRender(header.column.columnDef.header, header.getContext())}</th>)}</tr>)}</thead><tbody>{table.getRowModel().rows.map((row) => <tr key={row.id}>{row.getVisibleCells().map((cell) => <td key={cell.id}>{flexRender(cell.column.columnDef.cell, cell.getContext())}</td>)}</tr>)}</tbody></table></div>
      <div className="table-foot">Showing {filtered.length} recent events <span>Raw events retained for 90 days</span></div>
    </Panel>
  </div>
}

function SummaryMetric({ icon, label, value, detail }: { icon: ReactNode; label: string; value: string; detail: string }) {
  return <Panel><span>{icon}</span><div><small>{label}</small><strong>{value}</strong><em>{detail}</em></div></Panel>
}

function ReportPanel({ eyebrow, title, detailTo, serverId, children }: { eyebrow: string; title: string; detailTo: 'commands' | 'latency' | 'automod'; serverId: string; children: ReactNode }) {
  const to = detailTo === 'commands' ? '/servers/$serverId/analytics/commands' : detailTo === 'latency' ? '/servers/$serverId/analytics/latency' : '/servers/$serverId/analytics/automod'
  return <Panel className="analytics-chart-panel"><div className="panel-heading"><div><p className="eyebrow">{eyebrow}</p><h3>{title}</h3></div><Link to={to} params={{ serverId }} className="text-link">Details →</Link></div>{children}</Panel>
}

type Report = {
  eyebrow: string
  title: string
  description: string
  data: ChartDatum[]
  type: 'line' | 'bar'
  xLabel: string
  yLabel: string
  unit?: string
  color: string
  metrics: Array<{ label: string; value: string; note: string; icon: typeof Radio }>
  insights: Array<{ title: string; detail: string; tone: 'green' | 'amber' | 'violet' }>
}

const reports: Record<'commands' | 'latency' | 'automod', Report> = {
  commands: {
    eyebrow: 'Analytics / Commands', title: 'Command usage', description: 'Which slash commands people use, when they use them, and whether they succeed.', data: commandUsage, type: 'bar', xLabel: 'Slash command', yLabel: 'Invocations today', color: '#9b76f6',
    metrics: [{ label: 'Total invocations', value: '3,846', note: '+12.4% day over day', icon: TerminalSquare }, { label: 'Success rate', value: '99.72%', note: '11 failed interactions', icon: ShieldCheck }, { label: 'Unique members', value: '1,924', note: '15.2% of server', icon: Radio }],
    insights: [{ title: '/rank is the most-used command', detail: 'It accounts for 33.4% of today’s command traffic.', tone: 'violet' }, { title: 'Failures are concentrated in /settings', detail: 'Eight of eleven failures were expired interactions.', tone: 'amber' }, { title: 'Evening activity is growing', detail: '20:00–22:00 UTC is up 18% week over week.', tone: 'green' }],
  },
  latency: {
    eyebrow: 'Analytics / Performance', title: 'Response latency', description: 'End-to-end slash-command response time, including Discord REST and persistence.', data: latencyTrend, type: 'line', xLabel: 'Time (UTC)', yLabel: 'p95 response latency', unit: ' ms', color: '#4aa8e8',
    metrics: [{ label: 'p50 latency', value: '38 ms', note: '-4 ms day over day', icon: Gauge }, { label: 'p95 latency', value: '94 ms', note: 'Below 150 ms target', icon: Radio }, { label: 'p99 latency', value: '184 ms', note: '3 slow requests', icon: Database }],
    insights: [{ title: 'All latency objectives are healthy', detail: 'The p95 has stayed below 150 ms for 29 days.', tone: 'green' }, { title: 'Discord REST dominates slow requests', detail: 'It contributes 61% of p99 response time.', tone: 'violet' }, { title: '18:00 UTC is the busiest window', detail: 'Higher concurrency added roughly 13 ms to p95.', tone: 'amber' }],
  },
  automod: {
    eyebrow: 'Analytics / Safety', title: 'Automod effectiveness', description: 'Rule matches and actions without retaining member message content.', data: automodActions, type: 'bar', xLabel: 'Rule category', yLabel: 'Actions today', color: '#e1a04b',
    metrics: [{ label: 'Actions today', value: '187', note: '-8.2% day over day', icon: ShieldCheck }, { label: 'Auto-resolved', value: '92%', note: '14 need review', icon: Radio }, { label: 'False-positive rate', value: '0.8%', note: '2 actions reversed', icon: Filter }],
    insights: [{ title: 'Spam remains the leading trigger', detail: '74 actions came from repeated-message and link patterns.', tone: 'amber' }, { title: 'Mention floods are down', detail: 'Matches fell 21% after raising slowmode in two channels.', tone: 'green' }, { title: 'No content is retained', detail: 'Analytics stores the rule ID and outcome only.', tone: 'violet' }],
  },
}

type ReportKind = keyof typeof reports

export function CommandsAnalyticsPage() { return <AnalyticsDetail kind="commands" /> }
export function LatencyAnalyticsPage() { return <AnalyticsDetail kind="latency" /> }
export function AutomodAnalyticsPage() { return <AnalyticsDetail kind="automod" /> }

function AnalyticsDetail({ kind }: { kind: ReportKind }) {
  const { serverId } = useParams({ strict: false }) as { serverId: string }
  const report = reports[kind]
  const summary = useQuery({ queryKey: ['guild-stats', serverId], queryFn: () => getGuildStats(serverId), refetchInterval: 60_000 })
  const volume = useQuery({ queryKey: ['command-volume', 'v2', serverId], queryFn: () => getCommandVolume(serverId), refetchInterval: 60_000 })
  const audit = useQuery({ queryKey: ['audit-events', serverId], queryFn: () => getAuditEvents(serverId), refetchInterval: 60_000 })
  const events = (audit.data ?? []).map(auditToAnalytics)
  const data = isDemoMode ? report.data : kind === 'commands' ? (volume.data ?? []) : kind === 'latency' ? latencyFromEvents(events) : automodFromEvents(events)
  const metrics = isDemoMode ? report.metrics : liveMetrics(kind, summary.data, events)
  const insights = isDemoMode ? report.insights : liveInsights(kind, data, events)
  return <div className="page-stack">
    <Link to="/servers/$serverId/analytics" params={{ serverId }} className="back-link"><ArrowLeft size={14} />Back to analytics</Link>
    <section className="page-intro"><div><p className="eyebrow">{report.eyebrow}</p><h2>{report.title}</h2><p>{report.description}</p></div><Button onClick={() => downloadCsv(`orivane-${serverId}-${report.title.toLowerCase().replaceAll(' ', '-')}.csv`, data, serverId)}><ArrowDownToLine size={16} />Export CSV</Button></section>
    <section className="analytics-summary report-summary">{metrics.map(({ label, value, note, icon: Icon }) => <SummaryMetric key={label} icon={<Icon size={17} />} label={label} value={value} detail={note} />)}</section>
    <Panel className="detail-chart-panel"><div className="panel-heading"><div><p className="eyebrow">Last 24 hours</p><h3>{report.title}</h3></div><Badge tone="neutral">UTC</Badge></div><MetricChart data={data} xLabel={report.xLabel} yLabel={report.yLabel} unit={report.unit} type={report.type} color={report.color} ariaLabel={`${report.title} chart`} /></Panel>
    <section className="report-bottom"><Panel className="insight-panel"><div className="panel-heading"><div><p className="eyebrow">Interpretation</p><h3>Key insights</h3></div></div>{insights.map((insight) => <div className="insight-row" key={insight.title}><Badge tone={insight.tone}>Insight</Badge><span><strong>{insight.title}</strong><small>{insight.detail}</small></span></div>)}</Panel><Panel className="report-notes"><p className="eyebrow">About this report</p><h3>Data details</h3><dl><div><dt>Timezone</dt><dd>UTC</dd></div><div><dt>Refresh</dt><dd>Every 60 seconds</dd></div><div><dt>Raw retention</dt><dd>90 days</dd></div><div><dt>Message content</dt><dd>Never stored</dd></div></dl></Panel></section>
  </div>
}

function auditToAnalytics(event: Awaited<ReturnType<typeof getAuditEvents>>[number]): AnalyticsEvent {
  return {
    id: String(event.id),
    event: event.action,
    module: String(event.metadata.module ?? event.source),
    outcome: event.outcome === 'failure' ? 'failed' : event.outcome === 'denied' ? 'blocked' : 'success',
    latency: Number(event.metadata.latency_ms ?? 0),
    occurredAt: new Date(event.occurred_at).toLocaleString(),
  }
}

function latencyFromEvents(events: AnalyticsEvent[]): ChartDatum[] {
  return events
    .filter((event) => event.event.startsWith('slash_command.') && event.latency > 0)
    .slice(0, 12)
    .reverse()
    .map((event) => ({ label: shortTime(event.occurredAt), value: event.latency, detail: `${event.event} · ${event.outcome}` }))
}

function automodFromEvents(events: AnalyticsEvent[]): ChartDatum[] {
  const counts = new Map<string, number>()
  for (const event of events.filter((item) => item.event.startsWith('automod.'))) {
    const category = event.event.split('.')[1]?.replaceAll('_', ' ') ?? 'other'
    counts.set(category, (counts.get(category) ?? 0) + 1)
  }
  return [...counts].map(([label, value]) => ({ label, value, detail: `${value.toLocaleString()} audited action${value === 1 ? '' : 's'}` }))
}

function shortTime(value: string) {
  const date = new Date(value)
  return Number.isNaN(date.getTime()) ? value : date.toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' })
}

function liveMetrics(kind: ReportKind, stats: Awaited<ReturnType<typeof getGuildStats>> | undefined, events: AnalyticsEvent[]): Report['metrics'] {
  const commandEvents = events.filter((event) => event.event.startsWith('slash_command.'))
  const successfulCommands = commandEvents.filter((event) => event.outcome === 'success').length
  const latencies = commandEvents.map((event) => event.latency).filter((value) => value > 0).sort((a, b) => a - b)
  const percentile = (position: number) => latencies.length ? `${Math.round(latencies[Math.min(latencies.length - 1, Math.floor((latencies.length - 1) * position))])} ms` : '—'
  if (kind === 'commands') return [
    { label: 'Total invocations', value: stats?.commands_today.toLocaleString() ?? '—', note: 'Audited today', icon: TerminalSquare },
    { label: 'Success rate', value: commandEvents.length ? `${(successfulCommands / commandEvents.length * 100).toFixed(2)}%` : '—', note: `${commandEvents.length} recent audited interactions`, icon: ShieldCheck },
    { label: 'Active members', value: stats?.active_members_30d.toLocaleString() ?? '—', note: 'Distinct command users in 30 days', icon: Radio },
  ]
  if (kind === 'latency') return [
    { label: 'p50 latency', value: percentile(0.5), note: `${latencies.length} recent measured requests`, icon: Gauge },
    { label: 'p95 latency', value: stats?.p95_latency_ms == null ? '—' : `${Math.round(stats.p95_latency_ms)} ms`, note: 'Computed from audited commands', icon: Radio },
    { label: 'p99 latency', value: percentile(0.99), note: latencies.length ? 'Recent retained sample' : 'No latency samples yet', icon: Database },
  ]
  return [
    { label: 'Actions today', value: stats?.automod_actions_today.toLocaleString() ?? '—', note: 'Audited automod actions', icon: ShieldCheck },
    { label: 'Recent successes', value: events.filter((event) => event.event.startsWith('automod.') && event.outcome === 'success').length.toLocaleString(), note: 'Within retained event results', icon: Radio },
    { label: 'Needs review', value: events.filter((event) => event.event.startsWith('automod.') && event.outcome !== 'success').length.toLocaleString(), note: 'Blocked or failed outcomes', icon: Filter },
  ]
}

function liveInsights(kind: ReportKind, data: ChartDatum[], events: AnalyticsEvent[]): Report['insights'] {
  const sampleCount = kind === 'commands' ? events.filter((event) => event.event.startsWith('slash_command.')).length : kind === 'latency' ? data.length : events.filter((event) => event.event.startsWith('automod.')).length
  const measured = sampleCount
    ? { title: `${sampleCount.toLocaleString()} recent events measured`, detail: 'This report is derived from the retained audit stream.', tone: 'green' as const }
    : { title: 'No measured events yet', detail: 'This report will populate as the bot records matching activity.', tone: 'amber' as const }
  return [
    measured,
    { title: 'Figures are scoped to this server', detail: 'Global owner activity and other guilds are excluded.', tone: 'violet' },
    { title: 'No message content is retained', detail: 'Only operational metadata, outcomes, and timing are used.', tone: 'green' },
  ]
}

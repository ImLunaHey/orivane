import { Link } from '@tanstack/react-router'
import type { ColumnDef } from '@tanstack/react-table'
import { Activity, Bot, Command, Radio, Server, ShieldCheck, Users, Zap } from 'lucide-react'
import { useState } from 'react'
import { useQuery } from '@tanstack/react-query'
import { DataTable } from '../components/data-table'
import { MetricChart } from '../components/metric-chart'
import { Badge, Panel } from '../components/ui'
import type { ChartDatum } from '../lib/analytics-data'
import { demoServers, type ConnectedServer } from '../lib/server-data'
import { getCommandVolume, getServers, isDemoMode } from '../lib/api'

const fleetTraffic: ChartDatum[] = [
  { label: '00:00', value: 184, detail: 'Across 248 active servers · 99.8% success' },
  { label: '04:00', value: 241, detail: 'Utility commands were 41% of traffic' },
  { label: '08:00', value: 396, detail: 'European server activity increased' },
  { label: '12:00', value: 512, detail: 'Peak concurrency: 68 interactions' },
  { label: '16:00', value: 604, detail: 'Moderation commands: 84' },
  { label: '20:00', value: 728, detail: 'Daily fleet peak · 99.7% success' },
  { label: 'Now', value: 681, detail: 'Current four-hour window' },
]

type Installer = { id: string; name: string; servers: number; activity: number; largest: string }

const serverColumns: Array<ColumnDef<ConnectedServer>> = [
  { accessorKey: 'name', header: 'Server', cell: ({ row }) => <Link className="table-primary-link" to="/servers/$serverId" params={{ serverId: row.original.id }}><span className="guild-avatar">{row.original.name.charAt(0)}</span><span><strong>{row.original.name}</strong><small>{row.original.id}</small></span></Link> },
  { accessorKey: 'members', header: 'Members', cell: ({ getValue }) => getValue<number>().toLocaleString() },
  { accessorKey: 'commands', header: 'Commands today', cell: ({ getValue }) => getValue<number>().toLocaleString() },
  { accessorKey: 'installer', header: 'Installed by', cell: ({ row }) => row.original.installerId ? <Link className="table-user-link" to="/users/$userId" params={{ userId: row.original.installerId }}>{row.original.installer}</Link> : <span>Unattributed</span> },
  { accessorKey: 'shard', header: 'Shard', cell: ({ getValue }) => `Shard ${getValue<number>()}` },
  { accessorKey: 'status', header: 'Status', cell: ({ getValue }) => { const status = getValue<string>(); return <Badge tone={status === 'Degraded' ? 'amber' : 'green'}>{status}</Badge> } },
  { id: 'open', header: 'Actions', enableSorting: false, cell: ({ row }) => <Link className="text-link" to="/servers/$serverId" params={{ serverId: row.original.id }}>Open →</Link> },
]

const installerColumns: Array<ColumnDef<Installer>> = [
  { accessorKey: 'name', header: 'Installer', cell: ({ row }) => <Link className="table-person" to="/users/$userId" params={{ userId: row.original.id }}><span className="case-avatar">{row.original.name.slice(0, 2).toUpperCase()}</span><span><strong>{row.original.name}</strong><small>{row.original.id}</small></span></Link> },
  { accessorKey: 'servers', header: 'Servers' },
  { accessorKey: 'activity', header: 'Install commands (30d)', cell: ({ getValue }) => getValue<number>().toLocaleString() },
  { accessorKey: 'largest', header: 'Largest install' },
]

export function BotOverviewPage() {
  const servers = useQuery({ queryKey: ['servers'], queryFn: getServers })
  const traffic = useQuery({ queryKey: ['command-volume', 'v2', 'global'], queryFn: () => getCommandVolume(), initialData: isDemoMode ? fleetTraffic : [] })
  const fleet = servers.data ?? []
  const memberTotal = fleet.reduce((total, server) => total + server.members, 0)
  const commandTotal = fleet.reduce((total, server) => total + server.commands, 0)
  const attributedInstallers = new Set(fleet.map((server) => server.installerId).filter(Boolean))
  const multiInstallers = [...attributedInstallers].filter((id) => fleet.filter((server) => server.installerId === id).length > 1).length
  const sortedSizes = fleet.map((server) => server.members).sort((a, b) => a - b)
  const median = sortedSizes.length ? sortedSizes[Math.floor(sortedSizes.length / 2)] : 0
  const largest = fleet[0]
  const installers = [...attributedInstallers].map((id) => {
    const installed = fleet.filter((server) => server.installerId === id)
    return { id, name: `${id.slice(0, 4)}…${id.slice(-4)}`, servers: installed.length, activity: installed.reduce((total, server) => total + server.commands, 0), largest: [...installed].sort((a, b) => b.members - a.members)[0]?.name ?? '—' }
  }).filter((installer) => installer.servers > 1)
  const sizeBuckets = [
    { label: '<100', value: fleet.filter((server) => server.members < 100).length },
    { label: '100–1k', value: fleet.filter((server) => server.members >= 100 && server.members < 1_000).length },
    { label: '1k–10k', value: fleet.filter((server) => server.members >= 1_000 && server.members < 10_000).length },
    { label: '10k–50k', value: fleet.filter((server) => server.members >= 10_000 && server.members < 50_000).length },
    { label: '50k+', value: fleet.filter((server) => server.members >= 50_000).length },
  ].map((bucket) => ({ ...bucket, detail: `${fleet.length ? (bucket.value / fleet.length * 100).toFixed(1) : '0.0'}% of connected servers` }))
  return <div className="page-stack">
    <section className="page-intro"><div><p className="eyebrow">Global operations</p><h2>Orivane bot</h2><p>Fleet-wide health, traffic, and growth across every connected Discord server.</p></div><Link to="/servers" className="button"><Server size={16} />View all servers</Link></section>
    <section className="stats-grid"><BotMetric icon={Server} label="Connected servers" value={fleet.length.toLocaleString()} note="Currently connected" /><BotMetric icon={Users} label="Total members" value={memberTotal.toLocaleString()} note="Non-unique server memberships" /><BotMetric icon={Command} label="Commands today" value={commandTotal.toLocaleString()} note="Audited slash commands" /><BotMetric icon={Activity} label="Active shards" value={String(new Set(fleet.map((server) => server.shard)).size)} note="Reporting shards" /></section>
    <section className="bot-secondary-metrics"><Panel><small>Attributed installers</small><strong>{attributedInstallers.size.toLocaleString()}</strong><span>Known Discord install owners</span></Panel><Panel><small>Multi-server installers</small><strong>{multiInstallers.toLocaleString()}</strong><span>Attributed to 2+ installs</span></Panel><Panel><small>Largest server</small><strong>{largest?.members.toLocaleString() ?? '—'}</strong><span>{largest ? `members · ${largest.name}` : 'No connected guild data'}</span></Panel><Panel><small>Median server</small><strong>{median.toLocaleString()}</strong><span>members per connected guild</span></Panel></section>
    <section className="dashboard-grid">
      <Panel className="chart-panel"><div className="panel-heading"><div><p className="eyebrow">Fleet traffic</p><h3>Commands across all servers</h3></div><Badge tone="green">Audited</Badge></div><MetricChart data={traffic.data ?? []} xLabel="Time (UTC)" yLabel="Commands / 4h" ariaLabel="Bot-wide command volume across all Discord servers" /></Panel>
      <Panel className="health-panel"><div className="panel-heading"><div><p className="eyebrow">Infrastructure</p><h3>Global runtime</h3></div><Badge tone="neutral">Measured</Badge></div><div className="health-list"><div><span className="service-icon violet"><Bot size={17} /></span><span><strong>Gateway footprint</strong><small>{new Set(fleet.map((server) => server.shard)).size} reporting shards</small></span><em>{fleet.length} guilds</em></div><div><span className="service-icon blue"><Radio size={17} /></span><span><strong>Command stream</strong><small>Audited since UTC midnight</small></span><em>{commandTotal.toLocaleString()}</em></div><div><span className="service-icon green"><ShieldCheck size={17} /></span><span><strong>Install attribution</strong><small>Known installation actors</small></span><em>{attributedInstallers.size.toLocaleString()}</em></div></div></Panel>
    </section>
    <section className="dashboard-grid bot-insights-grid">
      <Panel className="chart-panel"><div className="panel-heading"><div><p className="eyebrow">Install footprint</p><h3>Servers by member count</h3></div><Badge tone="violet">{fleet.length} total</Badge></div><MetricChart data={sizeBuckets} xLabel="Server size (members)" yLabel="Connected servers" type="bar" color="#4aa8e8" ariaLabel="Distribution of connected Discord servers by member count" /></Panel>
      <Panel className="system-panel"><div className="panel-heading"><div><p className="eyebrow">Install correlation</p><h3>Users with multiple servers</h3></div><Badge tone="neutral">{installers.length.toLocaleString()} users</Badge></div><DataTable data={installers} columns={installerColumns} getRowId={(row) => row.id} ariaLabel="Users who installed Orivane in multiple servers" /></Panel>
    </section>
    <Panel className="system-panel"><div className="panel-heading"><div><p className="eyebrow">Server fleet</p><h3>Largest servers</h3></div><Link to="/servers" className="text-link">All servers →</Link></div><ServerDirectory servers={fleet.slice(0, 4)} /></Panel>
  </div>
}

export function ServersPage() {
  const [query, setQuery] = useState('')
  const servers = useQuery({ queryKey: ['servers'], queryFn: getServers })
  const filtered = (servers.data ?? []).filter((server) => `${server.name} ${server.id} ${server.status} ${server.installer}`.toLowerCase().includes(query.toLowerCase()))
  return <div className="page-stack"><section className="page-intro"><div><p className="eyebrow">Global operations</p><h2>Servers</h2><p>Select a Discord server to open its configuration, moderation, analytics, and members.</p></div><Badge tone="violet">{(servers.data?.length ?? 0).toLocaleString()} connected</Badge></section><Panel className="system-panel"><div className="table-toolbar"><div><p className="eyebrow">Directory</p><h3>Connected servers</h3></div><label className="search-box"><Server size={15} /><input aria-label="Search connected servers" value={query} onChange={(event) => setQuery(event.target.value)} placeholder="Find a server" /></label></div><DataTable data={filtered} columns={serverColumns} getRowId={(row) => row.id} ariaLabel="Connected Discord servers" emptyMessage={servers.isLoading ? 'Loading connected servers…' : 'No connected servers found'} /></Panel></div>
}

function ServerDirectory({ servers }: { servers: typeof demoServers }) {
  return <DataTable data={servers} columns={serverColumns} getRowId={(row) => row.id} ariaLabel="Highest activity Discord servers" />
}

function BotMetric({ icon: Icon, label, value, note }: { icon: typeof Zap; label: string; value: string; note: string }) {
  return <Panel className="stat-card"><div className="stat-icon"><Icon size={18} /></div><p>{label}</p><strong>{value}</strong><span className="stat-change positive">{note}</span></Panel>
}

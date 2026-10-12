import { Link, useParams } from '@tanstack/react-router'
import { Activity, ArrowLeft, CalendarDays, CheckCircle2, Clock3, Command, Database, Gauge, LoaderCircle, MessageSquare, Search, ShieldCheck, UserRoundCheck, Users } from 'lucide-react'
import { useState } from 'react'
import { useMutation, useQuery } from '@tanstack/react-query'
import { Badge, Button, Panel } from '../components/ui'
import { downloadCsv } from '../lib/export'
import { getAuditEvents, getGuildMembers, getGuildStats, getHealthReport, isDemoMode, syncCommands } from '../lib/api'

const commands = [
  { name: '/ping', module: 'Utility', description: 'Check whether the bot is responding.', uses: '1,284', success: '99.99%', p95: '42 ms' },
  { name: '/help', module: 'Utility', description: 'Show commands enabled in this server.', uses: '438', success: '99.99%', p95: '39 ms' },
  { name: '/serverinfo', module: 'Utility', description: 'Show this server’s bot configuration summary.', uses: '214', success: '99.53%', p95: '44 ms' },
]

const members = [
  { id: '118742006320914432', name: 'silver.arc', displayName: 'Silver', role: 'Member', joined: 'Today', joinedAt: '11 Oct 2026, 18:42', activity: 'High', cases: 1, messages: '184', commands: '23', lastSeen: '4 minutes ago', roles: ['Member', 'Level 10'], accountAge: '4 years', risk: 'Low' },
  { id: '227843117431025543', name: 'nocturne', displayName: 'Nocturne', role: 'Member', joined: '3 days ago', joinedAt: '8 Oct 2026, 09:14', activity: 'Medium', cases: 2, messages: '96', commands: '12', lastSeen: '2 hours ago', roles: ['Member', 'Art'], accountAge: '2 years', risk: 'Medium' },
  { id: '336954228542136654', name: 'mossbyte', displayName: 'Moss', role: 'Moderator', joined: '2 months ago', joinedAt: '12 Aug 2026, 15:05', activity: 'High', cases: 0, messages: '2,814', commands: '318', lastSeen: 'Online now', roles: ['Moderator', 'Member', 'Level 40'], accountAge: '6 years', risk: 'Low' },
  { id: '445065339653247765', name: 'orbitals', displayName: 'Orbitals', role: 'Member', joined: '4 months ago', joinedAt: '2 Jun 2026, 11:37', activity: 'Low', cases: 0, messages: '341', commands: '18', lastSeen: '12 days ago', roles: ['Member'], accountAge: '3 years', risk: 'Low' },
  { id: '554176440764358876', name: 'papercrane', displayName: 'Paper Crane', role: 'Administrator', joined: '1 year ago', joinedAt: '19 Sep 2025, 20:01', activity: 'High', cases: 0, messages: '8,419', commands: '754', lastSeen: 'Online now', roles: ['Administrator', 'Moderator', 'Member'], accountAge: '8 years', risk: 'Low' },
]

export function HealthPage() {
  const health = useQuery({ queryKey: ['health'], queryFn: getHealthReport, refetchInterval: 30_000 })
  const report = health.data
  const services = report?.services ?? []
  const operational = report?.status === 'operational'
  return <div className="page-stack"><PageIntro eyebrow="System" title="Health" description="Live operational state for the bot and its supporting services." action={<Button disabled={health.isFetching} onClick={() => health.refetch()}><Activity size={16} />Refresh</Button>} />
    <section className="system-metrics"><Metric icon={CheckCircle2} label="Overall status" value={report ? (operational ? 'Operational' : 'Degraded') : 'Checking'} note="Live dependency probes" /><Metric icon={Clock3} label="Refresh interval" value="30 sec" note="Automatic health checks" /><Metric icon={Gauge} label="Dependencies" value={String(services.length)} note="Directly probed services" /><Metric icon={ShieldCheck} label="Data source" value="Live" note="No synthetic uptime claims" /></section>
    <Panel className="system-panel"><div className="panel-heading"><div><p className="eyebrow">Dependencies</p><h3>Services</h3></div><Badge tone={operational ? 'green' : 'amber'}>{report ? (operational ? 'All operational' : 'Degraded') : 'Checking'}</Badge></div><div className="service-table">{services.map(({ name, status, latency_ms: latency }) => { const Icon = name === 'Redis' ? Gauge : Database; return <div key={name}><span className="system-icon"><Icon size={17} /></span><span><strong>{name}</strong><small>Direct API dependency probe</small></span><Badge tone={status === 'operational' ? 'green' : 'red'}>{status}</Badge><em>{latency} ms</em></div> })}</div></Panel>
  </div>
}

export function CommandsPage() {
  const [query, setQuery] = useState('')
  const sync = useMutation({ mutationFn: syncCommands })
  const audit = useQuery({ queryKey: ['audit-events', 'global'], queryFn: () => getAuditEvents() })
  const commandDefinitions = [{ name: '/ping', description: 'Check whether the bot is responding.' }, { name: '/help', description: 'Show commands enabled in this server.' }, { name: '/serverinfo', description: 'Show this server’s bot configuration summary.' }]
  const liveCommands = commandDefinitions.map((definition) => {
    const events = (audit.data ?? []).filter((event) => event.action === `slash_command.${definition.name.slice(1)}`)
    const latencies = events.map((event) => Number(event.metadata.latency_ms)).filter(Number.isFinite).sort((a, b) => a - b)
    const p95 = latencies.length ? latencies[Math.min(latencies.length - 1, Math.floor((latencies.length - 1) * 0.95))] : null
    return { ...definition, module: 'Utility', uses: events.length.toLocaleString(), success: events.length ? `${(events.filter((event) => event.outcome === 'success').length / events.length * 100).toFixed(2)}%` : '—', p95: p95 == null ? '—' : `${Math.round(p95)} ms` }
  })
  const registry = isDemoMode ? commands : liveCommands
  const filtered = registry.filter((command) => `${command.name} ${command.module} ${command.description}`.toLowerCase().includes(query.toLowerCase()))
  return <div className="page-stack"><PageIntro eyebrow="System" title="Commands" description="Slash-command registry, usage, and reliability." action={<Button disabled={sync.isPending} onClick={() => sync.mutate()}>{sync.isPending ? <LoaderCircle className="spin" size={16} /> : <Command size={16} />}Sync commands</Button>} />
    {sync.isSuccess && <div className="success-banner" role="status">Synchronized {sync.data.synchronized} slash command{sync.data.synchronized === 1 ? '' : 's'} with Discord.</div>}
    {sync.isError && <div className="error-banner" role="alert">{sync.error.message}</div>}
    <Panel className="system-panel"><div className="table-toolbar"><div><h3>Global command registry</h3><Badge tone="violet">{registry.length} commands</Badge></div><label className="search-box"><Search size={15} /><input aria-label="Search slash commands" value={query} onChange={(event) => setQuery(event.target.value)} placeholder="Find a command" /></label></div><div className="command-list">{filtered.map((item) => <div key={item.name}><code>{item.name}</code><span><strong>{item.description}</strong><small>{item.module}</small></span><span><strong>{item.uses}</strong><small>recent audited uses</small></span><span><strong>{item.success}</strong><small>success</small></span><span><strong>{item.p95}</strong><small>p95</small></span></div>)}</div></Panel>
  </div>
}

export function MembersPage() {
  const { serverId } = useParams({ strict: false }) as { serverId: string }
  const [query, setQuery] = useState('')
  const observed = useQuery({ queryKey: ['guild-members', serverId], queryFn: () => getGuildMembers(serverId) })
  const summary = useQuery({ queryKey: ['guild-stats', serverId], queryFn: () => getGuildStats(serverId) })
  const directory = isDemoMode ? members : (observed.data ?? []).map(observedMember)
  const filtered = directory.filter((member) => `${member.name} ${member.id}`.toLowerCase().includes(query.toLowerCase()))
  return <div className="page-stack"><PageIntro eyebrow="Community" title="Members" description="Member activity, roles, and moderation context." action={<Button onClick={() => downloadCsv(`orivane-${serverId}-members.csv`, filtered, serverId)}><Users size={16} />Export members</Button>} />
    <section className="system-metrics three"><Metric icon={Users} label="Observed members" value={directory.length.toLocaleString()} note="Seen in commands or moderation" /><Metric icon={UserRoundCheck} label="Active (30d)" value={summary.data?.active_members_30d.toLocaleString() ?? '—'} note="Distinct slash-command users" /><Metric icon={ShieldCheck} label="Open cases" value={summary.data?.open_cases.toLocaleString() ?? '—'} note="Current moderation cases" /></section>
    <Panel className="system-panel"><div className="table-toolbar"><div><h3>Member directory</h3><Badge tone="neutral">{isDemoMode ? 'Demo data' : 'Observed identities'}</Badge></div><label className="search-box"><Search size={15} /><input aria-label="Search server members" value={query} onChange={(event) => setQuery(event.target.value)} placeholder="Find a member" /></label></div><div className="member-list">{filtered.map((member) => <Link key={member.id} to="/servers/$serverId/members/$memberId" params={{ serverId, memberId: member.id }}><span className="case-avatar">{member.name.slice(-2).toUpperCase()}</span><span><strong>{member.name}</strong><small>{member.id} · {member.lastSeen}</small></span><Badge tone={member.role === 'Member' ? 'neutral' : 'violet'}>{member.role}</Badge><span><strong>{member.activity}</strong><small>activity</small></span><span><strong>{member.cases}</strong><small>cases</small></span><em>View →</em></Link>)}</div></Panel>
  </div>
}

export function MemberDetailPage() {
  const { serverId, memberId } = useParams({ strict: false }) as { serverId: string; memberId: string }
  const observed = useQuery({ queryKey: ['guild-members', serverId], queryFn: () => getGuildMembers(serverId) })
  const member = isDemoMode ? members.find((item) => item.id === memberId) : observed.data?.map(observedMember).find((item) => item.id === memberId)
  if (!isDemoMode && observed.isLoading) return <div className="loading-state">Loading member details…</div>
  if (!member) return <div className="empty-state"><h2>Member not found</h2><Link to="/servers/$serverId/members" params={{ serverId }}>Return to members</Link></div>

  return <div className="page-stack">
    <Link to="/servers/$serverId/members" params={{ serverId }} className="back-link"><ArrowLeft size={14} />Back to members</Link>
    <section className="member-profile-header"><span className="member-profile-avatar">{member.name.slice(0, 2).toUpperCase()}</span><div><p className="eyebrow">Discord member · {member.id}</p><h2>{member.displayName}</h2><p>@{member.name} · Last seen {member.lastSeen}</p></div><Badge tone={member.risk === 'Low' ? 'green' : 'amber'}>{member.risk} risk</Badge></section>
    <section className="system-metrics"><Metric icon={MessageSquare} label="Messages (30d)" value={member.messages} note={`${member.activity} activity`} /><Metric icon={Command} label="Commands (30d)" value={member.commands} note="Slash commands only" /><Metric icon={ShieldCheck} label="Moderation cases" value={String(member.cases)} note={member.cases ? 'Review available below' : 'No recorded cases'} /><Metric icon={CalendarDays} label="Account age" value={member.accountAge} note={`Joined ${member.joined}`} /></section>
    <section className="member-detail-grid"><Panel className="profile-panel"><div className="panel-heading"><div><p className="eyebrow">Profile</p><h3>Server context</h3></div></div><dl className="detail-list"><div><dt>Discord ID</dt><dd>{member.id}</dd></div><div><dt>Joined server</dt><dd>{member.joinedAt}</dd></div><div><dt>Highest role</dt><dd>{member.role}</dd></div><div><dt>Last active</dt><dd>{member.lastSeen}</dd></div></dl><div className="role-list"><p className="eyebrow">Roles</p>{member.roles.map((role) => <Badge key={role} tone={role === 'Member' ? 'neutral' : 'violet'}>{role}</Badge>)}</div></Panel><Panel className="profile-panel"><div className="panel-heading"><div><p className="eyebrow">Moderation</p><h3>Case history</h3></div><Badge tone={member.cases ? 'amber' : 'green'}>{member.cases} cases</Badge></div>{member.cases ? <div className="member-case"><span>#1042</span><div><strong>Automod warning</strong><small>Repeated message pattern · resolved by mossbyte</small></div><time>Today, 19:04</time></div> : <div className="quiet-state"><ShieldCheck size={24} /><strong>No moderation history</strong><span>This member has no recorded cases.</span></div>}</Panel></section>
  </div>
}

function observedMember(member: Awaited<ReturnType<typeof getGuildMembers>>[number]): typeof members[number] {
  return {
    id: member.id,
    name: member.username ?? `Discord user ${member.id.slice(-4)}`,
    displayName: member.display_name ?? member.username ?? `Discord user ${member.id.slice(-4)}`,
    role: 'Observed',
    joined: 'Not tracked',
    joinedAt: 'Not available from operational telemetry',
    activity: member.commands_30d ? 'Active' : 'No command activity',
    cases: member.cases,
    messages: 'Not tracked',
    commands: member.commands_30d.toLocaleString(),
    lastSeen: member.last_seen ? new Date(member.last_seen).toLocaleString() : 'No activity timestamp',
    roles: [],
    accountAge: 'Not tracked',
    risk: member.cases ? 'Medium' : 'Low',
  }
}

function PageIntro({ eyebrow, title, description, action }: { eyebrow: string; title: string; description: string; action: React.ReactNode }) {
  return <section className="page-intro"><div><p className="eyebrow">{eyebrow}</p><h2>{title}</h2><p>{description}</p></div>{action}</section>
}

function Metric({ icon: Icon, label, value, note }: { icon: typeof Activity; label: string; value: string; note: string }) {
  return <Panel><span className="system-icon"><Icon size={17} /></span><div><small>{label}</small><strong>{value}</strong><em>{note}</em></div></Panel>
}

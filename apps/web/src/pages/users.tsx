import { Link, useParams } from '@tanstack/react-router'
import type { ColumnDef } from '@tanstack/react-table'
import { ArrowLeft, Clock3, Command, Download, Network, Server, ShieldCheck, UserRoundSearch, Users } from 'lucide-react'
import { useState } from 'react'
import { useQuery } from '@tanstack/react-query'
import { DataTable } from '../components/data-table'
import { DiscordAvatar } from '../components/discord-avatar'
import { UserAuditHistory } from '../components/user-audit-history'
import { Badge, Button, Panel } from '../components/ui'
import { globalUsers, type GlobalUser, type UserServerContext } from '../lib/user-data'
import { downloadCsv } from '../lib/export'
import { getUser, getUsers, isDemoMode } from '../lib/api'
import { UserMetric } from './users/user-metric'

const userColumns: Array<ColumnDef<GlobalUser>> = [
  { accessorKey: 'username', header: 'User', cell: ({ row }) => <Link className="table-primary-link" to="/users/$userId" params={{ userId: row.original.id }}><DiscordAvatar className="case-avatar" avatarUrl={row.original.avatarUrl} name={row.original.username} /><span><strong>{row.original.displayName}</strong><small>@{row.original.username} · {row.original.id}</small></span></Link> },
  { accessorKey: 'servers', header: 'Servers' },
  { accessorKey: 'installedServers', header: 'Installs', cell: ({ getValue }) => { const count = getValue<number>(); return count ? <Badge tone="violet">{count}</Badge> : '—' } },
  { accessorKey: 'commands', header: 'Commands (30d)', cell: ({ getValue }) => getValue<number>().toLocaleString() },
  { accessorKey: 'cases', header: 'Cases', cell: ({ getValue }) => { const count = getValue<number>(); return <Badge tone={count ? 'amber' : 'green'}>{count}</Badge> } },
  { accessorKey: 'lastSeen', header: 'Last active' },
  { id: 'open', header: 'Actions', enableSorting: false, cell: ({ row }) => <Link className="text-link" to="/users/$userId" params={{ userId: row.original.id }}>Details →</Link> },
]

const contextColumns: Array<ColumnDef<UserServerContext>> = [
  { accessorKey: 'serverName', header: 'Server', cell: ({ row }) => <Link className="table-primary-link" to="/servers/$serverId" params={{ serverId: row.original.serverId }}><span className="guild-avatar">{row.original.serverName.charAt(0)}</span><span><strong>{row.original.serverName}</strong><small>{row.original.serverId}</small></span></Link> },
  { accessorKey: 'relationship', header: 'Relationship', cell: ({ getValue }) => { const value = getValue<UserServerContext['relationship']>(); return <Badge tone={value === 'Member' ? 'neutral' : 'violet'}>{value}</Badge> } },
  { accessorKey: 'commands', header: 'Commands (30d)' },
  { accessorKey: 'cases', header: 'Cases', cell: ({ getValue }) => { const count = getValue<number>(); return <Badge tone={count ? 'amber' : 'green'}>{count}</Badge> } },
  { accessorKey: 'lastSeen', header: 'Last active' },
]

export function UsersPage() {
  const [query, setQuery] = useState('')
  const users = useQuery({ queryKey: ['global-users'], queryFn: getUsers, initialData: isDemoMode ? globalUsers : undefined })
  const directory = users.data ?? []
  const filtered = directory.filter((user) => `${user.username} ${user.displayName} ${user.id} ${user.kind}`.toLowerCase().includes(query.toLowerCase()))
  const multiServer = directory.filter((user) => user.servers > 1).length
  const installers = directory.filter((user) => user.installedServers > 0)
  const multiInstallers = installers.filter((user) => user.installedServers > 1).length
  const withCases = directory.filter((user) => user.cases > 0).length
  return <div className="page-stack">
    <section className="page-intro"><div><p className="eyebrow">Cross-server identity</p><h2>Global users</h2><p>Discord users observed across the bot, correlated by Discord user ID rather than server membership.</p></div><Button onClick={() => downloadCsv('orivane-global-users.csv', directory.map(({ contexts, ...user }) => ({ ...user, serverContexts: contexts.length })))}><Download size={16} />Export users</Button></section>
    <section className="system-metrics"><UserMetric icon={Users} label="Observed identities" value={directory.length.toLocaleString()} note="From audited activity" /><UserMetric icon={Network} label="Multi-server users" value={multiServer.toLocaleString()} note="Seen in 2+ servers" /><UserMetric icon={UserRoundSearch} label="Attributed installers" value={installers.length.toLocaleString()} note={`${multiInstallers} attributed to 2+ installs`} /><UserMetric icon={ShieldCheck} label="Users with cases" value={withCases.toLocaleString()} note={directory.length ? `${(withCases / directory.length * 100).toFixed(1)}% of observed identities` : 'No user data yet'} /></section>
    <Panel className="system-panel"><div className="table-toolbar"><div><p className="eyebrow">Identity directory</p><h3>Users across all servers</h3></div><label className="search-box"><UserRoundSearch size={16} /><input aria-label="Search global users" value={query} onChange={(event) => setQuery(event.target.value)} placeholder="Name or Discord ID" /></label></div><DataTable data={filtered} columns={userColumns} getRowId={(row) => row.id} ariaLabel="Global Discord users" emptyMessage={users.isLoading ? 'Loading global users…' : 'No users found'} /></Panel>
  </div>
}
export function UserDetailPage() {
  const { userId } = useParams({ strict: false }) as { userId: string }
  const userQuery = useQuery({ queryKey: ['global-user', userId], queryFn: () => getUser(userId), initialData: isDemoMode ? globalUsers.find((item) => item.id === userId) : undefined })
  const user = userQuery.data
  if (userQuery.isLoading) return <div className="loading-state">Loading global user…</div>
  if (!user) return <div className="empty-state"><h2>User not found</h2><Link to="/users">Return to global users</Link></div>

  return <div className="page-stack">
    <Link to="/users" className="back-link"><ArrowLeft size={14} />Back to global users</Link>
    <section className="member-profile-header"><DiscordAvatar className="member-profile-avatar" avatarUrl={user.avatarUrl} name={user.username} /><div><p className="eyebrow">Global Discord user · {user.id}</p><h2>{user.displayName}</h2><p>@{user.username} · Last seen {user.lastSeen}</p></div><Badge tone={user.kind === 'Installer' ? 'violet' : 'neutral'}>{user.kind}</Badge></section>
    <section className="system-metrics"><UserMetric icon={Server} label="Shared servers" value={String(user.servers)} note="Correlated by Discord ID" /><UserMetric icon={Network} label="Bot installations" value={String(user.installedServers)} note={user.installedServers > 1 ? 'Multi-server installer' : user.installedServers ? 'Single install' : 'Has not installed bot'} /><UserMetric icon={Command} label="Commands (30d)" value={user.commands.toLocaleString()} note="Across all servers" /><UserMetric icon={ShieldCheck} label="Moderation cases" value={String(user.cases)} note="Across all servers" /></section>
    <UserAuditHistory userId={user.id} />
    <Panel className="system-panel"><div className="panel-heading"><div><p className="eyebrow">Cross-server footprint</p><h3>Server relationships</h3></div><Badge tone="neutral">{user.contexts.length} loaded</Badge></div><DataTable data={user.contexts} columns={contextColumns} getRowId={(row) => row.serverId} ariaLabel={`Server relationships for ${user.username}`} /></Panel>
    <section className="member-detail-grid"><Panel className="profile-panel"><div className="panel-heading"><div><p className="eyebrow">Identity</p><h3>Global profile</h3></div></div><dl className="detail-list"><div><dt>Discord ID</dt><dd>{user.id}</dd></div><div><dt>Username</dt><dd>@{user.username}</dd></div><div><dt>Account age</dt><dd>{user.accountAge}</dd></div><div><dt>Last active</dt><dd>{user.lastSeen}</dd></div><div><dt>Commands from installs (30d)</dt><dd>{user.installedServers ? user.installCommands30d.toLocaleString() : 'Not an installer'}</dd></div></dl></Panel><Panel className="profile-panel"><div className="panel-heading"><div><p className="eyebrow">Correlation</p><h3>What this means</h3></div></div><div className="global-user-notes"><span><Network size={17} /><div><strong>Identity matched by Discord ID</strong><small>Names and display names can change; the snowflake ID is the stable key.</small></div></span><span><Clock3 size={17} /><div><strong>Install activity is attributed</strong><small>Install command volume measures actual bot use in servers this person installed it in.</small></div></span><span><ShieldCheck size={17} /><div><strong>Server context stays separate</strong><small>Roles and moderation cases remain scoped to the server where they occurred.</small></div></span></div></Panel></section>
  </div>
}

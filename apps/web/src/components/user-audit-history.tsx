import { useInfiniteQuery } from '@tanstack/react-query'
import { Link } from '@tanstack/react-router'
import type { ColumnDef } from '@tanstack/react-table'
import { useMemo } from 'react'
import { getUserAuditEvents } from '../lib/api'
import type { AuditEvent } from '../types'
import { DataTable } from './data-table'
import { Badge, Button, Panel } from './ui'

type UserAuditHistoryProps = {
  userId: string
  guildId?: string
}

export function UserAuditHistory({ userId, guildId }: UserAuditHistoryProps) {
  const audit = useInfiniteQuery({
    queryKey: ['user-audit-events', guildId ?? 'global', userId],
    queryFn: ({ pageParam }) => getUserAuditEvents(userId, guildId, pageParam ?? undefined),
    initialPageParam: null as number | null,
    getNextPageParam: (lastPage) => lastPage.length === 25 ? (lastPage.at(-1)?.id ?? null) : null,
  })
  const events = useMemo(() => audit.data?.pages.flat() ?? [], [audit.data])
  const columns = useMemo<Array<ColumnDef<AuditEvent>>>(() => [
    { accessorKey: 'occurred_at', header: 'Time', cell: ({ getValue }) => <time dateTime={getValue<string>()}>{new Date(getValue<string>()).toLocaleString()}</time> },
    { id: 'relationship', header: 'Relationship', cell: ({ row }) => <Badge tone={row.original.actor_id === userId ? 'violet' : 'neutral'}>{row.original.actor_id === userId ? 'Actor' : 'Target'}</Badge> },
    { accessorKey: 'action', header: 'Action', cell: ({ row }) => <span className="audit-action"><strong>{friendlyAction(row.original.action)}</strong><small>{row.original.source.replaceAll('_', ' ')}</small></span> },
    ...(!guildId ? [{ accessorKey: 'guild_id', header: 'Scope', cell: ({ row }: { row: { original: AuditEvent } }) => row.original.guild_id ?? 'Bot global' } as ColumnDef<AuditEvent>] : []),
    { accessorKey: 'outcome', header: 'Outcome', cell: ({ getValue }) => { const outcome = getValue<AuditEvent['outcome']>(); return <Badge tone={outcome === 'success' ? 'green' : outcome === 'denied' ? 'amber' : 'red'}>{outcome}</Badge> } },
    { id: 'details', header: 'Details', enableSorting: false, cell: ({ row }) => guildId ? <Link className="text-link" to="/servers/$serverId/audit/$eventId" params={{ serverId: guildId, eventId: String(row.original.id) }}>Inspect →</Link> : <Link className="text-link" to="/audit/$eventId" params={{ eventId: String(row.original.id) }}>Inspect →</Link> },
  ], [guildId, userId])

  return <Panel className="system-panel audit-table-panel">
    <div className="panel-heading user-audit-heading"><div><p className="eyebrow">Accountability</p><h3>Audit history</h3><p>Actions performed by this user and actions that targeted them.</p></div><Badge tone="neutral">{events.length} events</Badge></div>
    {audit.isLoading ? <div className="audit-loading">Loading audit history…</div> : audit.isError ? <div className="audit-loading audit-error">Could not load audit history: {audit.error.message}</div> : <><DataTable data={events} columns={columns} getRowId={(event) => String(event.id)} ariaLabel={`Audit history for Discord user ${userId}`} emptyMessage="No audit events recorded for this user" />{audit.hasNextPage && <div className="table-pagination"><Button disabled={audit.isFetchingNextPage} onClick={() => audit.fetchNextPage()}>{audit.isFetchingNextPage ? 'Loading…' : 'Load older events'}</Button></div>}</>}
  </Panel>
}

function friendlyAction(action: string) {
  return action.split('.').map((part) => part.charAt(0).toUpperCase() + part.slice(1).replaceAll('_', ' ')).join(' · ')
}

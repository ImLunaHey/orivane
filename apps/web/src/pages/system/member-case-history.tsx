import { useInfiniteQuery } from '@tanstack/react-query'
import { Link } from '@tanstack/react-router'
import type { ColumnDef } from '@tanstack/react-table'
import { useMemo } from 'react'
import { getModerationCases } from '../../lib/api'
import type { ModerationCase } from '../../types'
import { DataTable } from '../../components/data-table'
import { Badge, Button, Panel } from '../../components/ui'

export function MemberCaseHistory({ serverId, memberId }: { serverId: string; memberId: string }) {
  const cases = useInfiniteQuery({
    queryKey: ['moderation-cases', serverId, 'member', memberId],
    queryFn: ({ pageParam }) => getModerationCases(serverId, { memberId, beforeId: pageParam ?? undefined, limit: 25 }),
    initialPageParam: null as number | null,
    getNextPageParam: (lastPage) => lastPage.length === 25 ? (lastPage.at(-1)?.id ?? null) : null,
  })
  const rows = useMemo(() => cases.data?.pages.flat() ?? [], [cases.data])
  const columns = useMemo<Array<ColumnDef<ModerationCase>>>(() => [
    { accessorKey: 'id', header: 'Case', cell: ({ getValue }) => `#${getValue<number>()}` },
    { accessorKey: 'created_at', header: 'Created', cell: ({ getValue }) => <time dateTime={getValue<string>()}>{new Date(getValue<string>()).toLocaleString()}</time> },
    { accessorKey: 'action', header: 'Action', cell: ({ getValue }) => <strong className="case-action">{getValue<string>()}</strong> },
    { accessorKey: 'reason', header: 'Reason' },
    { accessorKey: 'status', header: 'Status', cell: ({ getValue }) => { const status = getValue<string>(); return <Badge tone={status === 'open' ? 'amber' : 'green'}>{status}</Badge> } },
    { id: 'details', header: 'Details', enableSorting: false, cell: ({ row }) => <Link className="text-link" to="/servers/$serverId/moderation/$caseId" params={{ serverId, caseId: String(row.original.id) }}>Inspect →</Link> },
  ], [serverId])

  return <Panel className="system-panel">
    <div className="panel-heading user-audit-heading"><div><p className="eyebrow">Moderation</p><h3>Case history</h3><p>Recorded moderation actions for this member in this server.</p></div><Badge tone={rows.length ? 'amber' : 'green'}>{rows.length} loaded</Badge></div>
    {cases.isLoading ? <div className="audit-loading">Loading moderation cases…</div> : cases.isError ? <div className="audit-loading audit-error">Could not load moderation cases: {cases.error.message}</div> : <><DataTable data={rows} columns={columns} getRowId={(item) => String(item.id)} ariaLabel={`Moderation cases for Discord member ${memberId}`} emptyMessage="No moderation cases recorded for this member" />{cases.hasNextPage && <div className="table-pagination"><Button disabled={cases.isFetchingNextPage} onClick={() => cases.fetchNextPage()}>{cases.isFetchingNextPage ? 'Loading…' : 'Load older cases'}</Button></div>}</>}
  </Panel>
}

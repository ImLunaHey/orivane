import type { ColumnDef } from '@tanstack/react-table'
import { DataTable } from '../../components/data-table'
import type { ConnectedServer } from '../../lib/server-data'

export function ServerDirectory({ servers, columns }: { servers: ConnectedServer[]; columns: Array<ColumnDef<ConnectedServer>> }) {
  return <DataTable data={servers} columns={columns} getRowId={(row) => row.id} ariaLabel="Highest activity Discord servers" />
}

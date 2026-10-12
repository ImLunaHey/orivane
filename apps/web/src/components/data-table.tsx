import { flexRender, getCoreRowModel, getSortedRowModel, useReactTable, type ColumnDef, type SortingState } from '@tanstack/react-table'
import { ArrowDown, ArrowUp, ArrowUpDown } from 'lucide-react'
import { useMemo, useState } from 'react'

type DataTableProps<TData> = {
  data: readonly TData[]
  columns: Array<ColumnDef<TData>>
  getRowId?: (row: TData) => string
  emptyMessage?: string
  ariaLabel: string
}

export function DataTable<TData>({ data, columns, getRowId, emptyMessage = 'No results', ariaLabel }: DataTableProps<TData>) {
  const [sorting, setSorting] = useState<SortingState>([])
  const tableData = useMemo(() => [...data], [data])
  const table = useReactTable({
    data: tableData,
    columns,
    state: { sorting },
    onSortingChange: setSorting,
    getCoreRowModel: getCoreRowModel(),
    getSortedRowModel: getSortedRowModel(),
    getRowId,
  })

  return <div className="table-scroll"><table className="data-table" aria-label={ariaLabel}><thead>{table.getHeaderGroups().map((group) => <tr key={group.id}>{group.headers.map((header) => {
    const sorted = header.column.getIsSorted()
    return <th key={header.id} aria-sort={sorted === 'asc' ? 'ascending' : sorted === 'desc' ? 'descending' : undefined}>{header.isPlaceholder ? null : header.column.getCanSort() ? <button onClick={header.column.getToggleSortingHandler()} title="Toggle column sorting">{flexRender(header.column.columnDef.header, header.getContext())}{sorted === 'asc' ? <ArrowUp size={13} /> : sorted === 'desc' ? <ArrowDown size={13} /> : <ArrowUpDown size={13} />}</button> : flexRender(header.column.columnDef.header, header.getContext())}</th>
  })}</tr>)}</thead><tbody>{table.getRowModel().rows.length ? table.getRowModel().rows.map((row) => <tr key={row.id}>{row.getVisibleCells().map((cell) => <td key={cell.id}>{flexRender(cell.column.columnDef.cell, cell.getContext())}</td>)}</tr>) : <tr><td colSpan={columns.length} className="table-empty">{emptyMessage}</td></tr>}</tbody></table></div>
}

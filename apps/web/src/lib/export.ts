import { recordDashboardActivity } from './api'

export function downloadCsv<T extends object>(filename: string, rows: readonly T[], guildId?: string) {
  const headers = [...new Set(rows.flatMap((row) => Object.keys(row)))]
  const csv = [headers, ...rows.map((row) => headers.map((header) => (row as Record<string, unknown>)[header]))]
    .map((row) => row.map(csvCell).join(','))
    .join('\n')
  const url = URL.createObjectURL(new Blob([csv], { type: 'text/csv;charset=utf-8' }))
  const link = document.createElement('a')
  link.href = url
  link.download = filename
  link.click()
  URL.revokeObjectURL(url)
  void recordDashboardActivity({ action: 'dashboard.data.exported', guild_id: guildId, path: window.location.pathname, export: filename })
}

function csvCell(value: unknown) {
  const serialized = typeof value === 'object' && value !== null ? JSON.stringify(value) : String(value ?? '')
  return `"${serialized.replaceAll('"', '""')}"`
}

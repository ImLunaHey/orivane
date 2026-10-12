import { useQuery } from '@tanstack/react-query'
import { AlertTriangle, RefreshCw } from 'lucide-react'
import { getResourceDiagnostics, isDemoMode } from '../lib/api'

export function ResourceHealthBanner({ serverId }: { serverId: string }) {
  const diagnostics = useQuery({
    queryKey: ['resource-diagnostics', serverId],
    queryFn: () => getResourceDiagnostics(serverId),
    enabled: !isDemoMode,
    staleTime: 60_000,
    retry: false,
  })
  if (isDemoMode || (!diagnostics.isError && diagnostics.data?.status !== 'attention')) return null

  const issues = diagnostics.data?.issues ?? []
  return <section className="resource-health-banner" role="alert" aria-live="polite">
    <AlertTriangle size={20} aria-hidden="true" />
    <div>
      <strong>{diagnostics.isError ? 'Discord resources could not be checked' : `${issues.length} Discord resource issue${issues.length === 1 ? '' : 's'} ${issues.length === 1 ? 'needs' : 'need'} attention`}</strong>
      {diagnostics.isError
        ? <p>{diagnostics.error instanceof Error ? diagnostics.error.message : 'Discord resource checks failed.'}</p>
        : <ul>{issues.slice(0, 5).map((issue) => <li key={`${issue.path}-${issue.id ?? issue.kind}`}><code>{friendlyPath(issue.path)}</code> — {issue.message}</li>)}</ul>}
      {issues.length > 5 && <p>{issues.length - 5} additional issues are available after the visible resources are corrected.</p>}
    </div>
    <button type="button" onClick={() => diagnostics.refetch()} disabled={diagnostics.isFetching}>
      <RefreshCw size={16} aria-hidden="true" />{diagnostics.isFetching ? 'Checking…' : 'Retry'}
    </button>
  </section>
}

function friendlyPath(path: string) {
  return path.replace(/^modules\./, '').replaceAll('_', ' ')
}

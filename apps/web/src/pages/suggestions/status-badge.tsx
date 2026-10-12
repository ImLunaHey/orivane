import { Badge } from '../../components/ui'
import type { SuggestionStatus } from '../../types'

export function labelStatus(status: SuggestionStatus) { return `${status.charAt(0).toUpperCase()}${status.slice(1)}` }
export function StatusBadge({ status }: { status: SuggestionStatus }) { const tone = status === 'implemented' || status === 'approved' ? 'green' : status === 'rejected' ? 'red' : status === 'pending' ? 'amber' : 'neutral'; return <Badge tone={tone}>{labelStatus(status)}</Badge> }

import { Badge } from '../../components/ui'
import type { ReminderStatus } from '../../types'

export function titleCase(value: string) { return `${value.charAt(0).toUpperCase()}${value.slice(1).replaceAll('_', ' ')}` }
export function ReminderStatusBadge({ status }: { status: ReminderStatus }) { const tone = status === 'scheduled' ? 'violet' : status === 'delivered' ? 'green' : status === 'failed' ? 'red' : 'neutral'; return <Badge tone={tone}>{titleCase(status)}</Badge> }

import { Badge } from '../../components/ui'
import type { GiveawayStatus } from '../../types'

export function GiveawayStatusBadge({ status }: { status: GiveawayStatus }) { const tone = status === 'active' ? 'green' : status === 'delivery_failed' ? 'red' : status === 'ended' ? 'violet' : 'neutral'; const label = status === 'delivery_failed' ? 'Delivery failed' : `${status.charAt(0).toUpperCase()}${status.slice(1).replaceAll('_', ' ')}`; return <Badge tone={tone}>{label}</Badge> }

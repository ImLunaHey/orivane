import type { ReactNode } from 'react'
import { Panel } from '../../components/ui'

export function SummaryMetric({ icon, label, value, detail }: { icon: ReactNode; label: string; value: string; detail: string }) {
  return <Panel><span>{icon}</span><div><small>{label}</small><strong>{value}</strong><em>{detail}</em></div></Panel>
}

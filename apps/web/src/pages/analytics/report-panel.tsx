import { Link } from '@tanstack/react-router'
import type { ReactNode } from 'react'
import { Panel } from '../../components/ui'

export function ReportPanel({ eyebrow, title, detailTo, serverId, children }: { eyebrow: string; title: string; detailTo: 'commands' | 'latency' | 'automod'; serverId: string; children: ReactNode }) {
  const to = detailTo === 'commands' ? '/servers/$serverId/analytics/commands' : detailTo === 'latency' ? '/servers/$serverId/analytics/latency' : '/servers/$serverId/analytics/automod'
  return <Panel className="analytics-chart-panel"><div className="panel-heading"><div><p className="eyebrow">{eyebrow}</p><h3>{title}</h3></div><Link to={to} params={{ serverId }} className="text-link">Details →</Link></div>{children}</Panel>
}

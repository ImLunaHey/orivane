import { Zap } from 'lucide-react'
import { Panel } from '../../components/ui'

export function BotMetric({ icon: Icon, label, value, note }: { icon: typeof Zap; label: string; value: string; note: string }) {
  return <Panel className="stat-card"><div className="stat-icon"><Icon size={18} /></div><p>{label}</p><strong>{value}</strong><span className="stat-change positive">{note}</span></Panel>
}

import { Users } from 'lucide-react'
import { Panel } from '../../components/ui'

export function UserMetric({ icon: Icon, label, value, note }: { icon: typeof Users; label: string; value: string; note: string }) {
  return <Panel><span className="system-icon"><Icon size={17} /></span><div><small>{label}</small><strong>{value}</strong><em>{note}</em></div></Panel>
}

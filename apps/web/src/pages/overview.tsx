import { useQuery } from '@tanstack/react-query'
import { Link, useParams } from '@tanstack/react-router'
import { Bot, CheckCircle2, Clock3, Server, ShieldCheck, Sparkles, Zap } from 'lucide-react'
import { CommandChart } from '../components/command-chart'
import { Badge, Panel } from '../components/ui'
import { getAuditEvents, getCommandVolume, getGuildConfig, getGuildStats, isDemoMode } from '../lib/api'
import { analyticsEvents } from '../lib/demo-data'
import { useCommandPalette } from '../lib/palette'

export function OverviewPage() {
  const { serverId } = useParams({ strict: false }) as { serverId: string }
  const openPalette = useCommandPalette()
  const { data: config } = useQuery({ queryKey: ['guild-config', serverId], queryFn: () => getGuildConfig(serverId) })
  const { data: summary } = useQuery({ queryKey: ['guild-stats', serverId], queryFn: () => getGuildStats(serverId) })
  const { data: commandVolume } = useQuery({ queryKey: ['command-volume', 'v2', serverId], queryFn: () => getCommandVolume(serverId) })
  const { data: auditEvents = [] } = useQuery({ queryKey: ['audit-events', serverId], queryFn: () => getAuditEvents(serverId) })
  const enabled = config ? Object.values(config.modules).filter((module) => module.enabled).length : 0
  const stats = [
    { label: 'Commands today', value: summary?.commands_today.toLocaleString() ?? '—', note: 'Audited slash commands', icon: Zap },
    { label: 'Active members', value: summary?.active_members_30d.toLocaleString() ?? '—', note: 'Unique command users · 30d', icon: Bot },
    { label: 'Automod actions', value: summary?.automod_actions_today.toLocaleString() ?? '—', note: 'Audited today', icon: ShieldCheck },
    { label: 'p95 response', value: summary?.p95_latency_ms == null ? '—' : `${Math.round(summary.p95_latency_ms)} ms`, note: summary?.p95_latency_ms == null ? 'Awaiting latency samples' : 'Last 24 hours', icon: Clock3 },
  ]

  return (
    <div className="page-stack">
      <section className="hero-row">
        <div><p className="eyebrow">{new Intl.DateTimeFormat(undefined, { dateStyle: 'full' }).format(new Date())}</p><h2>Server operations</h2><p>Configuration, audited activity, and health for this Discord server.</p></div>
        <button className="primary-action" onClick={openPalette}><Sparkles size={17} />Open command palette <kbd>⌘ K</kbd></button>
      </section>

      <section className="stats-grid">
        {stats.map(({ label, value, note, icon: Icon }) => (
          <Panel key={label} className="stat-card">
            <div className="stat-icon"><Icon size={18} /></div>
            <p>{label}</p><strong>{value}</strong>
            <span className="stat-change">{note}</span>
          </Panel>
        ))}
      </section>

      <section className="dashboard-grid">
        <Panel className="chart-panel">
          <div className="panel-heading"><div><p className="eyebrow">Activity</p><h3>Command volume</h3></div><div className="chart-total"><strong>{summary?.commands_today.toLocaleString() ?? '—'}</strong><span>today</span></div></div>
          <CommandChart data={commandVolume ?? []} />
        </Panel>
        <Panel className="health-panel">
          <div className="panel-heading"><div><p className="eyebrow">Runtime</p><h3>Server readiness</h3></div><Badge tone="neutral">Measured</Badge></div>
          <div className="health-list">
            <div><span className="service-icon violet"><Bot size={17} /></span><span><strong>Commands today</strong><small>Audited interactions</small></span><em>{summary?.commands_today.toLocaleString() ?? '—'}</em></div>
            <div><span className="service-icon blue"><Server size={17} /></span><span><strong>Open cases</strong><small>Moderation workflow</small></span><em>{summary?.open_cases.toLocaleString() ?? '—'}</em></div>
            <div><span className="service-icon green"><CheckCircle2 size={17} /></span><span><strong>Modules</strong><small>{enabled} enabled</small></span><em>Ready</em></div>
          </div>
        </Panel>
      </section>

      <Panel className="list-panel">
        <div className="panel-heading"><div><p className="eyebrow">Live stream</p><h3>Recent activity</h3></div><Link to="/servers/$serverId/analytics" params={{ serverId }} className="text-link">View analytics →</Link></div>
        <div className="activity-list">
          {(isDemoMode ? analyticsEvents : auditEvents.map((event) => ({ id: String(event.id), event: event.action, module: String(event.metadata.module ?? event.source), outcome: event.outcome === 'failure' ? 'failed' as const : event.outcome === 'denied' ? 'blocked' as const : 'success' as const, latency: Number(event.metadata.latency_ms ?? 0), occurredAt: new Date(event.occurred_at).toLocaleString() }))).slice(0, 4).map((event) => (
            <div key={event.id}><span className={`event-dot ${event.outcome}`} /><span><strong>{event.event}</strong><small>{event.module}</small></span><Badge tone={event.outcome === 'failed' ? 'red' : event.outcome === 'blocked' ? 'amber' : 'green'}>{event.outcome}</Badge><em>{event.latency ? `${event.latency} ms` : '—'}</em><time>{event.occurredAt}</time></div>
          ))}
        </div>
      </Panel>
    </div>
  )
}

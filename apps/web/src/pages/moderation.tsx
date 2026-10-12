import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query'
import { Link, useParams } from '@tanstack/react-router'
import { ArrowLeft, Clock3, Gavel, LoaderCircle, MessageSquareWarning, ShieldAlert, UserRoundCheck, X } from 'lucide-react'
import { useState } from 'react'
import { Badge, Button, Panel } from '../components/ui'
import { createModerationCase, getModerationCase, getModerationCases } from '../lib/api'
import type { NewModerationCase } from '../types'

export function ModerationPage() {
  const { serverId } = useParams({ strict: false }) as { serverId: string }
  const client = useQueryClient()
  const [creating, setCreating] = useState(false)
  const [draft, setDraft] = useState<NewModerationCase>({ member_id: '', action: 'warn', reason: '' })
  const queryKey = ['moderation-cases', serverId] as const
  const cases = useQuery({ queryKey, queryFn: () => getModerationCases(serverId) })
  const create = useMutation({
    mutationFn: () => createModerationCase(serverId, draft),
    onSuccess: async () => {
      await client.invalidateQueries({ queryKey })
      setCreating(false)
      setDraft({ member_id: '', action: 'warn', reason: '' })
    },
  })

  return <div className="page-stack">
    <section className="page-intro"><div><p className="eyebrow">Trust & safety</p><h2>Moderation</h2><p>A durable, searchable record of human and automated actions.</p></div><Button onClick={() => setCreating(true)}><Gavel size={16} />New case</Button></section>
    {creating && <Panel className="settings-panel"><div className="panel-heading"><div><p className="eyebrow">Case workflow</p><h3>Create moderation case</h3></div><button className="icon-button" aria-label="Close case form" onClick={() => setCreating(false)}><X size={18} /></button></div><form className="form-grid" onSubmit={(event) => { event.preventDefault(); create.mutate() }}><label><span>Member Discord ID</span><small>The stable Discord snowflake</small><input required inputMode="numeric" pattern="[1-9][0-9]*" value={draft.member_id} onChange={(event) => setDraft({ ...draft, member_id: event.target.value })} /></label><label><span>Action</span><small>Recorded moderation outcome</small><select value={draft.action} onChange={(event) => setDraft({ ...draft, action: event.target.value as NewModerationCase['action'] })}><option value="note">Note</option><option value="warn">Warn</option><option value="timeout">Timeout</option><option value="kick">Kick</option><option value="ban">Ban</option></select></label><label className="wide"><span>Reason</span><small>Visible to authorized moderators in the audit trail</small><textarea required maxLength={1000} value={draft.reason} onChange={(event) => setDraft({ ...draft, reason: event.target.value })} /></label><div className="save-row"><Button type="submit" disabled={create.isPending}>{create.isPending && <LoaderCircle className="spin" size={16} />}Create case</Button></div></form>{create.isError && <div className="error-banner" role="alert">{create.error.message}</div>}</Panel>}
    <section className="moderation-grid"><Panel><span className="service-icon amber"><MessageSquareWarning size={18} /></span><strong>{cases.data?.filter((item) => item.status === 'open').length ?? '—'}</strong><p>Open cases</p><small>Requires moderator review</small></Panel><Panel><span className="service-icon violet"><Clock3 size={18} /></span><strong>{cases.data?.filter((item) => item.action === 'timeout' && item.status === 'open').length ?? '—'}</strong><p>Active timeout cases</p><small>Tracked actions</small></Panel><Panel><span className="service-icon green"><UserRoundCheck size={18} /></span><strong>{cases.data?.length ?? '—'}</strong><p>Loaded cases</p><small>Newest 200 records</small></Panel><Panel><span className="service-icon blue"><ShieldAlert size={18} /></span><strong>0</strong><p>Critical alerts</p><small>All clear</small></Panel></section>
    <Panel className="list-panel"><div className="panel-heading"><div><p className="eyebrow">Case log</p><h3>Recent cases</h3></div><Badge tone="neutral">{cases.data?.length ?? 0} loaded</Badge></div>{cases.isLoading ? <div className="loading-state"><LoaderCircle className="spin" />Loading cases…</div> : <div className="case-list">{cases.data?.map((item) => <Link key={item.id} to="/servers/$serverId/moderation/$caseId" params={{ serverId, caseId: String(item.id) }}><span className="case-avatar">{item.member_id.slice(-2)}</span><span><strong>{item.member_id}</strong><small>CASE-{String(item.id).padStart(4, '0')} · {item.reason}</small></span><Badge tone={item.action === 'warn' ? 'amber' : item.action === 'timeout' ? 'violet' : 'neutral'}>{item.action}</Badge><em>{item.moderator_id}</em><time>{new Date(item.created_at).toLocaleString()}</time></Link>)}</div>}</Panel>
  </div>
}

export function ModerationCasePage() {
  const { serverId, caseId } = useParams({ strict: false }) as { serverId: string; caseId: string }
  const query = useQuery({ queryKey: ['moderation-case', serverId, caseId], queryFn: () => getModerationCase(serverId, caseId) })
  if (query.isLoading) return <div className="loading-state"><LoaderCircle className="spin" />Loading case…</div>
  if (!query.data) return <div className="empty-state"><h2>Case not found</h2><Link to="/servers/$serverId/moderation" params={{ serverId }}>Return to moderation</Link></div>
  const item = query.data
  return <div className="page-stack"><Link to="/servers/$serverId/moderation" params={{ serverId }} className="back-link"><ArrowLeft size={14} />Back to moderation</Link><section className="page-intro"><div><p className="eyebrow">Moderation case · CASE-{String(item.id).padStart(4, '0')}</p><h2>{item.action.charAt(0).toUpperCase() + item.action.slice(1)} for {item.member_id}</h2><p>{item.reason}</p></div><Badge tone={item.status === 'open' ? 'amber' : 'green'}>{item.status}</Badge></section><section className="member-detail-grid"><Panel className="profile-panel"><div className="panel-heading"><div><p className="eyebrow">Case record</p><h3>Details</h3></div></div><dl className="detail-list"><div><dt>Member ID</dt><dd>{item.member_id}</dd></div><div><dt>Moderator ID</dt><dd>{item.moderator_id}</dd></div><div><dt>Action</dt><dd>{item.action}</dd></div><div><dt>Status</dt><dd>{item.status}</dd></div><div><dt>Created</dt><dd>{new Date(item.created_at).toLocaleString()}</dd></div><div><dt>Updated</dt><dd>{new Date(item.updated_at).toLocaleString()}</dd></div></dl></Panel><Panel className="profile-panel"><div className="panel-heading"><div><p className="eyebrow">Reason</p><h3>Moderator context</h3></div></div><p className="case-reason">{item.reason}</p><p className="muted-copy">Creation and every later case mutation are written to the server audit trail.</p></Panel></section></div>
}

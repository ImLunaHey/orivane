import type { ColumnDef } from '@tanstack/react-table'
import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query'
import { Link, useNavigate, useParams } from '@tanstack/react-router'
import { ArrowLeft, Check, Download, FilePlus2, LoaderCircle, Send, Trash2 } from 'lucide-react'
import { useEffect, useMemo, useState } from 'react'
import { DataTable } from '../components/data-table'
import { DiscordChannelSelect } from '../components/discord-channel-select'
import { DiscordMemberLink } from '../components/discord-member-link'
import { DiscordMemberSelect } from '../components/discord-member-select'
import { DiscordRoleSelect } from '../components/discord-role-select'
import { MetricChart } from '../components/metric-chart'
import { Badge, Button, Panel, SectionHeader, Toggle } from '../components/ui'
import { addTicketNote, changeTicket, deleteTicketPanel, deployTicketPanel, getGuildChannels, getGuildConfig, getGuildMembers, getGuildRoles, getTicket, getTicketAnalytics, getTicketEvents, getTicketNotes, getTicketPanel, getTicketPanels, getTickets, saveGuildConfig, saveTicketPanel, ticketTranscriptUrl, updateTicketMetadata } from '../lib/api'
import type { GuildConfig, NewTicketPanel, Ticket, TicketPanel } from '../types'
import { Area } from './tickets/area'
import { Field } from './tickets/field'
import { Loading } from './tickets/loading'
import { Switch } from './tickets/switch'

export function TicketsPage(){
  const {serverId}=useParams({strict:false}) as {serverId:string};const client=useQueryClient();const configKey=['guild-config',serverId] as const
  const config=useQuery({queryKey:configKey,queryFn:()=>getGuildConfig(serverId)});const panels=useQuery({queryKey:['ticket-panels',serverId],queryFn:()=>getTicketPanels(serverId)});const tickets=useQuery({queryKey:['tickets',serverId],queryFn:()=>getTickets(serverId)});const analytics=useQuery({queryKey:['ticket-analytics',serverId],queryFn:()=>getTicketAnalytics(serverId)});const [draft,setDraft]=useState<GuildConfig|null>(null);useEffect(()=>setDraft(config.data??null),[config.data,serverId]);const save=useMutation({mutationFn:saveGuildConfig,onSuccess:s=>{client.setQueryData(configKey,s);setDraft(s)}})
  const deploy=useMutation({mutationFn:(id:string)=>deployTicketPanel(serverId,id),onSuccess:()=>client.invalidateQueries({queryKey:['ticket-panels',serverId]})})
  const panelColumns=useMemo<Array<ColumnDef<TicketPanel>>>(()=>[
    {accessorKey:'name',header:'Panel',cell:({row})=><div><Link to="/servers/$serverId/modules/tickets/panels/$panelId" params={{serverId,panelId:String(row.original.id)}}>{row.original.name}</Link><small>{row.original.title}</small></div>},{accessorKey:'enabled',header:'Status',cell:({row})=><Badge tone={row.original.enabled?'green':'neutral'}>{row.original.enabled?'Enabled':'Disabled'}</Badge>},{accessorKey:'panel_message_id',header:'Deployment',cell:({row})=>row.original.panel_message_id?'Published':'Not published'},{id:'actions',header:'Actions',enableSorting:false,cell:({row})=><div className="table-actions"><Link className="button table-action" to="/servers/$serverId/modules/tickets/panels/$panelId" params={{serverId,panelId:String(row.original.id)}}>Edit</Link><Button className="table-action" disabled={!row.original.enabled||deploy.isPending} onClick={()=>deploy.mutate(String(row.original.id))}><Send size={15}/>Deploy</Button></div>}
  ],[serverId,deploy])
  const ticketColumns=useMemo<Array<ColumnDef<Ticket>>>(()=>[
    {accessorKey:'id',header:'Ticket',cell:({row})=><Link to="/servers/$serverId/modules/tickets/$ticketId" params={{serverId,ticketId:String(row.original.id)}}>#{row.original.id} · {row.original.subject}</Link>},{accessorKey:'opener_id',header:'Member',cell:({row})=><DiscordMemberLink serverId={serverId} userId={row.original.opener_id} username={row.original.opener_username} displayName={row.original.opener_display_name}/>},{accessorKey:'status',header:'Status',cell:({row})=><Badge tone={row.original.status==='open'?'green':'neutral'}>{row.original.status}</Badge>},{accessorKey:'assigned_to',header:'Assigned',cell:({row})=>row.original.assigned_to?<DiscordMemberLink serverId={serverId} userId={row.original.assigned_to}/>: 'Unassigned'},{accessorKey:'updated_at',header:'Updated',cell:({row})=>formatDate(row.original.updated_at)}
  ],[serverId]);if(!draft||config.isLoading)return <Loading/>
  const items=tickets.data??[],summary=analytics.data,module=draft.modules.tickets,changed=JSON.stringify(config.data)!==JSON.stringify(draft),chart=['open','closed'].map(status=>({label:status[0].toUpperCase()+status.slice(1),value:items.filter(x=>x.status===status).length,detail:`${items.filter(x=>x.status===status).length} ${status} tickets`}))
  return <div className="page-stack tickets-page"><Link className="back-link" to="/servers/$serverId/modules" params={{serverId}}><ArrowLeft size={16}/>All modules</Link><section className="page-intro"><div><p className="eyebrow">Member support</p><h2>Tickets</h2><p>Publish intake panels, triage private support channels, assign staff, and retain a complete lifecycle record.</p></div><Link className="button primary-action" to="/servers/$serverId/modules/tickets/panels/$panelId" params={{serverId,panelId:'new'}}><FilePlus2 size={16}/>New panel</Link></section>
  <section className="forms-summary" aria-label="Ticket summary"><Panel><small>Total tickets</small><strong>{summary?.total??items.length}</strong></Panel><Panel><small>Open now</small><strong>{summary?.open??items.filter(x=>x.status==='open').length}</strong></Panel><Panel><small>Unassigned</small><strong>{summary?.unassigned??items.filter(x=>x.status==='open'&&!x.assigned_to).length}</strong></Panel><Panel><small>Awaiting auto-close</small><strong>{summary?.overdue??0}</strong></Panel><Panel><small>Median first response</small><strong>{duration(summary?.median_first_response_minutes)}</strong></Panel><Panel><small>Median resolution</small><strong>{duration(summary?.median_resolution_minutes)}</strong></Panel></section>
  <Panel className="table-panel"><SectionHeader eyebrow="Entry points" title="Ticket panels" detail="Each panel can use its own destination, category, support roles, intake prompt, and open-ticket limit."/><DataTable data={panels.data??[]} columns={panelColumns} getRowId={x=>String(x.id)} ariaLabel="Ticket panels" emptyMessage={panels.isLoading?'Loading ticket panels…':'No ticket panels configured'}/></Panel>
  <Panel className="analytics-chart-panel"><SectionHeader eyebrow="Workload" title="Tickets by status"/><MetricChart data={chart} xLabel="Ticket status" yLabel="Tickets" ariaLabel="Tickets grouped by status" type="bar" color="#8b5cf6"/></Panel>
  <Panel className="table-panel"><SectionHeader eyebrow="Inbox" title="Support tickets" detail="Open a ticket to review its assignment, lifecycle, intake note, and transcript."/><DataTable data={items} columns={ticketColumns} getRowId={x=>String(x.id)} ariaLabel="Support tickets" emptyMessage={tickets.isLoading?'Loading tickets…':'No tickets yet'}/></Panel>
  {(panels.isError||tickets.isError||analytics.isError||deploy.isError||save.isError)&&<div className="error-banner" role="alert">{panels.error?.message??tickets.error?.message??analytics.error?.message??deploy.error?.message??save.error?.message}</div>}
  <div className="bottom-save-bar"><div className="module-save-toggle"><Toggle label="Enable Tickets" checked={module.enabled} onChange={enabled=>setDraft({...draft,modules:{...draft.modules,tickets:{...module,enabled}}})}/><span><strong>Tickets module</strong><small>Controls panel buttons and the /ticket and /close slash commands.</small></span></div><div className="save-row">{save.isSuccess&&!changed&&<span className="saved-label"><Check size={15}/>Saved</span>}<Button disabled={!changed||save.isPending} onClick={()=>save.mutate(draft)}>Save changes</Button></div></div></div>
}

export function TicketPanelEditorPage(){
  const {serverId,panelId}=useParams({strict:false}) as {serverId:string;panelId:string};const navigate=useNavigate();const client=useQueryClient();const existing=useQuery({queryKey:['ticket-panel',serverId,panelId],queryFn:()=>getTicketPanel(serverId,panelId),enabled:panelId!=='new'});const channels=useQuery({queryKey:['guild-channels',serverId],queryFn:()=>getGuildChannels(serverId)});const roles=useQuery({queryKey:['guild-roles',serverId],queryFn:()=>getGuildRoles(serverId)});const [draft,setDraft]=useState<NewTicketPanel|null>(panelId==='new'?blankPanel():null);useEffect(()=>{if(existing.data)setDraft(fromPanel(existing.data))},[existing.data]);const save=useMutation({mutationFn:saveTicketPanel,onSuccess:item=>{void client.invalidateQueries({queryKey:['ticket-panels',serverId]});void navigate({to:'/servers/$serverId/modules/tickets/panels/$panelId',params:{serverId,panelId:String(item.id)},replace:true})}});const remove=useMutation({mutationFn:()=>deleteTicketPanel(serverId,panelId),onSuccess:()=>{void client.invalidateQueries({queryKey:['ticket-panels',serverId]});void client.invalidateQueries({queryKey:['tickets',serverId]});void navigate({to:'/servers/$serverId/modules/tickets',params:{serverId}})}});if(!draft)return <Loading/>;const update=<K extends keyof NewTicketPanel>(key:K,value:NewTicketPanel[K])=>setDraft({...draft,[key]:value})
  return <div className="page-stack ticket-panel-editor"><Link className="back-link" to="/servers/$serverId/modules/tickets" params={{serverId}}><ArrowLeft size={16}/>All tickets</Link><section className="page-intro"><div><p className="eyebrow">Ticket entry point</p><h2>{panelId==='new'?'New ticket panel':draft.name}</h2><p>Configure the public message and the private support channel created from it.</p></div><Badge tone={draft.enabled?'green':'neutral'}>{draft.enabled?'Enabled':'Disabled'}</Badge></section>
  <Panel className="form-detail-panel"><SectionHeader eyebrow="Panel" title="Public message" detail="Members see this message and button in the selected channel."/><div className="section-body form-grid"><Field title="Internal name" value={draft.name} onChange={v=>update('name',v)}/><DiscordChannelSelect title="Panel channel" detail="Where the open-ticket button is published" value={draft.channel_id} kind="text" channels={channels.data??[]} pending={channels.isLoading} error={channels.isError} onChange={v=>update('channel_id',v??'')}/><Field title="Message title" value={draft.title} onChange={v=>update('title',v)}/><Field title="Button label" value={draft.button_label} onChange={v=>update('button_label',v)}/><Area title="Description" value={draft.description} onChange={v=>update('description',v)}/></div></Panel>
  <Panel className="form-detail-panel"><SectionHeader eyebrow="Private channel" title="Routing and access"/><div className="section-body form-grid"><DiscordChannelSelect title="Ticket category" detail="Private ticket channels are created here" value={draft.category_id} kind="category" channels={channels.data??[]} pending={channels.isLoading} error={channels.isError} onChange={v=>update('category_id',v??'')}/><DiscordRoleSelect title="Support roles" detail="Roles that can view and claim tickets" values={draft.support_role_ids} roles={roles.data??[]} pending={roles.isLoading} error={roles.isError} onChange={v=>update('support_role_ids',v)}/><Field title="Channel name" value={draft.channel_name} onChange={v=>update('channel_name',v)}/><label><span>Open tickets per member</span><small>Limit for this panel</small><input type="number" min={1} max={5} value={draft.max_open_per_user} onChange={e=>update('max_open_per_user',Number(e.target.value))}/></label><Area title="Welcome message" value={draft.welcome_message} onChange={v=>update('welcome_message',v)}/></div></Panel>
  <Panel className="form-detail-panel"><SectionHeader eyebrow="Intake" title="Opening questions" detail="Discord opens a native modal before creating the channel."/><div className="section-body form-grid"><Switch title="Ask for ticket details" checked={draft.intake_enabled} onChange={v=>update('intake_enabled',v)}/><Field title="Subject prompt" value={draft.intake_label} onChange={v=>update('intake_label',v)}/><Switch title="Panel enabled" checked={draft.enabled} onChange={v=>update('enabled',v)}/></div></Panel>
  <Panel className="form-detail-panel"><SectionHeader eyebrow="Operations" title="Service policy" detail="Set the initial triage priority, inactivity cleanup, and transcript retention for tickets opened from this panel."/><div className="section-body form-grid"><label><span>Default priority</span><select value={draft.default_priority} onChange={e=>update('default_priority',e.target.value as NewTicketPanel['default_priority'])}><option value="low">Low</option><option value="normal">Normal</option><option value="high">High</option><option value="urgent">Urgent</option></select></label><label><span>Auto-close after</span><small>Hours without activity; use 0 to disable</small><input type="number" min={0} max={8760} value={draft.auto_close_hours} onChange={e=>update('auto_close_hours',Number(e.target.value))}/></label><label><span>Transcript retention</span><small>Days after a ticket closes</small><input type="number" min={1} max={3650} value={draft.transcript_retention_days} onChange={e=>update('transcript_retention_days',Number(e.target.value))}/></label></div></Panel>
  {(save.isError||remove.isError)&&<div className="error-banner" role="alert">{save.error?.message??remove.error?.message}</div>}<div className="bottom-save-bar"><span>Saving panel settings is audited. Deploy it from the ticket overview when ready.</span><div className="save-row">{panelId!=='new'&&<Button onClick={()=>remove.mutate()} disabled={remove.isPending}><Trash2 size={16}/>Delete</Button>}<Button className="primary-action" disabled={save.isPending||!valid(draft)} onClick={()=>save.mutate({guildId:serverId,panelId:panelId==='new'?undefined:panelId,panel:draft})}>{save.isPending&&<LoaderCircle className="spin" size={16}/>}Save panel</Button></div></div></div>
}

export function TicketDetailPage() {
  const { serverId, ticketId } = useParams({ strict: false }) as { serverId: string; ticketId: string }
  const client = useQueryClient()
  const key = ['ticket', serverId, ticketId] as const
  const ticket = useQuery({ queryKey: key, queryFn: () => getTicket(serverId, ticketId) })
  const events = useQuery({ queryKey: ['ticket-events', serverId, ticketId], queryFn: () => getTicketEvents(serverId, ticketId) })
  const notes = useQuery({ queryKey: ['ticket-notes', serverId, ticketId], queryFn: () => getTicketNotes(serverId, ticketId) })
  const members = useQuery({ queryKey: ['guild-members', serverId], queryFn: () => getGuildMembers(serverId) })
  const [reason, setReason] = useState('')
  const [note, setNote] = useState('')
  const [assignee, setAssignee] = useState<string | null>(null)
  const [priority, setPriority] = useState<Ticket['priority']>('normal')
  const [tags, setTags] = useState('')

  useEffect(() => {
    if (!ticket.data) return
    setAssignee(ticket.data.assigned_to)
    setPriority(ticket.data.priority)
    setTags(ticket.data.tags.join(', '))
  }, [ticket.data])

  const refresh = (item: Ticket) => {
    client.setQueryData(key, item)
    void client.invalidateQueries({ queryKey: ['tickets', serverId] })
    void client.invalidateQueries({ queryKey: ['ticket-analytics', serverId] })
    void client.invalidateQueries({ queryKey: ['ticket-events', serverId, ticketId] })
  }
  const action = useMutation({
    mutationFn: (kind: 'claim' | 'close' | 'reopen') => changeTicket({ guildId: serverId, ticketId, action: kind, reason }),
    onSuccess: refresh,
  })
  const transfer = useMutation({
    mutationFn: () => changeTicket({ guildId: serverId, ticketId, action: 'transfer', user_id: assignee ?? undefined }),
    onSuccess: refresh,
  })
  const metadata = useMutation({
    mutationFn: () => updateTicketMetadata({ guildId: serverId, ticketId, priority, tags: tags.split(',').map((tag) => tag.trim()).filter(Boolean) }),
    onSuccess: refresh,
  })
  const createNote = useMutation({
    mutationFn: () => addTicketNote({ guildId: serverId, ticketId, body: note }),
    onSuccess: () => {
      setNote('')
      void client.invalidateQueries({ queryKey: ['ticket-notes', serverId, ticketId] })
      void client.invalidateQueries({ queryKey: ['ticket-events', serverId, ticketId] })
    },
  })

  if (ticket.isLoading) return <Loading />
  if (!ticket.data) return <div className="empty-state"><h2>Ticket not found</h2></div>
  const item = ticket.data
  const error = action.error ?? transfer.error ?? metadata.error ?? createNote.error ?? notes.error ?? members.error

  return <div className="page-stack ticket-detail-page">
    <Link className="back-link" to="/servers/$serverId/modules/tickets" params={{ serverId }}><ArrowLeft size={16}/>All tickets</Link>
    <section className="page-intro"><div><p className="eyebrow">Ticket #{item.id}</p><h2>{item.subject}</h2><p>Opened {formatDate(item.opened_at)} in channel {item.channel_id}</p></div><Badge tone={item.status === 'open' ? 'green' : 'neutral'}>{item.status}</Badge></section>
    <section className="forms-summary">
      <Panel><small>Opened by</small><strong><DiscordMemberLink serverId={serverId} userId={item.opener_id} username={item.opener_username} displayName={item.opener_display_name}/></strong></Panel>
      <Panel><small>Assigned to</small><strong>{item.assigned_to ? <DiscordMemberLink serverId={serverId} userId={item.assigned_to}/> : 'Unassigned'}</strong></Panel>
      <Panel><small>Priority</small><strong>{item.priority}</strong></Panel>
      <Panel><small>Last activity</small><strong>{formatDate(item.last_activity_at)}</strong></Panel>
    </section>
    <Panel className="form-detail-panel"><SectionHeader eyebrow="Triage" title="Assignment and classification" detail="Route work to a known team member, set urgency, and add searchable tags."/><div className="section-body form-grid">
      <DiscordMemberSelect title="Assigned team member" detail="Members seen by Orivane in this server" value={assignee} members={members.data ?? []} pending={members.isLoading} error={members.isError} onChange={setAssignee}/>
      <div className="field-action"><span>Assignment</span><Button disabled={!assignee || assignee === item.assigned_to || transfer.isPending} onClick={() => transfer.mutate()}>Transfer ticket</Button></div>
      <label><span>Priority</span><select value={priority} onChange={(event) => setPriority(event.target.value as Ticket['priority'])}><option value="low">Low</option><option value="normal">Normal</option><option value="high">High</option><option value="urgent">Urgent</option></select></label>
      <label><span>Tags</span><small>Comma-separated, up to 10</small><input value={tags} onChange={(event) => setTags(event.target.value)} placeholder="billing, account-access"/></label>
      <div className="field-action wide"><span>Classification</span><Button disabled={metadata.isPending} onClick={() => metadata.mutate()}>Save priority and tags</Button></div>
    </div></Panel>
    <Panel className="form-detail-panel"><SectionHeader eyebrow="Intake" title="Opening note"/><div className="section-body"><p>{item.opening_note || 'No opening note was provided.'}</p></div></Panel>
    <Panel className="form-detail-panel"><SectionHeader eyebrow="Internal" title="Staff notes" detail="These notes stay in Orivane and are never posted to the ticket channel."/><div className="section-body ticket-notes"><label><span>New internal note</span><textarea value={note} maxLength={2000} onChange={(event) => setNote(event.target.value)} placeholder="Add context for the support team…"/></label><Button disabled={!note.trim() || createNote.isPending} onClick={() => createNote.mutate()}>Add note</Button>{(notes.data ?? []).map((entry) => <article key={entry.id}><DiscordMemberLink serverId={serverId} userId={entry.author_id}/><p>{entry.body}</p><time>{formatDate(entry.created_at)}</time></article>)}{!notes.isLoading && !notes.data?.length && <p className="muted-copy">No internal notes yet.</p>}</div></Panel>
    <Panel className="form-detail-panel"><SectionHeader eyebrow="History" title="Ticket lifecycle"/><div className="section-body answer-list">{(events.data ?? []).map((event) => <article key={event.id}><span>{event.action}</span><div><strong>{event.action.replaceAll('_', ' ')}</strong><p><DiscordMemberLink serverId={serverId} userId={event.actor_id}/> · {formatDate(event.created_at)}</p></div></article>)}</div></Panel>
    <Panel className="form-detail-panel"><SectionHeader eyebrow="Transcript" title="Conversation record" detail="Captured when the ticket closes and retained according to its panel policy."/><div className="section-body ticket-transcript-actions">{item.transcript ? <><pre>{item.transcript}</pre><a className="button" href={ticketTranscriptUrl(serverId, ticketId)} download><Download size={16}/>Download transcript</a></> : <p>Transcript capture will appear here when this ticket is closed.</p>}</div></Panel>
    {error && <div className="error-banner" role="alert">{error.message}</div>}
    <div className="bottom-save-bar"><label><span>Close reason</span><input value={reason} onChange={(event) => setReason(event.target.value)} placeholder="Why is this ticket being closed?"/></label><div className="save-row">{item.status === 'open' ? <><Button onClick={() => action.mutate('claim')}>Claim</Button><Button className="primary-action" onClick={() => action.mutate('close')}>Close ticket</Button></> : <Button className="primary-action" onClick={() => action.mutate('reopen')}>Reopen ticket</Button>}</div></div>
  </div>
}

function blankPanel():NewTicketPanel{return{name:'General support',title:'Need help?',description:'Open a private ticket and our team will get back to you.',channel_id:'',category_id:'',support_role_ids:[],button_label:'Open ticket',welcome_message:'Welcome {user}. {support_roles} will be with you shortly.',channel_name:'ticket-{user_id}',intake_enabled:true,intake_label:'What can we help with?',max_open_per_user:1,auto_close_hours:72,transcript_retention_days:90,default_priority:'normal',enabled:true}}
function fromPanel(p:TicketPanel):NewTicketPanel{return{name:p.name,title:p.title,description:p.description,channel_id:p.panel_channel_id??'',category_id:p.category_id,support_role_ids:p.support_role_ids,button_label:p.button_label,welcome_message:p.welcome_message,channel_name:p.channel_name,intake_enabled:p.intake_enabled,intake_label:p.intake_label,max_open_per_user:p.max_open_per_user,auto_close_hours:p.auto_close_hours,transcript_retention_days:p.transcript_retention_days,default_priority:p.default_priority,enabled:p.enabled}}
function valid(p:NewTicketPanel){return Boolean(p.name.trim()&&p.title.trim()&&p.channel_id&&p.category_id&&p.support_role_ids.length&&p.button_label.trim()&&p.welcome_message.trim()&&p.channel_name.trim()&&p.intake_label.trim()&&p.max_open_per_user>=1&&p.max_open_per_user<=5&&p.auto_close_hours>=0&&p.auto_close_hours<=8760&&p.transcript_retention_days>=1&&p.transcript_retention_days<=3650)}
function formatDate(value:string){return new Intl.DateTimeFormat('en-GB',{dateStyle:'medium',timeStyle:'short'}).format(new Date(value))};function duration(value:number|null|undefined){if(value==null)return 'No data';if(value<60)return `${Math.round(value)} min`;if(value<1440)return `${(value/60).toFixed(1)} hr`;return `${(value/1440).toFixed(1)} days`}

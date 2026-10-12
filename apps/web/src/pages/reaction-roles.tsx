import type { ColumnDef } from '@tanstack/react-table'
import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query'
import { Link, useNavigate, useParams } from '@tanstack/react-router'
import { ArrowLeft, Check, LoaderCircle, Plus, Send, Trash2 } from 'lucide-react'
import { useEffect, useMemo, useState } from 'react'
import { DataTable } from '../components/data-table'
import { DiscordChannelSelect } from '../components/discord-channel-select'
import { DiscordRoleSelect } from '../components/discord-role-select'
import { Badge, Button, Panel, SectionHeader, Toggle } from '../components/ui'
import { deployReactionRolePanel, getGuildChannels, getGuildConfig, getGuildRoles, saveGuildConfig } from '../lib/api'
import type { GuildConfig, ReactionRoleOption, ReactionRolePanel } from '../types'

export function ReactionRolesPage() {
  const { serverId } = useParams({ strict: false }) as { serverId: string }
  const client = useQueryClient()
  const key = ['guild-config', serverId] as const
  const config = useQuery({ queryKey: key, queryFn: () => getGuildConfig(serverId) })
  const [draft, setDraft] = useState<GuildConfig | null>(null)
  useEffect(() => setDraft(config.data ?? null), [config.data, serverId])
  const save = useMutation({ mutationFn: saveGuildConfig, onSuccess: (next) => { client.setQueryData(key, next); setDraft(next) } })
  const deploy = useMutation({ mutationFn: (panelId: string) => deployReactionRolePanel(serverId, panelId), onSuccess: (next) => { client.setQueryData(key, next); setDraft(next) } })
  const columns = useMemo<Array<ColumnDef<ReactionRolePanel>>>(() => [
    { accessorKey: 'name', header: 'Panel', cell: ({ row }) => <div><Link to="/servers/$serverId/modules/reaction_roles/$panelId" params={{ serverId, panelId: row.original.id }}>{row.original.name}</Link><small>{row.original.title}</small></div> },
    { accessorKey: 'component_type', header: 'Control', cell: ({ row }) => row.original.component_type === 'select' ? 'Dropdown' : 'Buttons' },
    { accessorKey: 'options', header: 'Roles', cell: ({ row }) => row.original.options.length },
    { accessorKey: 'message_id', header: 'Deployment', cell: ({ row }) => row.original.message_id ? 'Published' : 'Not published' },
    { accessorKey: 'enabled', header: 'Status', cell: ({ row }) => <Badge tone={row.original.enabled ? 'green' : 'neutral'}>{row.original.enabled ? 'Enabled' : 'Disabled'}</Badge> },
    { id: 'actions', header: 'Actions', enableSorting: false, cell: ({ row }) => <div className="table-actions"><Link className="button table-action" to="/servers/$serverId/modules/reaction_roles/$panelId" params={{ serverId, panelId: row.original.id }}>Edit</Link><Button className="table-action" disabled={!row.original.enabled || deploy.isPending} onClick={() => deploy.mutate(row.original.id)}><Send size={15}/>{row.original.message_id ? 'Sync' : 'Deploy'}</Button></div> },
  ], [deploy, serverId])
  if (!draft) return <Loading/>
  const module = draft.modules.reaction_roles
  const changed = JSON.stringify(config.data) !== JSON.stringify(draft)
  return <div className="page-stack">
    <Link className="back-link" to="/servers/$serverId/modules" params={{ serverId }}><ArrowLeft size={16}/>All modules</Link>
    <section className="page-intro"><div><p className="eyebrow">Self-service roles</p><h2>Reaction roles</h2><p>Publish accessible buttons or dropdowns, enforce exclusive groups, and keep classic reaction mappings where needed.</p></div><Link className="button primary-action" to="/servers/$serverId/modules/reaction_roles/$panelId" params={{ serverId, panelId: 'new' }}><Plus size={16}/>New panel</Link></section>
    <section className="forms-summary"><Panel><small>Managed panels</small><strong>{module.panels.length}</strong></Panel><Panel><small>Published</small><strong>{module.panels.filter((panel) => panel.message_id).length}</strong></Panel><Panel><small>Managed roles</small><strong>{module.panels.reduce((total, panel) => total + panel.options.length, 0)}</strong></Panel><Panel><small>Classic mappings</small><strong>{module.bindings.length}</strong></Panel></section>
    <Panel className="table-panel"><SectionHeader eyebrow="Managed messages" title="Role panels" detail="Deploy once, then synchronize edits to the same Discord message."/><DataTable data={module.panels} columns={columns} getRowId={(panel) => panel.id} ariaLabel="Reaction-role panels" emptyMessage="No managed role panels configured"/></Panel>
    <Panel className="form-detail-panel"><SectionHeader eyebrow="Compatibility" title="Classic reaction mappings" detail="Existing emoji-to-role mappings continue to work. New setups should use managed panels for clearer labels and mobile accessibility."/><div className="section-body answer-list">{module.bindings.map((binding) => <article key={`${binding.message_id}:${binding.emoji}`}><span>{binding.emoji}</span><div><strong>Role {binding.role_id}</strong><p>Message {binding.message_id}</p></div></article>)}{!module.bindings.length && <p className="muted-copy">No classic mappings.</p>}</div></Panel>
    {(config.isError || save.isError || deploy.isError) && <div className="error-banner" role="alert">{config.error?.message ?? save.error?.message ?? deploy.error?.message}</div>}
    <div className="bottom-save-bar"><div className="module-save-toggle"><Toggle label="Enable Reaction roles" checked={module.enabled} onChange={(enabled) => setDraft({ ...draft, modules: { ...draft.modules, reaction_roles: { ...module, enabled } } })}/><span><strong>Reaction roles module</strong><small>Controls managed panels and classic reaction mappings.</small></span></div><div className="save-row">{save.isSuccess && !changed && <span className="saved-label"><Check size={15}/>Saved</span>}<Button disabled={!changed || save.isPending} onClick={() => save.mutate(draft)}>Save changes</Button></div></div>
  </div>
}

export function ReactionRolePanelPage() {
  const { serverId, panelId } = useParams({ strict: false }) as { serverId: string; panelId: string }
  const navigate = useNavigate()
  const client = useQueryClient()
  const key = ['guild-config', serverId] as const
  const config = useQuery({ queryKey: key, queryFn: () => getGuildConfig(serverId) })
  const channels = useQuery({ queryKey: ['guild-channels', serverId], queryFn: () => getGuildChannels(serverId) })
  const roles = useQuery({ queryKey: ['guild-roles', serverId], queryFn: () => getGuildRoles(serverId) })
  const [draft, setDraft] = useState<ReactionRolePanel | null>(panelId === 'new' ? blankPanel() : null)
  useEffect(() => {
    if (panelId !== 'new' && config.data) setDraft(config.data.modules.reaction_roles.panels.find((panel) => panel.id === panelId) ?? null)
  }, [config.data, panelId])
  const save = useMutation({ mutationFn: saveGuildConfig, onSuccess: (next) => { client.setQueryData(key, next); void navigate({ to: '/servers/$serverId/modules/reaction_roles/$panelId', params: { serverId, panelId: draft!.id }, replace: true }) } })
  const deploy = useMutation({ mutationFn: () => deployReactionRolePanel(serverId, draft!.id), onSuccess: (next) => { client.setQueryData(key, next); setDraft(next.modules.reaction_roles.panels.find((panel) => panel.id === draft!.id) ?? draft) } })
  if (!config.data || !draft) return <Loading/>
  const update = <K extends keyof ReactionRolePanel>(field: K, value: ReactionRolePanel[K]) => setDraft({ ...draft, [field]: value })
  const updateOption = (index: number, option: ReactionRoleOption) => update('options', draft.options.map((item, itemIndex) => itemIndex === index ? option : item))
  const persist = () => {
    const current = config.data!
    const exists = current.modules.reaction_roles.panels.some((panel) => panel.id === panelId)
    const panels = exists ? current.modules.reaction_roles.panels.map((panel) => panel.id === panelId ? draft : panel) : [...current.modules.reaction_roles.panels, draft]
    save.mutate({ ...current, modules: { ...current.modules, reaction_roles: { ...current.modules.reaction_roles, panels } } })
  }
  const remove = () => {
    const current = config.data!
    save.mutate({ ...current, modules: { ...current.modules, reaction_roles: { ...current.modules.reaction_roles, panels: current.modules.reaction_roles.panels.filter((panel) => panel.id !== panelId) } } }, { onSuccess: () => void navigate({ to: '/servers/$serverId/modules/reaction_roles', params: { serverId } }) })
  }
  return <div className="page-stack">
    <Link className="back-link" to="/servers/$serverId/modules/reaction_roles" params={{ serverId }}><ArrowLeft size={16}/>All role panels</Link>
    <section className="page-intro"><div><p className="eyebrow">Managed role message</p><h2>{panelId === 'new' ? 'New reaction-role panel' : draft.name}</h2><p>Configure the Discord message, interaction type, assignment behavior, and available roles.</p></div><Badge tone={draft.message_id ? 'green' : 'neutral'}>{draft.message_id ? 'Published' : 'Draft'}</Badge></section>
    <Panel className="form-detail-panel"><SectionHeader eyebrow="Message" title="Panel identity and destination"/><div className="section-body form-grid"><Field title="Internal name" value={draft.name} onChange={(name) => update('name', name)}/><Field title="Stable panel ID" value={draft.id} disabled={panelId !== 'new'} onChange={(id) => update('id', cleanId(id))}/><DiscordChannelSelect title="Destination channel" detail="Where the role panel is published" value={draft.channel_id || null} kind="text" channels={channels.data ?? []} pending={channels.isLoading} error={channels.isError} onChange={(channel) => update('channel_id', channel ?? '')}/><Field title="Message title" value={draft.title} onChange={(title) => update('title', title)}/><label className="wide"><span>Description</span><textarea value={draft.description} maxLength={1800} onChange={(event) => update('description', event.target.value)}/></label></div></Panel>
    <Panel className="form-detail-panel"><SectionHeader eyebrow="Behavior" title="How roles are changed"/><div className="section-body form-grid"><label><span>Interaction control</span><select value={draft.component_type} onChange={(event) => update('component_type', event.target.value as ReactionRolePanel['component_type'])}><option value="buttons">Buttons</option><option value="select">Dropdown</option></select></label><label><span>Assignment mode</span><select value={draft.mode} onChange={(event) => update('mode', event.target.value as ReactionRolePanel['mode'])}><option value="toggle">Toggle / synchronize</option><option value="add_only">Add only</option><option value="remove_only">Remove only</option></select></label><Switch title="Exclusive group" detail="A member can hold only one role from this panel" checked={draft.exclusive} onChange={(exclusive) => update('exclusive', exclusive)}/><Switch title="Panel enabled" detail="Disabled panels reject interactions and cannot deploy" checked={draft.enabled} onChange={(enabled) => update('enabled', enabled)}/><DiscordRoleSelect title="Required roles" detail="Members need at least one of these roles; leave empty for everyone" values={draft.required_role_ids} roles={roles.data ?? []} pending={roles.isLoading} error={roles.isError} onChange={(required_role_ids) => update('required_role_ids', required_role_ids)}/><DiscordRoleSelect title="Blocked roles" detail="Members with any of these roles cannot use this panel" values={draft.blocked_role_ids} roles={roles.data ?? []} pending={roles.isLoading} error={roles.isError} onChange={(blocked_role_ids) => update('blocked_role_ids', blocked_role_ids)}/></div></Panel>
    <Panel className="form-detail-panel"><SectionHeader eyebrow="Choices" title="Role options" detail="Buttons and Discord dropdowns support up to 25 options."/><div className="section-body reaction-role-options">{draft.options.map((option, index) => <article className="custom-command-card" key={`${option.id}-${index}`}><div className="custom-command-card-head"><strong>Option {index + 1}</strong><Button onClick={() => update('options', draft.options.filter((_, itemIndex) => itemIndex !== index))}><Trash2 size={16}/>Remove</Button></div><div className="form-grid"><Field title="Option ID" value={option.id} onChange={(id) => updateOption(index, { ...option, id: cleanId(id) })}/><Field title="Label" value={option.label} onChange={(label) => updateOption(index, { ...option, label })}/><Field title="Emoji" value={option.emoji} onChange={(emoji) => updateOption(index, { ...option, emoji })}/><Field title="Description" value={option.description} onChange={(description) => updateOption(index, { ...option, description })}/><DiscordRoleSelect title="Discord role" detail="Role managed by this choice" values={option.role_id ? [option.role_id] : []} roles={roles.data ?? []} pending={roles.isLoading} error={roles.isError} max={1} onChange={(values) => updateOption(index, { ...option, role_id: values[0] ?? '' })}/></div></article>)}<Button disabled={draft.options.length >= 25} onClick={() => update('options', [...draft.options, blankOption(draft.options.length)])}><Plus size={16}/>Add role option</Button></div></Panel>
    <Panel className="form-detail-panel"><SectionHeader eyebrow="Preview" title="Discord message"/><div className="section-body discord-message-preview"><strong>{draft.title || 'Choose your roles'}</strong><p>{draft.description || 'Select a role below.'}</p><div className="reaction-role-preview-controls">{draft.component_type === 'buttons' ? draft.options.map((option) => <button type="button" key={option.id}>{option.emoji} {option.label}</button>) : <button type="button">Choose your roles ▾</button>}</div></div></Panel>
    {(config.isError || save.isError || deploy.isError || channels.isError || roles.isError) && <div className="error-banner" role="alert">{config.error?.message ?? save.error?.message ?? deploy.error?.message ?? channels.error?.message ?? roles.error?.message}</div>}
    <div className="bottom-save-bar"><span>Save configuration before deploying or synchronizing the Discord message.</span><div className="save-row">{panelId !== 'new' && <Button disabled={save.isPending} onClick={remove}><Trash2 size={16}/>Delete</Button>}{panelId !== 'new' && <Button disabled={!valid(draft) || deploy.isPending} onClick={() => deploy.mutate()}><Send size={16}/>{draft.message_id ? 'Sync message' : 'Deploy message'}</Button>}<Button className="primary-action" disabled={!valid(draft) || save.isPending} onClick={persist}>{save.isPending && <LoaderCircle className="spin" size={16}/>}Save panel</Button></div></div>
  </div>
}

function blankPanel():ReactionRolePanel{return{id:`roles-${Date.now()}`,enabled:true,name:'Member roles',title:'Choose your roles',description:'Use the controls below to update your roles.',channel_id:'',message_id:null,component_type:'buttons',mode:'toggle',exclusive:false,required_role_ids:[],blocked_role_ids:[],options:[blankOption(0)]}}
function blankOption(index:number):ReactionRoleOption{return{id:`role-${index+1}`,label:`Role ${index+1}`,description:'',emoji:'✨',role_id:''}}
function cleanId(value:string){return value.toLowerCase().replace(/[^a-z0-9_-]/g,'').slice(0,64)}
function valid(panel:ReactionRolePanel){return Boolean(panel.id&&panel.name.trim()&&panel.title.trim()&&panel.channel_id&&panel.options.length&&panel.options.every((option)=>option.id&&option.label.trim()&&option.role_id))}
function Field({title,value,disabled=false,onChange}:{title:string;value:string;disabled?:boolean;onChange:(value:string)=>void}){return <label><span>{title}</span><input value={value} disabled={disabled} onChange={(event)=>onChange(event.target.value)}/></label>}
function Switch({title,detail,checked,onChange}:{title:string;detail:string;checked:boolean;onChange:(value:boolean)=>void}){return <div className="module-toggle-field"><span><strong>{title}</strong><small>{detail}</small></span><Toggle label={title} checked={checked} onChange={onChange}/></div>}
function Loading(){return <div className="loading-state"><LoaderCircle className="spin"/><span>Loading reaction roles…</span></div>}

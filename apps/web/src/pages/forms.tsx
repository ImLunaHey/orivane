import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query'
import { Link, useNavigate, useParams } from '@tanstack/react-router'
import type { ColumnDef } from '@tanstack/react-table'
import { ArrowLeft, Check, ClipboardList, Download, FilePlus2, LoaderCircle, Plus, Trash2 } from 'lucide-react'
import { useEffect, useMemo, useState } from 'react'
import { DataTable } from '../components/data-table'
import { DiscordChannelSelect } from '../components/discord-channel-select'
import { DiscordMemberLink } from '../components/discord-member-link'
import { DiscordRoleSelect } from '../components/discord-role-select'
import { Badge, Button, Panel, SectionHeader, Toggle } from '../components/ui'
import { getFormSubmission, getFormSubmissions, getGuildChannels, getGuildConfig, getGuildRoles, recordDashboardActivity, reviewFormSubmission, saveGuildConfig } from '../lib/api'
import type { FormQuestion, FormSubmission, GuildConfig, ServerForm } from '../types'

export function FormsPage() {
  const { serverId } = useParams({ strict: false }) as { serverId: string }
  const client = useQueryClient()
  const queryKey = ['guild-config', serverId] as const
  const config = useQuery({ queryKey, queryFn: () => getGuildConfig(serverId) })
  const channels = useQuery({ queryKey: ['guild-channels', serverId], queryFn: () => getGuildChannels(serverId) })
  const [draft, setDraft] = useState<GuildConfig | null>(null)
  const save = useMutation({ mutationFn: saveGuildConfig, onSuccess: (saved) => { client.setQueryData(queryKey, saved); setDraft(saved) } })
  useEffect(() => setDraft(config.data ?? null), [config.data, serverId])

  const columns = useMemo<Array<ColumnDef<ServerForm>>>(() => [
    { accessorKey: 'name', header: 'Form', cell: ({ row }) => <div className="form-name-cell"><ClipboardList size={17} /><span><Link to="/servers/$serverId/modules/forms/$formId" params={{ serverId, formId: row.original.id }}>{row.original.name}</Link><small>/form name:{row.original.id}</small></span></div> },
    { accessorKey: 'enabled', header: 'Status', cell: ({ row }) => <Badge tone={row.original.enabled ? 'green' : 'neutral'}>{row.original.enabled ? 'Enabled' : 'Disabled'}</Badge> },
    { id: 'destination', header: 'Destination', cell: ({ row }) => channels.data?.find((channel) => channel.id === row.original.destination_channel_id)?.name ?? 'Unavailable' },
    { id: 'questions', header: 'Questions', accessorFn: (row) => row.questions.length, cell: ({ row }) => `${row.original.questions.length} / 5` },
    { id: 'access', header: 'Access', cell: ({ row }) => row.original.allowed_role_ids.length ? `${row.original.allowed_role_ids.length} role${row.original.allowed_role_ids.length === 1 ? '' : 's'}` : 'Everyone' },
    { id: 'actions', header: '', enableSorting: false, cell: ({ row }) => <div className="table-actions"><Link className="button table-action" to="/servers/$serverId/modules/forms/$formId" params={{ serverId, formId: row.original.id }}>Edit</Link><Link className="button table-action" to="/servers/$serverId/modules/forms/$formId/submissions" params={{ serverId, formId: row.original.id }}>Submissions</Link></div> },
  ], [channels.data, serverId])

  if (config.isLoading || !draft) return <LoadingState />
  const forms = draft.modules.forms
  const hasChanges = JSON.stringify(config.data) !== JSON.stringify(draft)
  const totalQuestions = forms.forms.reduce((total, form) => total + form.questions.length, 0)
  const restricted = forms.forms.filter((form) => form.allowed_role_ids.length > 0).length

  return <div className="page-stack forms-page">
    <Link className="back-link" to="/servers/$serverId/modules" params={{ serverId }}><ArrowLeft size={16} />All modules</Link>
    <section className="page-intro"><div><p className="eyebrow">Community workflow</p><h2>Forms</h2><p>Create Discord-native forms, control who can submit them, and review responses from one workspace.</p></div><Badge tone={forms.enabled ? 'green' : 'neutral'}>{forms.enabled ? 'Module active' : 'Module disabled'}</Badge></section>
    <section className="forms-summary" aria-label="Forms summary"><Panel><small>Configured forms</small><strong>{forms.forms.length}</strong></Panel><Panel><small>Total questions</small><strong>{totalQuestions}</strong></Panel><Panel><small>Role restricted</small><strong>{restricted}</strong></Panel></section>
    <Panel className="form-detail-panel"><SectionHeader eyebrow="Data lifecycle" title="Submission retention" detail="Expired responses are permanently purged by the durable scheduler." /><div className="section-body form-grid"><NumberField title="Retention period" detail="Days to retain form submissions (1–3650)" min={1} max={3650} value={forms.retention_days} onChange={(retention_days) => setDraft({ ...draft, modules: { ...draft.modules, forms: { ...forms, retention_days } } })} /></div></Panel>
    <Panel className="table-panel forms-directory"><div className="table-toolbar"><div><p className="eyebrow">Directory</p><h3>Current forms</h3></div><Link className="button" to="/servers/$serverId/modules/forms/$formId" params={{ serverId, formId: 'new' }}><FilePlus2 size={16} />New form</Link></div><DataTable data={forms.forms} columns={columns} getRowId={(row) => row.id} ariaLabel="Configured Discord forms" emptyMessage="No forms configured yet" /></Panel>
    {save.isError && <div className="error-banner" role="alert">{save.error.message}</div>}
    <div className="bottom-save-bar"><div className="module-save-toggle"><Toggle label="Enable Forms" checked={forms.enabled} onChange={(enabled) => setDraft({ ...draft, modules: { ...draft.modules, forms: { ...forms, enabled } } })} /><span><strong>Forms module</strong><small>{forms.enabled ? 'Members can open enabled forms.' : 'All form commands are currently disabled.'}</small></span></div><div className="save-row">{save.isSuccess && !hasChanges && <span className="saved-label"><Check size={15} />Saved</span>}<Button disabled={!hasChanges || save.isPending} onClick={() => save.mutate(draft)}>{save.isPending && <LoaderCircle className="spin" size={16} />}Save changes</Button></div></div>
  </div>
}

export function FormDetailPage() {
  const { serverId, formId } = useParams({ strict: false }) as { serverId: string; formId: string }
  const navigate = useNavigate()
  const client = useQueryClient()
  const queryKey = ['guild-config', serverId] as const
  const config = useQuery({ queryKey, queryFn: () => getGuildConfig(serverId) })
  const channels = useQuery({ queryKey: ['guild-channels', serverId], queryFn: () => getGuildChannels(serverId) })
  const roles = useQuery({ queryKey: ['guild-roles', serverId], queryFn: () => getGuildRoles(serverId) })
  const [draft, setDraft] = useState<ServerForm | null>(null)
  const existing = config.data?.modules.forms.forms.find((form) => form.id === formId)
  useEffect(() => {
    if (!config.data) return
    setDraft(existing ? structuredClone(existing) : formId === 'new' ? newForm(config.data.modules.forms.forms.length + 1) : null)
  }, [config.data, existing, formId])

  const save = useMutation({ mutationFn: saveGuildConfig, onSuccess: (saved) => { client.setQueryData(queryKey, saved); void navigate({ to: '/servers/$serverId/modules/forms/$formId', params: { serverId, formId: draft!.id }, replace: true }) } })
  const remove = useMutation({ mutationFn: saveGuildConfig, onSuccess: (saved) => { client.setQueryData(queryKey, saved); void navigate({ to: '/servers/$serverId/modules/forms', params: { serverId } }) } })
  if (config.isLoading || !config.data) return <LoadingState />
  if (!draft) return <div className="empty-state"><h2>Form not found</h2><Link to="/servers/$serverId/modules/forms" params={{ serverId }}>Return to forms</Link></div>
  const updateQuestion = (index: number, question: FormQuestion) => setDraft({ ...draft, questions: draft.questions.map((item, itemIndex) => itemIndex === index ? question : item) })
  const addQuestion = () => { if (draft.questions.length < 5) setDraft({ ...draft, questions: [...draft.questions, { id: `question-${draft.questions.length + 1}`, label: 'New question', style: 'short', required: true, min_length: 0, max_length: 300, placeholder: '' }] }) }
  const deleteQuestion = (index: number) => setDraft({ ...draft, questions: draft.questions.filter((_, itemIndex) => itemIndex !== index) })
  const saveConfig = () => {
    const forms = existing ? config.data.modules.forms.forms.map((form) => form.id === formId ? draft : form) : [...config.data.modules.forms.forms, draft]
    save.mutate({ ...config.data, modules: { ...config.data.modules, forms: { ...config.data.modules.forms, forms } } })
  }
  const deleteForm = () => {
    const forms = config.data.modules.forms.forms.filter((form) => form.id !== formId)
    remove.mutate({ ...config.data, modules: { ...config.data.modules, forms: { ...config.data.modules.forms, enabled: forms.length > 0 && config.data.modules.forms.enabled, forms } } })
  }
  const hasChanges = !existing || JSON.stringify(existing) !== JSON.stringify(draft)

  return <div className="page-stack form-detail-page">
    <Link className="back-link" to="/servers/$serverId/modules/forms" params={{ serverId }}><ArrowLeft size={16} />All forms</Link>
    <section className="page-intro"><div><p className="eyebrow">{existing ? `/form name:${formId}` : 'New form'}</p><h2>{draft.name}</h2><p>{draft.description || 'Configure how this form appears and where submissions are delivered.'}</p></div><Badge tone={draft.enabled ? 'green' : 'neutral'}>{draft.enabled ? 'Enabled' : 'Disabled'}</Badge></section>
    {(channels.isError || roles.isError) && <div className="error-banner" role="alert">{[channels.isError && `Channels: ${channels.error.message}`, roles.isError && `Roles: ${roles.error.message}`].filter(Boolean).join(' · ')}</div>}
    <Panel className="form-detail-panel"><SectionHeader eyebrow="Configuration" title="General settings" action={<Toggle label="Enable this form" checked={draft.enabled} onChange={(enabled) => setDraft({ ...draft, enabled })} />} /><div className="section-body form-grid"><TextField title="Form ID" detail="Lowercase letters, numbers, hyphens, and underscores" value={draft.id} onChange={(id) => setDraft({ ...draft, id: id.toLowerCase().replace(/[^a-z0-9_-]/g, '') })} /><TextField title="Form name" detail="Title displayed in the Discord modal" value={draft.name} onChange={(name) => setDraft({ ...draft, name })} /><TextField wide title="Description" detail="Internal context for dashboard editors" value={draft.description} onChange={(description) => setDraft({ ...draft, description })} /><DiscordChannelSelect title="Submission channel" detail="Where completed responses are delivered" value={draft.destination_channel_id || null} kind="text" channels={channels.data ?? []} pending={channels.isLoading} error={channels.isError} onChange={(destination_channel_id) => setDraft({ ...draft, destination_channel_id: destination_channel_id ?? '' })} /><DiscordRoleSelect title="Allowed roles" detail="Leave empty to allow every member" values={draft.allowed_role_ids} roles={roles.data ?? []} pending={roles.isLoading} error={roles.isError} max={25} onChange={(allowed_role_ids) => setDraft({ ...draft, allowed_role_ids })} /><ToggleField title="Anonymous submissions" detail="Do not retain the submitter ID with answers" checked={draft.anonymous} onChange={(anonymous) => setDraft({ ...draft, anonymous })} /></div></Panel>
    <Panel className="question-editor-panel"><SectionHeader eyebrow="Discord modal" title="Questions" detail={`${draft.questions.length} of 5 fields configured`} action={<Button disabled={draft.questions.length >= 5} onClick={addQuestion}><Plus size={16} />Add question</Button>} /><div className="question-editor-list">{draft.questions.map((question, index) => <section className="question-editor-row" key={`${question.id}-${index}`}><header><div><span>{index + 1}</span><strong>{question.label || `Question ${index + 1}`}</strong></div><Button disabled={draft.questions.length <= 1} onClick={() => deleteQuestion(index)}><Trash2 size={16} />Remove</Button></header><div className="form-grid"><TextField title="Question ID" detail="Stable field identifier" value={question.id} onChange={(id) => updateQuestion(index, { ...question, id: id.toLowerCase().replace(/[^a-z0-9_-]/g, '') })} /><TextField title="Label" detail="Prompt shown above the Discord input" value={question.label} onChange={(label) => updateQuestion(index, { ...question, label })} /><SelectField title="Answer style" detail="Single line or long response" value={question.style} options={[["short", "Short answer"], ["paragraph", "Paragraph"]]} onChange={(style) => updateQuestion(index, { ...question, style: style as FormQuestion['style'] })} /><ToggleField title="Required answer" detail="Members must complete this field" checked={question.required} onChange={(required) => updateQuestion(index, { ...question, required })} /><NumberField title="Minimum length" detail="Minimum characters accepted" min={0} max={4000} value={question.min_length} onChange={(min_length) => updateQuestion(index, { ...question, min_length })} /><NumberField title="Maximum length" detail="Maximum characters accepted" min={1} max={4000} value={question.max_length} onChange={(max_length) => updateQuestion(index, { ...question, max_length })} /><TextField wide title="Placeholder" detail="Optional example shown before typing" value={question.placeholder} onChange={(placeholder) => updateQuestion(index, { ...question, placeholder })} /></div></section>)}</div></Panel>
    {(save.isError || remove.isError) && <div className="error-banner" role="alert">{save.error?.message ?? remove.error?.message}</div>}
    <div className="bottom-save-bar">{existing ? <Button className="danger-button" disabled={remove.isPending} onClick={deleteForm}><Trash2 size={16} />Delete form</Button> : <span />}<div className="save-row">{save.isSuccess && <span className="saved-label"><Check size={15} />Saved</span>}<Button disabled={!hasChanges || save.isPending} onClick={saveConfig}>{save.isPending && <LoaderCircle className="spin" size={16} />}Save changes</Button></div></div>
  </div>
}

export function FormSubmissionsPage() {
  const { serverId, formId } = useParams({ strict: false }) as { serverId: string; formId: string }
  const config = useQuery({ queryKey: ['guild-config', serverId], queryFn: () => getGuildConfig(serverId) })
  const submissions = useQuery({ queryKey: ['form-submissions', serverId, formId], queryFn: () => getFormSubmissions(serverId, formId) })
  const form = config.data?.modules.forms.forms.find((item) => item.id === formId) ?? reservedWorkflow(formId)
  const columns = useMemo<Array<ColumnDef<FormSubmission>>>(() => [
    { accessorKey: 'id', header: 'Submission', cell: ({ row }) => <Link to="/servers/$serverId/modules/forms/$formId/submissions/$submissionId" params={{ serverId, formId, submissionId: String(row.original.id) }}>#{row.original.id}</Link> },
    { accessorKey: 'submitter_id', header: 'Submitted by', cell: ({ row }) => row.original.submitter_id ? <DiscordMemberLink serverId={serverId} userId={row.original.submitter_id} username={row.original.username} displayName={row.original.display_name} /> : <Badge tone="neutral">Anonymous</Badge> },
    { id: 'answers', header: 'Answers', accessorFn: (row) => row.answers.length },
    { accessorKey:'status',header:'Status',cell:({row})=><Badge tone={row.original.status==='approved'?'green':row.original.status==='rejected'?'red':'neutral'}>{row.original.status}</Badge> },
    { accessorKey: 'created_at', header: 'Received', cell: ({ row }) => formatDate(row.original.created_at) },
    { id: 'actions', header: '', enableSorting: false, cell: ({ row }) => <Link className="button table-action" to="/servers/$serverId/modules/forms/$formId/submissions/$submissionId" params={{ serverId, formId, submissionId: String(row.original.id) }}>Inspect</Link> },
  ], [formId, serverId])

  if (config.isLoading) return <LoadingState />
  if (!form) return <div className="empty-state"><h2>Form not found</h2><Link to="/servers/$serverId/modules/forms" params={{ serverId }}>Return to forms</Link></div>
  return <div className="page-stack form-submissions-page">
    {formId === 'moderation-appeals' ? <Link className="back-link" to="/servers/$serverId/modules/$moduleId" params={{serverId,moduleId:'moderation_appeals'}}><ArrowLeft size={16}/>Appeal settings</Link> : <Link className="back-link" to="/servers/$serverId/modules/forms" params={{ serverId }}><ArrowLeft size={16} />All forms</Link>}
    <section className="page-intro"><div><p className="eyebrow">Response inbox</p><h2>{form.name}</h2><p>Review responses separately from the form's configuration.</p></div><div className="save-row"><Badge tone="neutral">{submissions.data?.length ?? 0} received</Badge><Button disabled={!submissions.data?.length} onClick={() => downloadSubmissions(serverId, formId, submissions.data ?? [])}><Download size={16}/>Export CSV</Button></div></section>
    <Panel className="table-panel form-submissions-panel"><SectionHeader eyebrow="Inbox" title="Submissions" />{submissions.isError ? <div className="table-empty">Could not load submissions: {submissions.error.message}</div> : <DataTable data={submissions.data ?? []} columns={columns} getRowId={(row) => String(row.id)} ariaLabel={`Submissions for ${form.name}`} emptyMessage={submissions.isLoading ? 'Loading submissions…' : 'No submissions yet'} />}</Panel>
  </div>
}

export function FormSubmissionPage() {
  const { serverId, formId, submissionId } = useParams({ strict: false }) as { serverId: string; formId: string; submissionId: string }
  const config = useQuery({ queryKey: ['guild-config', serverId], queryFn: () => getGuildConfig(serverId) })
  const submission = useQuery({ queryKey: ['form-submission', serverId, formId, submissionId], queryFn: () => getFormSubmission(serverId, formId, submissionId) })
  const client=useQueryClient();const[note,setNote]=useState('');const review=useMutation({mutationFn:reviewFormSubmission,onSuccess:(item)=>{client.setQueryData(['form-submission',serverId,formId,submissionId],item);client.invalidateQueries({queryKey:['form-submissions',serverId,formId]})}})
  const form = config.data?.modules.forms.forms.find((item) => item.id === formId)
  if (config.isLoading || submission.isLoading) return <LoadingState />
  if (submission.isError || !submission.data) return <div className="empty-state"><h2>Submission not found</h2><Link to="/servers/$serverId/modules/forms/$formId" params={{ serverId, formId }}>Return to form</Link></div>
  return <div className="page-stack submission-detail-page"><Link className="back-link" to="/servers/$serverId/modules/forms/$formId/submissions" params={{ serverId, formId }}><ArrowLeft size={16} />{form?.name ?? 'Form'} submissions</Link><section className="page-intro"><div><p className="eyebrow">Submission #{submission.data.id}</p><h2>{form?.name ?? formId}</h2><p>Received {formatDate(submission.data.created_at)}</p></div><div className="save-row"><Badge tone={submission.data.status==='approved'?'green':submission.data.status==='rejected'?'red':'neutral'}>{submission.data.status}</Badge>{submission.data.submitter_id ? <DiscordMemberLink serverId={serverId} userId={submission.data.submitter_id} username={submission.data.username} displayName={submission.data.display_name} /> : <Badge tone="neutral">Anonymous</Badge>}</div></section><Panel className="submission-answers"><SectionHeader eyebrow="Response" title="Submitted answers" action={<Badge tone="neutral">{submission.data.answers.length} answers</Badge>} /><div className="answer-list">{submission.data.answers.map((answer, index) => <article key={`${answer.question_id}-${index}`}><span>{String(index + 1).padStart(2, '0')}</span><div><strong>{answer.label}</strong><p>{answer.answer || 'No answer provided'}</p><small>{answer.question_id}</small></div></article>)}</div></Panel><Panel className="form-detail-panel"><SectionHeader eyebrow="Staff decision" title={formId==='moderation-appeals'?'Review appeal':'Review submission'} detail="Every decision is attributed and written to the audit log."/><div className="section-body form-grid"><label className="wide"><span>Decision note</span><small>Optional context retained with the decision</small><textarea value={note} onChange={event=>setNote(event.target.value)} maxLength={1000}/></label></div><div className="bottom-save-bar"><span>{submission.data.review_note??'No decision note yet.'}</span><div className="save-row"><Button onClick={()=>review.mutate({guildId:serverId,formId,submissionId,status:'closed',note:note||null})}>Close</Button><Button className="danger-button" onClick={()=>review.mutate({guildId:serverId,formId,submissionId,status:'rejected',note:note||null})}>Reject</Button><Button onClick={()=>review.mutate({guildId:serverId,formId,submissionId,status:'approved',note:note||null})}>Approve</Button></div></div></Panel></div>
}

function newForm(index: number): ServerForm { return { enabled: false, id: `form-${index}`, name: `Form ${index}`, description: '', destination_channel_id: '', allowed_role_ids: [], anonymous: false, questions: [{ id: 'question-1', label: 'Your response', style: 'paragraph', required: true, min_length: 1, max_length: 1000, placeholder: 'Type your answer' }] } }
function reservedWorkflow(id:string):ServerForm|undefined { return id==='moderation-appeals'?{enabled:true,id,name:'Moderation appeals',description:'Moderation action review requests',destination_channel_id:'',allowed_role_ids:[],anonymous:false,questions:[]}:undefined }
function TextField({ title, detail, value, onChange, wide = false }: { title: string; detail: string; value: string; onChange: (value: string) => void; wide?: boolean }) { return <label className={wide ? 'wide' : undefined}><span>{title}</span><small>{detail}</small><input value={value} onChange={(event) => onChange(event.target.value)} /></label> }
function NumberField({ title, detail, value, min, max, onChange }: { title: string; detail: string; value: number; min: number; max: number; onChange: (value: number) => void }) { return <label><span>{title}</span><small>{detail}</small><input type="number" min={min} max={max} value={value} onChange={(event) => onChange(Number(event.target.value))} /></label> }
function SelectField({ title, detail, value, options, onChange }: { title: string; detail: string; value: string; options: [string, string][]; onChange: (value: string) => void }) { return <label><span>{title}</span><small>{detail}</small><select value={value} onChange={(event) => onChange(event.target.value)}>{options.map(([option, label]) => <option value={option} key={option}>{label}</option>)}</select></label> }
function ToggleField({ title, detail, checked, onChange }: { title: string; detail: string; checked: boolean; onChange: (value: boolean) => void }) { return <div className="module-toggle-field"><span><strong>{title}</strong><small>{detail}</small></span><Toggle label={title} checked={checked} onChange={onChange} /></div> }
function formatDate(value: string) { return new Intl.DateTimeFormat(undefined, { dateStyle: 'medium', timeStyle: 'short' }).format(new Date(value)) }
export function submissionsCsv(items: FormSubmission[]) { const rows = [['submission_id','status','submitter_id','username','display_name','received_at','answers'], ...items.map((item) => [String(item.id),item.status,item.submitter_id??'',item.username??'',item.display_name??'',item.created_at,JSON.stringify(item.answers)])]; return rows.map((row) => row.map(csvCell).join(',')).join('\n') }
function csvCell(value:string){const safe=/^[=+\-@]/.test(value)?`'${value}`:value;return `"${safe.replaceAll('"','""')}"`}
function downloadSubmissions(serverId:string,formId:string,items:FormSubmission[]){const url=URL.createObjectURL(new Blob([submissionsCsv(items)],{type:'text/csv;charset=utf-8'}));const anchor=document.createElement('a');anchor.href=url;anchor.download=`${formId}-submissions.csv`;anchor.click();URL.revokeObjectURL(url);void recordDashboardActivity({action:'dashboard.data.exported',guild_id:serverId,export:'form_submissions_csv'})}
function LoadingState() { return <div className="loading-state"><LoaderCircle className="spin" /><span>Loading forms…</span></div> }

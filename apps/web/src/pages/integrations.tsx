import { useParams } from '@tanstack/react-router'
import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query'
import { Check, Clipboard, KeyRound, LoaderCircle, ShieldCheck, Trash2 } from 'lucide-react'
import { useState } from 'react'
import { Badge, Button, Panel } from '../components/ui'
import { createMcpAccessToken, getMcpAccessTokens, revokeMcpAccessToken } from '../lib/api'
import type { CreatedMcpAccessToken } from '../types'
import { PageIntro } from './system/page-intro'

export function IntegrationsPage() {
  const { serverId } = useParams({ strict: false }) as { serverId: string }
  const client = useQueryClient()
  const [name, setName] = useState('My AI assistant')
  const [expiresInDays, setExpiresInDays] = useState(90)
  const [created, setCreated] = useState<CreatedMcpAccessToken | null>(null)
  const [copied, setCopied] = useState(false)
  const tokens = useQuery({ queryKey: ['mcp-access-tokens', serverId], queryFn: () => getMcpAccessTokens(serverId) })
  const create = useMutation({
    mutationFn: () => createMcpAccessToken({ guildId: serverId, name, expiresInDays }),
    onSuccess: async (token) => {
      setCreated(token)
      setCopied(false)
      await client.invalidateQueries({ queryKey: ['mcp-access-tokens', serverId] })
    },
  })
  const revoke = useMutation({
    mutationFn: (tokenId: number) => revokeMcpAccessToken(serverId, tokenId),
    onSuccess: async () => client.invalidateQueries({ queryKey: ['mcp-access-tokens', serverId] }),
  })
  const copyToken = async () => {
    if (!created) return
    await navigator.clipboard.writeText(created.access_token)
    setCopied(true)
  }

  return <div className="page-stack">
    <PageIntro eyebrow="Server integrations" title="AI agent access" description="Let an MCP-compatible assistant help manage this server under your Discord identity and through the same audit trail as the dashboard." />
    <section className="notice"><ShieldCheck /><div><strong>Every credential is limited to this server.</strong><p>It cannot inspect Orivane’s fleet, other servers, global users, infrastructure, or bot-owner controls. Every change is attributed to your Discord account.</p></div></section>

    {created && <Panel className="settings-panel mcp-created-token" aria-live="polite">
      <div className="panel-heading"><div><p className="eyebrow">Copy now</p><h3>Your new access token</h3></div><Badge tone="amber">Shown once</Badge></div>
      <p className="muted-copy">Store this in your MCP client’s secret manager. Orivane only stores a one-way hash and cannot show it again.</p>
      <div className="token-copy-row"><code>{created.access_token}</code><Button type="button" onClick={() => void copyToken()}>{copied ? <Check size={16} /> : <Clipboard size={16} />}{copied ? 'Copied' : 'Copy token'}</Button></div>
    </Panel>}

    <Panel className="settings-panel">
      <div className="panel-heading"><div><p className="eyebrow">New connection</p><h3>Create a server access token</h3></div><KeyRound size={20} /></div>
      <div className="form-grid"><label><span>Connection name</span><small>Use the assistant or device name</small><input value={name} maxLength={80} onChange={(event) => setName(event.target.value)} /></label><label><span>Expires after</span><small>Shorter lifetimes reduce risk</small><select value={expiresInDays} onChange={(event) => setExpiresInDays(Number(event.target.value))}><option value={30}>30 days</option><option value={90}>90 days</option><option value={180}>180 days</option><option value={365}>1 year</option></select></label></div>
      {create.isError && <div className="error-banner" role="alert">{create.error.message}</div>}
      <div className="module-save-bar"><span><strong>Server-scoped access</strong><small>You can revoke this connection at any time.</small></span><Button disabled={create.isPending || !name.trim()} onClick={() => create.mutate()}>{create.isPending ? <LoaderCircle className="spin" size={16} /> : <KeyRound size={16} />}Create access token</Button></div>
    </Panel>

    <Panel className="system-panel">
      <div className="panel-heading"><div><p className="eyebrow">Connected assistants</p><h3>Active access tokens</h3></div><Badge tone="neutral">{tokens.data?.length ?? 0}</Badge></div>
      {tokens.isLoading && <div className="loading-state">Loading connections…</div>}
      {tokens.isError && <div className="error-banner" role="alert">{tokens.error.message}</div>}
      {!tokens.isLoading && !tokens.data?.length && <div className="empty-state"><h3>No assistants connected</h3><p>Create a token above when you are ready to connect an MCP client.</p></div>}
      {!!tokens.data?.length && <div className="mcp-token-list">{tokens.data.map((token) => <article key={token.id}><span className="system-icon"><KeyRound size={17} /></span><span><strong>{token.name}</strong><small>Created {new Date(token.created_at).toLocaleDateString()} · {token.last_used_at ? `Last used ${new Date(token.last_used_at).toLocaleString()}` : 'Never used'}</small></span><Badge tone={new Date(token.expires_at) > new Date() ? 'green' : 'red'}>Expires {new Date(token.expires_at).toLocaleDateString()}</Badge><Button className="danger-button" disabled={revoke.isPending} onClick={() => revoke.mutate(token.id)}><Trash2 size={15} />Revoke</Button></article>)}</div>}
    </Panel>
  </div>
}

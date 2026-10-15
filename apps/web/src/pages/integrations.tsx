import { useParams } from '@tanstack/react-router'
import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query'
import { Check, Clipboard, KeyRound, LoaderCircle, ShieldCheck, Trash2 } from 'lucide-react'
import { useState } from 'react'
import { Badge, Button, Panel, SectionHeader } from '../components/ui'
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

  return <div className="page-stack integrations-page">
    <PageIntro eyebrow="Server integrations" title="AI agent access" description="Let an MCP-compatible assistant help manage this server under your Discord identity and through the same audit trail as the dashboard." />
    <section className="integration-security-notice">
      <span className="integration-security-icon"><ShieldCheck size={20} /></span>
      <div><strong>Every credential is limited to this server.</strong><p>It cannot inspect Orivane’s fleet, other servers, global users, infrastructure, or bot-owner controls. Every change is attributed to your Discord account.</p></div>
    </section>

    {created && <Panel className="integration-panel mcp-created-token" aria-live="polite">
      <SectionHeader eyebrow="Copy now" title="Your new access token" detail="Store this in your MCP client’s secret manager. Orivane only stores a one-way hash and cannot show it again." action={<Badge tone="amber">Shown once</Badge>} />
      <div className="section-body"><div className="token-copy-row"><code>{created.access_token}</code><Button type="button" onClick={() => void copyToken()}>{copied ? <Check size={16} /> : <Clipboard size={16} />}{copied ? 'Copied' : 'Copy token'}</Button></div></div>
    </Panel>}

    <div className="integrations-grid">
      <Panel className="integration-panel integration-create-panel">
        <SectionHeader eyebrow="New connection" title="Create a server access token" detail="Name the assistant and choose how long it should retain access." action={<KeyRound size={20} />} />
        <div className="section-body integration-create-fields"><div className="form-grid"><label><span>Connection name</span><small>Use the assistant or device name</small><input value={name} maxLength={80} onChange={(event) => setName(event.target.value)} /></label><label><span>Expires after</span><small>Shorter lifetimes reduce risk</small><select value={expiresInDays} onChange={(event) => setExpiresInDays(Number(event.target.value))}><option value={30}>30 days</option><option value={90}>90 days</option><option value={180}>180 days</option><option value={365}>1 year</option></select></label></div>{create.isError && <div className="error-banner" role="alert">{create.error.message}</div>}</div>
        <footer className="integration-panel-footer"><span><strong>Server-scoped access</strong><small>You can revoke this connection at any time.</small></span><Button disabled={create.isPending || !name.trim()} onClick={() => create.mutate()}>{create.isPending ? <LoaderCircle className="spin" size={16} /> : <KeyRound size={16} />}Create access token</Button></footer>
      </Panel>

      <Panel className="integration-panel integration-connections-panel">
        <SectionHeader eyebrow="Connected assistants" title="Active access tokens" detail="Review access, recent use, expiry, and revoke credentials you no longer recognize." action={<Badge tone="neutral">{tokens.data?.length ?? 0}</Badge>} />
        <div className="integration-connections-body">
          {tokens.isLoading && <div className="integration-loading-state">Loading connections…</div>}
          {tokens.isError && <div className="error-banner" role="alert">{tokens.error.message}</div>}
          {!tokens.isLoading && !tokens.data?.length && <div className="integration-empty-state"><KeyRound size={22} /><h3>No assistants connected</h3><p>Create a token when you are ready to connect an MCP client.</p></div>}
          {!!tokens.data?.length && <div className="mcp-token-list">{tokens.data.map((token) => <article key={token.id}><div className="mcp-token-identity"><span className="system-icon"><KeyRound size={17} /></span><span><strong>{token.name}</strong><small>Created {new Date(token.created_at).toLocaleDateString()} · {token.last_used_at ? `Last used ${new Date(token.last_used_at).toLocaleString()}` : 'Never used'}</small></span></div><div className="mcp-token-actions"><Badge tone={new Date(token.expires_at) > new Date() ? 'green' : 'red'}>Expires {new Date(token.expires_at).toLocaleDateString()}</Badge><Button className="danger-button" disabled={revoke.isPending} onClick={() => revoke.mutate(token.id)}><Trash2 size={15} />Revoke</Button></div></article>)}</div>}
        </div>
      </Panel>
    </div>
  </div>
}

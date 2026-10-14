import { AlertTriangle, Bot, LockKeyhole, Route } from 'lucide-react'
import { CodeBlock } from '../../components/code-block'
import { PageHeader } from '../../components/page-header'

export function McpPage() {
  return <div className="content-page prose-page">
    <PageHeader eyebrow="Agent access" title="Model Context Protocol" description="Trusted agents can read and operate Orivane through the same validated, audited API used by the dashboard—without direct database, Discord-token, or arbitrary HTTP access." />
    <section className="notice warning"><AlertTriangle /><div><strong>The MCP token is bot-owner access.</strong><p>Use a unique secret of at least 32 random characters. Never reuse the bot token, OAuth client secret, or Better Auth secret, and never place it in a tracked file.</p></div></section>
    <section><h2>Security boundary</h2><div className="three-column-list"><div className="doc-panel"><LockKeyhole /><h3>Dedicated identity</h3><p>The API attributes calls to the synthetic owner actor <code>orivane-mcp</code>.</p></div><div className="doc-panel"><Route /><h3>Static allowlist</h3><p>Only known <code>/v1</code> method-and-path combinations can be forwarded.</p></div><div className="doc-panel"><Bot /><h3>Write separation</h3><p>Read and mutation tools are separate, with destructive metadata for client approvals.</p></div></div></section>
    <section><h2>Available tools</h2><div className="setting-table simple-table"><div className="setting-row setting-header"><span>Tool</span><span>Purpose</span><span>State</span></div><div className="setting-row"><code>api_endpoints</code><span>Discover every available method, path, description, and mutation flag.</span><span className="type-pill">read only</span></div><div className="setting-row"><code>api_get</code><span>Read an allowlisted endpoint with URL-encoded query parameters.</span><span className="type-pill">read only</span></div><div className="setting-row"><code>api_mutate</code><span>Call an allowlisted POST, PUT, PATCH, or DELETE with an optional JSON body.</span><span className="type-pill danger">destructive</span></div></div></section>
    <section><h2>Local stdio client</h2><p>Build the Rust binary, then inject the token through your agent host’s secret or environment mechanism.</p><CodeBlock label="Build" code="cargo build --release -p discord-bot-mcp" /><CodeBlock label="Generic MCP client configuration" code={`{
  "mcpServers": {
    "orivane": {
      "command": "/absolute/path/to/discord-bot/target/release/discord-bot-mcp",
      "env": {
        "MCP_TRANSPORT": "stdio",
        "ORIVANE_API_BASE_URL": "http://127.0.0.1:3010/",
        "ORIVANE_MCP_TOKEN": "<inject-secret-here>"
      }
    }
  }
}`} /></section>
    <section><h2>Authenticated Streamable HTTP</h2><p>Compose publishes the MCP health port on loopback and the web proxy exposes the protocol at <code>/mcp</code>. Every protocol request still requires the bearer token.</p><CodeBlock label="Endpoint" code={`POST https://your-orivane-host.example/mcp
Authorization: Bearer <ORIVANE_MCP_TOKEN>
Accept: application/json, text/event-stream
Content-Type: application/json`} /></section>
    <section><h2>How agents should operate</h2><ol className="plain-steps"><li><span>1</span><p>Call <code>api_endpoints</code> instead of guessing paths or payloads.</p></li><li><span>2</span><p>Prefer <code>api_get</code> and inspect current state before proposing a change.</p></li><li><span>3</span><p>Ask for explicit user confirmation before <code>api_mutate</code>.</p></li><li><span>4</span><p>Return the API request ID so the operation can be found in Orivane’s logs and audit pages.</p></li></ol></section>
  </div>
}

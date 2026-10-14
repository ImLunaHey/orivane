import { Link } from '@tanstack/react-router'
import { ArrowRight, Bot, Braces, CheckCircle2, ShieldCheck, SlidersHorizontal } from 'lucide-react'
import { CodeBlock } from '../../components/code-block'
import { modules, settingCount } from '../../content/modules'
import { FeatureCard } from './feature-card'

const featuredIds = ['automod', 'tickets', 'forms', 'leveling', 'anti_raid', 'managed_embeds']

export function HomePage() {
  const commandCount = new Set(modules.flatMap((module) => module.commands)).size
  const settings = modules.reduce((total, module) => total + settingCount(module), 0)
  const featured = featuredIds.flatMap((id) => modules.filter((module) => module.id === id))
  return <div className="landing-page">
    <section className="hero">
      <div className="hero-copy"><p className="eyebrow">Discord operations, without the guesswork</p><h1>Build a calmer server with <span>Orivane.</span></h1><p>Configure moderation, onboarding, support, engagement, and automation from one audited control plane. These docs map every live module and setting to the behavior members see in Discord.</p><div className="hero-actions"><Link className="button primary" to="/guides/getting-started">Start configuring <ArrowRight size={17} /></Link><Link className="button secondary" to="/modules">Browse modules</Link></div></div>
      <div className="hero-console" aria-label="Example Orivane setup flow">
        <div className="console-top"><span /><span /><span /><small>orivane / setup</small></div>
        <div className="console-body"><p><span>01</span><CheckCircle2 />Connect Discord and choose a server</p><p><span>02</span><SlidersHorizontal />Enable only the modules you need</p><p><span>03</span><ShieldCheck />Review resource diagnostics</p><p><span>04</span><Bot />Test the slash-command workflow</p><div className="console-status"><i />Configuration validated and audited</div></div>
      </div>
    </section>

    <section className="proof-strip" aria-label="Documentation coverage"><div><strong>{modules.length}</strong><span>live modules</span></div><div><strong>{settings}</strong><span>documented settings</span></div><div><strong>{commandCount}</strong><span>slash commands</span></div><div><strong>0</strong><span>prefix commands</span></div></section>

    <section className="landing-section"><div className="section-heading"><div><p className="eyebrow">Explore the platform</p><h2>Start with the workflow you need</h2></div><Link to="/modules">See all {modules.length} modules <ArrowRight size={16} /></Link></div><div className="feature-grid">{featured.map((module) => <FeatureCard key={module.id} module={module} />)}</div></section>

    <section className="split-section"><div><p className="eyebrow">A predictable control plane</p><h2>Every change has a visible path.</h2><p>Server configuration is validated before it is stored, refreshed through the shared cache, and attached to a durable audit event. The dashboard never needs your bot token, and every Discord-facing feature remains isolated in its own Rust crate.</p><ul className="check-list"><li><CheckCircle2 />Server-scoped permissions through Discord OAuth</li><li><CheckCircle2 />Before-and-after state for configuration mutations</li><li><CheckCircle2 />Content-safe analytics and correlated request IDs</li><li><CheckCircle2 />Mockable Discord API boundary for deterministic tests</li></ul></div><CodeBlock label="Architecture" code={`Dashboard or trusted MCP agent\n              ↓\n      Axum configuration API\n         ↙           ↘\n   PostgreSQL       Redis\n         ↘           ↙\n     Rust gateway workers\n              ↓\n           Discord`} /></section>

    <section className="cta-panel"><span className="cta-icon"><Braces /></span><div><p className="eyebrow">For operators and agents</p><h2>Use the dashboard—or the same audited API through MCP.</h2><p>Agent access has a dedicated owner credential, explicit route allowlist, read/write separation, and both stdio and authenticated Streamable HTTP transports.</p></div><Link className="button secondary" to="/guides/mcp">Set up MCP <ArrowRight size={17} /></Link></section>
  </div>
}

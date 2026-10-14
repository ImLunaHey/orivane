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

    <section className="split-section"><div><p className="eyebrow">A predictable control centre</p><h2>Every change has a visible path.</h2><p>Configure your server from the dashboard or a server-scoped AI assistant. Orivane validates changes, applies them to Discord, and records the result in your server audit log.</p><ul className="check-list"><li><CheckCircle2 />Server access verified through Discord</li><li><CheckCircle2 />Before-and-after state for configuration changes</li><li><CheckCircle2 />Useful analytics without fake activity</li><li><CheckCircle2 />Request IDs for tracing failed actions</li></ul></div><CodeBlock label="How a change flows" code={`You approve a dashboard or MCP change\n                 ↓\n       Orivane validates access\n                 ↓\n      Configuration is updated\n                 ↓\n       Discord action is applied\n                 ↓\n    Result appears in the audit log`} /></section>

    <section className="cta-panel"><span className="cta-icon"><Braces /></span><div><p className="eyebrow">Your server, your choice</p><h2>Use the dashboard—or connect your AI assistant.</h2><p>Every MCP connection is issued after a Discord permission check, limited to one server, expires automatically, and can be revoked from the dashboard.</p></div><Link className="button secondary" to="/guides/mcp">Connect an assistant <ArrowRight size={17} /></Link></section>
  </div>
}

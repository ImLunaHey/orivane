import { Link, useParams } from '@tanstack/react-router'
import { ArrowLeft, CheckCircle2, Command, Settings2 } from 'lucide-react'
import { findModule, settingCount } from '../../content/modules'
import { DemoPanel } from '../demos/demo-panel'
import { NotFoundPage } from '../not-found/not-found-page'
import { SettingTable } from './setting-table'

export function ModulePage() {
  const { moduleId } = useParams({ from: '/modules/$moduleId' })
  const module = findModule(moduleId)
  if (!module) return <NotFoundPage />
  return <article className="content-page module-page">
    <Link className="back-link" to="/modules"><ArrowLeft size={16} />All modules</Link>
    <header className="module-hero"><div><div className="hero-meta"><span className="category-pill">{module.category}</span><span>Live Rust module</span></div><h1>{module.title}</h1><p>{module.description}</p></div><dl><div><dt>Configuration fields</dt><dd>{settingCount(module)}</dd></div><div><dt>Slash commands</dt><dd>{module.commands.length}</dd></div></dl></header>
    <section className="module-overview-grid"><div className="doc-panel"><p className="eyebrow">What it does</p><h2>Capabilities</h2><ul className="check-list">{module.highlights.map((highlight) => <li key={highlight}><CheckCircle2 />{highlight}</li>)}</ul></div><div className="doc-panel"><p className="eyebrow">Member interface</p><h2>Slash commands</h2>{module.commands.length ? <div className="command-list">{module.commands.map((command) => <code key={command}>{command}</code>)}</div> : <p>This module is driven by Discord events, dashboard actions, or message components; it does not add a slash command.</p>}</div></section>
    <section className="setup-flow"><div className="section-heading"><div><p className="eyebrow">Recommended setup</p><h2>Configure in this order</h2></div></div><ol><li><span>1</span><div><strong>Grant resources</strong><p>Make sure Orivane can view and act in every selected channel, category, and role.</p></div></li><li><span>2</span><div><strong>Set policy</strong><p>Review the module defaults below, add repeatable rules, and choose explicit destinations.</p></div></li><li><span>3</span><div><strong>Enable and test</strong><p>Save at the bottom of the dashboard page, then exercise one private-server workflow and inspect its audit event.</p></div></li></ol></section>
    {module.demo ? <DemoPanel kind={module.demo} /> : null}
    <section className="configuration-intro"><Settings2 /><div><p className="eyebrow">Configuration reference</p><h2>Every available setting</h2><p>Keys match the serialized Rust configuration contract. Repeatable entries use <code>[]</code>; Discord resources are selected by name in the dashboard rather than typed as raw IDs.</p></div></section>
    {module.settingGroups.map((group) => <SettingTable key={group.title} group={group} />)}
    <section className="audit-callout"><Command /><div><h2>Validate the outcome, not just the toggle</h2><p>After saving, open resource diagnostics and the server audit log. Configuration changes include the actor, request ID, before/after state, and outcome; Discord actions emit their own correlated operational events.</p></div></section>
  </article>
}

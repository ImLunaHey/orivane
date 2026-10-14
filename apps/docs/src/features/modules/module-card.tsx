import { Link } from '@tanstack/react-router'
import { ArrowRight } from 'lucide-react'
import { settingCount } from '../../content/modules'
import type { ModuleDoc } from '../../content/types'

export function ModuleCard({ module }: { module: ModuleDoc }) {
  return <article className="module-doc-card"><div><span className="category-pill">{module.category}</span>{module.demo ? <span className="demo-pill">Interactive demo</span> : null}</div><h2>{module.title}</h2><p>{module.summary}</p><dl><div><dt>Settings</dt><dd>{settingCount(module)}</dd></div><div><dt>Commands</dt><dd>{module.commands.length || 'Event driven'}</dd></div></dl><Link to="/modules/$moduleId" params={{ moduleId: module.id }}>Read guide <ArrowRight size={16} /></Link></article>
}

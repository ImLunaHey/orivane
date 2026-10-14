import { Search } from 'lucide-react'
import { useMemo, useState } from 'react'
import { PageHeader } from '../../components/page-header'
import { moduleCategories, modules } from '../../content/modules'
import type { ModuleCategory } from '../../content/types'
import { ModuleCard } from './module-card'

export function ModuleIndexPage() {
  const [query, setQuery] = useState('')
  const [category, setCategory] = useState<ModuleCategory | 'All'>('All')
  const filtered = useMemo(() => modules.filter((module) => {
    const categoryMatch = category === 'All' || module.category === category
    const needle = query.trim().toLowerCase()
    const textMatch = !needle || [module.title, module.summary, module.description, ...module.commands].join(' ').toLowerCase().includes(needle)
    return categoryMatch && textMatch
  }), [category, query])
  return <div className="content-page">
    <PageHeader eyebrow="Feature reference" title="Every Orivane module" description="All modules below are live Rust implementations. Open one to see its exact configuration surface, member workflow, slash commands, and an interactive demo where the behavior can be modeled safely in your browser." />
    <div className="catalog-controls"><label className="catalog-search"><span className="sr-only">Filter modules</span><Search size={17} /><input value={query} onChange={(event) => setQuery(event.target.value)} placeholder="Filter modules" /></label><div className="filter-chips" aria-label="Filter by category"><button type="button" aria-pressed={category === 'All'} onClick={() => setCategory('All')}>All</button>{moduleCategories.map((item) => <button key={item} type="button" aria-pressed={category === item} onClick={() => setCategory(item)}>{item}</button>)}</div></div>
    <p className="result-count">Showing {filtered.length} of {modules.length} modules</p>
    <div className="module-doc-grid">{filtered.map((module) => <ModuleCard key={module.id} module={module} />)}</div>
    {!filtered.length ? <div className="empty-search">No modules match that filter.</div> : null}
  </div>
}

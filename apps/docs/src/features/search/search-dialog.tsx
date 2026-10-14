import { Link } from '@tanstack/react-router'
import { Search, X } from 'lucide-react'
import { useEffect, useEffectEvent, useMemo, useRef, useState } from 'react'
import { modules, settingCount } from '../../content/modules'

export function SearchDialog({ open, onClose }: { open: boolean; onClose: () => void }) {
  const [query, setQuery] = useState('')
  const input = useRef<HTMLInputElement>(null)
  const close = useEffectEvent(onClose)
  useEffect(() => {
    if (!open) return
    input.current?.focus()
    const closeOnEscape = (event: KeyboardEvent) => { if (event.key === 'Escape') close() }
    window.addEventListener('keydown', closeOnEscape)
    return () => window.removeEventListener('keydown', closeOnEscape)
  }, [open])
  const results = useMemo(() => {
    const needle = query.trim().toLowerCase()
    if (!needle) return modules.slice(0, 8)
    return modules.filter((module) => [module.title, module.summary, module.category, ...module.commands, ...module.settingGroups.flatMap((group) => group.settings.flatMap((setting) => [setting.key, setting.label]))].join(' ').toLowerCase().includes(needle)).slice(0, 12)
  }, [query])
  if (!open) return null
  return <div className="dialog-backdrop" role="presentation" onMouseDown={(event) => { if (event.target === event.currentTarget) onClose() }}>
    <dialog className="search-dialog" open aria-labelledby="search-title">
      <div className="search-dialog-head"><div><Search size={19} /><h2 id="search-title">Search documentation</h2></div><button type="button" aria-label="Close search" onClick={onClose}><X size={19} /></button></div>
      <label className="search-input"><span className="sr-only">Search modules, settings, and commands</span><Search size={18} /><input ref={input} value={query} onChange={(event) => setQuery(event.target.value)} placeholder="Try automod, retention, or /warn" /></label>
      <div className="search-results" aria-live="polite">
        {results.map((module) => <Link key={module.id} to="/modules/$moduleId" params={{ moduleId: module.id }} onClick={onClose}>
          <span><strong>{module.title}</strong><small>{module.category} · {settingCount(module)} documented settings</small></span><span>Open →</span>
        </Link>)}
        {!results.length ? <p className="empty-search">No matching documentation. Try a module name, slash command, or configuration key.</p> : null}
      </div>
    </dialog>
  </div>
}

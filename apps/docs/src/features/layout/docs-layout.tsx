import { Link, Outlet } from '@tanstack/react-router'
import { BookOpen, Bot, Code2, Menu, Search, X } from 'lucide-react'
import { useCallback, useEffect, useState } from 'react'
import { ModuleNav } from './module-nav'
import { SearchDialog } from '../search/search-dialog'

export function DocsLayout() {
  const [searchOpen, setSearchOpen] = useState(false)
  const [menuOpen, setMenuOpen] = useState(false)
  const closeSearch = useCallback(() => setSearchOpen(false), [])
  useEffect(() => {
    const openSearch = (event: KeyboardEvent) => {
      if ((event.metaKey || event.ctrlKey) && event.key.toLowerCase() === 'k') {
        event.preventDefault()
        setSearchOpen(true)
      }
      if (event.key === 'Escape') setMenuOpen(false)
    }
    window.addEventListener('keydown', openSearch)
    return () => window.removeEventListener('keydown', openSearch)
  }, [])
  useEffect(() => {
    if (!menuOpen && !searchOpen) return
    const previous = document.body.style.overflow
    document.body.style.overflow = 'hidden'
    return () => { document.body.style.overflow = previous }
  }, [menuOpen, searchOpen])
  return <div className="docs-shell">
    <a className="skip-link" href="#main-content">Skip to documentation</a>
    <header className="topbar">
      <Link className="brand" to="/"><span className="brand-mark"><Bot size={21} /></span><span><strong>Orivane</strong><small>Documentation</small></span></Link>
      <button className="search-button" type="button" onClick={() => setSearchOpen(true)}><Search size={17} /><span>Search docs</span><kbd>Ctrl K</kbd></button>
      <div className="topbar-actions">
        <a href="https://github.com/ImLunaHey/orivane" target="_blank" rel="noreferrer"><Code2 size={18} /><span>GitHub</span></a>
        <button className="menu-button" type="button" aria-label={menuOpen ? 'Close navigation' : 'Open navigation'} aria-expanded={menuOpen} onClick={() => setMenuOpen((value) => !value)}>{menuOpen ? <X /> : <Menu />}</button>
      </div>
    </header>
    <aside className={`sidebar ${menuOpen ? 'sidebar-open' : ''}`}>
      <nav aria-label="Documentation">
        <p className="nav-label">Start here</p>
        <Link to="/" activeOptions={{ exact: true }} activeProps={{ 'data-active': true }} onClick={() => setMenuOpen(false)}><BookOpen size={16} />Overview</Link>
        <Link to="/guides/getting-started" activeProps={{ 'data-active': true }} onClick={() => setMenuOpen(false)}>Getting started</Link>
        <Link to="/guides/mcp" activeProps={{ 'data-active': true }} onClick={() => setMenuOpen(false)}>Agent access (MCP)</Link>
        <p className="nav-label nav-label-spaced">Reference</p>
        <Link to="/modules" activeOptions={{ exact: true }} activeProps={{ 'data-active': true }} onClick={() => setMenuOpen(false)}>All modules</Link>
        <Link to="/reference/commands" activeProps={{ 'data-active': true }} onClick={() => setMenuOpen(false)}>Slash commands</Link>
        <Link to="/reference/audit" activeProps={{ 'data-active': true }} onClick={() => setMenuOpen(false)}>Audit and analytics</Link>
        <p className="nav-label nav-label-spaced">Modules</p>
        <ModuleNav />
      </nav>
    </aside>
    {menuOpen && <button className="nav-backdrop" type="button" aria-label="Close navigation overlay" onClick={() => setMenuOpen(false)} />}
    <main id="main-content" tabIndex={-1}><Outlet /></main>
    <SearchDialog open={searchOpen} onClose={closeSearch} />
  </div>
}

import { Link, Outlet, useNavigate, useRouterState } from '@tanstack/react-router'
import { Activity, BarChart3, Bot, ChevronDown, Command, FileClock, LayoutDashboard, LogOut, Menu, Server, Settings2, Shield, UserRoundSearch, Users, X } from 'lucide-react'
import { useEffect, useState } from 'react'
import { useQuery } from '@tanstack/react-query'
import { getServers, isDemoMode, recordDashboardActivity } from '../lib/api'
import { authClient } from '../lib/auth-client'
import { PaletteContext } from '../lib/palette'
import { demoServers } from '../lib/server-data'
import { Badge } from './ui'
import { PageErrorBoundary } from './page-error-boundary'
import { ResourceHealthBanner } from './resource-health-banner'

const demoServerId = '901234567890123456'
let lastRecordedPath = ''

const globalNavigation = [
  { to: '/', label: 'Bot overview', icon: LayoutDashboard },
  { to: '/servers', label: 'Servers', icon: Server },
  { to: '/users', label: 'Global users', icon: UserRoundSearch },
  { to: '/audit', label: 'Audit log', icon: FileClock },
  { to: '/health', label: 'Health', icon: Activity },
  { to: '/commands', label: 'Commands', icon: Command },
] as const

const serverNavigation = [
  { to: '/servers/$serverId', label: 'Overview', icon: LayoutDashboard },
  { to: '/servers/$serverId/modules', label: 'Modules', icon: Settings2 },
  { to: '/servers/$serverId/moderation', label: 'Moderation', icon: Shield },
  { to: '/servers/$serverId/analytics', label: 'Analytics', icon: BarChart3 },
  { to: '/servers/$serverId/members', label: 'Members', icon: Users },
  { to: '/servers/$serverId/audit', label: 'Audit log', icon: FileClock },
] as const

export function AppShell() {
  const [mobileOpen, setMobileOpen] = useState(false)
  const [paletteOpen, setPaletteOpen] = useState(false)
  const [serverMenuOpen, setServerMenuOpen] = useState(false)
  const [accountMenuOpen, setAccountMenuOpen] = useState(false)
  const session = authClient.useSession()
  const navigate = useNavigate()
  const pathname = useRouterState({ select: (state) => state.location.pathname })
  const serverId = pathname.match(/^\/servers\/([^/]+)/)?.[1] ?? demoServerId
  const serversQuery = useQuery({ queryKey: ['servers'], queryFn: getServers })
  const servers = serversQuery.data ?? (isDemoMode ? demoServers : [])
  const selectedServer = servers.find((server) => server.id === serverId) ?? { id: serverId, name: `Server ${serverId.slice(-4)}`, members: 0, commands: 0, status: 'Connected' as const, shard: 0, installer: 'Unattributed', installerId: '' }
  const inServer = pathname.startsWith(`/servers/${serverId}`)
  const current = pageTitle(pathname)

  useEffect(() => {
    const onKeyDown = (event: KeyboardEvent) => {
      if ((event.metaKey || event.ctrlKey) && event.key.toLowerCase() === 'k') {
        event.preventDefault()
        setPaletteOpen((open) => !open)
      }
      if (event.key === 'Escape') {
        setPaletteOpen(false)
        setServerMenuOpen(false)
        setAccountMenuOpen(false)
      }
    }
    window.addEventListener('keydown', onKeyDown)
    return () => window.removeEventListener('keydown', onKeyDown)
  }, [])

  useEffect(() => {
    if (lastRecordedPath === pathname) return
    lastRecordedPath = pathname
    const scopedServerId = pathname.match(/^\/servers\/([^/]+)/)?.[1]
    void recordDashboardActivity({ action: 'dashboard.page.viewed', guild_id: scopedServerId, path: pathname })
  }, [pathname])

  return (
    <div className="app-shell">
      <aside aria-label="Primary navigation" className={`sidebar ${mobileOpen ? 'sidebar-open' : ''}`}>
        <div className="brand-row">
          <div className="brand-mark"><Bot size={20} strokeWidth={2.3} /></div>
          <div><strong>Orivane</strong><span>Control centre</span></div>
          <button className="mobile-close" onClick={() => setMobileOpen(false)} aria-label="Close navigation"><X size={20} /></button>
        </div>

        {inServer && <div className="server-switcher" onBlur={(event) => { if (!event.currentTarget.contains(event.relatedTarget)) setServerMenuOpen(false) }} onKeyDown={(event) => { if (event.key === 'Escape') setServerMenuOpen(false) }}>
          <button className="guild-picker" aria-haspopup="menu" aria-expanded={serverMenuOpen} onClick={() => setServerMenuOpen((open) => !open)}>
            <span className="guild-avatar">{selectedServer.name.charAt(0)}</span>
            <span><strong>{selectedServer.name}</strong><small>{selectedServer.id.slice(0, 4)}…{selectedServer.id.slice(-4)}</small></span>
            <ChevronDown size={16} className={serverMenuOpen ? 'chevron-open' : ''} />
          </button>
          {serverMenuOpen && <div className="server-menu" role="menu">
            <p>Switch server</p>
            {servers.map((server) => <Link key={server.id} to="/servers/$serverId" params={{ serverId: server.id }} role="menuitem" className={server.id === serverId ? 'server-option server-option-active' : 'server-option'} onClick={() => { setServerMenuOpen(false); setMobileOpen(false) }}><span className="guild-avatar">{server.name.charAt(0)}</span><span><strong>{server.name}</strong><small>{server.members.toLocaleString()} members</small></span>{server.id === serverId && <span className="menu-check">✓</span>}</Link>)}
            <Link to="/servers" role="menuitem" className="all-servers-link" onClick={() => { setServerMenuOpen(false); setMobileOpen(false) }}><Server size={15} />View all servers</Link>
          </div>}
        </div>}

        <nav aria-label="Primary navigation">
          <p className="nav-heading">Bot</p>
          {(inServer ? globalNavigation.slice(0, 2) : globalNavigation).map(({ to, label, icon: Icon }) => (
            <Link key={to} to={to} activeOptions={{ exact: true }} className="nav-link" activeProps={{ className: 'nav-link nav-link-active' }} onClick={() => setMobileOpen(false)}>
              <Icon size={18} />{label}{label === 'Health' && <span className="health-dot" />}
            </Link>
          ))}
          {inServer && <><p className="nav-heading nav-heading-space">{selectedServer.name}</p>
          {serverNavigation.map(({ to, label, icon: Icon }) => (
            <Link key={to} to={to} params={{ serverId }} activeOptions={{ exact: label === 'Overview' }} className="nav-link" activeProps={{ className: 'nav-link nav-link-active' }} onClick={() => setMobileOpen(false)}>
              <Icon size={18} />{label}
            </Link>
          ))}</>}
        </nav>

        <div className="sidebar-foot">
          <Link to="/health" className="live-row"><span className="live-dot" /><span><strong>Runtime monitoring</strong><small>Live health and alerts</small></span></Link>
          <div className="account-switcher" onBlur={(event) => { if (!event.currentTarget.contains(event.relatedTarget)) setAccountMenuOpen(false) }}>
            <button className="user-row" aria-haspopup="menu" aria-expanded={accountMenuOpen} onClick={() => setAccountMenuOpen((open) => !open)}>
              {session.data?.user.image ? <img className="user-avatar user-avatar-image" src={session.data.user.image} alt="" /> : <span className="user-avatar">{session.data?.user.name?.slice(0, 2).toUpperCase() ?? 'LH'}</span>}
              <span><strong>{session.data?.user.name ?? 'Luna'}</strong><small>{isDemoMode ? 'Demo owner' : 'Signed in'}</small></span><ChevronDown size={15} />
            </button>
            {accountMenuOpen && <div className="account-menu" role="menu">
              <div><strong>{session.data?.user.name ?? 'Local demo'}</strong><small>{session.data?.user.email ?? 'No live account connected'}</small></div>
              {!isDemoMode && <button role="menuitem" onClick={() => void authClient.signOut().then(() => window.location.assign('/'))}><LogOut size={16} />Sign out</button>}
            </div>}
          </div>
        </div>
      </aside>

      {mobileOpen && <button className="backdrop" onClick={() => setMobileOpen(false)} aria-label="Close navigation overlay" />}

      <main className="main-area">
        <header className="topbar">
          <button className="mobile-menu" onClick={() => setMobileOpen(true)} aria-label="Open navigation"><Menu size={20} /></button>
          <div><span className="breadcrumb">{inServer ? `${selectedServer.name} · ${serverId}` : 'Orivane bot'}</span><h1>{current}</h1></div>
          <div className="topbar-actions">
            {isDemoMode && <Badge tone="amber">Demo data</Badge>}
            <div className="status-pill"><span />All systems operational</div>
          </div>
        </header>
        <PaletteContext.Provider value={() => setPaletteOpen(true)}>
          <div className="page-wrap">{inServer && <ResourceHealthBanner serverId={serverId} />}<PageErrorBoundary key={pathname}><Outlet /></PageErrorBoundary></div>
        </PaletteContext.Provider>
      </main>

      {paletteOpen && (
        <div className="palette-backdrop" role="presentation" onMouseDown={() => setPaletteOpen(false)}>
          <div className="command-palette" role="dialog" aria-modal="true" aria-label="Command palette" onMouseDown={(event) => event.stopPropagation()}>
            <div className="palette-title"><Command size={16} /><span>Go to</span><button onClick={() => setPaletteOpen(false)}>Esc</button></div>
            {globalNavigation.map(({ to, label, icon: Icon }) => <button key={to} onClick={() => { void navigate({ to }); setPaletteOpen(false) }}><Icon size={17} /><span>{label}</span><small>Bot →</small></button>)}
            {serverNavigation.map(({ to, label, icon: Icon }) => <button key={to} onClick={() => { void navigate({ to, params: { serverId } }); setPaletteOpen(false) }}><Icon size={17} /><span>{label}</span><small>Server →</small></button>)}
          </div>
        </div>
      )}
    </div>
  )
}

function pageTitle(pathname: string) {
  if (pathname === '/') return 'Bot overview'
  if (pathname === '/servers') return 'Servers'
  if (pathname === '/users') return 'Global users'
  if (pathname.startsWith('/users/')) return 'User details'
  if (pathname === '/health') return 'Health'
  if (pathname === '/commands') return 'Commands'
  if (pathname === '/audit') return 'Global audit log'
  if (/^\/audit\/[^/]+$/.test(pathname)) return 'Audit event details'
  if (pathname.endsWith('/audit')) return 'Server audit log'
  if (pathname.includes('/audit/')) return 'Audit event details'
  if (/\/modules\/forms\/[^/]+\/submissions\/[^/]+$/.test(pathname)) return 'Form submission'
  if (pathname.endsWith('/submissions')) return 'Form submissions'
  if (/\/modules\/forms\/[^/]+$/.test(pathname)) return 'Form settings'
  if (pathname.endsWith('/modules/forms')) return 'Forms'
  if (/\/modules\/tickets\/panels\/[^/]+$/.test(pathname)) return 'Ticket panel'
  if (/\/modules\/tickets\/[^/]+$/.test(pathname)) return 'Ticket details'
  if (pathname.endsWith('/modules/tickets')) return 'Tickets'
  if (/\/modules\/suggestions\/[^/]+$/.test(pathname)) return 'Suggestion details'
  if (pathname.endsWith('/modules/suggestions')) return 'Suggestions'
  if (pathname.endsWith('/modules/giveaways/new')) return 'Create giveaway'
  if (/\/modules\/giveaways\/[^/]+$/.test(pathname)) return 'Giveaway details'
  if (pathname.endsWith('/modules/giveaways')) return 'Giveaways'
  if (/\/modules\/reminders\/[^/]+$/.test(pathname)) return 'Reminder details'
  if (pathname.endsWith('/modules/reminders')) return 'Reminders'
  if (/\/modules\/auto_delete\/[^/]+\/[^/]+$/.test(pathname)) return 'Deletion job details'
  if (pathname.endsWith('/modules/auto_delete')) return 'Auto-delete'
  if (/\/modules\/managed_embeds\/[^/]+$/.test(pathname)) return 'Embed editor'
  if (pathname.endsWith('/modules/managed_embeds')) return 'Managed embeds'
  if (/\/moderation\/[^/]+$/.test(pathname)) return 'Moderation case details'
  if (pathname.includes('/members/')) return 'Member details'
  if (pathname.includes('/analytics/')) return 'Analytics report'
  const segment = pathname.split('/').filter(Boolean).at(-1) ?? 'overview'
  return segment === 'servers' || /^\d+$/.test(segment) ? 'Overview' : `${segment.charAt(0).toUpperCase()}${segment.slice(1)}`
}

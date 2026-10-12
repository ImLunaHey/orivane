import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import { RouterProvider } from '@tanstack/react-router'
import axe from 'axe-core'
import { act, cleanup, fireEvent, render, screen, waitFor } from '@testing-library/react'
import { describe, expect, it } from 'vitest'
import { router } from './router'

const representativeRoutes = [
  '/',
  '/servers',
  '/users',
  '/users/312058643210567680',
  '/audit',
  '/audit/1042',
  '/health',
  '/commands',
  '/servers/901234567890123456',
  '/servers/901234567890123456/modules',
  '/servers/901234567890123456/modules/welcome',
  '/servers/901234567890123456/modules/managed_embeds',
  '/servers/901234567890123456/modules/managed_embeds/1',
  '/servers/901234567890123456/moderation',
  '/servers/901234567890123456/moderation/187',
  '/servers/901234567890123456/analytics',
  '/servers/901234567890123456/analytics/commands',
  '/servers/901234567890123456/analytics/latency',
  '/servers/901234567890123456/analytics/automod',
  '/servers/901234567890123456/members',
  '/servers/901234567890123456/members/118742006320914432',
  '/servers/901234567890123456/audit',
  '/servers/901234567890123456/audit/1042',
] as const

describe('dashboard accessibility', () => {
  it('has no detectable accessibility violations on representative routes', async () => {
    const queryClient = new QueryClient({ defaultOptions: { queries: { retry: false } } })
    const { container } = render(<QueryClientProvider client={queryClient}><RouterProvider router={router} /></QueryClientProvider>)

    for (const path of representativeRoutes) {
      await act(async () => { await router.navigate({ to: path as never }) })
      await waitFor(() => expect(router.state.location.pathname).toBe(path))
      const results = await axe.run(container, {
        rules: {
          // jsdom has no layout engine, so contrast is covered by browser QA.
          'color-contrast': { enabled: false },
        },
      })
      expect(results.violations.map(({ id, help }) => `${id}: ${help}`), path).toEqual([])
    }
    cleanup()
  }, 30_000)

  it('opens the server switcher and navigates to the selected server', async () => {
    const queryClient = new QueryClient({ defaultOptions: { queries: { retry: false } } })
    render(<QueryClientProvider client={queryClient}><RouterProvider router={router} /></QueryClientProvider>)
    await act(async () => { await router.navigate({ to: '/servers/$serverId', params: { serverId: '901234567890123456' } }) })
    await waitFor(() => expect(router.state.location.pathname).toBe('/servers/901234567890123456'))

    fireEvent.click(screen.getByRole('button', { name: /Luna's studio/i }))
    expect(screen.getByRole('menu')).toBeInTheDocument()
    fireEvent.click(screen.getByRole('menuitem', { name: /Orbit Labs/i }))

    await waitFor(() => expect(router.state.location.pathname).toBe('/servers/827345678901234567'))
    expect(screen.queryByRole('menu')).not.toBeInTheDocument()
    cleanup()
  })

  it('keeps bot navigation global and links installers to global user details', async () => {
    const queryClient = new QueryClient({ defaultOptions: { queries: { retry: false } } })
    render(<QueryClientProvider client={queryClient}><RouterProvider router={router} /></QueryClientProvider>)
    await act(async () => { await router.navigate({ to: '/servers' }) })
    await waitFor(() => expect(router.state.location.pathname).toBe('/servers'))

    expect(screen.queryByRole('button', { name: /Luna's studio/i })).not.toBeInTheDocument()
    expect(screen.queryByRole('link', { name: 'Modules' })).not.toBeInTheDocument()
    expect(screen.getByRole('link', { name: 'Global users' })).toBeInTheDocument()

    fireEvent.click(screen.getAllByRole('link', { name: 'lunahey' })[0])
    await waitFor(() => expect(router.state.location.pathname).toBe('/users/312058643210567680'))
    expect(screen.getByRole('heading', { name: 'Luna' })).toBeInTheDocument()
    cleanup()
  })

  it('filters and inspects the global audit trail', async () => {
    const queryClient = new QueryClient({ defaultOptions: { queries: { retry: false } } })
    render(<QueryClientProvider client={queryClient}><RouterProvider router={router} /></QueryClientProvider>)
    await act(async () => { await router.navigate({ to: '/audit' }) })
    await waitFor(() => expect(screen.getByRole('table', { name: 'Global audit events' })).toBeInTheDocument())

    expect(screen.getByRole('link', { name: 'Audit log' })).toBeInTheDocument()
    expect(screen.getAllByText(/Guild · Config · Updated/i).length).toBeGreaterThan(0)
    fireEvent.change(screen.getByLabelText('Filter by outcome'), { target: { value: 'denied' } })
    expect(screen.queryByText('Actor lacks Ban Members permission')).not.toBeInTheDocument()
    fireEvent.click(screen.getByRole('link', { name: 'Inspect →' }))
    await waitFor(() => expect(router.state.location.pathname).toBe('/audit/1039'))
    await waitFor(() => expect(screen.getByRole('heading', { name: /Slash Command · Ban/i })).toBeInTheDocument())
    expect(screen.getByText('Actor lacks Ban Members permission')).toBeVisible()
    fireEvent.click(screen.getByRole('link', { name: 'Back to global audit log' }))
    await waitFor(() => expect(router.state.location.pathname).toBe('/audit'))
    cleanup()
  })
})

import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import { RouterProvider } from '@tanstack/react-router'
import { act, cleanup, fireEvent, render, screen, waitFor } from '@testing-library/react'
import { afterEach, beforeEach, describe, expect, it } from 'vitest'
import { router } from '../router'

const serverId = '901234567890123456'

function renderDashboard() {
  const queryClient = new QueryClient({ defaultOptions: { queries: { retry: false }, mutations: { retry: false } } })
  return render(<QueryClientProvider client={queryClient}><RouterProvider router={router} /></QueryClientProvider>)
}

async function navigateTo(path: string) {
  await act(async () => { await router.navigate({ to: path as never }) })
  await waitFor(() => expect(router.state.location.pathname).toBe(path))
}

describe('module configuration pages', () => {
  beforeEach(() => localStorage.clear())
  afterEach(() => cleanup())

  it('lists every module, toggles one, and persists the changed configuration', async () => {
    renderDashboard()
    await navigateTo(`/servers/${serverId}/modules`)
    await screen.findAllByText('View details →')

    expect(screen.getAllByText('Live')).toHaveLength(9)
    expect(screen.getAllByText('Configuration only')).toHaveLength(1)
    fireEvent.click(screen.getByRole('switch', { name: 'Enable Autorole' }))
    const save = screen.getByRole('button', { name: 'Save changes' })
    expect(save).toBeEnabled()
    fireEvent.click(save)
    await screen.findByText('Saved')
    expect(JSON.parse(localStorage.getItem(`orivane:config:${serverId}`) ?? '{}').modules.autorole.enabled).toBe(true)
  })

  it('renders and edits settings for every module type', async () => {
    renderDashboard()
    const cases = [
      ['utility', 'Utility settings', 'Ephemeral responses', 'switch', true],
      ['welcome', 'Welcome settings', 'Welcome channel', null, null],
      ['autorole', 'Autorole settings', 'Roles to assign', null, null],
      ['logging', 'Server logging settings', 'Logging channel', null, null],
      ['moderation', 'Moderation settings', 'Default reason', 'text', 'Be kind'],
      ['automod', 'Automod settings', 'Maximum mentions', 'number', '8'],
      ['leveling', 'Leveling settings', 'Minimum XP', 'number', '20'],
      ['starboard', 'Starboard settings', 'Reaction threshold', 'number', '5'],
      ['tickets', 'Tickets settings', 'Support roles', null, null],
      ['custom_commands', 'Custom commands settings', 'Command limit', 'number', '30'],
    ] as const

    for (const [moduleId, heading, control, kind, value] of cases) {
      await navigateTo(`/servers/${serverId}/modules/${moduleId}`)
      await screen.findByRole('heading', { name: heading })
      const input = screen.getByLabelText(new RegExp(`^${control}`))
      if (kind === 'switch') fireEvent.click(input)
      if (kind === 'text' || kind === 'number') fireEvent.change(input, { target: { value } })
      if (kind) expect(screen.getByRole('button', { name: 'Save changes' })).toBeEnabled()
      if (moduleId === 'starboard') {
        expect(screen.getByText('Not active in Discord yet')).toBeVisible()
      }
    }
  })

  it('handles unknown module URLs without crashing', async () => {
    renderDashboard()
    await navigateTo(`/servers/${serverId}/modules/not-real`)
    expect(await screen.findByRole('heading', { name: 'Module not found' })).toBeVisible()
    expect(screen.getByRole('link', { name: 'Return to modules' })).toBeVisible()
  })
})

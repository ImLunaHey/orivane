import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import { RouterProvider } from '@tanstack/react-router'
import { act, cleanup, fireEvent, render, screen, waitFor, within } from '@testing-library/react'
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

    expect(screen.getAllByText('Live')).toHaveLength(15)
    expect(screen.queryByText('Configuration only')).not.toBeInTheDocument()
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
      ['reaction_roles', 'Reaction roles settings', 'Enable Reaction roles', 'switch', true],
      ['autoresponder', 'Autoresponder settings', 'Enable Autoresponder', 'switch', true],
      ['anti_raid', 'Anti-raid settings', 'Join threshold', 'number', '20'],
      ['scheduled_messages', 'Scheduled messages settings', 'Enable Scheduled messages', 'switch', true],
    ] as const

    for (const [moduleId, heading, control, kind, value] of cases) {
      await navigateTo(`/servers/${serverId}/modules/${moduleId}`)
      await screen.findByRole('heading', { name: heading })
      const input = screen.getByLabelText(new RegExp(`^${control}`))
      if (kind === 'switch') fireEvent.click(input)
      if (kind === 'text' || kind === 'number') fireEvent.change(input, { target: { value } })
      if (kind) expect(screen.getByRole('button', { name: 'Save changes' })).toBeEnabled()
      expect(screen.queryByText('Not active in Discord yet')).not.toBeInTheDocument()
    }
  })

  it('handles unknown module URLs without crashing', async () => {
    renderDashboard()
    await navigateTo(`/servers/${serverId}/modules/not-real`)
    expect(await screen.findByRole('heading', { name: 'Module not found' })).toBeVisible()
    expect(screen.getByRole('link', { name: 'Return to modules' })).toBeVisible()
  })

  it('creates and edits a custom slash response', async () => {
    renderDashboard()
    await navigateTo(`/servers/${serverId}/modules/custom_commands`)
    await screen.findByRole('heading', { name: 'Custom commands settings' })
    fireEvent.click(screen.getByRole('button', { name: 'Add command' }))
    fireEvent.change(screen.getByLabelText(/^Command name/), { target: { value: 'Greeting!' } })
    fireEvent.change(screen.getByLabelText(/^Description/), { target: { value: 'Greets a member' } })
    fireEvent.change(screen.getByLabelText(/^Response/), { target: { value: 'Hello {user}' } })
    fireEvent.click(screen.getByRole('switch', { name: 'Ephemeral response' }))
    fireEvent.click(screen.getByRole('button', { name: 'Save changes' }))
    await screen.findByText('Saved')
    const saved = JSON.parse(localStorage.getItem(`orivane:config:${serverId}`) ?? '{}')
    expect(saved.modules.custom_commands.commands[0]).toMatchObject({ name: 'greeting', description: 'Greets a member', response: 'Hello {user}', ephemeral: true })
    fireEvent.click(screen.getByRole('button', { name: 'Remove' }))
    expect(screen.getByText('No custom commands yet')).toBeVisible()
  })

  it('reveals optional welcome and goodbye editors', async () => {
    renderDashboard()
    await navigateTo(`/servers/${serverId}/modules/welcome`)
    await screen.findByRole('heading', { name: 'Welcome settings' })
    fireEvent.click(screen.getByRole('switch', { name: 'Direct-message new members' }))
    fireEvent.click(screen.getByRole('switch', { name: 'Goodbye messages' }))
    expect(screen.getByText('Direct welcome message')).toBeVisible()
    expect(screen.getByText('Goodbye message')).toBeVisible()
    expect(screen.getByRole('combobox', { name: 'Goodbye channel' })).toBeVisible()
  })

  it('configures delayed and exclusive autoroles', async () => {
    renderDashboard()
    await navigateTo(`/servers/${serverId}/modules/autorole`)
    await screen.findByRole('heading', { name: 'Autorole settings' })
    fireEvent.change(screen.getByLabelText(/^Assignment delay/), { target: { value: '30' } })
    fireEvent.change(screen.getByLabelText(/^Removal delay/), { target: { value: '3600' } })
    fireEvent.click(screen.getByRole('switch', { name: 'Only one self-assignable role' }))
    expect(screen.getByRole('button', { name: 'Save changes' })).toBeEnabled()
  })

  it('creates and removes a reaction-role mapping', async () => {
    renderDashboard()
    await navigateTo(`/servers/${serverId}/modules/reaction_roles`)
    await screen.findByRole('heading', { name: 'Reaction roles settings' })
    fireEvent.click(screen.getByRole('button', { name: 'Add mapping' }))
    fireEvent.change(screen.getByLabelText(/^Message ID/), { target: { value: 'message 123456789012345678' } })
    fireEvent.change(screen.getByLabelText(/^Emoji/), { target: { value: '🌙' } })
    expect(screen.getByLabelText(/^Message ID/)).toHaveValue('123456789012345678')
    fireEvent.click(screen.getByRole('button', { name: 'Remove' }))
    expect(screen.getByText('No reaction mappings yet')).toBeVisible()
  })

  it('creates and edits an automatic response', async () => {
    renderDashboard()
    await navigateTo(`/servers/${serverId}/modules/autoresponder`)
    await screen.findByRole('heading', { name: 'Autoresponder settings' })
    fireEvent.click(screen.getByRole('button', { name: 'Add response' }))
    fireEvent.change(screen.getByLabelText(/^Trigger/), { target: { value: 'hello' } })
    fireEvent.change(screen.getByLabelText(/^Match mode/), { target: { value: 'exact' } })
    fireEvent.change(screen.getByLabelText(/^Response/), { target: { value: 'Hello {user}' } })
    fireEvent.click(screen.getByRole('switch', { name: 'Case sensitive' }))
    expect(screen.getByRole('button', { name: 'Save changes' })).toBeEnabled()
    fireEvent.click(screen.getByRole('button', { name: 'Remove' }))
    expect(screen.getByText('No automatic responses yet')).toBeVisible()
  })

  it('configures anti-raid containment and timeout policy', async () => {
    renderDashboard()
    await navigateTo(`/servers/${serverId}/modules/anti_raid`)
    await screen.findByRole('heading', { name: 'Anti-raid settings' })
    fireEvent.change(screen.getByLabelText(/^Detection window/), { target: { value: '120' } })
    fireEvent.change(screen.getByLabelText(/^Minimum account age/), { target: { value: '14' } })
    fireEvent.click(screen.getByRole('switch', { name: 'Require profile avatar' }))
    fireEvent.click(screen.getByRole('switch', { name: 'Contain every raid join' }))
    fireEvent.change(screen.getByLabelText(/^Raid action/), { target: { value: 'timeout' } })
    fireEvent.change(screen.getByLabelText(/^Timeout duration/), { target: { value: '7200' } })
    expect(screen.getByRole('button', { name: 'Save changes' })).toBeEnabled()
  })

  it('creates and edits a recurring scheduled message', async () => {
    renderDashboard()
    await navigateTo(`/servers/${serverId}/modules/scheduled_messages`)
    await screen.findByRole('heading', { name: 'Scheduled messages settings' })
    fireEvent.click(screen.getByRole('button', { name: 'Add schedule' }))
    fireEvent.change(screen.getByLabelText(/^Schedule ID/), { target: { value: 'daily update!' } })
    fireEvent.change(screen.getByLabelText(/^Repeat interval/), { target: { value: '60' } })
    fireEvent.change(screen.getByLabelText(/^Message/), { target: { value: 'Hourly update' } })
    fireEvent.click(screen.getByRole('switch', { name: 'End automatically' }))
    expect(screen.getByLabelText(/^Schedule ID/)).toHaveValue('dailyupdate')
    expect(screen.getByLabelText(/^Final delivery cutoff/)).toBeVisible()
    fireEvent.click(screen.getByRole('button', { name: 'Remove' }))
    expect(screen.getByText('No schedules yet')).toBeVisible()
  })

  it('lists forms and keeps the module save action after the directory', async () => {
    renderDashboard()
    await navigateTo(`/servers/${serverId}/modules/forms`)
    await screen.findByRole('heading', { name: 'Forms' })
    expect(await screen.findByRole('heading', { name: 'Current forms' })).toBeVisible()
    expect(screen.getByRole('link', { name: 'Staff application' })).toBeVisible()
    expect(screen.getByRole('link', { name: 'New form' })).toBeVisible()
    fireEvent.click(screen.getByRole('switch', { name: 'Enable Forms' }))
    const save = screen.getByRole('button', { name: 'Save changes' })
    expect(save).toBeEnabled()
    expect(save.compareDocumentPosition(screen.getByRole('heading', { name: 'Current forms' })) & Node.DOCUMENT_POSITION_PRECEDING).toBeTruthy()
  })

  it('creates a native modal form on a dedicated details page', async () => {
    renderDashboard()
    await navigateTo(`/servers/${serverId}/modules/forms/new`)
    await screen.findByRole('heading', { name: 'Form 2' })
    fireEvent.change(screen.getByLabelText(/^Form ID/), { target: { value: 'staff application!' } })
    fireEvent.change(screen.getByLabelText(/^Form name/), { target: { value: 'Volunteer application' } })
    fireEvent.change(screen.getByLabelText(/^Label/), { target: { value: 'Why should we choose you?' } })
    fireEvent.click(screen.getByRole('button', { name: 'Add question' }))
    expect(screen.getByLabelText(/^Form ID/)).toHaveValue('staffapplication')
    expect(screen.getAllByRole('button', { name: 'Remove' })).toHaveLength(2)
    fireEvent.click(screen.getAllByRole('button', { name: 'Remove' })[1])
    expect(screen.getByRole('button', { name: 'Save changes' })).toBeEnabled()
  })

  it('shows form submissions and opens each response on a details page', async () => {
    renderDashboard()
    await navigateTo(`/servers/${serverId}/modules/forms/staff-application/submissions`)
    await screen.findByRole('heading', { name: 'Staff application' })
    expect(screen.getByRole('heading', { name: 'Submissions' })).toBeVisible()
    expect(screen.getByRole('link', { name: 'Inspect' })).toBeVisible()
    await navigateTo(`/servers/${serverId}/modules/forms/staff-application/submissions/1`)
    await screen.findByRole('heading', { name: 'Submitted answers' })
    expect(screen.getByText('I have moderated community servers for several years.')).toBeVisible()
    expect(screen.getByRole('link', { name: 'View submitter' })).toBeVisible()
  })

  it('operates the responsive navigation, server menu, account menu, and command palette', async () => {
    renderDashboard()
    await navigateTo(`/servers/${serverId}/modules`)
    fireEvent.click(screen.getByRole('button', { name: 'Open navigation' }))
    expect(screen.getByRole('button', { name: 'Close navigation overlay' })).toBeVisible()
    fireEvent.click(screen.getByRole('button', { name: 'Close navigation overlay' }))

    const guildPicker = screen.getByRole('button', { name: /Luna's studio/ })
    fireEvent.click(guildPicker)
    expect(screen.getByRole('menu')).toBeVisible()
    fireEvent.keyDown(window, { key: 'Escape' })
    expect(screen.queryByText('Switch server')).not.toBeInTheDocument()

    fireEvent.click(screen.getByRole('button', { name: /Luna.*Demo owner/ }))
    expect(screen.getByText('No live account connected')).toBeVisible()
    fireEvent.keyDown(window, { key: 'Escape' })

    fireEvent.keyDown(window, { key: 'k', ctrlKey: true })
    const palette = screen.getByRole('dialog', { name: 'Command palette' })
    fireEvent.click(within(palette).getByRole('button', { name: /Servers/ }))
    await waitFor(() => expect(router.state.location.pathname).toBe('/servers'))
  })
})

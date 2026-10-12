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

    expect(screen.getAllByText('Live')).toHaveLength(29)
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
      ['moderation', 'Moderation settings', 'Default reason', 'text', 'Be kind'],
      ['automod', 'Automod settings', 'Enable Automod', 'switch', true],
      ['leveling', 'Leveling settings', 'Minimum XP', 'number', '20'],
      ['starboard', 'Starboard settings', 'Reaction threshold', 'number', '5'],
      ['custom_commands', 'Custom commands settings', 'Command limit', 'number', '30'],
      ['autoresponder', 'Autoresponder settings', 'Enable Autoresponder', 'switch', true],
      ['anti_raid', 'Anti-raid settings', 'Join threshold', 'number', '20'],
      ['scheduled_messages', 'Scheduled messages settings', 'Enable Scheduled messages', 'switch', true],
      ['verification', 'Verification settings', 'Minimum account age', 'number', '7'],
      ['moderation_appeals', 'Moderation appeals settings', 'Submission cooldown', 'number', '48'],
      ['invite_tracking', 'Invite tracking settings', 'Enable Invite tracking', 'switch', true],
      ['polls', 'Polls settings', 'Default duration', 'number', '60'],
      ['counters', 'Counters and goals settings', 'Enable Counters and goals', 'switch', true],
      ['afk', 'AFK settings', 'Maximum AFK duration', 'number', '72'],
      ['voice_text_linking', 'Voice and text linking settings', 'Enable Voice and text linking', 'switch', true],
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

  it('configures the new community workflow editors', async () => {
    renderDashboard()

    await navigateTo(`/servers/${serverId}/modules/moderation_appeals`)
    fireEvent.click(await screen.findByRole('combobox', { name: 'Review channel' }))
    fireEvent.click(screen.getByRole('option', { name: 'welcome' }))
    fireEvent.change(screen.getByLabelText(/^Appeal prompt/), { target: { value: 'Tell the team what happened.' } })

    await navigateTo(`/servers/${serverId}/modules/invite_tracking`)
    fireEvent.click(await screen.findByRole('combobox', { name: 'Join attribution log' }))
    fireEvent.click(screen.getByRole('option', { name: 'welcome' }))
    fireEvent.click(screen.getByRole('button', { name: 'Add reward' }))
    fireEvent.change(screen.getByLabelText(/^Attributed invites/), { target: { value: '5' } })
    fireEvent.click(screen.getByRole('button', { name: 'Reward role' }))
    fireEvent.click(screen.getByRole('option', { name: 'Member' }))

    await navigateTo(`/servers/${serverId}/modules/polls`)
    fireEvent.change(await screen.findByLabelText(/^Maximum options/), { target: { value: '4' } })
    fireEvent.click(screen.getByRole('switch', { name: 'Allow anonymous polls' }))
    fireEvent.click(screen.getByRole('button', { name: 'Poll manager roles' }))
    fireEvent.click(screen.getByRole('option', { name: 'Support' }))

    await navigateTo(`/servers/${serverId}/modules/counters`)
    fireEvent.click(await screen.findByRole('button', { name: 'Add counter' }))
    fireEvent.click(screen.getByRole('combobox', { name: 'Counter channel' }))
    fireEvent.click(screen.getByRole('option', { name: 'Lounge' }))
    fireEvent.change(screen.getByLabelText(/^Metric/), { target: { value: 'goal' } })
    fireEvent.change(screen.getByLabelText(/^Current progress/), { target: { value: '25' } })
    fireEvent.change(screen.getByLabelText(/^Goal/), { target: { value: '100' } })
    fireEvent.change(screen.getByLabelText(/^Channel name template/), { target: { value: 'Goal: {count}/{goal}' } })
    fireEvent.click(screen.getByRole('button', { name: 'Remove' }))
    expect(screen.getByText('No counters configured')).toBeVisible()

    await navigateTo(`/servers/${serverId}/modules/afk`)
    fireEvent.click(await screen.findByRole('switch', { name: 'Clear on message' }))
    fireEvent.click(screen.getByRole('switch', { name: 'Mention notifications' }))

    await navigateTo(`/servers/${serverId}/modules/voice_text_linking`)
    fireEvent.click(await screen.findByRole('button', { name: 'Add link' }))
    fireEvent.click(screen.getByRole('combobox', { name: 'Voice channel' }))
    fireEvent.click(screen.getByRole('option', { name: 'Lounge' }))
    fireEvent.click(screen.getByRole('combobox', { name: 'Linked text channel' }))
    fireEvent.click(screen.getByRole('option', { name: 'welcome' }))
    fireEvent.click(screen.getByRole('switch', { name: 'Announce leaves' }))
    fireEvent.click(screen.getByRole('button', { name: 'Remove' }))
    expect(screen.getByText('No linked rooms')).toBeVisible()
  })

  it('keeps moderation appeal submissions separate and reviews them on a detail page', async () => {
    renderDashboard()
    await navigateTo(`/servers/${serverId}/modules/moderation_appeals`)
    fireEvent.click(await screen.findByRole('link', { name: 'View submissions' }))
    await waitFor(() => expect(router.state.location.pathname).toBe(`/servers/${serverId}/modules/forms/moderation-appeals/submissions`))
    expect(await screen.findByRole('table', { name: 'Submissions for Moderation appeals' })).toBeVisible()
    fireEvent.click(await screen.findByRole('link', { name: 'Inspect' }))
    expect(await screen.findByRole('heading', { name: 'Review appeal' })).toBeVisible()
    fireEvent.change(screen.getByLabelText(/^Decision note/), { target: { value: 'Reviewed by the moderation team.' } })
    fireEvent.click(screen.getByRole('button', { name: 'Approve' }))
    await screen.findByText('approved')
    expect(screen.getAllByText('Reviewed by the moderation team.')).toHaveLength(2)
  })

  it('configures category-based Discord event logging on its dedicated page', async () => {
    renderDashboard()
    await navigateTo(`/servers/${serverId}/modules/logging`)
    await screen.findByRole('heading', { name: 'Discord event logging' })

    expect(screen.getByRole('combobox', { name: 'Default logging channel' })).toBeVisible()
    expect(screen.getByRole('switch', { name: 'Log messages' })).toBeChecked()
    expect(screen.getByRole('switch', { name: 'Log moderation' })).toBeChecked()
    expect(screen.getByRole('table', { name: 'Recent Discord logging events' })).toBeVisible()

    fireEvent.click(screen.getByRole('switch', { name: 'Log voice' }))
    const save = screen.getByRole('button', { name: 'Save changes' })
    expect(save).toBeEnabled()
    fireEvent.click(save)
    await screen.findByText('Saved')
    expect(JSON.parse(localStorage.getItem(`orivane:config:${serverId}`) ?? '{}').modules.logging.voice_events).toBe(false)
  })

  it('configures moderator roles and warning escalation thresholds', async () => {
    renderDashboard()
    await navigateTo(`/servers/${serverId}/modules/moderation`)
    await screen.findByRole('heading', { name: 'Moderation settings' })

    fireEvent.click(screen.getByRole('button', { name: 'Moderator roles' }))
    fireEvent.click(screen.getByRole('option', { name: 'Support' }))
    expect(screen.getByRole('button', { name: 'Remove Support' })).toBeVisible()

    fireEvent.click(screen.getByRole('button', { name: 'Add threshold' }))
    fireEvent.change(screen.getByLabelText(/^Warning count/), { target: { value: '3' } })
    fireEvent.change(screen.getByLabelText(/^Escalation action/), { target: { value: 'ban' } })
    expect(screen.queryByLabelText(/^Timeout length/)).not.toBeInTheDocument()

    fireEvent.click(screen.getByRole('button', { name: 'Save changes' }))
    await screen.findByText('Saved')
    const moderation = JSON.parse(localStorage.getItem(`orivane:config:${serverId}`) ?? '{}').modules.moderation
    expect(moderation.moderator_role_ids).toEqual(['100000000000000101'])
    expect(moderation.warning_thresholds).toEqual([{ warnings: 3, action: 'ban', timeout_minutes: 60 }])
  })

  it('creates an advanced automod rule with an independent action', async () => {
    renderDashboard()
    await navigateTo(`/servers/${serverId}/modules/automod`)
    await screen.findByRole('heading', { name: 'Automod settings' })

    fireEvent.click(screen.getByRole('button', { name: 'Add rule' }))
    fireEvent.change(screen.getByLabelText(/^Rule ID/), { target: { value: 'Rapid Spam!' } })
    fireEvent.change(screen.getByLabelText(/^Filter/), { target: { value: 'rapid_spam' } })
    fireEvent.change(screen.getByLabelText(/^Action/), { target: { value: 'timeout' } })
    fireEvent.change(screen.getByLabelText(/^Message count/), { target: { value: '6' } })
    fireEvent.change(screen.getByLabelText(/^Detection window/), { target: { value: '12' } })
    fireEvent.click(screen.getByRole('button', { name: 'Save changes' }))
    await screen.findByText('Saved')

    const rule = JSON.parse(localStorage.getItem(`orivane:config:${serverId}`) ?? '{}').modules.automod.rules[0]
    expect(rule).toMatchObject({ id: 'rapidspam', kind: 'rapid_spam', action: 'timeout', threshold: 6, window_seconds: 12 })
  })

  it('configures leveling anti-farm, role rewards, voice XP, and rank cards', async () => {
    renderDashboard()
    await navigateTo(`/servers/${serverId}/modules/leveling`)
    await screen.findByRole('heading', { name: 'Leveling settings' })
    fireEvent.change(screen.getByLabelText(/^Minimum message length/), { target: { value: '12' } })
    fireEvent.click(screen.getByRole('button', { name: 'Add reward' }))
    fireEvent.change(screen.getByLabelText(/^Required level/), { target: { value: '5' } })
    fireEvent.click(screen.getByRole('button', { name: 'Reward role' }))
    fireEvent.click(screen.getByRole('option', { name: 'Member' }))
    fireEvent.click(screen.getByRole('switch', { name: 'Voice XP' }))
    fireEvent.change(screen.getByLabelText(/^Voice XP per minute/), { target: { value: '4' } })
    fireEvent.click(screen.getByRole('button', { name: 'Save changes' }))
    await screen.findByText('Saved')
    const leveling = JSON.parse(localStorage.getItem(`orivane:config:${serverId}`) ?? '{}').modules.leveling
    expect(leveling).toMatchObject({ minimum_message_length: 12, voice_xp_enabled: true, voice_xp_per_minute: 4 })
    expect(leveling.role_rewards).toEqual([{ level: 5, role_id: '100000000000000103' }])
  })

  it('configures temporary voice and manages active channels', async () => {
    renderDashboard()
    await navigateTo(`/servers/${serverId}/modules/temporary_voice`)
    expect(await screen.findByRole('heading', { name: /Temporary voice/ })).toBeVisible()
    fireEvent.click(screen.getByRole('switch', { name: 'Enable Temporary voice' }))
    fireEvent.click(screen.getByRole('combobox', { name: 'Creation lobby' }))
    fireEvent.click(screen.getByRole('option', { name: 'Lounge' }))
    fireEvent.click(screen.getByRole('combobox', { name: 'Channel category' }))
    fireEvent.click(screen.getByRole('option', { name: 'Community' }))
    fireEvent.click(screen.getByRole('button', { name: 'Save changes' }))
    await screen.findByText('1 active')
    expect(screen.getByRole('table', { name: 'Active temporary voice channels' })).toBeVisible()
    fireEvent.click(screen.getByRole('button', { name: 'Delete' }))
    await waitFor(() => expect(screen.getByText('0 active')).toBeVisible())
  })

  it('uses dedicated ticket inbox, panel editor, and ticket detail pages', async () => {
    renderDashboard()
    await navigateTo(`/servers/${serverId}/modules/tickets`)
    expect(await screen.findByRole('heading', { name: 'Ticket panels' })).toBeVisible()
    expect(screen.getByRole('table', { name: 'Support tickets' })).toBeVisible()
    expect(screen.getByLabelText('Tickets grouped by status')).toBeVisible()
    fireEvent.click(screen.getByRole('link', { name: 'New panel' }))
    expect(await screen.findByRole('heading', { name: 'New ticket panel' })).toBeVisible()
    expect(screen.getByRole('button', { name: 'Save panel' })).toBeDisabled()
    await navigateTo(`/servers/${serverId}/modules/tickets/42`)
    expect(await screen.findByRole('heading', { name: 'Verification help' })).toBeVisible()
    expect(screen.getByRole('heading', { name: 'Ticket lifecycle' })).toBeVisible()
    expect(screen.getByText('Silver')).toBeVisible()
    fireEvent.click(screen.getByRole('button', { name: 'Claim' }))
    await waitFor(() => expect(screen.queryByText('Unassigned')).not.toBeInTheDocument())
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

  it('opens the managed reaction-role panel editor', async () => {
    renderDashboard()
    await navigateTo(`/servers/${serverId}/modules/reaction_roles`)
    await screen.findByRole('heading', { name: 'Reaction roles' })
    expect(screen.getByRole('link', { name: 'New panel' })).toBeVisible()
    fireEvent.click(screen.getByRole('switch', { name: 'Enable Reaction roles' }))
    fireEvent.click(screen.getByRole('button', { name: 'Save changes' }))
    await navigateTo(`/servers/${serverId}/modules/reaction_roles/new`)
    await screen.findByRole('heading', { name: 'New reaction-role panel' })
    fireEvent.change(screen.getByLabelText(/^Internal name/), { target: { value: 'Color roles' } })
    fireEvent.change(screen.getByLabelText(/^Stable panel ID/), { target: { value: 'Color Roles!' } })
    fireEvent.click(screen.getByRole('combobox', { name: 'Destination channel' }))
    fireEvent.click(screen.getByRole('option', { name: 'welcome' }))
    fireEvent.change(screen.getByLabelText(/^Message title/), { target: { value: 'Pick a color' } })
    fireEvent.change(screen.getAllByLabelText(/^Description/)[0], { target: { value: 'Choose one color role.' } })
    fireEvent.change(screen.getByLabelText(/^Interaction control/), { target: { value: 'select' } })
    fireEvent.change(screen.getByLabelText(/^Assignment mode/), { target: { value: 'add_only' } })
    fireEvent.click(screen.getByRole('switch', { name: 'Exclusive group' }))
    fireEvent.click(screen.getByRole('switch', { name: 'Panel enabled' }))
    fireEvent.click(screen.getByRole('switch', { name: 'Panel enabled' }))
    fireEvent.change(screen.getByLabelText(/^Option ID/), { target: { value: 'Red Role!' } })
    fireEvent.change(screen.getByLabelText(/^Label/), { target: { value: 'Red' } })
    fireEvent.change(screen.getByLabelText(/^Emoji/), { target: { value: '🔴' } })
    fireEvent.change(screen.getAllByLabelText(/^Description/)[1], { target: { value: 'The red team' } })
    fireEvent.click(screen.getByRole('button', { name: 'Discord role' }))
    fireEvent.click(screen.getByRole('option', { name: 'Support' }))
    fireEvent.click(screen.getByRole('button', { name: 'Add role option' }))
    expect(screen.getByText('Option 2')).toBeVisible()
    fireEvent.click(screen.getAllByRole('button', { name: 'Remove' })[1])
    expect(screen.queryByText('Option 2')).not.toBeInTheDocument()
    fireEvent.click(screen.getByRole('button', { name: 'Save panel' }))
    await screen.findByRole('button', { name: 'Deploy message' })
    fireEvent.click(screen.getByRole('button', { name: 'Deploy message' }))
    await screen.findByRole('button', { name: 'Sync message' })
    fireEvent.click(screen.getByRole('button', { name: 'Delete' }))
    await screen.findByRole('heading', { name: 'Reaction roles' })
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
    fireEvent.change(screen.getByLabelText(/^Schedule type/), { target: { value: 'weekdays' } })
    fireEvent.click(screen.getByRole('button', { name: 'Mon' }))
    fireEvent.click(screen.getByRole('button', { name: 'Fri' }))
    expect(screen.getByRole('button', { name: 'Mon' })).toHaveAttribute('aria-pressed', 'true')
    expect(screen.getByRole('button', { name: 'Fri' })).toHaveAttribute('aria-pressed', 'true')
    expect(screen.queryByLabelText(/^Repeat interval/)).not.toBeInTheDocument()
    fireEvent.change(screen.getByLabelText(/^Schedule type/), { target: { value: 'once' } })
    expect(screen.getByLabelText(/^Delivery time/)).toBeVisible()
    expect(screen.queryByRole('switch', { name: 'End automatically' })).not.toBeInTheDocument()
    fireEvent.change(screen.getByLabelText(/^Content source/), { target: { value: '1' } })
    expect(screen.queryByLabelText(/^Message/)).not.toBeInTheDocument()
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
    expect(screen.getByRole('link', { name: 'Edit' })).toHaveAttribute('href', `/servers/${serverId}/modules/forms/staff-application`)
    expect(screen.getByRole('link', { name: 'Submissions' })).toHaveAttribute('href', `/servers/${serverId}/modules/forms/staff-application/submissions`)
    fireEvent.change(screen.getByLabelText(/^Retention period/), { target: { value: '180' } })
    fireEvent.click(screen.getByRole('switch', { name: 'Enable Forms' }))
    const save = screen.getByRole('button', { name: 'Save changes' })
    expect(save).toBeEnabled()
    expect(save.compareDocumentPosition(screen.getByRole('heading', { name: 'Current forms' })) & Node.DOCUMENT_POSITION_PRECEDING).toBeTruthy()
  })

  it('reviews ordinary form submissions with the shared workflow', async () => {
    renderDashboard()
    await navigateTo(`/servers/${serverId}/modules/forms/staff-application/submissions/1`)
    expect(await screen.findByRole('heading', { name: 'Review submission' })).toBeVisible()
    fireEvent.change(screen.getByLabelText(/^Decision note/), { target: { value: 'Strong application.' } })
    fireEvent.click(screen.getByRole('button', { name: 'Approve' }))
    await screen.findByText('approved')
    expect(screen.getAllByText('Strong application.')).toHaveLength(2)
  })

  it('creates a native modal form on a dedicated details page', async () => {
    renderDashboard()
    await navigateTo(`/servers/${serverId}/modules/forms/new`)
    await screen.findByRole('heading', { name: 'Form 2' })
    expect(screen.queryByRole('navigation', { name: 'Form sections' })).not.toBeInTheDocument()
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
    expect(screen.getByRole('link', { name: /Silver.*@silver\.arc/ })).toHaveAttribute('href', `/servers/${serverId}/members/118742006320914432`)
  })

  it('configures suggestions and manages a proposal on its own details page', async () => {
    renderDashboard()
    await navigateTo(`/servers/${serverId}/modules/suggestions`)
    await screen.findByRole('heading', { name: 'Suggestions' })
    expect(await screen.findByRole('heading', { name: 'Current suggestions' })).toBeVisible()
    expect(screen.getByLabelText('Number of suggestions grouped by workflow status')).toBeVisible()
    expect(screen.getAllByRole('link', { name: 'Inspect' })).toHaveLength(3)
    fireEvent.change(screen.getByLabelText(/^Submission cooldown/), { target: { value: '120' } })
    fireEvent.click(screen.getByRole('switch', { name: 'Enable Suggestions' }))
    fireEvent.click(screen.getByRole('button', { name: 'Save changes' }))
    await screen.findByText('Saved')

    await navigateTo(`/servers/${serverId}/modules/suggestions/3`)
    await screen.findByRole('heading', { name: 'Review suggestion' })
    expect(screen.getByText('Add a monthly community showcase for member projects.')).toBeVisible()
    expect(screen.getByRole('link', { name: /Silver.*@silver\.arc/ })).toHaveAttribute('href', `/servers/${serverId}/members/118742006320914432`)
    fireEvent.change(screen.getByLabelText(/^Status/), { target: { value: 'approved' } })
    fireEvent.change(screen.getByLabelText(/^Staff note/), { target: { value: 'Added to the roadmap.' } })
    fireEvent.click(screen.getByRole('button', { name: 'Save decision' }))
    await screen.findByText('Saved')
    expect(JSON.parse(localStorage.getItem(`orivane:suggestions:${serverId}`) ?? '[]')[0]).toMatchObject({ status: 'approved', staff_note: 'Added to the roadmap.' })
  })

  it('creates, inspects, and ends a giveaway through dedicated pages', async () => {
    renderDashboard()
    await navigateTo(`/servers/${serverId}/modules/giveaways`)
    expect(await screen.findByRole('heading', { name: 'Current giveaways' })).toBeVisible()
    expect(screen.getByLabelText('Number of giveaways grouped by lifecycle status')).toBeVisible()
    expect(screen.getAllByRole('link', { name: 'Inspect' })).toHaveLength(2)
    fireEvent.click(screen.getByRole('switch', { name: 'Enable Giveaways' }))
    fireEvent.click(screen.getByRole('button', { name: 'Save changes' }))
    await screen.findByText('Saved')

    await navigateTo(`/servers/${serverId}/modules/giveaways/new`)
    await screen.findByRole('heading', { name: 'Create giveaway', level: 2 })
    fireEvent.change(screen.getByLabelText(/^Title/), { target: { value: 'Launch celebration' } })
    fireEvent.click(screen.getByRole('button', { name: 'Create giveaway' }))
    await screen.findByRole('heading', { name: 'Launch celebration' })
    expect(screen.getByRole('heading', { name: 'Entrants' })).toBeVisible()
    expect(screen.getByRole('link', { name: /Silver.*@silver\.arc/ })).toBeVisible()
    fireEvent.click(screen.getByRole('button', { name: 'End and draw now' }))
    expect(await screen.findByRole('heading', { name: 'Winners' })).toBeVisible()
    expect(screen.getByRole('button', { name: 'Reroll winners' })).toBeVisible()
  })

  it('configures reminders and cancels one from its details page', async () => {
    renderDashboard()
    await navigateTo(`/servers/${serverId}/modules/reminders`)
    expect(await screen.findByRole('heading', { name: 'Server reminders' })).toBeVisible()
    expect(screen.getByLabelText('Number of reminders grouped by delivery status')).toBeVisible()
    expect(screen.getAllByRole('link', { name: 'Inspect' })).toHaveLength(2)
    fireEvent.change(screen.getByLabelText(/^Active reminders per member/), { target: { value: '12' } })
    fireEvent.click(screen.getByRole('switch', { name: 'Enable Reminders' }))
    fireEvent.click(screen.getByRole('button', { name: 'Save changes' }))
    await screen.findByText('Saved')

    await navigateTo(`/servers/${serverId}/modules/reminders/2`)
    expect(await screen.findByRole('heading', { name: 'Community game night starts soon' })).toBeVisible()
    expect(screen.getByRole('link', { name: /Silver.*@silver\.arc/ })).toHaveAttribute('href', `/servers/${serverId}/members/118742006320914432`)
    fireEvent.click(screen.getByRole('button', { name: 'Cancel reminder' }))
    await screen.findByText('Cancelled')
    expect(screen.queryByRole('button', { name: 'Cancel reminder' })).not.toBeInTheDocument()
  })

  it('creates an auto-delete rule and shows its content-safe queue', async () => {
    renderDashboard(); await navigateTo(`/servers/${serverId}/modules/auto_delete`)
    expect(await screen.findByRole('heading',{name:'Recent deletion jobs'})).toBeVisible()
    expect(screen.getByLabelText('Message deletion jobs grouped by status')).toBeVisible()
    fireEvent.click(screen.getByRole('button',{name:'Add rule'}))
    fireEvent.change(screen.getByLabelText(/^Rule ID/),{target:{value:'Media cleanup!'}})
    fireEvent.change(screen.getByLabelText(/^Delete after/),{target:{value:'120'}})
    expect(screen.getByLabelText(/^Rule ID/)).toHaveValue('mediacleanup')
    fireEvent.click(screen.getByRole('switch',{name:'Enable Auto-delete'}))
    expect(screen.getByRole('button',{name:'Save changes'})).toBeEnabled()
    await navigateTo(`/servers/${serverId}/modules/auto_delete/100000000000000005/300000000000000009`)
    expect(await screen.findByRole('heading',{name:'Message 300000000000000009'})).toBeVisible()
    expect(screen.getByRole('heading',{name:'Job details'})).toBeVisible()
  })

  it('creates, previews, delivers, and synchronizes a managed embed', async () => {
    renderDashboard()
    await navigateTo(`/servers/${serverId}/modules/managed_embeds`)
    expect(await screen.findByRole('heading', { name: 'Embed templates' })).toBeVisible()
    expect(screen.getByLabelText('Managed embed delivery attempts by outcome')).toBeVisible()
    fireEvent.click(screen.getByRole('switch', { name: 'Enable Managed embeds' }))
    fireEvent.click(screen.getByRole('button', { name: 'Save changes' }))
    await screen.findByText('Saved')

    fireEvent.click(screen.getByRole('link', { name: 'New template' }))
    expect(await screen.findByRole('heading', { name: 'Create embed' })).toBeVisible()
    fireEvent.change(screen.getByLabelText('Template name'), { target: { value: 'Launch update' } })
    fireEvent.change(screen.getAllByLabelText(/^Title/)[0], { target: { value: 'Hello {user}' } })
    fireEvent.click(screen.getByRole('button', { name: 'Add field' }))
    expect(screen.getByLabelText('Discord embed preview')).toHaveTextContent('Hello @imlunahey')
    fireEvent.click(screen.getByRole('button', { name: 'Save template' }))
    expect(await screen.findByRole('heading', { name: 'Launch update' })).toBeVisible()

    fireEvent.click(screen.getByRole('combobox', { name: 'Destination channel' }))
    fireEvent.click(screen.getByRole('option', { name: 'general' }))
    fireEvent.click(screen.getByRole('button', { name: 'Send embed' }))
    expect(await screen.findByRole('button', { name: 'Sync message' })).toBeVisible()
    fireEvent.click(screen.getByRole('button', { name: 'Sync message' }))
    await waitFor(() => expect(screen.getAllByText('edit').length).toBeGreaterThan(0))
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

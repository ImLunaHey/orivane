import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import { RouterProvider } from '@tanstack/react-router'
import { act, cleanup, fireEvent, render, screen, waitFor } from '@testing-library/react'
import { afterEach, beforeEach, describe, expect, it } from 'vitest'
import { router } from '../router'

const serverId = '901234567890123456'

function renderDashboard() {
  const client = new QueryClient({ defaultOptions: { queries: { retry: false }, mutations: { retry: false } } })
  return render(<QueryClientProvider client={client}><RouterProvider router={router} /></QueryClientProvider>)
}

async function navigateTo(path: string) {
  await act(async () => { await router.navigate({ to: path as never }) })
  await waitFor(() => expect(router.state.location.pathname).toBe(path))
}

describe('community operations pages', () => {
  beforeEach(() => localStorage.clear())
  afterEach(() => cleanup())

  it('renders durable activity for every community module', async () => {
    renderDashboard()
    const modules = [
      ['moderation_appeals', 'Moderation appeals', 'Appeal inbox'],
      ['polls', 'Polls', 'Poll history'],
      ['invite_tracking', 'Invite tracking', 'Invite leaderboard'],
      ['verification', 'Verification', 'Verification attempts'],
      ['counters', 'Counters and goals', 'Counter runs'],
      ['afk', 'AFK statuses', 'Current away members'],
      ['voice_text_linking', 'Voice and text activity', 'Linked voice activity'],
    ] as const

    for (const [moduleId, heading, history] of modules) {
      await navigateTo(`/servers/${serverId}/modules/${moduleId}/operations`)
      expect(await screen.findByRole('heading', { name: heading })).toBeVisible()
      expect(screen.getByRole('heading', { name: history })).toBeVisible()
      expect(screen.getByText('1', { selector: '.system-metrics strong' })).toBeVisible()
    }

    expect(screen.getByText(/Lounge|100000000000000006/)).toBeVisible()
  })

  it('shows invite attribution and clears an AFK status', async () => {
    renderDashboard()
    await navigateTo(`/servers/${serverId}/modules/invite_tracking/operations`)
    expect(await screen.findByRole('table', { name: 'Invite leaderboard' })).toBeVisible()
    expect(screen.getByText('14')).toBeVisible()
    expect(screen.getAllByText(/Silver|silver\.arc/).length).toBeGreaterThan(0)

    await navigateTo(`/servers/${serverId}/modules/afk/operations`)
    fireEvent.click(await screen.findByRole('button', { name: 'Clear' }))
    await waitFor(() => expect(screen.getByText('No activity has been recorded yet')).toBeVisible())
  })

  it('reviews an appeal on its own detail page', async () => {
    renderDashboard()
    await navigateTo(`/servers/${serverId}/modules/moderation_appeals/operations/2`)
    expect(await screen.findByRole('heading', { name: 'Moderation appeal' })).toBeVisible()
    expect(screen.getByText('Case #42')).toBeVisible()
    fireEvent.change(screen.getByLabelText('Internal review note'), { target: { value: 'Evidence checked.' } })
    fireEvent.click(screen.getByRole('button', { name: 'Approve' }))
    await waitFor(() => expect(screen.getByText('approved')).toBeVisible())
  })

  it('shows poll results and closes an active poll', async () => {
    renderDashboard()
    await navigateTo(`/servers/${serverId}/modules/polls/operations/7`)
    expect(await screen.findByRole('heading', { name: 'Which event should we run next?' })).toBeVisible()
    expect(screen.getAllByText('Game night').length).toBeGreaterThan(0)
    expect(screen.getByText('18 · 60.0%')).toBeVisible()
    fireEvent.click(screen.getByRole('button', { name: 'Close poll' }))
    await waitFor(() => expect(screen.queryByRole('button', { name: 'Close poll' })).not.toBeInTheDocument())
    expect(screen.getByText('closed')).toBeVisible()
  })

  it('handles unsupported operation and record routes', async () => {
    renderDashboard()
    await navigateTo(`/servers/${serverId}/modules/welcome/operations`)
    expect(await screen.findByRole('heading', { name: 'Operations page not found' })).toBeVisible()
    await navigateTo(`/servers/${serverId}/modules/verification/operations/4`)
    expect(await screen.findByRole('heading', { name: 'This activity has no separate detail view' })).toBeVisible()
  })
})

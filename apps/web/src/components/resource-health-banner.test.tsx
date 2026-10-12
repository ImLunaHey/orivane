import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import { fireEvent, render, screen, waitFor } from '@testing-library/react'
import { beforeEach, describe, expect, it, vi } from 'vitest'

const getResourceDiagnostics = vi.fn()
vi.mock('../lib/api', () => ({
  isDemoMode: false,
  getResourceDiagnostics: (...args: unknown[]) => getResourceDiagnostics(...args),
}))

import { ResourceHealthBanner } from './resource-health-banner'

function renderBanner() {
  const client = new QueryClient({ defaultOptions: { queries: { retry: false } } })
  return render(<QueryClientProvider client={client}><ResourceHealthBanner serverId="123" /></QueryClientProvider>)
}

describe('ResourceHealthBanner', () => {
  beforeEach(() => getResourceDiagnostics.mockReset())

  it('stays hidden when configured resources are healthy', async () => {
    getResourceDiagnostics.mockResolvedValue({ status: 'healthy', channels_available: true, roles_available: true, issues: [] })
    renderBanner()
    await waitFor(() => expect(getResourceDiagnostics).toHaveBeenCalledWith('123'))
    expect(screen.queryByRole('alert')).not.toBeInTheDocument()
  })

  it('shows actionable missing-resource details and retries', async () => {
    getResourceDiagnostics.mockResolvedValue({
      status: 'attention', channels_available: true, roles_available: true,
      issues: [{ path: 'modules.welcome.channel_id', kind: 'channel', id: '456', message: 'Select a replacement.' }],
    })
    renderBanner()
    expect(await screen.findByText('1 Discord resource issue needs attention')).toBeInTheDocument()
    expect(screen.getByText(/welcome.channel id/)).toBeInTheDocument()
    fireEvent.click(screen.getByRole('button', { name: 'Retry' }))
    await waitFor(() => expect(getResourceDiagnostics).toHaveBeenCalledTimes(2))
  })
})

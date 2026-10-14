import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import { RouterProvider } from '@tanstack/react-router'
import { act, cleanup, fireEvent, render, screen, waitFor } from '@testing-library/react'
import { afterEach, describe, expect, it } from 'vitest'
import { router } from '../router'

const serverId = '901234567890123456'

describe('server integrations', () => {
  afterEach(() => cleanup())

  it('creates a server-scoped MCP token and only reveals it once', async () => {
    const queryClient = new QueryClient({ defaultOptions: { queries: { retry: false }, mutations: { retry: false } } })
    render(<QueryClientProvider client={queryClient}><RouterProvider router={router} /></QueryClientProvider>)
    await act(async () => { await router.navigate({ to: `/servers/${serverId}/integrations` as never }) })
    await waitFor(() => expect(screen.getByRole('heading', { name: 'AI agent access' })).toBeInTheDocument())

    expect(screen.getByText('No assistants connected')).toBeInTheDocument()
    fireEvent.change(screen.getByRole('textbox', { name: /Connection name/ }), { target: { value: 'Support assistant' } })
    fireEvent.click(screen.getByRole('button', { name: 'Create access token' }))

    expect(await screen.findByText('Your new access token')).toBeInTheDocument()
    expect(screen.getByText(/orv_demo/)).toBeInTheDocument()
    expect(screen.getByText('Shown once')).toBeInTheDocument()
  })
})

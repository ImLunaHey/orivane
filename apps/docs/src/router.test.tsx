import { RouterProvider, createMemoryHistory } from '@tanstack/react-router'
import { cleanup, fireEvent, render, screen, waitFor, within } from '@testing-library/react'
import axe from 'axe-core'
import { afterEach, describe, expect, it } from 'vitest'
import { createDocsRouter } from './router'

afterEach(cleanup)

async function renderRoute(path: string) {
  const router = createDocsRouter(createMemoryHistory({ initialEntries: [path] }))
  await router.load()
  const view = render(<RouterProvider router={router} />)
  await waitFor(() => expect(document.querySelector('main')).toBeInTheDocument())
  return { router, ...view }
}

describe('documentation routes', () => {
  it('renders an accessible landing page with real catalog totals', async () => {
    await renderRoute('/docs/')
    expect(screen.getByRole('heading', { name: /Build a calmer server/ })).toBeInTheDocument()
    expect(screen.getByText('29')).toBeInTheDocument()
    expect((await axe.run(document.body, { rules: { 'color-contrast': { enabled: false } } })).violations).toEqual([])
  })

  it('filters the full module catalog', async () => {
    await renderRoute('/docs/modules')
    expect(screen.getByText('Showing 29 of 29 modules')).toBeInTheDocument()
    fireEvent.change(screen.getByLabelText('Filter modules'), { target: { value: 'phishing' } })
    expect(screen.getByText('Showing 1 of 29 modules')).toBeInTheDocument()
    expect(screen.getByRole('heading', { name: 'Automod' })).toBeInTheDocument()
    fireEvent.change(screen.getByLabelText('Filter modules'), { target: { value: 'impossible query' } })
    expect(screen.getByText('No modules match that filter.')).toBeInTheDocument()
  })

  it('renders a module detail with every settings group and demo', async () => {
    await renderRoute('/docs/modules/automod')
    expect(screen.getByRole('heading', { name: 'Automod', level: 1 })).toBeInTheDocument()
    expect(screen.getByText('Global filters')).toBeInTheDocument()
    expect(screen.getByText('rules[].kind')).toBeInTheDocument()
    expect(screen.getByText('Try the behavior safely')).toBeInTheDocument()
  })

  it('renders static guides and references', async () => {
    const pages = [
      ['/docs/guides/getting-started', 'Getting started'],
      ['/docs/guides/mcp', 'Model Context Protocol'],
      ['/docs/reference/commands', 'Slash-command catalog'],
      ['/docs/reference/audit', 'Audit logging and analytics'],
    ] as const
    for (const [path, heading] of pages) {
      const { unmount } = await renderRoute(path)
      expect(screen.getByRole('heading', { name: heading, level: 1 })).toBeInTheDocument()
      unmount()
    }
  })

  it('opens searchable keyboard navigation and closes it with Escape', async () => {
    await renderRoute('/docs/')
    fireEvent.keyDown(window, { key: 'k', ctrlKey: true })
    expect(screen.getByRole('dialog', { name: 'Search documentation' })).toBeInTheDocument()
    fireEvent.change(screen.getByPlaceholderText(/Try automod/), { target: { value: '/warn' } })
    expect(within(screen.getByRole('dialog')).getByText('Moderation')).toBeInTheDocument()
    fireEvent.keyDown(window, { key: 'Escape' })
    expect(screen.queryByRole('dialog')).not.toBeInTheDocument()
  })

  it('renders module and route not-found states', async () => {
    const module = await renderRoute('/docs/modules/not-real')
    expect(screen.getByText('Documentation page not found')).toBeInTheDocument()
    module.unmount()
    await renderRoute('/docs/nope')
    expect(screen.getByText('Documentation page not found')).toBeInTheDocument()
  })
})

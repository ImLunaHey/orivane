import { fireEvent, render, screen } from '@testing-library/react'
import { afterEach, expect, it, vi } from 'vitest'
import { PageErrorBoundary } from './page-error-boundary'

let shouldThrow = true

function FlakyPage() {
  if (shouldThrow) throw new Error('broken chart')
  return <h2>Recovered page</h2>
}

afterEach(() => {
  shouldThrow = true
  vi.restoreAllMocks()
})

it('contains a page crash and allows the content to retry', () => {
  vi.spyOn(console, 'error').mockImplementation(() => undefined)
  render(<PageErrorBoundary><FlakyPage /></PageErrorBoundary>)
  expect(screen.getByRole('heading', { name: 'This page could not be displayed' })).toBeVisible()
  expect(screen.getByText('broken chart')).toBeInTheDocument()
  shouldThrow = false
  fireEvent.click(screen.getByRole('button', { name: 'Retry page' }))
  expect(screen.getByRole('heading', { name: 'Recovered page' })).toBeVisible()
})

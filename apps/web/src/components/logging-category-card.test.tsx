import { fireEvent, render, screen } from '@testing-library/react'
import { describe, expect, it, vi } from 'vitest'
import { LoggingCategoryCard } from './logging-category-card'

describe('LoggingCategoryCard', () => {
  it('exposes category state and clears a channel override', () => {
    const onEnabledChange = vi.fn()
    const onChannelChange = vi.fn()
    render(<LoggingCategoryCard title="Messages" detail="Content-safe message lifecycle events." enabled channelId="200" channels={[{ id: '200', name: 'mod-log', kind: 'GuildText', position: 1, parent_id: null }]} pending={false} error={false} onEnabledChange={onEnabledChange} onChannelChange={onChannelChange} />)

    fireEvent.click(screen.getByRole('switch', { name: 'Log messages' }))
    expect(onEnabledChange).toHaveBeenCalledWith(false)
    expect(screen.getByRole('combobox', { name: 'Messages channel override' })).toHaveTextContent('mod-log')

    fireEvent.click(screen.getByRole('button', { name: 'Use default channel' }))
    expect(onChannelChange).toHaveBeenCalledWith(null)
  })
})

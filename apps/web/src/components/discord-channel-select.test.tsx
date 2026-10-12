import { fireEvent, render, screen, within } from '@testing-library/react'
import { describe, expect, it, vi } from 'vitest'
import { DiscordChannelSelect } from './discord-channel-select'

const channels = [
  { id: 'category-1', name: 'Information', kind: 'GuildCategory', position: 0, parent_id: null },
  { id: 'channel-1', name: 'welcome', kind: 'GuildText', position: 1, parent_id: 'category-1' },
  { id: 'category-2', name: 'Community', kind: 'GuildCategory', position: 2, parent_id: null },
  { id: 'channel-2', name: 'general', kind: 'GuildText', position: 3, parent_id: 'category-2' },
  { id: 'channel-3', name: 'announcements', kind: 'GuildAnnouncement', position: 4, parent_id: null },
]

describe('DiscordChannelSelect', () => {
  it('groups text channels beneath Discord categories', () => {
    const onChange = vi.fn()
    render(<DiscordChannelSelect title="Welcome channel" detail="Greeting destination" value={null} kind="text" channels={channels} pending={false} error={false} onChange={onChange} />)

    const picker = screen.getByRole('combobox', { name: 'Welcome channel' })
    fireEvent.click(picker)
    expect(screen.getByRole('listbox', { name: 'Welcome channel' })).toBeInTheDocument()
    expect(within(screen.getByRole('group', { name: 'Information' })).getByRole('option', { name: 'welcome' })).toBeInTheDocument()
    expect(within(screen.getByRole('group', { name: 'Community' })).getByRole('option', { name: 'general' })).toBeInTheDocument()
    expect(screen.queryByRole('option', { name: /Information \/ welcome/ })).not.toBeInTheDocument()

    fireEvent.click(screen.getByRole('option', { name: 'general' }))
    expect(onChange).toHaveBeenCalledWith('channel-2')
  })
})

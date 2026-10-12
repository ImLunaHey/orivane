import { fireEvent, render, screen, within } from '@testing-library/react'
import { describe, expect, it, vi } from 'vitest'
import { DiscordChannelMultiSelect } from './discord-channel-multi-select'
import { DiscordRoleSelect } from './discord-role-select'

const channels = [
  { id: '10', name: 'general', kind: 'GuildText', position: 1, parent_id: null },
  { id: '11', name: 'news', kind: 'GuildAnnouncement', position: 2, parent_id: null },
  { id: '12', name: 'Community', kind: 'GuildCategory', position: 0, parent_id: null },
]
const roles = [
  { id: '20', name: 'Moderator', position: 10, managed: false },
  { id: '21', name: 'Support', position: 9, managed: false },
]

describe('Discord resource multi-selects', () => {
  it('searches, selects, and removes channels', () => {
    const onChange = vi.fn()
    const { rerender } = render(<DiscordChannelMultiSelect title="Ignored channels" detail="Ignored" values={[]} channels={channels} pending={false} error={false} onChange={onChange} />)
    fireEvent.click(screen.getByRole('button', { name: 'Ignored channels' }))
    fireEvent.change(screen.getByRole('textbox', { name: 'Search ignored channels' }), { target: { value: 'gen' } })
    const listbox = screen.getByRole('listbox')
    expect(within(listbox).getByRole('option', { name: 'general' })).toBeVisible()
    expect(within(listbox).queryByRole('option', { name: 'news' })).not.toBeInTheDocument()
    fireEvent.click(within(listbox).getByRole('option', { name: 'general' }))
    expect(onChange).toHaveBeenCalledWith(['10'])

    rerender(<DiscordChannelMultiSelect title="Ignored channels" detail="Ignored" values={['10']} channels={channels} pending={false} error={false} onChange={onChange} />)
    fireEvent.click(screen.getByRole('button', { name: 'Remove general' }))
    expect(onChange).toHaveBeenLastCalledWith([])
  })

  it('selects roles and enforces the configured maximum', () => {
    const onChange = vi.fn()
    const { rerender } = render(<DiscordRoleSelect title="Support roles" detail="Support" values={[]} roles={roles} pending={false} error={false} max={1} onChange={onChange} />)
    fireEvent.click(screen.getByRole('button', { name: 'Support roles' }))
    fireEvent.click(screen.getByRole('option', { name: 'Moderator' }))
    expect(onChange).toHaveBeenCalledWith(['20'])

    rerender(<DiscordRoleSelect title="Support roles" detail="Support" values={['20']} roles={roles} pending={false} error={false} max={1} onChange={onChange} />)
    expect(screen.getByRole('option', { name: 'Support' })).toBeDisabled()
    fireEvent.click(screen.getByRole('button', { name: 'Remove Moderator' }))
    expect(onChange).toHaveBeenLastCalledWith([])
  })

  it('communicates pending and failure states', () => {
    const noop = vi.fn()
    const { rerender } = render(<DiscordRoleSelect title="Roles" detail="Roles" values={[]} roles={[]} pending error={false} onChange={noop} />)
    expect(screen.getByRole('button', { name: 'Roles' })).toBeDisabled()
    expect(screen.getByText('Loading Discord roles…')).toBeVisible()
    rerender(<DiscordRoleSelect title="Roles" detail="Roles" values={[]} roles={[]} pending={false} error onChange={noop} />)
    expect(screen.getByText('Could not load Discord roles')).toBeVisible()
  })
})

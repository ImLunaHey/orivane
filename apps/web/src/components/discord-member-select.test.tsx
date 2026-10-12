import { fireEvent, render, screen } from '@testing-library/react'
import { describe, expect, it, vi } from 'vitest'
import { DiscordMemberSelect } from './discord-member-select'

const members = [
  { id: '100', username: 'imlunahey', display_name: 'luna', commands_30d: 4, cases: 0, last_seen: null },
  { id: '200', username: 'helper', display_name: 'Support Helper', commands_30d: 1, cases: 0, last_seen: null },
]

describe('DiscordMemberSelect', () => {
  it('searches known members by username and selects one', () => {
    const onChange = vi.fn()
    render(<DiscordMemberSelect title="Assigned team member" detail="Ticket owner" value={null} members={members} pending={false} error={false} onChange={onChange}/>)

    fireEvent.click(screen.getByRole('combobox', { name: 'Assigned team member' }))
    fireEvent.change(screen.getByRole('textbox', { name: 'Search members' }), { target: { value: 'helper' } })
    expect(screen.queryByRole('option', { name: /luna/ })).not.toBeInTheDocument()
    fireEvent.click(screen.getByRole('option', { name: /Support Helper/ }))
    expect(onChange).toHaveBeenCalledWith('200')
  })

  it('renders loading and error states as disabled controls', () => {
    const { rerender } = render(<DiscordMemberSelect title="Assignee" detail="Ticket owner" value={null} members={[]} pending error={false} onChange={vi.fn()}/>)
    expect(screen.getByRole('combobox', { name: 'Assignee' })).toBeDisabled()
    expect(screen.getByText('Loading members…')).toBeInTheDocument()
    rerender(<DiscordMemberSelect title="Assignee" detail="Ticket owner" value={null} members={[]} pending={false} error onChange={vi.fn()}/>)
    expect(screen.getByText('Could not load members')).toBeInTheDocument()
  })
})

import { fireEvent, render, screen } from '@testing-library/react'
import { describe, expect, it } from 'vitest'
import { AntiRaidDemo } from './anti-raid-demo'
import { AutomodDemo } from './automod-demo'
import { CounterDemo } from './counter-demo'
import { FormDemo } from './form-demo'
import { LevelingDemo } from './leveling-demo'
import { PollDemo } from './poll-demo'
import { WelcomeDemo } from './welcome-demo'
import { DemoPanel } from './demo-panel'

describe('client-side documentation demos', () => {
  it('renders welcome variables and updates the preview', () => {
    render(<WelcomeDemo />)
    expect(screen.getByText('@luna')).toBeInTheDocument()
    fireEvent.change(screen.getByLabelText('Welcome message'), { target: { value: 'Hi {username} at {server}' } })
    expect(screen.getByText("Hi imlunahey at Luna's Lab")).toBeInTheDocument()
  })

  it('shows layered Automod decisions', () => {
    render(<AutomodDemo />)
    expect(screen.getByText('Message would be blocked')).toBeInTheDocument()
    fireEvent.change(screen.getByLabelText('Test message'), { target: { value: 'A calm ordinary message' } })
    expect(screen.getByText('Message would pass')).toBeInTheDocument()
  })

  it('calculates leveling estimates from guarded numeric inputs', () => {
    render(<LevelingDemo />)
    fireEvent.change(screen.getByLabelText('Eligible messages'), { target: { value: '400' } })
    expect(screen.getByText('8,000 total XP')).toBeInTheDocument()
  })

  it('models both inactive and active anti-raid decisions', () => {
    render(<AntiRaidDemo />)
    expect(screen.getByText('Raid mode active')).toBeInTheDocument()
    fireEvent.change(screen.getByLabelText(/Joins in detection window/), { target: { value: '1' } })
    expect(screen.getByText('Normal join traffic')).toBeInTheDocument()
  })

  it('submits a local-only form preview', () => {
    render(<FormDemo />)
    fireEvent.change(screen.getByPlaceholderText('Tell us about your experience'), { target: { value: 'Moderator' } })
    fireEvent.click(screen.getByRole('button', { name: 'Submit' }))
    expect(screen.getByText(/Submission preview completed/)).toBeInTheDocument()
  })

  it('moves a poll vote and updates the total without creating extra votes', () => {
    render(<PollDemo />)
    const total = screen.getByText(/22 total votes/)
    fireEvent.click(screen.getByRole('button', { name: /Revise it/ }))
    expect(total).toHaveTextContent('22 total votes')
    expect(screen.getByRole('button', { name: /Revise it/ })).toHaveAttribute('aria-pressed', 'true')
  })

  it('renders counter variables and truncates to Discord limits', () => {
    render(<CounterDemo />)
    fireEvent.change(screen.getByLabelText('Current value'), { target: { value: '150' } })
    expect(screen.getByText('Members: 150 / 200 (50 left)')).toBeInTheDocument()
  })

  it('dispatches every documented demo kind', () => {
    const { rerender } = render(<DemoPanel kind="welcome" />)
    for (const kind of ['automod', 'leveling', 'anti-raid', 'forms', 'polls', 'counters'] as const) {
      rerender(<DemoPanel kind={kind} />)
      expect(screen.getByText('Try the behavior safely')).toBeInTheDocument()
    }
  })
})

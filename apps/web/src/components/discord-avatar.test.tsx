import { render } from '@testing-library/react'
import { describe, expect, it } from 'vitest'
import { DiscordAvatar } from './discord-avatar'

describe('DiscordAvatar', () => {
  it('renders a stored Discord avatar without duplicating adjacent accessible text', () => {
    render(<DiscordAvatar className="case-avatar" avatarUrl="https://cdn.discordapp.com/avatars/123/avatar.png" name="imlunahey" />)

    const image = document.querySelector('img')
    expect(image).toHaveAttribute('src', 'https://cdn.discordapp.com/avatars/123/avatar.png')
    expect(image).toHaveAttribute('alt', '')
  })

  it('falls back to initials when no avatar was observed', () => {
    const { container } = render(<DiscordAvatar className="case-avatar" avatarUrl={null} name="luna" />)

    expect(container.querySelector('.case-avatar')).toHaveTextContent('LU')
  })
})

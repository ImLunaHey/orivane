import { fireEvent, render, screen } from '@testing-library/react'
import { useState } from 'react'
import { describe, expect, it } from 'vitest'
import { WelcomeMessageEditor } from './welcome-message-editor'

function EditorHarness() {
  const [value, setValue] = useState('Welcome ')
  return <WelcomeMessageEditor value={value} serverId="1516120355163476170" serverName="Luna's Lab" onChange={setValue} />
}

describe('WelcomeMessageEditor', () => {
  it('inserts variables through the CodeMirror editor', () => {
    render(<EditorHarness />)
    const editor = screen.getByRole('textbox', { name: 'Welcome message' })

    fireEvent.click(screen.getByTitle('Insert Member mention'))
    expect(editor).toHaveTextContent('{user}Welcome')
    expect(screen.getByText('@imlunahey')).toHaveClass('discord-preview-mention')
  })
})

import { fireEvent, render, screen } from '@testing-library/react'
import { useState } from 'react'
import { describe, expect, it } from 'vitest'
import { DiscordEmbedPreview, TemplateTextEditor } from './embed-editor'
import type { EmbedPayload } from '../types'

function Harness(){const[value,setValue]=useState('Hello ');return <><TemplateTextEditor title="Description" detail="Embed copy" value={value} maxLength={100} onChange={setValue}/><DiscordEmbedPreview serverId="1516120355163476170" serverName="Luna's Lab" payload={{content:'',title:value,description:'Welcome to {server}, {user}',url:'',color:5797266,author_name:'',author_url:'',author_icon_url:'',footer_text:'',footer_icon_url:'',thumbnail_url:'',image_url:'',timestamp:false,fields:[]} satisfies EmbedPayload}/></>}

describe('managed embed editor components',()=>{
  it('offers variable autocomplete and renders Discord mentions',()=>{
    render(<Harness/>);const editor=screen.getByRole('textbox',{name:'Description'});fireEvent.input(editor,{target:{textContent:'Hello {user}'}});expect(screen.getByLabelText('Discord embed preview')).toHaveTextContent("Welcome to Luna's Lab, @imlunahey");expect(screen.getByText('@imlunahey')).toHaveClass('discord-preview-mention')
  })
})

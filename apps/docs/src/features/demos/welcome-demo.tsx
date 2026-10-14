import { useState } from 'react'

export function WelcomeDemo() {
  const [message, setMessage] = useState('Welcome {user} to {server}! Read the rules and say hello.')
  const [title, setTitle] = useState('A new member appeared')
  const rendered = message.replaceAll('{user}', '@luna').replaceAll('{username}', 'imlunahey').replaceAll('{server}', "Luna's Lab")
  return <div className="demo-grid"><div className="demo-controls"><label><span>Embed title</span><input value={title} onChange={(event) => setTitle(event.target.value)} /></label><label><span>Welcome message</span><textarea value={message} onChange={(event) => setMessage(event.target.value)} /></label><p className="demo-help">Variables: <code>{'{user}'}</code> <code>{'{username}'}</code> <code>{'{server}'}</code></p></div><div className="discord-preview"><div className="discord-avatar">O</div><div><div className="discord-author">Orivane <b>APP</b> <time>Today at 14:32</time></div><div className="discord-embed"><strong>{title || 'Untitled welcome'}</strong><p>{renderMentions(rendered)}</p><small>Luna's Lab · member #1,204</small></div></div></div></div>
}

function renderMentions(text: string) {
  const [before, after] = text.split('@luna')
  return <>{before}<mark>@luna</mark>{after}</>
}

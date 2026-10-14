import { ShieldAlert, ShieldCheck } from 'lucide-react'
import { useMemo, useState } from 'react'

export function AutomodDemo() {
  const [message, setMessage] = useState('FREE NITRO!!! join discord.gg/example @everyone @here')
  const [maxMentions, setMaxMentions] = useState(1)
  const result = useMemo(() => {
    const matches = [] as string[]
    const mentions = message.match(/@(?:everyone|here|[\w.-]+)/g)?.length ?? 0
    if (/discord(?:\.gg|\.com\/invite)\//i.test(message)) matches.push('Discord invite')
    if (/free\s+(?:nitro|gift)/i.test(message)) matches.push('Phishing phrase')
    if (mentions > maxMentions) matches.push(`Mention limit (${mentions}/${maxMentions})`)
    const letters = message.match(/[a-z]/gi) ?? []
    const capitals = letters.filter((letter) => letter === letter.toUpperCase()).length
    if (letters.length >= 8 && capitals / letters.length > 0.8) matches.push('Excessive capitals')
    return matches
  }, [maxMentions, message])
  return <div className="demo-grid"><div className="demo-controls"><label><span>Test message</span><textarea value={message} onChange={(event) => setMessage(event.target.value)} /></label><label><span>Maximum mentions: {maxMentions}</span><input type="range" min="0" max="10" value={maxMentions} onChange={(event) => setMaxMentions(Number(event.target.value))} /></label><p className="demo-help">This simplified browser model demonstrates layered matches. Production checks run in the Rust module without retaining message content.</p></div><div className={`rule-result ${result.length ? 'blocked' : 'allowed'}`}>{result.length ? <ShieldAlert /> : <ShieldCheck />}<div><strong>{result.length ? 'Message would be blocked' : 'Message would pass'}</strong><p>{result.length ? `${result.length} rules matched` : 'No enabled rule matched this example.'}</p>{result.map((match) => <span key={match}>{match}</span>)}</div></div></div>
}

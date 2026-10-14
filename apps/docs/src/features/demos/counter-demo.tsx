import { useState } from 'react'
import { NumericInput } from './numeric-input'

export function CounterDemo() {
  const [template, setTemplate] = useState('Members: {count} / {goal} ({remaining} left)')
  const [count, setCount] = useState(123)
  const [goal, setGoal] = useState(200)
  const rendered = template.replaceAll('{count}', String(count)).replaceAll('{goal}', String(goal)).replaceAll('{remaining}', String(Math.max(0, goal - count))).slice(0, 100)
  return <div className="demo-grid"><div className="demo-controls"><label><span>Channel-name template</span><input value={template} onChange={(event) => setTemplate(event.target.value)} /></label><div className="compact-inputs"><NumericInput label="Current value" value={count} setValue={setCount} min={0} /><NumericInput label="Goal" value={goal} setValue={setGoal} min={0} /></div><p className="demo-help">Discord channel names are limited to 100 characters. Orivane records each rendered value and update outcome.</p></div><div className="channel-preview"><span>🔊</span><div><small>VOICE CHANNEL PREVIEW</small><strong>{rendered || 'Untitled counter'}</strong></div></div></div>
}

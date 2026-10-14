import { Trophy } from 'lucide-react'
import { useMemo, useState } from 'react'
import { NumericInput } from './numeric-input'

export function LevelingDemo() {
  const [messages, setMessages] = useState(120)
  const [minimum, setMinimum] = useState(15)
  const [maximum, setMaximum] = useState(25)
  const result = useMemo(() => {
    const xp = Math.round(messages * ((minimum + maximum) / 2))
    const level = Math.floor(Math.sqrt(xp / 100))
    const currentFloor = level * level * 100
    const nextFloor = (level + 1) * (level + 1) * 100
    return { xp, level, progress: Math.round(((xp - currentFloor) / Math.max(1, nextFloor - currentFloor)) * 100) }
  }, [maximum, messages, minimum])
  return <div className="demo-grid"><div className="demo-controls"><NumericInput label="Eligible messages" value={messages} setValue={setMessages} min={0} max={100000} /><div className="compact-inputs"><NumericInput label="XP minimum" value={minimum} setValue={setMinimum} min={1} /><NumericInput label="XP maximum" value={maximum} setValue={setMaximum} min={minimum} /></div><p className="demo-help">Estimate uses the average configured award. Cooldowns and anti-farming rules determine which messages are eligible.</p></div><div className="rank-card"><Trophy /><div><small>ESTIMATED RANK CARD</small><strong>Level {result.level}</strong><p>{result.xp.toLocaleString()} total XP</p><div className="progress-track"><span style={{ width: `${Math.max(0, Math.min(100, result.progress))}%` }} /></div><small>{result.progress}% toward level {result.level + 1}</small></div></div></div>
}

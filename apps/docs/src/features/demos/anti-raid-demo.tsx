import { Shield, Siren } from 'lucide-react'
import { useState } from 'react'
import { NumericInput } from './numeric-input'

export function AntiRaidDemo() {
  const [joins, setJoins] = useState(12)
  const [threshold, setThreshold] = useState(10)
  const [age, setAge] = useState(2)
  const [minimumAge, setMinimumAge] = useState(7)
  const active = joins >= threshold
  const risky = age < minimumAge
  return <div className="demo-grid"><div className="demo-controls"><NumericInput label={`Joins in detection window: ${joins}`} value={joins} setValue={setJoins} min={0} max={50} range /><NumericInput label={`Raid threshold: ${threshold}`} value={threshold} setValue={setThreshold} min={2} max={30} range /><div className="compact-inputs"><NumericInput label="Account age (days)" value={age} setValue={setAge} min={0} /><NumericInput label="Minimum age" value={minimumAge} setValue={setMinimumAge} min={0} /></div></div><div className={`raid-scenario ${active ? 'active' : ''}`}>{active ? <Siren /> : <Shield />}<div><small>SIMULATED DECISION</small><strong>{active ? 'Raid mode active' : 'Normal join traffic'}</strong><p>{active ? (risky ? 'This new account matches the risk policy and receives the configured action.' : 'The burst is active, but this account is not risky unless “contain every raid join” is enabled.') : 'No automatic containment occurs before the join threshold is reached.'}</p><span>{risky ? 'Risky account' : 'Account passes age policy'}</span></div></div></div>
}

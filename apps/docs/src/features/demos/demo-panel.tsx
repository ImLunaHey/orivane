import type { DemoKind } from '../../content/types'
import { AntiRaidDemo } from './anti-raid-demo'
import { AutomodDemo } from './automod-demo'
import { CounterDemo } from './counter-demo'
import { FormDemo } from './form-demo'
import { LevelingDemo } from './leveling-demo'
import { PollDemo } from './poll-demo'
import { WelcomeDemo } from './welcome-demo'

export function DemoPanel({ kind }: { kind: DemoKind }) {
  return <section className="demo-section"><div className="demo-heading"><div><p className="eyebrow">Client-side playground</p><h2>Try the behavior safely</h2></div><span>Nothing is sent or saved</span></div>{renderDemo(kind)}</section>
}

function renderDemo(kind: DemoKind) {
  switch (kind) {
    case 'welcome': return <WelcomeDemo />
    case 'automod': return <AutomodDemo />
    case 'leveling': return <LevelingDemo />
    case 'anti-raid': return <AntiRaidDemo />
    case 'forms': return <FormDemo />
    case 'polls': return <PollDemo />
    case 'counters': return <CounterDemo />
  }
}

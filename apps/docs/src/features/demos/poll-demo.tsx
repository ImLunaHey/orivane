import { useState } from 'react'

const choices = ['Ship it', 'Revise it', 'Discuss first']

export function PollDemo() {
  const [selected, setSelected] = useState(0)
  const [counts, setCounts] = useState([14, 5, 3])
  const vote = (index: number) => {
    setCounts((current) => current.map((count, item) => count + (item === index ? 1 : 0) - (item === selected ? 1 : 0)))
    setSelected(index)
  }
  const total = counts.reduce((sum, count) => sum + count, 0)
  return <div className="poll-preview"><div><small>ORIVANE POLL</small><h3>Should we enable the new welcome flow?</h3><p>Choose one option. Selecting another safely moves your vote.</p></div>{choices.map((choice, index) => <button type="button" aria-pressed={selected === index} key={choice} onClick={() => vote(index)}><span>{choice}</span><span>{counts[index]} · {Math.round((counts[index] / total) * 100)}%</span><i style={{ width: `${(counts[index] / total) * 100}%` }} /></button>)}<small>{total} total votes · demo data</small></div>
}

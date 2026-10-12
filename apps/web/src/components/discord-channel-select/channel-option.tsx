import { Check } from 'lucide-react'
import type { ReactNode } from 'react'

export function ChannelOption({ id, label, selected, active, icon, onHover, onChoose }: { id: string; label: string; selected: boolean; active: boolean; icon: ReactNode; onHover: () => void; onChoose: () => void }) {
  return <button id={id} type="button" role="option" aria-selected={selected} tabIndex={-1} className={`discord-channel-option ${active ? 'is-active' : ''}`} onMouseEnter={onHover} onClick={onChoose}><span className="discord-channel-option-name">{icon}{label}</span>{selected && <Check className="discord-channel-check" size={17} />}</button>
}

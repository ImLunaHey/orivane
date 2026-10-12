import { Check, ChevronDown, Hash, X } from 'lucide-react'
import { useEffect, useId, useMemo, useRef, useState } from 'react'
import type { GuildChannel } from '../types'

type Props = {
  title: string
  detail: string
  values: string[]
  channels: GuildChannel[]
  pending: boolean
  error: boolean
  onChange: (values: string[]) => void
}

export function DiscordChannelMultiSelect({ title, detail, values, channels, pending, error, onChange }: Props) {
  const id = useId()
  const rootRef = useRef<HTMLDivElement>(null)
  const [open, setOpen] = useState(false)
  const [query, setQuery] = useState('')
  const available = useMemo(() => channels.filter((channel) => channel.kind === 'GuildText' || channel.kind === 'GuildAnnouncement'), [channels])
  const filtered = available.filter((channel) => channel.name.toLowerCase().includes(query.trim().toLowerCase()))
  const selected = available.filter((channel) => values.includes(channel.id))
  useEffect(() => {
    if (!open) return
    const close = (event: PointerEvent) => { if (!rootRef.current?.contains(event.target as Node)) setOpen(false) }
    document.addEventListener('pointerdown', close)
    return () => document.removeEventListener('pointerdown', close)
  }, [open])
  const toggle = (channel: GuildChannel) => onChange(values.includes(channel.id) ? values.filter((value) => value !== channel.id) : [...values, channel.id])
  return <div className={`discord-role-field wide ${open ? 'is-open' : ''}`} ref={rootRef}>
    <span id={`${id}-label`}>{title}</span><small id={`${id}-detail`}>{detail}</small>
    <button type="button" className="discord-channel-trigger" aria-labelledby={`${id}-label`} aria-describedby={`${id}-detail`} aria-haspopup="listbox" aria-expanded={open} disabled={pending || error} onClick={() => setOpen((value) => !value)}>
      <span className={selected.length ? undefined : 'placeholder'}><Hash size={17} />{pending ? 'Loading Discord channels…' : error ? 'Could not load Discord channels' : selected.length ? `${selected.length} channel${selected.length === 1 ? '' : 's'} selected` : 'Select server channels'}</span><ChevronDown className="discord-channel-chevron" size={17} />
    </button>
    {selected.length > 0 && <div className="discord-role-chips">{selected.map((channel) => <button type="button" key={channel.id} onClick={() => toggle(channel)} aria-label={`Remove ${channel.name}`}><Hash size={14} />{channel.name}<X size={14} /></button>)}</div>}
    {open && <div className="discord-role-menu"><input autoFocus aria-label={`Search ${title.toLowerCase()}`} value={query} onChange={(event) => setQuery(event.target.value)} placeholder="Search server channels" /><div role="listbox" aria-multiselectable="true">{filtered.map((channel) => { const checked = values.includes(channel.id); return <button type="button" role="option" aria-selected={checked} key={channel.id} onClick={() => toggle(channel)}><span><Hash size={16} />{channel.name}</span>{checked && <Check size={17} />}</button> })}{!filtered.length && <p>No matching channels</p>}</div></div>}
  </div>
}

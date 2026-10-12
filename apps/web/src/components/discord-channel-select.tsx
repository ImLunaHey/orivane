import { Check, ChevronDown, Folder, Hash, Minus } from 'lucide-react'
import { useEffect, useId, useMemo, useRef, useState } from 'react'
import type { GuildChannel } from '../types'

type DiscordChannelSelectProps = {
  title: string
  detail: string
  value: string | null
  kind: 'text' | 'category'
  channels: GuildChannel[]
  pending: boolean
  error: boolean
  onChange: (value: string | null) => void
}

type ChannelGroup = { id: string; label: string; channels: GuildChannel[] }

export function DiscordChannelSelect({ title, detail, value, kind, channels, pending, error, onChange }: DiscordChannelSelectProps) {
  const baseId = useId()
  const labelId = `${baseId}-label`
  const detailId = `${baseId}-detail`
  const listboxId = `${baseId}-listbox`
  const rootRef = useRef<HTMLDivElement>(null)
  const triggerRef = useRef<HTMLButtonElement>(null)
  const [open, setOpen] = useState(false)
  const [activeIndex, setActiveIndex] = useState(0)
  const disabled = pending || error
  const allowed = useMemo(() => channels.filter((channel) => kind === 'category'
    ? channel.kind === 'GuildCategory'
    : channel.kind === 'GuildText' || channel.kind === 'GuildAnnouncement'), [channels, kind])
  const groups = useMemo(() => channelGroups(kind, channels, allowed), [kind, channels, allowed])
  const orderedChannels = useMemo(() => groups.flatMap((group) => group.channels), [groups])
  const options = useMemo(() => [null, ...orderedChannels], [orderedChannels])
  const selected = allowed.find((channel) => channel.id === value)

  useEffect(() => {
    if (!open) return
    const closeOutside = (event: PointerEvent) => {
      if (!rootRef.current?.contains(event.target as Node)) setOpen(false)
    }
    document.addEventListener('pointerdown', closeOutside)
    return () => document.removeEventListener('pointerdown', closeOutside)
  }, [open])

  useEffect(() => {
    if (open) document.getElementById(optionId(baseId, activeIndex))?.scrollIntoView?.({ block: 'nearest' })
  }, [activeIndex, baseId, open])

  const openPicker = () => {
    const selectedIndex = orderedChannels.findIndex((channel) => channel.id === value)
    setActiveIndex(selectedIndex >= 0 ? selectedIndex + 1 : 0)
    setOpen(true)
  }

  const choose = (channel: GuildChannel | null) => {
    onChange(channel?.id ?? null)
    setOpen(false)
    triggerRef.current?.focus()
  }

  const move = (amount: number) => {
    setActiveIndex((current) => (current + amount + options.length) % options.length)
  }

  const onKeyDown = (event: React.KeyboardEvent<HTMLButtonElement>) => {
    if (disabled) return
    if (event.key === 'ArrowDown' || event.key === 'ArrowUp') {
      event.preventDefault()
      if (!open) openPicker()
      else move(event.key === 'ArrowDown' ? 1 : -1)
      return
    }
    if (event.key === 'Home' || event.key === 'End') {
      event.preventDefault()
      if (!open) openPicker()
      setActiveIndex(event.key === 'Home' ? 0 : options.length - 1)
      return
    }
    if ((event.key === 'Enter' || event.key === ' ') && open) {
      event.preventDefault()
      choose(options[activeIndex])
      return
    }
    if (event.key === 'Escape' && open) {
      event.preventDefault()
      setOpen(false)
    }
    if (event.key === 'Tab') setOpen(false)
  }

  return <div className={`discord-channel-field ${open ? 'is-open' : ''}`} ref={rootRef}>
    <span id={labelId}>{title}</span>
    <small id={detailId}>{detail}</small>
    <button
      ref={triggerRef}
      type="button"
      className="discord-channel-trigger"
      role="combobox"
      aria-labelledby={labelId}
      aria-describedby={detailId}
      aria-haspopup="listbox"
      aria-controls={listboxId}
      aria-expanded={open}
      aria-activedescendant={open ? optionId(baseId, activeIndex) : undefined}
      disabled={disabled}
      onKeyDown={onKeyDown}
      onClick={() => open ? setOpen(false) : openPicker()}
    >
      <span className={!selected ? 'placeholder' : undefined}>
        {selected ? <><ChannelIcon kind={selected.kind} />{selected.name}</> : value ? <>Unavailable channel · {value}</> : <>{placeholder(kind, pending, error)}</>}
      </span>
      <ChevronDown className="discord-channel-chevron" size={17} />
    </button>
    {open && <div id={listboxId} className="discord-channel-menu" role="listbox" aria-labelledby={labelId}>
      <ChannelOption
        id={optionId(baseId, 0)}
        label={kind === 'category' ? 'No category selected' : 'No channel selected'}
        selected={value === null}
        active={activeIndex === 0}
        icon={<Minus size={17} />}
        onHover={() => setActiveIndex(0)}
        onChoose={() => choose(null)}
      />
      {groups.map((group) => <div className="discord-channel-group" role="group" aria-labelledby={`${baseId}-group-${group.id}`} key={group.id}>
        <div className="discord-channel-group-label" id={`${baseId}-group-${group.id}`}>{group.label}</div>
        {group.channels.map((channel) => {
          const index = orderedChannels.indexOf(channel) + 1
          return <ChannelOption
            key={channel.id}
            id={optionId(baseId, index)}
            label={channel.name}
            selected={channel.id === value}
            active={activeIndex === index}
            icon={<ChannelIcon kind={channel.kind} />}
            onHover={() => setActiveIndex(index)}
            onChoose={() => choose(channel)}
          />
        })}
      </div>)}
    </div>}
  </div>
}

function ChannelOption({ id, label, selected, active, icon, onHover, onChoose }: { id: string; label: string; selected: boolean; active: boolean; icon: React.ReactNode; onHover: () => void; onChoose: () => void }) {
  return <button
    id={id}
    type="button"
    role="option"
    aria-selected={selected}
    tabIndex={-1}
    className={`discord-channel-option ${active ? 'is-active' : ''}`}
    onMouseEnter={onHover}
    onClick={onChoose}
  >
    <span className="discord-channel-option-name">{icon}{label}</span>
    {selected && <Check className="discord-channel-check" size={17} />}
  </button>
}

function ChannelIcon({ kind }: { kind: string }) {
  return kind === 'GuildCategory' ? <Folder size={17} /> : <Hash size={17} />
}

function channelGroups(kind: DiscordChannelSelectProps['kind'], allChannels: GuildChannel[], allowed: GuildChannel[]): ChannelGroup[] {
  if (kind === 'category') return [{ id: 'categories', label: 'Categories', channels: allowed }]
  const categories = allChannels.filter((channel) => channel.kind === 'GuildCategory')
  const categoryIds = new Set(categories.map((category) => category.id))
  const groups: ChannelGroup[] = []
  const uncategorized = allowed.filter((channel) => !channel.parent_id || !categoryIds.has(channel.parent_id))
  if (uncategorized.length) groups.push({ id: 'uncategorized', label: 'No category', channels: uncategorized })
  for (const category of categories) {
    const children = allowed.filter((channel) => channel.parent_id === category.id)
    if (children.length) groups.push({ id: category.id, label: category.name, channels: children })
  }
  return groups
}

function optionId(baseId: string, index: number) {
  return `${baseId}-option-${index}`
}

function placeholder(kind: DiscordChannelSelectProps['kind'], pending: boolean, error: boolean) {
  if (pending) return 'Loading Discord channels…'
  if (error) return 'Could not load Discord channels'
  return kind === 'category' ? 'Select a category' : 'Select a channel'
}

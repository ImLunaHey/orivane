import { Check, ChevronDown, Search, UserRound, X } from 'lucide-react'
import { useEffect, useId, useMemo, useRef, useState } from 'react'
import type { GuildMemberSummary } from '../lib/api'
import { DiscordAvatar } from './discord-avatar'

type Props = {
  title: string
  detail: string
  value: string | null
  members: GuildMemberSummary[]
  pending: boolean
  error: boolean
  onChange: (value: string | null) => void
}

export function DiscordMemberSelect({ title, detail, value, members, pending, error, onChange }: Props) {
  const id = useId()
  const root = useRef<HTMLDivElement>(null)
  const search = useRef<HTMLInputElement>(null)
  const [open, setOpen] = useState(false)
  const [query, setQuery] = useState('')
  const selected = members.find((member) => member.id === value)
  const results = useMemo(() => {
    const needle = query.trim().toLowerCase()
    if (!needle) return members
    return members.filter((member) => [member.username, member.display_name, member.id]
      .some((part) => part?.toLowerCase().includes(needle)))
  }, [members, query])

  useEffect(() => {
    if (!open) return
    const close = (event: PointerEvent) => {
      if (!root.current?.contains(event.target as Node)) setOpen(false)
    }
    document.addEventListener('pointerdown', close)
    search.current?.focus()
    return () => document.removeEventListener('pointerdown', close)
  }, [open])

  const choose = (memberId: string | null) => {
    onChange(memberId)
    setOpen(false)
    setQuery('')
  }

  return <div className={`discord-channel-field ${open ? 'is-open' : ''}`} ref={root}>
    <span id={`${id}-label`}>{title}</span>
    <small id={`${id}-detail`}>{detail}</small>
    <button
      type="button"
      className="discord-channel-trigger"
      role="combobox"
      aria-labelledby={`${id}-label`}
      aria-describedby={`${id}-detail`}
      aria-haspopup="listbox"
      aria-expanded={open}
      aria-controls={`${id}-options`}
      disabled={pending || error}
      onClick={() => setOpen((current) => !current)}
    >
      <span className={!selected ? 'placeholder' : undefined}>{selected ? <DiscordAvatar className="discord-member-avatar" avatarUrl={selected.avatar_url} name={memberLabel(selected)} /> : <UserRound size={17}/>} {selected ? memberLabel(selected) : value ? `Discord user ${value.slice(-4)}` : pending ? 'Loading members…' : error ? 'Could not load members' : 'Select a member'}</span>
      <ChevronDown className="discord-channel-chevron" size={17}/>
    </button>
    {open && <div id={`${id}-options`} className="discord-channel-menu discord-member-menu" role="listbox" aria-labelledby={`${id}-label`}>
      <label className="discord-member-search"><Search size={16}/><input ref={search} aria-label="Search members" value={query} onChange={(event) => setQuery(event.target.value)} placeholder="Search name or username"/></label>
      {value && <button type="button" role="option" aria-selected={false} className="discord-channel-option" onClick={() => choose(null)}><span className="discord-channel-option-name"><X size={17}/>Unassign</span></button>}
      {results.map((member) => <button key={member.id} type="button" role="option" aria-selected={member.id === value} className="discord-channel-option" onClick={() => choose(member.id)}>
        <span className="discord-channel-option-name"><DiscordAvatar className="discord-member-avatar" avatarUrl={member.avatar_url} name={memberLabel(member)} /><span>{memberLabel(member)}<small>@{member.username ?? member.id}</small></span></span>
        {member.id === value && <Check size={17}/>} 
      </button>)}
      {!results.length && <p className="discord-member-empty">No known members match that search.</p>}
    </div>}
  </div>
}

function memberLabel(member: GuildMemberSummary) {
  return member.display_name || member.username || `Discord user ${member.id.slice(-4)}`
}

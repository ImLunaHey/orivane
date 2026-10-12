import { Check, ChevronDown, Shield, X } from 'lucide-react'
import { useEffect, useId, useMemo, useRef, useState } from 'react'
import type { GuildRole } from '../types'

type DiscordRoleSelectProps = {
  title: string
  detail: string
  values: string[]
  roles: GuildRole[]
  pending: boolean
  error: boolean
  max?: number
  onChange: (values: string[]) => void
}

export function DiscordRoleSelect({ title, detail, values, roles, pending, error, max = 100, onChange }: DiscordRoleSelectProps) {
  const id = useId()
  const rootRef = useRef<HTMLDivElement>(null)
  const [open, setOpen] = useState(false)
  const [query, setQuery] = useState('')
  const selected = roles.filter((role) => values.includes(role.id))
  const filtered = useMemo(() => roles.filter((role) => role.name.toLowerCase().includes(query.trim().toLowerCase())), [query, roles])
  const disabled = pending || error

  useEffect(() => {
    if (!open) return
    const closeOutside = (event: PointerEvent) => {
      if (!rootRef.current?.contains(event.target as Node)) setOpen(false)
    }
    document.addEventListener('pointerdown', closeOutside)
    return () => document.removeEventListener('pointerdown', closeOutside)
  }, [open])

  const toggle = (role: GuildRole) => {
    if (values.includes(role.id)) onChange(values.filter((id) => id !== role.id))
    else if (values.length < max) onChange([...values, role.id])
  }

  return <div className={`discord-role-field wide ${open ? 'is-open' : ''}`} ref={rootRef}>
    <span id={`${id}-label`}>{title}</span>
    <small id={`${id}-detail`}>{detail}</small>
    <button type="button" className="discord-channel-trigger" aria-labelledby={`${id}-label`} aria-describedby={`${id}-detail`} aria-haspopup="listbox" aria-expanded={open} disabled={disabled} onClick={() => setOpen((value) => !value)}>
      <span className={selected.length ? undefined : 'placeholder'}><Shield size={17} />{rolePlaceholder(selected.length, pending, error)}</span>
      <ChevronDown className="discord-channel-chevron" size={17} />
    </button>
    {selected.length > 0 && <div className="discord-role-chips" aria-label="Selected roles">{selected.map((role) => <button type="button" key={role.id} onClick={() => toggle(role)} aria-label={`Remove ${role.name}`}><Shield size={14} />{role.name}<X size={14} /></button>)}</div>}
    {open && <div className="discord-role-menu">
      <input autoFocus aria-label={`Search ${title.toLowerCase()}`} value={query} onChange={(event) => setQuery(event.target.value)} placeholder="Search server roles" />
      <div role="listbox" aria-multiselectable="true" aria-labelledby={`${id}-label`}>
        {filtered.map((role) => {
          const checked = values.includes(role.id)
          return <button type="button" role="option" aria-selected={checked} key={role.id} onClick={() => toggle(role)} disabled={!checked && values.length >= max}>
            <span><Shield size={16} />{role.name}</span>{checked && <Check size={17} />}
          </button>
        })}
        {!filtered.length && <p>No matching roles</p>}
      </div>
    </div>}
  </div>
}

function rolePlaceholder(count: number, pending: boolean, error: boolean) {
  if (pending) return 'Loading Discord roles…'
  if (error) return 'Could not load Discord roles'
  if (!count) return 'Select server roles'
  return `${count} role${count === 1 ? '' : 's'} selected`
}

import type { GuildChannel } from '../types'
import { DiscordChannelSelect } from './discord-channel-select'
import { Toggle } from './ui'

export type LoggingCategoryCardProps = {
  title: string
  detail: string
  enabled: boolean
  channelId?: string | null
  channels: GuildChannel[]
  pending: boolean
  error: boolean
  onEnabledChange: (enabled: boolean) => void
  onChannelChange?: (channelId: string | null) => void
}

export function LoggingCategoryCard({ title, detail, enabled, channelId, channels, pending, error, onEnabledChange, onChannelChange }: LoggingCategoryCardProps) {
  return <section className={`logging-category-card ${enabled ? 'logging-category-enabled' : ''}`}>
    <header><span><strong>{title}</strong><small>{detail}</small></span><Toggle label={`Log ${title.toLowerCase()}`} checked={enabled} onChange={onEnabledChange} /></header>
    {enabled && onChannelChange && <div className="logging-category-channel">
      <DiscordChannelSelect title={`${title} channel override`} detail="Optional; otherwise the default logging channel is used" value={channelId ?? ''} kind="text" channels={channels} pending={pending} error={error} onChange={(value) => onChannelChange(value || null)} />
      {channelId && <button type="button" className="text-link" onClick={() => onChannelChange(null)}>Use default channel</button>}
    </div>}
  </section>
}

type DiscordAvatarProps = {
  avatarUrl?: string | null
  className: string
  name: string
}

export function DiscordAvatar({ avatarUrl, className, name }: DiscordAvatarProps) {
  if (avatarUrl) {
    return <img className={`${className} discord-avatar-image`} src={avatarUrl} alt="" loading="lazy" referrerPolicy="no-referrer" />
  }

  const initials = name.trim().slice(0, 2).toUpperCase() || '??'
  return <span className={className} aria-hidden="true">{initials}</span>
}

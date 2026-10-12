import { Link } from '@tanstack/react-router'

export function DiscordMemberLink({ serverId, userId, username, displayName }: { serverId: string; userId: string; username?: string | null; displayName?: string | null }) {
  const name = displayName || username || `Discord user ${userId.slice(-4)}`
  return <Link className="discord-member-link" to="/servers/$serverId/members/$memberId" params={{ serverId, memberId: userId }}>
    <span className="case-avatar">{(username || name).slice(0, 2).toUpperCase()}</span>
    <span><strong>{name}</strong><small>{username ? `@${username}` : userId}</small></span>
  </Link>
}

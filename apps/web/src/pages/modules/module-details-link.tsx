import { Link } from '@tanstack/react-router'
import type { ComponentProps, ReactNode } from 'react'
import type { GuildConfig } from '../../types'

type ModuleKey = keyof GuildConfig['modules']
type Props = {
  serverId: string
  moduleId: ModuleKey
  children: ReactNode
  className?: ComponentProps<typeof Link>['className']
}

export function ModuleDetailsLink({ serverId, moduleId, children, className }: Props) {
  switch (moduleId) {
    case 'forms': return <Link className={className} to="/servers/$serverId/modules/forms" params={{ serverId }}>{children}</Link>
    case 'suggestions': return <Link className={className} to="/servers/$serverId/modules/suggestions" params={{ serverId }}>{children}</Link>
    case 'giveaways': return <Link className={className} to="/servers/$serverId/modules/giveaways" params={{ serverId }}>{children}</Link>
    case 'reminders': return <Link className={className} to="/servers/$serverId/modules/reminders" params={{ serverId }}>{children}</Link>
    case 'auto_delete': return <Link className={className} to="/servers/$serverId/modules/auto_delete" params={{ serverId }}>{children}</Link>
    case 'managed_embeds': return <Link className={className} to="/servers/$serverId/modules/managed_embeds" params={{ serverId }}>{children}</Link>
    case 'logging': return <Link className={className} to="/servers/$serverId/modules/logging" params={{ serverId }}>{children}</Link>
    case 'tickets': return <Link className={className} to="/servers/$serverId/modules/tickets" params={{ serverId }}>{children}</Link>
    case 'reaction_roles': return <Link className={className} to="/servers/$serverId/modules/reaction_roles" params={{ serverId }}>{children}</Link>
    case 'temporary_voice': return <Link className={className} to="/servers/$serverId/modules/temporary_voice" params={{ serverId }}>{children}</Link>
    case 'economy': return <Link className={className} to="/servers/$serverId/modules/economy" params={{ serverId }}>{children}</Link>
    default: return <Link className={className} to="/servers/$serverId/modules/$moduleId" params={{ serverId, moduleId }}>{children}</Link>
  }
}

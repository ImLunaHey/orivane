import { TanStackDevtools } from '@tanstack/react-devtools'
import { TanStackRouterDevtoolsPanel } from '@tanstack/react-router-devtools'
import { router } from '../router'

export function DocsDevtools() {
  return <TanStackDevtools plugins={[{ id: 'router', name: 'TanStack Router', render: <TanStackRouterDevtoolsPanel router={router} /> }]} />
}

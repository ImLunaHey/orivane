import { TanStackDevtools } from '@tanstack/react-devtools'
import { ReactQueryDevtoolsPanel } from '@tanstack/react-query-devtools'
import { TanStackRouterDevtoolsPanel } from '@tanstack/react-router-devtools'
import { router } from '../router'

export function DashboardDevtools() {
  return (
    <TanStackDevtools
      config={{ position: 'bottom-right', hideUntilHover: true }}
      plugins={[
        { id: 'query', name: 'TanStack Query', render: <ReactQueryDevtoolsPanel /> },
        { id: 'router', name: 'TanStack Router', render: <TanStackRouterDevtoolsPanel router={router} /> },
      ]}
    />
  )
}

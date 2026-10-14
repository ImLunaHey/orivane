import { createRootRoute, createRoute, createRouter, type RouterHistory } from '@tanstack/react-router'
import { DocsLayout } from './features/layout/docs-layout'
import { NotFoundPage } from './features/not-found/not-found-page'
import { HomePage } from './features/home/home-page'
import { GettingStartedPage } from './features/guides/getting-started-page'
import { McpPage } from './features/guides/mcp-page'
import { ModuleIndexPage } from './features/modules/module-index-page'
import { ModulePage } from './features/modules/module-page'
import { CommandReferencePage } from './features/reference/command-reference-page'
import { AuditReferencePage } from './features/reference/audit-reference-page'

const rootRoute = createRootRoute({ component: DocsLayout, notFoundComponent: NotFoundPage })
const indexRoute = createRoute({ getParentRoute: () => rootRoute, path: '/', component: HomePage })
const gettingStartedRoute = createRoute({ getParentRoute: () => rootRoute, path: '/guides/getting-started', component: GettingStartedPage })
const mcpRoute = createRoute({ getParentRoute: () => rootRoute, path: '/guides/mcp', component: McpPage })
const modulesRoute = createRoute({ getParentRoute: () => rootRoute, path: '/modules', component: ModuleIndexPage })
const moduleRoute = createRoute({ getParentRoute: () => rootRoute, path: '/modules/$moduleId', component: ModulePage })
const commandsRoute = createRoute({ getParentRoute: () => rootRoute, path: '/reference/commands', component: CommandReferencePage })
const auditRoute = createRoute({ getParentRoute: () => rootRoute, path: '/reference/audit', component: AuditReferencePage })

const routeTree = rootRoute.addChildren([indexRoute, gettingStartedRoute, mcpRoute, modulesRoute, moduleRoute, commandsRoute, auditRoute])
export function createDocsRouter(history?: RouterHistory) {
  return createRouter({ routeTree, basepath: '/docs', defaultPreload: 'intent', scrollRestoration: true, history })
}
export const router = createDocsRouter()

declare module '@tanstack/react-router' {
  interface Register { router: typeof router }
}

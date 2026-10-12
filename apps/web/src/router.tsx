import { createRootRoute, createRoute, createRouter, lazyRouteComponent } from '@tanstack/react-router'
import { AppShell } from './components/app-shell'
import { AuthGate } from './components/auth-gate'

const rootRoute = createRootRoute({ component: () => <AuthGate><AppShell /></AuthGate>, notFoundComponent: () => <div className="empty-state"><h2>Page not found</h2><a href="/">Return to bot overview</a></div> })

const indexRoute = createRoute({ getParentRoute: () => rootRoute, path: '/', component: lazyRouteComponent(() => import('./pages/bot-overview'), 'BotOverviewPage') })
const serversRoute = createRoute({ getParentRoute: () => rootRoute, path: '/servers', component: lazyRouteComponent(() => import('./pages/bot-overview'), 'ServersPage') })
const usersRoute = createRoute({ getParentRoute: () => rootRoute, path: '/users', component: lazyRouteComponent(() => import('./pages/users'), 'UsersPage') })
const userDetailRoute = createRoute({ getParentRoute: () => rootRoute, path: '/users/$userId', component: lazyRouteComponent(() => import('./pages/users'), 'UserDetailPage') })
const healthRoute = createRoute({ getParentRoute: () => rootRoute, path: '/health', component: lazyRouteComponent(() => import('./pages/system'), 'HealthPage') })
const commandsRoute = createRoute({ getParentRoute: () => rootRoute, path: '/commands', component: lazyRouteComponent(() => import('./pages/system'), 'CommandsPage') })
const globalAuditRoute = createRoute({ getParentRoute: () => rootRoute, path: '/audit', component: lazyRouteComponent(() => import('./pages/audit'), 'GlobalAuditPage') })
const globalAuditDetailRoute = createRoute({ getParentRoute: () => rootRoute, path: '/audit/$eventId', component: lazyRouteComponent(() => import('./pages/audit'), 'GlobalAuditDetailPage') })

const serverOverviewRoute = createRoute({ getParentRoute: () => rootRoute, path: '/servers/$serverId', component: lazyRouteComponent(() => import('./pages/overview'), 'OverviewPage') })
const modulesRoute = createRoute({ getParentRoute: () => rootRoute, path: '/servers/$serverId/modules', component: lazyRouteComponent(() => import('./pages/modules'), 'ModulesPage') })
const formsRoute = createRoute({ getParentRoute: () => rootRoute, path: '/servers/$serverId/modules/forms', component: lazyRouteComponent(() => import('./pages/forms'), 'FormsPage') })
const formDetailRoute = createRoute({ getParentRoute: () => rootRoute, path: '/servers/$serverId/modules/forms/$formId', component: lazyRouteComponent(() => import('./pages/forms'), 'FormDetailPage') })
const formSubmissionsRoute = createRoute({ getParentRoute: () => rootRoute, path: '/servers/$serverId/modules/forms/$formId/submissions', component: lazyRouteComponent(() => import('./pages/forms'), 'FormSubmissionsPage') })
const formSubmissionRoute = createRoute({ getParentRoute: () => rootRoute, path: '/servers/$serverId/modules/forms/$formId/submissions/$submissionId', component: lazyRouteComponent(() => import('./pages/forms'), 'FormSubmissionPage') })
const suggestionsRoute = createRoute({ getParentRoute: () => rootRoute, path: '/servers/$serverId/modules/suggestions', component: lazyRouteComponent(() => import('./pages/suggestions'), 'SuggestionsPage') })
const suggestionDetailRoute = createRoute({ getParentRoute: () => rootRoute, path: '/servers/$serverId/modules/suggestions/$suggestionId', component: lazyRouteComponent(() => import('./pages/suggestions'), 'SuggestionDetailPage') })
const giveawaysRoute = createRoute({ getParentRoute: () => rootRoute, path: '/servers/$serverId/modules/giveaways', component: lazyRouteComponent(() => import('./pages/giveaways'), 'GiveawaysPage') })
const giveawayCreateRoute = createRoute({ getParentRoute: () => rootRoute, path: '/servers/$serverId/modules/giveaways/new', component: lazyRouteComponent(() => import('./pages/giveaways'), 'GiveawayCreatePage') })
const giveawayDetailRoute = createRoute({ getParentRoute: () => rootRoute, path: '/servers/$serverId/modules/giveaways/$giveawayId', component: lazyRouteComponent(() => import('./pages/giveaways'), 'GiveawayDetailPage') })
const remindersRoute = createRoute({ getParentRoute: () => rootRoute, path: '/servers/$serverId/modules/reminders', component: lazyRouteComponent(() => import('./pages/reminders'), 'RemindersPage') })
const reminderDetailRoute = createRoute({ getParentRoute: () => rootRoute, path: '/servers/$serverId/modules/reminders/$reminderId', component: lazyRouteComponent(() => import('./pages/reminders'), 'ReminderDetailPage') })
const moduleDetailRoute = createRoute({ getParentRoute: () => rootRoute, path: '/servers/$serverId/modules/$moduleId', component: lazyRouteComponent(() => import('./pages/modules'), 'ModuleDetailPage') })
const moderationRoute = createRoute({ getParentRoute: () => rootRoute, path: '/servers/$serverId/moderation', component: lazyRouteComponent(() => import('./pages/moderation'), 'ModerationPage') })
const moderationCaseRoute = createRoute({ getParentRoute: () => rootRoute, path: '/servers/$serverId/moderation/$caseId', component: lazyRouteComponent(() => import('./pages/moderation'), 'ModerationCasePage') })
const analyticsRoute = createRoute({ getParentRoute: () => rootRoute, path: '/servers/$serverId/analytics', component: lazyRouteComponent(() => import('./pages/analytics'), 'AnalyticsPage') })
const commandAnalyticsRoute = createRoute({ getParentRoute: () => rootRoute, path: '/servers/$serverId/analytics/commands', component: lazyRouteComponent(() => import('./pages/analytics'), 'CommandsAnalyticsPage') })
const latencyAnalyticsRoute = createRoute({ getParentRoute: () => rootRoute, path: '/servers/$serverId/analytics/latency', component: lazyRouteComponent(() => import('./pages/analytics'), 'LatencyAnalyticsPage') })
const automodAnalyticsRoute = createRoute({ getParentRoute: () => rootRoute, path: '/servers/$serverId/analytics/automod', component: lazyRouteComponent(() => import('./pages/analytics'), 'AutomodAnalyticsPage') })
const membersRoute = createRoute({ getParentRoute: () => rootRoute, path: '/servers/$serverId/members', component: lazyRouteComponent(() => import('./pages/system'), 'MembersPage') })
const memberDetailRoute = createRoute({ getParentRoute: () => rootRoute, path: '/servers/$serverId/members/$memberId', component: lazyRouteComponent(() => import('./pages/system'), 'MemberDetailPage') })
const serverAuditRoute = createRoute({ getParentRoute: () => rootRoute, path: '/servers/$serverId/audit', component: lazyRouteComponent(() => import('./pages/audit'), 'ServerAuditPage') })
const serverAuditDetailRoute = createRoute({ getParentRoute: () => rootRoute, path: '/servers/$serverId/audit/$eventId', component: lazyRouteComponent(() => import('./pages/audit'), 'ServerAuditDetailPage') })

const routeTree = rootRoute.addChildren([
  indexRoute,
  serversRoute,
  usersRoute,
  userDetailRoute,
  healthRoute,
  commandsRoute,
  globalAuditRoute,
  globalAuditDetailRoute,
  serverOverviewRoute,
  modulesRoute,
  formsRoute,
  formDetailRoute,
  formSubmissionsRoute,
  formSubmissionRoute,
  suggestionsRoute,
  suggestionDetailRoute,
  giveawaysRoute,
  giveawayCreateRoute,
  giveawayDetailRoute,
  remindersRoute,
  reminderDetailRoute,
  moduleDetailRoute,
  moderationRoute,
  moderationCaseRoute,
  analyticsRoute,
  commandAnalyticsRoute,
  latencyAnalyticsRoute,
  automodAnalyticsRoute,
  membersRoute,
  memberDetailRoute,
  serverAuditRoute,
  serverAuditDetailRoute,
])

export const router = createRouter({ routeTree, defaultPreload: 'intent' })

declare module '@tanstack/react-router' {
  interface Register { router: typeof router }
}

export type UserServerContext = {
  serverId: string
  serverName: string
  relationship: 'Installer' | 'Administrator' | 'Moderator' | 'Member'
  commands: number
  cases: number
  lastSeen: string
}

export type GlobalUser = {
  id: string
  username: string
  displayName: string
  avatarUrl?: string | null
  kind: 'Installer' | 'Member'
  servers: number
  installedServers: number
  installCommands30d: number
  commands: number
  cases: number
  lastSeen: string
  accountAge: string
  contexts: UserServerContext[]
}

export const globalUsers: GlobalUser[] = [
  { id: '312058643210567680', username: 'lunahey', displayName: 'Luna', kind: 'Installer', servers: 3, installedServers: 2, installCommands30d: 18_420, commands: 128, cases: 0, lastSeen: 'Online now', accountAge: '8 years', contexts: [
    { serverId: '588456789012345678', serverName: 'Aurora Network', relationship: 'Installer', commands: 41, cases: 0, lastSeen: '12 minutes ago' },
    { serverId: '901234567890123456', serverName: "Luna's studio", relationship: 'Installer', commands: 79, cases: 0, lastSeen: 'Online now' },
    { serverId: '827345678901234567', serverName: 'Orbit Labs', relationship: 'Administrator', commands: 8, cases: 0, lastSeen: 'Yesterday' },
  ] },
  { id: '442169754321678791', username: 'moss.dev', displayName: 'Moss', kind: 'Installer', servers: 2, installedServers: 2, installCommands30d: 3_806, commands: 318, cases: 0, lastSeen: 'Online now', accountAge: '6 years', contexts: [
    { serverId: '649567890123456789', serverName: 'Moss Garden', relationship: 'Installer', commands: 244, cases: 0, lastSeen: 'Online now' },
    { serverId: '639567890123456780', serverName: 'Moss Archive', relationship: 'Installer', commands: 74, cases: 0, lastSeen: '3 hours ago' },
  ] },
  { id: '572270865432789802', username: 'orbital-admin', displayName: 'Orbital', kind: 'Installer', servers: 2, installedServers: 2, installCommands30d: 7_214, commands: 196, cases: 0, lastSeen: '34 minutes ago', accountAge: '5 years', contexts: [
    { serverId: '827345678901234567', serverName: 'Orbit Labs', relationship: 'Installer', commands: 157, cases: 0, lastSeen: '34 minutes ago' },
    { serverId: '817345678901234568', serverName: 'Orbit Community', relationship: 'Installer', commands: 39, cases: 0, lastSeen: '2 hours ago' },
  ] },
  { id: '682381976543890913', username: 'nightops', displayName: 'Night Ops', kind: 'Installer', servers: 1, installedServers: 1, installCommands30d: 2_109, commands: 84, cases: 0, lastSeen: '2 hours ago', accountAge: '4 years', contexts: [
    { serverId: '763456789012345678', serverName: 'Night Shift', relationship: 'Installer', commands: 84, cases: 0, lastSeen: '2 hours ago' },
  ] },
  { id: '118742006320914432', username: 'silver.arc', displayName: 'Silver', kind: 'Member', servers: 3, installedServers: 0, installCommands30d: 0, commands: 67, cases: 1, lastSeen: '4 minutes ago', accountAge: '4 years', contexts: [
    { serverId: '901234567890123456', serverName: "Luna's studio", relationship: 'Member', commands: 23, cases: 1, lastSeen: '4 minutes ago' },
    { serverId: '588456789012345678', serverName: 'Aurora Network', relationship: 'Member', commands: 38, cases: 0, lastSeen: '1 hour ago' },
    { serverId: '817345678901234568', serverName: 'Orbit Community', relationship: 'Member', commands: 6, cases: 0, lastSeen: '6 days ago' },
  ] },
  { id: '227843117431025543', username: 'nocturne', displayName: 'Nocturne', kind: 'Member', servers: 2, installedServers: 0, installCommands30d: 0, commands: 29, cases: 2, lastSeen: '2 hours ago', accountAge: '2 years', contexts: [
    { serverId: '901234567890123456', serverName: "Luna's studio", relationship: 'Member', commands: 12, cases: 2, lastSeen: '2 hours ago' },
    { serverId: '763456789012345678', serverName: 'Night Shift', relationship: 'Member', commands: 17, cases: 0, lastSeen: 'Yesterday' },
  ] },
  { id: '336954228542136654', username: 'mossbyte', displayName: 'Mossbyte', kind: 'Member', servers: 4, installedServers: 0, installCommands30d: 0, commands: 462, cases: 0, lastSeen: 'Online now', accountAge: '6 years', contexts: [
    { serverId: '901234567890123456', serverName: "Luna's studio", relationship: 'Moderator', commands: 318, cases: 0, lastSeen: 'Online now' },
    { serverId: '649567890123456789', serverName: 'Moss Garden', relationship: 'Administrator', commands: 91, cases: 0, lastSeen: '20 minutes ago' },
  ] },
]

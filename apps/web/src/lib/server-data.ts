export type ConnectedServer = {
  id: string
  name: string
  members: number
  commands: number
  status: 'Healthy' | 'Degraded' | 'Connected'
  shard: number
  installer: string
  installerId: string
  ownerId?: string
  installedAt?: string
}

export const demoServers: ConnectedServer[] = [
  { id: '588456789012345678', name: 'Aurora Network', members: 54_208, commands: 6_910, status: 'Healthy', shard: 1, installer: 'lunahey', installerId: '312058643210567680' },
  { id: '901234567890123456', name: "Luna's studio", members: 12_691, commands: 3_842, status: 'Healthy', shard: 0, installer: 'lunahey', installerId: '312058643210567680' },
  { id: '827345678901234567', name: 'Orbit Labs', members: 8_204, commands: 2_117, status: 'Healthy', shard: 1, installer: 'orbital-admin', installerId: '572270865432789802' },
  { id: '817345678901234568', name: 'Orbit Community', members: 1_278, commands: 286, status: 'Healthy', shard: 1, installer: 'orbital-admin', installerId: '572270865432789802' },
  { id: '763456789012345678', name: 'Night Shift', members: 3_841, commands: 986, status: 'Healthy', shard: 2, installer: 'nightops', installerId: '682381976543890913' },
  { id: '649567890123456789', name: 'Moss Garden', members: 1_955, commands: 421, status: 'Degraded', shard: 3, installer: 'moss.dev', installerId: '442169754321678791' },
  { id: '639567890123456780', name: 'Moss Archive', members: 612, commands: 94, status: 'Healthy', shard: 3, installer: 'moss.dev', installerId: '442169754321678791' },
]

import { beforeEach, describe, expect, it } from 'vitest'

import {
  createModerationCase,
  getAuditEvent,
  getAuditEvents,
  getCommandVolume,
  getGuildChannels,
  getGuildConfig,
  getGuildMembers,
  getGuildStats,
  getHealthReport,
  getModerationCase,
  getModerationCases,
  getServers,
  getUser,
  getUserAuditEvents,
  getUsers,
  isDemoMode,
  recordDashboardActivity,
  saveGuildConfig,
  syncCommands,
} from './api'

describe('dashboard demo configuration', () => {
  beforeEach(() => localStorage.clear())

  it('loads defaults and persists changes without a backend', async () => {
    expect(isDemoMode).toBe(true)
    const config = await getGuildConfig('guild-test')
    expect(config.guild_id).toBe('guild-test')
    config.modules.leveling.enabled = false
    await saveGuildConfig(config)
    expect((await getGuildConfig('guild-test')).modules.leveling.enabled).toBe(false)
    expect((await getAuditEvents('guild-test'))[0]?.action).toBe('guild.config.updated')
  })

  it('provides representative Discord, fleet, user, and telemetry data', async () => {
    expect(await getGuildChannels('guild-test')).toEqual(expect.arrayContaining([
      expect.objectContaining({ name: 'welcome', kind: 'GuildText' }),
      expect.objectContaining({ name: 'Information', kind: 'GuildCategory' }),
    ]))
    expect((await getServers()).length).toBeGreaterThan(1)
    expect((await getUsers())[0]).toHaveProperty('contexts')
    expect((await getUser('312058643210567680')).username).toBe('lunahey')
    await expect(getUser('missing')).rejects.toThrow('Global user not found')
    expect(await getGuildStats('guild-test')).toMatchObject({ commands_today: 3842, p95_latency_ms: 94 })
    expect((await getCommandVolume('guild-test')).length).toBeGreaterThan(0)
    expect(await getHealthReport()).toMatchObject({ status: 'operational' })
    expect(await getGuildMembers('guild-test')).toEqual([])
  })

  it('supports moderation cases and reports missing records', async () => {
    const original = await getModerationCases('guild-test')
    expect(original).toHaveLength(2)
    expect((await getModerationCase('guild-test', '187')).action).toBe('warn')
    await expect(getModerationCase('guild-test', '404')).rejects.toThrow('Moderation case not found')

    const created = await createModerationCase('guild-test', {
      member_id: 'member-1',
      action: 'timeout',
      reason: 'Test reason',
    })
    expect(created.id).toBe(188)
    expect((await getModerationCases('guild-test'))[0]).toEqual(created)
  })

  it('records dashboard actions, syncs commands, and resolves audit details', async () => {
    await recordDashboardActivity({ action: 'dashboard.page.viewed', guild_id: 'guild-test', path: '/test' })
    await recordDashboardActivity({ action: 'dashboard.data.exported', export: 'events.csv' })
    expect(await syncCommands()).toEqual({ synchronized: 3 })

    const events = await getAuditEvents()
    expect(events.some((event) => event.action === 'dashboard.page.viewed')).toBe(true)
    expect(events.some((event) => event.action === 'commands.synchronized')).toBe(true)
    expect((await getAuditEvent('1042')).id).toBe(1042)
    const userEvents = await getUserAuditEvents('227843117431025543')
    expect(userEvents.every((event) => event.actor_id === '227843117431025543' || event.target_id === '227843117431025543')).toBe(true)
    expect(userEvents.some((event) => event.target_id === '227843117431025543')).toBe(true)
    await expect(getAuditEvent('does-not-exist')).rejects.toThrow('Audit event not found')
  })
})

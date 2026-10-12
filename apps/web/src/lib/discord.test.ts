import { describe, expect, it } from 'vitest'
import { discordAccountAge, discordAccountCreatedAt } from './discord'

describe('Discord snowflakes', () => {
  it('derives the account creation timestamp from the Discord epoch', () => {
    expect(discordAccountCreatedAt('175928847299117063')?.toISOString()).toBe('2016-04-30T11:18:25.796Z')
  })

  it('formats account age and rejects malformed identifiers', () => {
    expect(discordAccountAge('175928847299117063', new Date('2026-04-30T12:00:00Z'))).toBe('10 years')
    expect(discordAccountCreatedAt('not-a-snowflake')).toBeNull()
    expect(discordAccountAge('not-a-snowflake')).toBe('Unavailable')
  })
})

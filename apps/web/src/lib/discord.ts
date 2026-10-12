const DISCORD_EPOCH_MS = 1_420_070_400_000n

export function discordAccountCreatedAt(id: string): Date | null {
  if (!/^\d{17,20}$/.test(id)) return null
  const milliseconds = (BigInt(id) >> 22n) + DISCORD_EPOCH_MS
  const createdAt = new Date(Number(milliseconds))
  return Number.isNaN(createdAt.getTime()) ? null : createdAt
}

export function discordAccountAge(id: string, now = new Date()): string {
  const createdAt = discordAccountCreatedAt(id)
  if (!createdAt) return 'Unavailable'
  const beforeAnniversary = now.getUTCMonth() < createdAt.getUTCMonth()
    || (now.getUTCMonth() === createdAt.getUTCMonth() && now.getUTCDate() < createdAt.getUTCDate())
  const years = Math.max(0, now.getUTCFullYear() - createdAt.getUTCFullYear() - Number(beforeAnniversary))
  if (years) return `${years} year${years === 1 ? '' : 's'}`
  const beforeMonthDay = now.getUTCDate() < createdAt.getUTCDate()
  const months = Math.max(0, (now.getUTCFullYear() - createdAt.getUTCFullYear()) * 12 + now.getUTCMonth() - createdAt.getUTCMonth() - Number(beforeMonthDay))
  return `${months} month${months === 1 ? '' : 's'}`
}

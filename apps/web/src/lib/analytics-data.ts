export type ChartDatum = {
  label: string
  value: number
  detail: string
}

export const hourlyCommands: ChartDatum[] = [
  { label: '00:00', value: 38, detail: 'Quiet period · 99.9% successful' },
  { label: '02:00', value: 45, detail: 'Mostly utility commands' },
  { label: '04:00', value: 64, detail: 'Leveling activity increased' },
  { label: '06:00', value: 96, detail: 'Morning activity started' },
  { label: '08:00', value: 112, detail: 'Peak /rank usage' },
  { label: '10:00', value: 129, detail: '8 moderation commands' },
  { label: '12:00', value: 145, detail: '99.7% successful' },
  { label: '14:00', value: 166, detail: 'Community event started' },
  { label: '16:00', value: 184, detail: '23 commands per minute peak' },
  { label: '18:00', value: 194, detail: 'Highest moderation activity' },
  { label: '20:00', value: 213, detail: 'Evening traffic peak' },
  { label: 'Now', value: 241, detail: 'Current two-hour window' },
]

export const commandUsage: ChartDatum[] = [
  { label: '/rank', value: 1284, detail: '33.4% of command traffic' },
  { label: '/profile', value: 817, detail: '21.3% of command traffic' },
  { label: '/ping', value: 642, detail: '16.7% of command traffic' },
  { label: '/warn', value: 418, detail: '10.9% of command traffic' },
  { label: '/timeout', value: 376, detail: '9.8% of command traffic' },
  { label: '/settings', value: 305, detail: '7.9% of command traffic' },
]

export const latencyTrend: ChartDatum[] = [
  { label: '00:00', value: 62, detail: 'p95 · REST latency 24 ms' },
  { label: '03:00', value: 57, detail: 'p95 · no retries' },
  { label: '06:00', value: 71, detail: 'p95 · 2 requests retried' },
  { label: '09:00', value: 83, detail: 'p95 · database peak 11 ms' },
  { label: '12:00', value: 78, detail: 'p95 · cache hit rate 98.7%' },
  { label: '15:00', value: 91, detail: 'p95 · Discord REST elevated' },
  { label: '18:00', value: 104, detail: 'p95 · busiest period' },
  { label: '21:00', value: 94, detail: 'p95 · current window' },
]

export const automodActions: ChartDatum[] = [
  { label: 'Spam', value: 74, detail: '39.6% · 68 messages removed' },
  { label: 'Mentions', value: 46, detail: '24.6% · 12 members timed out' },
  { label: 'Links', value: 31, detail: '16.6% · 4 domains blocked' },
  { label: 'Terms', value: 22, detail: '11.8% · content not retained' },
  { label: 'Caps', value: 14, detail: '7.5% · warnings only' },
]

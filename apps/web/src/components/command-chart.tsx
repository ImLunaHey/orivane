import type { ChartDatum } from '../lib/analytics-data'
import { MetricChart } from './metric-chart'

export function CommandChart({ data }: { data: readonly ChartDatum[] }) {
  return <MetricChart data={data} xLabel="Time (UTC)" yLabel="Commands per 4 hours" ariaLabel="Command volume over the last 24 hours" />
}

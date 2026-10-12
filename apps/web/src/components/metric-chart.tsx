import type { ChartDatum } from '../lib/analytics-data'
import { BarMetricChart } from './metric-chart/bar-metric-chart'
import { LineMetricChart } from './metric-chart/line-metric-chart'

export type MetricChartProps = {
  data: readonly ChartDatum[]
  xLabel: string
  yLabel: string
  ariaLabel: string
  unit?: string
  type?: 'line' | 'bar'
  color?: string
}

/**
 * The dashboard's single chart entry point. Required props ensure every chart
 * has named axes, an accessible description, and detailed tooltip data.
 */
export function MetricChart(props: MetricChartProps) {
  if (props.data.length === 0) {
    return <output className="chart-empty"><strong>No telemetry yet</strong><span>Live audited events will appear here as Orivane is used.</span></output>
  }
  return props.type === 'bar'
    ? <BarMetricChart {...props} />
    : <LineMetricChart {...props} />
}

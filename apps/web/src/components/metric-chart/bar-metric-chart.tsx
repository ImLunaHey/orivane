import { barY, defineChart } from '@tanstack/charts'
import { Chart } from '@tanstack/charts/react'
import { useMemo } from 'react'
import type { MetricChartProps } from '../metric-chart'
import { chartScales, chartTheme, chartTooltip } from './chart-config'

export function BarMetricChart({ data, xLabel, yLabel, unit = '', color = '#38bdf8', ariaLabel }: MetricChartProps) {
  const definition = useMemo(() => defineChart({ marks: [barY(data, { x: 'label', y: 'value', fill: color, fillOpacity: 0.82, inset: 3, maxThickness: 72 })], scales: chartScales(xLabel, yLabel, unit, 'band'), margin: { top: 20, right: 20, bottom: 58, left: 70 }, theme: chartTheme(color), focus: 'nearest-x', tooltip: chartTooltip(yLabel, unit) }), [color, data, unit, xLabel, yLabel])
  return <Chart definition={definition} height={310} ariaLabel={ariaLabel} className="metric-chart" />
}

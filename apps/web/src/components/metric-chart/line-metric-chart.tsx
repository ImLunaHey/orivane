import { defineChart, lineY } from '@tanstack/charts'
import { Chart } from '@tanstack/charts/react'
import { useMemo } from 'react'
import type { MetricChartProps } from '../metric-chart'
import { chartScales, chartTheme, chartTooltip } from './chart-config'

export function LineMetricChart({ data, xLabel, yLabel, unit = '', color = '#9b76f6', ariaLabel }: MetricChartProps) {
  const definition = useMemo(() => defineChart({ marks: [lineY(data, { x: 'label', y: 'value', stroke: color, strokeWidth: 2.5, points: true })], scales: chartScales(xLabel, yLabel, unit, 'point'), margin: { top: 20, right: 20, bottom: 58, left: 70 }, theme: chartTheme(color), focus: 'nearest-x', tooltip: chartTooltip(yLabel, unit) }), [color, data, unit, xLabel, yLabel])
  return <Chart definition={definition} height={310} ariaLabel={ariaLabel} className="metric-chart" />
}

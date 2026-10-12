import { barY, defineChart, lineY } from '@tanstack/charts'
import { scaleBand } from '@tanstack/charts/scales/band'
import { scaleLinear } from '@tanstack/charts/scales/linear'
import { scalePoint } from '@tanstack/charts/scales/point'
import { tooltip } from '@tanstack/charts/tooltip'
import { Chart } from '@tanstack/charts/react'
import { useMemo } from 'react'
import type { ChartDatum } from '../lib/analytics-data'

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

function LineMetricChart({ data, xLabel, yLabel, unit = '', color = '#9b76f6', ariaLabel }: MetricChartProps) {
  const definition = useMemo(() => defineChart({
    marks: [lineY(data, {
      x: 'label',
      y: 'value',
      stroke: color,
      strokeWidth: 2.5,
      points: true,
    })],
    scales: chartScales(xLabel, yLabel, unit, 'point'),
    margin: { top: 20, right: 20, bottom: 58, left: 70 },
    theme: chartTheme(color),
    focus: 'nearest-x',
    tooltip: chartTooltip(yLabel, unit),
  }), [color, data, unit, xLabel, yLabel])

  return <Chart definition={definition} height={310} ariaLabel={ariaLabel} className="metric-chart" />
}

function BarMetricChart({ data, xLabel, yLabel, unit = '', color = '#38bdf8', ariaLabel }: MetricChartProps) {
  const definition = useMemo(() => defineChart({
    marks: [barY(data, {
      x: 'label',
      y: 'value',
      fill: color,
      fillOpacity: 0.82,
      inset: 3,
      maxThickness: 72,
    })],
    scales: chartScales(xLabel, yLabel, unit, 'band'),
    margin: { top: 20, right: 20, bottom: 58, left: 70 },
    theme: chartTheme(color),
    focus: 'nearest-x',
    tooltip: chartTooltip(yLabel, unit),
  }), [color, data, unit, xLabel, yLabel])

  return <Chart definition={definition} height={310} ariaLabel={ariaLabel} className="metric-chart" />
}

function chartScales(xLabel: string, yLabel: string, unit: string, xScale: 'point' | 'band') {
  return {
    x: {
      scale: xScale === 'point' ? () => scalePoint().padding(0.2) : () => scaleBand().padding(0.18),
      axis: { label: { text: xLabel, offset: 42, fontSize: 14 }, tickLabels: { fontSize: 14 } },
    },
    y: {
      scale: scaleLinear,
      nice: true,
      grid: { stroke: '#2a2a30', strokeOpacity: 1 },
      axis: {
        label: { text: yLabel, offset: 52, fontSize: 14 },
        tickLabels: { fontSize: 14, format: (value: number) => `${formatValue(value)}${unit}` },
      },
    },
  }
}

function chartTooltip(yLabel: string, unit: string) {
  return {
    use: tooltip,
    placement: 'top' as const,
    sticky: true,
    content: (points: ReadonlyArray<{ datum: ChartDatum }>) => {
      const point = points[0]
      if (!point) return { title: 'No data', rows: [] }
      return {
        title: point.datum.label,
        rows: [
          { label: yLabel, value: `${formatValue(point.datum.value)}${unit}` },
          { label: 'Context', value: point.datum.detail },
        ],
      }
    },
  }
}

function chartTheme(color: string) {
  return {
    foreground: '#a1a1aa',
    muted: '#a0a0aa',
    grid: '#2a2a30',
    background: 'transparent',
    palette: [color],
    focusRing: { stroke: '#f4f4f5', fill: color, strokeWidth: 2 },
  }
}

function formatValue(value: number) {
  return value >= 1000 ? `${(value / 1000).toFixed(value % 1000 === 0 ? 0 : 1)}k` : value.toString()
}

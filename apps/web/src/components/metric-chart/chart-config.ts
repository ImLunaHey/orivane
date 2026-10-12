import { scaleBand } from '@tanstack/charts/scales/band'
import { scaleLinear } from '@tanstack/charts/scales/linear'
import { scalePoint } from '@tanstack/charts/scales/point'
import { tooltip } from '@tanstack/charts/tooltip'
import type { ChartDatum } from '../../lib/analytics-data'

export function chartScales(xLabel: string, yLabel: string, unit: string, xScale: 'point' | 'band') {
  return { x: { scale: xScale === 'point' ? () => scalePoint().padding(0.2) : () => scaleBand().padding(0.18), axis: { label: { text: xLabel, offset: 42, fontSize: 14 }, tickLabels: { fontSize: 14 } } }, y: { scale: scaleLinear, nice: true, grid: { stroke: '#2a2a30', strokeOpacity: 1 }, axis: { label: { text: yLabel, offset: 52, fontSize: 14 }, tickLabels: { fontSize: 14, format: (value: number) => `${formatValue(value)}${unit}` } } } }
}

export function chartTooltip(yLabel: string, unit: string) {
  return { use: tooltip, placement: 'top' as const, sticky: true, content: (points: ReadonlyArray<{ datum: ChartDatum }>) => { const point = points[0]; return point ? { title: point.datum.label, rows: [{ label: yLabel, value: `${formatValue(point.datum.value)}${unit}` }, { label: 'Context', value: point.datum.detail }] } : { title: 'No data', rows: [] } } }
}

export function chartTheme(color: string) {
  return { foreground: '#a1a1aa', muted: '#a0a0aa', grid: '#2a2a30', background: 'transparent', palette: [color], focusRing: { stroke: '#f4f4f5', fill: color, strokeWidth: 2 } }
}

function formatValue(value: number) { return value >= 1000 ? `${(value / 1000).toFixed(value % 1000 === 0 ? 0 : 1)}k` : value.toString() }

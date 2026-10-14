import type { Dispatch, SetStateAction } from 'react'

export function NumericInput({ label, value, setValue, min, max, range = false }: { label: string; value: number; setValue: Dispatch<SetStateAction<number>>; min: number; max?: number; range?: boolean }) {
  return <label><span>{label}</span><input type={range ? 'range' : 'number'} min={min} max={max} value={value} onChange={(event) => {
    const next = event.currentTarget.valueAsNumber
    if (Number.isFinite(next)) setValue(next)
  }} /></label>
}

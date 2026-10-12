import { createContext, useContext } from 'react'

export const PaletteContext = createContext<(() => void) | null>(null)

export function useCommandPalette() {
  const open = useContext(PaletteContext)
  if (!open) throw new Error('useCommandPalette must be used inside the application shell')
  return open
}

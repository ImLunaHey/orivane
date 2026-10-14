import '@testing-library/jest-dom/vitest'
import { vi } from 'vitest'

Object.defineProperty(navigator, 'clipboard', {
  configurable: true,
  value: { writeText: vi.fn<(text: string) => Promise<void>>().mockResolvedValue(undefined) },
})

Object.defineProperty(window, 'scrollTo', { configurable: true, value: vi.fn<(options?: ScrollToOptions | number, y?: number) => void>() })

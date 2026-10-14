import '@testing-library/jest-dom/vitest'
import { vi } from 'vitest'

vi.mock('../lib/auth-client', () => ({
  authClient: {
    useSession: () => ({
      data: null,
      error: null,
      isPending: false,
    }),
    signIn: { social: vi.fn() },
    signOut: vi.fn(),
  },
}))

window.scrollTo = () => undefined

Object.defineProperties(window, {
  requestAnimationFrame: {
    configurable: true,
    value: (callback: FrameRequestCallback) => window.setTimeout(() => callback(performance.now()), 0),
  },
  cancelAnimationFrame: {
    configurable: true,
    value: (handle: number) => window.clearTimeout(handle),
  },
})

Object.defineProperty(HTMLCanvasElement.prototype, 'getContext', {
  configurable: true,
  value: () => null,
})

if (!Range.prototype.getClientRects) {
  Range.prototype.getClientRects = () => ({
    length: 0,
    item: () => null,
    [Symbol.iterator]: function* () {},
  }) as DOMRectList
}

if (!Range.prototype.getBoundingClientRect) {
  Range.prototype.getBoundingClientRect = () => new DOMRect(0, 0, 0, 0)
}

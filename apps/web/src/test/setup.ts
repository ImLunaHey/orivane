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

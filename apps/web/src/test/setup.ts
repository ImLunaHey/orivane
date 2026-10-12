import '@testing-library/jest-dom/vitest'

window.scrollTo = () => undefined

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

import { describe, expect, it } from 'vitest'
import { isPointerOverPanelRect } from './toolbarPanelHover'

describe('isPointerOverPanelRect', () => {
  const rect = { left: 0, top: 0, right: 304, bottom: 400 }

  it('returns true when pointer is inside panel in screen space', () => {
    expect(isPointerOverPanelRect(150, 200, 100, 50, rect)).toBe(true)
  })

  it('returns false when pointer is outside panel', () => {
    expect(isPointerOverPanelRect(500, 200, 100, 50, rect)).toBe(false)
  })

  it('returns true on panel edges', () => {
    expect(isPointerOverPanelRect(100, 50, 100, 50, rect)).toBe(true)
    expect(isPointerOverPanelRect(404, 450, 100, 50, rect)).toBe(true)
  })
})

import { describe, expect, it } from 'vitest'
import { canStartElementDrag } from './dragInteraction'

describe('canStartElementDrag', () => {
  const base = {
    dragMode: 'hover' as const,
    hasHoveredElement: true,
    modifierDown: false,
  }

  it('returns false when drag mode is off', () => {
    expect(canStartElementDrag({ ...base, dragMode: 'off' })).toBe(false)
  })

  it('returns false when not over an element', () => {
    expect(canStartElementDrag({ ...base, hasHoveredElement: false })).toBe(false)
  })

  it('allows hover drag in hover mode', () => {
    expect(canStartElementDrag({ ...base, modifierDown: false })).toBe(true)
  })

  it('requires modifier in modifier mode', () => {
    expect(canStartElementDrag({ ...base, dragMode: 'modifier', modifierDown: false })).toBe(false)
    expect(canStartElementDrag({ ...base, dragMode: 'modifier', modifierDown: true })).toBe(true)
  })
})

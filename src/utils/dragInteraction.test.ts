import { describe, expect, it } from 'vitest'
import { canStartElementDrag, resolveHoverDragGate } from './dragInteraction'

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

describe('resolveHoverDragGate', () => {
  it('waits while the pointer is held still under the activation delay', () => {
    expect(resolveHoverDragGate({ heldMs: 0, movedPx: 0 })).toBe('wait')
    expect(resolveHoverDragGate({ heldMs: 199, movedPx: 0 })).toBe('wait')
  })

  it('activates the drag once held for 200ms without moving', () => {
    expect(resolveHoverDragGate({ heldMs: 200, movedPx: 0 })).toBe('drag')
    expect(resolveHoverDragGate({ heldMs: 800, movedPx: 3 })).toBe('drag')
  })

  it('treats a quick drag away as drawing intent (mis-touch protection)', () => {
    expect(resolveHoverDragGate({ heldMs: 0, movedPx: 6 })).toBe('draw')
    expect(resolveHoverDragGate({ heldMs: 120, movedPx: 20 })).toBe('draw')
  })

  it('respects custom thresholds', () => {
    expect(resolveHoverDragGate({ heldMs: 90, movedPx: 0, thresholdMs: 100 })).toBe('wait')
    expect(resolveHoverDragGate({ heldMs: 100, movedPx: 0, thresholdMs: 100 })).toBe('drag')
    expect(resolveHoverDragGate({ heldMs: 0, movedPx: 4, thresholdPx: 4 })).toBe('wait')
    expect(resolveHoverDragGate({ heldMs: 0, movedPx: 5, thresholdPx: 4 })).toBe('draw')
  })
})

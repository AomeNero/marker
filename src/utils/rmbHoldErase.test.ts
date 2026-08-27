import { describe, expect, it } from 'vitest'
import { IDLE_RMB_ERASE, beginRmbErase, canStartRmbErase, endRmbErase } from './rmbHoldErase'

describe('rmbHoldErase (immediate RMB erase)', () => {
  it('canStart is false when text box open or penetrating', () => {
    expect(
      canStartRmbErase({
        active: true,
        penetration: false,
        textBoxOpen: true,
      }),
    ).toBe(false)
    expect(
      canStartRmbErase({
        active: true,
        penetration: true,
        textBoxOpen: false,
      }),
    ).toBe(false)
  })

  it('canStart is true for normal annotation', () => {
    expect(
      canStartRmbErase({
        active: true,
        penetration: false,
        textBoxOpen: false,
      }),
    ).toBe(true)
  })

  it('begin then end restores previous tool', () => {
    const active = beginRmbErase('pen')
    expect(active).toEqual({ active: true, toolBefore: 'pen' })
    const out = endRmbErase(active)
    expect(out.wasActive).toBe(true)
    expect(out.restoreTool).toBe('pen')
    expect(out.next).toEqual(IDLE_RMB_ERASE)
  })

  it('begin while already eraser still restores eraser', () => {
    const out = endRmbErase(beginRmbErase('eraser'))
    expect(out.restoreTool).toBe('eraser')
  })

  it('end from idle is a no-op', () => {
    const out = endRmbErase(IDLE_RMB_ERASE)
    expect(out.wasActive).toBe(false)
    expect(out.restoreTool).toBeNull()
  })

  it('second begin after end works again', () => {
    const first = endRmbErase(beginRmbErase('highlighter'))
    expect(first.next).toEqual(IDLE_RMB_ERASE)
    const second = beginRmbErase('pen')
    expect(second.active).toBe(true)
    expect(endRmbErase(second).restoreTool).toBe('pen')
  })
})

import { describe, expect, it } from 'vitest'
import { EMPTY_TEXT_RMB_CLICK, TEXT_RMB_DOUBLE_MS, TEXT_RMB_DOUBLE_PX, noteTextRmbClick } from './textRmbDoubleClick'

describe('noteTextRmbClick', () => {
  it('arms on first click', () => {
    const { isDouble, next } = noteTextRmbClick(EMPTY_TEXT_RMB_CLICK, 10, 20, 1000)
    expect(isDouble).toBe(false)
    expect(next).toEqual({ lastAt: 1000, lastX: 10, lastY: 20 })
  })

  it('detects double click within window and distance', () => {
    const armed = noteTextRmbClick(EMPTY_TEXT_RMB_CLICK, 10, 20, 1000).next
    const { isDouble, next } = noteTextRmbClick(armed, 12, 21, 1000 + TEXT_RMB_DOUBLE_MS - 1)
    expect(isDouble).toBe(true)
    expect(next).toEqual(EMPTY_TEXT_RMB_CLICK)
  })

  it('does not count as double when too slow', () => {
    const armed = noteTextRmbClick(EMPTY_TEXT_RMB_CLICK, 10, 20, 1000).next
    const { isDouble } = noteTextRmbClick(armed, 10, 20, 1000 + TEXT_RMB_DOUBLE_MS + 1)
    expect(isDouble).toBe(false)
  })

  it('does not count as double when too far', () => {
    const armed = noteTextRmbClick(EMPTY_TEXT_RMB_CLICK, 10, 20, 1000).next
    const far = TEXT_RMB_DOUBLE_PX + 1
    const { isDouble } = noteTextRmbClick(armed, 10 + far, 20, 1100)
    expect(isDouble).toBe(false)
  })
})

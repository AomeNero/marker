import { describe, expect, it } from 'vitest'
import { resolveStoredToolState } from './toolState'

describe('resolveStoredToolState', () => {
  it('returns null for missing stored state', () => {
    expect(resolveStoredToolState(undefined)).toBeNull()
    expect(resolveStoredToolState(null)).toBeNull()
  })

  it('passes through a fully valid state', () => {
    const resolved = resolveStoredToolState({
      tool: 'laser',
      color: '#FFCC02',
      textOutline: { enabled: true, colorMode: 'fixed', color: '#FFFFFF', width: 3 },
    })
    expect(resolved).toEqual({
      tool: 'laser',
      color: '#FFCC02',
      textOutline: { enabled: true, colorMode: 'fixed', color: '#FFFFFF', width: 3 },
    })
  })

  it('keeps custom colors outside the palette', () => {
    expect(resolveStoredToolState({ color: '#123abc' })?.color).toBe('#123abc')
  })

  it('invalid fields resolve to null so callers keep defaults', () => {
    expect(resolveStoredToolState({ tool: 'magic', color: 'red' })).toEqual({
      tool: null,
      color: null,
      textOutline: null,
    })
  })

  it('rejects malformed hex colors', () => {
    expect(resolveStoredToolState({ color: '#FFF' })?.color).toBeNull()
    expect(resolveStoredToolState({ color: '#GGGGGG' })?.color).toBeNull()
    expect(resolveStoredToolState({ color: 'FFCC02' })?.color).toBeNull()
  })

  it('normalizes the text outline through normalizeTextOutline', () => {
    // Explicit outline object with no colorMode falls back to 'fixed' (existing semantics).
    const resolved = resolveStoredToolState({ textOutline: { enabled: true, width: 99 } })
    expect(resolved?.textOutline).toEqual({ enabled: true, colorMode: 'fixed', color: '#FFFFFF', width: 12 })
  })
})

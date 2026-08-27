import { describe, expect, it } from 'vitest'
import { isDragEnabled, resolveDragMode } from './dragMode'

describe('resolveDragMode', () => {
  it('prefers explicit dragMode off over legacy enableDragging', () => {
    expect(resolveDragMode({ dragMode: 'off', enableDragging: true })).toBe('off')
  })

  it('prefers dragMode when set', () => {
    expect(resolveDragMode({ dragMode: 'modifier' })).toBe('modifier')
  })

  it('migrates legacy enableDragging only', () => {
    expect(resolveDragMode({ enableDragging: true })).toBe('hover')
  })

  it('migrates legacy modifier flag', () => {
    expect(resolveDragMode({ enableDragging: true, dragRequiresModifier: true })).toBe('modifier')
  })

  it('defaults to off', () => {
    expect(resolveDragMode({})).toBe('off')
  })
})

describe('isDragEnabled', () => {
  it('returns false only for off', () => {
    expect(isDragEnabled('off')).toBe(false)
    expect(isDragEnabled('hover')).toBe(true)
    expect(isDragEnabled('modifier')).toBe(true)
  })
})

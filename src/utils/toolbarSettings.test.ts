import { describe, expect, it } from 'vitest'
import { isToolbarPinned, resolveToolbarVisibility } from './toolbarSettings'

describe('toolbarSettings', () => {
  it('defaults to space visibility', () => {
    expect(resolveToolbarVisibility()).toBe('space')
  })

  it('reads explicit toolbar visibility', () => {
    expect(resolveToolbarVisibility({ toolbarVisibility: 'always' })).toBe('always')
  })

  it('detects pinned toolbar', () => {
    expect(isToolbarPinned('always')).toBe(true)
    expect(isToolbarPinned('space')).toBe(false)
  })
})

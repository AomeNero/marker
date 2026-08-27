import { afterEach, describe, expect, it } from 'vitest'
import {
  TOOLBAR_PANEL_HEIGHT_COMPACT,
  getToolbarPanelHeight,
  rememberToolbarPanelHeight,
  resetToolbarPanelHeightCache,
} from './toolbarWindow'

describe('toolbar panel height cache', () => {
  afterEach(() => {
    resetToolbarPanelHeightCache()
  })

  it('defaults to measured compact height 234', () => {
    expect(TOOLBAR_PANEL_HEIGHT_COMPACT).toBe(234)
    expect(getToolbarPanelHeight()).toBe(234)
  })

  it('remembers measured heights for clamp/placement', () => {
    rememberToolbarPanelHeight(452.2)
    expect(getToolbarPanelHeight()).toBe(453)
  })
})

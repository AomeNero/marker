import { afterEach, describe, expect, it } from 'vitest'
import {
  TOOLBAR_PANEL_HEIGHT_COMPACT,
  getToolbarPanelHeight,
  rememberToolbarPanelHeight,
  resetToolbarPanelHeightCache,
  TOOLBAR_PANEL_WIDTH,
  getToolbarPanelWidth,
  rememberToolbarPanelWidth,
} from './toolbarWindow'

describe('toolbar panel height cache', () => {
  afterEach(() => {
    resetToolbarPanelHeightCache()
  })

  it('defaults to one-line bar height 46', () => {
    expect(TOOLBAR_PANEL_HEIGHT_COMPACT).toBe(46)
    expect(getToolbarPanelHeight()).toBe(46)
  })

  it('remembers measured heights for clamp/placement', () => {
    rememberToolbarPanelHeight(114.2)
    expect(getToolbarPanelHeight()).toBe(115)
  })
})

describe('toolbar panel width cache', () => {
  it('defaults to the bar width estimate and remembers measured widths', () => {
    expect(getToolbarPanelWidth()).toBe(TOOLBAR_PANEL_WIDTH)
    rememberToolbarPanelWidth(612.7)
    expect(getToolbarPanelWidth()).toBe(613)
  })
})

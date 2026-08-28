import { describe, expect, it } from 'vitest'
import {
  clampToolbarWindowPosition,
  adjustToolbarTopForHeightChange,
  migratePhysicalToLogical,
  overlayClientToScreenLogical,
  toolbarDockedScreenPosition,
  TOOLBAR_DOCK_MARGIN,
} from './toolbarPosition'

describe('overlayClientToScreenLogical', () => {
  it('adds monitor origin to overlay client coords (multi-monitor above primary)', () => {
    const monitor = { left: -283.3333333333333, top: -1440, width: 2560, height: 1440 }
    expect(overlayClientToScreenLogical(1310, 494, monitor)).toEqual({
      x: 1026.6666666666667,
      y: -946,
    })
  })
})

describe('toolbarDockedScreenPosition', () => {
  it('docks the bar at the monitor bottom-right with taskbar-safe margins', () => {
    const monitor = { left: -283.3333333333333, top: -1440, width: 2560, height: 1440 }
    const pos = toolbarDockedScreenPosition(580, 46, monitor)
    expect(pos.left).toBeCloseTo(-283.3333333333333 + 2560 - 580 - TOOLBAR_DOCK_MARGIN.right, 4)
    expect(pos.top).toBeCloseTo(-1440 + 1440 - 46 - TOOLBAR_DOCK_MARGIN.bottom, 4)
  })

  it('falls back to the viewport bottom-right when monitor bounds are unknown', () => {
    expect(toolbarDockedScreenPosition(580, 46, null, { width: 1920, height: 1080 })).toEqual({
      left: 1920 - 580 - TOOLBAR_DOCK_MARGIN.right,
      top: 1080 - 46 - TOOLBAR_DOCK_MARGIN.bottom,
    })
  })

  it('clamps inside the monitor when the bar is wider than the free space', () => {
    const monitor = { left: 0, top: 0, width: 640, height: 360 }
    const pos = toolbarDockedScreenPosition(700, 46, monitor)
    expect(pos.left).toBeGreaterThanOrEqual(0)
    expect(pos.top + 46).toBeLessThanOrEqual(360)
  })
})

describe('clampToolbarWindowPosition', () => {
  const monitor = { left: 0, top: 0, width: 1920, height: 1080 }

  it('keeps position inside monitor', () => {
    expect(clampToolbarWindowPosition(100, 200, 272, 400, monitor)).toEqual({ left: 100, top: 200 })
  })

  it('clamps panel dragged past right edge', () => {
    expect(clampToolbarWindowPosition(2000, 200, 272, 400, monitor).left).toBe(1920 - 272 - 8)
  })

  it('clamps panel dragged past bottom edge', () => {
    expect(clampToolbarWindowPosition(100, 900, 272, 400, monitor).top).toBe(1080 - 400 - 8)
  })
})

describe('adjustToolbarTopForHeightChange', () => {
  const monitor = { left: 0, top: 0, width: 1920, height: 1080 }
  const margin = 12
  const maxBottom = 1080 - margin

  it('grows upward when panel is on the bottom edge (更多)', () => {
    const oldH = 234
    const newH = 452
    const top = maxBottom - oldH
    const next = adjustToolbarTopForHeightChange(top, oldH, newH, monitor, margin)
    expect(next).toBe(maxBottom - newH)
    expect(next + newH).toBe(maxBottom)
    expect(next).toBeLessThan(top)
  })

  it('keeps top fixed when mid-screen expand still fits', () => {
    expect(adjustToolbarTopForHeightChange(200, 234, 452, monitor, margin)).toBe(200)
  })

  it('keeps bottom fixed when collapsing a bottom-anchored panel (收起)', () => {
    const oldH = 452
    const newH = 234
    const top = maxBottom - oldH
    const next = adjustToolbarTopForHeightChange(top, oldH, newH, monitor, margin)
    expect(next).toBe(maxBottom - newH)
    expect(next + newH).toBe(maxBottom)
  })

  it('ignores small outer-vs-content height noise', () => {
    expect(adjustToolbarTopForHeightChange(800, 234, 240, monitor, margin)).toBe(800)
  })
})

describe('migratePhysicalToLogical', () => {
  it('passes through logical coords unchanged', () => {
    const input = { left: 120, top: 80, coordSpace: 'logical' as const }
    expect(migratePhysicalToLogical(input, 1.25)).toEqual(input)
  })

  it('converts physical coords using scale factor', () => {
    expect(migratePhysicalToLogical({ left: 1250, top: 500, coordSpace: 'physical' }, 1.25)).toEqual({
      left: 1000,
      top: 400,
      coordSpace: 'logical',
    })
  })

  it('treats missing coordSpace as physical (legacy v1)', () => {
    expect(migratePhysicalToLogical({ left: 1500, top: 600 }, 1.5)).toEqual({
      left: 1000,
      top: 400,
      coordSpace: 'logical',
    })
  })

  it('falls back to scale 1 when scale factor is invalid', () => {
    expect(migratePhysicalToLogical({ left: 200, top: 100, coordSpace: 'physical' }, 0)).toEqual({
      left: 200,
      top: 100,
      coordSpace: 'logical',
    })
  })
})

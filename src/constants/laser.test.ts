import { describe, it, expect } from 'vitest'
import {
  LASER_DECAY_LENGTH,
  LASER_DECAY_MS,
  easeOut,
  isLaserTrailGone,
  laserSizeFromMapping,
  pruneAgedLaserPoints,
  smoothLaserPositions,
} from './laser'

describe('laserSizeFromMapping', () => {
  const now = 10_000

  it('keeps a fresh tip larger than an older point at the same index window', () => {
    const fresh = laserSizeFromMapping({ pressure: now, runningLength: 100, currentIndex: 50, totalLength: 51 }, now)
    const aged = laserSizeFromMapping(
      { pressure: now - LASER_DECAY_MS * 0.6, runningLength: 100, currentIndex: 50, totalLength: 51 },
      now,
    )
    expect(fresh).toBeGreaterThan(aged)
    expect(aged).toBeGreaterThan(0)
  })

  it('fades older timestamps before newer ones (first drawn first gone)', () => {
    const older = laserSizeFromMapping(
      { pressure: now - 900, runningLength: 20, currentIndex: 10, totalLength: 100 },
      now,
    )
    const newer = laserSizeFromMapping(
      { pressure: now - 100, runningLength: 180, currentIndex: 90, totalLength: 100 },
      now,
    )
    expect(newer).toBeGreaterThan(older)
  })

  it('is gone after time decay even near the tip index', () => {
    expect(
      laserSizeFromMapping(
        { pressure: now - LASER_DECAY_MS, runningLength: 100, currentIndex: 99, totalLength: 100 },
        now,
      ),
    ).toBe(0)
  })
})

describe('pruneAgedLaserPoints', () => {
  it('removes leading points that have fully aged out', () => {
    const now = 5000
    const pts = [{ t: 1000 }, { t: 2000 }, { t: 4500 }]
    const kept = pruneAgedLaserPoints(pts, now)
    expect(kept).toEqual([{ t: 4500 }])
  })
})

describe('smoothLaserPositions', () => {
  it('preserves timestamps while smoothing coordinates', () => {
    const out = smoothLaserPositions([
      { x: 0, y: 0, t: 100 },
      { x: 100, y: 0, t: 200 },
    ])
    expect(out[0]).toEqual([0, 0, 100])
    expect(out[1][0]).toBeGreaterThan(0)
    expect(out[1][0]).toBeLessThan(100)
    expect(out[1][2]).toBe(200)
  })
})

describe('easeOut', () => {
  it('maps 0→0 and 1→1 with a smooth curve', () => {
    expect(easeOut(0)).toBe(0)
    expect(easeOut(1)).toBe(1)
    expect(easeOut(0.5)).toBeGreaterThan(0.5)
  })
})

describe('isLaserTrailGone', () => {
  it('is true only when every point has aged out', () => {
    expect(isLaserTrailGone([{ t: 0 }, { t: 100 }], LASER_DECAY_MS + 100)).toBe(true)
    expect(isLaserTrailGone([{ t: 0 }, { t: LASER_DECAY_MS - 1 }], LASER_DECAY_MS)).toBe(false)
  })
})

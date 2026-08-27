/** How long each point stays fully visible after it was drawn. */
export const LASER_DECAY_MS = 1200

/**
 * Soft length cap in points from the tip (comet shape while drawing).
 * Time decay is what makes first-drawn segments disappear first.
 */
export const LASER_DECAY_LENGTH = 180

/**
 * Position-only smoothing (Excalidraw-style). Must NOT blend timestamps —
 * LaserPointer's built-in streamline lerps pressure/time and kills the chase.
 */
export const LASER_POSITION_STREAMLINE = 0.4

/** Quartic ease-out used by Excalidraw for laser size mapping. */
export function easeOut(k: number): number {
  return 1 - Math.pow(1 - k, 4)
}

export interface LaserSizeMappingInput {
  pressure: number
  runningLength: number
  currentIndex: number
  totalLength: number
}

export interface LaserInputPoint {
  x: number
  y: number
  t?: number
}

/**
 * Smooth x/y toward previous point; keep each point's original timestamp.
 */
export function smoothLaserPositions(
  points: LaserInputPoint[],
  streamline = LASER_POSITION_STREAMLINE,
): Array<[number, number, number]> {
  if (points.length === 0) return []
  const out: Array<[number, number, number]> = []
  let prevX = points[0].x
  let prevY = points[0].y
  const pull = 1 - streamline

  for (let i = 0; i < points.length; i++) {
    const p = points[i]
    const t = p.t ?? 0
    if (i === 0 || streamline <= 0) {
      prevX = p.x
      prevY = p.y
      out.push([prevX, prevY, t])
      continue
    }
    prevX = prevX + (p.x - prevX) * pull
    prevY = prevY + (p.y - prevY) * pull
    out.push([prevX, prevY, t])
  }
  return out
}

/**
 * Size factor: older points (earlier timestamps) shrink first.
 * Length window only soft-caps the comet; time creates the chase / stagger.
 */
export function laserSizeFromMapping(c: LaserSizeMappingInput, now: number): number {
  const age = now - c.pressure
  const timeFactor = Math.max(0, 1 - age / LASER_DECAY_MS)
  const pointsFromTip = Math.max(0, c.totalLength - c.currentIndex)
  const lengthFactor = (LASER_DECAY_LENGTH - Math.min(LASER_DECAY_LENGTH, pointsFromTip)) / LASER_DECAY_LENGTH
  // Prefer time so first-drawn → first-gone is obvious; length still tapers the tip.
  return easeOut(timeFactor) * (0.25 + 0.75 * easeOut(lengthFactor))
}

/** Drop points that have fully aged out (keeps the chase visible as the trail shortens). */
export function pruneAgedLaserPoints<T extends { t?: number }>(points: T[], now: number): T[] {
  let start = 0
  while (start < points.length && now - (points[start].t ?? 0) >= LASER_DECAY_MS) {
    start++
  }
  return start === 0 ? points : points.slice(start)
}

/** True once no points remain visible. */
export function isLaserTrailGone(points: { t?: number }[], now: number): boolean {
  if (points.length === 0) return true
  return points.every((p) => now - (p.t ?? 0) >= LASER_DECAY_MS)
}

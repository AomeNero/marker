import type { DragMode } from './dragMode'

export interface ElementDragGateOptions {
  dragMode: DragMode
  hasHoveredElement: boolean
  modifierDown: boolean
}

/** Whether a pointer down should start dragging an existing element (scheme A). */
export function canStartElementDrag(opts: ElementDragGateOptions): boolean {
  if (opts.dragMode === 'off' || !opts.hasHoveredElement) return false
  if (opts.dragMode === 'hover') return true
  return opts.modifierDown
}

/** Default hold time before a hover press becomes an element drag. */
export const HOVER_DRAG_ACTIVATE_MS = 200

export type HoverDragDecision = 'wait' | 'drag' | 'draw'

/**
 * Hover-drag mis-touch gate: a press that lands on an existing element only
 * becomes a drag after being held still for 200ms; moving away sooner counts
 * as drawing intent so strokes over old marks are never hijacked.
 */
export function resolveHoverDragGate(opts: {
  heldMs: number
  movedPx: number
  thresholdMs?: number
  thresholdPx?: number
}): HoverDragDecision {
  if (opts.movedPx > (opts.thresholdPx ?? 5)) return 'draw'
  if (opts.heldMs >= (opts.thresholdMs ?? HOVER_DRAG_ACTIVATE_MS)) return 'drag'
  return 'wait'
}

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

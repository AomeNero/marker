export type DragMode = 'off' | 'hover' | 'modifier'

export const DRAG_MODE_OPTIONS: DragMode[] = ['off', 'hover', 'modifier']

export function resolveDragMode(general?: {
  dragMode?: DragMode
  enableDragging?: boolean
  dragRequiresModifier?: boolean
}): DragMode {
  if (general?.dragMode !== undefined) return general.dragMode
  if (!general?.enableDragging) return 'off'
  if (general?.dragRequiresModifier) return 'modifier'
  return 'hover'
}

export function isDragEnabled(mode: DragMode): boolean {
  return mode !== 'off'
}

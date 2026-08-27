/** Timing / distance for double right-click to confirm text (issue #32). */
export const TEXT_RMB_DOUBLE_MS = 400
export const TEXT_RMB_DOUBLE_PX = 8

export type TextRmbClickState = {
  lastAt: number
  lastX: number
  lastY: number
}

export const EMPTY_TEXT_RMB_CLICK: TextRmbClickState = {
  lastAt: 0,
  lastX: 0,
  lastY: 0,
}

/** Returns whether this click completes a double-click, and the updated arming state. */
export function noteTextRmbClick(
  state: TextRmbClickState,
  clientX: number,
  clientY: number,
  now: number,
  windowMs = TEXT_RMB_DOUBLE_MS,
  maxDistPx = TEXT_RMB_DOUBLE_PX,
): { isDouble: boolean; next: TextRmbClickState } {
  const dx = clientX - state.lastX
  const dy = clientY - state.lastY
  const isDouble = state.lastAt > 0 && now - state.lastAt <= windowMs && dx * dx + dy * dy <= maxDistPx * maxDistPx

  if (isDouble) {
    return { isDouble: true, next: { ...EMPTY_TEXT_RMB_CLICK } }
  }
  return {
    isDouble: false,
    next: { lastAt: now, lastX: clientX, lastY: clientY },
  }
}

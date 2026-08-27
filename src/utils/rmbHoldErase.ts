export type RmbEraseGesture = {
  active: boolean
  toolBefore: string | null
}

export const IDLE_RMB_ERASE: RmbEraseGesture = {
  active: false,
  toolBefore: null,
}

export type RmbEraseEnd = {
  next: RmbEraseGesture
  restoreTool: string | null
  wasActive: boolean
}

export function canStartRmbErase(opts: { active: boolean; penetration: boolean; textBoxOpen: boolean }): boolean {
  return opts.active && !opts.penetration && !opts.textBoxOpen
}

export function beginRmbErase(currentTool: string): RmbEraseGesture {
  return { active: true, toolBefore: currentTool }
}

export function endRmbErase(gesture: RmbEraseGesture): RmbEraseEnd {
  if (!gesture.active) {
    return { next: IDLE_RMB_ERASE, restoreTool: null, wasActive: false }
  }
  return {
    next: IDLE_RMB_ERASE,
    restoreTool: gesture.toolBefore,
    wasActive: true,
  }
}

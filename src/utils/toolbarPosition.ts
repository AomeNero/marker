const STORAGE_KEY = 'marker-toolbar-position'
/** Legacy standalone window position (physical pixels). */
const WINDOW_STORAGE_KEY_V1 = 'marker-toolbar-window-position'
/** Standalone window position (logical pixels). */
const WINDOW_STORAGE_KEY_V2 = 'marker-toolbar-window-position-v2'

export type ToolbarCoordSpace = 'logical' | 'physical'

export interface ToolbarPosition {
  left: number
  top: number
  coordSpace?: ToolbarCoordSpace
}

export interface MonitorLogicalBounds {
  left: number
  top: number
  width: number
  height: number
}

/** Overlay client (CSS) coords → screen logical position for toolbar window placement. */
export function overlayClientToScreenLogical(
  clientX: number,
  clientY: number,
  monitor: MonitorLogicalBounds,
): { x: number; y: number } {
  return {
    x: monitor.left + clientX,
    y: monitor.top + clientY,
  }
}

/** Screen-logical top-left for a space-triggered toolbar popup centered on the pointer. */
export function toolbarPopupScreenPosition(
  clientX: number,
  clientY: number,
  panelWidth: number,
  panelHeight: number,
  monitor: MonitorLogicalBounds | null,
  fallbackViewport?: { width: number; height: number },
): { left: number; top: number } {
  if (monitor) {
    const anchor = overlayClientToScreenLogical(clientX, clientY, monitor)
    const left = anchor.x - panelWidth / 2
    const top = anchor.y - panelHeight / 2
    return clampToolbarWindowPosition(left, top, panelWidth, panelHeight, monitor, 12)
  }
  const vw = fallbackViewport?.width ?? 1920
  const vh = fallbackViewport?.height ?? 1080
  const margin = 12
  return {
    left: Math.max(margin, Math.min(clientX - panelWidth / 2, vw - panelWidth - margin)),
    top: Math.max(margin, Math.min(clientY - panelHeight / 2, vh - panelHeight - margin)),
  }
}

/** Keep a toolbar panel fully inside the overlay monitor (logical coordinates). */
export function clampToolbarWindowPosition(
  left: number,
  top: number,
  panelWidth: number,
  panelHeight: number,
  monitor: MonitorLogicalBounds,
  margin = 8,
): { left: number; top: number } {
  const minLeft = monitor.left + margin
  const minTop = monitor.top + margin
  const maxLeft = Math.max(minLeft, monitor.left + monitor.width - panelWidth - margin)
  const maxTop = Math.max(minTop, monitor.top + monitor.height - panelHeight - margin)
  return {
    left: Math.min(Math.max(left, minLeft), maxLeft),
    top: Math.min(Math.max(top, minTop), maxTop),
  }
}

/**
 * After compact↔expanded height change: grow/shrink upward when the panel sits on (or
 * would cross) the bottom edge; otherwise keep the top fixed so mid-screen panels expand down.
 *
 * `margin` should match the clamp used for placement (space popup uses 12; drag clamp uses 8).
 * Bottom-anchored detection allows a few px slack for float / margin mismatch.
 */
export function adjustToolbarTopForHeightChange(
  top: number,
  oldHeight: number,
  newHeight: number,
  monitor: MonitorLogicalBounds,
  margin = 8,
): number {
  if (oldHeight <= 0 || newHeight <= 0) return top
  const delta = newHeight - oldHeight
  // Ignore sub-pixel / outer-vs-content noise (esp. macOS WKWebView outerSize).
  if (Math.abs(delta) < 48) return top

  const minTop = monitor.top + margin
  const maxBottom = monitor.top + monitor.height - margin
  const oldBottom = top + oldHeight
  const wasBottomAnchored = oldBottom >= maxBottom - 16

  let nextTop = top
  if (delta > 0) {
    if (wasBottomAnchored || top + newHeight > maxBottom) {
      nextTop = oldBottom - newHeight
    }
  } else if (wasBottomAnchored) {
    nextTop = oldBottom - newHeight
  }

  const maxTop = Math.max(minTop, maxBottom - newHeight)
  return Math.min(Math.max(nextTop, minTop), maxTop)
}

function storageKey(forStandaloneWindow: boolean): string {
  return forStandaloneWindow ? WINDOW_STORAGE_KEY_V2 : STORAGE_KEY
}

function parseStoredPosition(raw: string): ToolbarPosition | null {
  try {
    const parsed = JSON.parse(raw) as ToolbarPosition
    if (typeof parsed.left !== 'number' || typeof parsed.top !== 'number') return null
    return parsed
  } catch {
    return null
  }
}

/** Convert legacy physical screen coords to logical (WebView / Tauri LogicalPosition). */
export function migratePhysicalToLogical(position: ToolbarPosition, scaleFactor: number): ToolbarPosition {
  if (position.coordSpace === 'logical') return position
  const scale = scaleFactor > 0 ? scaleFactor : 1
  return {
    left: position.left / scale,
    top: position.top / scale,
    coordSpace: 'logical',
  }
}

export function loadToolbarPosition(forStandaloneWindow = false): ToolbarPosition | null {
  try {
    if (forStandaloneWindow) {
      const v2Raw = localStorage.getItem(WINDOW_STORAGE_KEY_V2)
      if (v2Raw) {
        const parsed = parseStoredPosition(v2Raw)
        if (parsed) return { ...parsed, coordSpace: parsed.coordSpace ?? 'logical' }
      }

      const v1Raw = localStorage.getItem(WINDOW_STORAGE_KEY_V1)
      if (!v1Raw) return null
      const legacy = parseStoredPosition(v1Raw)
      if (!legacy) return null
      return { ...legacy, coordSpace: legacy.coordSpace ?? 'physical' }
    }

    const raw = localStorage.getItem(STORAGE_KEY)
    if (!raw) return null
    return parseStoredPosition(raw)
  } catch {
    return null
  }
}

export function saveToolbarPosition(left: number, top: number, forStandaloneWindow = false): void {
  try {
    if (forStandaloneWindow) {
      const payload: ToolbarPosition = { left, top, coordSpace: 'logical' }
      localStorage.setItem(WINDOW_STORAGE_KEY_V2, JSON.stringify(payload))
      localStorage.removeItem(WINDOW_STORAGE_KEY_V1)
      return
    }
    localStorage.setItem(STORAGE_KEY, JSON.stringify({ left, top }))
  } catch {
    // ignore quota / private mode
  }
}

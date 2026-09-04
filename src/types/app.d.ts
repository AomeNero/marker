export type DragMode = 'off' | 'hover' | 'modifier'

export interface AppConfig {
  shortcuts: {
    toggleDrawing: string
    clearDrawing: string
  }
  general: {
    dragMode?: DragMode
    /** @deprecated Read for migration only; use dragMode */
    enableDragging?: boolean
    /** @deprecated Read for migration only; use dragMode */
    dragRequiresModifier?: boolean
    locale?: string
    preserveDrawings: boolean
    whiteboardPreserveDrawings: boolean
    angleSnapStep?: 15 | 30 | 45
    toolbarVisibility?: ToolbarVisibility
    defaultEntryMode?: DefaultEntryMode
    eraserMode?: EraserMode
    penCursorStyle?: PenCursorStyle
    crosshairCursorStyle?: CrosshairCursorStyle
    strokeSmoothing?: StrokeSmoothing
    lineWidths?: {
      stroke: number
      highlighter: number
      eraser: number
      text: number
    }
    /** Last-used toolbar state; invalid fields fall back to defaults on load. */
    toolState?: {
      tool?: string
      color?: string
      textOutline?: {
        enabled?: boolean
        colorMode?: 'auto' | 'fixed'
        color?: string
        width?: number
      }
    }
    /** Five stroke-width presets (XS/S/M/L/XL); invalid arrays fall back to defaults. */
    widthPresets?: number[]
    autoStart?: boolean
    theme?: 'dark' | 'light' | 'system'
  }
}

export type ToolbarVisibility = 'space' | 'always'
export type DefaultEntryMode = 'screen' | 'whiteboard'
export type EraserMode = 'stroke' | 'object'
export type PenCursorStyle = 'pen' | 'dot'
export type CrosshairCursorStyle = 'crosshair' | 'dot'
export type StrokeSmoothing = 'off' | 'standard' | 'strong'

export interface SaveResult {
  ok: boolean
  failed?: string[]
}

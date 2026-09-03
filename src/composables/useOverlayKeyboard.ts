import type { Ref, ComputedRef } from 'vue'
import type { Tool } from './drawingTypes'
import { isMacOS } from '../utils/platform'
import { usesCrosshairCursor } from '../utils/crosshairCursor'
import { logActionEvent } from '../utils/diagnosticEvents'

const TOOL_KEYS: Tool[] = ['pen', 'highlighter', 'laser', 'arrow', 'rect', 'ellipse', 'line']

/** True while pointer is down for draw/drag — modifier keys serve drawing, not copy. */
let pointerGestureActive = false

/**
 * True only after a physical Meta/Control keydown while the pointer is idle.
 * Spurious macOS Mod+C after pen-up sends C with metaKey but no prior Meta keydown (issue #22).
 */
let copyModifierPhysicallyDown = false

export interface KeyboardContext {
  active: Ref<boolean>
  showToolbarPopup: Ref<boolean>
  toolbarPinned: Ref<boolean> | ComputedRef<boolean>
  showQuickColors: Ref<boolean>
  quickColorsPos: Ref<{ x: number; y: number }>
  textBoxPos: Ref<{ x: number; y: number } | null>
  currentTool: Ref<Tool>
  whiteboardMode: Ref<boolean>
  isDrawing: Ref<boolean>
  lastPointerX: () => number
  lastPointerY: () => number
  mousePos: Ref<{ x: number; y: number }>
}

export interface KeyboardActions {
  cycleColor: (direction: number) => void
  showToolTip: (tool: Tool) => void
  showStampTip?: () => void
  cycleStampKind?: () => void
  resetStampCounter?: () => void
  /** Press E again while eraser is selected: stroke ↔ object. */
  cycleEraserMode?: () => void
  /** Tip for eraser including current mode (first select). */
  showEraserTip?: () => void
  /** Press 1 again while pen is selected: pen icon ↔ dot. */
  cyclePenCursorStyle?: () => void
  /** Tip for pen including current cursor style (first select). */
  showPenTip?: () => void
  /** Re-press shape/laser key: crosshair ↔ dot. */
  cycleCrosshairCursorStyle?: () => void
  /** Tip for shape/laser including current crosshair style. */
  showCrosshairTip?: () => void
  undo: () => void
  redo: () => void
  removeSelected?: () => void
  hasSelection?: () => boolean
  clearSelection?: () => void
  exitDrawing: () => void
  enterWhiteboardMode: () => void
  exitWhiteboardMode: () => void
  copyScreen: () => void
  copyWhiteboard: () => void
  /** Mod+S / Mod+O / Mod+I — annotation file save / open / insert. */
  saveAnnotations?: () => void
  openAnnotations?: () => void
  insertAnnotations?: () => void
  toggleToolbarPopupVisible: () => void
  toggleInkVisible: () => void
  commitCurrentTextBox: (cancel?: boolean) => void
}

function modDown(e: KeyboardEvent): boolean {
  return e.ctrlKey || (isMacOS() && e.metaKey)
}

/** Mod+C without Shift — used when the toolbar window has OS focus (overlay never sees the chord). */
export function isCopyShortcut(e: KeyboardEvent): boolean {
  if (!modDown(e) || e.shiftKey) return false
  return e.key === 'c' || e.key === 'C'
}

export function trackCopyModifierKeyDown(e: KeyboardEvent): void {
  if ((e.key === 'Control' || e.key === 'Meta') && !pointerGestureActive) {
    copyModifierPhysicallyDown = true
  }
}

export function trackCopyModifierKeyUp(e: KeyboardEvent): void {
  if (e.key === 'Control' || e.key === 'Meta') {
    copyModifierPhysicallyDown = false
  }
}

export function resetCopyModifierState(): void {
  copyModifierPhysicallyDown = false
  pointerGestureActive = false
}

/** Pointer down: modifier keys are reserved for draw/drag until pointer up. */
export function invalidateCopyModifierForPointerInteraction(): void {
  copyModifierPhysicallyDown = false
  pointerGestureActive = true
}

/** Pointer up: gesture ends; copy modifier must be freshly pressed (no timer). */
export function markPointerInteractionEnded(): void {
  pointerGestureActive = false
}

/** For tests: read whether copy modifier is considered physically held. */
export function isCopyModifierPhysicallyDown(): boolean {
  return copyModifierPhysicallyDown
}

/** For tests: read pointer gesture state. */
export function isPointerGestureActive(): boolean {
  return pointerGestureActive
}

function shouldTriggerKeyboardCopy(e: KeyboardEvent, ctx: KeyboardContext): boolean {
  if (ctx.isDrawing.value || pointerGestureActive) return false
  if (!isCopyShortcut(e)) return false
  return copyModifierPhysicallyDown
}

function triggerKeyboardCopy(ctx: KeyboardContext, actions: KeyboardActions): void {
  if (ctx.whiteboardMode.value) {
    actions.copyWhiteboard()
  } else {
    actions.copyScreen()
  }
}

export function createKeyDownHandler(ctx: KeyboardContext, actions: KeyboardActions) {
  return function onKeyDown(e: KeyboardEvent) {
    if (!ctx.active.value) return

    trackCopyModifierKeyDown(e)

    // Prevent Alt key from triggering system menu focus
    if (e.key === 'Alt') {
      e.preventDefault()
    }

    // Quick color palette mode
    if (ctx.showQuickColors.value) {
      if (shouldTriggerKeyboardCopy(e, ctx)) {
        e.preventDefault()
        actions.copyScreen()
      } else if (e.key === 'Escape') {
        logActionEvent('quick colors closed', { reason: 'keyboard' })
        ctx.showQuickColors.value = false
      } else if (e.key === 'q' || e.key === 'Q') {
        logActionEvent('color cycled', { reason: 'keyboard', direction: -1, context: 'quick-colors' })
        actions.cycleColor(-1)
      } else if (e.key === 'r' || e.key === 'R') {
        logActionEvent('color cycled', { reason: 'keyboard', direction: 1, context: 'quick-colors' })
        actions.cycleColor(1)
      } else if (e.key === ' ') {
        e.preventDefault()
        logActionEvent('toolbar popup toggled', { reason: 'keyboard', context: 'quick-colors' })
        ctx.mousePos.value = { ...ctx.quickColorsPos.value }
        ctx.showQuickColors.value = false
        actions.toggleToolbarPopupVisible()
      }
      return
    }

    // Text box mode
    if (ctx.textBoxPos.value) {
      if (e.key === 'Escape') {
        logActionEvent('text box cancelled', { reason: 'keyboard' })
        actions.commitCurrentTextBox(true)
      }
      return
    }

    // Toolbar popup toggle (Space) — also recalls the bar after draw-hide in pinned mode
    if (e.key === ' ') {
      e.preventDefault()
      logActionEvent('toolbar popup toggled', { reason: 'keyboard' })
      ctx.mousePos.value = { x: ctx.lastPointerX(), y: ctx.lastPointerY() }
      actions.toggleToolbarPopupVisible()
      return
    }

    // Color cycling
    if (e.key === 'q' || e.key === 'Q') {
      logActionEvent('color cycled', { reason: 'keyboard', direction: -1 })
      actions.cycleColor(-1)
      return
    }
    if (e.key === 'r' || e.key === 'R') {
      logActionEvent('color cycled', { reason: 'keyboard', direction: 1 })
      actions.cycleColor(1)
      return
    }

    // Ink show/hide toggle — V (pairs with the toolbar eye button)
    if ((e.key === 'v' || e.key === 'V') && !modDown(e)) {
      if (ctx.isDrawing.value) return
      logActionEvent('ink visibility toggled', { reason: 'keyboard' })
      actions.toggleInkVisible()
      return
    }

    // Eraser — E selects; E again cycles stroke ↔ object mode
    if (e.key === 'e' || e.key === 'E') {
      if (ctx.isDrawing.value) return
      if (ctx.currentTool.value === 'eraser') {
        actions.cycleEraserMode?.()
        return
      }
      logActionEvent('tool selected', { reason: 'keyboard', tool: 'eraser' })
      ctx.currentTool.value = 'eraser'
      if (actions.showEraserTip) {
        actions.showEraserTip()
      } else {
        actions.showToolTip('eraser')
      }
      return
    }

    // Text tool
    if (e.key === 't' || e.key === 'T') {
      if (ctx.isDrawing.value) return
      logActionEvent('tool selected', { reason: 'keyboard', tool: 'text' })
      ctx.currentTool.value = 'text'
      actions.showToolTip('text')
      return
    }

    // Stamp tool — N selects; N again cycles number ↔ letter; Shift+N resets active counter.
    // Ignore Ctrl/Meta so we don't steal OS chords (⌘N / ⌘⇧N on macOS, Ctrl+N elsewhere).
    if ((e.key === 'n' || e.key === 'N') && !e.ctrlKey && !e.metaKey) {
      if (ctx.isDrawing.value) return
      if (e.shiftKey) {
        e.preventDefault()
        logActionEvent('stamp counter reset', { reason: 'keyboard' })
        ctx.currentTool.value = 'stamp'
        actions.resetStampCounter?.()
        return
      }
      if (ctx.currentTool.value === 'stamp') {
        logActionEvent('stamp kind cycled', { reason: 'keyboard' })
        actions.cycleStampKind?.()
      } else {
        logActionEvent('tool selected', { reason: 'keyboard', tool: 'stamp' })
        ctx.currentTool.value = 'stamp'
        actions.showStampTip?.()
      }
      return
    }

    // Tool switching (1-7). Pen/crosshair tools: re-press cycles style.
    // Ignore all tool changes while a stroke is active — action.tool is fixed at pointer-down.
    if (e.key >= '1' && e.key <= '7') {
      if (ctx.isDrawing.value) return
      const tool = TOOL_KEYS[parseInt(e.key) - 1]
      if (tool === 'pen' && ctx.currentTool.value === 'pen') {
        actions.cyclePenCursorStyle?.()
        return
      }
      if (usesCrosshairCursor(tool) && ctx.currentTool.value === tool) {
        actions.cycleCrosshairCursorStyle?.()
        return
      }
      logActionEvent('tool selected', { reason: 'keyboard', tool, key: e.key })
      ctx.currentTool.value = tool
      if (tool === 'pen' && actions.showPenTip) {
        actions.showPenTip()
      } else if (tool === 'eraser' && actions.showEraserTip) {
        actions.showEraserTip()
      } else if (usesCrosshairCursor(tool) && actions.showCrosshairTip) {
        actions.showCrosshairTip()
      } else {
        actions.showToolTip(tool)
      }
      return
    }

    // Whiteboard mode toggle
    if (e.key === 'w' || e.key === 'W') {
      if (ctx.whiteboardMode.value) {
        logActionEvent('whiteboard exit requested', { reason: 'keyboard' })
        actions.exitWhiteboardMode()
      } else {
        logActionEvent('whiteboard enter requested', { reason: 'keyboard' })
        actions.enterWhiteboardMode()
      }
      return
    }

    // Copy: idle pointer + physical Mod keydown before C (issue #22)
    if (shouldTriggerKeyboardCopy(e, ctx)) {
      e.preventDefault()
      triggerKeyboardCopy(ctx, actions)
      return
    }

    // Annotation files: Mod+S save, Mod+O open (replace), Mod+I insert.
    if (modDown(e) && (e.key === 's' || e.key === 'S')) {
      e.preventDefault()
      logActionEvent('annotations save requested', { reason: 'keyboard', shortcut: 'mod+s' })
      actions.saveAnnotations?.()
      return
    }
    if (modDown(e) && (e.key === 'o' || e.key === 'O')) {
      e.preventDefault()
      logActionEvent('annotations open requested', { reason: 'keyboard', shortcut: 'mod+o' })
      actions.openAnnotations?.()
      return
    }
    if (modDown(e) && (e.key === 'i' || e.key === 'I')) {
      e.preventDefault()
      logActionEvent('annotations insert requested', { reason: 'keyboard', shortcut: 'mod+i' })
      actions.insertAnnotations?.()
      return
    }

    // Don't process edit shortcuts when toolbar popup is open (space mode)
    if (ctx.showToolbarPopup.value && !ctx.toolbarPinned.value) return

    // Delete / Backspace removes the current selection (select tool or leftover selection).
    if (e.key === 'Delete' || e.key === 'Backspace') {
      if (actions.hasSelection?.()) {
        e.preventDefault()
        logActionEvent('selection deleted', { reason: 'keyboard', key: e.key })
        actions.removeSelected?.()
      }
      return
    }

    // Undo/Redo/Clear/Exit
    // macOS WKWebView often reports Cmd+Shift+Z as key 'z' (lowercase) even with shiftKey;
    // require !shiftKey on undo so Mod+Shift+Z never falls through as undo.
    const keyZ = e.key === 'z' || e.key === 'Z'
    const keyY = e.key === 'y' || e.key === 'Y'
    if (modDown(e) && e.shiftKey && keyZ) {
      e.preventDefault()
      logActionEvent('redo', { reason: 'keyboard', shortcut: 'mod+shift+z' })
      actions.redo()
    } else if (modDown(e) && !e.shiftKey && keyZ) {
      e.preventDefault()
      logActionEvent('undo', { reason: 'keyboard', shortcut: 'mod+z' })
      actions.undo()
    } else if (modDown(e) && !e.shiftKey && keyY) {
      e.preventDefault()
      logActionEvent('redo', { reason: 'keyboard', shortcut: 'mod+y' })
      actions.redo()
    } else if (e.key === 'Escape') {
      if (actions.hasSelection?.()) {
        logActionEvent('selection cleared', { reason: 'keyboard', shortcut: 'escape' })
        actions.clearSelection?.()
        return
      }
      if (ctx.whiteboardMode.value) {
        logActionEvent('whiteboard exit requested', { reason: 'keyboard', shortcut: 'escape' })
        actions.exitWhiteboardMode()
      } else {
        logActionEvent('exit drawing requested', { reason: 'keyboard', shortcut: 'escape' })
        actions.exitDrawing()
      }
    }
  }
}

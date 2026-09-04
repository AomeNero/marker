import type { TextOutlineStyle, Tool } from '../composables/drawingTypes'
import { normalizeTextOutline } from '../constants/textOutline'

/** Tool ids accepted in persisted tool state (mirrors the frontend `Tool` union). */
const VALID_TOOLS: readonly string[] = [
  'select',
  'pen',
  'highlighter',
  'laser',
  'arrow',
  'rect',
  'ellipse',
  'line',
  'eraser',
  'text',
  'stamp',
]

/** Shape of `general.toolState` as stored in the Rust config (all fields optional). */
export interface StoredToolState {
  tool?: string
  color?: string
  textOutline?: Partial<TextOutlineStyle>
}

export interface ResolvedToolState {
  tool: Tool | null
  color: string | null
  textOutline: TextOutlineStyle | null
}

function isHexColor(value: string): boolean {
  return /^#[0-9a-f]{6}$/i.test(value)
}

/**
 * Validate persisted tool state field-by-field. Invalid fields resolve to null
 * so the caller keeps its in-memory defaults; the text outline goes through the
 * same normalizeTextOutline path as runtime edits. Colors are validated by
 * format only — custom (non-palette) colors are preserved.
 */
export function resolveStoredToolState(stored?: StoredToolState | null): ResolvedToolState | null {
  if (!stored) return null
  return {
    tool: stored.tool && VALID_TOOLS.includes(stored.tool) ? (stored.tool as Tool) : null,
    color: stored.color && isHexColor(stored.color) ? stored.color : null,
    textOutline: stored.textOutline ? normalizeTextOutline(stored.textOutline) : null,
  }
}
